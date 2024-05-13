use activitypub_federation::{
    activity_queue::queue_activity,
    config::Data,
    kinds::activity::UndoType,
    protocol::{context::WithContext, verification::verify_domains_match},
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use derivative::Derivative;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    entity::{follower, user},
    error::{Context, Error},
    format_err,
    queue::{Event, Notification, NotificationType},
    state::State,
};

use super::{follow::Follow, generate_object_id, like::Like, person::LocalPerson};

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Undo<T> {
    #[serde(rename = "type")]
    pub ty: UndoType,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub id: Url,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    pub actor: Url,
    pub object: T,
}

impl<T> Undo<T> {
    pub fn new(object: T) -> Result<Self, Error> {
        Ok(Self {
            ty: Default::default(),
            id: generate_object_id()?,
            actor: LocalPerson::id(),
            object,
        })
    }
}

impl<T> Undo<T>
where
    T: std::fmt::Debug + Serialize + Send + Sync + 'static,
    Undo<T>: ActivityHandler<DataType = State, Error = Error>,
{
    #[tracing::instrument(skip(data))]
    pub async fn send(self, data: &Data<State>, inboxes: Vec<Url>) -> Result<(), Error> {
        let me = LocalPerson::get(&*data.db).await?;
        let with_context = WithContext::new_default(self);
        queue_activity(&with_context, &me, inboxes, data).await?;
        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Undo<Follow> {
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
        verify_domains_match(self.object.id(), &self.id)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let follower_id = user::Entity::find()
            .filter(user::Column::Uri.eq(self.object.actor.as_str()))
            .select_only()
            .column(user::Column::Id)
            .into_tuple::<uuid::Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?
            .context_not_found("follower not found")?;

        let existing_count = follower::Entity::find_by_id(follower_id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_count == 0 {
            return Err(format_err!(NOT_FOUND, "follower not found"));
        }

        follower::Entity::delete_by_id(follower_id)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        let event = Event::Notification(Notification::new(NotificationType::DeleteFollower {
            user_id: follower_id.into(),
        }));
        event.send(&*data.db).await?;

        Ok(())
    }
}

#[async_trait]
impl ActivityHandler for Undo<Like> {
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
        verify_domains_match(self.object.id(), &self.id)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let res = self.object.id.dereference_local(data).await;
        match res {
            Ok(object) => {
                object.delete(data).await?;
                Ok(())
            }
            Err(error) => {
                if let Some(activitypub_federation::error::Error::NotFound) =
                    error
                        .inner
                        .downcast_ref::<activitypub_federation::error::Error>()
                {
                    Err(format_err!(NOT_FOUND, "not found"))
                } else {
                    Err(error)
                }
            }
        }
    }
}
