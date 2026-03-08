//! Feodo Tracker API endpoints

/// Base URLs for Feodo Tracker API
pub struct FeodoTrackerEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for FeodoTrackerEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://feodotracker.abuse.ch",
            ws_base: None, // Feodo Tracker does not support WebSocket
        }
    }
}

/// Feodo Tracker API endpoint enum
#[derive(Debug, Clone)]
pub enum FeodoTrackerEndpoint {
    /// IP blocklist (past 30 days) - JSON format with full metadata
    IpBlocklist,
    /// Aggressive blocklist (all historical) - CSV format
    IpBlocklistAggressive,
    /// Recommended blocklist - JSON array of IPs only
    IpBlocklistRecommended,
    /// IP blocklist CSV
    IpBlocklistCsv,
}

impl FeodoTrackerEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::IpBlocklist => "/downloads/ipblocklist.json",
            Self::IpBlocklistAggressive => "/downloads/ipblocklist_aggressive.csv",
            Self::IpBlocklistRecommended => "/downloads/ipblocklist_recommended.json",
            Self::IpBlocklistCsv => "/downloads/ipblocklist.csv",
        }
    }
}
