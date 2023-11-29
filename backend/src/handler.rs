use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use axum::{http::Request, middleware::Next, response::Response, routing, Json, Router};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Level;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_redoc::{Redoc, Servable};

use crate::state::State;

mod ap;
mod api;
mod file;
mod frontend;
mod nodeinfo;
mod well_known;

async fn server_header_middleware<B>(req: Request<B>, next: Next<B>) -> Response {
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        "server",
        format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .parse()
            .unwrap(),
    );
    resp
}

#[derive(OpenApi)]
#[openapi(
    paths(
        self::api::auth::post_login,
        self::api::auth::get_check,
        self::api::emoji::get_emojis,
        self::api::emoji::post_emoji,
        self::api::emoji::get_emoji,
        self::api::emoji::delete_emoji,
        self::api::event::get_event_stream,
        self::api::file::get_files,
        self::api::file::post_file,
        self::api::file::get_file,
        self::api::file::delete_file,
        self::api::follow::get_follows,
        self::api::follow::post_follow,
        self::api::follow::delete_follow,
        self::api::follower::get_followers,
        self::api::follower::delete_follower,
        self::api::hashtag::get_hashtag_posts,
        self::api::notification::get_notifications,
        self::api::notification::get_notification,
        self::api::post::get_posts,
        self::api::post::post_post,
        self::api::post::get_post,
        self::api::post::delete_post,
        self::api::post::get_post_reactions,
        self::api::post::post_post_reaction,
        self::api::post::delete_post_reaction,
        self::api::reaction::get_reaction,
        self::api::report::get_reports,
        self::api::report::post_report,
        self::api::report::get_report,
        self::api::resolve::get_resolve_user,
        self::api::resolve::get_resolve_link,
        self::api::setting::get_setting,
        self::api::setting::put_setting,
        self::api::setting::post_initial_setting,
    ),
    components(schemas(
        crate::dto::IdResponse,
        crate::dto::NameResponse,
        crate::dto::User,
        crate::dto::Visibility,
        crate::dto::Mention,
        crate::dto::File,
        crate::dto::Emoji,
        crate::dto::CreateContentReaction,
        crate::dto::CreateEmojiReaction,
        crate::dto::CreateReaction,
        crate::dto::Reaction,
        crate::dto::Post,
        crate::dto::CreatePost,
        crate::dto::LocalFile,
        crate::dto::LocalEmoji,
        crate::dto::CreateEmoji,
        crate::dto::Follow,
        crate::dto::CreateFollow,
        crate::dto::Setting,
        crate::dto::Object,
        crate::dto::Report,
        crate::dto::CreateReport,
        crate::queue::Event,
        crate::queue::Update,
        crate::queue::Notification,
        crate::queue::NotificationType,
        self::api::auth::PostLoginReq,
        self::api::auth::PostLoginResp,
        self::api::setting::PutSettingReq,
        self::api::setting::PostInitialSettingReq,
    )),
    modifiers(&AccessKeyAddon),
)]
struct ApiDoc;

struct AccessKeyAddon;

impl Modify for AccessKeyAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "access_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }
}

pub async fn create_router(federation_config: FederationConfig<State>) -> anyhow::Result<Router> {
    let api = self::api::create_router();
    let file = self::file::create_router();
    let well_known = self::well_known::create_router();

    let follow = self::ap::follow::create_router();
    let like = self::ap::like::create_router();
    let note = self::ap::note::create_router();
    let person = self::ap::person::create_router();

    let assets = self::frontend::assets::create_router();

    let router = Router::new()
        .nest("/api", api)
        .nest("/file", file)
        .nest("/.well-known", well_known)
        .route(
            "/nodeinfo/2.0",
            routing::get(self::nodeinfo::get_nodeinfo_2_0),
        )
        .nest("/follow", follow)
        .nest("/like", like)
        .nest("/note", note)
        .nest("/person", person)
        .route("/inbox", routing::post(self::ap::post_inbox))
        .route(
            "/openapi.json",
            routing::get(|| async move { Json(ApiDoc::openapi()) }),
        )
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .route("/", routing::get(self::frontend::get_index))
        .route("/*path", routing::get(self::frontend::get_not_found))
        .layer(FederationMiddleware::new(federation_config))
        .nest("/assets", assets)
        .merge(Redoc::with_url("/api-doc", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(server_header_middleware));

    Ok(router)
}
