use std::str::FromStr;

use activitypub_federation::{config::Data, traits::Object};
use axum::{extract, routing, Json, Router};
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::{stream::FuturesOrdered, TryStreamExt};
use migration::ConnectionTrait;
use mime::Mime;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    ap::delete::Delete,
    entity::{
        emoji, local_file, mention, post, post_emoji, reaction, remote_file, sea_orm_active_enums,
        user,
    },
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_posts).post(post_post))
        .route("/:id", routing::get(get_post).delete(delete_post))
        .route("/:id/reaction", routing::post(post_reaction))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Visibility {
    Public,
    Home,
    Followers,
    DirectMessage,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Mention {
    user_uri: Url,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPostsQuery {
    #[serde(default)]
    after: Option<Ulid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostRespUser {
    handle: String,
    host: String,
    uri: Url,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostRespFile {
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    url: Url,
    alt: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostRespEmoji {
    name: String,
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    image_url: Url,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostRespReaction {
    user: Option<GetPostRespUser>,
    content: String,
    emoji: Option<GetPostRespEmoji>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPostResp {
    id: Ulid,
    created_at: DateTime<FixedOffset>,
    reply_id: Option<Ulid>,
    text: String,
    title: Option<String>,
    user: Option<GetPostRespUser>,
    visibility: Visibility,
    is_sensitive: bool,
    uri: Url,
    files: Vec<GetPostRespFile>,
    reactions: Vec<GetPostRespReaction>,
    mentions: Vec<Mention>,
    emojis: Vec<GetPostRespEmoji>,
}

impl GetPostResp {
    async fn from_model(post: post::Model, db: &impl ConnectionTrait) -> Result<Self> {
        let user = if post.user_id.is_some() {
            let user = post
                .find_related(user::Entity)
                .one(db)
                .await
                .context_internal_server_error("failed to query database")?
                .context_internal_server_error("user not found")?;
            Some(GetPostRespUser {
                handle: user.handle,
                host: user.host,
                uri: user
                    .uri
                    .parse()
                    .context_internal_server_error("malformed user URI")?,
            })
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
                Some(GetPostRespFile {
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
                    Some(GetPostRespEmoji {
                        name: reaction.content.clone(),
                        media_type: Mime::from_str(&media_type).ok()?,
                        image_url: Url::parse(&image_url).ok()?,
                    })
                } else {
                    None
                };

                let user = if let Some(user) = user {
                    Some(GetPostRespUser {
                        handle: user.handle,
                        host: user.host,
                        uri: user.uri.parse().ok()?,
                    })
                } else {
                    None
                };

                Some(GetPostRespReaction {
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
                Some(GetPostRespEmoji {
                    name: emoji.name,
                    media_type: emoji.media_type.parse().ok()?,
                    image_url: emoji.image_url.parse().ok()?,
                })
            })
            .collect::<Vec<_>>();

        Ok(GetPostResp {
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
        })
    }
}

#[tracing::instrument(skip(data, _access))]
async fn get_posts(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<GetPostsQuery>,
) -> Result<Json<Vec<GetPostResp>>> {
    let pagination_query = post::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(post::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let posts = pagination_query
        .order_by_desc(post::Column::Id)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let posts = posts
        .into_iter()
        .map(|post| GetPostResp::from_model(post, &*data.db))
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;
    Ok(Json(posts))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostPostReq {
    #[serde(default)]
    reply_id: Option<Ulid>,
    text: String,
    #[serde(default)]
    title: Option<String>,
    visibility: Visibility,
    is_sensitive: bool,
    #[serde(default)]
    files: Vec<Ulid>,
    #[serde(default)]
    mentions: Vec<Mention>,
    #[serde(default)]
    emojis: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PostPostResp {
    id: Ulid,
}

#[tracing::instrument(skip(data, _access, req))]
async fn post_post(
    data: Data<State>,
    _access: Access,
    Json(req): Json<PostPostReq>,
) -> Result<Json<PostPostResp>> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    if let Some(reply_id) = req.reply_id {
        let reply_post_count = post::Entity::find_by_id(reply_id)
            .count(&tx)
            .await
            .context_internal_server_error("failed to request database")?;
        if reply_post_count == 0 {
            return Err(format_err!(NOT_FOUND, "reply target post not found"));
        }
    }

    let emojis = emoji::Entity::find()
        .filter(emoji::Column::Name.is_in(req.emojis))
        .find_also_related(local_file::Entity)
        .all(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    let id = Ulid::new();
    let post_activemodel = post::ActiveModel {
        id: ActiveValue::Set(id.into()),
        created_at: ActiveValue::Set(Utc::now().fixed_offset()),
        reply_id: ActiveValue::Set(req.reply_id.map(Into::into)),
        text: ActiveValue::Set(req.text),
        title: ActiveValue::Set(req.title),
        user_id: ActiveValue::Set(None),
        visibility: ActiveValue::Set(match req.visibility {
            Visibility::Public => sea_orm_active_enums::Visibility::Public,
            Visibility::Home => sea_orm_active_enums::Visibility::Home,
            Visibility::Followers => sea_orm_active_enums::Visibility::Followers,
            Visibility::DirectMessage => sea_orm_active_enums::Visibility::DirectMessage,
        }),
        is_sensitive: ActiveValue::Set(req.is_sensitive),
        uri: ActiveValue::Set(post::Model::ap_id_from_id(id)?.to_string()),
    };
    let post = post_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    for (idx, local_file_id) in req.files.into_iter().enumerate() {
        let file = local_file::Entity::find_by_id(local_file_id)
            .one(&tx)
            .await
            .context_internal_server_error("failed to query database")?
            .context_not_found("file not found")?;
        file.attach_to_post(post.id.into(), idx as u8, &tx).await?;
    }

    let emojis = emojis
        .into_iter()
        .filter_map(|(emoji, file)| file.map(|file| (emoji, file)))
        .filter_map(|(emoji, file)| {
            Some(post_emoji::ActiveModel {
                post_id: ActiveValue::Set(post.id),
                uri: ActiveValue::Set(emoji.ap_id().ok()?.to_string()),
                name: ActiveValue::Set(emoji.name),
                media_type: ActiveValue::Set(file.media_type),
                image_url: ActiveValue::Set(file.url),
            })
        })
        .collect::<Vec<_>>();
    if !emojis.is_empty() {
        post_emoji::Entity::insert_many(emojis)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;
    }

    let mentions = req
        .mentions
        .into_iter()
        .map(|mention| mention::ActiveModel {
            post_id: ActiveValue::Set(post.id),
            user_uri: ActiveValue::Set(mention.user_uri.to_string()),
            name: ActiveValue::Set(mention.name),
        })
        .collect::<Vec<_>>();
    if !mentions.is_empty() {
        mention::Entity::insert_many(mentions)
            .exec(&tx)
            .await
            .context_internal_server_error("failed to insert to database")?;
    }

    tx.commit()
        .await
        .context_internal_server_error("failed to commmit database transaction")?;

    let post_id = post.id.into();
    let post = post.into_json(&data).await?;
    let create = post.into_create()?;
    create.send(&data).await?;

    Ok(Json(PostPostResp { id: post_id }))
}

#[tracing::instrument(skip(data, _access))]
async fn get_post(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<Json<GetPostResp>> {
    let post = post::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("post not found")?;
    Ok(Json(GetPostResp::from_model(post, &*data.db).await?))
}

#[tracing::instrument(skip(data, _access))]
async fn delete_post(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = post::Entity::find_by_id(id)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        let uri = existing.uri.clone();

        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;

        let delete = Delete::new(
            uri.parse()
                .context_internal_server_error("malformed post URI")?,
        )?;
        delete.send(&data).await?;

        Ok(())
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostReactionReqContent {
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostReactionReqEmoji {
    emoji_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PostReactionReq {
    Content(PostReactionReqContent),
    Emoji(PostReactionReqEmoji),
}

#[tracing::instrument(skip(data, _access))]
async fn post_reaction(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
    Json(req): Json<PostReactionReq>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing_post_count = post::Entity::find_by_id(id)
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_post_count == 0 {
        return Err(format_err!(NOT_FOUND, "post not found"));
    }

    let existing_reaction_count = reaction::Entity::find()
        .filter(
            reaction::Column::PostId
                .eq(uuid::Uuid::from(id))
                .and(reaction::Column::UserId.is_null()),
        )
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_reaction_count > 0 {
        return Err(format_err!(CONFLICT, "already reacted post"));
    }

    let (content, emoji_uri, emoji_media_type, emoji_image_url) = match req {
        PostReactionReq::Emoji(req) => {
            let (emoji, file) = emoji::Entity::find_by_id(req.emoji_name)
                .find_also_related(local_file::Entity)
                .one(&tx)
                .await
                .context_internal_server_error("failed to query database")?
                .context_not_found("emoji not found")?;
            let file = file.context_internal_server_error("failed to find emoji file")?;
            (
                format!(":{}:", emoji.name),
                Some(emoji.ap_id()?.to_string()),
                Some(file.media_type),
                Some(file.url),
            )
        }
        PostReactionReq::Content(req) => (req.content, None, None, None),
    };

    let reaction_id = Ulid::new();
    let reaction_activemodel = reaction::ActiveModel {
        id: ActiveValue::Set(reaction_id.into()),
        user_id: ActiveValue::Set(None),
        post_id: ActiveValue::Set(id.into()),
        content: ActiveValue::Set(content),
        uri: ActiveValue::Set(reaction::Model::ap_id_from_id(reaction_id)?.to_string()),
        emoji_uri: ActiveValue::Set(emoji_uri),
        emoji_media_type: ActiveValue::Set(emoji_media_type),
        emoji_image_url: ActiveValue::Set(emoji_image_url),
    };
    let reaction = reaction_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transation")?;

    let like = reaction.into_json(&data).await?;
    like.send(&data).await?;

    Ok(())
}
