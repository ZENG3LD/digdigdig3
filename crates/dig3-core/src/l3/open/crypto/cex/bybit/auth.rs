//! # Bybit Authentication
//!
//! Implementation of request signing for Bybit V5 API.
//!
//! ## Signature Algorithm
//!
//! 1. Create prehash string: `timestamp + api_key + recv_window + (query_string OR json_body)`
//! 2. HMAC-SHA256 with API secret
//! 3. Convert to lowercase hexadecimal string
//!
//! ## Required Headers
//!
//! - `X-BAPI-API-KEY` - API key
//! - `X-BAPI-SIGN` - Signature (lowercase hex)
//! - `X-BAPI-TIMESTAMP` - Timestamp (milliseconds)
//! - `X-BAPI-RECV-WINDOW` - Request validity window (default 5000ms)
//! - `Content-Type` - "application/json" (for POST requests)
//!
//! ## Key Differences from KuCoin
//!
//! - **Simpler**: No passphrase, no API key version header
//! - **API key in signature**: Unlike KuCoin, the API key is part of the signed string
//! - **Recv window**: Explicit validity window parameter (KuCoin has implicit 5s)
//! - **Hex encoding**: Lowercase hex (not Base64 like KuCoin)
//! - **Header prefix**: `X-BAPI-` (not `KC-API-`)

use std::collections::HashMap;

use crate::core::{
    hmac_sha256_hex, timestamp_millis,
    Credentials,
};

/// Bybit authentication handler
#[derive(Clone)]
pub struct BybitAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// Positive = server is ahead, Negative = server is behind
    time_offset_ms: i64,
}

impl BybitAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> Self {
        Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset_ms: 0,
        }
    }

    /// Sync time with server
    /// Call this with server timestamp from /v5/market/time response
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = timestamp_millis() as i64;
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp (local + offset = ~server time)
    fn get_timestamp(&self) -> u64 {
        let local = timestamp_millis() as i64;
        (local + self.time_offset_ms) as u64
    }

    /// Sign request and return headers
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method ("GET", "POST", etc.)
    /// * `param_str` - Query string for GET (without leading "?") OR JSON body for POST
    ///
    /// # Signature Format
    ///
    /// Prehash string: `timestamp + api_key + recv_window + param_str`
    ///
    /// Where:
    /// - timestamp: milliseconds since epoch
    /// - api_key: your API key
    /// - recv_window: "5000" (5 seconds validity)
    /// - param_str: query parameters (GET) or JSON body (POST)
    ///
    /// # Examples
    ///
    /// GET request:
    /// ```ignore
    /// let headers = auth.sign_request("GET", "category=spot&symbol=BTCUSDT");
    /// ```
    ///
    /// POST request:
    /// ```ignore
    /// let body = r#"{"category":"spot","symbol":"BTCUSDT","side":"Buy"}"#;
    /// let headers = auth.sign_request("POST", body);
    /// ```
    pub fn sign_request(
        &self,
        method: &str,
        param_str: &str,
    ) -> HashMap<String, String> {
        let timestamp = self.get_timestamp();
        let timestamp_str = timestamp.to_string();
        let recv_window = "5000"; // 5 seconds validity window

        // Prehash string: timestamp + api_key + recv_window + params
        let str_to_sign = format!(
            "{}{}{}{}",
            timestamp_str,
            self.api_key,
            recv_window,
            param_str
        );

        // HMAC-SHA256 with lowercase hex encoding
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            str_to_sign.as_bytes(),
        );

        let mut headers = HashMap::new();
        headers.insert("X-BAPI-API-KEY".to_string(), self.api_key.clone());
        headers.insert("X-BAPI-SIGN".to_string(), signature);
        headers.insert("X-BAPI-TIMESTAMP".to_string(), timestamp_str);
        headers.insert("X-BAPI-RECV-WINDOW".to_string(), recv_window.to_string());

        // Add Content-Type for POST requests
        if method.to_uppercase() == "POST" {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        headers
    }

    /// Sign WebSocket authentication request
    ///
    /// # WebSocket Auth Format
    ///
    /// String to sign: `GET/realtime{expires}`
    ///
    /// Where expires = current_time + 10 seconds (in milliseconds)
    ///
    /// Returns: (api_key, expires, signature)
    pub fn sign_websocket_auth(&self) -> (String, String, String) {
        let timestamp = self.get_timestamp();
        let expires = timestamp + 10_000; // 10 seconds from now
        let expires_str = expires.to_string();

        // String to sign: GET/realtime{expires}
        let str_to_sign = format!("GET/realtime{}", expires_str);

        // HMAC-SHA256 with lowercase hex encoding
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            str_to_sign.as_bytes(),
        );

        (self.api_key.clone(), expires_str, signature)
    }

    /// Get API key (for headers without signature)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request_get() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BybitAuth::new(&credentials);

        let headers = auth.sign_request("GET", "category=spot&symbol=BTCUSDT");

        assert!(headers.contains_key("X-BAPI-API-KEY"));
        assert!(headers.contains_key("X-BAPI-SIGN"));
        assert!(headers.contains_key("X-BAPI-TIMESTAMP"));
        assert!(headers.contains_key("X-BAPI-RECV-WINDOW"));
        assert_eq!(headers.get("X-BAPI-API-KEY"), Some(&"test_key".to_string()));
        assert_eq!(headers.get("X-BAPI-RECV-WINDOW"), Some(&"5000".to_string()));
        // Content-Type should NOT be present for GET
        assert!(!headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_sign_request_post() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BybitAuth::new(&credentials);

        let body = r#"{"category":"spot","symbol":"BTCUSDT"}"#;
        let headers = auth.sign_request("POST", body);

        assert!(headers.contains_key("X-BAPI-API-KEY"));
        assert!(headers.contains_key("X-BAPI-SIGN"));
        assert!(headers.contains_key("X-BAPI-TIMESTAMP"));
        assert!(headers.contains_key("X-BAPI-RECV-WINDOW"));
        // Content-Type SHOULD be present for POST
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_sign_websocket_auth() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BybitAuth::new(&credentials);

        let (api_key, expires, signature) = auth.sign_websocket_auth();

        assert_eq!(api_key, "test_key");
        assert!(!expires.is_empty());
        assert!(!signature.is_empty());
        // Signature should be lowercase hex (64 characters for SHA256)
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
