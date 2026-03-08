//! NVD authentication
//!
//! Authentication type: API Key (header-based, optional)
//!
//! NVD uses optional API key authentication via HTTP header.
//! Header name: `apiKey`
//! Without API key: 5 requests per 30 seconds
//! With API key: 50 requests per 30 seconds

use std::collections::HashMap;

/// NVD authentication credentials
#[derive(Clone)]
pub struct NvdAuth {
    pub api_key: Option<String>,
}

impl NvdAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `NVD_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("NVD_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (will use lower rate limits)
    pub fn public() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to request headers
    ///
    /// NVD requires API key as a header:
    /// `apiKey: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("apiKey".to_string(), key.clone());
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

impl Default for NvdAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
