//! URLhaus (abuse.ch) API endpoints

/// Base URLs for URLhaus API
pub struct UrlhausEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for UrlhausEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://urlhaus-api.abuse.ch/v1",
            ws_base: None, // URLhaus does not support WebSocket
        }
    }
}

/// URLhaus API endpoint enum
#[derive(Debug, Clone)]
pub enum UrlhausEndpoint {
    /// Get recent malicious URLs with optional limit
    RecentUrls { limit: u32 },
    /// Lookup URL information
    UrlLookup,
    /// Lookup host information
    HostLookup,
    /// Lookup payload (malware sample) information
    PayloadLookup,
    /// Lookup URLs by tag
    TagLookup,
}

impl UrlhausEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::RecentUrls { limit } => format!("/urls/recent/limit/{}/", limit),
            Self::UrlLookup => "/url/".to_string(),
            Self::HostLookup => "/host/".to_string(),
            Self::PayloadLookup => "/payload/".to_string(),
            Self::TagLookup => "/tag/".to_string(),
        }
    }
}
