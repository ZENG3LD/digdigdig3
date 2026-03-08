//! VirusTotal authentication
//!
//! Authentication type: API Key (header)
//!
//! VirusTotal uses API key authentication via the `x-apikey` header.

use std::collections::HashMap;

/// VirusTotal authentication credentials
#[derive(Clone)]
pub struct VirusTotalAuth {
    pub api_key: Option<String>,
}

impl VirusTotalAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `VIRUSTOTAL_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("VIRUSTOTAL_API_KEY").ok(),
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
    /// VirusTotal requires API key in the `x-apikey` header.
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("x-apikey".to_string(), key.clone());
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

impl Default for VirusTotalAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
