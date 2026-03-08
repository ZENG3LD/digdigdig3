//! OpenAQ authentication
//!
//! Authentication type: API Key (header, optional)
//!
//! OpenAQ supports optional API key authentication via X-API-Key header.
//! Basic access is available without authentication, but API key provides better rate limits.

use std::collections::HashMap;

/// OpenAQ authentication credentials
#[derive(Clone)]
pub struct OpenAqAuth {
    pub api_key: Option<String>,
}

impl OpenAqAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OPENAQ_API_KEY` (optional)
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENAQ_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create public access (no authentication)
    pub fn public() -> Self {
        Self {
            api_key: None,
        }
    }

    /// Add authentication to request headers
    ///
    /// OpenAQ uses X-API-Key header:
    /// `X-API-Key: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("X-API-Key".to_string(), key.clone());
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

impl Default for OpenAqAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
