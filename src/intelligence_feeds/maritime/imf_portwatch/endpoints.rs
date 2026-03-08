//! IMF PortWatch API endpoints

/// Base URLs for IMF PortWatch API
pub struct ImfPortWatchEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ImfPortWatchEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://portwatch.imf.org/api",
            ws_base: None, // IMF PortWatch does not support WebSocket
        }
    }
}

/// IMF PortWatch API endpoint enum
#[derive(Debug, Clone)]
pub enum ImfPortWatchEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // CHOKEPOINT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List all 28 maritime chokepoints
    Chokepoints,
    /// Get traffic statistics for a specific chokepoint
    ChokepointStats,

    // ═══════════════════════════════════════════════════════════════════════
    // PORT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List major ports
    Ports,
    /// Get port traffic statistics
    PortStats,

    // ═══════════════════════════════════════════════════════════════════════
    // TRADE FLOW ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get global trade flow data
    TradeFlows,

    // ═══════════════════════════════════════════════════════════════════════
    // DISRUPTION ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get active disruptions
    Disruptions,
}

impl ImfPortWatchEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Chokepoints => "/portwatch/v1/chokepoints",
            Self::ChokepointStats => "/portwatch/v1/chokepoints/statistics",
            Self::Ports => "/portwatch/v1/ports",
            Self::PortStats => "/portwatch/v1/ports/statistics",
            Self::TradeFlows => "/portwatch/v1/trade-flows",
            Self::Disruptions => "/portwatch/v1/disruptions",
        }
    }

    /// Get endpoint path with ID parameter
    pub fn path_with_id(&self, id: &str) -> String {
        match self {
            Self::ChokepointStats => format!("/portwatch/v1/chokepoints/{}/statistics", id),
            Self::PortStats => format!("/portwatch/v1/ports/{}/statistics", id),
            _ => self.path().to_string(),
        }
    }
}
