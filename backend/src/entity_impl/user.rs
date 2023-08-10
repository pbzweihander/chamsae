use activitypub_federation::{
    config::Data,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::Person,
    entity::user,
    error::{Context, Error},
    format_err,
    handler::AppState,
};

#[async_trait]
impl Object for user::Model {
    type DataType = AppState;
    type Kind = Person;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_fetched_at.naive_utc())
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        user::Entity::find()
            .filter(user::Column::Uri.eq(object_id.to_string()))
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = Url::parse(&self.uri).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: Default::default(),
            id: id.clone().into(),
            preferred_username: self.handle,
            name: self.name,
            inbox: self
                .inbox
                .parse()
                .context_internal_server_error("failed to parse inbox URL")?,
            public_key: PublicKey {
                id: format!("{}#main-key", id),
                owner: id,
                public_key_pem: self.public_key,
            },
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
        let this = Self {
            id: Ulid::new().to_string(),
            created_at: Utc::now().fixed_offset(),
            last_fetched_at: Utc::now().fixed_offset(),
            handle: json.preferred_username,
            name: json.name,
            host: json
                .id
                .inner()
                .host()
                .ok_or_else(|| format_err!(BAD_REQUEST, "invalid host"))?
                .to_string(),
            inbox: json.inbox.to_string(),
            public_key: json.public_key.public_key_pem,
            uri: json.id.inner().to_string(),
        };

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_id = user::Entity::find()
            .filter(user::Column::Uri.eq(json.id.inner().to_string()))
            .select_only()
            .column(user::Column::Id)
            .into_tuple::<String>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            Self { id, ..this }
        } else {
            let this_activemodel: user::ActiveModel = this.into();
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

impl Actor for user::Model {
    fn id(&self) -> Url {
        self.uri.parse().expect("malformed user URI")
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        None
    }

    fn inbox(&self) -> Url {
        self.inbox.parse().expect("malformed user inbox URL")
    }
}
