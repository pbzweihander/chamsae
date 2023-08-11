use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    config::CONFIG,
    entity::user,
    error::{Context, Error},
    format_err,
    state::State,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub ty: PersonType,
    pub id: ObjectId<user::Model>,
    pub preferred_username: String,
    pub name: Option<String>,
    pub inbox: Url,
    pub public_key: PublicKey,
}

#[derive(Debug)]
pub struct LocalPerson;

#[async_trait]
impl Object for LocalPerson {
    type DataType = State;
    type Kind = Person;
    type Error = Error;

    #[tracing::instrument(skip(_data))]
    async fn read_from_id(
        object_id: Url,
        _data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        if object_id == Self.id() {
            Ok(Some(Self))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(_data))]
    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = Self.id();
        Ok(Self::Kind {
            ty: Default::default(),
            id: id.clone().into(),
            preferred_username: CONFIG.user_handle.clone(),
            name: None,
            inbox: Self.inbox(),
            public_key: PublicKey {
                id: format!("{}#main-key", id),
                owner: id,
                public_key_pem: Self.public_key_pem().to_string(),
            },
        })
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(_data))]
    async fn from_json(
        _json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        Err(format_err!(NOT_IMPLEMENTED, "unimplemented"))
    }
}

impl Actor for LocalPerson {
    fn id(&self) -> Url {
        CONFIG.user_id.clone().unwrap()
    }

    fn public_key_pem(&self) -> &str {
        &CONFIG.user_public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        Some(CONFIG.user_private_key.clone())
    }

    fn inbox(&self) -> Url {
        CONFIG.inbox_url.clone().unwrap()
    }
}
