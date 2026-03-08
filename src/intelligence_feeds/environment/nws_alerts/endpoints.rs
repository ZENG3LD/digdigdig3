//! NWS Weather Alerts API endpoints

/// Base URLs for NWS API
pub struct NwsAlertsEndpoints {
    pub rest_base: &'static str,
}

impl Default for NwsAlertsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.weather.gov",
        }
    }
}

/// NWS Alerts API endpoint enum
#[derive(Debug, Clone)]
pub enum NwsAlertsEndpoint {
    /// Get all active alerts
    ActiveAlerts,
    /// Get specific alert by ID
    AlertById(String),
    /// Get active alerts by zone
    AlertsByZone(String),
    /// Get active alerts by state/area
    AlertsByArea(String),
}

impl NwsAlertsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::ActiveAlerts => "/alerts/active".to_string(),
            Self::AlertById(id) => format!("/alerts/{}", id),
            Self::AlertsByZone(zone) => format!("/alerts/active/zone/{}", zone),
            Self::AlertsByArea(area) => format!("/alerts/active/area/{}", area),
        }
    }
}
