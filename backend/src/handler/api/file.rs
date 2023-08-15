use activitypub_federation::config::Data;
use axum::{body::Bytes, extract, routing, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait};
use ulid::Ulid;

use crate::{
    dto::{CreateFileQuery, IdPaginationQuery, IdResponse, LocalFile},
    entity::local_file,
    error::{Context, Result},
    format_err,
    state::State,
};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new()
        .route("/", routing::get(get_files).post(post_file))
        .route("/:id", routing::get(get_file).delete(delete_file))
}

#[utoipa::path(
    get,
    path = "/api/file",
    params(IdPaginationQuery),
    responses(
        (status = 200, body = Vec<LocalFile>),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_files(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<IdPaginationQuery>,
) -> Result<Json<Vec<LocalFile>>> {
    let pagination_query = local_file::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(local_file::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let files = pagination_query
        .order_by_desc(local_file::Column::Id)
        .limit(query.size)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let files = files
        .into_iter()
        .filter_map(|file| LocalFile::from_model(file).ok())
        .collect::<Vec<_>>();
    Ok(Json(files))
}

#[utoipa::path(
    post,
    path = "/api/file",
    params(CreateFileQuery),
    responses(
        (status = 200, body = IdResponse),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access, req))]
async fn post_file(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<CreateFileQuery>,
    req: Bytes,
) -> Result<Json<IdResponse>> {
    let file = local_file::Model::put(req, query.media_type, query.alt, &*data.db).await?;
    Ok(Json(IdResponse { id: file.id.into() }))
}

#[utoipa::path(
    get,
    path = "/api/file/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200, body = LocalFile),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_file(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<Json<LocalFile>> {
    let file = local_file::Entity::find_by_id(id)
        .order_by_desc(local_file::Column::Id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("file not found")?;
    Ok(Json(LocalFile::from_model(file)?))
}

#[utoipa::path(
    delete,
    path = "/api/file/{id}",
    params(
        ("id" = String, format = "ulid"),
    ),
    responses(
        (status = 200),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn delete_file(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<()> {
    let tx = data
        .db
        .begin()
        .await
        .context_internal_server_error("failed to begin database transaction")?;

    let existing = local_file::Entity::find_by_id(id)
        .one(&tx)
        .await
        .context_internal_server_error("failed to query database")?;

    if let Some(existing) = existing {
        if existing.post_id.is_some() || existing.emoji_name.is_some() {
            return Err(format_err!(
                BAD_REQUEST,
                "cannot delete file currently in use"
            ));
        }

        existing.delete(&tx).await?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;
    }

    Ok(())
}
