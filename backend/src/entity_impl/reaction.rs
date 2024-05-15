use std::str::FromStr;

use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, protocol::verification::verify_domains_match,
    traits::Object,
};
use async_trait::async_trait;
use mime::Mime;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
    TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{
        like::Like,
        person::LocalPerson,
        tag::{Emoji, EmojiIcon, Tag},
    },
    config::CONFIG,
    entity::{post, reaction, user},
    error::{Context, Error},
    queue::{Event, Update},
    state::State,
};

impl reaction::Model {
    pub fn ap_id_from_id(id: Ulid) -> Result<Url, Error> {
        Url::parse(&format!("https://{}/like/{}", CONFIG.public_domain, id))
            .context_internal_server_error("failed to construct follow URL ID")
    }

    pub fn ap_id(&self) -> Result<Url, Error> {
        Self::ap_id_from_id(self.id.into())
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

        let user_id: Url = if self.user_id.is_some() {
            self.find_related(user::Entity)
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
            LocalPerson::id()
        };

        let post_id: Url = self
            .find_related(post::Entity)
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
            content: Some(self.content),
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
            id: Ulid::new().into(),
            user_id: Some(user.id),
            post_id: post.id,
            content: json.content.unwrap_or_else(|| "❤️".to_string()),
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
            .into_tuple::<uuid::Uuid>()
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?;

        let this = if let Some(id) = existing_id {
            let this_activemodel: reaction::ActiveModel = this.into();
            let mut this_activemodel = this_activemodel.reset_all();
            this_activemodel.id = ActiveValue::Unchanged(id);
            this_activemodel
                .update(&tx)
                .await
                .context_internal_server_error("failed to update database")?
        } else {
            let this_activemodel: reaction::ActiveModel = this.into();
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

    #[tracing::instrument(skip(data))]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let post_id = self.post_id;

        ModelTrait::delete(self, &*data.db)
            .await
            .context_internal_server_error("failed to delete from database")?;

        let event = Event::Update(Update::DeleteReaction {
            post_id: post_id.into(),
        });
        event.send(&*data.db).await?;

        Ok(())
    }
}
