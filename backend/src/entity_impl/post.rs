use activitypub_federation::{
    config::Data,
    kinds::public,
    protocol::verification::verify_domains_match,
    traits::{Actor, Object},
};
use async_trait::async_trait;
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{
        note::{Attachment, Note},
        person::LocalPerson,
        tag::{Emoji, EmojiIcon, Mention, Tag},
    },
    config::CONFIG,
    entity::{local_file, mention, post, post_emoji, remote_file, sea_orm_active_enums, user},
    error::{Context, Error},
    format_err,
    state::State,
};

impl post::Model {
    pub fn ap_id_from_id(id: Ulid) -> Result<Url, Error> {
        Url::parse(&format!("https://{}/ap/note/{}", CONFIG.domain, id))
            .context_internal_server_error("failed to construct follow URL ID")
    }

    pub fn ap_id(&self) -> Result<Url, Error> {
        Self::ap_id_from_id(self.id.into())
    }
}

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
        let user_id = if self.user_id.is_some() {
            let user = self
                .find_related(user::Entity)
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

        let mentions = self
            .find_related(mention::Entity)
            .all(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;
        let emojis = self
            .find_related(post_emoji::Entity)
            .all(&*data.db)
            .await
            .context_internal_server_error("failed to query database")?;

        let mention_user_uris = mentions
            .iter()
            .filter_map(|mention| Url::parse(&mention.user_uri).ok())
            .collect::<Vec<_>>();

        let to = match self.visibility {
            sea_orm_active_enums::Visibility::Public => {
                vec![public()]
            }
            sea_orm_active_enums::Visibility::Home
            | sea_orm_active_enums::Visibility::Followers => {
                vec![LocalPerson.followers()?]
            }
            sea_orm_active_enums::Visibility::DirectMessage => mention_user_uris.clone(),
        };
        let cc = match self.visibility {
            sea_orm_active_enums::Visibility::Public => {
                let mut cc = mention_user_uris.clone();
                cc.push(LocalPerson.followers()?);
                cc
            }
            sea_orm_active_enums::Visibility::Home => {
                let mut cc = mention_user_uris.clone();
                cc.push(public());
                cc
            }
            sea_orm_active_enums::Visibility::Followers => mention_user_uris,
            sea_orm_active_enums::Visibility::DirectMessage => Vec::new(),
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

        let tag = mentions
            .into_iter()
            .filter_map(|mention| {
                Some(Tag::Mention(Mention {
                    ty: Default::default(),
                    href: mention.user_uri.parse().ok()?,
                    name: mention.name,
                }))
            })
            .chain(emojis.into_iter().filter_map(|emoji| {
                Some(Tag::Emoji(Emoji {
                    ty: Default::default(),
                    id: emoji.uri.parse().ok()?,
                    name: emoji.name,
                    icon: EmojiIcon {
                        ty: Default::default(),
                        media_type: emoji.media_type.parse().ok()?,
                        url: emoji.image_url.parse().ok()?,
                    },
                }))
            }))
            .collect::<Vec<_>>();

        Ok(Self::Kind {
            ty: Default::default(),
            id: id.into(),
            attributed_to: user_id.into(),
            published: self.created_at,
            to,
            cc,
            summary: self.title,
            content: self.text,
            in_reply_to: in_reply_to_id.map(Into::into),
            attachment,
            sensitive: self.is_sensitive,
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
        let user = json.attributed_to.dereference(data).await?;

        let visibility = if json.to.contains(&public()) {
            sea_orm_active_enums::Visibility::Public
        } else if json.cc.contains(&public()) {
            sea_orm_active_enums::Visibility::Home
        } else if json
            .to
            .iter()
            .any(|to| to.to_string().ends_with("/followers"))
        {
            sea_orm_active_enums::Visibility::Followers
        } else {
            sea_orm_active_enums::Visibility::DirectMessage
        };

        let this = Self {
            id: Ulid::new().into(),
            created_at: json.published,
            reply_id: None,
            text: json.content,
            title: json.summary,
            user_id: Some(user.id),
            visibility,
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
            .into_tuple::<uuid::Uuid>()
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

        let mentions = json
            .tag
            .iter()
            .filter_map(|tag| {
                if let Tag::Mention(mention) = tag {
                    Some(mention::ActiveModel {
                        post_id: ActiveValue::Set(this.id),
                        user_uri: ActiveValue::Set(mention.href.to_string()),
                        name: ActiveValue::Set(mention.name.clone()),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !mentions.is_empty() {
            mention::Entity::insert_many(mentions)
                .on_conflict(
                    OnConflict::columns([mention::Column::PostId, mention::Column::UserUri])
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&tx)
                .await
                .context_internal_server_error("failed to insert to database")?;
        }

        let emojis = json
            .tag
            .iter()
            .filter_map(|tag| {
                if let Tag::Emoji(emoji) = tag {
                    Some(post_emoji::ActiveModel {
                        post_id: ActiveValue::Set(this.id),
                        name: ActiveValue::Set(emoji.name.clone()),
                        uri: ActiveValue::Set(emoji.id.to_string()),
                        media_type: ActiveValue::Set(emoji.icon.media_type.to_string()),
                        image_url: ActiveValue::Set(emoji.icon.url.to_string()),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !emojis.is_empty() {
            post_emoji::Entity::insert_many(emojis)
                .on_conflict(
                    OnConflict::columns([post_emoji::Column::PostId, post_emoji::Column::Name])
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
