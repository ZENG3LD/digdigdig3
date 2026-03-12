//! # Bitquery Authentication
//!
//! OAuth 2.0 Bearer token authentication for Bitquery API.
//!
//! ## Authentication Method
//!
//! - Type: OAuth 2.0 Access Token
//! - Format: `ory_at_...` (Bearer token)
//! - Header: `Authorization: Bearer ory_at_YOUR_TOKEN`
//! - WebSocket: Token in URL parameter `?token=ory_at_YOUR_TOKEN`
//!
//! ## Token Acquisition
//!
//! 1. Sign up at https://account.bitquery.io/auth/signup
//! 2. Generate OAuth token in IDE or dashboard
//! 3. Use token in Authorization header
//!
//! ## No HMAC/Signing Required
//!
//! Unlike exchanges, Bitquery uses simple OAuth tokens.
//! No request signing or HMAC computation needed.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Bitquery OAuth authentication
#[derive(Clone)]
pub struct BitqueryAuth {
    /// OAuth 2.0 access token (format: ory_at_...)
    access_token: String,
}

impl BitqueryAuth {
    /// Create new auth handler from credentials
    ///
    /// # Arguments
    ///
    /// * `credentials` - Credentials with API key containing OAuth token
    ///
    /// # Example
    ///
    /// ```ignore
    /// let credentials = Credentials {
    ///     api_key: "ory_at_YOUR_OAUTH_TOKEN".to_string(),
    ///     api_secret: String::new(),
    ///     passphrase: None,
    /// };
    /// let auth = BitqueryAuth::new(&credentials)?;
    /// ```
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_key.is_empty() {
            return Err(ExchangeError::Auth(
                "Bitquery requires OAuth access token in api_key field".to_string()
            ));
        }

        // Validate token format (should start with ory_at_)
        if !credentials.api_key.starts_with("ory_at_") {
            return Err(ExchangeError::Auth(
                "Invalid Bitquery token format. Expected format: ory_at_...".to_string()
            ));
        }

        Ok(Self {
            access_token: credentials.api_key.clone(),
        })
    }

    /// Create auth from environment variable
    ///
    /// Reads token from `BITQUERY_TOKEN` environment variable.
    pub fn from_env() -> ExchangeResult<Self> {
        let token = std::env::var("BITQUERY_TOKEN")
            .map_err(|_| ExchangeError::Auth(
                "BITQUERY_TOKEN environment variable not set".to_string()
            ))?;

        let credentials = Credentials {
            api_key: token,
            api_secret: String::new(),
            passphrase: None,
        };

        Self::new(&credentials)
    }

    /// Add authentication headers to HTTP request
    ///
    /// Adds `Authorization: Bearer ory_at_...` header.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.access_token)
        );
        headers.insert(
            "Content-Type".to_string(),
            "application/json".to_string()
        );
    }

    /// Get WebSocket URL with authentication token
    ///
    /// Returns WebSocket URL with token as query parameter.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base WebSocket URL (e.g., wss://streaming.bitquery.io/graphql)
    ///
    /// # Returns
    ///
    /// WebSocket URL with token: `wss://streaming.bitquery.io/graphql?token=ory_at_...`
    pub fn get_ws_url(&self, base_url: &str) -> String {
        format!("{}?token={}", base_url, self.access_token)
    }

    /// Get access token
    ///
    /// Returns the raw OAuth access token.
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_valid_token() {
        let credentials = Credentials {
            api_key: "ory_at_test_token_12345".to_string(),
            api_secret: String::new(),
            passphrase: None,
        };

        let auth = BitqueryAuth::new(&credentials).unwrap();
        assert_eq!(auth.access_token(), "ory_at_test_token_12345");
    }

    #[test]
    fn test_new_with_invalid_token_format() {
        let credentials = Credentials {
            api_key: "invalid_token_format".to_string(),
            api_secret: String::new(),
            passphrase: None,
        };

        let result = BitqueryAuth::new(&credentials);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Bitquery token format"));
    }

    #[test]
    fn test_new_with_empty_token() {
        let credentials = Credentials {
            api_key: String::new(),
            api_secret: String::new(),
            passphrase: None,
        };

        let result = BitqueryAuth::new(&credentials);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires OAuth access token"));
    }

    #[test]
    fn test_sign_headers() {
        let credentials = Credentials {
            api_key: "ory_at_test_token".to_string(),
            api_secret: String::new(),
            passphrase: None,
        };

        let auth = BitqueryAuth::new(&credentials).unwrap();
        let mut headers = HashMap::new();
        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer ory_at_test_token".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_get_ws_url() {
        let credentials = Credentials {
            api_key: "ory_at_test_token".to_string(),
            api_secret: String::new(),
            passphrase: None,
        };

        let auth = BitqueryAuth::new(&credentials).unwrap();
        let ws_url = auth.get_ws_url("wss://streaming.bitquery.io/graphql");

        assert_eq!(
            ws_url,
            "wss://streaming.bitquery.io/graphql?token=ory_at_test_token"
        );
    }
}
