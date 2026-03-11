//! # GMX Authentication
//!
//! GMX REST API endpoints are **public** and do not require authentication.
//! Trading operations require blockchain wallet signatures, not API keys.
//!
//! This module provides a no-op auth handler for consistency with V5 architecture.

use std::collections::HashMap;
use crate::core::{Credentials, ExchangeResult};

/// GMX authentication handler (no-op for public REST endpoints)
#[derive(Clone)]
pub struct GmxAuth {
    // Store credentials in case future trading features need wallet private key
    #[allow(dead_code)]
    credentials: Option<Credentials>,
}

impl GmxAuth {
    /// Create new auth handler
    ///
    /// Note: GMX REST endpoints are public and don't require API keys.
    /// This accepts credentials for future wallet-based trading features.
    pub fn new(credentials: Option<&Credentials>) -> ExchangeResult<Self> {
        Ok(Self {
            credentials: credentials.cloned(),
        })
    }

    /// Create public-only auth handler (no credentials)
    pub fn public() -> Self {
        Self {
            credentials: None,
        }
    }

    /// Sign request and return headers
    ///
    /// GMX REST endpoints are public, so this returns empty headers.
    /// Future wallet-based trading would require EIP-712 signatures.
    pub fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        _body: &str,
    ) -> HashMap<String, String> {
        // All GMX REST endpoints are public
        // Return minimal headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// Check if credentials are available
    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_auth() {
        let auth = GmxAuth::public();
        assert!(!auth.has_credentials());

        let headers = auth.sign_request("GET", "/prices/tickers", "");
        assert!(headers.contains_key("Content-Type"));
        assert_eq!(headers.len(), 1); // Only Content-Type header
    }

    #[test]
    fn test_auth_with_credentials() {
        let credentials = Credentials::new("test_key", "test_secret");
        let auth = GmxAuth::new(Some(&credentials)).unwrap();

        assert!(auth.has_credentials());

        // Still returns public headers for REST endpoints
        let headers = auth.sign_request("GET", "/prices/tickers", "");
        assert!(headers.contains_key("Content-Type"));
    }
}
