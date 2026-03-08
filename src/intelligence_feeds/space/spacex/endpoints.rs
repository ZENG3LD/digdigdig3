//! SpaceX API endpoints

/// Base URLs for SpaceX API
pub struct SpaceXEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SpaceXEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.spacexdata.com/v4",
            ws_base: None, // SpaceX does not support WebSocket
        }
    }
}

/// SpaceX API endpoint enum
#[derive(Debug, Clone)]
pub enum SpaceXEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCH ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all launches
    LaunchesAll,

    /// Get latest launch
    LaunchesLatest,

    /// Get next upcoming launch
    LaunchesNext,

    /// Get all upcoming launches
    LaunchesUpcoming,

    /// Get all past launches
    LaunchesPast,

    // ═══════════════════════════════════════════════════════════════════════
    // ROCKET ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all rockets
    Rockets,

    // ═══════════════════════════════════════════════════════════════════════
    // CREW ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all crew members
    Crew,

    // ═══════════════════════════════════════════════════════════════════════
    // STARLINK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all Starlink satellites
    Starlink,

    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCHPAD ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all launch pads
    Launchpads,

    // ═══════════════════════════════════════════════════════════════════════
    // LANDPAD ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all landing pads
    Landpads,

    // ═══════════════════════════════════════════════════════════════════════
    // PAYLOAD ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all payloads
    Payloads,

    // ═══════════════════════════════════════════════════════════════════════
    // CAPSULE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all capsules
    Capsules,
}

impl SpaceXEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Launches
            Self::LaunchesAll => "/launches",
            Self::LaunchesLatest => "/launches/latest",
            Self::LaunchesNext => "/launches/next",
            Self::LaunchesUpcoming => "/launches/upcoming",
            Self::LaunchesPast => "/launches/past",

            // Rockets
            Self::Rockets => "/rockets",

            // Crew
            Self::Crew => "/crew",

            // Starlink
            Self::Starlink => "/starlink",

            // Launchpads
            Self::Launchpads => "/launchpads",

            // Landpads
            Self::Landpads => "/landpads",

            // Payloads
            Self::Payloads => "/payloads",

            // Capsules
            Self::Capsules => "/capsules",
        }
    }
}
