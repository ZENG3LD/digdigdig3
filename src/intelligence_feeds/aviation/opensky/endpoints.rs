//! OpenSky Network API endpoints

/// Base URLs for OpenSky Network API
pub struct OpenskyEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenskyEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://opensky-network.org/api",
            ws_base: None, // OpenSky does not support WebSocket
        }
    }
}

/// OpenSky Network API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenskyEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // STATE VECTOR ENDPOINTS (real-time aircraft positions)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get state vectors for all aircraft
    /// Anonymous: 10 req/10s, Authenticated: varies by response size
    StatesAll,

    /// Get state vectors from own sensors (authenticated only)
    /// Credits: varies by response size
    StatesOwn,

    // ═══════════════════════════════════════════════════════════════════════
    // FLIGHT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get flights in time range (all departures in interval)
    /// Credits: 4 per query
    FlightsAll,

    /// Get flights by specific aircraft (ICAO24 address)
    /// Credits: 1 per query
    FlightsAircraft,

    /// Get arrivals at specific airport in time range
    /// Credits: 2 per query
    FlightsArrival,

    /// Get departures from specific airport in time range
    /// Credits: 2 per query
    FlightsDeparture,

    // ═══════════════════════════════════════════════════════════════════════
    // TRACK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get flight track (waypoints) for specific aircraft
    /// Credits: 1 per query
    TracksAll,
}

impl OpenskyEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // State vectors
            Self::StatesAll => "/states/all",
            Self::StatesOwn => "/states/own",

            // Flights
            Self::FlightsAll => "/flights/all",
            Self::FlightsAircraft => "/flights/aircraft",
            Self::FlightsArrival => "/flights/arrival",
            Self::FlightsDeparture => "/flights/departure",

            // Tracks
            Self::TracksAll => "/tracks/all",
        }
    }
}

/// Format ICAO24 address (aircraft identifier)
///
/// ICAO24 addresses are 6-character hex strings (e.g., "abc123", "a1b2c3")
/// Should be lowercase according to OpenSky documentation
pub fn format_icao24(icao24: &str) -> String {
    icao24.trim().to_lowercase()
}

/// Format airport ICAO code
///
/// Airport codes are 4-character uppercase strings (e.g., "KJFK", "EDDF")
pub fn format_airport_icao(airport: &str) -> String {
    airport.trim().to_uppercase()
}

/// Format UNIX timestamp for API
///
/// OpenSky expects UNIX timestamps in seconds (not milliseconds)
pub fn format_timestamp(timestamp: i64) -> String {
    timestamp.to_string()
}

/// Parse UNIX timestamp from API response
pub fn _parse_timestamp(timestamp: i64) -> i64 {
    timestamp
}
