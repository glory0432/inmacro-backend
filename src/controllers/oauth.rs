use crate::{
    dto::{
        request::AuthRequest,
        response::{GoogleUserProfileResponse, JWTTokenResponse},
    },
    utils::{
        config::{AUTH_URL, OPENID_URL},
        errors::ApiError,
        jwt::generate_token_pair,
    },
    AppState,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use oauth2::{reqwest::async_http_client, AuthorizationCode, TokenResponse};
use std::sync::Arc;
pub async fn get_google_auth_url(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ()> {
    let auth_url = format!(
        "{}?response_type=code&client_id={}&include_granted_scopes=true&redirect_uri={}&scope=email%20profile",
        AUTH_URL, state.env.client_id, state.env.redirect_url
    );
    return Ok(auth_url);
}
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Json(query): Json<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let token = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await?;
    let ctx = reqwest::Client::new();
    let profile = ctx
        .get(OPENID_URL)
        .bearer_auth(token.access_token().secret().to_owned())
        .send()
        .await?;
    if profile.status().is_success() {
        let profile = profile.json::<GoogleUserProfileResponse>().await.unwrap();
        let user_id:(i64,) = sqlx::query_as("INSERT INTO users (email, auth_provider, full_name, profile_picture_url) VALUES ($1, $2, $3, $4) ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email RETURNING id")
            .bind(profile.email.clone())
            .bind("google")
            .bind(profile.name.clone())
            .bind(profile.picture.clone())
            .fetch_one(&state.db)
            .await?;
        let (access_token, refresh_token) = generate_token_pair(state, user_id.0)?;
        let response = Json(JWTTokenResponse {
            api_token: access_token,
            refreshToken: refresh_token,
            access_token: "".to_string(),
        })
        .into_response();
        return Ok(response);
    }
    return Err(ApiError::Unauthorized);
}
