use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize)]
pub struct GoogleUserProfileResponse {
    pub email: String,
    pub picture: String,
    pub name: String,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct JWTTokenResponse {
    pub api_token: String,
    pub refreshToken: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UserInfoResponse {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub pic: String,
}
