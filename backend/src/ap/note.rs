use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::CreateType, link::MentionType, object::NoteType, public},
    protocol::{context::WithContext, helpers::deserialize_one_or_many},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{post, user},
    error::Error,
    state::State,
    util::get_follower_inboxes,
};

use super::{generate_object_id, person::LocalPerson};

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

impl Note {
    pub fn into_create(self) -> Result<CreateNote, Error> {
        CreateNote::new(self)
    }
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

impl CreateNote {
    pub fn new(note: Note) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: note.attributed_to.clone(),
            to: vec![public()],
            object: note,
        })
    }

    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let inboxes = get_follower_inboxes(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        send_activity(with_context, &LocalPerson, inboxes, data).await
    }
}

#[async_trait]
impl ActivityHandler for CreateNote {
    type DataType = State;
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
