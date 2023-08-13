use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FlagType,
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::ActivityHandler,
};
use async_trait::async_trait;
use derivative::Derivative;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::person::LocalPerson,
    entity::{report, user},
    error::{Context, Error},
    state::State,
};

use super::generate_object_id;

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    #[serde(rename = "type")]
    pub ty: FlagType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub object: Url,
    pub content: String,
}

impl Flag {
    pub fn new(target_user_uri: Url, content: String) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson::id(),
            object: target_user_uri,
            content,
        })
    }

    pub async fn send(self, data: &Data<State>, inbox: Url) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &me, vec![inbox], data).await
    }
}

#[async_trait]
impl ActivityHandler for Flag {
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
        let from_user_id: ObjectId<user::Model> = self.actor.into();
        let from_user = from_user_id.dereference(data).await?;

        let report_activemodel = report::ActiveModel {
            id: ActiveValue::Set(Ulid::new().into()),
            from_user_id: ActiveValue::Set(from_user.id),
            content: ActiveValue::Set(self.content),
        };
        report_activemodel
            .insert(&*data.db)
            .await
            .context_internal_server_error("failed to insert to database")?;

        Ok(())
    }
}
