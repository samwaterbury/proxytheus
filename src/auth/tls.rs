use anyhow::Result;
use reqwest::Request;

/// Arguments for TLS client certificate authentication.
pub struct TlsOptions {
    pub cert: String,
    pub key: String,
}

pub struct TlsState {
    cert: String,
    key: String,
}

impl TlsState {
    pub fn new(args: TlsOptions) -> Self {
        Self {
            cert: args.cert,
            key: args.key,
        }
    }
}

/// Authorize a request by adding the TLS client certificate and key.
pub fn authorize_request(request: &mut Request, state: &TlsState) -> Result<()> {
    request.headers_mut().insert(
        "X-Forwarded-Client-Cert",
        state.cert.clone().parse().unwrap(),
    );
    request
        .headers_mut()
        .insert("X-Forwarded-Client-Key", state.key.clone().parse().unwrap());

    Ok(())
}
