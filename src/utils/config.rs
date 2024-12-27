use chrono::{Duration, TimeDelta};
use dotenv::dotenv;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
pub const OPENID_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
#[derive(Clone)]
pub struct Environment {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_access_expired: u64,
    pub jwt_refresh_expired: u64,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub smtp_sender_email: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub crypto_data_database_url: String,
}
impl Environment {
    pub fn default() -> Self {
        dotenv().ok();
        let client_id = env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or("".into());
        let redis_url = env::var("REDIS_URL").unwrap_or("".into());
        let client_secret = env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap_or("".into());
        let crypto_data_database_url = env::var("CRYPTO_DATA_DATABASE_URL").unwrap_or("".into());
        let redirect_url = env::var("REDIRECT_URL").unwrap_or("".into());
        let database_url = env::var("DATABASE_URL").unwrap_or("".into());
        let jwt_access_secret = env::var("JWT_ACCESS_TOKEN_SECRET").unwrap_or("".into());
        let jwt_refresh_secret = env::var("JWT_REFRESH_TOKEN_SECRET").unwrap_or("".into());
        let jwt_access_expired = env::var("JWT_ACCESS_TOKEN_EXPIRED_DATE")
            .unwrap_or("".into())
            .parse::<u64>()
            .unwrap_or(900);
        let jwt_refresh_expired = env::var("JWT_REFRESH_TOKEN_EXPIRED_DATE")
            .unwrap_or("".into())
            .parse::<u64>()
            .unwrap_or(604800);
        let smtp_sender_email = env::var("SMTP_SENDER_EMAIL").unwrap_or("".into());
        let smtp_username = env::var("SMTP_USERNAME").unwrap_or("".into());
        let smtp_password = env::var("SMTP_PASSWORD").unwrap_or("".into());
        Environment {
            client_id,
            client_secret,
            redirect_url,
            database_url,
            redis_url,
            jwt_access_expired,
            jwt_refresh_expired,
            jwt_access_secret,
            jwt_refresh_secret,
            smtp_sender_email,
            smtp_username,
            smtp_password,
            crypto_data_database_url,
        }
    }
}

pub fn subscribe_tracing() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true),
        )
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("sqlx=off".parse().unwrap())
                .add_directive("hyper=warn".parse().unwrap())
                .add_directive("axum=info".parse().unwrap()),
        )
        .init();
}
