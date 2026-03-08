//! SAM.gov API endpoints

/// Base URLs for SAM.gov API
pub struct SamGovEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SamGovEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.sam.gov",
            ws_base: None, // SAM.gov does not support WebSocket
        }
    }
}

/// SAM.gov API endpoint enum
#[derive(Debug, Clone)]
pub enum SamGovEndpoint {
    /// Search entities by various criteria
    Entities,
    /// Search contract opportunities
    Opportunities,
}

impl SamGovEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Entities => "/entity-information/v3/entities",
            Self::Opportunities => "/opportunities/v2/search",
        }
    }
}
