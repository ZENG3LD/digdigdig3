//! Upstream forwarding: build the reqwest request, stream the response body
//! back without buffering, strip hop-by-hop headers, never forward client
//! auth/cookies upstream.

use std::sync::OnceLock;
use std::time::Duration;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use reqwest::Client;

use crate::error::ProxyError;
use crate::target::Destination;

/// Distinct User-Agent so upstream venues can identify this proxy in their
/// own logs / rate-limit dashboards, instead of masquerading as a browser.
pub const PROXY_USER_AGENT: &str = concat!("digdigdig3-proxy/", env!("CARGO_PKG_VERSION"));

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(60);

/// Hop-by-hop headers per RFC 7230 §6.1 plus a few proxy-specific ones that
/// must never be blindly forwarded in either direction.
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "host",
];

/// Client-side headers that must NEVER be forwarded upstream: cookies and
/// auth are the caller's business with the proxy (there is none — no auth
/// here), not the destination venue's.
const STRIP_FROM_CLIENT_REQUEST: &[&str] = &["cookie", "authorization", "origin", "referer"];

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

/// Shared upstream client — built once and reused so connections to the
/// same venue get pooled instead of a fresh TLS handshake per request.
/// `Client::builder().build()` only fails on TLS backend init, which cannot
/// happen here (rustls-tls is compiled in and never user-configurable), so
/// `Client::new()` is a safe fallback that never actually triggers.
fn http_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TOTAL_TIMEOUT)
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.iter().any(|h| h.eq_ignore_ascii_case(name))
}

/// Strip hop-by-hop + auth/cookie headers from a header map, returning only
/// what is safe to forward.
pub fn sanitize_request_headers(headers: &HeaderMap) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in headers.iter() {
        let lower = name.as_str();
        if is_hop_by_hop(lower) || STRIP_FROM_CLIENT_REQUEST.iter().any(|s| s.eq_ignore_ascii_case(lower)) {
            continue;
        }
        out.append(name.clone(), value.clone());
    }
    out
}

/// Strip hop-by-hop headers from an upstream response before relaying it to
/// the client. CORS headers are added separately by the caller.
pub fn sanitize_response_headers(headers: &reqwest::header::HeaderMap) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in headers.iter() {
        if is_hop_by_hop(name.as_str()) {
            continue;
        }
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            out.append(name, value);
        }
    }
    out
}

/// Forward a GET/HEAD request to `dest`, streaming the upstream response
/// body back to the client without buffering it in memory.
pub async fn relay(
    method: reqwest::Method,
    dest: &Destination,
    client_headers: &HeaderMap,
) -> Result<Response, ProxyError> {
    let client = http_client();

    let mut req = client
        .request(method, &dest.url)
        .header(reqwest::header::USER_AGENT, PROXY_USER_AGENT);

    let sanitized = sanitize_request_headers(client_headers);
    for (name, value) in sanitized.iter() {
        // reqwest::header types are a different crate re-export of the same
        // http crate — convert via raw bytes.
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            req = req.header(name, value);
        }
    }

    let upstream = req.send().await.map_err(|e| {
        if e.is_timeout() {
            ProxyError::UpstreamError(format!("upstream timeout: {e}"))
        } else if e.is_connect() {
            ProxyError::UpstreamError(format!("upstream connect failed: {e}"))
        } else {
            ProxyError::UpstreamError(format!("upstream request failed: {e}"))
        }
    })?;

    let status = StatusCode::from_u16(upstream.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let response_headers = sanitize_response_headers(upstream.headers());
    let body = Body::from_stream(upstream.bytes_stream());

    let mut response = Response::new(body);
    *response.status_mut() = status;
    *response.headers_mut() = response_headers;
    Ok(response.into_response())
}
