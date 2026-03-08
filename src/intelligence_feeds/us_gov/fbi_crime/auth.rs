//! FBI Crime Data API authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! FBI Crime Data API requires an API key as a query parameter.
//! The API key is provided by api.data.gov (free registration).

use std::collections::HashMap;

/// FBI Crime Data API authentication credentials
#[derive(Clone)]
pub struct FbiCrimeAuth {
    pub api_key: Option<String>,
}

impl FbiCrimeAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variable: `FBI_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("FBI_API_KEY").ok(),
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
    /// FBI Crime Data API requires:
    /// - API key as `api_key` parameter
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        // Add API key
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

impl Default for FbiCrimeAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
