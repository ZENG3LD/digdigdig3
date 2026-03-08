//! EU TED authentication
//!
//! Authentication type: None (public API)
//!
//! EU TED API is publicly accessible without authentication.
//! Optional API key via EU Login can be used for higher rate limits.

use std::collections::HashMap;

/// EU TED authentication credentials
#[derive(Clone)]
pub struct EuTedAuth {
    pub api_key: Option<String>,
}

impl EuTedAuth {
    /// Create new auth from environment variable
    ///
    /// Expects optional environment variable: `EU_TED_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("EU_TED_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (public access)
    pub fn public() -> Self {
        Self { api_key: None }
    }

    /// Add authentication to headers if API key is present
    ///
    /// EU TED may use API key as a header (implementation-dependent)
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

impl Default for EuTedAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
