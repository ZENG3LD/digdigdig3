//! # BingX Authentication
//!
//! Реализация подписи запросов для BingX API.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать все параметры (кроме signature) в query string: `key1=value1&key2=value2`
//! 2. Добавить timestamp в milliseconds
//! 3. HMAC-SHA256 с secret key
//! 4. Encode в hex (lowercase)
//!
//! ## Headers
//!
//! - `X-BX-APIKEY` - API key
//!
//! ## Parameters
//!
//! - `timestamp` - Request timestamp (ms)
//! - `signature` - HMAC-SHA256 signature (hex)
//! - `recvWindow` - Optional validity window (default: 5000ms)

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, encode_hex_lower, timestamp_millis,
    Credentials, ExchangeResult,
};

/// BingX аутентификация
#[derive(Clone)]
pub struct BingxAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (milliseconds)
    time_offset_ms: i64,
}

impl BingxAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset_ms: 0,
        })
    }

    /// Sync time with server
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = timestamp_millis() as i64;
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp (local + offset = ~server time)
    fn get_timestamp(&self) -> u64 {
        let local = timestamp_millis() as i64;
        (local + self.time_offset_ms) as u64
    }

    /// Подписать запрос и вернуть параметры с signature
    ///
    /// BingX signature process:
    /// 1. Build query string from all params (alphabetically sorted recommended)
    /// 2. Add timestamp parameter
    /// 3. Generate HMAC-SHA256 signature
    /// 4. Add signature parameter
    /// 5. Return updated params map
    pub fn sign_request(
        &self,
        params: &mut HashMap<String, String>,
    ) -> HashMap<String, String> {
        // Add timestamp
        let timestamp = self.get_timestamp();
        params.insert("timestamp".to_string(), timestamp.to_string());

        // Build parameter string (sorted for consistency)
        let param_string = self.build_query_string(params);

        // Generate signature: HMAC-SHA256 -> hex (lowercase)
        let signature_bytes = hmac_sha256(
            self.api_secret.as_bytes(),
            param_string.as_bytes(),
        );
        let signature = encode_hex_lower(&signature_bytes);

        // Add signature to params
        params.insert("signature".to_string(), signature);

        // Return headers
        let mut headers = HashMap::new();
        headers.insert("X-BX-APIKEY".to_string(), self.api_key.clone());
        headers
    }

    /// Build query string from parameters (sorted)
    fn build_query_string(&self, params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<String> = params
            .iter()
            .filter(|(k, _)| k.as_str() != "signature") // Exclude signature from signing
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        pairs.sort(); // Sort for consistency
        pairs.join("&")
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
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BingxAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTC-USDT".to_string());
        params.insert("side".to_string(), "BUY".to_string());

        let headers = auth.sign_request(&mut params);

        // Check headers
        assert_eq!(headers.get("X-BX-APIKEY"), Some(&"test_key".to_string()));

        // Check params include timestamp and signature
        assert!(params.contains_key("timestamp"));
        assert!(params.contains_key("signature"));
        assert_eq!(params.get("symbol"), Some(&"BTC-USDT".to_string()));
    }

    #[test]
    fn test_query_string_sorted() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BingxAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("z_last".to_string(), "value1".to_string());
        params.insert("a_first".to_string(), "value2".to_string());
        params.insert("m_middle".to_string(), "value3".to_string());

        let query = auth.build_query_string(&params);

        // Should be sorted alphabetically
        assert_eq!(query, "a_first=value2&m_middle=value3&z_last=value1");
    }
}
