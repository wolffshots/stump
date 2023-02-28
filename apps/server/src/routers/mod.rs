use std::env;

use axum::Router;
use stump_core::db::models::User;

use crate::config::{
	cors::get_cors_layer,
	session::{get_session_layer, get_test_session_layer},
	state::AppState,
	utils::is_debug,
};

mod api;
mod opds;
mod spa;
mod sse;
mod utoipa;
mod ws;

pub(crate) fn get_app_router(app_state: AppState, port: u16) -> Router {
	let cors_layer = get_cors_layer(port);

	Router::new()
		.merge(mount(app_state.clone()))
		.with_state(app_state.clone())
		.layer(get_session_layer())
		.layer(cors_layer)
}

#[allow(unused)]
pub(crate) fn get_test_router(
	app_state: AppState,
	user_sessions: Option<Vec<User>>,
) -> Router {
	Router::new()
		.merge(mount(app_state.clone()))
		.with_state(app_state.clone())
		.layer(get_test_session_layer(user_sessions))
}

pub(crate) fn mount(app_state: AppState) -> Router<AppState> {
	let mut app_router = Router::new();

	let enable_swagger =
		env::var("ENABLE_SWAGGER_UI").unwrap_or_else(|_| String::from("true"));
	if enable_swagger != "false" || is_debug() {
		app_router = app_router.merge(utoipa::mount(app_state.clone()));
	}

	app_router
		.merge(spa::mount())
		.merge(ws::mount())
		.merge(sse::mount())
		.merge(api::mount(app_state.clone()))
		.merge(opds::mount(app_state))
}
