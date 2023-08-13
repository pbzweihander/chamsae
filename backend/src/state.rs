use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::config::CONFIG;

#[derive(Clone)]
pub struct State {
    pub db: Arc<DatabaseConnection>,
    pub http_client: reqwest::Client,
}

impl State {
    pub fn new(db: DatabaseConnection) -> anyhow::Result<Self> {
        let http_client = anyhow::Context::context(
            reqwest::Client::builder()
                .danger_accept_invalid_certs(CONFIG.debug)
                .build(),
            "failed to build HTTP client",
        )?;
        Ok(State {
            db: Arc::new(db),
            http_client,
        })
    }
}
