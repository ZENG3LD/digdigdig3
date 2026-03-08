//! OFAC API endpoints

/// Base URLs for OFAC API
pub struct OfacEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OfacEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.ofac-api.com/v4",
            ws_base: None, // OFAC API does not support WebSocket
        }
    }
}

/// OFAC API endpoint enum
#[derive(Debug, Clone)]
pub enum OfacEndpoint {
    /// Search sanctioned entities
    Search,
    /// Screen a name/entity against SDN list
    Screen,
    /// List available sanction sources
    Sources,
    /// Get Specially Designated Nationals list
    Sdn,
}

impl OfacEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Search => "/search",
            Self::Screen => "/screen",
            Self::Sources => "/sources",
            Self::Sdn => "/sdn",
        }
    }
}
