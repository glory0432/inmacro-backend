use std::sync::Arc;

use crate::controllers::oauth;
use crate::AppState;
use axum::routing::{get, post};

pub fn add_routers(
    router: axum::Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> axum::Router<Arc<AppState>> {
    router
        .route(
            "/api/v1/auth/google/signin",
            get(oauth::get_google_auth_url),
        )
        .route("/api/v1/auth/google/callback", post(oauth::oauth_callback))
        .with_state(state)
}
