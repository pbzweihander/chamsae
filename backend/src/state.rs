use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stopper::Stopper;

use crate::{config::CONFIG, error::Error};

#[derive(Clone)]
pub struct State {
    pub db: Arc<DatabaseConnection>,
    pub redis_client: redis::Client,
    pub redis_connection_manager: redis::aio::ConnectionManager,
    pub http_client: reqwest::Client,
    pub stopper: Stopper,
}

impl State {
    pub async fn new(
        db: DatabaseConnection,
        redis_client: redis::Client,
        stopper: Stopper,
    ) -> anyhow::Result<Self> {
        use anyhow::Context;

        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(CONFIG.debug)
            .build()
            .context("failed to build HTTP client")?;
        let redis_connection_manager = redis::aio::ConnectionManager::new(redis_client.clone())
            .await
            .context("failed to build Redis connection manager")?;
        Ok(State {
            db: Arc::new(db),
            redis_client,
            redis_connection_manager,
            http_client,
            stopper,
        })
    }

    pub fn redis(&self) -> redis::aio::ConnectionManager {
        self.redis_connection_manager.clone()
    }

    pub async fn redis_pubsub(&self) -> Result<redis::aio::PubSub, Error> {
        use crate::error::Context;

        let conn = self
            .redis_client
            .get_async_connection()
            .await
            .context_internal_server_error("failed to get Redis connection")?;
        Ok(conn.into_pubsub())
    }
}
