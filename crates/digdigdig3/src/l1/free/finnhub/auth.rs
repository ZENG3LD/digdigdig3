//! # Finnhub Authentication
//!
//! Simple API key authentication for Finnhub API.
//!
//! ## Authentication Methods
//!
//! Finnhub supports two methods:
//! 1. Header: `X-Finnhub-Token: YOUR_API_KEY` (recommended)
//! 2. Query parameter: `?token=YOUR_API_KEY` (alternative)
//!
//! This implementation uses the header method by default for security.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Finnhub authentication
#[derive(Clone)]
pub struct FinnhubAuth {
    api_key: String,
}

impl FinnhubAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_key.is_empty() {
            return Err(ExchangeError::Auth("Finnhub requires API key".to_string()));
        }

        Ok(Self {
            api_key: credentials.api_key.clone(),
        })
    }

    /// Get authentication header (recommended method)
    /// Format: "X-Finnhub-Token: YOUR_API_KEY"
    pub fn _get_auth_header(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("X-Finnhub-Token".to_string(), self.api_key.clone());
        headers
    }

    /// Add API key as query parameter (alternative method)
    pub fn add_to_params(&self, params: &mut HashMap<String, String>) {
        params.insert("token".to_string(), self.api_key.clone());
    }

    /// Get API key directly
    pub fn _api_key(&self) -> &str {
        &self.api_key
    }

    /// Get WebSocket URL with authentication
    /// WebSocket requires token as query parameter
    pub fn ws_url_with_auth(&self, base_url: &str) -> String {
        format!("{}?token={}", base_url, self.api_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = FinnhubAuth::new(&credentials).unwrap();

        let headers = auth._get_auth_header();

        assert_eq!(
            headers.get("X-Finnhub-Token"),
            Some(&"test_api_key".to_string())
        );
    }

    #[test]
    fn test_auth_params() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = FinnhubAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        auth.add_to_params(&mut params);

        assert_eq!(params.get("token"), Some(&"test_api_key".to_string()));
    }

    #[test]
    fn test_ws_url() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = FinnhubAuth::new(&credentials).unwrap();

        let ws_url = auth.ws_url_with_auth("wss://ws.finnhub.io");

        assert_eq!(ws_url, "wss://ws.finnhub.io?token=test_api_key");
    }

    #[test]
    fn test_empty_api_key() {
        let credentials = Credentials::new("", "");
        let result = FinnhubAuth::new(&credentials);

        assert!(result.is_err());
    }
}
