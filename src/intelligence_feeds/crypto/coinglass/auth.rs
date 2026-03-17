//! # Coinglass Authentication
//!
//! Реализация аутентификации для Coinglass API V4.
//!
//! ## Алгоритм
//!
//! Coinglass uses simple API key authentication:
//! - REST API: API key in header `CG-API-KEY`
//! - WebSocket: API key as query parameter `cg-api-key`
//!
//! No signing, no timestamps, no complex HMAC.
//! Just include the API key in every request.

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

/// Coinglass аутентификация
#[derive(Clone)]
pub struct CoinglassAuth {
    api_key: String,
}

impl CoinglassAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        if credentials.api_key.is_empty() {
            return Err(ExchangeError::Auth("Coinglass requires API key".to_string()));
        }

        Ok(Self {
            api_key: credentials.api_key.clone(),
        })
    }

    /// Get authentication headers for REST API
    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("CG-API-KEY".to_string(), self.api_key.clone());
        headers.insert("accept".to_string(), "application/json".to_string());
        headers
    }

    /// Get WebSocket URL with authentication
    pub fn get_ws_url(&self, base_ws_url: &str) -> String {
        format!("{}?cg-api-key={}", base_ws_url, self.api_key)
    }

    /// Get API key (for testing/debugging)
    #[cfg(test)]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_creation() {
        let credentials = Credentials {
            api_key: "test_key_123".to_string(),
            api_secret: String::new(),
            passphrase: None,
            testnet: false,
        };

        let auth = CoinglassAuth::new(&credentials).unwrap();
        assert_eq!(auth.api_key(), "test_key_123");
    }

    #[test]
    fn test_auth_headers() {
        let credentials = Credentials {
            api_key: "test_key_123".to_string(),
            api_secret: String::new(),
            passphrase: None,
            testnet: false,
        };

        let auth = CoinglassAuth::new(&credentials).unwrap();
        let headers = auth.get_headers();

        assert_eq!(headers.get("CG-API-KEY").unwrap(), "test_key_123");
        assert_eq!(headers.get("accept").unwrap(), "application/json");
    }

    #[test]
    fn test_ws_url() {
        let credentials = Credentials {
            api_key: "test_key_123".to_string(),
            api_secret: String::new(),
            passphrase: None,
            testnet: false,
        };

        let auth = CoinglassAuth::new(&credentials).unwrap();
        let ws_url = auth.get_ws_url("wss://open-ws.coinglass.com/ws-api");

        assert_eq!(ws_url, "wss://open-ws.coinglass.com/ws-api?cg-api-key=test_key_123");
    }

    #[test]
    fn test_missing_api_key() {
        let credentials = Credentials {
            api_key: String::new(),
            api_secret: String::new(),
            passphrase: None,
            testnet: false,
        };

        let result = CoinglassAuth::new(&credentials);
        assert!(result.is_err());
    }
}
