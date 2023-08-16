use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::verification::verify_domains_match,
    traits::{ActivityHandler, Object},
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QuerySelect, TransactionTrait,
};
use url::Url;

use crate::{
    ap::{follow::Follow, person::LocalPerson},
    entity::{follower, user},
    error::{Context, Error},
    queue::{Event, Notification, NotificationType},
    state::State,
};

#[async_trait]
impl Object for follower::Model {
    type DataType = State;
    type Kind = Follow;
    type Error = Error;

    #[tracing::instrument(skip(data))]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        follower::Entity::find()
            .filter(follower::Column::Uri.eq(object_id.to_string()))
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")
    }

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let from_user_id = self
            .find_related(user::Entity)
            .select_only()
            .column(user::Column::Uri)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("failed to find target user")?;
        let from_user_id =
            Url::parse(&from_user_id).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: Default::default(),
            id: Some(
                Url::parse(&self.uri).context_internal_server_error("malformed follower URI")?,
            ),
            actor: from_user_id,
            object: LocalPerson::id(),
        })
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let uri = json.id().clone();
        let actor: ObjectId<user::Model> = json.actor.into();
        let from_user = actor.dereference(data).await?;
        let this = Self {
            from_id: from_user.id,
            uri: uri.to_string(),
        };

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_count = follower::Entity::find()
            .filter(
                follower::Column::Uri
                    .eq(uri.as_str())
                    .or(follower::Column::FromId.eq(from_user.id)),
            )
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if existing_count == 0 {
            let this_activemodel: follower::ActiveModel = this.into();
            let this = this_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;
            tx.commit()
                .await
                .context_internal_server_error("failed to commit database transaction")?;
            this
        } else {
            this
        };

        Ok(this)
    }

    #[tracing::instrument(skip(data))]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let user_id = self.from_id;

        ModelTrait::delete(self, &*data.db)
            .await
            .context_internal_server_error("failed to delete from database")?;

        let event = Event::Notification(Notification::new(NotificationType::DeleteFollower {
            user_id: user_id.into(),
        }));
        event.send(&*data.db).await?;

        Ok(())
    }
}
