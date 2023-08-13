use activitypub_federation::config::Data;
use axum::{
    body::StreamBody,
    extract,
    http::header,
    response::{IntoResponse, Response},
    routing, Router,
};
use axum_extra::body::AsyncReadBody;
use sea_orm::EntityTrait;
use ulid::Ulid;

use crate::{
    entity::local_file,
    error::{Context, Result},
    state::State,
};

pub(super) fn create_router() -> Router {
    Router::new().route("/:id", routing::get(get_file))
}

#[tracing::instrument(skip(data))]
async fn get_file(data: Data<State>, extract::Path(id): extract::Path<Ulid>) -> Result<Response> {
    let file = local_file::Entity::find_by_id(id)
        .one(&*data.db)
        .await
        .context_internal_server_error("failed to query database")?
        .context_not_found("file not found")?;

    let headers = [(header::CONTENT_TYPE, &file.media_type)];
    Ok(if file.is_local() {
        let body = tokio::fs::File::open(&file.object_store_key)
            .await
            .context_internal_server_error("failed to open object from local filesystem")?;
        (headers, AsyncReadBody::new(body)).into_response()
    } else {
        let resp = data
            .http_client
            .get(&file.url)
            .send()
            .await
            .context_internal_server_error("failed to request to object URL")?;
        let body = resp.bytes_stream();
        (headers, StreamBody::new(body)).into_response()
    })
}
