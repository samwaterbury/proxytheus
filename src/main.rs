//! Entry point for the proxy server.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::{
    routing::{any, get},
    Extension, Router,
};
use clap::Parser;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber;

use crate::auth::{AuthMechanism, OAuthClientCredentialsOptions, TlsOptions};
use crate::routes::{health, metrics, SharedState};

mod auth;
mod routes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Host address to listen on
    #[arg(short = 'a', long, env = "HOST", default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, env = "PORT", default_value = "3000")]
    port: u16,

    /// Metrics endpoint to proxy to
    #[arg(short = 'e', long, env = "ENDPOINT")]
    endpoint: String,

    /// Oauth2 client ID
    #[arg(long, env = "OAUTH2_CLIENT_ID")]
    client_id: Option<String>,

    /// Oauth2 client secret
    #[arg(long, env = "OAUTH2_CLIENT_SECRET")]
    client_secret: Option<String>,

    /// Authorization endpoint
    #[arg(long, env = "OAUTH2_AUTH_URL")]
    auth_url: Option<String>,

    /// Token endpoint
    #[arg(long, env = "OAUTH2_TOKEN_URL")]
    token_url: Option<String>,

    /// Optional audience parameter to include in the token request
    #[arg(long, env = "OAUTH2_AUDIENCE")]
    audience: Option<String>,

    /// Name of the header to use for the access token
    #[arg(long, env = "OAUTH2_HEADER_NAME", default_value = "Authorization")]
    header_name: String,

    /// Header format for the access token to be included in the request
    #[arg(long, env = "OAUTH2_HEADER_FORMAT", default_value = "Bearer {}")]
    header_value: String,

    /// Contents of the TLS certificate
    #[arg(long, env = "TLS_CERT")]
    cert: Option<String>,

    /// Filepath to the TLS certificate
    #[arg(long, env = "TLS_CERT_FILE")]
    cert_file: Option<String>,

    /// Contents of the TLS key
    #[arg(long, env = "TLS_KEY")]
    key: Option<String>,

    /// Filepath to the TLS key
    #[arg(long, env = "TLS_KEY_FILE")]
    key_file: Option<String>,
}

/// Determine the authentication method to use based on the given arguments.
fn determine_auth(args: Args) -> AuthMechanism {
    match args {
        Args {
            client_id: None,
            client_secret: None,
            auth_url: None,
            token_url: None,
            audience: None,
            cert: None,
            cert_file: None,
            key: None,
            key_file: None,
            ..
        } => {
            info!("No authentication configured.");
            AuthMechanism::None
        }
        Args {
            client_id: Some(client_id),
            client_secret: Some(client_secret),
            auth_url: Some(auth_url),
            token_url: Some(token_url),
            audience,
            header_name,
            header_value,
            ..
        } => {
            info!("OAuth2 client credentials authentication configured.");
            AuthMechanism::oauth_client_credentials(OAuthClientCredentialsOptions {
                client_id,
                client_secret,
                auth_url,
                token_url,
                audience,
                header_name: header_name.to_string(),
                header_value: header_value.to_string(),
            })
        }
        Args {
            cert: Some(cert),
            key: Some(key),
            ..
        } => {
            info!("TLS authentication configured.");
            AuthMechanism::tls(TlsOptions { cert: cert, key })
        }
        Args {
            cert_file: Some(cert_file),
            key_file: Some(key_file),
            ..
        } => {
            info!("TLS authentication configured.");
            AuthMechanism::tls(TlsOptions::from_files(cert_file, key_file))
        }
        _ => panic!("Invalid arguments"),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Configure tracing
    tracing_subscriber::fmt::init();

    // Extract the address to listen on
    let host: Ipv4Addr = args.address.parse().expect("Invalid host address.");
    let addr = SocketAddr::from((host, args.port));
    let endpoint = args.endpoint.clone();

    // Determine the auth mechanism to use
    let auth = determine_auth(args);

    // For TLS authentication, we need to add the cert and key to the client
    let http_client = match &auth {
        AuthMechanism::Tls(state) => reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .use_rustls_tls()
            .identity(state.identity())
            .build()
            .unwrap(),
        _ => reqwest::Client::new(),
    };

    // Build the router
    let state = SharedState {
        endpoint,
        auth: Arc::new(Mutex::new(auth)),
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/*path", any(metrics))
        .layer(Extension(http_client))
        .with_state(state);

    // Start the server
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
