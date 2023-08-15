use activitypub_federation::config::{Data, FederationConfig, FederationMiddleware};
use axum::{http::Request, middleware::Next, response::Response, routing, Json, Router};
use cached::{AsyncRedisCache, IOCachedAsync};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
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
    error::{Context, Result},
    state::State,
};

mod ap;
mod api;
mod file;
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
    let ap = self::ap::create_router();
    let api = self::api::create_router();
    let file = self::file::create_router();
    let well_known = self::well_known::create_router();

    let router = Router::new()
        .nest("/api", api)
        .nest("/ap", ap)
        .nest("/file", file)
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct NodeInfoSoftware {
    name: String,
    version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsageUsers {
    total: usize,
    active_month: usize,
    active_half_year: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoUsage {
    users: NodeInfoUsageUsers,
    local_posts: u64,
    local_comments: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoMetadataMaintainer {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfoMetadata {
    node_name: String,
    node_description: Option<String>,
    maintainer: NodeInfoMetadataMaintainer,
    theme_color: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeInfo {
    version: String,
    software: NodeInfoSoftware,
    protocols: Vec<String>,
    usage: NodeInfoUsage,
    open_registrations: bool,
    metadata: NodeInfoMetadata,
}

async fn get_nodeinfo_2_0(data: Data<State>) -> Result<Json<NodeInfo>> {
    static CACHE: OnceCell<AsyncRedisCache<u8, NodeInfo>> = OnceCell::const_new();
    let cache = CACHE
        .get_or_try_init(|| async move {
            AsyncRedisCache::new("fn_cache:get_nodeinfo_2_0", 60 * 10)
                .set_connection_string(CONFIG.redis_url.as_str())
                .build()
                .await
                .context_internal_server_error("failed to build Redis cache")
        })
        .await?;
    if let Some(cached) = cache
        .cache_get(&0)
        .await
        .context_internal_server_error("failed to get cache from Redis")?
    {
        Ok(Json(cached))
    } else {
        let setting = setting::Model::get(&*data.db).await?;
        let local_post_count = post::Entity::find()
            .filter(post::Column::UserId.is_null())
            .count(&*data.db)
            .await?;

        let nodeinfo = NodeInfo {
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
        };

        cache
            .cache_set(0, nodeinfo.clone())
            .await
            .context_internal_server_error("failed to set cache to Redis")?;

        Ok(Json(nodeinfo))
    }
}
