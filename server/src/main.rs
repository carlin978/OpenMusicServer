use anyhow::Context;
use axum::{Router, routing::get};

mod config;

#[tokio::main]
async fn main() {
    let config = config::load_config()
        .context("Failed to load config")
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api", oms_api::get_api_router());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
