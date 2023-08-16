use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::AnnounceType,
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use derivative::Derivative;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::post,
    error::{Context, Error},
    queue::{Event, Notification, NotificationType, Update},
    state::State,
};

use super::NoteOrAnnounce;

#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Announce {
    #[serde(rename = "type")]
    pub ty: AnnounceType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: ObjectId<post::Model>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    pub published: DateTime<FixedOffset>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub to: Vec<Url>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_vec_display"))]
    #[serde(default)]
    pub cc: Vec<Url>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub object: ObjectId<post::Model>,
}

#[async_trait]
impl ActivityHandler for Announce {
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
        verify_domains_match(self.id.inner(), &self.actor)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let post = post::Model::from_json(NoteOrAnnounce::Announce(self), data).await?;

        let event = Event::Update(Update::CreatePost {
            post_id: post.id.into(),
        });
        event.send(&*data.db).await?;

        if let Some(repost_id) = post.repost_id {
            let local_person_reposted_count = post::Entity::find_by_id(repost_id)
                .filter(post::Column::UserId.is_null())
                .count(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?;
            if local_person_reposted_count > 0 {
                let event = Event::Notification(Notification::new(NotificationType::Reposted {
                    user_id: post
                        .user_id
                        .context_internal_server_error("malformed user ID")?
                        .into(),
                    post_id: repost_id.into(),
                }));
                event.send(&*data.db).await?;
            }
        }

        Ok(())
    }
}
