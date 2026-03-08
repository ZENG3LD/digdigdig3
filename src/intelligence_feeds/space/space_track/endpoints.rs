//! Space-Track.org API endpoints

/// Base URLs for Space-Track API
pub struct SpaceTrackEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SpaceTrackEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.space-track.org",
            ws_base: None, // Space-Track does not support WebSocket
        }
    }
}

/// Space-Track API endpoint enum
#[derive(Debug, Clone)]
pub enum SpaceTrackEndpoint {
    // Authentication endpoint
    Login,

    // Satellite catalog - recent launches
    SatelliteCatalog,

    // General Perturbations (TLE) data for specific satellite
    GeneralPerturbations { norad_id: u32 },

    // Decay predictions
    Decay,

    // Space debris tracking
    Debris,

    // Launch sites
    LaunchSites,

    // Tracking & Impact Predictions
    Tip,
}

impl SpaceTrackEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Login => "/ajaxauth/login".to_string(),
            Self::SatelliteCatalog => {
                "/basicspacedata/query/class/satcat/orderby/LAUNCH desc/limit/25/format/json".to_string()
            }
            Self::GeneralPerturbations { norad_id } => {
                format!(
                    "/basicspacedata/query/class/gp/NORAD_CAT_ID/{}/format/json",
                    norad_id
                )
            }
            Self::Decay => {
                "/basicspacedata/query/class/decay/orderby/DECAY_EPOCH desc/limit/25/format/json".to_string()
            }
            Self::Debris => {
                "/basicspacedata/query/class/gp/OBJECT_TYPE/DEBRIS/orderby/LAUNCH desc/limit/50/format/json".to_string()
            }
            Self::LaunchSites => {
                "/basicspacedata/query/class/launch_site/format/json".to_string()
            }
            Self::Tip => {
                "/basicspacedata/query/class/tip/format/json".to_string()
            }
        }
    }
}
