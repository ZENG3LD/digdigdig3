//! Wingbits API endpoints

/// Base URLs for Wingbits API
pub struct WingbitsEndpoints {
    pub rest_base: String,
    pub ws_base: Option<&'static str>,
}

impl Default for WingbitsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.wingbits.com".to_string(),
            ws_base: None, // Wingbits does not support WebSocket
        }
    }
}

impl WingbitsEndpoints {
    /// Create new endpoints with custom base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            rest_base: base_url.into(),
            ws_base: None,
        }
    }
}

/// Wingbits API endpoint enum
#[derive(Debug, Clone)]
pub enum WingbitsEndpoint {
    /// Get aircraft details by ICAO 24-bit address
    Details { icao24: String },
    /// Get batch aircraft details (POST endpoint)
    BatchDetails,
}

impl WingbitsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Details { icao24 } => format!("/api/wingbits/details/{}", icao24),
            Self::BatchDetails => "/api/wingbits/details/batch".to_string(),
        }
    }
}
