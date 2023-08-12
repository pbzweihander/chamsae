use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use chrono::{DateTime, FixedOffset, Utc};
use mime::Mime;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
    entity::{emoji, local_file},
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_emojis).post(post_emoji))
        .route("/:name", routing::get(get_emoji).delete(delete_emoji))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetEmojisQuery {
    #[serde(default)]
    after: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetEmojiResp {
    name: String,
    created_at: DateTime<FixedOffset>,
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    image_url: Url,
}

#[tracing::instrument(skip(data, _access))]
async fn get_emojis(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<GetEmojisQuery>,
) -> Result<Json<Vec<GetEmojiResp>>> {
    let pagination_query = emoji::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(emoji::Column::CreatedAt.lt(after))
    } else {
        pagination_query
    };
    let emojis = pagination_query
        .find_also_related(local_file::Entity)
        .order_by_desc(emoji::Column::CreatedAt)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let emojis = emojis
        .into_iter()
        .filter_map(|(emoji, file)| file.map(|file| (emoji, file)))
        .filter_map(|(emoji, file)| {
            Some(GetEmojiResp {
                name: emoji.name,
                created_at: emoji.created_at,
                media_type: file.media_type.parse().ok()?,
                image_url: file.url.parse().ok()?,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(emojis))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostEmojiReq {
    file_id: Ulid,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PostEmojiResp {
    name: String,
}

#[tracing::instrument(skip(data, _access))]
async fn post_emoji(
    data: Data<State>,
    _access: Access,
    Json(req): Json<PostEmojiReq>,
) -> Result<Json<PostEmojiResp>> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing_count = emoji::Entity::find_by_id(&req.name)
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_count > 0 {
        return Err(format_err!(CONFLICT, "emoji name already exists"));
    }

    let file = local_file::Entity::find_by_id(req.file_id)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("file not found")?;

    let emoji_activemodel = emoji::ActiveModel {
        name: ActiveValue::Set(req.name),
        created_at: ActiveValue::Set(Utc::now().fixed_offset()),
    };

    let emoji = emoji_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    file.attach_to_emoji(emoji.name.clone(), &tx).await?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    Ok(Json(PostEmojiResp { name: emoji.name }))
}

#[tracing::instrument(skip(data, _access))]
async fn get_emoji(
    data: Data<State>,
    _access: Access,
    extract::Path(name): extract::Path<String>,
) -> Result<Json<GetEmojiResp>> {
    let (emoji, file) = emoji::Entity::find_by_id(name)
        .find_also_related(local_file::Entity)
        .order_by_desc(emoji::Column::CreatedAt)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("emoji not found")?;
    let file = file.context_internal_server_error("file not found")?;
    Ok(Json(GetEmojiResp {
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
    }))
}

#[tracing::instrument(skip(data, _access))]
async fn delete_emoji(
    data: Data<State>,
    _access: Access,
    extract::Path(name): extract::Path<String>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = emoji::Entity::find_by_id(name)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;
    }

    Ok(())
}
