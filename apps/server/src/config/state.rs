use std::sync::Arc;

use stump_core::prelude::Ctx;

pub type AppState = Arc<Ctx>;

pub trait AppStateMock {
	fn _mock() -> AppState;
}

impl AppStateMock for AppState {
	fn _mock() -> AppState {
		Arc::new(Ctx::_mock())
	}
}
