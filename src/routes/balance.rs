use std::sync::Arc;

use crate::controllers::balance;
use crate::AppState;
use axum::routing::get;

pub fn add_routers(
    router: axum::Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> axum::Router<Arc<AppState>> {
    router
        .route("/api/v1/balance", get(balance::get_balance_data))
        .route(
            "/api/v1/balance/latest",
            get(balance::get_latest_balance_data),
        )
        .with_state(state)
}
