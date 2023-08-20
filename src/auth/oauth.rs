use anyhow::Result;
use oauth2::{
    basic::BasicClient, reqwest::http_client, AuthUrl, ClientId, ClientSecret, TokenResponse,
    TokenUrl,
};
use reqwest::{
    header::{HeaderName, HeaderValue},
    Request,
};
use time::OffsetDateTime;
use tracing::debug;

/// Arguments for OAuth client credentials flow authentication.
pub struct OAuthClientCredentialsOptions {
    // Used to generate the token
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub audience: Option<String>,

    // Used to authenticate the request
    pub header_name: String,
    pub header_value: String,
}

#[derive(Clone)]
struct CachedToken {
    token: String,
    expires_at: OffsetDateTime,
}

pub struct OAuthClientCredentialsState {
    // Used to generate the token
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
    audience: Option<String>,

    // Used to authenticate the request
    header_name: String,
    header_value: String,

    // Cached token
    token: Option<CachedToken>,
}

impl OAuthClientCredentialsState {
    pub fn new(options: OAuthClientCredentialsOptions) -> Self {
        Self {
            client_id: options.client_id,
            client_secret: options.client_secret,
            auth_url: options.auth_url,
            token_url: options.token_url,
            audience: options.audience,
            header_name: options.header_name,
            header_value: options.header_value,
            token: None,
        }
    }

    async fn generate_token(&mut self) -> Result<CachedToken> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_url.clone())?,
            Some(TokenUrl::new(self.token_url.clone())?),
        );

        // Exchange the client credentials for a token
        let mut token_request = client.exchange_client_credentials();
        if Some(self.audience.clone()) != None {
            token_request =
                token_request.add_extra_param("audience", self.audience.clone().unwrap());
        }
        let token_result = token_request.request(http_client)?;

        // If no expiry is set, default to 1 hour from now
        let expires_in = token_result
            .expires_in()
            .unwrap_or(std::time::Duration::from_secs(3600));

        let token = CachedToken {
            token: token_result.access_token().secret().to_string(),
            expires_at: OffsetDateTime::now_utc() + expires_in,
        };

        debug!("Generated new token expiring at {:?}", token.expires_at);
        Ok(token)
    }

    async fn token(&mut self) -> Result<String> {
        match self {
            Self {
                token: Some(CachedToken { expires_at, token }),
                ..
            } if expires_at > &mut OffsetDateTime::now_utc() => Ok(token.clone()),
            _ => {
                let token = self.generate_token().await?;
                self.token = Some(token.clone());
                Ok(token.token)
            }
        }
    }
}

/// Authorize a request by adding a header with the OAuth token.
///
/// This function modifies the given request by adding an authorization header
/// with the OAuth token. If the token has expired, a new one is generated.
pub async fn authorize_request(
    request: &mut Request,
    state: &mut OAuthClientCredentialsState,
) -> Result<()> {
    let token = state.token().await?;

    request.headers_mut().insert(
        HeaderName::from_bytes(state.header_name.as_bytes()).unwrap(),
        HeaderValue::from_str(&state.header_value.replace("{}", &token))?,
    );

    Ok(())
}
