//! AviationStack API endpoints

/// Base URLs for AviationStack API
pub struct AviationStackEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AviationStackEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "http://api.aviationstack.com/v1",
            ws_base: None, // AviationStack does not support WebSocket
        }
    }
}

/// AviationStack API endpoint enum
#[derive(Debug, Clone)]
pub enum AviationStackEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // FLIGHT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get real-time flight data
    /// Query params: flight_iata, airline_iata, dep_iata, arr_iata, flight_status
    Flights,

    // ═══════════════════════════════════════════════════════════════════════
    // DATABASE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get airport database
    /// Query params: search, country_iso2, iata_code
    Airports,

    /// Get airline database
    /// Query params: search, country_iso2, iata_code
    Airlines,

    /// Get aircraft types database
    AircraftTypes,

    /// Get cities database
    Cities,

    /// Get countries database
    Countries,

    /// Get flight routes
    /// Query params: dep_iata, arr_iata, airline_iata
    Routes,
}

impl AviationStackEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Flights => "/flights",
            Self::Airports => "/airports",
            Self::Airlines => "/airlines",
            Self::AircraftTypes => "/aircraft_types",
            Self::Cities => "/cities",
            Self::Countries => "/countries",
            Self::Routes => "/routes",
        }
    }
}

/// Format IATA code (3-letter airport/airline code)
///
/// IATA codes are 3-character uppercase strings (e.g., "JFK", "LAX", "AA")
pub fn format_iata(iata: &str) -> String {
    iata.trim().to_uppercase()
}

/// Format country ISO2 code
///
/// Country codes are 2-character uppercase strings (e.g., "US", "GB", "DE")
pub fn format_country_iso2(country: &str) -> String {
    country.trim().to_uppercase()
}
