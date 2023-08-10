use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::CreateType, actor::PersonType, link::MentionType, object::NoteType},
    protocol::{helpers::deserialize_one_or_many, public_key::PublicKey},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{post, user},
    error::Error,
    handler::AppState,
};

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct Mention {
    #[serde(rename = "type")]
    pub ty: MentionType,
    pub href: Url,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub ty: NoteType,
    pub id: ObjectId<post::Model>,
    pub attributed_to: ObjectId<user::Model>,
    pub to: Vec<Url>,
    pub content: String,
    pub in_reply_to: Option<ObjectId<post::Model>>,
    pub tag: Vec<Mention>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    #[serde(rename = "type")]
    pub ty: CreateType,
    pub id: Url,
    pub actor: ObjectId<user::Model>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: Note,
}

#[async_trait]
impl ActivityHandler for CreateNote {
    type DataType = AppState;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        post::Model::verify(&self.object, &self.id, data).await
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        post::Model::from_json(self.object, data).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivity {
    CreateNote(CreateNote),
}
