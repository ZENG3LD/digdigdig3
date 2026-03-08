//! # Polygon.io Authentication
//!
//! Simple API key authentication for Polygon.io API.
//!
//! ## Authentication Methods
//!
//! Polygon supports two methods:
//! 1. Query parameter: `?apiKey=YOUR_API_KEY` (recommended)
//! 2. Authorization header: `Authorization: Bearer YOUR_API_KEY`
//!
//! This implementation uses the query parameter method by default.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Polygon.io authentication
#[derive(Clone)]
pub struct PolygonAuth {
    api_key: String,
}

impl PolygonAuth {
    /// Create new auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_key.is_empty() {
            return Err(ExchangeError::Auth("Polygon requires API key".to_string()));
        }

        Ok(Self {
            api_key: credentials.api_key.clone(),
        })
    }

    /// Add API key as query parameter
    /// This is the recommended method for Polygon
    pub fn add_to_params(&self, params: &mut HashMap<String, String>) {
        params.insert("apiKey".to_string(), self.api_key.clone());
    }

    /// Get Authorization header (alternative method)
    /// Format: "Bearer YOUR_API_KEY"
    pub fn _get_auth_header(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        headers
    }

    /// Get API key directly
    pub fn _api_key(&self) -> &str {
        &self.api_key
    }

    /// Get WebSocket auth message
    /// WebSocket requires API key in params field
    pub fn ws_auth_message(&self) -> serde_json::Value {
        serde_json::json!({
            "action": "auth",
            "params": self.api_key
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_params() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = PolygonAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        auth.add_to_params(&mut params);

        assert_eq!(params.get("apiKey"), Some(&"test_api_key".to_string()));
    }

    #[test]
    fn test_auth_header() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = PolygonAuth::new(&credentials).unwrap();

        let headers = auth._get_auth_header();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test_api_key".to_string())
        );
    }

    #[test]
    fn test_ws_auth_message() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = PolygonAuth::new(&credentials).unwrap();

        let msg = auth.ws_auth_message();

        assert_eq!(msg["action"], "auth");
        assert_eq!(msg["params"], "test_api_key");
    }

    #[test]
    fn test_empty_api_key() {
        let credentials = Credentials::new("", "");
        let result = PolygonAuth::new(&credentials);

        assert!(result.is_err());
    }
}
