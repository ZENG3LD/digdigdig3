//! # Binance Authentication
//!
//! Реализация подписи запросов для Binance API.
//!
//! ## Алгоритм подписи
//!
//! 1. Собрать query string: `param1=value1&param2=value2&timestamp=...`
//! 2. HMAC-SHA256 с secret key
//! 3. Encode в hex (lowercase)
//! 4. Добавить signature в query string
//!
//! ## Headers
//!
//! - `X-MBX-APIKEY` - API key
//!
//! ## Query Parameters
//!
//! - `timestamp` - Timestamp (ms)
//! - `signature` - HMAC SHA256 signature (hex)

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, encode_hex, timestamp_millis,
    Credentials, ExchangeResult,
};

/// Binance аутентификация
#[derive(Clone)]
pub struct BinanceAuth {
    api_key: String,
    api_secret: String,
    /// Time offset: server_time - local_time (milliseconds)
    /// Positive = server is ahead, Negative = server is behind
    time_offset_ms: i64,
}

impl BinanceAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
            time_offset_ms: 0,
        })
    }

    /// Sync time with server
    /// Call this with server timestamp from /api/v3/time response
    pub fn sync_time(&mut self, server_time_ms: i64) {
        let local_time = timestamp_millis() as i64;
        self.time_offset_ms = server_time_ms - local_time;
    }

    /// Get adjusted timestamp (local + offset = ~server time)
    fn get_timestamp(&self) -> u64 {
        let local = timestamp_millis() as i64;
        (local + self.time_offset_ms) as u64
    }

    /// Подписать запрос и вернуть headers + query params
    ///
    /// # Parameters
    /// - `query_params`: Existing query parameters (will be modified to include timestamp & signature)
    ///
    /// # Returns
    /// HashMap with headers (X-MBX-APIKEY)
    pub fn sign_request(
        &self,
        query_params: &mut HashMap<String, String>,
    ) -> HashMap<String, String> {
        // Add timestamp
        let timestamp = self.get_timestamp();
        query_params.insert("timestamp".to_string(), timestamp.to_string());

        // Build query string for signature
        let query_string = self.build_query_string(query_params);

        // Generate signature
        let signature = encode_hex(&hmac_sha256(
            self.api_secret.as_bytes(),
            query_string.as_bytes(),
        ));

        // Add signature to params
        query_params.insert("signature".to_string(), signature);

        // Return headers
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY".to_string(), self.api_key.clone());

        headers
    }

    /// Build query string from params (sorted by key for consistency)
    fn build_query_string(&self, params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Sort for consistency (not required by Binance, but helpful for debugging)
        pairs.sort();
        pairs.join("&")
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
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BinanceAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());

        let headers = auth.sign_request(&mut params);

        assert!(params.contains_key("timestamp"));
        assert!(params.contains_key("signature"));
        assert_eq!(headers.get("X-MBX-APIKEY"), Some(&"test_key".to_string()));
    }

    #[test]
    fn test_build_query_string() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BinanceAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTCUSDT".to_string());
        params.insert("side".to_string(), "BUY".to_string());

        let query = auth.build_query_string(&params);

        // Should contain both parameters
        assert!(query.contains("symbol=BTCUSDT"));
        assert!(query.contains("side=BUY"));
    }
}
