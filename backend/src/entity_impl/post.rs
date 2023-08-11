use activitypub_federation::{
    config::Data,
    kinds::public,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{note::Note, person::LocalPerson},
    entity::{post, sea_orm_active_enums, user},
    error::{Context, Error},
    format_err,
    state::State,
};

#[async_trait]
impl Object for post::Model {
    type DataType = State;
    type Kind = Note;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        post::Entity::find()
            .filter(post::Column::Uri.eq(object_id.to_string()))
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let user_id = if let Some(user_id) = &self.user_id {
            let user = user::Entity::find_by_id(user_id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .ok_or_else(|| format_err!(INTERNAL_SERVER_ERROR, "failed to find user"))?;

            Url::parse(&user.uri).context_internal_server_error("malformed user URI")?
        } else {
            LocalPerson.id()
        };
        let id = Url::parse(&self.uri).context_internal_server_error("malformed post URI")?;
        let in_reply_to_id = if let Some(reply_id) = self.reply_id {
            let reply_post = post::Entity::find_by_id(reply_id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .ok_or_else(|| {
                    format_err!(INTERNAL_SERVER_ERROR, "failed to find reply target post")
                })?;

            Some(Url::parse(&reply_post.uri).context_internal_server_error("malformed post URI")?)
        } else {
            None
        };
        Ok(Self::Kind {
            ty: Default::default(),
            id: id.into(),
            attributed_to: user_id.into(),
            to: vec![public()],
            content: self.text,
            in_reply_to: in_reply_to_id.map(Into::into),
            tag: vec![],
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
        let user = json.attributed_to.dereference(data).await?;
        let this = Self {
            id: Ulid::new().to_string(),
            created_at: Utc::now().fixed_offset(),
            reply_id: None,
            text: json.content,
            title: None,
            user_id: Some(user.id),
            visibility: sea_orm_active_enums::Visibility::Public,
            uri: json.id.inner().to_string(),
        };

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_id = post::Entity::find()
            .filter(post::Column::Uri.eq(json.id.inner().to_string()))
            .select_only()
            .column(post::Column::Id)
            .into_tuple::<String>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            Self { id, ..this }
        } else {
            let this_activemodel: post::ActiveModel = this.into();
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

    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;
        let existing_count = post::Entity::find_by_id(&self.id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_count == 0 {
            return Ok(());
        }
        ModelTrait::delete(self, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;
        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;
        Ok(())
    }
}
