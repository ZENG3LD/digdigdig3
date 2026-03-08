//! OpenCorporates authentication
//!
//! Authentication type: API Token (query parameter)
//!
//! OpenCorporates allows requests without authentication (free tier) but
//! provides higher rate limits with an API token passed as query parameter.

use std::collections::HashMap;

/// OpenCorporates authentication credentials
#[derive(Clone)]
pub struct OpenCorporatesAuth {
    pub api_token: Option<String>,
}

impl OpenCorporatesAuth {
    /// Create new auth from environment variables
    ///
    /// Expects environment variable: `OPENCORPORATES_API_TOKEN`
    pub fn from_env() -> Self {
        Self {
            api_token: std::env::var("OPENCORPORATES_API_TOKEN").ok(),
        }
    }

    /// Create auth with explicit API token
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            api_token: Some(api_token.into()),
        }
    }

    /// Create auth without token (free tier)
    pub fn anonymous() -> Self {
        Self {
            api_token: None,
        }
    }

    /// Add authentication to query parameters
    ///
    /// OpenCorporates requires API token as `api_token` parameter (optional)
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        // Add API token if available
        if let Some(token) = &self.api_token {
            params.insert("api_token".to_string(), token.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_token.is_some()
    }

    /// Get API token (for debugging/logging - use carefully)
    pub fn get_api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }
}

impl Default for OpenCorporatesAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
