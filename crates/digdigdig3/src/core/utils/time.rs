//! # Time Utilities
//!
//! Timestamp generation в разных форматах.

use chrono::{Utc, SecondsFormat};

/// Timestamp в миллисекундах (KuCoin, Binance, Bybit)
pub fn timestamp_millis() -> u64 {
    Utc::now().timestamp_millis() as u64
}

/// Timestamp в секундах (Gate.io)
pub fn timestamp_seconds() -> u64 {
    Utc::now().timestamp() as u64
}

/// Timestamp в ISO 8601 формате (OKX)
/// Пример: "2020-12-08T09:08:57.715Z"
pub fn timestamp_iso8601() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

// ─── Wasm-safe wall-clock helpers ────────────────────────────────────────────
//
// `std::time::SystemTime::now()` compiles on wasm32-unknown-unknown but PANICS
// at runtime ("unsupported").  Use `js_sys::Date::now()` on wasm instead —
// it calls `performance.now()` plus the epoch base and returns f64 ms.

/// Milliseconds since Unix epoch. Wasm-safe.
///
/// Native: `std::time::SystemTime::now().duration_since(UNIX_EPOCH)`.
/// Wasm32: `js_sys::Date::now() as i64`.
#[inline]
pub(crate) fn now_ms() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_millis() {
        let ts = timestamp_millis();
        assert!(ts > 1700000000000); // After 2023
    }

    #[test]
    fn test_timestamp_seconds() {
        let ts = timestamp_seconds();
        assert!(ts > 1700000000); // After 2023
    }

    #[test]
    fn test_timestamp_iso8601() {
        let ts = timestamp_iso8601();
        assert!(ts.contains("T"));
        assert!(ts.ends_with("Z"));
    }
}
