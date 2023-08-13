use std::sync::Arc;

use anyhow::Context;
use pgmq::PGMQueue;
use sea_orm::DatabaseConnection;
use stopper::Stopper;

use crate::{config::CONFIG, queue::init_queue};

#[derive(Clone)]
pub struct State {
    pub db: Arc<DatabaseConnection>,
    pub http_client: reqwest::Client,
    pub queue: PGMQueue,
    pub stopper: Stopper,
}

impl State {
    pub async fn new(db: DatabaseConnection, stopper: Stopper) -> anyhow::Result<Self> {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(CONFIG.debug)
            .build()
            .context("failed to build HTTP client")?;
        let pool = db.get_postgres_connection_pool();
        let queue = PGMQueue::new_with_pool(pool.clone())
            .await
            .context("failed to build message queue")?;
        init_queue(&queue).await?;
        Ok(State {
            db: Arc::new(db),
            http_client,
            queue,
            stopper,
        })
    }
}
