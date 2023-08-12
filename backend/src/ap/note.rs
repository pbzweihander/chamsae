use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{
        activity::CreateType,
        object::{DocumentType, NoteType},
        public,
    },
    protocol::context::WithContext,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use mime::Mime;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follow, post, user},
    error::{Context, Error},
    state::State,
    util::get_follower_inboxes,
};

use super::{generate_object_id, person::LocalPerson, tag::Tag};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    #[serde(rename = "type")]
    pub ty: DocumentType,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub url: Url,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub ty: NoteType,
    pub id: ObjectId<post::Model>,
    pub attributed_to: ObjectId<user::Model>,
    pub to: Vec<Url>,
    #[serde(default)]
    pub summary: Option<String>,
    pub content: String,
    pub in_reply_to: Option<ObjectId<post::Model>>,
    #[serde(default)]
    pub attachment: Vec<Attachment>,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub tag: Vec<Tag>,
}

impl Note {
    pub fn into_create(self) -> Result<CreateNote, Error> {
        CreateNote::new(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    #[serde(rename = "type")]
    pub ty: CreateType,
    pub id: Url,
    pub actor: ObjectId<user::Model>,
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

    #[tracing::instrument(skip(data))]
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
        let existing_following_user_count = user::Entity::find()
            .filter(user::Column::Uri.eq(self.actor.inner().to_string()))
            .inner_join(follow::Entity)
            .count(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;
        // if zero, the note was from not following user. ignore.
        if existing_following_user_count != 0 {
            post::Model::from_json(self.object, data).await?;
        }
        Ok(())
    }
}
