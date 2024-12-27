use crate::{utils::errors::*, AppState};
use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::{error, info};

pub static DECODE_HEADER: Lazy<Validation> = Lazy::new(|| Validation::default());
pub static ENCODE_HEADER: Lazy<Header> = Lazy::new(|| Header::default());

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct UserClaims {
    pub iat: i64,
    pub exp: i64,
    pub uid: i64,
}

impl UserClaims {
    pub fn new(duration: Duration, user_id: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self {
            iat: now,
            exp: now + duration.as_secs() as i64,
            uid: user_id,
        }
    }

    pub fn decode(token: &str, key: &str) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(key.as_ref()),
            &DECODE_HEADER,
        )
    }

    pub fn encode(&self, key: &str) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(
            &ENCODE_HEADER,
            self,
            &EncodingKey::from_secret(key.as_ref()),
        )
    }
}

pub fn generate_token_pair(
    state: Arc<AppState>,
    user_id: i64,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    info!("Generating token pair for user_id: {}", user_id);

    let access_token = UserClaims::new(Duration::from_secs(state.env.jwt_access_expired), user_id)
        .encode(&state.env.jwt_access_secret)?;

    let refresh_token =
        UserClaims::new(Duration::from_secs(state.env.jwt_refresh_expired), user_id)
            .encode(&state.env.jwt_refresh_secret)?;

    info!("Successfully generated token pair for user_id: {}", user_id);

    Ok((access_token, refresh_token))
}

#[async_trait::async_trait]
impl FromRequestParts<Arc<AppState>> for UserClaims {
    type Rejection = ApiError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, ApiError> {
        info!("Extracting and decoding UserClaims from request parts");

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await?;

        let user_claims = UserClaims::decode(bearer.token(), &state.env.jwt_access_secret)?.claims;

        info!(
            "Successfully extracted and decoded UserClaims from token for user_id: {}",
            user_claims.uid
        );

        Ok(user_claims)
    }
}
