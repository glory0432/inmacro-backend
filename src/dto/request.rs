use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GetVolumeDataRequest {
    pub symbol: String,
    pub interval: String,
    pub exchange_id: Option<i64>,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct Get24HVolumeDataRequest {
    pub symbol: String,
    pub interval: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}
