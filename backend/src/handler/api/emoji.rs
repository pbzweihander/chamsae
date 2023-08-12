use activitypub_federation::config::Data;
use axum::{extract, routing, Json, Router};
use chrono::Utc;

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

use crate::{
    dto::{CreateEmoji, LocalEmoji, NameResponse, TimestampPaginationQuery},
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

#[tracing::instrument(skip(data, _access))]
async fn get_emojis(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<TimestampPaginationQuery>,
) -> Result<Json<Vec<LocalEmoji>>> {
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
        .filter_map(|(emoji, file)| LocalEmoji::from_model(emoji, file).ok())
        .collect::<Vec<_>>();
    Ok(Json(emojis))
}

#[tracing::instrument(skip(data, _access))]
async fn post_emoji(
    data: Data<State>,
    _access: Access,
    Json(req): Json<CreateEmoji>,
) -> Result<Json<NameResponse>> {
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

    Ok(Json(NameResponse { name: emoji.name }))
}

#[tracing::instrument(skip(data, _access))]
async fn get_emoji(
    data: Data<State>,
    _access: Access,
    extract::Path(name): extract::Path<String>,
) -> Result<Json<LocalEmoji>> {
    let (emoji, file) = emoji::Entity::find_by_id(name)
        .find_also_related(local_file::Entity)
        .order_by_desc(emoji::Column::CreatedAt)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("emoji not found")?;
    let file = file.context_internal_server_error("file not found")?;
    Ok(Json(LocalEmoji::from_model(emoji, file)?))
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
