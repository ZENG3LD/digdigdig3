//! AviationStack authentication
//!
//! Authentication type: API Key via query parameter
//!
//! AviationStack uses API key authentication passed as a query parameter:
//! - Query param: access_key=YOUR_API_KEY
//!
//! Free tier: 100 requests per month (HTTP only)

use std::collections::HashMap;

/// AviationStack authentication credentials
#[derive(Clone)]
pub struct AviationStackAuth {
    pub api_key: String,
}

impl AviationStackAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `AVIATIONSTACK_API_KEY`
    pub fn from_env() -> Self {
        let api_key = std::env::var("AVIATIONSTACK_API_KEY")
            .expect("AVIATIONSTACK_API_KEY environment variable not set");
        Self { api_key }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Add authentication to query parameters
    ///
    /// AviationStack requires API key as query parameter:
    /// `access_key=YOUR_API_KEY`
    pub fn sign_params(&self, params: &mut HashMap<String, String>) {
        params.insert("access_key".to_string(), self.api_key.clone());
    }

    /// Get API key (for debugging/logging)
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

impl Default for AviationStackAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
