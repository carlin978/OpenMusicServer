use serde::Deserialize;
use std::sync::{Arc, Mutex};

pub mod config;
pub mod tasker;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
	pub port: u16,
	#[serde(default)]
	pub in_memory_database: bool,
}

#[derive(Clone)]
pub struct AppState {
	pub db: sqlx::Pool<sqlx::Sqlite>,
	pub tasker: Arc<Mutex<tasker::ThreadedTaskRunner>>,
}
impl AppState {
	pub fn new(db: sqlx::Pool<sqlx::Sqlite>) -> Self {
		Self {
			db,
			tasker: Arc::new(Mutex::new(tasker::ThreadedTaskRunner::new())),
		}
	}
}
