use std::env;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::{
	cors::get_cors_layer, session::get_session_layer, state::AppState, utils::is_debug,
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
		.with_state(app_state)
		.layer(get_session_layer())
		.layer(cors_layer)
		// TODO: I want to ignore traces for asset requests, e.g. /assets/chunk-SRMZVY4F.02115dd3.js
		.layer(TraceLayer::new_for_http())
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

#[cfg(test)]
pub(crate) fn get_test_router(app_state: AppState) -> Router {
	use crate::{
		config::session::get_test_session_layer, utils::test::fake_login_handler,
	};
	Router::new()
		.merge(mount(app_state.clone()))
		.route("/fake-login", axum::routing::post(fake_login_handler))
		.with_state(app_state.clone())
		.layer(get_test_session_layer())
}
