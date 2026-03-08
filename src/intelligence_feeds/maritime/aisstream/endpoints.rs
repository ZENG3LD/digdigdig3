//! AISStream.io API endpoints

/// Base URLs for AISStream.io API
pub struct AisStreamEndpoints {
    pub ws_base: &'static str,
    pub rest_base: Option<&'static str>,
}

impl Default for AisStreamEndpoints {
    fn default() -> Self {
        Self {
            ws_base: "wss://stream.aisstream.io/v0/stream",
            rest_base: None, // AISStream is WebSocket-only
        }
    }
}

/// AISStream.io API endpoint enum
#[derive(Debug, Clone)]
pub enum AisStreamEndpoint {
    /// WebSocket stream endpoint (primary interface)
    Stream,
}

impl AisStreamEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Stream => "/v0/stream",
        }
    }
}

/// Ship type constants for filtering
pub mod ship_types {
    /// Passenger ships (60-69)
    pub const PASSENGER_MIN: u32 = 60;
    pub const PASSENGER_MAX: u32 = 69;

    /// Cargo ships (70-79)
    pub const CARGO_MIN: u32 = 70;
    pub const CARGO_MAX: u32 = 79;

    /// Tanker ships (80-89)
    pub const TANKER_MIN: u32 = 80;
    pub const TANKER_MAX: u32 = 89;
}

/// Well-known maritime chokepoints and areas
pub mod areas {
    use super::super::parser::BoundingBox;

    /// Suez Canal area
    pub fn suez_canal() -> BoundingBox {
        BoundingBox {
            lat_min: 29.5,
            lon_min: 32.0,
            lat_max: 31.5,
            lon_max: 33.0,
        }
    }

    /// Strait of Hormuz
    pub fn strait_of_hormuz() -> BoundingBox {
        BoundingBox {
            lat_min: 25.5,
            lon_min: 55.5,
            lat_max: 27.5,
            lon_max: 57.5,
        }
    }

    /// Panama Canal
    pub fn panama_canal() -> BoundingBox {
        BoundingBox {
            lat_min: 8.5,
            lon_min: -80.5,
            lat_max: 9.5,
            lon_max: -79.0,
        }
    }

    /// Singapore Strait
    pub fn singapore_strait() -> BoundingBox {
        BoundingBox {
            lat_min: 1.0,
            lon_min: 103.5,
            lat_max: 1.5,
            lon_max: 104.5,
        }
    }

    /// Strait of Malacca
    pub fn strait_of_malacca() -> BoundingBox {
        BoundingBox {
            lat_min: 1.0,
            lon_min: 98.0,
            lat_max: 6.0,
            lon_max: 101.0,
        }
    }
}
