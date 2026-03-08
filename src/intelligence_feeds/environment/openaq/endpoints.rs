//! OpenAQ API endpoints

/// Base URLs for OpenAQ API
pub struct OpenAqEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenAqEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.openaq.org/v2",
            ws_base: None, // OpenAQ does not support WebSocket
        }
    }
}

/// OpenAQ API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenAqEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // LOCATION ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get monitoring locations
    Locations,
    /// Get specific location by ID
    LocationById,

    // ═══════════════════════════════════════════════════════════════════════
    // MEASUREMENT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get air quality measurements
    Measurements,
    /// Get latest measurements from all locations
    Latest,
    /// Get averaged data
    Averages,

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List countries with data
    Countries,
    /// List cities with data
    Cities,
    /// List measured parameters (PM2.5, PM10, O3, NO2, SO2, CO)
    Parameters,
}

impl OpenAqEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Locations
            Self::Locations => "/locations",
            Self::LocationById => "/locations", // ID appended in connector

            // Measurements
            Self::Measurements => "/measurements",
            Self::Latest => "/latest",
            Self::Averages => "/averages",

            // Metadata
            Self::Countries => "/countries",
            Self::Cities => "/cities",
            Self::Parameters => "/parameters",
        }
    }
}
