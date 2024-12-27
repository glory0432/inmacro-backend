mod controllers;
mod dto;
mod routes;
mod utils;

use crate::utils::{config::*, oauth::build_oauth_client};
use axum::Router;
use oauth2::basic::BasicClient;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};
use utils::redis::{RedisClient, RedisClientBuilder};

#[derive(Clone)]
pub struct AppState {
    pub env: Environment,
    pub oauth_client: BasicClient,
    pub db: PgPool,
    pub redis: RedisClient,
    pub crypto_data_db: PgPool,
}

#[tokio::main]
async fn main() {
    subscribe_tracing();

    let env = Environment::default();
    let app_database = PgPool::connect(&env.clone().database_url)
        .await
        .map_err(|e| {
            error!("💥 Error to connect to the database: {}", e);
        })
        .unwrap();
    let crypto_data_database = PgPool::connect(&env.clone().crypto_data_database_url)
        .await
        .map_err(|e| {
            error!("💥 Error to connect to the crypto data database: {}", e);
        })
        .unwrap();
    info!("✔ Connected to the Database!");
    let app_redis = RedisClient::build_from_config(&env.redis_url)
        .map_err(|e| {
            error!("💥 Error in redis connection: {}", e);
        })
        .unwrap();
    info!("✔ Connected to the Redis!");
    let app_state = Arc::new(AppState {
        env: env.clone(),
        oauth_client: build_oauth_client(env.client_id, env.client_secret, env.redirect_url),
        db: app_database,
        redis: app_redis,
        crypto_data_db: crypto_data_database,
    });

    let app_router: Router = routes::create_router(app_state);
    let app_listener = TcpListener::bind("0.0.0.0:9000")
        .await
        .map_err(|e| {
            error!("💥 Error to bind: {}", e);
        })
        .unwrap();

    info!("🚀 Server running started...");
    axum::serve(app_listener, app_router)
        .await
        .map_err(|e| {
            error!("Error to run the server: {}", e);
        })
        .unwrap();
}
