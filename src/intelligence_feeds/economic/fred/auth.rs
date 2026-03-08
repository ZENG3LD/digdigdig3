//! FRED authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! FRED uses simple API key authentication via query parameter.
//! No HMAC signatures, no headers - just append `api_key=YOUR_KEY` to the URL.

use std::collections::HashMap;

/// FRED authentication credentials
#[derive(Clone)]
pub struct FredAuth {
    pub api_key: Option<String>,
}

impl FredAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `FRED_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("FRED_API_KEY").ok(),
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
    /// FRED requires API key as a query parameter:
    /// `?api_key=YOUR_32_CHAR_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("api_key".to_string(), key.clone());
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

impl Default for FredAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
