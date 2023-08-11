use activitypub_federation::config::Data;
use axum::{body::Bytes, extract, routing, Json, Router};
use mime::Mime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{entity::local_file, error::Result, state::State};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/", routing::post(post_file))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostFileQuery {
    #[serde(with = "mime_serde_shim")]
    media_type: Mime,
    alt: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PostFileResp {
    id: Uuid,
}

#[tracing::instrument(skip(data, _access, req))]
async fn post_file(
    data: Data<State>,
    _access: Access,
    extract::Query(query): extract::Query<PostFileQuery>,
    req: Bytes,
) -> Result<Json<PostFileResp>> {
    let file = local_file::Model::new(req, query.media_type, query.alt, &*data.db).await?;
    Ok(Json(PostFileResp { id: file.id }))
}
