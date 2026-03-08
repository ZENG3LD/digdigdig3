//! AlienVault OTX authentication
//!
//! Authentication type: API Key (header)
//!
//! OTX uses API key authentication via HTTP header.
//! The API key is passed as `X-OTX-API-KEY: YOUR_API_KEY` in the request headers.

use std::collections::HashMap;

/// AlienVault OTX authentication credentials
#[derive(Clone)]
pub struct OtxAuth {
    pub api_key: Option<String>,
}

impl OtxAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OTX_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OTX_API_KEY").ok(),
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
    /// OTX requires API key as a header:
    /// `X-OTX-API-KEY: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("X-OTX-API-KEY".to_string(), key.clone());
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

impl Default for OtxAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
