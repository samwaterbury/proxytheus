//! Route handlers for the server.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{header::HeaderMap, Method, Response, StatusCode},
    Extension,
};
use reqwest;
use tokio::sync::Mutex;
use tracing::{debug, error};

use crate::auth::{authorize_request, AuthMechanism};

#[derive(Clone)]
pub struct SharedState {
    pub auth: Arc<Mutex<AuthMechanism>>,
    pub endpoint: String,
}

/// Health check endpoint.
pub async fn health() {
    debug!("Health check success.");
}

/// Metrics endpoint.
///
/// This endpoint proxies the metrics endpoint from the configured metrics
/// endpoint and adds any required authentication details. The response is
/// returned unaltered.
pub async fn metrics(
    Extension(http_client): Extension<reqwest::Client>,
    State(state): State<SharedState>,
    method: Method,
    headers: HeaderMap,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Response<Body>, StatusCode> {
    debug!("Metrics request received at path: {}", path);

    // The first path segment should be `metrics`. After that there may be zero
    // or more additional path segments which will be added to the base target
    // endpoint URL. This effectively causes the metrics proxy to be "mounted"
    // at `/metrics` on the server.
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() < 1 || parts[0] != "metrics" {
        error!("Invalid path: {}", path);
        return Err(StatusCode::NOT_FOUND);
    }
    let destination = if parts.len() == 1 {
        state.endpoint.clone()
    } else {
        let mut segments = parts.clone();
        segments.remove(0);
        construct_url(&state.endpoint, &segments)
    };

    // Clone the incoming request
    let mut request = http_client
        .request(method, destination)
        .headers(headers)
        .query(&params)
        .body(body.to_vec())
        .build()
        .map_err(|e| {
            error!("Error building request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Modify the request to include the required authentication
    let mut auth = state.auth.lock().await;
    authorize_request(&mut request, &mut auth)
        .await
        .map_err(|e| {
            error!("Error authorizing request: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Drop the lock on the auth mechanism mutex
    drop(auth);

    // Send the request and mirror the response
    match http_client.execute(request).await {
        Ok(response) => Ok(Response::builder()
            .status(response.status())
            .body(Body::from(response.bytes().await.unwrap()))
            .unwrap()),
        Err(e) => {
            error!("Error sending request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Construct a URL from a base and path.
///
/// The base URL is the target metrics URL being proxied. The path is the path
/// of the incoming request. The path will be appended to the base URL to form
/// the final URL.
fn construct_url(base: &str, parts: &[&str]) -> String {
    let mut url = reqwest::Url::parse(base).unwrap();
    url.path_segments_mut().unwrap().extend(parts);
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_url() {
        assert_eq!(
            construct_url("http://some.endpoint", &vec!["metrics"]),
            "http://some.endpoint/metrics"
        );
        assert_eq!(
            construct_url("http://some.endpoint", &vec!["metrics", "sub", "path"]),
            "http://some.endpoint/metrics/sub/path"
        );
        assert_eq!(
            construct_url("http://some.endpoint/suffix", &vec!["metrics"]),
            "http://some.endpoint/suffix/metrics"
        );
        assert_eq!(
            construct_url(
                "http://some.endpoint/suffix",
                &vec!["metrics", "sub", "path"]
            ),
            "http://some.endpoint/suffix/metrics/sub/path"
        );
    }
}
