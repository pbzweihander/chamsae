use std::sync::Arc;

use activitypub_federation::{
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name, Webfinger},
};
use axum::{extract, http::Request, middleware::Next, response::Response, routing, Json, Router};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::Level;

use crate::{
    ap::person::LocalPerson,
    config::CONFIG,
    error::{Context, Result},
    format_err,
    state::State,
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

pub async fn create_router(db: DatabaseConnection) -> anyhow::Result<Router> {
    let state = State { db: Arc::new(db) };

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

    let router = Router::new()
        .nest("/api", api)
        .nest("/ap", ap)
        // TODO: We cannot use nested router because of https://github.com/LemmyNet/activitypub-federation-rust/issues/73
        .route("/ap/inbox", routing::post(self::ap::post_inbox))
        .route("/.well-known/webfinger", routing::get(get_webfinger))
        .layer(FederationMiddleware::new(federation_config))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(Level::INFO)))
        .layer(axum::middleware::from_fn(server_header_middleware));

    Ok(router)
}

#[derive(Debug, Deserialize)]
struct GetWebfingerQuery {
    resource: String,
}

#[tracing::instrument(skip(data))]
async fn get_webfinger(
    extract::Query(query): extract::Query<GetWebfingerQuery>,
    data: Data<State>,
) -> Result<Json<Webfinger>> {
    let name = extract_webfinger_name(&query.resource, &data)
        .context_bad_request("failed to extract resource name")?;
    if name == CONFIG.user_handle {
        let resp = build_webfinger_response(name, LocalPerson::id());
        Ok(Json(resp))
    } else {
        Err(format_err!(NOT_FOUND, "user not found"))
    }
}
