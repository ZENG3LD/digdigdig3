//! Destination-URL extraction for the two proxy shapes.
//!
//! Mirrors the contract documented in
//! `digdigdig3::core::proxy_allowlist` (prefix mode / encoded `?url=` mode)
//! and `digdigdig3::core::http::url_override::assemble_rest_url` (the
//! client-side counterpart that builds these same shapes).

use digdigdig3::core::proxy_allowlist::is_allowed_rest_host;
use url::Url;

use crate::error::ProxyError;

/// A validated destination: full HTTPS URL to forward to, host already
/// checked against the allowlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub url: String,
    pub host: String,
}

/// Prefix-mode extraction: the proxy receives `/{host}/{*rest}?query`.
/// `host` is the first path segment; `rest` is everything after it
/// (already re-joined with a leading `/`); `query` is the raw query string
/// (without `?`), forwarded verbatim.
///
/// The real destination is assembled as `https://{host}{rest}?{query}`.
pub fn from_prefix(host: &str, rest: &str, query: Option<&str>) -> Result<Destination, ProxyError> {
    if host.is_empty() {
        return Err(ProxyError::BadRequest(
            "missing destination host in path".to_string(),
        ));
    }
    if !is_allowed_rest_host(host) {
        return Err(ProxyError::HostNotAllowed(host.to_string()));
    }

    let path = if rest.is_empty() || rest.starts_with('/') {
        rest.to_string()
    } else {
        format!("/{rest}")
    };

    let mut url = format!("https://{host}{path}");
    if let Some(q) = query.filter(|q| !q.is_empty()) {
        url.push('?');
        url.push_str(q);
    }

    Ok(Destination {
        url,
        host: host.to_ascii_lowercase(),
    })
}

/// Encoded mode: `?url=<percent-encoded full URL>`. Decode + parse, then
/// validate the extracted host.
pub fn from_encoded(raw_url_param: &str) -> Result<Destination, ProxyError> {
    if raw_url_param.is_empty() {
        return Err(ProxyError::BadRequest("empty url parameter".to_string()));
    }

    // `axum::extract::Query` / the router already percent-decodes query
    // values for us by the time this fn is called, so `raw_url_param` is
    // the plain target URL string.
    let parsed = Url::parse(raw_url_param)
        .map_err(|e| ProxyError::BadRequest(format!("invalid url parameter: {e}")))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(ProxyError::BadRequest(format!(
            "unsupported scheme '{}': only http/https allowed",
            parsed.scheme()
        )));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| ProxyError::BadRequest("url parameter has no host".to_string()))?
        .to_string();

    if !is_allowed_rest_host(&host) {
        return Err(ProxyError::HostNotAllowed(host));
    }

    Ok(Destination {
        url: parsed.to_string(),
        host: host.to_ascii_lowercase(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_mode_valid_host() {
        let dest = from_prefix(
            "api.binance.com",
            "/api/v3/klines",
            Some("symbol=BTCUSDT&interval=1m&limit=1"),
        )
        .expect("should be allowed");
        assert_eq!(
            dest.url,
            "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1m&limit=1"
        );
        assert_eq!(dest.host, "api.binance.com");
    }

    #[test]
    fn prefix_mode_no_query() {
        let dest = from_prefix("api.kucoin.com", "/api/v1/timestamp", None).expect("allowed");
        assert_eq!(dest.url, "https://api.kucoin.com/api/v1/timestamp");
    }

    #[test]
    fn prefix_mode_empty_rest() {
        let dest = from_prefix("api.binance.com", "", None).expect("allowed");
        assert_eq!(dest.url, "https://api.binance.com");
    }

    #[test]
    fn prefix_mode_rejects_disallowed_host() {
        let err = from_prefix("evil.com", "/steal", None).unwrap_err();
        assert!(matches!(err, ProxyError::HostNotAllowed(h) if h == "evil.com"));
    }

    #[test]
    fn prefix_mode_rejects_ssrf_suffix_confusion() {
        let err = from_prefix("api.binance.com.evil.com", "/api/v3/klines", None).unwrap_err();
        assert!(matches!(err, ProxyError::HostNotAllowed(_)));
    }

    #[test]
    fn prefix_mode_rejects_empty_host() {
        let err = from_prefix("", "/api/v3/klines", None).unwrap_err();
        assert!(matches!(err, ProxyError::BadRequest(_)));
    }

    #[test]
    fn prefix_mode_case_insensitive_host() {
        let dest = from_prefix("API.BINANCE.COM", "/api/v3/time", None).expect("allowed");
        assert_eq!(dest.host, "api.binance.com");
    }

    #[test]
    fn encoded_mode_valid_host() {
        let dest = from_encoded("https://api.binance.com/api/v3/klines?symbol=BTCUSDT")
            .expect("allowed");
        assert_eq!(
            dest.url,
            "https://api.binance.com/api/v3/klines?symbol=BTCUSDT"
        );
        assert_eq!(dest.host, "api.binance.com");
    }

    #[test]
    fn encoded_mode_rejects_disallowed_host() {
        let err = from_encoded("https://evil.com/steal").unwrap_err();
        assert!(matches!(err, ProxyError::HostNotAllowed(h) if h == "evil.com"));
    }

    #[test]
    fn encoded_mode_rejects_garbage_url() {
        let err = from_encoded("not-a-url-at-all").unwrap_err();
        assert!(matches!(err, ProxyError::BadRequest(_)));
    }

    #[test]
    fn encoded_mode_rejects_empty() {
        let err = from_encoded("").unwrap_err();
        assert!(matches!(err, ProxyError::BadRequest(_)));
    }

    #[test]
    fn encoded_mode_rejects_non_http_scheme() {
        let err = from_encoded("file:///etc/passwd").unwrap_err();
        assert!(matches!(err, ProxyError::BadRequest(_)));
    }

    #[test]
    fn encoded_mode_rejects_ssrf_metadata_ip() {
        let err = from_encoded("http://169.254.169.254/latest/meta-data/").unwrap_err();
        assert!(matches!(err, ProxyError::HostNotAllowed(_)));
    }
}
