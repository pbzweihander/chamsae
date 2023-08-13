use activitypub_federation::config::FederationConfig;
use anyhow::Context;
use migration::MigratorTrait;
use sea_orm::Database;
use tracing_subscriber::EnvFilter;

mod ap;
mod config;
mod dto;
mod entity;
mod entity_impl;
mod error;
mod handler;
mod object_store;
mod state;
mod util;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if crate::config::CONFIG.debug {
        tracing::warn!("Enabling debug mode... DO NOT USE IN PRODUCTION!");
    }

    let db = Database::connect(format!(
        "postgresql://{}:{}@{}:{}/{}",
        crate::config::CONFIG.database_user,
        crate::config::CONFIG.database_password,
        crate::config::CONFIG.database_host,
        crate::config::CONFIG.database_port,
        crate::config::CONFIG.database_database,
    ))
    .await
    .context("failed to connect to database")?;

    migration::Migrator::up(&db, None)
        .await
        .context("failed to migrate database")?;

    let state = crate::state::State::new(db).context("failed to construct app state")?;
    let federation_config = FederationConfig::builder()
        .domain(&crate::config::CONFIG.domain)
        .app_data(state.clone())
        .debug(crate::config::CONFIG.debug)
        .build()
        .await
        .context("failed to build federation config")?;

    let router = crate::handler::create_router(federation_config)
        .await
        .context("failed to create router")?;

    let listen_addr = &crate::config::CONFIG.listen_addr;
    tracing::info!(%listen_addr, "starting http server...");
    axum::Server::bind(&listen_addr.parse()?)
        .serve(router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
