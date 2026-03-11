//! # Bithumb Authentication
//!
//! Реализация подписи запросов для Bithumb Pro API.
//!
//! ## Алгоритм подписи
//!
//! 1. Добавить apiKey и timestamp к параметрам
//! 2. Отсортировать параметры по ключу (алфавитный порядок)
//! 3. Собрать строку: `key1=value1&key2=value2&...`
//! 4. HMAC-SHA256 с secret key
//! 5. Конвертировать в lowercase hex
//!
//! ## Headers/Parameters
//!
//! - `apiKey` - API key (в параметрах)
//! - `timestamp` - Timestamp в миллисекундах (в параметрах)
//! - `signature` - Signature в lowercase (в параметрах)
//! - `msgNo` - Опциональный unique message number (в параметрах)

use std::collections::HashMap;

use crate::core::{
    hmac_sha256, timestamp_millis,
    Credentials, ExchangeResult,
};

/// Bithumb аутентификация
#[derive(Clone)]
pub struct BithumbAuth {
    api_key: String,
    api_secret: String,
}

impl BithumbAuth {
    /// Создать новый auth handler
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: credentials.api_key.clone(),
            api_secret: credentials.api_secret.clone(),
        })
    }

    /// Подписать параметры запроса и вернуть обновленные параметры
    ///
    /// Bithumb Pro использует parameter signing:
    /// 1. Добавляет apiKey и timestamp в параметры
    /// 2. Сортирует параметры по ключу
    /// 3. Формирует строку подписи: `key1=value1&key2=value2&...`
    /// 4. Подписывает HMAC-SHA256
    /// 5. Добавляет signature в параметры (lowercase)
    pub fn sign_request(
        &self,
        params: &mut HashMap<String, String>,
    ) -> HashMap<String, String> {
        // Add apiKey and timestamp
        params.insert("apiKey".to_string(), self.api_key.clone());
        params.insert("timestamp".to_string(), timestamp_millis().to_string());

        // Sort parameters alphabetically by key
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();

        // Build signature string: key1=value1&key2=value2&...
        let signature_string: String = keys.iter()
            .map(|k| format!("{}={}", k, params[*k]))
            .collect::<Vec<_>>()
            .join("&");

        // Generate HMAC-SHA256 signature
        let signature_bytes = hmac_sha256(
            self.api_secret.as_bytes(),
            signature_string.as_bytes(),
        );

        // Convert to lowercase hex string
        let signature = signature_bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        // Add signature to params
        params.insert("signature".to_string(), signature);

        params.clone()
    }

    /// Получить API key (для headers без подписи)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Получить API secret (для WebSocket аутентификации)
    pub fn api_secret(&self) -> &str {
        &self.api_secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BithumbAuth::new(&credentials).unwrap();

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), "BTC-USDT".to_string());
        params.insert("quantity".to_string(), "0.5".to_string());

        let signed_params = auth.sign_request(&mut params);

        // Should contain apiKey, timestamp, signature, and original params
        assert!(signed_params.contains_key("apiKey"));
        assert!(signed_params.contains_key("timestamp"));
        assert!(signed_params.contains_key("signature"));
        assert!(signed_params.contains_key("symbol"));
        assert!(signed_params.contains_key("quantity"));

        // Signature should be lowercase hex (64 characters for SHA256)
        let signature = signed_params.get("signature").unwrap();
        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn test_signature_deterministic() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = BithumbAuth::new(&credentials).unwrap();

        // With same timestamp, signature should be deterministic
        let timestamp = timestamp_millis().to_string();

        let mut params1 = HashMap::new();
        params1.insert("symbol".to_string(), "BTC-USDT".to_string());
        params1.insert("timestamp".to_string(), timestamp.clone());
        params1.insert("apiKey".to_string(), auth.api_key().to_string());

        let mut params2 = HashMap::new();
        params2.insert("apiKey".to_string(), auth.api_key().to_string());
        params2.insert("timestamp".to_string(), timestamp.clone());
        params2.insert("symbol".to_string(), "BTC-USDT".to_string());

        // Build signature strings manually
        let mut keys1: Vec<&String> = params1.keys().collect();
        keys1.sort();
        let sig_str1: String = keys1.iter()
            .map(|k| format!("{}={}", k, params1[*k]))
            .collect::<Vec<_>>()
            .join("&");

        let mut keys2: Vec<&String> = params2.keys().collect();
        keys2.sort();
        let sig_str2: String = keys2.iter()
            .map(|k| format!("{}={}", k, params2[*k]))
            .collect::<Vec<_>>()
            .join("&");

        // Signature strings should be identical when sorted
        assert_eq!(sig_str1, sig_str2);
    }
}
