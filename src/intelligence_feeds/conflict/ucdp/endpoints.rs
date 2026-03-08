//! UCDP API endpoints

/// Base URLs for UCDP API
pub struct UcdpEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for UcdpEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://ucdpapi.pcr.uu.se/api",
            ws_base: None, // UCDP does not support WebSocket
        }
    }
}

/// UCDP API endpoint enum
#[derive(Debug, Clone)]
pub enum UcdpEndpoint {
    /// Get georeferenced event data
    GeoEvents,
    /// Get battle-related deaths
    BattleDeaths,
    /// Get non-state conflicts
    NonState,
    /// Get one-sided violence
    OneSided,
    /// Get state-based conflicts
    StateConflict,
}

impl UcdpEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::GeoEvents => "/gedevents/24.1",
            Self::BattleDeaths => "/battledeaths/24.1",
            Self::NonState => "/nonstate/24.1",
            Self::OneSided => "/onesided/24.1",
            Self::StateConflict => "/stateconflict/24.1",
        }
    }
}
