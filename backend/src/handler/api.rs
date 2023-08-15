use axum::{routing, Router};

pub mod auth;
pub mod emoji;
pub mod event;
pub mod file;
pub mod follow;
pub mod follower;
pub mod hashtag;
pub mod notification;
pub mod post;
pub mod reaction;
pub mod report;
pub mod resolve;
pub mod setting;

pub(super) fn create_router() -> Router {
    let auth = self::auth::create_router();
    let emoji = self::emoji::create_router();
    let event = self::event::create_router();
    let file = self::file::create_router();
    let follow = self::follow::create_router();
    let follower = self::follower::create_router();
    let hashtag = self::hashtag::create_router();
    let notification = self::notification::create_router();
    let post = self::post::create_router();
    let reaction = self::reaction::create_router();
    let report = self::report::create_router();
    let resolve = self::resolve::create_router();
    let setting = self::setting::create_router();

    Router::new()
        .nest("/auth", auth)
        .nest("/emoji", emoji)
        .nest("/event", event)
        .nest("/file", file)
        .nest("/follow", follow)
        .nest("/follower", follower)
        .nest("/hashtag", hashtag)
        .nest("/notification", notification)
        .nest("/post", post)
        .nest("/reaction", reaction)
        .nest("/report", report)
        .nest("/resolve", resolve)
        .nest("/setting", setting)
        .route("/healthz", routing::get(get_healthz))
}

async fn get_healthz() -> &'static str {
    "OK"
}
