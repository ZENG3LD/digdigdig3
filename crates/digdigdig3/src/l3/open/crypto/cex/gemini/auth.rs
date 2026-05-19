//! # Gemini Authentication
//!
//! Реализация подписи запросов для Gemini API.
//!
//! ## Алгоритм подписи
//!
//! 1. Создать JSON payload с request и nonce
//! 2. Base64 encode payload
//! 3. HMAC-SHA384 с secret key
//! 4. Encode в hex (lowercase)
//!
//! ## Headers
//!
//! - `X-GEMINI-APIKEY` - API key
//! - `X-GEMINI-PAYLOAD` - Base64-encoded JSON payload
//! - `X-GEMINI-SIGNATURE` - HMAC-SHA384 hex signature
//! - `Content-Type` - "text/plain"
//! - `Content-Length` - "0"
//! - `Cache-Control` - "no-cache"

use std::collections::HashMap;
use serde_json::{json, Value};

use crate::core::{
    hmac_sha384, encode_base64, timestamp_millis,
    Credentials, ExchangeResult,
};

/// Gemini аутентификация
#[derive(Clone)]
pub struct GeminiAuth {
    api_key: String,
    api_secret: String,
}

impl GeminiAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
        })
    }

    /// Generate nonce (millisecond timestamp)
    pub fn generate_nonce() -> u64 {
        timestamp_millis()
    }

    /// Подписать запрос и вернуть headers
    ///
    /// # Arguments
    /// * `endpoint` - API endpoint path (e.g., "/v1/balances")
    /// * `params` - Additional parameters to include in payload
    pub fn sign_request(
        &self,
        endpoint: &str,
        params: HashMap<String, Value>,
    ) -> ExchangeResult<HashMap<String, String>> {
        let nonce = Self::generate_nonce();

        // Build JSON payload
        let mut payload = json!({
            "request": endpoint,
            "nonce": nonce,
        });

        // Merge additional parameters
        if let Some(obj) = payload.as_object_mut() {
            for (key, value) in params {
                obj.insert(key, value);
            }
        }

        let payload_str = payload.to_string();

        // Base64 encode payload
        let b64_payload = encode_base64(payload_str.as_bytes());

        // Generate HMAC-SHA384 signature
        let signature_bytes = hmac_sha384(
            self.api_secret.as_bytes(),
            b64_payload.as_bytes(),
        );
        let signature = hex::encode(signature_bytes);

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("X-GEMINI-APIKEY".to_string(), self.api_key.clone());
        headers.insert("X-GEMINI-PAYLOAD".to_string(), b64_payload);
        headers.insert("X-GEMINI-SIGNATURE".to_string(), signature);
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Content-Length".to_string(), "0".to_string());
        headers.insert("Cache-Control".to_string(), "no-cache".to_string());

        Ok(headers)
    }

    /// Sign WebSocket connection request
    pub fn sign_websocket_request(
        &self,
        endpoint: &str,
    ) -> ExchangeResult<HashMap<String, String>> {
        // WebSocket authentication uses same mechanism but with empty params
        self.sign_request(endpoint, HashMap::new())
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
        let auth = GeminiAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("/v1/balances", HashMap::new()).unwrap();

        assert!(headers.contains_key("X-GEMINI-APIKEY"));
        assert!(headers.contains_key("X-GEMINI-PAYLOAD"));
        assert!(headers.contains_key("X-GEMINI-SIGNATURE"));
        assert_eq!(headers.get("X-GEMINI-APIKEY"), Some(&"test_key".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(headers.get("Content-Length"), Some(&"0".to_string()));
        assert_eq!(headers.get("Cache-Control"), Some(&"no-cache".to_string()));
    }

    #[test]
    fn test_signature_is_hex() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GeminiAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("/v1/balances", HashMap::new()).unwrap();
        let signature = headers.get("X-GEMINI-SIGNATURE").unwrap();

        // SHA384 produces 48 bytes = 96 hex characters
        assert_eq!(signature.len(), 96);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_with_params() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GeminiAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), json!("btcusd"));
        params.insert("amount".to_string(), json!("0.5"));

        let headers = auth.sign_request("/v1/order/new", params).unwrap();

        assert!(headers.contains_key("X-GEMINI-PAYLOAD"));

        // Decode and verify payload contains params
        let b64_payload = headers.get("X-GEMINI-PAYLOAD").unwrap();
        let payload_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            b64_payload
        ).unwrap();
        let payload_str = String::from_utf8(payload_bytes).unwrap();

        assert!(payload_str.contains("btcusd"));
        assert!(payload_str.contains("0.5"));
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = GeminiAuth::generate_nonce();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let nonce2 = GeminiAuth::generate_nonce();

        // Nonce should be strictly increasing
        assert!(nonce2 > nonce1);

        // Should be millisecond timestamp (13 digits)
        assert!(nonce1 > 1_600_000_000_000);
    }
}
