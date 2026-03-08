//! OpenFIGI authentication
//!
//! Authentication type: Optional API Key (HTTP header)
//!
//! OpenFIGI uses optional API key authentication via HTTP header.
//! Without API key: 5 requests/min, 100 jobs/request
//! With API key: 25 requests/min, 100 jobs/request

use std::collections::HashMap;

/// OpenFIGI authentication credentials
#[derive(Clone)]
pub struct OpenFigiAuth {
    pub api_key: Option<String>,
}

impl OpenFigiAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OPENFIGI_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENFIGI_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (free tier with lower limits)
    pub fn no_auth() -> Self {
        Self {
            api_key: None,
        }
    }

    /// Add authentication to HTTP headers
    ///
    /// OpenFIGI requires API key as HTTP header:
    /// `X-OPENFIGI-APIKEY: YOUR_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("X-OPENFIGI-APIKEY".to_string(), key.clone());
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

impl Default for OpenFigiAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
