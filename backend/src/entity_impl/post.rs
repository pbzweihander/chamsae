use activitypub_federation::{
    config::Data,
    kinds::public,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use async_trait::async_trait;
use chrono::Utc;
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use url::Url;
use uuid::Uuid;

use crate::{
    ap::{
        note::{Attachment, Note},
        person::LocalPerson,
    },
    entity::{local_file, post, remote_file, sea_orm_active_enums, user},
    error::{Context, Error},
    format_err,
    state::State,
};

#[async_trait]
impl Object for post::Model {
    type DataType = State;
    type Kind = Note;
    type Error = Error;

    #[tracing::instrument(skip(data))]
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

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let user_id = if let Some(user_id) = &self.user_id {
            let user = user::Entity::find_by_id(*user_id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("failed to find user")?;

            Url::parse(&user.uri).context_internal_server_error("malformed user URI")?
        } else {
            LocalPerson.id()
        };

        let id = Url::parse(&self.uri).context_internal_server_error("malformed post URI")?;

        let in_reply_to_id = if let Some(reply_id) = &self.reply_id {
            let reply_post = post::Entity::find_by_id(*reply_id)
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("failed to find reply target post")?;

            Some(Url::parse(&reply_post.uri).context_internal_server_error("malformed post URI")?)
        } else {
            None
        };

        let remote_files = self
            .find_related(remote_file::Entity)
            .order_by_asc(remote_file::Column::Order)
            .all(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;

        let local_files = self
            .find_related(local_file::Entity)
            .order_by_asc(local_file::Column::Order)
            .all(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;

        let attachment = remote_files
            .into_iter()
            .filter_map(|file| {
                Some(Attachment {
                    ty: Default::default(),
                    media_type: file.media_type.parse().ok()?,
                    url: file.url.parse().ok()?,
                    name: file.alt,
                })
            })
            .chain(local_files.into_iter().filter_map(|file| {
                Some(Attachment {
                    ty: Default::default(),
                    media_type: file.media_type.parse().ok()?,
                    url: file.url.parse().ok()?,
                    name: file.alt,
                })
            }))
            .collect::<Vec<_>>();

        Ok(Self::Kind {
            ty: Default::default(),
            id: id.into(),
            attributed_to: user_id.into(),
            to: vec![public()],
            summary: self.title,
            content: self.text,
            in_reply_to: in_reply_to_id.map(Into::into),
            attachment,
            sensitive: self.is_sensitive,
            tag: vec![],
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
        let user = json.attributed_to.dereference(data).await?;
        let this = Self {
            id: Uuid::new_v4(),
            created_at: Utc::now().fixed_offset(),
            reply_id: None,
            text: json.content,
            title: json.summary,
            user_id: Some(user.id),
            visibility: sea_orm_active_enums::Visibility::Public,
            is_sensitive: json.sensitive,
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
            .into_tuple::<Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            Self { id, ..this }
        } else {
            let this_activemodel: post::ActiveModel = this.into();
            this_activemodel
                .insert(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?
        };

        let remote_files = json
            .attachment
            .into_iter()
            .enumerate()
            .map(|(idx, attachment)| remote_file::ActiveModel {
                post_id: ActiveValue::Set(this.id),
                order: ActiveValue::Set(idx as i16),
                media_type: ActiveValue::Set(attachment.media_type.to_string()),
                url: ActiveValue::Set(attachment.url.to_string()),
                alt: ActiveValue::Set(attachment.name),
            })
            .collect::<Vec<_>>();

        if !remote_files.is_empty() {
            remote_file::Entity::insert_many(remote_files)
                .on_conflict(
                    OnConflict::columns([remote_file::Column::PostId, remote_file::Column::Order])
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;
        }

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        Ok(this)
    }

    #[tracing::instrument(skip(data))]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_count = post::Entity::find_by_id(self.id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_count == 0 {
            return Err(format_err!(NOT_FOUND, "post not found"));
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
