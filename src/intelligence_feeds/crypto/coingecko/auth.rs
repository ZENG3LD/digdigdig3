//! CoinGecko authentication
//!
//! Authentication type: Optional API Key (header)
//!
//! CoinGecko supports optional API key authentication via headers:
//! - Free tier: No key required (10-30 calls/min)
//! - Demo key: x-cg-demo-key header (30 calls/min)
//! - Pro key: x-cg-pro-key header (higher limits)

use std::collections::HashMap;

/// CoinGecko authentication credentials
#[derive(Clone)]
pub struct CoinGeckoAuth {
    pub api_key: Option<String>,
    pub is_pro: bool,
}

impl CoinGeckoAuth {
    /// Create new auth from environment variable
    ///
    /// Expects environment variable: `COINGECKO_API_KEY`
    /// Optional: `COINGECKO_PRO=true` for pro API key
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("COINGECKO_API_KEY").ok(),
            is_pro: std::env::var("COINGECKO_PRO")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }

    /// Create auth with explicit API key (demo tier)
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            is_pro: false,
        }
    }

    /// Create auth with explicit API key (pro tier)
    pub fn new_pro(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            is_pro: true,
        }
    }

    /// Create auth without API key (free tier)
    pub fn free() -> Self {
        Self {
            api_key: None,
            is_pro: false,
        }
    }

    /// Add authentication to headers
    ///
    /// CoinGecko uses different header names based on tier:
    /// - Demo: x-cg-demo-key
    /// - Pro: x-cg-pro-key
    pub fn add_auth_headers(&self, headers: &mut HashMap<String, String>) {
        if let Some(key) = &self.api_key {
            let header_name = if self.is_pro {
                "x-cg-pro-key"
            } else {
                "x-cg-demo-key"
            };
            headers.insert(header_name.to_string(), key.clone());
        }
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get tier name for debugging
    pub fn tier_name(&self) -> &'static str {
        match (self.api_key.is_some(), self.is_pro) {
            (false, _) => "free",
            (true, false) => "demo",
            (true, true) => "pro",
        }
    }
}

impl Default for CoinGeckoAuth {
    fn default() -> Self {
        Self::from_env()
    }
}
