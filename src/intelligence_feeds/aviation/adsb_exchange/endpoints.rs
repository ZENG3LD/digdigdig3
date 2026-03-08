//! ADS-B Exchange API endpoints

/// Base URLs for ADS-B Exchange API
pub struct AdsbExchangeEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AdsbExchangeEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://adsbexchange-com1.p.rapidapi.com",
            ws_base: None, // ADS-B Exchange does not support WebSocket via RapidAPI
        }
    }
}

/// ADS-B Exchange API endpoint enum
#[derive(Debug, Clone)]
pub enum AdsbExchangeEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // LOCATION-BASED ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get aircraft near a location (lat/lon/distance in nautical miles)
    AircraftNearLocation { lat: f64, lon: f64, dist_nm: u32 },

    // ═══════════════════════════════════════════════════════════════════════
    // AIRCRAFT LOOKUP ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get aircraft by ICAO hex code (e.g., "a1b2c3")
    AircraftByHex { icao_hex: String },
    /// Get aircraft by callsign (e.g., "UAL123")
    AircraftByCallsign { callsign: String },
    /// Get aircraft by registration (e.g., "N12345")
    AircraftByRegistration { registration: String },
    /// Get aircraft by type (e.g., "B738", "F16")
    AircraftByType { aircraft_type: String },

    // ═══════════════════════════════════════════════════════════════════════
    // SPECIAL CATEGORY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get ALL military aircraft currently airborne (UNFILTERED)
    MilitaryAircraft,
    /// Get aircraft by squawk code (e.g., "7700" for emergency)
    AircraftBySquawk { squawk: String },
    /// Get LADD (Limited Aircraft Data Display) aircraft - military/sensitive
    LaddAircraft,
}

impl AdsbExchangeEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Location-based
            Self::AircraftNearLocation { lat, lon, dist_nm } => {
                format!("/v2/lat/{}/lon/{}/dist/{}/", lat, lon, dist_nm)
            }

            // Aircraft lookup
            Self::AircraftByHex { icao_hex } => {
                format!("/v2/hex/{}/", icao_hex)
            }
            Self::AircraftByCallsign { callsign } => {
                format!("/v2/callsign/{}/", callsign)
            }
            Self::AircraftByRegistration { registration } => {
                format!("/v2/registration/{}/", registration)
            }
            Self::AircraftByType { aircraft_type } => {
                format!("/v2/type/{}/", aircraft_type)
            }

            // Special categories
            Self::MilitaryAircraft => "/v2/mil/".to_string(),
            Self::AircraftBySquawk { squawk } => {
                format!("/v2/sqk/{}/", squawk)
            }
            Self::LaddAircraft => "/v2/ladd/".to_string(),
        }
    }
}
