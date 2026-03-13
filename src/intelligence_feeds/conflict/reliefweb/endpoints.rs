//! ReliefWeb API endpoints

/// Base URLs for ReliefWeb API
pub struct ReliefWebEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ReliefWebEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.reliefweb.int/v2",
            ws_base: None, // ReliefWeb does not support WebSocket
        }
    }
}

/// ReliefWeb API endpoint enum
#[derive(Debug, Clone)]
pub enum ReliefWebEndpoint {
    /// Humanitarian reports and situation updates
    Reports,
    /// Natural disasters and crises
    Disasters,
    /// Country profiles
    Countries,
    /// Humanitarian job listings
    Jobs,
    /// Training opportunities
    Training,
    /// Organizations and sources
    Sources,
}

impl ReliefWebEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Reports => "/reports",
            Self::Disasters => "/disasters",
            Self::Countries => "/countries",
            Self::Jobs => "/jobs",
            Self::Training => "/training",
            Self::Sources => "/sources",
        }
    }
}
