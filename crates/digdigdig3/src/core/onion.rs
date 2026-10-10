//! Onion-page classification for a call that never produced a readable status.
//!
//! Native is never an onion page. A readable HTTP status stays the original
//! error: a 401 is a missing key, including when the page host is `.onion`.

use crate::core::types::{ExchangeError, WebSocketError};

pub const SENTENCE: &str = "Does not work on onion. Switch to clearnet.";

pub fn host_is_onion(host: &str) -> bool {
    let host = host.trim_end_matches('.').trim();
    host.len() > ".onion".len() && host.to_ascii_lowercase().ends_with(".onion")
}

pub fn page_is_onion() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().hostname().ok())
            .is_some_and(|host| host_is_onion(&host))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn classify_exchange(err: ExchangeError) -> ExchangeError {
    classify_exchange_on(page_is_onion(), err)
}

pub fn classify_exchange_on(onion: bool, err: ExchangeError) -> ExchangeError {
    if onion && matches!(err, ExchangeError::Network(_) | ExchangeError::Timeout(_)) {
        ExchangeError::OnionBlocked
    } else {
        err
    }
}

pub fn classify_ws(err: WebSocketError) -> WebSocketError {
    classify_ws_on(page_is_onion(), err)
}

/// Connect-time failures only. A parse error, a wire-absent channel, and an
/// auth rejection stay themselves.
pub fn classify_ws_on(onion: bool, err: WebSocketError) -> WebSocketError {
    if onion && matches!(err, WebSocketError::ConnectionError(_) | WebSocketError::Timeout) {
        WebSocketError::OnionBlocked
    } else {
        err
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_suffix() {
        assert!(host_is_onion("wcc2vwh5yq3guekzyt27h5vutvwphchg4evpijiwwsgwso7l7vwegzyd.onion"));
        assert!(host_is_onion("Example.ONION"));
        assert!(host_is_onion("example.onion."));
        assert!(!host_is_onion("onion"));
        assert!(!host_is_onion("notonion.com"));
        assert!(!host_is_onion("evil.onion.example"));
    }

    #[test]
    fn unreachable_on_onion_is_the_sentence() {
        let net = classify_exchange_on(true, ExchangeError::Network("error sending request".into()));
        assert!(matches!(net, ExchangeError::OnionBlocked));
        assert_eq!(net.to_string(), SENTENCE);

        let timed = classify_exchange_on(true, ExchangeError::Timeout("12s".into()));
        assert_eq!(timed.to_string(), SENTENCE);

        let ws = classify_ws_on(true, WebSocketError::ConnectionError("403".into()));
        assert_eq!(ws.to_string(), SENTENCE);
        let ws_to = classify_ws_on(true, WebSocketError::Timeout);
        assert_eq!(ws_to.to_string(), SENTENCE);
    }

    #[test]
    fn clearnet_and_readable_status_stay() {
        let net = classify_exchange_on(false, ExchangeError::Network("reset".into()));
        assert!(matches!(net, ExchangeError::Network(_)));

        let auth = classify_exchange_on(
            true,
            ExchangeError::InvalidCredentials("API key".into()),
        );
        assert!(matches!(auth, ExchangeError::InvalidCredentials(_)));

        let denied = classify_exchange_on(true, ExchangeError::PermissionDenied("no".into()));
        assert!(matches!(denied, ExchangeError::PermissionDenied(_)));

        let limited = classify_exchange_on(
            true,
            ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "slow".into(),
            },
        );
        assert!(matches!(limited, ExchangeError::RateLimitExceeded { .. }));

        let parsed = classify_ws_on(true, WebSocketError::Parse("bad".into()));
        assert!(matches!(parsed, WebSocketError::Parse(_)));
        let absent = classify_ws_on(true, WebSocketError::WireAbsent("no channel".into()));
        assert!(matches!(absent, WebSocketError::WireAbsent(_)));
    }

    #[test]
    fn native_page_is_not_onion() {
        assert!(!page_is_onion());
    }
}
