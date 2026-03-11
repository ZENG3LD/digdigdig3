//! # Phemex Authentication
//!
//! Реализация подписи запросов для Phemex API.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать строку: `path + query + expiry + body`
//! 2. HMAC-SHA256 с secret key
//! 3. Encode в hex (lowercase)
//!
//! ## Headers
//!
//! - `x-phemex-access-token` - API key
//! - `x-phemex-request-expiry` - Unix timestamp (seconds)
//! - `x-phemex-request-signature` - Signature (hex)
//! - `Content-Type` - application/json

use std::collections::HashMap;

use crate::core::{
    hmac_sha256_hex, timestamp_seconds,
    Credentials, ExchangeResult,
};

/// Phemex аутентификация
#[derive(Clone)]
pub struct PhemexAuth {
    api_key: String,
    api_secret: String,
}

impl PhemexAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
        })
    }

    /// Generate expiry timestamp (current time + 60 seconds)
    fn get_expiry(&self) -> u64 {
        timestamp_seconds() + 60
    }

    /// Подписать запрос и вернуть headers
    ///
    /// # Message Format
    /// `path + query + expiry + body`
    ///
    /// # Notes
    /// - Query string should NOT include '?' prefix
    /// - Signature is hex-encoded (lowercase)
    pub fn sign_request(
        &self,
        path: &str,
        query: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let expiry = self.get_expiry();
        let expiry_str = expiry.to_string();

        // Signature message: path + query + expiry + body
        let sign_string = format!("{}{}{}{}", path, query, expiry_str, body);

        // HMAC SHA256, hex-encoded (lowercase)
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            sign_string.as_bytes(),
        );

        let mut headers = HashMap::new();
        headers.insert("x-phemex-access-token".to_string(), self.api_key.clone());
        headers.insert("x-phemex-request-expiry".to_string(), expiry_str);
        headers.insert("x-phemex-request-signature".to_string(), signature);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }

    /// Generate WebSocket signature (for private channels)
    ///
    /// # Message Format
    /// `api_key + expiry`
    pub fn sign_websocket(&self) -> (String, u64, String) {
        let expiry = self.get_expiry();
        let message = format!("{}{}", self.api_key, expiry);
        let signature = hmac_sha256_hex(
            self.api_secret.as_bytes(),
            message.as_bytes(),
        );

        (self.api_key.clone(), expiry, signature)
    }

    /// Получить API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request_get() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = PhemexAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("/spot/wallets", "", "");

        assert!(headers.contains_key("x-phemex-access-token"));
        assert!(headers.contains_key("x-phemex-request-expiry"));
        assert!(headers.contains_key("x-phemex-request-signature"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(headers.get("x-phemex-access-token"), Some(&"test_key".to_string()));
    }

    #[test]
    fn test_sign_request_with_query() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = PhemexAuth::new(&credentials).unwrap();

        // Query without '?' prefix
        let headers = auth.sign_request("/orders/activeList", "symbol=BTCUSD", "");

        assert!(headers.contains_key("x-phemex-request-signature"));
        // Signature should include query in the message
    }

    #[test]
    fn test_sign_request_with_body() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = PhemexAuth::new(&credentials).unwrap();

        let body = r#"{"symbol":"BTCUSD","side":"Buy"}"#;
        let headers = auth.sign_request("/orders", "", body);

        assert!(headers.contains_key("x-phemex-request-signature"));
        // Signature should include body in the message
    }

    #[test]
    fn test_sign_websocket() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = PhemexAuth::new(&credentials).unwrap();

        let (api_key, expiry, signature) = auth.sign_websocket();

        assert_eq!(api_key, "test_key");
        assert!(expiry > timestamp_seconds());
        assert!(!signature.is_empty());
        // Signature should be hex string (64 characters for SHA256)
        assert_eq!(signature.len(), 64);
    }
}
