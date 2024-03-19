use anyhow::Context;
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tower_http::compression::{
    predicate::NotForContentType, CompressionLayer, DefaultPredicate, Predicate,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod database;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    database::init_tables()
        .await
        .context("error while initializing database tables")?;

    let port = crate::utils::env::get_port();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(&addr)
        .await
        .context("error while binding to port")?;
    let app = routes::main_router()
        // add request url to response for logger to use
        .layer(axum::middleware::from_fn(
            middleware::logging::uri_middleware,
        ))
        // add custom logging format
        .layer(TraceLayer::new_for_http().on_response(middleware::logging::logging_middleware))
        // add gzip compression
        .layer(CompressionLayer::new().gzip(true).compress_when(
            DefaultPredicate::new().and(NotForContentType::new("application/json")),
        ));

    axum::serve(listener, app.into_make_service())
        .await
        .context("error while starting API server")?;

    tracing::info!("Server srarted");
    anyhow::Ok(())
}
