use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use derivative::Derivative;
use mime::Mime;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;
use utoipa::{IntoParams, ToSchema};

use crate::{
    entity::{
        emoji, follow, hashtag, local_file, mention, post, post_emoji, reaction, remote_file,
        report, sea_orm_active_enums, setting, user,
    },
    error::{Context, Result},
};

fn default_size() -> u64 {
    10
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct IdPaginationQuery {
    #[param(value_type = Option<String>, format = "ulid")]
    #[serde(default)]
    pub after: Option<Ulid>,
    #[param(default = 10)]
    #[serde(default = "default_size")]
    pub size: u64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct TimestampPaginationQuery {
    #[serde(default)]
    pub after: Option<DateTime<FixedOffset>>,
    #[param(default = 10)]
    #[serde(default = "default_size")]
    pub size: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IdResponse {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NameResponse {
    pub name: String,
}

#[derive(Derivative, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
    pub handle: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub host: String,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
    pub uri: Url,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
    #[schema(value_type = Option<String>, format = "ulid")]
    pub avatar_url: Option<Url>,
    #[derivative(Debug(format_with = "crate::fmt::debug_format_option_display"))]
    #[schema(value_type = Option<String>, format = "ulid")]
    pub banner_url: Option<Url>,
    pub manually_approves_followers: bool,
    pub is_bot: bool,
}

impl User {
    pub fn from_model(user: user::Model) -> Result<Self> {
        Ok(Self {
            id: user.id.into(),
            handle: user.handle,
            name: user.name,
            description: user.description,
            host: user.host,
            uri: user
                .uri
                .parse()
                .context_internal_server_error("malformed user URI")?,
            avatar_url: user.avatar_url.and_then(|url| url.parse().ok()),
            banner_url: user.banner_url.and_then(|url| url.parse().ok()),
            manually_approves_followers: user.manually_approves_followers,
            is_bot: user.is_bot,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public,
    Home,
    Followers,
    DirectMessage,
}

#[derive(Derivative, Deserialize, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
    pub user_uri: Url,
    pub name: String,
}

#[derive(Derivative, Deserialize, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[schema(value_type = String, format = "mime")]
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
    pub url: Url,
    pub alt: Option<String>,
}

#[derive(Derivative, Deserialize, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    pub name: String,
    #[schema(value_type = String, format = "mime")]
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
    pub image_url: Url,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateContentReaction {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmojiReaction {
    pub emoji_name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum CreateReaction {
    Content(CreateContentReaction),
    Emoji(CreateEmojiReaction),
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
    pub user: Option<User>,
    pub content: String,
    pub emoji: Option<Emoji>,
}

impl Reaction {
    pub fn from_model(reaction: reaction::Model, user: Option<user::Model>) -> Result<Self> {
        let emoji = if let (Some(media_type), Some(image_url)) =
            (reaction.emoji_media_type, reaction.emoji_image_url)
        {
            Some(Emoji {
                name: reaction.content.clone(),
                media_type: Mime::from_str(&media_type)
                    .context_internal_server_error("malformed reaction emoji MIME")?,
                image_url: Url::parse(&image_url)
                    .context_internal_server_error("malformed reaction emoji image URL")?,
            })
        } else {
            None
        };

        let user = if let Some(user) = user {
            Some(User::from_model(user)?)
        } else {
            None
        };

        Ok(Self {
            id: reaction.id.into(),
            user,
            content: reaction.content,
            emoji,
        })
    }
}

#[derive(Derivative, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
    pub created_at: DateTime<FixedOffset>,
    #[schema(value_type = Option<String>, format = "ulid")]
    pub reply_id: Option<Ulid>,
    #[schema(value_type = Vec<String>, format = "ulid")]
    pub replies_id: Vec<Ulid>,
    #[schema(value_type = Option<String>, format = "ulid")]
    pub repost_id: Option<Ulid>,
    pub text: String,
    pub title: Option<String>,
    pub source_content: Option<String>,
    pub source_media_type: Option<String>,
    pub user: Option<User>,
    pub visibility: Visibility,
    pub is_sensitive: bool,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
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

        let replies_id = post::Entity::find()
            .filter(post::Column::ReplyId.eq(post.id))
            .select_only()
            .column(post::Column::Id)
            .into_tuple::<uuid::Uuid>()
            .all(db)
            .await
            .context_internal_server_error("failed to query database")?;
        let replies_id = replies_id
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Ulid>>();

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
            .filter_map(|(reaction, user)| Reaction::from_model(reaction, user).ok())
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
            replies_id,
            repost_id: post.repost_id.map(Into::into),
            text: post.text,
            title: post.title,
            source_content: post.source_content,
            source_media_type: post.source_media_type,
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

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
    #[schema(value_type = Option<String>, format = "ulid")]
    #[serde(default)]
    pub reply_id: Option<Ulid>,
    #[schema(value_type = Option<String>, format = "ulid")]
    #[serde(default)]
    pub repost_id: Option<Ulid>,
    pub text: String,
    #[serde(default)]
    pub title: Option<String>,
    pub visibility: Visibility,
    #[serde(default)]
    pub is_sensitive: bool,
    #[schema(value_type = Vec<String>, format = "ulid")]
    #[serde(default)]
    pub files: Vec<Ulid>,
    #[serde(default)]
    pub mentions: Vec<Mention>,
    #[serde(default)]
    pub emojis: Vec<String>,
    #[serde(default)]
    pub hashtags: Vec<String>,
}

#[derive(Derivative, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalFile {
    #[schema(value_type = String, format = "ulid")]
    pub id: Ulid,
    pub posted: bool,
    pub emoji_name: Option<String>,
    #[schema(value_type = String, format = "mime")]
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
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

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileQuery {
    #[param(value_type = String, format = "mime")]
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Derivative, Serialize, ToSchema)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocalEmoji {
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
    #[schema(value_type = String, format = "mime")]
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    #[derivative(Debug(format_with = "std::fmt::Display::fmt"))]
    #[schema(value_type = String, format = "url")]
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

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmoji {
    #[schema(value_type = String, format = "ulid")]
    pub file_id: Ulid,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFollow {
    #[schema(value_type = String, format = "ulid")]
    pub to_id: Ulid,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub user_handle: String,
    pub user_name: Option<String>,
    pub user_description: Option<String>,
    pub instance_description: Option<String>,
    #[schema(value_type = Option<String>, format = "ulid")]
    pub avatar_file_id: Option<Ulid>,
    #[schema(value_type = Option<String>, format = "ulid")]
    pub banner_file_id: Option<Ulid>,
    pub maintainer_name: Option<String>,
    pub maintainer_email: Option<String>,
    pub theme_color: Option<String>,
}

impl Setting {
    pub fn from_model(setting: setting::Model) -> Self {
        Self {
            user_handle: setting.user_handle,
            user_name: setting.user_name,
            user_description: setting.user_description,
            instance_description: setting.instance_description,
            avatar_file_id: setting.avatar_file_id.map(Into::into),
            banner_file_id: setting.banner_file_id.map(Into::into),
            maintainer_name: setting.maintainer_name,
            maintainer_email: setting.maintainer_email,
            theme_color: setting.theme_color,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
pub enum Object {
    User(Box<User>),
    Post(Box<Post>),
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub from: User,
    pub content: String,
}

impl Report {
    pub fn from_model(report: report::Model, user: user::Model) -> Result<Self> {
        Ok(Self {
            from: User::from_model(user)?,
            content: report.content,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateReport {
    #[schema(value_type = String, format = "ulid")]
    pub user_id: Ulid,
    pub content: String,
}
