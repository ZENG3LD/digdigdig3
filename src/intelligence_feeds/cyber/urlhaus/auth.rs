//! URLhaus (abuse.ch) authentication
//!
//! Authentication type: API Key (header)
//!
//! URLhaus uses API key authentication via HTTP header.
//! The API key is passed as `Auth-Key: YOUR_API_KEY` in the request headers.

use std::collections::HashMap;

/// URLhaus authentication credentials
#[derive(Clone)]
pub struct UrlhausAuth {
    pub auth_key: Option<String>,
}

impl UrlhausAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `URLHAUS_AUTH_KEY`
    pub fn from_env() -> Self {
        Self {
            auth_key: std::env::var("URLHAUS_AUTH_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(auth_key: impl Into<String>) -> Self {
        Self {
            auth_key: Some(auth_key.into()),
        }
    }

    /// Add authentication to request headers
    ///
    /// URLhaus requires API key as a header:
    /// `Auth-Key: YOUR_API_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.auth_key {
            headers.insert("Auth-Key".to_string(), key.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth_key.is_some()
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_auth_key(&self) -> Option<&str> {
        self.auth_key.as_deref()
    }
}

impl Default for UrlhausAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
