use axum::{routing, Router};

mod auth;
mod follow;
mod post;

pub(super) fn create_router() -> Router {
    let auth = self::auth::create_router();
    let follow = self::follow::create_router();
    let post = self::post::create_router();

    Router::new()
        .nest("/auth", auth)
        .nest("/follow", follow)
        .nest("/post", post)
        .route("/healthz", routing::get(get_healthz))
}

async fn get_healthz() -> &'static str {
    "OK"
}
