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
