use activitypub_federation::{config::Data, traits::Object};
use axum::{extract, routing, Json, Router};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait,
};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    entity::follow,
    error::{Context, Result},
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::post(post_follow))
        .route("/:id", routing::delete(delete_follow))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostFollowReq {
    to_id: Ulid,
}

async fn post_follow(
    data: Data<State>,
    _access: Access,
    Json(req): Json<PostFollowReq>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing_count = follow::Entity::find()
        .filter(follow::Column::ToId.eq(req.to_id.to_string()))
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_count != 0 {
        return Ok(());
    }

    let follow_activemodel = follow::ActiveModel {
        id: ActiveValue::Set(Ulid::new().to_string()),
        to_id: ActiveValue::Set(req.to_id.to_string()),
        accepted: ActiveValue::Set(false),
    };
    let follow = follow_activemodel
        .insert(&tx)
        .await
        .context_internal_server_error("failed to insert to database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    let follow = follow.into_json(&data).await?;
    follow.send(&data).await?;

    Ok(())
}

async fn delete_follow(
    data: Data<State>,
    extract::Path(id): extract::Path<Ulid>,
    _access: Access,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing_count = follow::Entity::find()
        .filter(follow::Column::ToId.eq(id.to_string()))
        .count(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if existing_count == 0 {
        return Ok(());
    }

    follow::Entity::delete_by_id(id.to_string())
        .exec(&tx)
        .await
        .context_internal_server_error("failed to delete from database")?;

    tx.commit()
        .await
        .context_internal_server_error("failed to commit database transaction")?;

    // TODO: broadcast via ActivityPub

    Ok(())
}
