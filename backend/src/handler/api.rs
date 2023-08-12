use axum::{routing, Router};

mod auth;
mod emoji;
mod file;
mod follow;
mod post;

pub(super) fn create_router() -> Router {
    let auth = self::auth::create_router();
    let emoji = self::emoji::create_router();
    let file = self::file::create_router();
    let follow = self::follow::create_router();
    let post = self::post::create_router();

    Router::new()
        .nest("/auth", auth)
        .nest("/emoji", emoji)
        .nest("/file", file)
        .nest("/follow", follow)
        .nest("/post", post)
        .route("/healthz", routing::get(get_healthz))
}

async fn get_healthz() -> &'static str {
    "OK"
}
