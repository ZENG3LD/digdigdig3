//! Axum router: prefix-mode + encoded-mode proxy routes, `/health`, and a
//! permissive CORS layer (including preflight OPTIONS) applied to every
//! response, error responses included.

use std::collections::HashMap;

use axum::extract::{Path, RawQuery};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::Serialize;

use crate::error::ProxyError;
use crate::forward;
use crate::target::{self, Destination};

/// CORS headers applied to EVERY response — success, error, and preflight.
fn cors_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        axum::http::HeaderValue::from_static("*"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_METHODS,
        axum::http::HeaderValue::from_static("GET, HEAD, OPTIONS"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
        axum::http::HeaderValue::from_static("*"),
    );
    headers
}

fn with_cors(mut response: Response) -> Response {
    let cors = cors_headers();
    for (name, value) in cors.iter() {
        response.headers_mut().insert(name.clone(), value.clone());
    }
    response
}

#[derive(Serialize)]
struct HealthBody {
    ok: bool,
    version: &'static str,
    allowlist_size: usize,
}

async fn health() -> impl IntoResponse {
    let body = HealthBody {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
        allowlist_size: digdigdig3::core::proxy_allowlist::REST_HOST_ALLOWLIST.len(),
    };
    with_cors((StatusCode::OK, Json(body)).into_response())
}

/// Build the shared router: prefix-mode catch-all, encoded-mode `/proxy`,
/// and `/health`. Mounted by the binary's `main.rs` and reused directly by
/// tests (no network needed for `oneshot` calls).
pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/proxy", any(encoded_handler))
        .route("/*rest", any(prefix_handler))
}

/// Reject anything other than GET/HEAD (OPTIONS is answered separately by
/// each handler as a preflight response, never forwarded upstream).
fn reject_unless_get_or_head(method: &Method) -> Result<(), ProxyError> {
    if method == Method::GET || method == Method::HEAD {
        Ok(())
    } else {
        Err(ProxyError::MethodNotAllowed)
    }
}

fn preflight_response() -> Response {
    with_cors((StatusCode::NO_CONTENT, ()).into_response())
}

async fn do_relay(method: Method, dest: Destination, headers: HeaderMap) -> Response {
    let reqwest_method = match reqwest::Method::from_bytes(method.as_str().as_bytes()) {
        Ok(m) => m,
        Err(_) => return with_cors(ProxyError::MethodNotAllowed.into_response()),
    };
    match forward::relay(reqwest_method, &dest, &headers).await {
        Ok(response) => with_cors(response),
        Err(err) => with_cors(err.into_response()),
    }
}

/// Prefix mode: `/{host}/{*rest}?query`. `axum` gives us the raw remainder
/// path via the catch-all + the original URI for the query string.
async fn prefix_handler(
    method: Method,
    Path(rest_path): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    if method == Method::OPTIONS {
        return preflight_response();
    }
    if let Err(e) = reject_unless_get_or_head(&method) {
        return with_cors(e.into_response());
    }

    // rest_path is the full remainder after the leading slash, e.g.
    // "api.binance.com/api/v3/klines". First segment = host.
    let mut parts = rest_path.splitn(2, '/');
    let host = parts.next().unwrap_or_default();
    let rest = parts.next().unwrap_or("");
    let rest_with_slash = if rest.is_empty() {
        String::new()
    } else {
        format!("/{rest}")
    };

    let dest = match target::from_prefix(host, &rest_with_slash, query.as_deref()) {
        Ok(d) => d,
        Err(e) => return with_cors(e.into_response()),
    };

    do_relay(method, dest, headers).await
}

/// Encoded mode: `/proxy?url=<percent-encoded full URL>`.
async fn encoded_handler(method: Method, RawQuery(query): RawQuery, headers: HeaderMap) -> Response {
    if method == Method::OPTIONS {
        return preflight_response();
    }
    if let Err(e) = reject_unless_get_or_head(&method) {
        return with_cors(e.into_response());
    }

    let query = match query {
        Some(q) => q,
        None => {
            return with_cors(
                ProxyError::BadRequest("missing ?url= parameter".to_string()).into_response(),
            )
        }
    };

    let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let raw_url = match params.get("url") {
        Some(u) => u,
        None => {
            return with_cors(
                ProxyError::BadRequest("missing ?url= parameter".to_string()).into_response(),
            )
        }
    };

    let dest = match target::from_encoded(raw_url) {
        Ok(d) => d,
        Err(e) => return with_cors(e.into_response()),
    };

    do_relay(method, dest, headers).await
}
