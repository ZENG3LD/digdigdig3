//! # MEXC Authentication
//!
//! Implementation of request signing for MEXC Spot API.
//!
//! ## Signature Algorithm
//!
//! 1. Build query string with parameters in alphabetical order
//! 2. HMAC-SHA256 with API secret
//! 3. Convert to lowercase hexadecimal string
//!
//! ## Required Headers/Parameters
//!
//! - `X-MEXC-APIKEY` - API key (header)
//! - `timestamp` - Timestamp in milliseconds (parameter)
//! - `recvWindow` - Request validity window, default 5000ms (parameter, optional)
//! - `signature` - HMAC SHA256 signature (parameter)
//!
//! ## Key Differences from Bybit
//!
//! - **Simpler**: Signature is just HMAC-SHA256 of query string
//! - **Parameter-based**: Signature goes in query params, not headers
//! - **Alphabetical sorting**: Parameters must be sorted for GET/DELETE
//! - **Recv window**: Optional, default 5000ms, max 60000ms

use std::collections::HashMap;

use crate::core::{
    hmac_sha256_hex, timestamp_millis,
    Credentials,
};

/// MEXC authentication handler
#[derive(Clone)]
pub struct MexcAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// Positive = server is ahead, Negative = server is behind
    time_offset_ms: i64,
}

impl MexcAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> Self {
        Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset_ms: 0,
        }
    }

    /// Sync time with server
    /// Call this with server timestamp from /api/v3/time response
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = timestamp_millis() as i64;
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp (local + offset = ~server time)
    fn get_timestamp(&self) -> u64 {
        let local = timestamp_millis() as i64;
        (local + self.time_offset_ms) as u64
    }

    /// Sign request and return headers + modified parameters
    ///
    /// # Arguments
    ///
    /// * `params` - Request parameters (will be modified to add timestamp and signature)
    ///
    /// # Signature Format
    ///
    /// 1. Add timestamp and recvWindow to params
    /// 2. Sort parameters alphabetically by key
    /// 3. Build query string: key1=value1&key2=value2&...
    /// 4. HMAC-SHA256 sign the query string with API secret
    /// 5. Add signature to params
    ///
    /// # Returns
    ///
    /// Tuple of (headers, modified_params)
    ///
    /// # Examples
    ///
    /// GET request:
    /// ```ignore
    /// let mut params = HashMap::new();
    /// params.insert("symbol".to_string(), "BTCUSDT".to_string());
    /// let (headers, params) = auth.sign_request(params);
    /// ```
    ///
    /// POST request:
    /// ```ignore
    /// let mut params = HashMap::new();
    /// params.insert("symbol".to_string(), "BTCUSDT".to_string());
    /// params.insert("side".to_string(), "BUY".to_string());
    /// params.insert("type".to_string(), "LIMIT".to_string());
    /// params.insert("quantity".to_string(), "0.1".to_string());
    /// params.insert("price".to_string(), "90000".to_string());
    /// let (headers, params) = auth.sign_request(params);
    /// ```
    pub fn sign_request(
        &self,
        mut params: HashMap<String, String>,
    ) -> (HashMap<String, String>, HashMap<String, String>) {
        let timestamp = self.get_timestamp();

        // Add timestamp and recvWindow to params
        params.insert("timestamp".to_string(), timestamp.to_string());
        params.insert("recvWindow".to_string(), "5000".to_string());

        // Sort parameters alphabetically and build query string
        let mut sorted_keys: Vec<&String> = params.keys().collect();
        sorted_keys.sort();

        let query_string: Vec<String> = sorted_keys
            .iter()
            .map(|k| format!("{}={}", k, params.get(*k).expect("Key exists in params HashMap")))
            .collect();
        let query_string = query_string.join("&");

        // HMAC-SHA256 sign the query string
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            query_string.as_bytes(),
        );

        // Add signature to params
        params.insert("signature".to_string(), signature);

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("X-MEXC-APIKEY".to_string(), self.api_key.clone());

        (headers, params)
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
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = MexcAuth::new(&credentials);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());

        let (headers, signed_params) = auth.sign_request(params);

        // Check headers
        assert_eq!(headers.get("X-MEXC-APIKEY"), Some(&"test_key".to_string()));

        // Check parameters include required fields
        assert!(signed_params.contains_key("timestamp"));
        assert!(signed_params.contains_key("recvWindow"));
        assert!(signed_params.contains_key("signature"));
        assert_eq!(signed_params.get("symbol"), Some(&"BTCUSDT".to_string()));
        assert_eq!(signed_params.get("recvWindow"), Some(&"5000".to_string()));

        // Signature should be 64 character lowercase hex
        let signature = signed_params.get("signature").unwrap();
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn test_sign_request_multiple_params() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = MexcAuth::new(&credentials);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("side".to_string(), "BUY".to_string());
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("quantity".to_string(), "0.1".to_string());
        params.insert("price".to_string(), "90000".to_string());

        let (headers, signed_params) = auth.sign_request(params);

        // Check all parameters are present
        assert!(signed_params.contains_key("symbol"));
        assert!(signed_params.contains_key("side"));
        assert!(signed_params.contains_key("type"));
        assert!(signed_params.contains_key("quantity"));
        assert!(signed_params.contains_key("price"));
        assert!(signed_params.contains_key("timestamp"));
        assert!(signed_params.contains_key("recvWindow"));
        assert!(signed_params.contains_key("signature"));

        // Check headers
        assert_eq!(headers.get("X-MEXC-APIKEY"), Some(&"test_key".to_string()));
    }
}
