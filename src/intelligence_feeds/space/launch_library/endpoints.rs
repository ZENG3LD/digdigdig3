//! Launch Library 2 API endpoints

/// Base URLs for Launch Library 2 API
pub struct LaunchLibraryEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for LaunchLibraryEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ll.thespacedevs.com/2.2.0",
            ws_base: None, // Launch Library 2 does not support WebSocket
        }
    }
}

/// Launch Library 2 API endpoint enum
#[derive(Debug, Clone)]
pub enum LaunchLibraryEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCH ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get upcoming launches
    LaunchUpcoming,
    /// Get previous launches
    LaunchPrevious,
    /// Get launch details by ID
    LaunchDetail,

    // ═══════════════════════════════════════════════════════════════════════
    // EVENT ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get upcoming events (landings, dockings, etc.)
    EventUpcoming,
    /// Get previous events
    EventPrevious,

    // ═══════════════════════════════════════════════════════════════════════
    // ASTRONAUT ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get astronaut data
    Astronaut,

    // ═══════════════════════════════════════════════════════════════════════
    // SPACE STATION ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get space station data
    SpaceStation,

    // ═══════════════════════════════════════════════════════════════════════
    // AGENCY ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get space agency data
    Agency,

    // ═══════════════════════════════════════════════════════════════════════
    // ROCKET ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get rocket/launch vehicle configuration data
    Rocket,

    // ═══════════════════════════════════════════════════════════════════════
    // SPACECRAFT ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get spacecraft data
    Spacecraft,
}

impl LaunchLibraryEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Launch endpoints
            Self::LaunchUpcoming => "/launch/upcoming/",
            Self::LaunchPrevious => "/launch/previous/",
            Self::LaunchDetail => "/launch/", // Requires ID appended

            // Event endpoints
            Self::EventUpcoming => "/event/upcoming/",
            Self::EventPrevious => "/event/previous/",

            // Astronaut endpoint
            Self::Astronaut => "/astronaut/",

            // Space station endpoint
            Self::SpaceStation => "/space_station/",

            // Agency endpoint
            Self::Agency => "/agency/",

            // Rocket endpoint
            Self::Rocket => "/config/launcher/",

            // Spacecraft endpoint
            Self::Spacecraft => "/config/spacecraft/",
        }
    }

    /// Build URL with ID parameter for detail endpoints
    pub fn path_with_id(&self, id: &str) -> String {
        format!("{}{}/", self.path(), id)
    }
}
