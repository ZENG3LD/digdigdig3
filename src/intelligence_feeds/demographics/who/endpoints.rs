//! WHO GHO API endpoints

/// Base URLs for WHO GHO API
pub struct WhoEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for WhoEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ghoapi.azureedge.net/api",
            ws_base: None, // WHO GHO does not support WebSocket
        }
    }
}

/// WHO GHO API endpoint enum
#[derive(Debug, Clone)]
pub enum WhoEndpoint {
    /// Get all indicators
    Indicators,
    /// Get data for a specific indicator
    IndicatorData(String),
    /// Get list of countries
    Countries,
    /// Get list of regions
    Regions,
}

impl WhoEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Indicators => "/Indicator".to_string(),
            Self::IndicatorData(code) => format!("/{}", code),
            Self::Countries => "/DIMENSION/COUNTRY".to_string(),
            Self::Regions => "/DIMENSION/REGION".to_string(),
        }
    }
}
