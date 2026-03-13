//! AIS API endpoints

/// Base URLs for Datalastic AIS API
pub struct AisEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for AisEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.datalastic.com/api/v0",
            ws_base: None, // Datalastic AIS does not support WebSocket
        }
    }
}

/// AIS API endpoint enum
#[derive(Debug, Clone)]
pub enum AisEndpoint {
    /// Search vessels by name, MMSI, IMO, callsign
    VesselFind,
    /// Get vessel details by UUID
    VesselInfo,
    /// Get vessel position history
    VesselHistory,
    /// Get vessel current position (premium)
    VesselPro,
    /// Search ports
    PortFind,
    /// Get port details by UUID
    PortInfo,
    /// Get live fleet positions in area
    FleetLiveMap,

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all vessels within a given radius of a point (premium)
    VesselInradius,
    /// Get bulk vessel data for a list of MMSIs
    VesselBulk,
    /// Get vessel estimated arrival (ETA) and destination (premium)
    VesselProEstimated,
}

impl AisEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::VesselFind => "/vessel_find",
            Self::VesselInfo => "/vessel_info",
            Self::VesselHistory => "/vessel_history",
            Self::VesselPro => "/vessel_pro",
            Self::PortFind => "/port_find",
            Self::PortInfo => "/port_info",
            Self::FleetLiveMap => "/fleet_live_map",

            // C7 additions
            Self::VesselInradius => "/vessel_inradius",
            Self::VesselBulk => "/vessel_bulk",
            Self::VesselProEstimated => "/vessel_pro_estimated",
        }
    }
}
