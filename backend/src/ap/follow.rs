use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::{AcceptType, FollowType},
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follow, follower, user},
    error::{Context, Error},
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Clone, Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowAccept {
    #[serde(rename = "type")]
    pub ty: AcceptType,
    pub id: Url,
    pub actor: Url,
    pub object: Follow,
}

impl FollowAccept {
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let actor: ObjectId<user::Model> = self.object.actor.clone().into();
        let inbox = actor.dereference(data).await?.inbox;
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

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.actor, &self.id).context_bad_request("failed to verify domain")
    }

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

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.object.id, &self.id)
            .context_bad_request("failed to verify domain")
    }

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
