use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::LikeType,
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{post, reaction},
    error::{Context, Error},
    state::State,
};

use super::tag::Tag;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    #[serde(rename = "type")]
    pub ty: LikeType,
    pub id: ObjectId<reaction::Model>,
    pub actor: Url,
    pub object: ObjectId<post::Model>,
    pub content: String,
    #[serde(default)]
    pub tag: Vec<Tag>,
}

#[async_trait]
impl ActivityHandler for Like {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        self.id.inner()
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.actor, self.id.inner())
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        reaction::Model::from_json(self, data).await?;
        Ok(())
    }
}
