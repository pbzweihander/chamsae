use std::sync::Arc;

use axum::{http::Request, middleware::Next, response::Response, Router};
use sea_orm::DatabaseConnection;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::CONFIG;

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
struct AppState {
    db: Arc<DatabaseConnection>,
}

pub fn create_router(db: DatabaseConnection) -> Router {
    let state = AppState { db: Arc::new(db) };

    let api = self::api::create_router();

    let router = Router::new().nest("/api", api).with_state(state);

    let router = if let Some(dir) = &CONFIG.static_files_directory_path {
        router.nest_service(
            "/",
            ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))),
        )
    } else {
        router
    };

    router.layer(axum::middleware::from_fn(server_header_middleware))
}
