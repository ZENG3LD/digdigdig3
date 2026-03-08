//! OpenWeatherMap authentication
//!
//! Authentication type: API Key (query parameter)
//!
//! OpenWeatherMap uses simple API key authentication via query parameter.
//! Just append `appid=YOUR_KEY` to the URL.

use std::collections::HashMap;

/// OpenWeatherMap authentication credentials
#[derive(Clone)]
pub struct OpenWeatherMapAuth {
    pub api_key: Option<String>,
}

impl OpenWeatherMapAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `OPENWEATHERMAP_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENWEATHERMAP_API_KEY").ok(),
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
    /// OpenWeatherMap requires API key as a query parameter:
    /// `?appid=YOUR_API_KEY`
    ///
    /// Also adds units=metric by default
    pub fn sign_query(&self, params: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            params.insert("appid".to_string(), key.clone());
        }
        // Always use metric units
        params.insert("units".to_string(), "metric".to_string());
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

impl Default for OpenWeatherMapAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
