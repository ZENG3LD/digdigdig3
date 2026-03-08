//! NASA authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! NASA uses simple API key authentication via query parameter.
//! No HMAC signatures, no headers - just append `api_key=YOUR_KEY` to the URL.
//!
//! Default key: DEMO_KEY (rate limit: 30 requests/hour)
//! Custom key: NASA_API_KEY environment variable (rate limit: 1000 requests/hour)

use std::collections::HashMap;

/// NASA authentication credentials
#[derive(Clone)]
pub struct NasaAuth {
    pub api_key: String,
}

impl NasaAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `NASA_API_KEY`
    /// Falls back to "DEMO_KEY" if not set
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("NASA_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string()),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Add authentication to query parameters
    ///
    /// NASA requires API key as a query parameter:
    /// `?api_key=YOUR_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        params.insert("api_key".to_string(), self.api_key.clone());
    }

    /// Check if using DEMO_KEY (for rate limit awareness)
    pub fn is_demo_key(&self) -> bool {
        self.api_key == "DEMO_KEY"
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

impl Default for NasaAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
