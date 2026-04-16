//! # Bitget Authentication
//!
//! Реализация подписи запросов для Bitget API.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать строку: `timestamp + method + requestPath + queryString + body`
//! 2. HMAC-SHA256 с secret key
//! 3. Encode в Base64
//!
//! ## Headers
//!
//! - `ACCESS-KEY` - API key
//! - `ACCESS-SIGN` - Signature (Base64)
//! - `ACCESS-TIMESTAMP` - Timestamp (ms)
//! - `ACCESS-PASSPHRASE` - Passphrase
//! - `Content-Type` - application/json

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, encode_base64, timestamp_millis,
    Credentials, ExchangeResult, ExchangeError,
};

/// Bitget аутентификация
#[derive(Clone)]
pub struct BitgetAuth {
    api_key: String,
    api_secret: String,
    passphrase: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// Positive = server is ahead, Negative = server is behind
    time_offset_ms: i64,
}

impl BitgetAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        let passphrase = credentials.passphrase.clone()
            .ok_or_else(|| ExchangeError::Auth("Bitget requires passphrase".to_string()))?;

        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            passphrase,
            time_offset_ms: 0,
        })
    }

    /// Sync time with server
    /// Call this with server timestamp from /api/spot/v1/public/time response
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = timestamp_millis() as i64;
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp (local + offset = ~server time)
    fn get_timestamp(&self) -> u64 {
        let local = timestamp_millis() as i64;
        (local + self.time_offset_ms) as u64
    }

    /// Подписать запрос и вернуть headers
    ///
    /// # Prehash String Format
    /// `timestamp + method + requestPath + queryString + body`
    ///
    /// # Examples
    /// - GET with query: `1695806875837GET/api/spot/v1/market/ticker?symbol=BTCUSDT_SPBL`
    /// - GET without query: `1695806875837GET/api/spot/v1/account/assets`
    /// - POST: `1695806875837POST/api/spot/v1/trade/orders{"symbol":"BTCUSDT_SPBL",...}`
    pub fn sign_request(
        &self,
        method: &str,
        request_path: &str,
        query_string: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let timestamp = self.get_timestamp();
        let timestamp_str = timestamp.to_string();

        // Prehash: timestamp + method + requestPath + queryString + body
        // queryString includes the '?' if present
        let sign_string = format!(
            "{}{}{}{}{}",
            timestamp_str,
            method.to_uppercase(),
            request_path,
            query_string,
            body
        );

        let signature = encode_base64(&hmac_sha256(
            self.api_secret.as_bytes(),
            sign_string.as_bytes(),
        ));

        let mut headers = HashMap::new();
        headers.insert("ACCESS-KEY".to_string(), self.api_key.clone());
        headers.insert("ACCESS-SIGN".to_string(), signature);
        headers.insert("ACCESS-TIMESTAMP".to_string(), timestamp_str);
        headers.insert("ACCESS-PASSPHRASE".to_string(), self.passphrase.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }

    /// Получить API key (для headers без подписи)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Получить API secret (для WebSocket подписи)
    pub fn api_secret(&self) -> &str {
        &self.api_secret
    }

    /// Получить passphrase (для WebSocket аутентификации)
    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = BitgetAuth::new(&credentials).unwrap();
        let headers = auth.sign_request("GET", "/api/spot/v1/account/assets", "", "");

        assert!(headers.contains_key("ACCESS-KEY"));
        assert!(headers.contains_key("ACCESS-SIGN"));
        assert!(headers.contains_key("ACCESS-TIMESTAMP"));
        assert!(headers.contains_key("ACCESS-PASSPHRASE"));
        assert_eq!(headers.get("ACCESS-KEY"), Some(&"test_key".to_string()));
        assert_eq!(headers.get("ACCESS-PASSPHRASE"), Some(&"test_pass".to_string()));
    }

    #[test]
    fn test_sign_request_with_query() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = BitgetAuth::new(&credentials).unwrap();
        let headers = auth.sign_request(
            "GET",
            "/api/spot/v1/market/ticker",
            "?symbol=BTCUSDT_SPBL",
            ""
        );

        assert!(headers.contains_key("ACCESS-SIGN"));
        // Signature should be different from request without query
        let headers_no_query = auth.sign_request("GET", "/api/spot/v1/market/ticker", "", "");
        assert_ne!(
            headers.get("ACCESS-SIGN"),
            headers_no_query.get("ACCESS-SIGN")
        );
    }

    #[test]
    fn test_sign_request_post() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = BitgetAuth::new(&credentials).unwrap();
        let body = r#"{"symbol":"BTCUSDT_SPBL","side":"buy"}"#;
        let headers = auth.sign_request("POST", "/api/spot/v1/trade/orders", "", body);

        assert!(headers.contains_key("ACCESS-SIGN"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
}
