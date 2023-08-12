use activitypub_federation::config::Data;
use axum::{routing, Json, Router};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::{emoji, local_file},
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/", routing::post(post_emoji))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostEmojiReq {
    file_id: Uuid,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PostEmojiResp {
    id: Uuid,
}

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

    let existing_count = emoji::Entity::find()
        .filter(emoji::Column::Name.eq(&req.name))
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
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::Set(req.name),
    };

    let emoji = emoji_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    file.attach_to_emoji(emoji.id, &tx).await?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    Ok(Json(PostEmojiResp { id: emoji.id }))
}
