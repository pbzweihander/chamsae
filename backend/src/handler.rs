use std::sync::Arc;

use activitypub_federation::{
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name, Webfinger},
};
use axum::{extract, http::Request, middleware::Next, response::Response, routing, Json, Router};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Level;

use crate::{
    config::CONFIG,
    error::{Context, Result},
    format_err,
};

mod ap;
mod api;

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

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

pub async fn create_router(db: DatabaseConnection) -> anyhow::Result<Router> {
    let state = AppState { db: Arc::new(db) };

    let federation_config = anyhow::Context::context(
        FederationConfig::builder()
            .domain(&crate::config::CONFIG.domain)
            .app_data(state.clone())
            .debug(true) // TODO: remove
            .build()
            .await,
        "failed to build federation config",
    )?;

    // let ap = self::ap::create_router();
    let api = self::api::create_router();

    let router = Router::new()
        .nest("/api", api)
        .with_state(state)
        // .nest("/ap", ap)
        // TODO: We cannot use nested router because of https://github.com/LemmyNet/activitypub-federation-rust/issues/73
        .route("/ap/user", routing::get(self::ap::get_user))
        .route("/ap/inbox", routing::post(self::ap::post_inbox))
        .route("/.well-known/webfinger", routing::get(get_webfinger))
        .layer(FederationMiddleware::new(federation_config))
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)),
        );

    let router = if let Some(dir) = &CONFIG.static_files_directory_path {
        router.nest_service(
            "/",
            ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))),
        )
    } else {
        router
    };

    Ok(router.layer(axum::middleware::from_fn(server_header_middleware)))
}

#[derive(Deserialize)]
struct GetWebfingerQuery {
    resource: String,
}

async fn get_webfinger(
    extract::Query(query): extract::Query<GetWebfingerQuery>,
    data: Data<AppState>,
) -> Result<Json<Webfinger>> {
    let name = extract_webfinger_name(&query.resource, &data)
        .context_bad_request("failed to extract resource name")?;
    if name == CONFIG.user_handle {
        let resp = build_webfinger_response(
            name,
            format!("https://{}/ap/user", CONFIG.domain)
                .parse()
                .context_internal_server_error("failed to construct URL")?,
        );
        Ok(Json(resp))
    } else {
        Err(format_err!(NOT_FOUND, "user not found"))
    }
}
