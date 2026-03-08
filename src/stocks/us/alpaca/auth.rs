//! Alpaca authentication
//!
//! Authentication type: API Key ID + Secret (no HMAC required)
//!
//! Alpaca uses simple header-based authentication:
//! - APCA-API-KEY-ID: Your API key ID
//! - APCA-API-SECRET-KEY: Your API secret key
//!
//! No signatures or HMAC required - much simpler than crypto exchanges.

use std::collections::HashMap;

/// Authentication credentials for Alpaca
#[derive(Clone)]
pub struct AlpacaAuth {
    pub api_key_id: Option<String>,
    pub api_secret_key: Option<String>,
}

impl AlpacaAuth {
    /// Create new auth from environment variables
    ///
    /// Looks for:
    /// - ALPACA_API_KEY_ID or APCA_API_KEY_ID
    /// - ALPACA_API_SECRET_KEY or APCA_API_SECRET_KEY
    pub fn from_env() -> Self {
        Self {
            api_key_id: std::env::var("ALPACA_API_KEY_ID")
                .or_else(|_| std::env::var("APCA_API_KEY_ID"))
                .ok(),
            api_secret_key: std::env::var("ALPACA_API_SECRET_KEY")
                .or_else(|_| std::env::var("APCA_API_SECRET_KEY"))
                .ok(),
        }
    }

    /// Create auth with explicit credentials
    pub fn new(api_key_id: impl Into<String>, api_secret_key: impl Into<String>) -> Self {
        Self {
            api_key_id: Some(api_key_id.into()),
            api_secret_key: Some(api_secret_key.into()),
        }
    }

    /// Create auth with no credentials (for testing endpoints that don't require auth)
    pub fn none() -> Self {
        Self {
            api_key_id: None,
            api_secret_key: None,
        }
    }

    /// Add authentication headers to request
    ///
    /// Alpaca uses two headers:
    /// - APCA-API-KEY-ID: The API key ID
    /// - APCA-API-SECRET-KEY: The API secret key
    ///
    /// No signatures or timestamps required!
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key_id) = &self.api_key_id {
            headers.insert("APCA-API-KEY-ID".to_string(), key_id.clone());
        }

        if let Some(secret_key) = &self.api_secret_key {
            headers.insert("APCA-API-SECRET-KEY".to_string(), secret_key.clone());
        }
    }

    /// Check if auth credentials are present
    pub fn has_credentials(&self) -> bool {
        self.api_key_id.is_some() && self.api_secret_key.is_some()
    }

    /// Get API key ID (for logging/debugging - don't log the secret!)
    pub fn key_id(&self) -> Option<&str> {
        self.api_key_id.as_deref()
    }
}

impl Default for AlpacaAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_auth() {
        let auth = AlpacaAuth::new("test_key_id", "test_secret_key");
        assert!(auth.has_credentials());
        assert_eq!(auth.key_id(), Some("test_key_id"));
    }

    #[test]
    fn test_none_auth() {
        let auth = AlpacaAuth::none();
        assert!(!auth.has_credentials());
        assert_eq!(auth.key_id(), None);
    }

    #[test]
    fn test_sign_headers() {
        let auth = AlpacaAuth::new("my_key_id", "my_secret");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(headers.get("APCA-API-KEY-ID"), Some(&"my_key_id".to_string()));
        assert_eq!(headers.get("APCA-API-SECRET-KEY"), Some(&"my_secret".to_string()));
    }

    #[test]
    fn test_sign_headers_no_creds() {
        let auth = AlpacaAuth::none();
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert!(headers.is_empty());
    }
}
