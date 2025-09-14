use anyhow::Context;
use axum::Router;
use oms_types::{AppState, Config, config};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::env;
use tower::ServiceBuilder;
use tower_http::{
	compression::CompressionLayer, cors::CorsLayer, decompression::DecompressionLayer, services::ServeDir,
};

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
					std::fs::create_dir_all(data_dir.clone()).ok();

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

	let app = Router::new()
		.layer(
			ServiceBuilder::new()
				.layer(CorsLayer::new())
				.layer(CompressionLayer::new())
				.layer(DecompressionLayer::new()),
		)
		.fallback_service(ServeDir::new(if cfg!(debug_assertions) {
			"www/dist".to_string()
		} else {
			if cfg!(windows) {
				env::current_exe()?
					.parent()
					.unwrap()
					.join("www")
					.to_str()
					.context("Non-Unicode paths aren't supported")?
					.to_string()
			} else {
				#[cfg(all(not(debug_assertions), not(windows)))]
				return env!(
					"OMS_WWW_LOCATION",
					"OMS_WWW_LOCATION must be set for non-Windows release builds"
				)
				.to_string();
				unreachable!()
			}
		}));

	let _ = tokio::join!(
		spawn_server!(serve!(config.port, app)),
		spawn_server!(serve!(5335, oms_api::get_router(state)))
	);

	Ok(())
}
