//! FBI Crime Data API endpoints

/// Base URLs for FBI Crime Data API
pub struct FbiCrimeEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for FbiCrimeEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.usa.gov/crime/fbi/sapi",
            ws_base: None, // FBI Crime Data API does not support WebSocket
        }
    }
}

/// FBI Crime Data API endpoint enum
#[derive(Debug, Clone)]
pub enum FbiCrimeEndpoint {
    /// National crime estimates
    NationalEstimates,
    /// State-level crime estimates
    StateEstimates { state: String },
    /// Summarized offense data for a state
    SummarizedOffense { state: String, offense: String },
    /// National agency participation rates
    NationalParticipation,
    /// List of agencies
    Agencies,
    /// NIBRS offender data by offense and state
    NibrsOffender { offense: String, state: String },
    /// NIBRS victim data by offense and state
    NibrsVictim { offense: String, state: String },
}

impl FbiCrimeEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::NationalEstimates => "/api/estimates/national".to_string(),
            Self::StateEstimates { state } => format!("/api/estimates/states/{}", state),
            Self::SummarizedOffense { state, offense } => {
                format!("/api/summarized/state/{}/{}", state, offense)
            }
            Self::NationalParticipation => "/api/participation/national".to_string(),
            Self::Agencies => "/api/agencies".to_string(),
            Self::NibrsOffender { offense, state } => {
                format!("/api/nibrs/{}/offender/states/{}/count", offense, state)
            }
            Self::NibrsVictim { offense, state } => {
                format!("/api/nibrs/{}/victim/states/{}/count", offense, state)
            }
        }
    }
}
