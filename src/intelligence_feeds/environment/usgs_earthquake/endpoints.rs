//! USGS Earthquake API endpoints

/// Base URLs for USGS Earthquake API
pub struct UsgsEarthquakeEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for UsgsEarthquakeEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://earthquake.usgs.gov/fdsnws/event/1",
            ws_base: None, // USGS does not support WebSocket
        }
    }
}

/// USGS Earthquake API endpoint enum
#[derive(Debug, Clone)]
pub enum UsgsEarthquakeEndpoint {
    /// Query earthquakes with filters (GeoJSON format)
    Query,
    /// Count earthquakes matching criteria
    Count,
}

impl UsgsEarthquakeEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Query => "/query",
            Self::Count => "/count",
        }
    }
}
