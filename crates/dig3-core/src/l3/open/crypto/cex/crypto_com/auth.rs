//! # Crypto.com Authentication
//!
//! HMAC-SHA256 signature implementation for Crypto.com Exchange API v1.
//!
//! ## Signature Algorithm
//!
//! 1. Sort parameters alphabetically by key
//! 2. Concatenate as: key1 + value1 + key2 + value2 + ...
//! 3. Build payload: method + id + api_key + params_string + nonce
//! 4. HMAC-SHA256 with API secret
//! 5. Encode as lowercase hex string
//!
//! ## Example
//!
//! ```
//! method: private/create-order
//! id: 1
//! api_key: my_api_key
//! params: {
//!   "instrument_name": "BTCUSD-PERP",
//!   "price": "50000.00",
//!   "quantity": "0.5",
//!   "side": "BUY",
//!   "type": "LIMIT"
//! }
//! nonce: 1587523073344
//!
//! params_string (sorted): instrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT
//! payload: private/create-order1my_api_keyinstrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT1587523073344
//! signature: HMAC-SHA256(payload, api_secret) -> hex
//! ```

use std::collections::BTreeMap;
use serde_json::Value;

use crate::core::{
    hmac_sha256_hex, timestamp_millis,
    Credentials, ExchangeResult,
};

/// Crypto.com authentication handler
#[derive(Clone)]
pub struct CryptoComAuth {
    api_key: String,
    api_secret: String,
}

impl CryptoComAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
        })
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Generate signature for REST API request
    ///
    /// # Arguments
    /// - `method` - API method (e.g., "private/create-order")
    /// - `id` - Request ID (integer)
    /// - `params` - Request parameters as JSON Value
    /// - `nonce` - Timestamp in milliseconds
    ///
    /// # Returns
    /// Hex-encoded HMAC-SHA256 signature
    pub fn sign_request(
        &self,
        method: &str,
        id: i64,
        params: &Value,
        nonce: i64,
    ) -> String {
        // Build parameter string (sorted alphabetically)
        let params_string = self.build_params_string(params);

        // Build signature payload
        let payload = format!(
            "{}{}{}{}{}",
            method,
            id,
            &self.api_key,
            params_string,
            nonce
        );

        // HMAC-SHA256 and encode as hex
        hmac_sha256_hex(self.api_secret.as_bytes(), payload.as_bytes())
    }

    /// Generate signature for WebSocket authentication
    ///
    /// WebSocket auth uses simpler payload: method + id + api_key + nonce
    /// (no params for auth method)
    pub fn sign_ws_auth(&self, id: i64, nonce: i64) -> String {
        let payload = format!("public/auth{}{}{}", id, &self.api_key, nonce);
        hmac_sha256_hex(self.api_secret.as_bytes(), payload.as_bytes())
    }

    /// Build parameter string from JSON params
    ///
    /// Rules:
    /// 1. Sort keys alphabetically
    /// 2. Concatenate as: key1 + value1 + key2 + value2 + ...
    /// 3. No separators, no spaces
    /// 4. Values are stringified (numbers as strings without quotes)
    ///
    /// # Example
    /// ```
    /// params: {"side": "BUY", "price": "50000.00", "quantity": "0.5"}
    /// result: "price50000.00quantity0.5sideBUY"
    /// ```
    fn build_params_string(&self, params: &Value) -> String {
        let obj = match params.as_object() {
            Some(o) => o,
            None => return String::new(), // Empty params
        };

        // Sort keys alphabetically using BTreeMap
        let sorted: BTreeMap<&String, &Value> = obj.iter().collect();

        // Concatenate key + value pairs
        let mut result = String::new();
        for (key, value) in sorted {
            result.push_str(key);
            result.push_str(&self.value_to_string(value));
        }

        result
    }

    /// Convert JSON value to string for signature
    ///
    /// Rules:
    /// - String: raw value without quotes
    /// - Number: as string without quotes
    /// - Boolean: "true" or "false"
    /// - Null: "null"
    /// - Array/Object: JSON stringified
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(), // Arrays/Objects as JSON
        }
    }

    /// Generate nonce (milliseconds timestamp)
    pub fn generate_nonce() -> i64 {
        timestamp_millis() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_params_string() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        let params = json!({
            "instrument_name": "BTCUSD-PERP",
            "side": "BUY",
            "type": "LIMIT",
            "price": "50000.00",
            "quantity": "0.5"
        });

        let result = auth.build_params_string(&params);

        // Keys should be sorted: instrument_name, price, quantity, side, type
        assert_eq!(
            result,
            "instrument_nameBTCUSD-PERPprice50000.00quantity0.5sideBUYtypeLIMIT"
        );
    }

    #[test]
    fn test_build_params_string_empty() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        let params = json!({});
        let result = auth.build_params_string(&params);

        assert_eq!(result, "");
    }

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        let params = json!({
            "instrument_name": "BTCUSD-PERP"
        });

        let signature = auth.sign_request("private/get-open-orders", 1, &params, 1234567890);

        // Signature should be 64-character hex string (SHA256)
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_ws_auth() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        let signature = auth.sign_ws_auth(1, 1234567890);

        // Signature should be 64-character hex string
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_value_to_string() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        assert_eq!(auth.value_to_string(&json!("test")), "test");
        assert_eq!(auth.value_to_string(&json!(123)), "123");
        assert_eq!(auth.value_to_string(&json!(123.45)), "123.45");
        assert_eq!(auth.value_to_string(&json!(true)), "true");
        assert_eq!(auth.value_to_string(&json!(false)), "false");
        assert_eq!(auth.value_to_string(&json!(null)), "null");
    }

    #[test]
    fn test_signature_consistency() {
        // Same inputs should produce same signature
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = CryptoComAuth::new(&credentials).unwrap();

        let params = json!({"side": "BUY", "price": "50000"});
        let sig1 = auth.sign_request("private/create-order", 1, &params, 1234567890);
        let sig2 = auth.sign_request("private/create-order", 1, &params, 1234567890);

        assert_eq!(sig1, sig2);
    }
}
