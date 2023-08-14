use activitypub_federation::{config::Data, traits::ActivityHandler};
use async_trait::async_trait;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{error::Error, format_err, state::State};

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct OtherActivity {
    #[serde(rename = "type")]
    pub ty: String,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

#[async_trait]
impl ActivityHandler for OtherActivity {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        &self.actor
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[tracing::instrument(skip(_data))]
    async fn receive(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        if let Ok(activity) = serde_json::to_string(&self) {
            tracing::warn!("unimplemented activity received: {}", activity);
        } else {
            tracing::warn!(activity = ?self, "unimplemented activity received");
        }
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented activity"))
    }
}
