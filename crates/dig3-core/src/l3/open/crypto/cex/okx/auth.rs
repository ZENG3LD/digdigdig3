//! # OKX Authentication
//!
//! Реализация подписи запросов для OKX API v5.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать prehash: `timestamp + method + requestPath + body`
//! 2. HMAC-SHA256 с secret key
//! 3. Encode в Base64
//!
//! ## Headers
//!
//! - `OK-ACCESS-KEY` - API key
//! - `OK-ACCESS-SIGN` - Signature (Base64)
//! - `OK-ACCESS-TIMESTAMP` - Timestamp (ISO 8601 с миллисекундами)
//! - `OK-ACCESS-PASSPHRASE` - Passphrase (plain text, не шифруется)
//!
//! ## Отличия от KuCoin
//!
//! - OKX использует ISO 8601 timestamp вместо миллисекунд
//! - Passphrase НЕ шифруется (отправляется как есть)
//! - Headers начинаются с `OK-ACCESS-` вместо `KC-API-`

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, encode_base64, timestamp_iso8601,
    Credentials, ExchangeResult, ExchangeError,
};

/// OKX аутентификация
#[derive(Clone)]
pub struct OkxAuth {
    api_key: String,
    api_secret: String,
    pub passphrase: String, // Public for WebSocket login
}

impl OkxAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        let passphrase = credentials.passphrase.clone()
            .ok_or_else(|| ExchangeError::Auth("OKX requires passphrase".to_string()))?;

        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            passphrase,
        })
    }

    /// Подписать запрос и вернуть headers
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, DELETE)
    /// * `endpoint` - Request path including query params (e.g., `/api/v5/account/balance?ccy=BTC`)
    /// * `body` - JSON body as string (empty string for GET requests)
    pub fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let timestamp = timestamp_iso8601();

        // Prehash: timestamp + method + requestPath + body
        let prehash = format!("{}{}{}{}", timestamp, method.to_uppercase(), endpoint, body);

        // Sign with HMAC-SHA256 and encode to Base64
        let signature = encode_base64(&hmac_sha256(
            self.api_secret.as_bytes(),
            prehash.as_bytes(),
        ));

        let mut headers = HashMap::new();
        headers.insert("OK-ACCESS-KEY".to_string(), self.api_key.clone());
        headers.insert("OK-ACCESS-SIGN".to_string(), signature);
        headers.insert("OK-ACCESS-TIMESTAMP".to_string(), timestamp);
        headers.insert("OK-ACCESS-PASSPHRASE".to_string(), self.passphrase.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }

    /// Создать headers для демо-торговли (testnet)
    pub fn sign_request_testnet(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let mut headers = self.sign_request(method, endpoint, body);
        headers.insert("x-simulated-trading".to_string(), "1".to_string());
        headers
    }

    /// Получить API key (для headers без подписи)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Создать WebSocket login signature
    ///
    /// Для WebSocket login используется специальный prehash:
    /// `timestamp + "GET" + "/users/self/verify"`
    pub fn sign_websocket_login(&self, timestamp: &str) -> String {
        let prehash = format!("{}GET/users/self/verify", timestamp);
        encode_base64(&hmac_sha256(
            self.api_secret.as_bytes(),
            prehash.as_bytes(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = OkxAuth::new(&credentials).unwrap();
        let headers = auth.sign_request("GET", "/api/v5/account/balance", "");

        assert!(headers.contains_key("OK-ACCESS-KEY"));
        assert!(headers.contains_key("OK-ACCESS-SIGN"));
        assert!(headers.contains_key("OK-ACCESS-TIMESTAMP"));
        assert!(headers.contains_key("OK-ACCESS-PASSPHRASE"));
        assert_eq!(headers.get("OK-ACCESS-KEY"), Some(&"test_key".to_string()));
        assert_eq!(headers.get("OK-ACCESS-PASSPHRASE"), Some(&"test_pass".to_string()));
    }

    #[test]
    fn test_sign_request_testnet() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = OkxAuth::new(&credentials).unwrap();
        let headers = auth.sign_request_testnet("POST", "/api/v5/trade/order", r#"{"instId":"BTC-USDT"}"#);

        assert_eq!(headers.get("x-simulated-trading"), Some(&"1".to_string()));
    }

    #[test]
    fn test_websocket_login_signature() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = OkxAuth::new(&credentials).unwrap();
        let timestamp = "2020-12-08T09:08:57.715Z";
        let signature = auth.sign_websocket_login(timestamp);

        // Should produce a base64 string
        assert!(!signature.is_empty());
        assert!(signature.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }
}
