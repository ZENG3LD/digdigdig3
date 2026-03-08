//! OpenSanctions authentication
//!
//! Authentication type: API Key (Authorization header)
//!
//! OpenSanctions uses optional API key authentication via Authorization header.
//! Format: `Authorization: ApiKey YOUR_KEY`

use std::collections::HashMap;

/// OpenSanctions authentication credentials
#[derive(Clone)]
pub struct OpenSanctionsAuth {
    pub api_key: Option<String>,
}

impl OpenSanctionsAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OPENSANCTIONS_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENSANCTIONS_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Create auth without API key (uses free tier)
    pub fn anonymous() -> Self {
        Self {
            api_key: None,
        }
    }

    /// Add authentication to headers
    ///
    /// OpenSanctions requires API key as Authorization header:
    /// `Authorization: ApiKey YOUR_KEY`
    pub fn sign_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("Authorization".to_string(), format!("ApiKey {}", key));
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

impl Default for OpenSanctionsAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
