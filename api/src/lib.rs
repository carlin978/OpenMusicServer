use axum::Router;
use oms_types::AppState;

pub fn get_router(state: AppState) -> Router<()> {
	Router::new().with_state(state)
}
