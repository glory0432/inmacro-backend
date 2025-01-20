pub mod balance;
pub mod oauth;
pub mod user;
pub mod volume;
use std::sync::Arc;

use crate::AppState;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
pub fn create_router(state: Arc<AppState>) -> Router {
    let router = Router::new();
    let router = oauth::add_routers(router, state.clone());
    let router = user::add_routers(router, state.clone());
    let router = volume::add_routers(router, state.clone());
    let router = balance::add_routers(router, state.clone());
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let router = router.layer(cors);
    router.with_state(state)
}
