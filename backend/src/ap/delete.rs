use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    kinds::{activity::DeleteType, object::TombstoneType},
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::ActivityHandler,
};
use async_trait::async_trait;
use derivative::Derivative;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::post,
    error::{Context, Error},
    format_err,
    state::State,
};

use super::{generate_object_id, person::LocalPerson};

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    #[serde(rename = "type")]
    pub ty: TombstoneType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
    #[serde(rename = "type")]
    pub ty: DeleteType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    pub object: Tombstone,
}

impl Delete {
    pub fn new(id: Url) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson::id(),
            object: Tombstone {
                ty: Default::default(),
                id,
            },
        })
    }

    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>, inboxes: Vec<Url>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &me, inboxes, data).await
    }
}

#[async_trait]
impl ActivityHandler for Delete {
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
        verify_domains_match(&self.object.id, &self.id)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let post_id = self.object.id;
        let res = post::Entity::delete_many()
            .filter(post::Column::Uri.eq(post_id.as_str()))
            .exec(&*data.db)
            .await
            .context_internal_server_error("failed to delete from database")?;
        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(format_err!(NOT_FOUND, "post not found"))
        }
    }
}
