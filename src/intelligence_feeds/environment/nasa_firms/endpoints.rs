//! NASA FIRMS API endpoints

/// Base URLs for NASA FIRMS API
pub struct NasaFirmsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NasaFirmsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://firms.modaps.eosdis.nasa.gov/api",
            ws_base: None, // NASA FIRMS does not support WebSocket
        }
    }
}

/// NASA FIRMS API endpoint enum
#[derive(Debug, Clone)]
pub enum NasaFirmsEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // FIRE DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get fire data by geographic area
    /// Path format: /area?source={source}&area={bbox}&day_range={days}&date={date}&format=json
    Area,

    /// Get fire data by country
    /// Path format: /country?source={source}&country={code}&day_range={days}&date={date}&format=json
    Country,
}

impl NasaFirmsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Area => "/area",
            Self::Country => "/country",
        }
    }
}
