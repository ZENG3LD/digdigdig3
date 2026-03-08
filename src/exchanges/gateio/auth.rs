//! # Gate.io Authentication
//!
//! Request signing implementation for Gate.io API V4.
//!
//! ## Signature Algorithm
//!
//! 1. Build prehash string: `method + "\n" + url + "\n" + query_string + "\n" + payload_hash + "\n" + timestamp`
//! 2. HMAC-SHA512 with secret key
//! 3. Convert to lowercase hexadecimal
//!
//! ## Headers
//!
//! - `KEY` - API key
//! - `SIGN` - Signature (lowercase hex)
//! - `Timestamp` - Unix timestamp in seconds
//! - `Content-Type` - "application/json" (for POST requests)

use std::collections::HashMap;

use crate::core::{
    hmac_sha512, sha512,
    Credentials, ExchangeResult,
    timestamp_millis,
};

/// Gate.io authentication
#[derive(Clone)]
pub struct GateioAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (seconds)
    time_offset: i64,
}

impl GateioAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset: 0,
        })
    }

    /// Sync time with server
    /// Call this with server timestamp from /spot/time response
    pub fn sync_time(&mut self, server_time_seconds: i64) {
        let local_time_seconds = (timestamp_millis() / 1000) as i64;
        self.time_offset = server_time_seconds - local_time_seconds;
    }

    /// Get adjusted timestamp in seconds
    fn get_timestamp(&self) -> u64 {
        let local_seconds = (timestamp_millis() / 1000) as i64;
        (local_seconds + self.time_offset) as u64
    }

    /// Calculate SHA512 hash of payload and return lowercase hex
    fn hash_payload(&self, payload: &str) -> String {
        let hash_bytes = sha512(payload.as_bytes());
        hex::encode(hash_bytes)
    }

    /// Sign request and return headers
    ///
    /// # Signature String Format
    /// ```text
    /// method + "\n" + url + "\n" + query_string + "\n" + payload_hash + "\n" + timestamp
    /// ```
    pub fn sign_request(
        &self,
        method: &str,
        url: &str,
        query_string: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let timestamp = self.get_timestamp();
        let timestamp_str = timestamp.to_string();

        // Hash the payload
        let payload_hash = self.hash_payload(body);

        // Build prehash string
        // Format: method + "\n" + url + "\n" + query_string + "\n" + payload_hash + "\n" + timestamp
        let prehash = format!(
            "{}\n{}\n{}\n{}\n{}",
            method.to_uppercase(),
            url,
            query_string,
            payload_hash,
            timestamp_str
        );

        // Calculate HMAC-SHA512 signature
        let signature_bytes = hmac_sha512(
            self.api_secret.as_bytes(),
            prehash.as_bytes(),
        );
        let signature = hex::encode(signature_bytes);

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("KEY".to_string(), self.api_key.clone());
        headers.insert("SIGN".to_string(), signature);
        headers.insert("Timestamp".to_string(), timestamp_str);

        if method.to_uppercase() == "POST" || method.to_uppercase() == "PUT" || method.to_uppercase() == "PATCH" {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        headers
    }

    /// Get API key (for headers without signature)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Sign WebSocket authentication message
    ///
    /// # WebSocket Signature Format
    /// ```text
    /// "channel={channel}&event={event}&time={timestamp}"
    /// ```
    pub fn sign_ws(&self, sign_str: &str) -> ExchangeResult<String> {
        // Calculate HMAC-SHA512 signature
        let signature_bytes = hmac_sha512(
            self.api_secret.as_bytes(),
            sign_str.as_bytes(),
        );
        let signature = hex::encode(signature_bytes);
        Ok(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GateioAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("GET", "/api/v4/spot/accounts", "", "");

        assert!(headers.contains_key("KEY"));
        assert!(headers.contains_key("SIGN"));
        assert!(headers.contains_key("Timestamp"));
        assert_eq!(headers.get("KEY").unwrap(), "test_key");
    }

    #[test]
    fn test_post_request_headers() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GateioAuth::new(&credentials).unwrap();

        let body = r#"{"currency_pair":"BTC_USDT","side":"buy"}"#;
        let headers = auth.sign_request("POST", "/api/v4/spot/orders", "", body);

        assert!(headers.contains_key("Content-Type"));
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_hash_payload() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GateioAuth::new(&credentials).unwrap();

        let empty_hash = auth.hash_payload("");
        // SHA512 of empty string
        assert_eq!(
            empty_hash,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        );
    }
}
