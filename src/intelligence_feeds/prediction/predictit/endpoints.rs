//! PredictIt API endpoints

/// Base URLs for PredictIt API
pub struct PredictItEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for PredictItEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.predictit.org/api/marketdata",
            ws_base: None, // PredictIt does not support WebSocket
        }
    }
}

/// PredictIt API endpoint enum
#[derive(Debug, Clone)]
pub enum PredictItEndpoint {
    /// Get all markets with contracts
    AllMarkets,
    /// Get specific market by ID
    Market,
}

impl PredictItEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::AllMarkets => "/all",
            Self::Market => "/markets",
        }
    }
}
