//! UN COMTRADE authentication
//!
//! Authentication type: API Key (header)
//!
//! COMTRADE uses subscription key authentication via HTTP header.
//! Header: `Ocp-Apim-Subscription-Key: YOUR_KEY`

use std::collections::HashMap;

/// COMTRADE authentication credentials
#[derive(Clone)]
pub struct ComtradeAuth {
    pub api_key: Option<String>,
}

impl ComtradeAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `COMTRADE_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("COMTRADE_API_KEY").ok(),
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
    /// COMTRADE requires API key as a header:
    /// `Ocp-Apim-Subscription-Key: YOUR_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("Ocp-Apim-Subscription-Key".to_string(), key.clone());
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

impl Default for ComtradeAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
