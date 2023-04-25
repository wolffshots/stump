use axum::Json;
use axum_sessions::extractors::WritableSession;
use axum_test_helper::TestClient;
use serde::{Deserialize, Serialize};
use stump_core::{db::models::User, prelude::UserRole, prisma::user};

use crate::errors::ApiResult;

use super::hash_password;

pub(crate) fn fake_prisma_user(username: &str) -> user::Data {
	user::Data {
		id: "fake".to_string(),
		username: username.to_string(),
		hashed_password: hash_password("password").unwrap(),
		role: UserRole::Member.to_string(),
		user_preferences: None,
		read_progresses: None,
		reading_lists: None,
		user_preferences_id: None,
		avatar_url: None,
		media_annotations: None,
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FakeLogin {
	is_server_owner: Option<bool>,
}

pub(crate) async fn fake_login_handler(
	mut session: WritableSession,
	Json(input): Json<FakeLogin>,
) -> ApiResult<Json<User>> {
	let is_server_owner = input.is_server_owner.unwrap_or(false);
	let role = if is_server_owner {
		UserRole::ServerOwner
	} else {
		UserRole::Member
	}
	.to_string();

	let user = User {
		username: String::from("fake"),
		role,
		..Default::default()
	};

	session
		.insert("user", user.clone())
		.expect("Failed to insert test user into session");

	Ok(Json(user))
}

pub(crate) async fn fake_login(test_client: &TestClient, is_server_owner: bool) {
	let fake_login = FakeLogin {
		is_server_owner: Some(is_server_owner),
	};

	let response = test_client
		.post("/fake-login")
		.json(&fake_login)
		.send()
		.await;

	assert_eq!(response.status(), 200);
}
