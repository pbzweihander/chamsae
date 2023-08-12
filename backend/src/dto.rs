use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use migration::ConnectionTrait;
use mime::Mime;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    entity::{
        emoji, follow, hashtag, local_file, mention, post, post_emoji, reaction, remote_file,
        sea_orm_active_enums, user,
    },
    error::{Context, Result},
};

#[derive(Debug, Deserialize)]
pub struct IdPaginationQuery {
    #[serde(default)]
    pub after: Option<Ulid>,
}

#[derive(Debug, Deserialize)]
pub struct TimestampPaginationQuery {
    #[serde(default)]
    pub after: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize)]
pub struct IdResponse {
    pub id: Ulid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameResponse {
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Ulid,
    pub handle: String,
    pub name: Option<String>,
    pub host: String,
    pub uri: Url,
    pub avatar_url: Option<Url>,
    pub banner_url: Option<Url>,
    pub manually_approves_followers: bool,
}

impl User {
    pub fn from_model(user: user::Model) -> Result<Self> {
        Ok(Self {
            id: user.id.into(),
            handle: user.handle,
            name: user.name,
            host: user.host,
            uri: user
                .uri
                .parse()
                .context_internal_server_error("malformed user URI")?,
            avatar_url: user.avatar_url.and_then(|url| url.parse().ok()),
            banner_url: user.banner_url.and_then(|url| url.parse().ok()),
            manually_approves_followers: user.manually_approves_followers,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public,
    Home,
    Followers,
    DirectMessage,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    pub user_uri: Url,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub url: Url,
    pub alt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub name: String,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub image_url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContentReaction {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmojiReaction {
    pub emoji_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CreateReaction {
    Content(CreateContentReaction),
    Emoji(CreateEmojiReaction),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub user: Option<User>,
    pub content: String,
    pub emoji: Option<Emoji>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: Ulid,
    pub created_at: DateTime<FixedOffset>,
    pub reply_id: Option<Ulid>,
    pub text: String,
    pub title: Option<String>,
    pub user: Option<User>,
    pub visibility: Visibility,
    pub is_sensitive: bool,
    pub uri: Url,
    pub files: Vec<File>,
    pub reactions: Vec<Reaction>,
    pub mentions: Vec<Mention>,
    pub emojis: Vec<Emoji>,
    pub hashtags: Vec<String>,
}

impl Post {
    pub async fn from_model(post: post::Model, db: &impl ConnectionTrait) -> Result<Self> {
        let user = if post.user_id.is_some() {
            let user = post
                .find_related(user::Entity)
                .one(db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("user not found")?;
            Some(User::from_model(user)?)
        } else {
            None
        };

        let remote_files = post
            .find_related(remote_file::Entity)
            .order_by_asc(remote_file::Column::Order)
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;

        let files = remote_files
            .into_iter()
            .filter_map(|file| {
                Some(File {
                    media_type: file.media_type.parse().ok()?,
                    url: file.url.parse().ok()?,
                    alt: file.alt,
                })
            })
            .collect::<Vec<_>>();

        let reactions = reaction::Entity::find()
            .filter(reaction::Column::PostId.eq(post.id))
            .find_also_related(user::Entity)
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;
        let reactions = reactions
            .into_iter()
            .filter_map(|(reaction, user)| {
                let emoji = if let (Some(media_type), Some(image_url)) =
                    (reaction.emoji_media_type, reaction.emoji_image_url)
                {
                    Some(Emoji {
                        name: reaction.content.clone(),
                        media_type: Mime::from_str(&media_type).ok()?,
                        image_url: Url::parse(&image_url).ok()?,
                    })
                } else {
                    None
                };

                let user = if let Some(user) = user {
                    Some(User::from_model(user).ok()?)
                } else {
                    None
                };

                Some(Reaction {
                    user,
                    content: reaction.content,
                    emoji,
                })
            })
            .collect::<Vec<_>>();

        let mentions = post
            .find_related(mention::Entity)
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;
        let mentions = mentions
            .into_iter()
            .filter_map(|mention| {
                Some(Mention {
                    user_uri: mention.user_uri.parse().ok()?,
                    name: mention.name,
                })
            })
            .collect::<Vec<_>>();

        let emojis = post
            .find_related(post_emoji::Entity)
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;
        let emojis = emojis
            .into_iter()
            .filter_map(|emoji| {
                Some(Emoji {
                    name: emoji.name,
                    media_type: emoji.media_type.parse().ok()?,
                    image_url: emoji.image_url.parse().ok()?,
                })
            })
            .collect::<Vec<_>>();

        let hashtags = post
            .find_related(hashtag::Entity)
            .select_only()
            .column(hashtag::Column::Name)
            .into_tuple::<String>()
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;

        Ok(Self {
            id: post.id.into(),
            created_at: post.created_at,
            reply_id: post.reply_id.map(Into::into),
            text: post.text,
            title: post.title,
            user,
            visibility: match post.visibility {
                sea_orm_active_enums::Visibility::Public => Visibility::Public,
                sea_orm_active_enums::Visibility::Home => Visibility::Home,
                sea_orm_active_enums::Visibility::Followers => Visibility::Followers,
                sea_orm_active_enums::Visibility::DirectMessage => Visibility::DirectMessage,
            },
            is_sensitive: post.is_sensitive,
            uri: post
                .uri
                .parse()
                .context_internal_server_error("malformed post URI")?,
            files,
            reactions,
            mentions,
            emojis,
            hashtags,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
    #[serde(default)]
    pub reply_id: Option<Ulid>,
    pub text: String,
    #[serde(default)]
    pub title: Option<String>,
    pub visibility: Visibility,
    #[serde(default)]
    pub is_sensitive: bool,
    #[serde(default)]
    pub files: Vec<Ulid>,
    #[serde(default)]
    pub mentions: Vec<Mention>,
    #[serde(default)]
    pub emojis: Vec<String>,
    #[serde(default)]
    pub hashtags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFile {
    pub id: Ulid,
    pub posted: bool,
    pub emoji_name: Option<String>,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub url: Url,
    pub alt: Option<String>,
}

impl LocalFile {
    pub fn from_model(file: local_file::Model) -> Result<Self> {
        Ok(Self {
            id: file.id.into(),
            posted: file.post_id.is_some(),
            emoji_name: file.emoji_name,
            media_type: file
                .media_type
                .parse()
                .context_internal_server_error("malformed file media type")?,
            url: file
                .url
                .parse()
                .context_internal_server_error("malformed file URL")?,
            alt: file.alt,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileQuery {
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalEmoji {
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub image_url: Url,
}

impl LocalEmoji {
    pub fn from_model(emoji: emoji::Model, file: local_file::Model) -> Result<Self> {
        Ok(Self {
            name: emoji.name,
            created_at: emoji.created_at,
            media_type: file
                .media_type
                .parse()
                .context_internal_server_error("malformed media type")?,
            image_url: file
                .url
                .parse()
                .context_internal_server_error("malformed file URL")?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmoji {
    pub file_id: Ulid,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    #[serde(flatten)]
    pub user: User,
    pub accepted: bool,
}

impl Follow {
    pub fn from_model(follow: follow::Model, user: user::Model) -> Result<Self> {
        Ok(Self {
            user: User::from_model(user)?,
            accepted: follow.accepted,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFollow {
    pub to_id: Ulid,
}
