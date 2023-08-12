use activitypub_federation::config::Data;
use axum::{body::Bytes, extract, routing, Json, Router};
use mime::Mime;
use sea_orm::{
    ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

use crate::{
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetFilesQuery {
    #[serde(default)]
    after: Option<Ulid>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetFileResp {
    id: Ulid,
    posted: bool,
    emoji_name: Option<String>,
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    url: Url,
    alt: Option<String>,
}

impl GetFileResp {
    fn from_model(file: local_file::Model) -> Result<Self> {
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

#[tracing::instrument(skip(data, _access))]
async fn get_files(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<GetFilesQuery>,
) -> Result<Json<Vec<GetFileResp>>> {
    let pagination_query = local_file::Entity::find();
    let pagination_query = if let Some(after) = query.after {
        pagination_query.filter(local_file::Column::Id.lt(uuid::Uuid::from(after)))
    } else {
        pagination_query
    };
    let files = pagination_query
        .order_by_desc(local_file::Column::Id)
        .limit(100)
        .all(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?;
    let files = files
        .into_iter()
        .filter_map(|file| GetFileResp::from_model(file).ok())
        .collect::<Vec<_>>();
    Ok(Json(files))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostFileQuery {
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    #[serde(default)]
    alt: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PostFileResp {
    id: Ulid,
}

#[tracing::instrument(skip(data, _access, req))]
async fn post_file(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<PostFileQuery>,
    req: Bytes,
) -> Result<Json<PostFileResp>> {
    let file = local_file::Model::new(req, query.media_type, query.alt, &*data.db).await?;
    Ok(Json(PostFileResp { id: file.id.into() }))
}

#[tracing::instrument(skip(data, _access))]
async fn get_file(
    data: Data<State>,
    _access: Access,
    extract::Path(id): extract::Path<Ulid>,
) -> Result<Json<GetFileResp>> {
    let file = local_file::Entity::find_by_id(id)
        .order_by_desc(local_file::Column::Id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("file not found")?;
    Ok(Json(GetFileResp::from_model(file)?))
}

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

        ModelTrait::delete(existing, &tx)
            .await
            .context_internal_server_error("failed to delete from database")?;

        tx.commit()
            .await
            .context_internal_server_error("failed to commit database transaction")?;
    }

    Ok(())
}
