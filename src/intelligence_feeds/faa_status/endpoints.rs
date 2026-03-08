//! FAA Airport Status API endpoints

/// Base URLs for FAA NASSTATUS API
pub struct FaaStatusEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for FaaStatusEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://nasstatus.faa.gov",
            ws_base: None, // FAA does not support WebSocket
        }
    }
}

/// FAA Airport Status API endpoint enum
#[derive(Debug, Clone)]
pub enum FaaStatusEndpoint {
    /// Airport status information endpoint
    AirportStatusInfo,
}

impl FaaStatusEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::AirportStatusInfo => "/api/airport-status-information",
        }
    }
}
