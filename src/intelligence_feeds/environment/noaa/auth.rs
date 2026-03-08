//! NOAA CDO authentication
//!
//! Authentication type: API Key (header)
//!
//! NOAA CDO uses API key authentication via HTTP header.
//! The key is sent as: `token: YOUR_API_KEY`

use std::collections::HashMap;

/// NOAA CDO authentication credentials
#[derive(Clone)]
pub struct NoaaAuth {
    pub api_key: Option<String>,
}

impl NoaaAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `NOAA_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("NOAA_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication to headers
    ///
    /// NOAA CDO requires API key as a header:
    /// `token: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("token".to_string(), key.clone());
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

impl Default for NoaaAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
