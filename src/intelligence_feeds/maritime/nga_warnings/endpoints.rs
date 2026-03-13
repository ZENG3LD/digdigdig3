//! NGA Maritime Warnings API endpoints

/// Base URLs for NGA MSI API
pub struct NgaWarningsEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NgaWarningsEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://msi.nga.mil/api/publications",
            ws_base: None, // NGA MSI does not support WebSocket
        }
    }
}

/// NGA Maritime Warnings API endpoint enum
#[derive(Debug, Clone)]
pub enum NgaWarningsEndpoint {
    /// Broadcast warnings endpoint (status=A for active)
    BroadcastWarnings,
    /// Navigational warnings endpoint
    NavigationalWarnings,
    /// Get specific warning by ID
    WarningById { id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// ASAM anti-piracy / maritime security incident reports
    AsamPiracyReports,
    /// MODU (Mobile Offshore Drilling Unit) positions
    ModuPositions,
    /// World Port Index (WPI) — global port database
    WorldPortIndex,
}

impl NgaWarningsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::BroadcastWarnings => "/broadcast-warn".to_string(),
            Self::NavigationalWarnings => "/navwarn".to_string(),
            Self::WarningById { id } => format!("/warn/{}", id),

            // C7 additions
            Self::AsamPiracyReports => "/asam".to_string(),
            Self::ModuPositions => "/modu".to_string(),
            Self::WorldPortIndex => "/wpi".to_string(),
        }
    }
}
