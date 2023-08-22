//! Implements the various supported forms of authentication.

use anyhow::Result;
use reqwest::Request;

use crate::auth::{oauth::OAuthClientCredentialsState, tls::TlsState};
pub use oauth::OAuthClientCredentialsOptions;
pub use tls::TlsOptions;

mod oauth;
mod tls;

pub enum AuthMechanism {
    None,
    OAuthClientCredentials(OAuthClientCredentialsState),
    Tls(TlsState),
}

impl AuthMechanism {
    pub fn oauth_client_credentials(options: OAuthClientCredentialsOptions) -> Self {
        AuthMechanism::OAuthClientCredentials(OAuthClientCredentialsState::new(options))
    }

    pub fn tls(options: TlsOptions) -> Self {
        AuthMechanism::Tls(TlsState::new(options))
    }
}

/// Authorize a request by adding the necessary header or certificate.
///
/// This function modifies the given request in some way to authorize it. The
/// nature of the modification depends on the authentication mechanism:
///
/// - `AuthMechanism::None`: No modification is made.
/// - `AuthMechanism::Oauth`: An authorization header is added to the request.
///   The header name and value are determined by the given `OAuthParams`. If a
///   header already exists on the request with the same name, it is replaced.
/// - `AuthMechanism::Tls`: A certificate and key are added to the request. The
///   certificate and key are determined by the given `TlsParams`.
pub async fn authorize_request(request: &mut Request, auth: &mut AuthMechanism) -> Result<()> {
    match auth {
        AuthMechanism::None => (),
        AuthMechanism::OAuthClientCredentials(state) => {
            oauth::authorize_request(request, state).await?;
        }
        AuthMechanism::Tls(_) => (),
    }

    Ok(())
}
