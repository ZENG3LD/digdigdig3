//! # Uniswap Authentication
//!
//! Minimal authentication for Uniswap APIs.
//!
//! ## Trading API
//! Uses `x-api-key` header for authenticated endpoints (quote, swap).
//!
//! ## The Graph Subgraph
//! API key embedded in URL path.
//!
//! ## Ethereum RPC
//! Provider-specific authentication (Infura/Alchemy API keys in URL).
//!
//! ## WebSocket
//! Provider-specific authentication (same as RPC).

use std::collections::HashMap;
use crate::core::{Credentials, ExchangeError, ExchangeResult};

// ═══════════════════════════════════════════════════════════════════════════════
// UNISWAP AUTH
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap authentication (minimal)
#[derive(Debug, Clone)]
pub struct UniswapAuth {
    /// Trading API key (for quote/swap endpoints)
    api_key: Option<String>,
    /// The Graph API key (for subgraph queries)
    subgraph_key: Option<String>,
    /// Ethereum RPC provider URL (with embedded API key if needed)
    rpc_url: Option<String>,
    /// Ethereum WebSocket URL (with embedded API key if needed)
    ws_url: Option<String>,
    /// Wallet private key (for signing transactions)
    private_key: Option<String>,
}

impl UniswapAuth {
    /// Create new auth from credentials
    ///
    /// Expected credentials format:
    /// - `api_key`: Uniswap Trading API key
    /// - `api_secret`: The Graph API key
    /// - `passphrase`: Ethereum RPC URL
    /// - Additional: private key for transaction signing (optional)
    pub fn new(credentials: &Credentials) -> ExchangeResult<Self> {
        Ok(Self {
            api_key: Some(credentials.api_key.clone()),
            subgraph_key: Some(credentials.api_secret.clone()),
            rpc_url: credentials.passphrase.clone(),
            ws_url: None, // Can be set separately
            private_key: None, // For future transaction signing support
        })
    }

    /// Create auth with only API key (for quote/swap only)
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key: Some(api_key),
            subgraph_key: None,
            rpc_url: None,
            ws_url: None,
            private_key: None,
        }
    }

    /// Create auth without credentials (public endpoints only)
    pub fn public() -> Self {
        Self {
            api_key: None,
            subgraph_key: None,
            rpc_url: None,
            ws_url: None,
            private_key: None,
        }
    }

    /// Set The Graph API key
    pub fn set_subgraph_key(&mut self, key: String) {
        self.subgraph_key = Some(key);
    }

    /// Set Ethereum RPC URL (with API key if needed)
    pub fn set_rpc_url(&mut self, url: String) {
        self.rpc_url = Some(url);
    }

    /// Set Ethereum WebSocket URL (with API key if needed)
    pub fn set_ws_url(&mut self, url: String) {
        self.ws_url = Some(url);
    }

    /// Set wallet private key (for transaction signing)
    pub fn set_private_key(&mut self, key: String) {
        self.private_key = Some(key);
    }

    /// Get headers for Trading API request
    pub fn trading_api_headers(&self) -> ExchangeResult<HashMap<String, String>> {
        let mut headers = HashMap::new();

        if let Some(ref key) = self.api_key {
            headers.insert("x-api-key".to_string(), key.clone());
        } else {
            return Err(ExchangeError::Auth(
                "Trading API key required for authenticated requests".to_string(),
            ));
        }

        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Get The Graph subgraph URL with API key
    pub fn subgraph_url(&self, base_url: &str) -> ExchangeResult<String> {
        if let Some(ref key) = self.subgraph_key {
            // The Graph uses API key in URL path
            // Format: https://gateway.thegraph.com/api/{api_key}/subgraphs/id/{subgraph_id}
            if base_url.contains("/subgraphs/id/") {
                let url = base_url.replace(
                    "https://gateway.thegraph.com/api/subgraphs/id/",
                    &format!("https://gateway.thegraph.com/api/{}/subgraphs/id/", key),
                );
                Ok(url)
            } else {
                Ok(base_url.to_string())
            }
        } else {
            // Use public endpoint
            Ok(base_url.to_string())
        }
    }

    /// Get Ethereum RPC URL
    pub fn rpc_url(&self, default: &str) -> String {
        self.rpc_url
            .as_deref()
            .unwrap_or(default)
            .to_string()
    }

    /// Get Ethereum WebSocket URL
    pub fn ws_url(&self, default: &str) -> String {
        self.ws_url
            .as_deref()
            .unwrap_or(default)
            .to_string()
    }

    /// Check if we have Trading API authentication
    pub fn has_trading_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Check if we have subgraph authentication
    pub fn has_subgraph_key(&self) -> bool {
        self.subgraph_key.is_some()
    }

    /// Check if we have private key for transaction signing
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    /// Get headers for public requests
    pub fn public_headers() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_auth() {
        let auth = UniswapAuth::public();
        assert!(!auth.has_trading_api_key());
        assert!(!auth.has_subgraph_key());
        assert!(!auth.has_private_key());
    }

    #[test]
    fn test_api_key_auth() {
        let auth = UniswapAuth::with_api_key("test_key".to_string());
        assert!(auth.has_trading_api_key());
        assert!(!auth.has_subgraph_key());

        let headers = auth.trading_api_headers().unwrap();
        assert_eq!(headers.get("x-api-key"), Some(&"test_key".to_string()));
    }

    #[test]
    fn test_subgraph_url_with_key() {
        let mut auth = UniswapAuth::public();
        auth.set_subgraph_key("my_key".to_string());

        let base_url = "https://gateway.thegraph.com/api/subgraphs/id/abc123";
        let url = auth.subgraph_url(base_url).unwrap();
        assert!(url.contains("/api/my_key/subgraphs/id/"));
    }

    #[test]
    fn test_rpc_url() {
        let mut auth = UniswapAuth::public();
        auth.set_rpc_url("https://eth.llamarpc.com".to_string());

        let url = auth.rpc_url("https://default.com");
        assert_eq!(url, "https://eth.llamarpc.com");
    }
}
