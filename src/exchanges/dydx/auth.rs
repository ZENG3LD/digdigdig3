//! # dYdX v4 Authentication
//!
//! dYdX v4 uses blockchain wallet-based authentication, NOT API keys with HMAC.
//!
//! ## Authentication Architecture
//!
//! - **Indexer API** (read-only): No authentication required
//! - **Node API** (write operations): Requires signed blockchain transactions
//!
//! ## For This Implementation
//!
//! Since we're implementing read-only market data access via the Indexer API:
//! - No authentication headers needed
//! - No signatures required
//! - All endpoints are public
//!
//! ## Future: Write Operations
//!
//! For order placement/cancellation (Node API gRPC):
//! - Requires Cosmos wallet with mnemonic phrase
//! - Transaction signing with private key
//! - Gas fees paid in DYDX tokens
//! - NOT HMAC-SHA256 like traditional CEX

use std::collections::HashMap;

use crate::core::{Credentials, ExchangeResult};

/// dYdX v4 аутентификация (placeholder для будущей gRPC поддержки)
///
/// Текущая реализация: только Indexer API (read-only, без аутентификации)
/// Будущее: Node API (gRPC) с Cosmos wallet signing
#[derive(Clone)]
pub struct DydxAuth {
    /// Optional credentials for future gRPC implementation
    _credentials: Option<Credentials>,
}

impl DydxAuth {
    /// Создать новый auth handler
    ///
    /// Note: Indexer API не требует аутентификации
    /// Credentials сохраняются для будущей поддержки Node API (gRPC)
    pub fn new(credentials: Option<&Credentials>) -> ExchangeResult<Self> {
        Ok(Self {
            _credentials: credentials.cloned(),
        })
    }

    /// Создать публичный auth handler (без credentials)
    pub fn public() -> Self {
        Self {
            _credentials: None,
        }
    }

    /// Получить headers для Indexer API запроса
    ///
    /// Indexer API не требует аутентификации, возвращаем пустой HashMap
    pub fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: &str,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// Проверить, установлены ли credentials (для будущего использования)
    pub fn has_credentials(&self) -> bool {
        self._credentials.is_some()
    }
}

impl Default for DydxAuth {
    fn default() -> Self {
        Self::public()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_auth() {
        let auth = DydxAuth::public();
        assert!(!auth.has_credentials());

        let headers = auth.sign_request("GET", "/v4/perpetualMarkets", "");
        assert!(headers.contains_key("Content-Type"));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_auth_with_credentials() {
        let credentials = Credentials::new("dummy_key", "dummy_secret");
        let auth = DydxAuth::new(Some(&credentials)).unwrap();
        assert!(auth.has_credentials());

        // Indexer API still doesn't use credentials
        let headers = auth.sign_request("GET", "/v4/perpetualMarkets", "");
        assert_eq!(headers.len(), 1); // Only Content-Type
    }

    #[test]
    fn test_auth_new_none() {
        let auth = DydxAuth::new(None).unwrap();
        assert!(!auth.has_credentials());
    }
}
