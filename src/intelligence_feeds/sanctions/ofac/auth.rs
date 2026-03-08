//! OFAC API authentication
//!
//! Authentication type: API Key (header)
//!
//! OFAC API uses header-based authentication:
//! `apiKey: YOUR_API_KEY`

use reqwest::header::HeaderMap;
use std::collections::HashMap;

/// OFAC API authentication credentials
#[derive(Clone)]
pub struct OfacAuth {
    pub api_key: Option<String>,
}

impl OfacAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OFAC_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OFAC_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication to request headers
    ///
    /// OFAC API uses header-based authentication:
    /// `apiKey: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HeaderMap) {
        if let Some(key) = &self.api_key {
            if let Ok(header_value) = key.parse() {
                headers.insert("apiKey", header_value);
            }
        }
    }

    /// Add authentication to query parameters (alternative method)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("apiKey".to_string(), key.clone());
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

impl Default for OfacAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
