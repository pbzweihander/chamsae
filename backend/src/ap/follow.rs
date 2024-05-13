use activitypub_federation::{
    activity_queue::queue_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::{AcceptType, FollowType, RejectType},
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use derivative::Derivative;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    config::CONFIG,
    entity::{follow, follower, user},
    error::{Context, Error},
    format_err,
    queue::{Event, Notification, NotificationType},
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    #[serde(rename = "type")]
    pub ty: FollowType,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
    pub id: Option<Url>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub object: Url,
}

impl Follow {
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let object: ObjectId<user::Model> = self.object.clone().into();
        let inbox = object.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        queue_activity(&with_context, &me, vec![inbox], data).await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        self.id.as_ref().unwrap_or(&self.actor)
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.actor, self.id()).context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let follower = follower::Model::from_json(self.clone(), data).await?;

        let accept = FollowAccept {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson::id(),
            object: self,
        };
        accept.send(data).await?;

        let event = Event::Notification(Notification::new(NotificationType::CreateFollower {
            user_id: follower.from_id.into(),
        }));
        event.send(&*data.db).await?;

        Ok(())
    }
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct FollowAccept {
    #[serde(rename = "type")]
    pub ty: AcceptType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    pub object: Follow,
}

impl FollowAccept {
    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let actor: ObjectId<user::Model> = self.object.actor.clone().into();
        let inbox = actor.dereference(data).await?.inbox;
        let inbox = Url::parse(&inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        queue_activity(&with_context, &me, vec![inbox], data).await?;
        Ok(())
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
        let follow_id: ObjectId<follow::Model> = self.object.id().clone().into();
        let follow = follow_id.dereference(data).await?;
        let mut follow_activemodel: follow::ActiveModel = follow.into();
        follow_activemodel.accepted = ActiveValue::Set(true);
        let follow = follow_activemodel
            .update(&*data.db)
            .await
            .context_internal_server_error("failed to update database")?;

        let event = Event::Notification(Notification::new(NotificationType::AcceptFollow {
            user_id: follow.to_id.into(),
        }));
        event.send(&*data.db).await?;

        Ok(())
    }
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct FollowReject {
    #[serde(rename = "type")]
    pub ty: RejectType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    pub object: Follow,
}

impl FollowReject {
    pub fn new(user_id: Ulid, user_uri: Url) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson::id(),
            object: Follow {
                ty: Default::default(),
                id: Some(
                    Url::parse(&format!("https://{}/follower/{}", CONFIG.domain, user_id))
                        .context_internal_server_error("failed to construct URL")?,
                ),
                actor: user_uri,
                object: LocalPerson::id(),
            },
        })
    }

    pub async fn send(self, data: &Data<State>, inbox: Url) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        queue_activity(&with_context, &me, vec![inbox], data).await?;
        Ok(())
    }
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

            let event = Event::Notification(Notification::new(NotificationType::RejectFollow {
                user_id: follow_id.into(),
            }));
            event.send(&*data.db).await?;

            Ok(())
        } else {
            Err(format_err!(NOT_FOUND, "follow not found"))
        }
    }
}
