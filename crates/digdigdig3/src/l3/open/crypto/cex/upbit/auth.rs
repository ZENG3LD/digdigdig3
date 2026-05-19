//! # Upbit Authentication
//!
//! Реализация JWT-based аутентификации для Upbit API.
//!
//! ## Алгоритм подписи (JWT)
//!
//! 1. Создать payload с:
//!    - `access_key` - API Access Key
//!    - `nonce` - UUID v4 (уникальный для каждого запроса)
//!    - `query_hash` - SHA-512 хеш параметров (если есть)
//!    - `query_hash_alg` - "SHA512" (если query_hash присутствует)
//! 2. Создать JWT header с `alg: "HS512"`, `typ: "JWT"`
//! 3. Base64URL encode header и payload
//! 4. HMAC-SHA512 с secret key от `{header}.{payload}`
//! 5. Base64URL encode signature
//! 6. Собрать JWT: `{header}.{payload}.{signature}`
//!
//! ## Headers
//!
//! - `Authorization` - `Bearer {JWT_TOKEN}`
//! - `Content-Type` - `application/json; charset=utf-8`

use std::collections::HashMap;
use sha2::{Sha512, Digest};
use hmac::{Hmac, Mac};
use uuid::Uuid;

use crate::core::{
    Credentials, ExchangeResult, ExchangeError,
};

type HmacSha512 = Hmac<Sha512>;

/// Upbit JWT аутентификация
#[derive(Clone)]
pub struct UpbitAuth {
    access_key: String,
    secret_key: String,
}

impl UpbitAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            access_key: credentials.api_key.clone(),
            secret_key: credentials.api_secret.clone(),
        })
    }

    /// Создать JWT токен для запроса
    ///
    /// # Arguments
    /// - `query_string` - URL-encoded query string для GET/DELETE или
    ///   сериализованные параметры для POST
    pub fn create_jwt_token(&self, query_string: Option<&str>) -> ExchangeResult<String> {
        // Generate unique nonce (UUID v4)
        let nonce = Uuid::new_v4().to_string();

        // Build payload
        let mut payload = serde_json::json!({
            "access_key": self.access_key,
            "nonce": nonce,
        });

        // Add query_hash if parameters exist
        if let Some(qs) = query_string {
            if !qs.is_empty() {
                let query_hash = self.sha512_hex(qs.as_bytes());
                payload["query_hash"] = serde_json::json!(query_hash);
                payload["query_hash_alg"] = serde_json::json!("SHA512");
            }
        }

        // JWT header
        let header = serde_json::json!({
            "alg": "HS512",
            "typ": "JWT"
        });

        // Serialize and Base64URL encode
        let header_json = serde_json::to_string(&header)
            .map_err(|e| ExchangeError::Auth(format!("Failed to serialize header: {}", e)))?;
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| ExchangeError::Auth(format!("Failed to serialize payload: {}", e)))?;

        let header_b64 = self.base64url_encode(header_json.as_bytes());
        let payload_b64 = self.base64url_encode(payload_json.as_bytes());

        // Create signature
        let message = format!("{}.{}", header_b64, payload_b64);
        let signature = self.hmac_sha512(self.secret_key.as_bytes(), message.as_bytes())?;
        let signature_b64 = self.base64url_encode(&signature);

        // Combine into JWT
        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }

    /// Подписать запрос и вернуть headers
    pub fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: Option<&str>,
    ) -> ExchangeResult<HashMap<String, String>> {
        let token = self.create_jwt_token(query_string)?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert("Content-Type".to_string(), "application/json; charset=utf-8".to_string());

        Ok(headers)
    }

    /// SHA-512 hash to hexadecimal string
    fn sha512_hex(&self, data: &[u8]) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// HMAC-SHA512
    fn hmac_sha512(&self, key: &[u8], message: &[u8]) -> ExchangeResult<Vec<u8>> {
        let mut mac = HmacSha512::new_from_slice(key)
            .map_err(|e| ExchangeError::Auth(format!("HMAC init failed: {}", e)))?;
        mac.update(message);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Base64URL encode (without padding)
    fn base64url_encode(&self, data: &[u8]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    /// Получить API key
    pub fn access_key(&self) -> &str {
        &self.access_key
    }
}

/// Конвертировать JSON body в query string для подписи
///
/// Для POST запросов нужно конвертировать JSON body в формат key=value&key=value
/// и отсортировать параметры алфавитно для консистентности.
pub fn json_to_query_string(json_body: &str) -> ExchangeResult<String> {
    if json_body.is_empty() {
        return Ok(String::new());
    }

    let value: serde_json::Value = serde_json::from_str(json_body)
        .map_err(|e| ExchangeError::Auth(format!("Failed to parse JSON: {}", e)))?;

    let obj = value.as_object()
        .ok_or_else(|| ExchangeError::Auth("JSON body is not an object".to_string()))?;

    // Collect key-value pairs and sort alphabetically
    let mut pairs: Vec<(String, String)> = obj.iter()
        .map(|(k, v): (&String, &serde_json::Value)| {
            let value_str = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => v.to_string(),
            };
            (k.clone(), value_str)
        })
        .collect();

    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    // URL encode and join
    let query_string = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();

    Ok(query_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_jwt_token() {
        let credentials = Credentials::new("test_access_key", "test_secret_key");
        let auth = UpbitAuth::new(&credentials).unwrap();

        // Test without query string
        let token = auth.create_jwt_token(None).unwrap();
        assert!(token.contains('.'));
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3); // header.payload.signature

        // Test with query string
        let token = auth.create_jwt_token(Some("market=SGD-BTC&state=wait")).unwrap();
        assert!(token.contains('.'));
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_access_key", "test_secret_key");
        let auth = UpbitAuth::new(&credentials).unwrap();

        let headers = auth.sign_request("GET", "/v1/balances", None).unwrap();

        assert!(headers.contains_key("Authorization"));
        assert!(headers.get("Authorization").unwrap().starts_with("Bearer "));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json; charset=utf-8".to_string()));
    }

    #[test]
    fn test_json_to_query_string() {
        let json = r#"{"market":"SGD-BTC","side":"bid","volume":"0.1"}"#;
        let qs = json_to_query_string(json).unwrap();

        // Should be alphabetically sorted
        assert!(qs.contains("market=SGD-BTC"));
        assert!(qs.contains("side=bid"));
        assert!(qs.contains("volume=0.1"));

        // Check order (alphabetical)
        let market_pos = qs.find("market").unwrap();
        let side_pos = qs.find("side").unwrap();
        let volume_pos = qs.find("volume").unwrap();
        assert!(market_pos < side_pos);
        assert!(side_pos < volume_pos);
    }

    #[test]
    fn test_sha512_hex() {
        let credentials = Credentials::new("test", "test");
        let auth = UpbitAuth::new(&credentials).unwrap();

        let hash = auth.sha512_hex(b"test");
        assert_eq!(hash.len(), 128); // SHA-512 produces 128 hex characters
    }
}
