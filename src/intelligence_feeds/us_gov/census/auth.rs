//! US Census Bureau authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! Census uses simple API key authentication via query parameter.
//! The API key is free and can be obtained from: https://api.census.gov/data/key_signup.html

use std::collections::HashMap;

/// Census authentication credentials
#[derive(Clone)]
pub struct CensusAuth {
    pub api_key: Option<String>,
}

impl CensusAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `CENSUS_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("CENSUS_API_KEY").ok(),
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    /// Add authentication to query parameters
    ///
    /// Census requires API key as a query parameter:
    /// `?key=YOUR_API_KEY`
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("key".to_string(), key.clone());
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

impl Default for CensusAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
