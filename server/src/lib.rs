use anyhow::Context;
use axum::{Router, routing::get};
use oms_types::{AppState, Config, config};
use sqlx::sqlite::SqlitePoolOptions;

macro_rules! serve {
	($port:expr, $app:expr) => {{
		let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", $port))
			.await
			.unwrap();
		axum::serve(listener, $app)
	}};
}

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

	let app = Router::new().route("/", get(async || "Hello, World!"));

	let _ = tokio::join!(serve!(config.port, app), serve!(5335, oms_api::get_router(state)));

	Ok(())
}
