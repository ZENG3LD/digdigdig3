//! ADS-B Exchange authentication
//!
//! Authentication type: RapidAPI Key (headers)
//!
//! ADS-B Exchange via RapidAPI requires two headers:
//! - X-RapidAPI-Key: your_api_key
//! - X-RapidAPI-Host: adsbexchange-com1.p.rapidapi.com

use std::collections::HashMap;

/// ADS-B Exchange authentication credentials
#[derive(Clone)]
pub struct AdsbExchangeAuth {
    pub api_key: Option<String>,
    pub host: &'static str,
}

impl AdsbExchangeAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `ADSBX_API_KEY`
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("ADSBX_API_KEY").ok(),
            host: "adsbexchange-com1.p.rapidapi.com",
        }
    }

    /// Create auth with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            host: "adsbexchange-com1.p.rapidapi.com",
        }
    }

    /// Add authentication headers
    ///
    /// ADS-B Exchange via RapidAPI requires two headers:
    /// - `X-RapidAPI-Key: YOUR_KEY`
    /// - `X-RapidAPI-Host: adsbexchange-com1.p.rapidapi.com`
    pub fn add_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            headers.insert("X-RapidAPI-Key".to_string(), key.clone());
        }
        headers.insert("X-RapidAPI-Host".to_string(), self.host.to_string());
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

impl Default for AdsbExchangeAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
