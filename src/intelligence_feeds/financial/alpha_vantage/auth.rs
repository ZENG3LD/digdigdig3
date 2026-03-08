//! Alpha Vantage authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! Alpha Vantage uses simple API key authentication via query parameter.
//! No HMAC signatures, no headers - just append `apikey=YOUR_KEY` to the URL.

use std::collections::HashMap;

/// Alpha Vantage authentication credentials
#[derive(Clone)]
pub struct AlphaVantageAuth {
    pub api_key: Option<String>,
}

impl AlphaVantageAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `ALPHA_VANTAGE_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ALPHA_VANTAGE_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication to query parameters
    ///
    /// Alpha Vantage requires API key as a query parameter:
    /// `?apikey=YOUR_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("apikey".to_string(), key.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

impl Default for AlphaVantageAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
