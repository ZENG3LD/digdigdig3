//! Sentinel Hub API endpoints

/// Base URLs for Sentinel Hub API
pub struct SentinelHubEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SentinelHubEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://services.sentinel-hub.com",
            ws_base: None, // Sentinel Hub does not support WebSocket
        }
    }
}

/// Sentinel Hub API endpoint enum
#[derive(Debug, Clone)]
pub enum SentinelHubEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // AUTHENTICATION
    // ═══════════════════════════════════════════════════════════════════════
    /// OAuth2 token endpoint
    Token,

    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// STAC catalog search
    CatalogSearch,
    /// Process satellite imagery
    Process,
    /// Statistical analysis of imagery
    Statistical,
}

impl SentinelHubEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Token => "/oauth/token".to_string(),
            Self::CatalogSearch => "/api/v1/catalog/search".to_string(),
            Self::Process => "/api/v1/process".to_string(),
            Self::Statistical => "/api/v1/statistical".to_string(),
        }
    }
}
