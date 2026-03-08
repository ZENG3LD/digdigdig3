//! # KuCoin Authentication
//!
//! Реализация подписи запросов для KuCoin API.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать строку: `timestamp + method + endpoint + body`
//! 2. HMAC-SHA256 с secret key
//! 3. Encode в Base64
//! 4. Passphrase тоже подписывается HMAC-SHA256 + Base64
//!
//! ## Headers
//!
//! - `KC-API-KEY` - API key
//! - `KC-API-SIGN` - Signature (Base64)
//! - `KC-API-TIMESTAMP` - Timestamp (ms)
//! - `KC-API-PASSPHRASE` - Encrypted passphrase (Base64)
//! - `KC-API-KEY-VERSION` - "2" (для encrypted passphrase)

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, encode_base64, timestamp_millis,
    Credentials, ExchangeResult, ExchangeError,
};

/// KuCoin аутентификация
#[derive(Clone)]
pub struct KuCoinAuth {
    api_key: String,
    api_secret: String,
    passphrase: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// Positive = server is ahead, Negative = server is behind
    time_offset_ms: i64,
}

impl KuCoinAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        let passphrase = credentials.passphrase.clone()
            .ok_or_else(|| ExchangeError::Auth("KuCoin requires passphrase".to_string()))?;

        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            passphrase,
            time_offset_ms: 0,
        })
    }

    /// Sync time with server
    /// Call this with server timestamp from /api/v1/timestamp response
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
    pub fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let timestamp = self.get_timestamp();
        let timestamp_str = timestamp.to_string();

        // Signature: timestamp + method + endpoint + body
        let sign_string = format!("{}{}{}{}", timestamp_str, method.to_uppercase(), endpoint, body);
        let signature = encode_base64(&hmac_sha256(
            self.api_secret.as_bytes(),
            sign_string.as_bytes(),
        ));

        // Encrypted passphrase
        let encrypted_passphrase = encode_base64(&hmac_sha256(
            self.api_secret.as_bytes(),
            self.passphrase.as_bytes(),
        ));

        let mut headers = HashMap::new();
        headers.insert("KC-API-KEY".to_string(), self.api_key.clone());
        headers.insert("KC-API-SIGN".to_string(), signature);
        headers.insert("KC-API-TIMESTAMP".to_string(), timestamp_str);
        headers.insert("KC-API-PASSPHRASE".to_string(), encrypted_passphrase);
        headers.insert("KC-API-KEY-VERSION".to_string(), "2".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        headers
    }

    /// Получить API key (для headers без подписи)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret")
            .with_passphrase("test_pass");

        let auth = KuCoinAuth::new(&credentials).unwrap();
        let headers = auth.sign_request("GET", "/api/v1/accounts", "");

        assert!(headers.contains_key("KC-API-KEY"));
        assert!(headers.contains_key("KC-API-SIGN"));
        assert!(headers.contains_key("KC-API-TIMESTAMP"));
        assert!(headers.contains_key("KC-API-PASSPHRASE"));
        assert_eq!(headers.get("KC-API-KEY-VERSION"), Some(&"2".to_string()));
    }
}
