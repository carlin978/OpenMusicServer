use anyhow::Context;
use axum::{Router, routing::get};
use oms_types::{AppState, Config, config};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
pub async fn init(config: Config) -> anyhow::Result<()> {
	let db = {
		let db_url = if config.in_memory_database {
			"sqlite::memory:".to_string()
		} else {
			let path = config::get_app_data_dir()
				.context("Failed to get app data directory from environment")?
				.join("oms.db");

			format!(
				"sqlite:{}",
				path.to_str().context("Non-unicode paths aren't supported")?
			)
		};

		SqlitePoolOptions::new()
			.connect(&db_url)
			.await
			.context("Failed to connect to SQLite database")?
	};

	let state = AppState::new(db);

	let app = Router::new()
		.route("/", get(|| async { "Hello, World!" }))
		.nest("/api", oms_api::get_router())
		.with_state(state);

	let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
		.await
		.unwrap();
	axum::serve(listener, app).await.unwrap();

	Ok(())
}
