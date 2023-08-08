use axum::{routing, Router};

use super::AppState;

mod auth;
mod post;

pub(super) fn create_router() -> Router<AppState> {
    let auth = self::auth::create_router();

    Router::new()
        .nest("/auth", auth)
        .route("/healthz", routing::get(get_healthz))
}

async fn get_healthz() -> &'static str {
    "OK"
}
