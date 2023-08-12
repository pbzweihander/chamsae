use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, actor::PersonType, object::ImageType},
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
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
pub struct PersonImage {
    #[serde(rename = "type")]
    pub ty: ImageType,
    pub url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub ty: PersonType,
    pub id: ObjectId<user::Model>,
    pub preferred_username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon: Option<PersonImage>,
    #[serde(default)]
    pub image: Option<PersonImage>,
    pub inbox: Url,
    #[serde(default)]
    pub shared_inbox: Option<Url>,
    pub public_key: PublicKey,
}

#[derive(Debug)]
pub struct LocalPerson;

impl LocalPerson {
    pub fn followers(&self) -> Result<Url, Error> {
        Url::parse(&format!("{}/followers", self.id()))
            .context_internal_server_error("failed to construct followers URL")
    }
}

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
            icon: None,
            image: None,
            inbox: Self.inbox(),
            shared_inbox: Some(Self.inbox()),
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonUpdate {
    #[serde(rename = "type")]
    pub ty: UpdateType,
    pub id: Url,
    pub actor: ObjectId<user::Model>,
    #[serde(default)]
    pub to: Vec<Url>,
    pub object: Person,
}

#[async_trait]
impl ActivityHandler for PersonUpdate {
    type DataType = State;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.object.id.inner())
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        user::Model::from_json(self.object, data).await?;
        Ok(())
    }
}
