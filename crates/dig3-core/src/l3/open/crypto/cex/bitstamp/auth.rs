//! # Bitstamp Authentication
//!
//! Implementation of request signing for Bitstamp V2 API.
//!
//! ## Signature Algorithm (V2 Method)
//!
//! 1. Create string-to-sign with specific format (includes method, host, path, query, body)
//! 2. HMAC-SHA256 with API secret
//! 3. Convert to uppercase hexadecimal string
//!
//! ## Required Headers
//!
//! - `X-Auth` - "BITSTAMP " + API key
//! - `X-Auth-Signature` - Signature (uppercase hex)
//! - `X-Auth-Nonce` - UUID v4 (unique for each request)
//! - `X-Auth-Timestamp` - Timestamp (milliseconds)
//! - `X-Auth-Version` - "v2"
//! - `Content-Type` - "application/x-www-form-urlencoded" (if body present)
//!
//! ## String-to-Sign Format
//!
//! ```text
//! BITSTAMP {api_key}
//! {method}
//! {host}
//! {path}
//! {query}
//! {content_type}
//! {nonce}
//! {timestamp}
//! {version}
//! {body}
//! ```
//!
//! Note: Each component is separated by newline. Content-type line is omitted if body is empty.

use std::collections::HashMap;
use uuid::Uuid;

use crate::core::{
    hmac_sha256_hex, timestamp_millis,
    Credentials,
};

/// Bitstamp authentication handler
#[derive(Clone)]
pub struct BitstampAuth {
    api_key: String,
    api_secret: String,
}

impl BitstampAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> Self {
        Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
        }
    }

    /// Sign request and return headers
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method ("GET", "POST")
    /// * `path` - Full API path (e.g., "/api/v2/balance/")
    /// * `query` - Query string without leading "?" (empty string if no query)
    /// * `body` - Request body for POST (empty string if no body)
    ///
    /// # Signature Format
    ///
    /// String-to-sign:
    /// ```text
    /// BITSTAMP {api_key}
    /// {method}
    /// www.bitstamp.net
    /// {path}
    /// {query}
    /// {content_type}  <- omitted if body is empty
    /// {nonce}
    /// {timestamp}
    /// v2
    /// {body}
    /// ```
    ///
    /// # Examples
    ///
    /// GET request:
    /// ```ignore
    /// let headers = auth.sign_request("GET", "/api/v2/ticker/btcusd/", "", "");
    /// ```
    ///
    /// POST request with body:
    /// ```ignore
    /// let body = "amount=1.0&price=1000";
    /// let headers = auth.sign_request("POST", "/api/v2/buy/btcusd/", "", body);
    /// ```
    pub fn sign_request(
        &self,
        method: &str,
        path: &str,
        query: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let nonce = Uuid::new_v4().to_string();
        let timestamp = timestamp_millis().to_string();
        let host = "www.bitstamp.net";
        let version = "v2";
        let content_type = "application/x-www-form-urlencoded";

        // Build string-to-sign
        // Format: BITSTAMP {key}\n{method}\n{host}\n{path}\n{query}\n{content_type}\n{nonce}\n{timestamp}\n{version}\n{body}
        let str_to_sign = if body.is_empty() {
            // No content-type line if body is empty
            format!(
                "BITSTAMP {}\n{}\n{}\n{}\n{}\n\n{}\n{}\n{}\n",
                self.api_key,
                method.to_uppercase(),
                host,
                path,
                query,
                nonce,
                timestamp,
                version
            )
        } else {
            format!(
                "BITSTAMP {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                self.api_key,
                method.to_uppercase(),
                host,
                path,
                query,
                content_type,
                nonce,
                timestamp,
                version,
                body
            )
        };

        // HMAC-SHA256 with uppercase hex encoding
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            str_to_sign.as_bytes(),
        ).to_uppercase();

        let mut headers = HashMap::new();
        headers.insert("X-Auth".to_string(), format!("BITSTAMP {}", self.api_key));
        headers.insert("X-Auth-Signature".to_string(), signature);
        headers.insert("X-Auth-Nonce".to_string(), nonce);
        headers.insert("X-Auth-Timestamp".to_string(), timestamp);
        headers.insert("X-Auth-Version".to_string(), version.to_string());

        // Add Content-Type for POST requests with body
        if !body.is_empty() {
            headers.insert("Content-Type".to_string(), content_type.to_string());
        }

        headers
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
        let auth = BitstampAuth::new(&credentials);

        let headers = auth.sign_request("GET", "/api/v2/ticker/btcusd/", "", "");

        assert!(headers.contains_key("X-Auth"));
        assert!(headers.contains_key("X-Auth-Signature"));
        assert!(headers.contains_key("X-Auth-Nonce"));
        assert!(headers.contains_key("X-Auth-Timestamp"));
        assert!(headers.contains_key("X-Auth-Version"));
        assert_eq!(headers.get("X-Auth"), Some(&"BITSTAMP test_key".to_string()));
        assert_eq!(headers.get("X-Auth-Version"), Some(&"v2".to_string()));
        // Content-Type should NOT be present for GET without body
        assert!(!headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_sign_request_post_with_body() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BitstampAuth::new(&credentials);

        let body = "amount=1.0&price=1000";
        let headers = auth.sign_request("POST", "/api/v2/buy/btcusd/", "", body);

        assert!(headers.contains_key("X-Auth"));
        assert!(headers.contains_key("X-Auth-Signature"));
        assert!(headers.contains_key("X-Auth-Nonce"));
        assert!(headers.contains_key("X-Auth-Timestamp"));
        assert!(headers.contains_key("X-Auth-Version"));
        // Content-Type SHOULD be present for POST with body
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/x-www-form-urlencoded".to_string())
        );

        // Signature should be uppercase hex (64 characters for SHA256)
        let signature = headers.get("X-Auth-Signature").unwrap();
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(signature.chars().any(|c| c.is_ascii_uppercase())); // Should have uppercase
    }

    #[test]
    fn test_sign_request_post_without_body() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BitstampAuth::new(&credentials);

        let headers = auth.sign_request("POST", "/api/v2/balance/", "", "");

        assert!(headers.contains_key("X-Auth"));
        assert!(headers.contains_key("X-Auth-Signature"));
        // Content-Type should NOT be present if body is empty
        assert!(!headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_nonce_uniqueness() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BitstampAuth::new(&credentials);

        let headers1 = auth.sign_request("GET", "/api/v2/ticker/btcusd/", "", "");
        let headers2 = auth.sign_request("GET", "/api/v2/ticker/btcusd/", "", "");

        let nonce1 = headers1.get("X-Auth-Nonce").unwrap();
        let nonce2 = headers2.get("X-Auth-Nonce").unwrap();

        // Nonces should be different (UUID v4)
        assert_ne!(nonce1, nonce2);

        // Nonces should be valid UUIDs (36 characters with hyphens)
        assert_eq!(nonce1.len(), 36);
        assert_eq!(nonce2.len(), 36);
    }
}
