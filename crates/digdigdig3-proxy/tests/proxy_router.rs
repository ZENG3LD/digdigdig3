//! Router-level tests using axum/tower `oneshot` — no live network needed
//! except the single `--ignored` live test at the bottom.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use digdigdig3_proxy::router::build_router;
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("valid json body")
}

#[tokio::test]
async fn health_reports_allowlist_size() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .expect("ACAO header present"),
        "*"
    );
    let body = body_json(response).await;
    assert_eq!(body["ok"], true);
    assert!(body["allowlist_size"].as_u64().unwrap_or(0) > 0);
}

#[tokio::test]
async fn prefix_mode_disallowed_host_returns_403() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evil.com/steal")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .expect("ACAO header present on error response too"),
        "*"
    );
    let body = body_json(response).await;
    assert_eq!(body["error"], "host_not_allowed");
}

#[tokio::test]
async fn prefix_mode_ssrf_suffix_confusion_rejected() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api.binance.com.evil.com/api/v3/klines")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn encoded_mode_disallowed_host_returns_403() {
    let app = build_router();
    let target = "https%3A%2F%2Fevil.com%2Fsteal";
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/proxy?url={target}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = body_json(response).await;
    assert_eq!(body["error"], "host_not_allowed");
}

#[tokio::test]
async fn encoded_mode_garbage_url_returns_400() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/proxy?url=not-a-url-at-all")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert_eq!(body["error"], "bad_request");
}

#[tokio::test]
async fn encoded_mode_missing_url_param_returns_400() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/proxy")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_method_rejected_with_405() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api.binance.com/api/v3/order")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    let body = body_json(response).await;
    assert_eq!(body["error"], "method_not_allowed");
}

#[tokio::test]
async fn preflight_options_returns_204_with_cors_headers() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api.binance.com/api/v3/klines")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers().get("access-control-allow-origin").expect("ACAO"),
        "*"
    );
    let methods = response
        .headers()
        .get("access-control-allow-methods")
        .expect("allow-methods present")
        .to_str()
        .expect("ascii");
    assert!(methods.contains("GET"));
    assert!(methods.contains("HEAD"));
    assert!(methods.contains("OPTIONS"));
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-headers")
            .expect("allow-headers present"),
        "*"
    );
}

#[tokio::test]
async fn preflight_on_proxy_route_also_answered() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/proxy?url=https%3A%2F%2Fapi.binance.com%2Fapi%2Fv3%2Ftime")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

// ── hop-by-hop header stripping (unit-level, no network) ──────────────────

#[test]
fn hop_by_hop_headers_stripped_from_client_request() {
    use axum::http::HeaderMap;
    use digdigdig3_proxy::forward::sanitize_request_headers;

    let mut headers = HeaderMap::new();
    headers.insert("connection", "keep-alive".parse().expect("header value"));
    headers.insert("cookie", "session=abc".parse().expect("header value"));
    headers.insert("authorization", "Bearer secret".parse().expect("header value"));
    headers.insert("accept", "application/json".parse().expect("header value"));

    let sanitized = sanitize_request_headers(&headers);
    assert!(sanitized.get("connection").is_none());
    assert!(sanitized.get("cookie").is_none());
    assert!(sanitized.get("authorization").is_none());
    assert!(sanitized.get("accept").is_some(), "accept header should pass through");
}

// ── live test (real network, public no-key endpoint) ───────────────────────

#[tokio::test]
#[ignore = "hits real Binance REST endpoint"]
async fn live_prefix_mode_binance_server_time() {
    let app = build_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api.binance.com/api/v3/time")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .expect("ACAO header present"),
        "*"
    );
    let body = body_json(response).await;
    assert!(body.get("serverTime").is_some(), "expected serverTime field: {body:?}");
}
