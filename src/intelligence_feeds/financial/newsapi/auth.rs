//! NewsAPI authentication
//!
//! Authentication type: API Key (header or query parameter)
//!
//! NewsAPI supports two authentication methods:
//! 1. Header: `X-Api-Key: YOUR_API_KEY`
//! 2. Query param: `apiKey=YOUR_API_KEY`
//!
//! We use the header method as it's more secure.

use reqwest::header::HeaderMap;
use std::collections::HashMap;

/// NewsAPI authentication credentials
#[derive(Clone)]
pub struct NewsApiAuth {
    pub api_key: Option<String>,
}

impl NewsApiAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `NEWSAPI_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("NEWSAPI_KEY").ok(),
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
    /// NewsAPI supports header-based authentication:
    /// `X-Api-Key: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HeaderMap) {
        if let Some(key) = &self.api_key {
            if let Ok(header_value) = key.parse() {
                headers.insert("X-Api-Key", header_value);
            }
        }
    }

    /// Add authentication to query parameters (alternative method)
    ///
    /// NewsAPI also supports query param authentication:
    /// `?apiKey=YOUR_API_KEY`
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

impl Default for NewsApiAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
