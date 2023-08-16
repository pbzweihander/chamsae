use activitypub_federation::config::FederationConfig;
use anyhow::Context;
use dotenvy::dotenv;
use migration::MigratorTrait;
use sea_orm::Database;
use stopper::Stopper;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod ap;
mod config;
mod dto;
mod entity;
mod entity_impl;
mod error;
mod fmt;
mod handler;
mod object_store;
mod queue;
mod state;
mod util;

async fn shutdown_signal(stopper: Stopper) {
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
    stopper.stop();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    if crate::config::CONFIG.debug {
        tracing::warn!("Enabling debug mode... DO NOT USE IN PRODUCTION!");
    }

    let db = Database::connect(crate::config::CONFIG.database_url.as_str())
        .await
        .context("failed to connect to database")?;

    migration::Migrator::up(&db, None)
        .await
        .context("failed to migrate database")?;

    let stopper = Stopper::new();
    let state = crate::state::State::new(db, stopper.clone())
        .await
        .context("failed to construct app state")?;
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
        .with_graceful_shutdown(shutdown_signal(stopper))
        .await?;

    Ok(())
}
