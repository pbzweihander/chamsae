use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{
        activity::CreateType,
        object::{DocumentType, NoteType},
    },
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use derivative::Derivative;
use mime::Mime;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{mention, post},
    error::{Context, Error},
    queue::{Notification, NotificationType},
    state::State,
};

use super::{generate_object_id, person::LocalPerson, tag::Tag, NoteOrAnnounce};

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
    pub attributed_to: Url,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
    #[serde(default)]
    pub quote_url: Option<ObjectId<post::Model>>,
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

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNote {
    #[serde(rename = "type")]
    pub ty: CreateType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
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
}

#[async_trait]
impl ActivityHandler for CreateNote {
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
        verify_domains_match(&self.id, &self.actor).context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let post = post::Model::from_json(NoteOrAnnounce::Note(self.object), data).await?;

        let notification = Notification::new(NotificationType::CreatePost {
            post_id: post.id.into(),
        });
        notification.send(&*data.db, &mut data.redis()).await?;

        let local_person_mentioned_count = post
            .find_related(mention::Entity)
            .filter(mention::Column::UserUri.eq(LocalPerson::id().as_str()))
            .count(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;
        if local_person_mentioned_count > 0 {
            let notification = Notification::new(NotificationType::Mentioned {
                post_id: post.id.into(),
            });
            notification.send(&*data.db, &mut data.redis()).await?;
        }

        if let Some(repost_id) = post.repost_id {
            let local_person_reposted_count = post::Entity::find_by_id(repost_id)
                .filter(post::Column::UserId.is_null())
                .count(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?;
            if local_person_reposted_count > 0 {
                let notification = Notification::new(NotificationType::Quoted {
                    post_id: post.id.into(),
                });
                notification.send(&*data.db, &mut data.redis()).await?;
            }
        }

        Ok(())
    }
}
