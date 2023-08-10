use activitypub_federation::{
    config::Data, protocol::verification::verify_domains_match, traits::Object,
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::Follower,
    config::CONFIG,
    entity::{follower, user},
    error::{Context, Error},
    format_err,
    state::State,
};

#[async_trait]
impl Object for follower::Model {
    type DataType = State;
    type Kind = Follower;
    type Error = Error;

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

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let from_user_id = user::Entity::find_by_id(self.from_id)
            .select_only()
            .column(user::Column::Uri)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .ok_or_else(|| format_err!(INTERNAL_SERVER_ERROR, "failed to find target user"))?;
        let from_user_id =
            Url::parse(&from_user_id).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: Default::default(),
            id: Url::parse(&self.uri)
                .context_internal_server_error("malformed follower URI")?
                .into(),
            actor: from_user_id.into(),
            object: CONFIG.user_id.clone().unwrap(),
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let from_user = json.actor.dereference(data).await?;
        let this = Self {
            id: Ulid::new().to_string(),
            from_id: from_user.id.clone(),
            uri: json.id.inner().to_string(),
        };

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_id = follower::Entity::find()
            .filter(
                follower::Column::Uri
                    .eq(json.id.inner().to_string())
                    .or(follower::Column::FromId.eq(&from_user.id)),
            )
            .select_only()
            .column(follower::Column::Id)
            .into_tuple::<String>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            Self { id, ..this }
        } else {
            let this_activemodel: follower::ActiveModel = this.into();
            let this = this_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;
            tx.commit()
                .await
                .context_internal_server_error("failed to commit database transaction")?;
            this
        };

        Ok(this)
    }
}
