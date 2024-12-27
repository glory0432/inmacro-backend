use std::sync::Arc;

use crate::controllers::user;
use crate::AppState;
use axum::routing::{get, post};

pub fn add_routers(
    router: axum::Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> axum::Router<Arc<AppState>> {
    router
        .route("/api/v1/login", post(user::login))
        .route("/api/v1/register", post(user::signup))
        .route("/api/v1/confirm", post(user::confirm))
        .route("/api/v1/user", get(user::user_info))
        .route("/api/v1/auth/forgot-password", post(user::forgot_password))
        .with_state(state)
}
