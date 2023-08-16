use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgListener;
use stopper::Stopper;

use crate::{config::CONFIG, error::Error};

#[derive(Clone)]
pub struct State {
    pub db: Arc<DatabaseConnection>,
    pub db_pool: Pool<Postgres>,
    pub http_client: reqwest::Client,
    pub stopper: Stopper,
}

impl State {
    pub async fn new(db: DatabaseConnection, stopper: Stopper) -> anyhow::Result<Self> {
        use anyhow::Context;

        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(CONFIG.debug)
            .build()
            .context("failed to build HTTP client")?;
        let db_pool = db.get_postgres_connection_pool().clone();
        Ok(State {
            db: Arc::new(db),
            db_pool,
            http_client,
            stopper,
        })
    }

    pub async fn pg_listener(&self) -> Result<PgListener, Error> {
        use crate::error::Context;

        PgListener::connect_with(&self.db_pool)
            .await
            .context_internal_server_error("cannot connect Postgres listener")
    }
}
