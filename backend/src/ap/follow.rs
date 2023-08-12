use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::{AcceptType, FollowType, RejectType},
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follow, follower, user},
    error::{Context, Error},
    format_err,
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    #[serde(rename = "type")]
    pub ty: FollowType,
    pub id: Url,
    pub actor: Url,
    pub object: Url,
}

impl Follow {
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let object: ObjectId<user::Model> = self.object.clone().into();
        let inbox = object.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalPerson, vec![inbox], data).await
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.actor, &self.id).context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        follower::Model::from_json(self.clone(), data).await?;
        let accept = FollowAccept {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson.id(),
            object: self,
        };
        accept.send(data).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowAccept {
    #[serde(rename = "type")]
    pub ty: AcceptType,
    pub id: Url,
    pub actor: Url,
    pub object: Follow,
}

impl FollowAccept {
    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let actor: ObjectId<user::Model> = self.object.actor.clone().into();
        let inbox = actor.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalPerson, vec![inbox], data).await
    }
}

#[async_trait]
impl ActivityHandler for FollowAccept {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.actor, &self.object.object)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let follow_id: ObjectId<follow::Model> = self.object.id.into();
        let follow = follow_id.dereference(data).await?;
        let mut follow_activemodel: follow::ActiveModel = follow.into();
        follow_activemodel.accepted = ActiveValue::Set(true);
        follow_activemodel
            .update(&*data.db)
            .await
            .context_internal_server_error("failed to update database")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowReject {
    #[serde(rename = "type")]
    pub ty: RejectType,
    pub id: Url,
    pub actor: Url,
    pub object: Follow,
}

#[async_trait]
impl ActivityHandler for FollowReject {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, &self.actor).context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let follow_user_id = self.object.object;

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let follow_id = user::Entity::find()
            .filter(user::Column::Uri.eq(follow_user_id.as_str()))
            .inner_join(follow::Entity)
            .select_only()
            .column(follow::Column::ToId)
            .into_tuple::<uuid::Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        if let Some(follow_id) = follow_id {
            follow::Entity::delete_by_id(follow_id)
                .exec(&tx)
                .await
                .context_internal_server_error("failed to delete from database")?;

            tx.commit()
                .await
                .context_internal_server_error("failed to commit database transaction")?;

            Ok(())
        } else {
            Err(format_err!(NOT_FOUND, "follow not found"))
        }
    }
}
