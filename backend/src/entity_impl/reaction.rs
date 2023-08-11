use std::str::FromStr;

use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use async_trait::async_trait;
use mime::Mime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QuerySelect, TransactionTrait,
};
use url::Url;
use uuid::Uuid;

use crate::{
    ap::{
        like::Like,
        person::LocalPerson,
        tag::{Emoji, EmojiIcon, Tag},
    },
    config::CONFIG,
    entity::{post, reaction, user},
    error::{Context, Error},
    format_err,
    state::State,
};

impl reaction::Model {
    pub fn ap_id(&self) -> Result<Url, Error> {
        Url::parse(&format!("https://{}/ap/like/{}", CONFIG.domain, self.id))
            .context_internal_server_error("failed to construct follow URL ID")
    }
}

#[async_trait]
impl Object for reaction::Model {
    type DataType = State;
    type Kind = Like;
    type Error = Error;

    #[tracing::instrument(skip(data))]
    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        reaction::Entity::find()
            .filter(reaction::Column::Uri.eq(object_id.to_string()))
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")
    }

    #[tracing::instrument(skip(data))]
    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let id = self.ap_id()?;

        let user_id: Url = if let Some(user_id) = self.user_id {
            user::Entity::find_by_id(user_id)
                .select_only()
                .column(user::Column::Uri)
                .into_tuple::<String>()
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("user not found")?
                .parse()
                .context_internal_server_error("malformed user URI")?
        } else {
            LocalPerson.id()
        };

        let post_id: Url = post::Entity::find_by_id(self.post_id)
            .select_only()
            .column(post::Column::Uri)
            .into_tuple::<String>()
            .one(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?
            .context_internal_server_error("post not found")?
            .parse()
            .context_internal_server_error("malformed post URI")?;

        let tag = if let (Some(emoji_id), Some(emoji_media_type), Some(emoji_image_url)) =
            (self.emoji_uri, self.emoji_media_type, self.emoji_image_url)
        {
            let emoji_id =
                Url::parse(&emoji_id).context_internal_server_error("malformed emoji URI")?;
            let emoji_media_type = Mime::from_str(&emoji_media_type)
                .context_internal_server_error("malformed emoji mime")?;
            let emoji_image_url = Url::parse(&emoji_image_url)
                .context_internal_server_error("malformed emoji image URL")?;

            vec![Tag::Emoji(Emoji {
                ty: Default::default(),
                id: emoji_id,
                name: self.content.clone(),
                icon: EmojiIcon {
                    ty: Default::default(),
                    media_type: emoji_media_type,
                    url: emoji_image_url,
                },
            })]
        } else {
            Vec::new()
        };

        Ok(Self::Kind {
            ty: Default::default(),
            id: id.into(),
            actor: user_id,
            object: post_id.into(),
            content: self.content,
            tag,
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
        let user_id: ObjectId<user::Model> = json.actor.into();
        let user = user_id.dereference(data).await?;
        let post = json.object.dereference(data).await?;

        let (emoji_uri, emoji_media_type, emoji_image_url) =
            if let Some(Tag::Emoji(emoji)) = json.tag.first() {
                (
                    Some(emoji.id.to_string()),
                    Some(emoji.icon.media_type.to_string()),
                    Some(emoji.icon.url.to_string()),
                )
            } else {
                (None, None, None)
            };

        let this = Self {
            id: Uuid::new_v4(),
            user_id: Some(user.id),
            post_id: post.id,
            content: json.content,
            uri: json.id.inner().to_string(),
            emoji_uri,
            emoji_media_type,
            emoji_image_url,
        };

        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;

        let existing_id = reaction::Entity::find()
            .filter(reaction::Column::Uri.eq(json.id.inner().to_string()))
            .select_only()
            .column(reaction::Column::Id)
            .into_tuple::<Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            Self { id, ..this }
        } else {
            let this_activemodel: reaction::ActiveModel = this.into();
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

    #[tracing::instrument(skip(data))]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let tx = data
            .db
            .begin()
            .await
            .context_internal_server_error("failed to begin database transaction")?;
        let existing_count = reaction::Entity::find_by_id(self.id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to query database")?;
        if existing_count == 0 {
            return Err(format_err!(NOT_FOUND, "reaction not found"));
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
