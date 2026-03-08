//! # Tiingo Authentication
//!
//! Simple API token authentication for Tiingo API.
//!
//! ## Authentication Methods
//!
//! Tiingo supports two methods for REST:
//! 1. Authorization header: `Authorization: Token YOUR_API_KEY` (recommended)
//! 2. Query parameter: `?token=YOUR_API_KEY`
//!
//! For WebSocket:
//! - Include `authorization` field in subscribe message
//!
//! This implementation uses the Authorization header method for REST.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Tiingo authentication handler
#[derive(Clone)]
pub struct TiingoAuth {
    api_token: String,
}

impl TiingoAuth {
    /// Create new auth handler from credentials
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_key.is_empty() {
            return Err(ExchangeError::Auth("Tiingo requires API token".to_string()));
        }

        Ok(Self {
            api_token: credentials.api_key.clone(),
        })
    }

    /// Get Authorization header for REST API
    /// Format: "Authorization: Token YOUR_API_KEY"
    pub fn get_auth_header(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Token {}", self.api_token)
        );
        headers.insert(
            "Content-Type".to_string(),
            "application/json".to_string()
        );
        headers
    }

    /// Add API token as query parameter (alternative method)
    /// Parameter: ?token=YOUR_API_KEY
    pub fn add_to_params(&self, params: &mut HashMap<String, String>) {
        params.insert("token".to_string(), self.api_token.clone());
    }

    /// Get API token directly
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    /// Get WebSocket auth field for subscribe message
    /// WebSocket requires `authorization` field (not "Token" prefix)
    pub fn ws_auth_token(&self) -> &str {
        &self.api_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header() {
        let credentials = Credentials::new("test_api_token", "");
        let auth = TiingoAuth::new(&credentials).unwrap();

        let headers = auth.get_auth_header();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Token test_api_token".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_auth_params() {
        let credentials = Credentials::new("test_api_token", "");
        let auth = TiingoAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        auth.add_to_params(&mut params);

        assert_eq!(params.get("token"), Some(&"test_api_token".to_string()));
    }

    #[test]
    fn test_ws_auth_token() {
        let credentials = Credentials::new("test_api_token", "");
        let auth = TiingoAuth::new(&credentials).unwrap();

        assert_eq!(auth.ws_auth_token(), "test_api_token");
    }

    #[test]
    fn test_empty_api_key() {
        let credentials = Credentials::new("", "");
        let result = TiingoAuth::new(&credentials);

        assert!(result.is_err());
    }
}
