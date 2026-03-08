//! Whale Alert authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! All Whale Alert endpoints require API key authentication via query parameter.
//! No header-based auth, no OAuth, no HMAC signatures - just simple API key in URL.

use std::collections::HashMap;

/// Authentication credentials for Whale Alert
#[derive(Clone)]
pub struct WhaleAlertAuth {
    pub api_key: Option<String>,
}

impl WhaleAlertAuth {
    /// Create new auth from environment variable
    ///
    /// Looks for WHALE_ALERT_API_KEY environment variable
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("WHALE_ALERT_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without credentials (will fail on actual requests)
    pub fn none() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to query parameters
    ///
    /// Whale Alert uses query parameter authentication:
    /// - Parameter name: api_key
    /// - Format: plain API key string
    /// - Location: Query string (not headers)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("api_key".to_string(), key.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Add authentication headers (NOT USED by Whale Alert)
    ///
    /// Included for trait compatibility, but Whale Alert does not use header auth
    pub fn sign_headers(&self, _headers: &mut HashMap<String, String>) {
        // Whale Alert does NOT use header authentication
        // API key is always in query parameter
    }

    /// Generate signature (NOT USED by Whale Alert)
    ///
    /// Whale Alert does not require HMAC signatures - simple API key only
    pub fn generate_signature(
        &self,
        _timestamp: i64,
        _method: &str,
        _path: &str,
        _query: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err("Signature not required for Whale Alert - uses simple API key".into())
    }
}

impl Default for WhaleAlertAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let auth = WhaleAlertAuth::new("test_key_123");
        assert!(auth.is_authenticated());
        assert_eq!(auth.api_key.as_deref(), Some("test_key_123"));
    }

    #[test]
    fn test_auth_none() {
        let auth = WhaleAlertAuth::none();
        assert!(!auth.is_authenticated());
    }

    #[test]
    fn test_sign_query() {
        let auth = WhaleAlertAuth::new("my_api_key");
        let mut params = HashMap::new();
        params.insert("start".to_string(), "1640000000".to_string());

        auth.sign_query(&mut params);

        assert_eq!(params.get("api_key"), Some(&"my_api_key".to_string()));
        assert_eq!(params.get("start"), Some(&"1640000000".to_string()));
    }

    #[test]
    fn test_headers_not_used() {
        let auth = WhaleAlertAuth::new("my_api_key");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        // Should not add any headers
        assert!(headers.is_empty());
    }
}
