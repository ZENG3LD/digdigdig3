//! GFW authentication
//!
//! Authentication type: API Key (HTTP header)
//!
//! GFW uses API key authentication via the x-api-key header.

use std::collections::HashMap;

/// GFW authentication credentials
#[derive(Clone)]
pub struct GfwAuth {
    pub api_key: Option<String>,
}

impl GfwAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `GFW_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("GFW_API_KEY").ok(),
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
    /// GFW requires API key as a header:
    /// `x-api-key: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("x-api-key".to_string(), key.clone());
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

impl Default for GfwAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
