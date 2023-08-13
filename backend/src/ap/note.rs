use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{
        activity::CreateType,
        object::{DocumentType, NoteType},
    },
    protocol::context::WithContext,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use derivative::Derivative;
use mime::Mime;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follow, post, user},
    error::{Context, Error},
    state::State,
};

use super::{generate_object_id, person::LocalPerson, tag::Tag};

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    #[serde(rename = "type")]
    pub ty: DocumentType,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub url: Url,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub ty: NoteType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: ObjectId<post::Model>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub attributed_to: ObjectId<user::Model>,
    pub published: DateTime<FixedOffset>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub to: Vec<Url>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub cc: Vec<Url>,
    #[serde(default)]
    pub summary: Option<String>,
    pub content: String,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
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

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    #[serde(rename = "type")]
    pub ty: CreateType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: ObjectId<user::Model>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub to: Vec<Url>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub cc: Vec<Url>,
    pub object: Note,
}

impl CreateNote {
    pub fn new(note: Note) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: note.attributed_to.clone(),
            to: note.to.clone(),
            cc: note.cc.clone(),
            object: note,
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
