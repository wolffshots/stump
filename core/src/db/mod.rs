pub(crate) mod dao;
pub mod migration;
pub mod models;
pub mod utils;

pub use dao::*;
use prisma_client_rust::MockStore;

use tracing::trace;

use crate::{
	config::get_config_dir,
	prisma::{self, PrismaClient},
};

/// Creates the PrismaClient. Will call `create_data_dir` as well
pub async fn create_client() -> PrismaClient {
	let config_dir = get_config_dir()
		.to_str()
		.expect("Error parsing config directory")
		.to_string();

	let profile = std::env::var("STUMP_PROFILE").unwrap_or_else(|_| "debug".to_string());
	let db_override = std::env::var("STUMP_DB_PATH").ok();

	if let Some(path) = db_override {
		create_client_with_url(&format!("file:{}/stump.db", &path)).await
	} else if profile == "release" {
		trace!(
			"Creating Prisma client with url: file:{}/stump.db",
			&config_dir
		);
		prisma::new_client_with_url(&format!("file:{}/stump.db", &config_dir))
			.await
			.expect("Failed to create Prisma client")
	} else {
		trace!(
			"Creating Prisma client with url: file:{}/prisma/dev.db",
			&env!("CARGO_MANIFEST_DIR")
		);
		create_client_with_url(&format!(
			"file:{}/prisma/dev.db",
			&env!("CARGO_MANIFEST_DIR")
		))
		.await
	}
}

pub async fn create_client_with_url(url: &str) -> PrismaClient {
	prisma::new_client_with_url(url)
		.await
		.expect("Failed to create Prisma client")
}

// TODO: I'll have to rework the integration test mock because I do want real DB action
// there I believe? We'll see, perhaps I don't :D
pub fn create_mocked_prisma() -> (PrismaClient, MockStore) {
	// let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("integration-tests");
	// create_client_with_url(&format!("file:{}/test.db", test_dir.to_str().unwrap())).await
	PrismaClient::_mock()
}
