use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::public,
    protocol::verification::verify_domains_match, traits::Object,
};
use async_trait::async_trait;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::{
        announce::Announce,
        note::{Attachment, Note, Source},
        person::LocalPerson,
        tag::{Emoji, EmojiIcon, Hashtag, Mention, Tag},
        NoteOrAnnounce,
    },
    config::CONFIG,
    entity::{
        hashtag, local_file, mention, post, post_emoji, remote_file, sea_orm_active_enums, user,
    },
    error::{Context, Error},
    queue::{Event, Update},
    state::State,
};

fn calculate_visibility(to: &[Url], cc: &[Url]) -> sea_orm_active_enums::Visibility {
    if to.contains(&public()) {
        sea_orm_active_enums::Visibility::Public
    } else if cc.contains(&public()) {
        sea_orm_active_enums::Visibility::Home
    } else if to.iter().any(|to| to.as_str().ends_with("/followers")) {
        sea_orm_active_enums::Visibility::Followers
    } else {
        sea_orm_active_enums::Visibility::DirectMessage
    }
}

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
    type Kind = NoteOrAnnounce;
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
        let user_uri = if self.user_id.is_some() {
            let user_uri = self
                .find_related(user::Entity)
                .select_only()
                .column(user::Column::Uri)
                .into_tuple::<String>()
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("failed to find user")?;

            Url::parse(&user_uri).context_internal_server_error("malformed user URI")?
        } else {
            LocalPerson::id()
        };

        let uri = Url::parse(&self.uri).context_internal_server_error("malformed post URI")?;

        let quote_uri = if let Some(repost_id) = self.repost_id {
            let repost_uri = post::Entity::find_by_id(repost_id)
                .select_only()
                .column(post::Column::Uri)
                .into_tuple::<String>()
                .one(&*data.db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("failed to find repost target post")?;
            let repost_uri =
                Url::parse(&repost_uri).context_internal_server_error("malformed post URI")?;

            if self.text.is_empty() {
                // Repost
                let to = match self.visibility {
                    sea_orm_active_enums::Visibility::Public => {
                        vec![public()]
                    }
                    sea_orm_active_enums::Visibility::Home
                    | sea_orm_active_enums::Visibility::Followers => {
                        vec![LocalPerson::followers()?]
                    }
                    sea_orm_active_enums::Visibility::DirectMessage => Vec::new(),
                };
                let cc = match self.visibility {
                    sea_orm_active_enums::Visibility::Public => {
                        vec![LocalPerson::followers()?]
                    }
                    sea_orm_active_enums::Visibility::Home => {
                        vec![public()]
                    }
                    sea_orm_active_enums::Visibility::Followers => Vec::new(),
                    sea_orm_active_enums::Visibility::DirectMessage => Vec::new(),
                };

                let announce = Announce {
                    ty: Default::default(),
                    id: uri.into(),
                    actor: user_uri,
                    published: self.created_at,
                    to,
                    cc,
                    object: repost_uri.into(),
                };
                return Ok(NoteOrAnnounce::Announce(announce));
            } else {
                // Quote
                Some(repost_uri)
            }
        } else {
            // Post
            None
        };

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
        let hashtags = self
            .find_related(hashtag::Entity)
            .select_only()
            .column(hashtag::Column::Name)
            .into_tuple::<String>()
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
                vec![LocalPerson::followers()?]
            }
            sea_orm_active_enums::Visibility::DirectMessage => mention_user_uris.clone(),
        };
        let cc = match self.visibility {
            sea_orm_active_enums::Visibility::Public => {
                let mut cc = mention_user_uris.clone();
                cc.push(LocalPerson::followers()?);
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
            .chain(hashtags.into_iter().map(|hashtag| {
                Tag::Hashtag(Hashtag {
                    ty: Default::default(),
                    name: hashtag,
                })
            }))
            .collect::<Vec<_>>();

        Ok(NoteOrAnnounce::Note(Note {
            ty: Default::default(),
            id: uri.into(),
            attributed_to: user_uri,
            quote_url: quote_uri.map(Into::into),
            published: self.created_at,
            to,
            cc,
            summary: self.title,
            content: self.text,
            source: Some(Source {
                content: self.source_content,
                media_type: self.source_media_type,
            }),
            in_reply_to: in_reply_to_id.map(Into::into),
            attachment,
            sensitive: self.is_sensitive,
            tag,
        }))
    }

    #[tracing::instrument(skip(_data))]
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        match json {
            NoteOrAnnounce::Note(json) => verify_domains_match(json.id.inner(), expected_domain)
                .context_bad_request("failed to verify domain"),
            NoteOrAnnounce::Announce(json) => {
                verify_domains_match(json.id.inner(), expected_domain)
                    .context_bad_request("failed to verify domain")
            }
        }
    }

    #[tracing::instrument(skip(data))]
    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        match json {
            NoteOrAnnounce::Note(json) => {
                let user_uri: ObjectId<user::Model> = json.attributed_to.into();
                let user = user_uri.dereference(data).await?;

                let repost_id = if let Some(repost_uri) = json.quote_url {
                    let repost_post = repost_uri.dereference(data).await?;
                    Some(repost_post.id)
                } else {
                    None
                };

                let visibility = calculate_visibility(&json.to, &json.cc);

                let mut this_activemodel = post::ActiveModel {
                    id: ActiveValue::Set(Ulid::new().into()),
                    created_at: ActiveValue::Set(json.published),
                    reply_id: ActiveValue::Set(None),
                    repost_id: ActiveValue::Set(repost_id),
                    text: ActiveValue::Set(json.content),
                    title: ActiveValue::Set(json.summary),
                    user_id: ActiveValue::Set(Some(user.id)),
                    visibility: ActiveValue::Set(visibility),
                    is_sensitive: ActiveValue::Set(json.sensitive),
                    uri: ActiveValue::Set(json.id.inner().to_string()),
                    source_content: ActiveValue::Set(
                        json.source
                            .as_ref()
                            .and_then(|source| source.content.clone()),
                    ),
                    source_media_type: ActiveValue::Set(
                        json.source
                            .as_ref()
                            .and_then(|source| source.media_type.clone()),
                    ),
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
                    this_activemodel.id = ActiveValue::Unchanged(id);
                    this_activemodel
                        .update(&tx)
                        .await
                        .context_internal_server_error("failed to update database")?
                } else {
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
                            OnConflict::columns([
                                remote_file::Column::PostId,
                                remote_file::Column::Order,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(&tx)
                        .await
                        .context_internal_server_error("failed to insert to database")?;
                }

                let mut mentions = Vec::new();
                let mut emojis = Vec::new();
                let mut hashtags = Vec::new();

                for tag in json.tag {
                    match tag {
                        Tag::Mention(mention) => {
                            mentions.push(mention::ActiveModel {
                                post_id: ActiveValue::Set(this.id),
                                user_uri: ActiveValue::Set(mention.href.to_string()),
                                name: ActiveValue::Set(mention.name.clone()),
                            });
                        }
                        Tag::Emoji(emoji) => {
                            emojis.push(post_emoji::ActiveModel {
                                post_id: ActiveValue::Set(this.id),
                                name: ActiveValue::Set(emoji.name.clone()),
                                uri: ActiveValue::Set(emoji.id.to_string()),
                                media_type: ActiveValue::Set(emoji.icon.media_type.to_string()),
                                image_url: ActiveValue::Set(emoji.icon.url.to_string()),
                            });
                        }
                        Tag::Hashtag(hashtag) => {
                            hashtags.push(hashtag::ActiveModel {
                                post_id: ActiveValue::Set(this.id),
                                name: ActiveValue::Set(
                                    hashtag
                                        .name
                                        .strip_prefix('#')
                                        .unwrap_or(&hashtag.name)
                                        .to_string(),
                                ),
                            });
                        }
                    }
                }

                if !mentions.is_empty() {
                    mention::Entity::insert_many(mentions)
                        .on_conflict(
                            OnConflict::columns([
                                mention::Column::PostId,
                                mention::Column::UserUri,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(&tx)
                        .await
                        .context_internal_server_error("failed to insert to database")?;
                }
                if !emojis.is_empty() {
                    post_emoji::Entity::insert_many(emojis)
                        .on_conflict(
                            OnConflict::columns([
                                post_emoji::Column::PostId,
                                post_emoji::Column::Name,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(&tx)
                        .await
                        .context_internal_server_error("failed to insert to database")?;
                }
                if !hashtags.is_empty() {
                    hashtag::Entity::insert_many(hashtags)
                        .on_conflict(
                            OnConflict::columns([hashtag::Column::PostId, hashtag::Column::Name])
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
            NoteOrAnnounce::Announce(json) => {
                let user_uri: ObjectId<user::Model> = json.actor.into();
                let user = user_uri.dereference(data).await?;

                let repost_post = json.object.dereference(data).await?;
                let repost_id = repost_post.id;

                let visibility = calculate_visibility(&json.to, &json.cc);

                let mut this_activemodel = post::ActiveModel {
                    id: ActiveValue::Set(Ulid::new().into()),
                    created_at: ActiveValue::Set(json.published),
                    reply_id: ActiveValue::Set(None),
                    repost_id: ActiveValue::Set(Some(repost_id)),
                    text: ActiveValue::Set(String::new()),
                    title: ActiveValue::Set(None),
                    user_id: ActiveValue::Set(Some(user.id)),
                    visibility: ActiveValue::Set(visibility),
                    is_sensitive: ActiveValue::Set(false),
                    uri: ActiveValue::Set(json.id.inner().to_string()),
                    source_content: ActiveValue::Set(None),
                    source_media_type: ActiveValue::Set(None),
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
                    this_activemodel.id = ActiveValue::Unchanged(id);
                    this_activemodel
                        .update(&tx)
                        .await
                        .context_internal_server_error("failed to update database")?
                } else {
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
    }

    #[tracing::instrument(skip(data))]
    async fn delete(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let id = self.id;
        ModelTrait::delete(self, &*data.db)
            .await
            .context_internal_server_error("failed to delete from database")?;
        let event = Event::Update(Update::DeletePost { post_id: id.into() });
        event.send(&*data.db).await?;
        Ok(())
    }
}
