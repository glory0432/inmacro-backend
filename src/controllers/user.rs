use crate::{
    dto::{
        request::{ConfirmRequest, ForgotPasswordRequest, LoginRequest, SignupRequest},
        response::{JWTTokenResponse, UserInfoResponse},
    },
    utils::{
        errors::ApiError,
        jwt::{generate_token_pair, UserClaims},
        session,
        smtp::send_confirmation_code,
    },
    AppState,
};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use bcrypt::verify;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::sync::Arc;
use tracing::error;

#[derive(sqlx::FromRow, Debug)]
struct User {
    id: i64,
    password_hash: Option<String>,
    profile_picture_url: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE (email = $1) AND (auth_provider = $2)",
    )
    .bind(req.email)
    .bind("local")
    .fetch_all(&state.db)
    .await?;
    if user.len() != 1 || user[0].password_hash.is_none() {
        return Err(ApiError::LoginError);
    }
    if verify(req.password, &user[0].password_hash.clone().unwrap()).unwrap_or(false) {
        let (access_token, refresh_token) = generate_token_pair(state, user[0].id)?;
        let response = Json(JWTTokenResponse {
            api_token: access_token,
            refreshToken: refresh_token,
            access_token: "".to_string(),
        })
        .into_response();
        return Ok(response);
    }
    Err(ApiError::LoginError)
}
pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE (email = $1)")
        .bind(req.email.clone())
        .fetch_all(&state.db)
        .await?;
    if user.len() != 0 {
        return Err(ApiError::AlreadySignUp);
    }
    let confirmation_code: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    send_confirmation_code(req.email.clone(), confirmation_code.clone(), state.clone())?;
    let session_result = session::set(
        &state.redis,
        (
            &session::SessionKey::Email(session::EmailKey { email: req.email }),
            &session::SessionData::Confirmation(session::ConfirmationData {
                code: confirmation_code,
                password: req.password,
            }),
        ),
    )
    .await;
    match session_result {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::RedisSessionSetError),
    }
}
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE (email = $1)")
        .bind(req.email.clone())
        .fetch_all(&state.db)
        .await?;
    if user.len() == 0 {
        return Err(ApiError::NoEmailFound);
    }
    let id = uuid::Uuid::new_v4().to_string();
    let session_result = session::set(
        &state.redis,
        (
            &session::SessionKey::UUID(session::UUIDKey { uuid: id }),
            &session::SessionData::PasswordReset(session::PasswordResetData { email: req.email }),
        ),
    )
    .await;
    match session_result {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::RedisSessionSetError),
    }
}
pub async fn confirm(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfirmRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let session_result = session::get(
        &state.redis,
        &session::SessionKey::Email(session::EmailKey {
            email: req.email.clone(),
        }),
    )
    .await;

    match session_result {
        Err(_) | Ok(None) => Err(ApiError::InvalidConfirmationEmail),
        Ok(Some(session_data)) => match session_data {
            session::SessionData::Confirmation(data) => {
                if data.code != req.code {
                    return Err(ApiError::InvalidConfirmationCode);
                }
                let _ = session::del(
                    &state.redis,
                    &session::SessionKey::Email(session::EmailKey {
                        email: req.email.clone(),
                    }),
                )
                .await;

                let password_hash = bcrypt::hash(data.password, 12);
                if password_hash.is_err() {
                    error!(
                        "Failed to get the hash of password: {}",
                        password_hash.unwrap_err()
                    );
                    return Err(ApiError::SignupError);
                }
                let password_hash = password_hash.unwrap();
                sqlx::query(
                    "INSERT INTO users (email, full_name, password_hash) VALUES ($1, $2, $3)",
                )
                .bind(req.email.clone())
                .bind(req.email.clone())
                .bind(password_hash)
                .execute(&state.db)
                .await?;
                return Ok(());
            }
            _ => Err(ApiError::InvalidConfirmationEmail),
        },
    }
}
pub async fn user_info(
    State(state): State<Arc<AppState>>,
    user: UserClaims,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE (id = $1)")
        .bind(user.uid.clone())
        .fetch_all(&state.db)
        .await?;
    if user.len() != 1 {
        return Err(ApiError::Unauthorized);
    }
    let mut info = UserInfoResponse::default();
    info.pic = user[0].profile_picture_url.clone();
    let response = Json(info).into_response();
    return Ok(response);
}
