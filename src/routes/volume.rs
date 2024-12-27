use std::sync::Arc;

use crate::controllers::volume;
use crate::AppState;
use axum::routing::{get, post};

pub fn add_routers(
    router: axum::Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> axum::Router<Arc<AppState>> {
    router
        .route("/api/v1/volume", get(volume::get_volume_data))
        .route("/api/v1/24hr", get(volume::get_24hr_volume_data))
        .with_state(state)
}
