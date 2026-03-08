//! Twelvedata authentication
//!
//! Authentication type: Simple API Key (no HMAC required)
//!
//! Twelvedata uses simple API key authentication:
//! - **Header-based (recommended)**: `Authorization: apikey YOUR_KEY`
//! - **Query parameter (alternative)**: `?apikey=YOUR_KEY`
//! - **Demo key**: `apikey=demo` for testing
//!
//! No signatures or HMAC required - simpler than crypto exchanges.

use std::collections::HashMap;

/// Authentication credentials for Twelvedata
#[derive(Clone)]
pub struct TwelvedataAuth {
    pub api_key: Option<String>,
}

impl TwelvedataAuth {
    /// Create new auth from environment variable
    ///
    /// Looks for: TWELVEDATA_API_KEY
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("TWELVEDATA_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth with demo key (for testing)
    ///
    /// Demo key has severe rate limits and limited endpoint access.
    /// Use only for testing, not production.
    pub fn demo() -> Self {
        Self {
            api_key: Some("demo".to_string()),
        }
    }

    /// Create auth with no credentials
    ///
    /// Some public endpoints work without authentication
    pub fn none() -> Self {
        Self { api_key: None }
    }

    /// Add authentication headers to request
    ///
    /// Twelvedata recommends header-based authentication:
    /// `Authorization: apikey YOUR_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("Authorization".to_string(), format!("apikey {}", key));
        }
    }

    /// Add authentication as query parameter
    ///
    /// Alternative method: `?apikey=YOUR_KEY`
    /// Less secure than headers, but works for simple testing.
    pub fn add_query_param(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("apikey".to_string(), key.clone());
        }
    }

    /// Check if auth credentials are present
    pub fn has_credentials(&self) -> bool {
        self.api_key.is_some()
    }

    /// Check if using demo key
    pub fn is_demo(&self) -> bool {
        self.api_key.as_deref() == Some("demo")
    }

    /// Get masked API key (for logging/debugging - show only first/last chars)
    pub fn masked_key(&self) -> Option<String> {
        self.api_key.as_ref().map(|key| {
            if key == "demo" {
                "demo".to_string()
            } else if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len() - 4..])
            } else {
                "****".to_string()
            }
        })
    }
}

impl Default for TwelvedataAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_auth() {
        let auth = TwelvedataAuth::new("test_api_key_12345");
        assert!(auth.has_credentials());
        assert!(!auth.is_demo());
    }

    #[test]
    fn test_demo_auth() {
        let auth = TwelvedataAuth::demo();
        assert!(auth.has_credentials());
        assert!(auth.is_demo());
        assert_eq!(auth.masked_key(), Some("demo".to_string()));
    }

    #[test]
    fn test_none_auth() {
        let auth = TwelvedataAuth::none();
        assert!(!auth.has_credentials());
        assert!(!auth.is_demo());
        assert_eq!(auth.masked_key(), None);
    }

    #[test]
    fn test_sign_headers() {
        let auth = TwelvedataAuth::new("my_secret_key");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"apikey my_secret_key".to_string())
        );
    }

    #[test]
    fn test_sign_headers_demo() {
        let auth = TwelvedataAuth::demo();
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"apikey demo".to_string())
        );
    }

    #[test]
    fn test_query_param() {
        let auth = TwelvedataAuth::new("my_key");
        let mut params = HashMap::new();

        auth.add_query_param(&mut params);

        assert_eq!(params.get("apikey"), Some(&"my_key".to_string()));
    }

    #[test]
    fn test_masked_key() {
        let auth = TwelvedataAuth::new("abcdefgh12345678");
        assert_eq!(auth.masked_key(), Some("abcd...5678".to_string()));

        let short_auth = TwelvedataAuth::new("short");
        assert_eq!(short_auth.masked_key(), Some("****".to_string()));
    }
}
