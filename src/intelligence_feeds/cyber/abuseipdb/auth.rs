//! AbuseIPDB authentication
//!
//! Authentication type: API Key (header)
//!
//! AbuseIPDB uses API key authentication via HTTP header.
//! The API key is passed as `Key: YOUR_API_KEY` in the request headers.

use std::collections::HashMap;

/// AbuseIPDB authentication credentials
#[derive(Clone)]
pub struct AbuseIpdbAuth {
    pub api_key: Option<String>,
}

impl AbuseIpdbAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `ABUSEIPDB_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ABUSEIPDB_API_KEY").ok(),
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
    /// AbuseIPDB requires API key as a header:
    /// `Key: YOUR_API_KEY`
    /// `Accept: application/json`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("Key".to_string(), key.clone());
        }
        headers.insert("Accept".to_string(), "application/json".to_string());
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

impl Default for AbuseIpdbAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
