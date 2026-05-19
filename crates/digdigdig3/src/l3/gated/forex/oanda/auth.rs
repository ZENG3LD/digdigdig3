//! # OANDA v20 Authentication
//!
//! Bearer token authentication for OANDA v20 REST API.
//!
//! ## Authentication Method
//!
//! OANDA uses Bearer token authentication:
//! - Single token per account
//! - No HMAC signing required
//! - No key/secret pairs
//! - Token is passed in Authorization header
//!
//! ## Headers
//!
//! - `Authorization: Bearer <TOKEN>`
//! - `Content-Type: application/json`

use std::collections::HashMap;

use crate::core::{Credentials, ExchangeResult, ExchangeError};

/// OANDA authentication handler
#[derive(Clone)]
pub struct OandaAuth {
    /// Bearer token
    token: String,
}

impl OandaAuth {
    /// Create new auth handler from credentials
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        // OANDA uses api_key field to store the bearer token
        let token = credentials.api_key.clone();

        if token.is_empty() {
            return Err(ExchangeError::Auth(
                "OANDA requires bearer token (store in api_key field)".to_string()
            ));
        }

        Ok(Self { token })
    }

    /// Get authentication headers
    ///
    /// Returns HashMap with:
    /// - Authorization: Bearer <token>
    /// - Content-Type: application/json
    pub fn sign_request(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.token));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// Get the bearer token
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_headers() {
        let credentials = Credentials::new("test_token_12345", "");
        let auth = OandaAuth::new(&credentials).unwrap();

        let headers = auth.sign_request();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer test_token_12345".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_empty_token() {
        let credentials = Credentials::new("", "");
        let result = OandaAuth::new(&credentials);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_access() {
        let credentials = Credentials::new("my_secret_token", "");
        let auth = OandaAuth::new(&credentials).unwrap();
        assert_eq!(auth.token(), "my_secret_token");
    }
}
