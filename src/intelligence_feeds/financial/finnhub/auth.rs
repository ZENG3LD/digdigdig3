//! Finnhub authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! Finnhub uses simple API key authentication via query parameter `token`.

use std::collections::HashMap;

/// Finnhub authentication credentials
#[derive(Clone)]
pub struct FinnhubAuth {
    pub api_key: Option<String>,
}

impl FinnhubAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `FINNHUB_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("FINNHUB_API_KEY").ok(),
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
    /// Finnhub requires API key as a query parameter:
    /// `?token=YOUR_API_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("token".to_string(), key.clone());
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

impl Default for FinnhubAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
