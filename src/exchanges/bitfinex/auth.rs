//! # Bitfinex Authentication
//!
//! HMAC-SHA384 signature implementation for Bitfinex API v2.
//!
//! ## Algorithm
//!
//! 1. Generate nonce (microseconds since epoch)
//! 2. Build signature string: `/api/{apiPath}{nonce}{bodyJson}`
//! 3. Calculate HMAC-SHA384 with API secret
//! 4. Encode as hexadecimal
//!
//! ## Headers
//!
//! - `Content-Type: application/json`
//! - `bfx-nonce` - Microseconds timestamp
//! - `bfx-apikey` - API key
//! - `bfx-signature` - Hex-encoded HMAC-SHA384

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::{
    hmac_sha384, encode_hex_lower, timestamp_millis,
    Credentials, ExchangeResult,
};

/// Bitfinex authentication
pub struct BitfinexAuth {
    api_key: String,
    api_secret: String,
    /// Last used nonce (for strictly increasing requirement)
    /// Uses AtomicU64 for thread-safe interior mutability
    last_nonce: AtomicU64,
}

impl BitfinexAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            last_nonce: AtomicU64::new(0),
        })
    }

    /// Generate nonce (microseconds since epoch)
    ///
    /// Nonce must be strictly increasing for each request.
    /// Uses milliseconds * 1000 to get microseconds.
    fn generate_nonce(&self) -> u64 {
        let nonce = timestamp_millis() * 1000;

        // Atomically update last_nonce to max(nonce, last_nonce + 1)
        self.last_nonce.fetch_max(nonce, Ordering::SeqCst);
        self.last_nonce.fetch_add(1, Ordering::SeqCst)
    }

    /// Sign request and return headers
    ///
    /// # Arguments
    /// - `api_path` - The endpoint path including version (e.g., "v2/auth/r/wallets")
    /// - `body` - JSON body as string (use "{}" for empty body)
    ///
    /// # Returns
    /// HashMap with headers: bfx-nonce, bfx-apikey, bfx-signature, Content-Type
    pub fn sign_request(
        &self,
        api_path: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let nonce = self.generate_nonce();
        let nonce_str = nonce.to_string();

        // Build signature string: /api/{apiPath}{nonce}{bodyJson}
        let signature_string = format!("/api/{}{}{}", api_path, nonce_str, body);

        // Calculate HMAC-SHA384
        let signature_bytes = hmac_sha384(
            self.api_secret.as_bytes(),
            signature_string.as_bytes(),
        );

        // Encode as hex (lowercase)
        let signature = encode_hex_lower(&signature_bytes);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("bfx-nonce".to_string(), nonce_str);
        headers.insert("bfx-apikey".to_string(), self.api_key.clone());
        headers.insert("bfx-signature".to_string(), signature);

        headers
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Sign authentication payload for WebSocket
    ///
    /// Used for WebSocket authentication.
    /// Format: HMAC-SHA384(auth_payload)
    pub fn sign_auth(&self, auth_payload: &str) -> String {
        let signature_bytes = hmac_sha384(
            self.api_secret.as_bytes(),
            auth_payload.as_bytes(),
        );
        encode_hex_lower(&signature_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BitfinexAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("v2/auth/r/wallets", "{}");

        assert!(headers.contains_key("bfx-nonce"));
        assert!(headers.contains_key("bfx-apikey"));
        assert!(headers.contains_key("bfx-signature"));
        assert_eq!(headers.get("bfx-apikey"), Some(&"test_key".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));

        // Signature should be hex string (96 characters for SHA384)
        let sig = headers.get("bfx-signature").unwrap();
        assert_eq!(sig.len(), 96); // SHA384 = 48 bytes * 2 (hex)
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_nonce_increasing() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BitfinexAuth::new(&credentials).unwrap();

        let nonce1 = auth.generate_nonce();
        let nonce2 = auth.generate_nonce();
        let nonce3 = auth.generate_nonce();

        assert!(nonce2 > nonce1);
        assert!(nonce3 > nonce2);
    }

    #[test]
    fn test_signature_format() {
        let credentials = Credentials::new("api_key_123", "api_secret_456");
        let auth = BitfinexAuth::new(&credentials).unwrap();

        let body = r#"{"type":"EXCHANGE LIMIT","symbol":"tBTCUSD","amount":"0.5","price":"10000"}"#;
        let headers = auth.sign_request("v2/auth/w/order/submit", body);

        // Verify all required headers present
        assert!(headers.contains_key("bfx-nonce"));
        assert!(headers.contains_key("bfx-apikey"));
        assert!(headers.contains_key("bfx-signature"));
        assert!(headers.contains_key("Content-Type"));

        // Nonce should be numeric string
        let nonce = headers.get("bfx-nonce").unwrap();
        assert!(nonce.parse::<u64>().is_ok());

        // Signature should be lowercase hex
        let sig = headers.get("bfx-signature").unwrap();
        assert!(sig.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }
}
