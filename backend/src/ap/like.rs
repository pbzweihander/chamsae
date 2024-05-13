use activitypub_federation::{
    activity_queue::queue_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::LikeType,
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use derivative::Derivative;
use sea_orm::{ColumnTrait, ModelTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{post, reaction, user},
    error::{Context, Error},
    queue::{Event, Notification, NotificationType, Update},
    state::State,
};

use super::{person::LocalPerson, tag::Tag};

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    #[serde(rename = "type")]
    pub ty: LikeType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: ObjectId<reaction::Model>,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub object: ObjectId<post::Model>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tag: Vec<Tag>,
}

impl Like {
    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let post = self.object.dereference(data).await?;
        let user = post
            .find_related(user::Entity)
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("user not found")?;
        let inbox =
            Url::parse(&user.inbox).context_internal_server_error("malformed user inbox URL")?;
        let with_context = WithContext::new_default(self);
        queue_activity(&with_context, &me, vec![inbox], data).await?;
        Ok(())
    }
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
        let reaction = reaction::Model::from_json(self, data).await?;

        let event = Event::Update(Update::CreateReaction {
            post_id: reaction.post_id.into(),
        });
        event.send(&*data.db).await?;

        let local_person_reacted_count = reaction
            .find_related(post::Entity)
            .filter(post::Column::UserId.is_null())
            .count(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;
        if local_person_reacted_count > 0 {
            let event = Event::Notification(Notification::new(NotificationType::Reacted {
                post_id: reaction.post_id.into(),
                reaction_id: reaction.id.into(),
            }));
            event.send(&*data.db).await?;
        }

        Ok(())
    }
}
