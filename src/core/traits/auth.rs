//! # Exchange Authentication Trait
//!
//! Трейт для аутентификации запросов к биржам.
//!
//! Каждая биржа реализует свою логику:
//! - Формат timestamp (millis, seconds, ISO 8601)
//! - Формат signature payload
//! - Encoding (Base64, Hex)
//! - Headers vs query params
//!
//! ## Пример реализации
//!
//! ```ignore
//! impl ExchangeAuth for KuCoinAuth {
//!     fn sign_request(&self, req: &mut AuthRequest) -> Result<()> {
//!         let timestamp = timestamp_millis();
//!         let payload = format!("{}{}{}{}", timestamp, req.method, req.path, req.body);
//!         let signature = encode_base64(&hmac_sha256(self.secret.as_bytes(), payload.as_bytes()));
//!
//!         req.headers.insert("KC-API-KEY", self.api_key.clone());
//!         req.headers.insert("KC-API-SIGN", signature);
//!         req.headers.insert("KC-API-TIMESTAMP", timestamp.to_string());
//!         // ...
//!         Ok(())
//!     }
//! }
//! ```

use std::collections::HashMap;
use crate::core::types::ExchangeResult;

/// Credentials для аутентификации
#[derive(Clone)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
}

impl Credentials {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            passphrase: None,
        }
    }

    pub fn with_passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }
}

/// Запрос для подписи
pub struct AuthRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub body: Option<&'a str>,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl<'a> AuthRequest<'a> {
    pub fn new(method: &'a str, path: &'a str) -> Self {
        Self {
            method,
            path,
            query: None,
            body: None,
            headers: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    pub fn with_query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }
}

/// Куда идёт signature
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureLocation {
    /// В headers (KuCoin, OKX, Bybit, Gate.io)
    Headers,
    /// В query params (Binance)
    QueryParams,
}

/// Трейт аутентификации для биржи
///
/// Каждая биржа реализует свою логику подписи.
pub trait ExchangeAuth: Send + Sync {
    /// Подписать запрос
    ///
    /// Модифицирует `req.headers` и/или `req.query_params`
    fn sign_request(
        &self,
        credentials: &Credentials,
        req: &mut AuthRequest<'_>,
    ) -> ExchangeResult<()>;

    /// Куда идёт signature
    fn signature_location(&self) -> SignatureLocation {
        SignatureLocation::Headers
    }
}
