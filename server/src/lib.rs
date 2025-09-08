use anyhow::Context;
use axum::{Router, routing::get};
use oms_types::{AppState, Config, config};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

macro_rules! serve {
	($port:expr, $app:expr) => {{
		let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", $port))
			.await
			.unwrap();
		axum::serve(listener, $app)
	}};
}

macro_rules! spawn_server {
	($server:expr) => {
		tokio::task::spawn(async move {
			$server.await.unwrap();
		})
	};
}

#[tokio::main]
pub async fn init(config: Config) -> anyhow::Result<()> {
	let db = {
		let mut connect_options = SqliteConnectOptions::new();

		connect_options = if config.in_memory_database {
			connect_options.in_memory(true)
		} else {
			connect_options
				.filename({
					let data_dir =
						config::get_app_data_dir().context("Failed to get data directory from environment")?;

					#[cfg(not(debug_assertions))]
					std::fs::create_dir_all(data_dir.clone());

					data_dir.join("oms.db")
				})
				.create_if_missing(true)
		};

		let pool = SqlitePoolOptions::new()
			.connect_with(connect_options)
			.await
			.context("Failed to connect to SQLite database")?;

		sqlx::migrate!("../migrations").run(&pool).await?;

		pool
	};

	let state = AppState::new(db);

	let app = Router::new().route("/", get(async || "Hello, World!"));

	let _ = tokio::join!(
		spawn_server!(serve!(config.port, app)),
		spawn_server!(serve!(5335, oms_api::get_router(state)))
	);

	Ok(())
}
