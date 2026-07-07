//! Proxy error type + JSON error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Errors the proxy can surface to the client. Every variant maps to a
/// specific HTTP status and a small JSON body — never a bare 500 with no
/// context, the caller (browser wasm client) needs to distinguish "your
/// request was malformed" from "the destination host is not on the
/// allowlist" from "the upstream venue is unreachable".
#[derive(Debug)]
pub enum ProxyError {
    /// The request path / query did not encode a usable destination
    /// (missing host segment, malformed `?url=`, unparsable URL, ...).
    BadRequest(String),
    /// The extracted destination host is not a member of
    /// [`digdigdig3::core::proxy_allowlist::REST_HOST_ALLOWLIST`].
    HostNotAllowed(String),
    /// HTTP method other than GET/HEAD/OPTIONS.
    MethodNotAllowed,
    /// The upstream request failed (connect timeout, DNS, TLS, transport).
    UpstreamError(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    detail: String,
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, kind, detail) = match self {
            ProxyError::BadRequest(detail) => {
                (StatusCode::BAD_REQUEST, "bad_request", detail)
            }
            ProxyError::HostNotAllowed(host) => (
                StatusCode::FORBIDDEN,
                "host_not_allowed",
                format!("destination host '{host}' is not on the REST allowlist"),
            ),
            ProxyError::MethodNotAllowed => (
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "only GET, HEAD and OPTIONS are supported (market-data REST is read-only)"
                    .to_string(),
            ),
            ProxyError::UpstreamError(detail) => {
                (StatusCode::BAD_GATEWAY, "upstream_error", detail)
            }
        };
        let body = ErrorBody { error: kind, detail };
        (status, axum::Json(body)).into_response()
    }
}
