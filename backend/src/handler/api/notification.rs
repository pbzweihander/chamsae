use std::convert::Infallible;

use activitypub_federation::config::Data;
use axum::{
    response::{sse::Event, Sse},
    routing, Router,
};
use futures_util::Stream;

use crate::{queue::notification_stream, state::State};

use super::auth::Access;

pub(super) fn create_router() -> Router {
    Router::new().route("/stream", routing::get(get_stream))
}

#[utoipa::path(
    get,
    path = "/api/notification/stream",
    responses(
        (status = 200, description = "SSE stream", body = Notification),
    ),
    security(
        ("access_key" = []),
    ),
)]
#[tracing::instrument(skip(data, _access))]
async fn get_stream(
    data: Data<State>,
    _access: Access,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(data.stopper.stop_stream(notification_stream(
        data.queue.clone(),
        data.stopper.clone(),
    )))
}
