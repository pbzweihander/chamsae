use std::sync::Arc;

use activitypub_federation::config::{Data, FederationConfig, FederationMiddleware};
use axum::{http::Request, middleware::Next, response::Response, routing, Json, Router};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::Serialize;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Level;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_redoc::{Redoc, Servable};

use crate::{
    config::CONFIG,
    entity::{post, setting},
    error::Result,
    state::State,
};

mod ap;
mod api;
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
        self::api::post::get_posts,
        self::api::post::post_post,
        self::api::post::get_post,
        self::api::post::delete_post,
        self::api::post::post_reaction,
        self::api::post::delete_reaction,
        self::api::resolve::get_user,
        self::api::resolve::get_link,
        self::api::setting::get_setting,
        self::api::setting::put_setting,
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
        self::api::auth::PostLoginReq,
        self::api::auth::PostLoginResp,
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

pub async fn create_router(db: DatabaseConnection) -> anyhow::Result<Router> {
    let http_client = anyhow::Context::context(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(CONFIG.debug)
            .danger_accept_invalid_hostnames(CONFIG.debug)
            .build(),
        "failed to build HTTP client",
    )?;
    let state = State {
        db: Arc::new(db),
        http_client,
    };

    let federation_config = anyhow::Context::context(
        FederationConfig::builder()
            .domain(&crate::config::CONFIG.domain)
            .app_data(state.clone())
            .debug(CONFIG.debug)
            .build()
            .await,
        "failed to build federation config",
    )?;

    let ap = self::ap::create_router();
    let api = self::api::create_router();
    let well_known = self::well_known::create_router();

    let router = Router::new()
        .nest("/api", api)
        .nest("/ap", ap)
        .nest("/.well-known", well_known)
        .route("/nodeinfo/2.0", routing::get(get_nodeinfo_2_0))
        // TODO: We cannot use nested router because of https://github.com/LemmyNet/activitypub-federation-rust/issues/73
        .route("/ap/inbox", routing::post(self::ap::post_inbox))
        .route(
            "/openapi.json",
            routing::get(|| async move { Json(ApiDoc::openapi()) }),
        )
        .merge(Redoc::with_url("/api-doc", ApiDoc::openapi()))
        .layer(FederationMiddleware::new(federation_config))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .layer(axum::middleware::from_fn(server_header_middleware));

    Ok(router)
}

#[derive(Debug, Serialize)]
struct NodeInfoSoftware {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsageUsers {
    total: usize,
    active_month: usize,
    active_half_year: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsage {
    users: NodeInfoUsageUsers,
    local_posts: u64,
    local_comments: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoMetadataMaintainer {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoMetadata {
    node_name: Option<String>,
    node_description: Option<String>,
    maintainer: NodeInfoMetadataMaintainer,
    theme_color: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfo {
    version: String,
    software: NodeInfoSoftware,
    protocols: Vec<String>,
    usage: NodeInfoUsage,
    open_registrations: bool,
    metadata: NodeInfoMetadata,
}

// TODO: cache
async fn get_nodeinfo_2_0(data: Data<State>) -> Result<Json<NodeInfo>> {
    let setting = setting::Model::get(&*data.db).await?;
    let local_post_count = post::Entity::find()
        .filter(post::Column::UserId.is_null())
        .count(&*data.db)
        .await?;

    Ok(Json(NodeInfo {
        version: "2.0".to_string(),
        software: NodeInfoSoftware {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        usage: NodeInfoUsage {
            users: NodeInfoUsageUsers {
                total: 1,
                active_month: 1,
                active_half_year: 1,
            },
            local_posts: local_post_count,
            local_comments: 0,
        },
        open_registrations: false,
        metadata: NodeInfoMetadata {
            node_name: setting.instance_name,
            node_description: setting.instance_description,
            maintainer: NodeInfoMetadataMaintainer {
                name: setting.maintainer_name,
                email: setting.maintainer_email,
            },
            theme_color: setting.theme_color,
        },
    }))
}
