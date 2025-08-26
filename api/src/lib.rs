use axum::Router;
use oms_types::AppState;

pub fn get_api_router() -> Router<AppState> {
	Router::new()
}
