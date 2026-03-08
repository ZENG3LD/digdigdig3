//! ACLED API endpoints

/// Base URLs for ACLED API
pub struct AcledEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AcledEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.acleddata.com/acled/read",
            ws_base: None, // ACLED does not support WebSocket
        }
    }
}

/// ACLED API endpoint enum
///
/// ACLED uses a single endpoint with query parameters for all operations
#[derive(Debug, Clone)]
pub enum AcledEndpoint {
    /// Get events (core endpoint - all filtering done via query params)
    Events,
}

impl AcledEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Events => "",
        }
    }
}
