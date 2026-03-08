//! CryptoCompare authentication
//!
//! Authentication type: API Key (simple)
//!
//! CryptoCompare uses simple API key authentication via query parameter or header.
//! No HMAC signing required (data provider, not exchange).

use std::collections::HashMap;

/// Authentication credentials for CryptoCompare
#[derive(Clone)]
pub struct CryptoCompareAuth {
    pub api_key: Option<String>,
}

impl CryptoCompareAuth {
    /// Create new auth from environment variable
    ///
    /// Looks for `CRYPTOCOMPARE_API_KEY` environment variable.
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("CRYPTOCOMPARE_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (public endpoints only, low rate limits)
    pub fn public() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to query parameters
    ///
    /// CryptoCompare preferred method: `?api_key=YOUR_KEY`
    ///
    /// This is the most common and recommended authentication method.
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("api_key".to_string(), key.clone());
        }
    }

    /// Add authentication to headers (alternative method)
    ///
    /// CryptoCompare alternative method: `Authorization: Apikey YOUR_KEY`
    ///
    /// Note: Query parameter is preferred, but this works too.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("Authorization".to_string(), format!("Apikey {}", key));
        }
    }

    /// Check if API key is available
    pub fn has_key(&self) -> bool {
        self.api_key.is_some()
    }
}

impl Default for CryptoCompareAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_query() {
        let auth = CryptoCompareAuth::new("test_key_123");
        let mut params = HashMap::new();
        params.insert("fsym".to_string(), "BTC".to_string());

        auth.sign_query(&mut params);

        assert_eq!(params.get("api_key"), Some(&"test_key_123".to_string()));
        assert_eq!(params.get("fsym"), Some(&"BTC".to_string()));
    }

    #[test]
    fn test_sign_headers() {
        let auth = CryptoCompareAuth::new("test_key_123");
        let mut headers = HashMap::new();

        auth.sign_headers(&mut headers);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Apikey test_key_123".to_string())
        );
    }

    #[test]
    fn test_public_auth() {
        let auth = CryptoCompareAuth::public();
        assert!(!auth.has_key());

        let mut params = HashMap::new();
        auth.sign_query(&mut params);
        assert!(!params.contains_key("api_key"));
    }
}
