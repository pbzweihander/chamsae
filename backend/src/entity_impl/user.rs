use activitypub_federation::{
    config::Data,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use async_trait::async_trait;
use chrono::{NaiveDateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QuerySelect,
    TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::person::{ActorType, Person, PersonImage},
    entity::user,
    error::{Context, Error},
    state::State,
};

#[async_trait]
impl Object for user::Model {
    type DataType = State;
    type Kind = Person;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_fetched_at.naive_utc())
    }

    #[tracing::instrument(skip(data))]
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

    #[tracing::instrument(skip(_data))]
    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = Url::parse(&self.uri).context_internal_server_error("malformed user URI")?;
        Ok(Self::Kind {
            ty: if self.is_bot {
                ActorType::Service
            } else {
                ActorType::Person
            },
            id: id.clone().into(),
            preferred_username: self.handle,
            name: self.name,
            summary: self.description,
            icon: self.avatar_url.and_then(|url| {
                Some(PersonImage {
                    ty: Default::default(),
                    url: url.parse().ok()?,
                })
            }),
            image: self.banner_url.and_then(|url| {
                Some(PersonImage {
                    ty: Default::default(),
                    url: url.parse().ok()?,
                })
            }),
            inbox: self
                .inbox
                .parse()
                .context_internal_server_error("malformed user inbox URL")?,
            shared_inbox: self
                .shared_inbox
                .map(|inbox| Url::parse(&inbox))
                .transpose()
                .context_internal_server_error("malformed user shared inbox URL")?,
            public_key: PublicKey {
                id: format!("{}#main-key", id),
                owner: id,
                public_key_pem: self.public_key,
            },
            manually_approves_followers: self.manually_approves_followers,
        })
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)
            .context_bad_request("failed to verify domain")
    }

    #[tracing::instrument(skip(data))]
    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let this = Self {
            id: Ulid::new().into(),
            last_fetched_at: Utc::now().fixed_offset(),
            handle: json.preferred_username,
            name: json.name,
            description: json.summary,
            host: json
                .id
                .inner()
                .host()
                .context_bad_request("invalid host")?
                .to_string(),
            inbox: json.inbox.to_string(),
            shared_inbox: json.shared_inbox.as_ref().map(Url::to_string),
            public_key: json.public_key.public_key_pem,
            uri: json.id.inner().to_string(),
            avatar_url: json.icon.map(|image| image.url.to_string()),
            banner_url: json.image.map(|image| image.url.to_string()),
            manually_approves_followers: json.manually_approves_followers,
            is_bot: match json.ty {
                ActorType::Person => false,
                ActorType::Service | ActorType::Application => true,
            },
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
            .into_tuple::<uuid::Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            let this_activemodel: user::ActiveModel = this.into();
            let mut this_activemodel = this_activemodel.reset_all();
            this_activemodel.id = ActiveValue::Unchanged(id);
            this_activemodel
                .update(&tx)
                .await
                .context_internal_server_error("failed to update database")?
        } else {
            let this_activemodel: user::ActiveModel = this.into();
            this_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?
        };

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

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
