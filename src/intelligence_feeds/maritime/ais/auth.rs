//! AIS authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! Datalastic AIS uses simple API key authentication via query parameter.
//! Just append `api-key=YOUR_KEY` to the URL.

use std::collections::HashMap;

/// AIS authentication credentials
#[derive(Clone)]
pub struct AisAuth {
    pub api_key: Option<String>,
}

impl AisAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `AIS_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("AIS_API_KEY").ok(),
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
    /// AIS requires API key as a query parameter:
    /// `?api-key=YOUR_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("api-key".to_string(), key.clone());
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

impl Default for AisAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
