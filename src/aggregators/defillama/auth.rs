//! # DefiLlama Authentication
//!
//! Реализация аутентификации для DefiLlama API.
//!
//! ## Authentication Pattern
//!
//! DefiLlama использует уникальную схему аутентификации:
//! - **NO signature/HMAC** (в отличие от CEX)
//! - **API key в URL path** (не в headers)
//! - **NO timestamp/nonce** требований
//!
//! ## URL Format
//!
//! - Free tier: `https://api.llama.fi/<endpoint>`
//! - Pro tier: `https://pro-api.llama.fi/<API_KEY>/<endpoint>`
//!
//! ## Rate Limits
//!
//! - Free tier: Lower rate limits
//! - Pro tier ($300/mo): Higher rate limits + 35 additional endpoints

use std::collections::HashMap;

use crate::core::{
    Credentials, ExchangeResult,
};

use super::endpoints::{DefiLlamaUrls, EndpointCategory};

/// DefiLlama аутентификация (URL-based)
#[derive(Clone)]
pub struct DefiLlamaAuth {
    /// API key (опционально, для Pro tier)
    api_key: Option<String>,
    /// URLs
    urls: DefiLlamaUrls,
}

impl DefiLlamaAuth {
    /// Создать новый auth handler
    ///
    /// # Arguments
    /// - `credentials`: Credentials с optional API key для Pro tier
    ///
    /// # Free Tier
    /// Если API key пустой или None, используется free tier
    pub fn new(credentials: Option<&Credentials>) -> ExchangeResult<Self> {
        let api_key = credentials
            .map(|c| c.api_key.clone())
            .filter(|key| !key.is_empty());

        Ok(Self {
            api_key,
            urls: DefiLlamaUrls::MAINNET,
        })
    }

    /// Build URL for the default category (Api)
    ///
    /// # Format
    /// - Free tier: `https://api.llama.fi/protocols`
    /// - Pro tier: `https://pro-api.llama.fi/<API_KEY>/protocols`
    pub fn build_url(&self, endpoint_path: &str) -> String {
        self.urls.build_url(self.api_key.as_deref(), EndpointCategory::Api, endpoint_path)
    }

    /// Build URL with explicit endpoint category for correct subdomain routing
    ///
    /// Different data types live on different subdomains:
    /// - Api: `api.llama.fi` (TVL, protocols)
    /// - Coins: `coins.llama.fi` (token prices)
    /// - Stablecoins: `stablecoins.llama.fi`
    /// - Yields: `yields.llama.fi`
    pub fn build_url_for(&self, category: EndpointCategory, endpoint_path: &str) -> String {
        self.urls.build_url(self.api_key.as_deref(), category, endpoint_path)
    }

    /// Get headers for request (minimal, no auth headers needed)
    ///
    /// DefiLlama doesn't use auth headers - все в URL
    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers
    }

    /// Check if using Pro tier
    pub fn is_pro_tier(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get API key (for validation/debugging)
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_url_building() {
        let auth = DefiLlamaAuth::new(None).unwrap();

        let url = auth.build_url("/protocols");
        assert_eq!(url, "https://api.llama.fi/protocols");
        assert!(!auth.is_pro_tier());
    }

    #[test]
    fn test_coins_subdomain() {
        let auth = DefiLlamaAuth::new(None).unwrap();

        let url = auth.build_url_for(EndpointCategory::Coins, "/prices/current/coingecko:bitcoin");
        assert_eq!(url, "https://coins.llama.fi/prices/current/coingecko:bitcoin");
    }

    #[test]
    fn test_stablecoins_subdomain() {
        let auth = DefiLlamaAuth::new(None).unwrap();

        let url = auth.build_url_for(EndpointCategory::Stablecoins, "/stablecoins");
        assert_eq!(url, "https://stablecoins.llama.fi/stablecoins");
    }

    #[test]
    fn test_yields_subdomain() {
        let auth = DefiLlamaAuth::new(None).unwrap();

        let url = auth.build_url_for(EndpointCategory::Yields, "/pools");
        assert_eq!(url, "https://yields.llama.fi/pools");
    }

    #[test]
    fn test_pro_tier_url_building() {
        let credentials = Credentials::new("test_api_key", "");
        let auth = DefiLlamaAuth::new(Some(&credentials)).unwrap();

        let url = auth.build_url("/protocols");
        assert_eq!(url, "https://pro-api.llama.fi/test_api_key/protocols");
        assert!(auth.is_pro_tier());
    }

    #[test]
    fn test_empty_api_key_uses_free_tier() {
        let credentials = Credentials::new("", "");
        let auth = DefiLlamaAuth::new(Some(&credentials)).unwrap();

        let url = auth.build_url("/protocols");
        assert_eq!(url, "https://api.llama.fi/protocols");
        assert!(!auth.is_pro_tier());
    }

    #[test]
    fn test_headers_no_auth() {
        let auth = DefiLlamaAuth::new(None).unwrap();
        let headers = auth.get_headers();

        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert!(!headers.contains_key("Authorization"));
        assert!(!headers.contains_key("X-API-KEY"));
    }
}
