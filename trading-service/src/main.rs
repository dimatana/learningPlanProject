mod api_impl;
mod config;
mod domain;
mod error;
mod kafka_consumer;
mod repository;
mod state;

use crate::api_impl::ApiImpl;
use crate::config::Config;
use crate::state::AppState;
use service_common::{docs_router, shutdown_signal};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .min_connections(config.database_min_connections)
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").context("KAFKA_BROKERS environment variable is not set")?;

    let consumer_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(err) = kafka_consumer::run(consumer_pool, kafka_brokers).await {
            warn!(error = %err, "kafka consumer task exited");
        }
    });

    let state = AppState { pool };
    let api_impl = ApiImpl::new(state);

    let app = trading_api_generated::server::new(api_impl)
        .merge(docs_router(
            include_str!("../openapi.yaml"),
            include_str!("swagger_ui.html"),
        ))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .context("failed to bind")?;

    info!(bind_addr = %config.bind_addr, "HTTP server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}
