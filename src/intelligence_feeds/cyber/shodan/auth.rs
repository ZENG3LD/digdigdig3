//! Shodan authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! Shodan uses simple API key authentication via query parameter.
//! The API key is passed as `key=YOUR_API_KEY` in the query string.

use std::collections::HashMap;

/// Shodan authentication credentials
#[derive(Clone)]
pub struct ShodanAuth {
    pub api_key: Option<String>,
}

impl ShodanAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `SHODAN_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("SHODAN_API_KEY").ok(),
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
    /// Shodan requires API key as a query parameter:
    /// `?key=YOUR_API_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("key".to_string(), key.clone());
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

impl Default for ShodanAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
