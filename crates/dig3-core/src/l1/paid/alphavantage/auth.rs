//! # AlphaVantage Authentication
//!
//! AlphaVantage uses simple API key authentication via query parameter.
//! NO HMAC signatures or OAuth - just a plain API key.

use std::collections::HashMap;

/// AlphaVantage authentication credentials
#[derive(Clone)]
pub struct AlphaVantageAuth {
    pub api_key: Option<String>,
}

impl AlphaVantageAuth {
    /// Create auth from environment variable
    ///
    /// Looks for `ALPHAVANTAGE_API_KEY` environment variable.
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ALPHAVANTAGE_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (will use demo key)
    ///
    /// Demo key only works with IBM stock, not forex or other symbols.
    pub fn demo() -> Self {
        Self { api_key: None }
    }

    /// Add API key to query parameters
    ///
    /// AlphaVantage uses query parameter authentication, NOT headers.
    /// The API key is added as `?apikey=YOUR_KEY` to the URL.
    ///
    /// If no API key is provided, falls back to "demo" key (limited functionality).
    pub fn add_to_params(&self, params: &mut HashMap<String, String>) {
        let key = self.api_key.as_deref().unwrap_or("demo");

        params.insert("apikey".to_string(), key.to_string());
    }

    /// Check if using demo API key
    pub fn is_demo(&self) -> bool {
        self.api_key.is_none()
    }
}

impl Default for AlphaVantageAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
