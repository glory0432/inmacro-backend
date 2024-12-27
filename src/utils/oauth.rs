use crate::utils::config::{AUTH_URL, TOKEN_URL};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

pub fn build_oauth_client(
    client_id: String,
    client_secret: String,
    redirect_url: String,
) -> BasicClient {
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}
