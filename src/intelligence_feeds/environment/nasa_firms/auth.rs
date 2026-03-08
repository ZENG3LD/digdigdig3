//! NASA FIRMS authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! NASA FIRMS requires API key authentication via MAP_KEY query parameter.
//! API keys are available for free at firms.modaps.eosdis.nasa.gov/api/

use std::collections::HashMap;

/// NASA FIRMS authentication credentials
#[derive(Clone)]
pub struct NasaFirmsAuth {
    pub api_key: String,
}

impl NasaFirmsAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `NASA_FIRMS_KEY`
    pub fn from_env() -> Self {
        let api_key = std::env::var("NASA_FIRMS_KEY")
            .expect("NASA_FIRMS_KEY environment variable not set");
        Self { api_key }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Add authentication to request query parameters
    ///
    /// NASA FIRMS uses MAP_KEY query parameter:
    /// `?MAP_KEY=YOUR_API_KEY`
    pub fn sign_params(&self, params: &mut HashMap<String, String>) {
        params.insert("MAP_KEY".to_string(), self.api_key.clone());
    }

    /// Get API key (for debugging/logging - use carefully)
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }
}

impl Default for NasaFirmsAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
