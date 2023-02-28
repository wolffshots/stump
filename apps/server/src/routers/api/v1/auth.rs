use axum::{
	extract::State,
	routing::{get, post},
	Json, Router,
};
use axum_sessions::extractors::{ReadableSession, WritableSession};
use stump_core::{
	db::models::User,
	prelude::{LoginOrRegisterArgs, UserRole},
	prisma::{user, user_preferences},
};

use crate::{
	config::state::AppState,
	errors::{ApiError, ApiResult},
	utils::{hash_password, verify_password},
};

pub(crate) fn mount() -> Router<AppState> {
	Router::new().nest(
		"/auth",
		Router::new()
			.route("/me", get(viewer))
			.route("/login", post(login))
			.route("/logout", post(logout))
			.route("/register", post(register)),
	)
}

#[utoipa::path(
	get,
	path = "/api/v1/auth/me",
	tag = "auth",
	responses(
		(status = 200, description = "Returns the currently logged in user from the session.", body = User),
		(status = 401, description = "No user is logged in (unauthorized).")
	)
)]
/// Returns the currently logged in user from the session. If no user is logged in, returns an
/// unauthorized error.
async fn viewer(session: ReadableSession) -> ApiResult<Json<User>> {
	if let Some(user) = session.get::<User>("user") {
		Ok(Json(user))
	} else {
		Err(ApiError::Unauthorized)
	}
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/login",
	tag = "auth",
	request_body = LoginOrRegisterArgs,
	responses(
		(status = 200, description = "Authenticates the user and returns the user object.", body = User),
		(status = 401, description = "Invalid username or password."),
		(status = 500, description = "An internal server error occurred.")
	)
)]
/// Authenticates the user and returns the user object. If the user is already logged in, returns the
/// user object from the session.
async fn login(
	mut session: WritableSession,
	State(state): State<AppState>,
	Json(input): Json<LoginOrRegisterArgs>,
) -> ApiResult<Json<User>> {
	if let Some(user) = session.get::<User>("user") {
		if input.username == user.username {
			return Ok(Json(user));
		}
	}

	let fetched_user = state
		.db
		.user()
		.find_unique(user::username::equals(input.username.to_owned()))
		.with(user::user_preferences::fetch())
		.exec()
		.await?;

	if let Some(db_user) = fetched_user {
		let matches = verify_password(&db_user.hashed_password, &input.password)?;
		if !matches {
			return Err(ApiError::Unauthorized);
		}

		let user: User = db_user.into();
		session
			.insert("user", user.clone())
			.expect("Failed to write user to session");

		return Ok(Json(user));
	}

	Err(ApiError::Unauthorized)
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/logout",
	tag = "auth",
	responses(
		(status = 200, description = "Destroys the session and logs the user out."),
		(status = 500, description = "Failed to destroy session.")
	)
)]
/// Destroys the session and logs the user out.
async fn logout(mut session: WritableSession) -> ApiResult<()> {
	session.destroy();
	if !session.is_destroyed() {
		return Err(ApiError::InternalServerError(
			"Failed to destroy session".to_string(),
		));
	}
	Ok(())
}

#[utoipa::path(
	post,
	path = "/api/v1/auth/register",
	tag = "auth",
	request_body = LoginOrRegisterArgs,
	responses(
		(status = 200, description = "Successfully registered new user.", body = User),
		(status = 403, description = "Must be server owner to register member accounts."),
		(status = 500, description = "An internal server error occurred.")
	)
)]
/// Attempts to register a new user. If no users exist in the database, the user is registered as a server owner.
/// Otherwise, the registration is rejected by all users except the server owner.
pub async fn register(
	session: ReadableSession,
	State(ctx): State<AppState>,
	Json(input): Json<LoginOrRegisterArgs>,
) -> ApiResult<Json<User>> {
	let db = ctx.get_db();

	let has_users = db.user().find_first(vec![]).exec().await?.is_some();

	let mut user_role = UserRole::default();

	let session_user = session.get::<User>("user");

	// TODO: move nested if to if let once stable
	if let Some(user) = session_user {
		if !user.is_admin() {
			return Err(ApiError::Forbidden(String::from(
				"You do not have permission to access this resource.",
			)));
		}
	} else if session_user.is_none() && has_users {
		// if users exist, a valid session is required to register a new user
		return Err(ApiError::Unauthorized);
	} else if !has_users {
		// if no users present, the user is automatically a server owner
		user_role = UserRole::ServerOwner;
	}

	let hashed_password = hash_password(&input.password)?;
	let created_user = db
		.user()
		.create(
			input.username.to_owned(),
			hashed_password,
			vec![user::role::set(user_role.into())],
		)
		.exec()
		.await?;

	// FIXME: these next two queries will be removed once nested create statements are
	// supported on the prisma client. Until then, this ugly mess is necessary.
	let _user_preferences = db
		.user_preferences()
		.create(vec![user_preferences::user::connect(user::id::equals(
			created_user.id.clone(),
		))])
		.exec()
		.await?;

	// This *really* shouldn't fail, so I am using expect here. It also doesn't
	// matter too much in the long run since this query will go away once above fixme
	// is resolved.
	let user = db
		.user()
		.find_unique(user::id::equals(created_user.id))
		.with(user::user_preferences::fetch())
		.exec()
		.await?
		.expect("Failed to fetch user after registration.");

	Ok(Json(user.into()))
}

#[cfg(test)]
mod tests {
	use axum::http::StatusCode;
	use axum_test_helper::TestClient;
	use serde_json::json;
	use stump_core::{db::models::User, prisma::user};

	use crate::{
		config::state::{AppState, AppStateMock},
		routers::get_test_router,
		utils::hash_password,
	};

	fn fake_user(username: &str) -> user::Data {
		user::Data {
			id: "test".to_string(),
			username: username.to_string(),
			hashed_password: hash_password("test").unwrap(),
			role: "test".to_string(),
			user_preferences: None,
			read_progresses: None,
			reading_lists: None,
			user_preferences_id: None,
		}
	}

	#[tokio::test]
	async fn viewer_with_valid_session() {
		let mock_state = AppState::_mock();
		let router = get_test_router(mock_state.clone(), None);
		let app = TestClient::new(router);
		// FIXME: this is private! I need to figure out how to fake a session...
		// let session_handle = session_layer.load_or_create(cookie_value.clone()).await;
		todo!()
	}

	#[tokio::test]
	async fn viewer_without_valid_session() {
		todo!()
	}

	#[tokio::test]
	async fn login_user_without_failure() {
		let mock_state = AppState::_mock();
		let router = get_test_router(mock_state.clone(), None);
		let app = TestClient::new(router);

		let mock_store = mock_state.mock_store.clone().unwrap();
		let client = mock_state.db.clone();

		mock_store
			.expect(
				client
					.user()
					.find_unique(user::username::equals("test".to_string()))
					.with(user::user_preferences::fetch()),
				Some(fake_user("test")),
			)
			.await;

		let res = app
			.post("/api/v1/auth/login")
			.json(&json!({"username": "test", "password": "password"}))
			.send()
			.await;

		assert_eq!(res.status(), StatusCode::OK);

		let res_body = res.json::<User>().await;
		assert_eq!(&res_body.username, "test");
	}

	#[tokio::test]
	async fn login_user_with_user_not_found_failure() {
		let mock_state = AppState::_mock();
		let router = get_test_router(mock_state.clone(), None);
		let app = TestClient::new(router);

		let mock_store = mock_state.mock_store.clone().unwrap();
		let client = mock_state.db.clone();

		mock_store
			.expect(
				client
					.user()
					.find_unique(user::username::equals("test".to_string()))
					.with(user::user_preferences::fetch()),
				None,
			)
			.await;

		let res = app
			.post("/api/v1/auth/login")
			.json(&json!({"username": "test", "password": "password"}))
			.send()
			.await;

		assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
	}

	#[tokio::test]
	async fn logout_user_wihout_failure() {
		todo!()
	}

	#[tokio::test]
	async fn logout_user_with_failure() {
		todo!()
	}

	#[tokio::test]
	async fn register_user_without_failure() {
		todo!()
	}

	#[tokio::test]
	async fn register_user_with_forbidden_failure() {
		todo!()
	}

	#[tokio::test]
	async fn register_user_with_unauthorized_failure() {
		todo!()
	}

	#[tokio::test]
	async fn register_user_with_unexpected_failure() {
		todo!()
	}
}
