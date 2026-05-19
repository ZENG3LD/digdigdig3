//! # MOEX ISS Authentication
//!
//! Authentication for MOEX ISS API.
//!
//! ## Authentication Types
//! - **Public access**: No authentication (15-minute delayed data)
//! - **Basic Auth**: Username/password from MOEX Passport (real-time data)
//!
//! ## Note
//! MOEX ISS does NOT use API keys. Authentication is via HTTP Basic Auth
//! with MOEX Passport credentials (username and password).

use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// MOEX authentication credentials
#[derive(Clone, Default)]
pub struct MoexAuth {
    /// MOEX Passport username (optional)
    pub username: Option<String>,
    /// MOEX Passport password (optional)
    pub password: Option<String>,
}

impl MoexAuth {
    /// Create public access (no authentication, 15-min delayed data)
    pub fn public() -> Self {
        Self {
            username: None,
            password: None,
        }
    }

    /// Create authenticated access from environment variables
    ///
    /// Reads:
    /// - `MOEX_USERNAME`
    /// - `MOEX_PASSWORD`
    pub fn from_env() -> Self {
        Self {
            username: std::env::var("MOEX_USERNAME").ok(),
            password: std::env::var("MOEX_PASSWORD").ok(),
        }
    }

    /// Create authenticated access with explicit credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Some(username.into()),
            password: Some(password.into()),
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// Get STOMP login/passcode credentials for WebSocket
    ///
    /// Returns `(login, passcode)` as optional strings.
    pub fn credentials(&self) -> (Option<String>, Option<String>) {
        (self.username.clone(), self.password.clone())
    }

    /// Add authentication headers to request
    ///
    /// For MOEX ISS, this uses HTTP Basic Authentication:
    /// ```text
    /// Authorization: Basic base64(username:password)
    /// ```
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let credentials = format!("{}:{}", username, password);
            let encoded = BASE64.encode(credentials.as_bytes());
            headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
        }
    }

    /// Add authentication to query parameters
    ///
    /// MOEX ISS does not use query parameter authentication.
    /// This method is provided for interface compatibility but does nothing.
    pub fn sign_query(&self, _params: &mut HashMap<String, String>) {
        // MOEX ISS does not support query parameter authentication
        // Authentication is done via HTTP Basic Auth headers only
    }

    /// Generate signature (not used for MOEX)
    ///
    /// MOEX ISS does not require HMAC signatures.
    /// This method is provided for interface compatibility but always returns error.
    pub fn generate_signature(
        &self,
        _timestamp: i64,
        _method: &str,
        _path: &str,
        _query: &str,
    ) -> Result<String, String> {
        Err("Signature generation not required for MOEX ISS API".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_auth() {
        let auth = MoexAuth::public();
        assert!(!auth.is_authenticated());

        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);
        assert!(headers.is_empty());
    }

    #[test]
    fn test_authenticated() {
        let auth = MoexAuth::new("testuser", "testpass");
        assert!(auth.is_authenticated());

        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);

        let auth_header = headers.get("Authorization").unwrap();
        assert!(auth_header.starts_with("Basic "));

        // Decode and verify
        let encoded = auth_header.strip_prefix("Basic ").unwrap();
        let decoded = BASE64.decode(encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, "testuser:testpass");
    }

    #[test]
    fn test_signature_not_supported() {
        let auth = MoexAuth::new("user", "pass");
        let result = auth.generate_signature(0, "GET", "/test", "");
        assert!(result.is_err());
    }
}
