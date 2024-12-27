use axum::{http::StatusCode, response::IntoResponse, response::Response};
use thiserror::Error;
use tracing::error;

const INTERNAL_SERVER_ERROR: &str = "Internal Server Error";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("SQL error: {0}")]
    SQL(#[from] sqlx::Error),
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Email building failed: {0}")]
    EmailBuildError(#[from] lettre::error::Error),
    #[error("Failed to send email via smtp: {0}")]
    EmailSendError(#[from] lettre::transport::smtp::Error),
    #[error("{0}")]
    CustomError(#[from] std::io::Error),
    #[error("OAuth token error: {0}")]
    TokenError(
        #[from]
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),
    #[error("You're not authorized!")]
    Unauthorized,
    #[error("Failed to set the session data in redis")]
    RedisSessionSetError,
    #[error("Invalid Credential")]
    LoginError,
    #[error("Signup Failed")]
    SignupError,
    #[error("No email found")]
    NoEmailFound,
    #[error("Invalid Confirmation email")]
    InvalidConfirmationEmail,
    #[error("Invalid Confirmation code")]
    InvalidConfirmationCode,
    #[error("You are already signed up")]
    AlreadySignUp,
    #[error("Attempted to parse a number to an integer but errored out: {0}")]
    ParseIntError(#[from] std::num::TryFromIntError),
    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
    #[error("Failed to extract typed header: {0}")]
    TypedHeaderError(#[from] axum_extra::typed_header::TypedHeaderRejection),
    #[error("Failed to decode jwt token: {0}")]
    JWTDecodeError(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("{}", self);

        let response = match self {
            Self::SQL(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::CustomError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::RedisSessionSetError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::Request(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::EmailBuildError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::EmailSendError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::TokenError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Code Error: {}", INTERNAL_SERVER_ERROR.to_string()),
            ),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized!".to_string()),
            Self::NoEmailFound => (StatusCode::NOT_FOUND, "No Email Found!".to_string()),
            Self::InvalidConfirmationEmail => (
                StatusCode::NOT_FOUND,
                "Invalid Confirmation email!".to_string(),
            ),
            Self::InvalidConfirmationCode => (
                StatusCode::NOT_FOUND,
                "Invalid Confirmation code!".to_string(),
            ),
            Self::AlreadySignUp => (
                StatusCode::UNAUTHORIZED,
                "You're already signed up.".to_string(),
            ),
            Self::LoginError => (StatusCode::NOT_FOUND, "Invalid Credential".to_string()),
            Self::SignupError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Signup Failed".to_string(),
            ),
            Self::ParseIntError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::FromRequestPartsError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::TypedHeaderError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_SERVER_ERROR.to_string(),
            ),
            Self::JWTDecodeError(_) => (StatusCode::UNAUTHORIZED, "Invalid JWT Code".to_string()),
        };
        error!("StatusCode: {}, Error Message: {}", response.0, response.1);
        response.into_response()
    }
}
