//! NASA API endpoints

/// Base URLs for NASA API
pub struct NasaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for NasaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.nasa.gov",
            ws_base: None, // NASA does not support WebSocket
        }
    }
}

/// NASA API endpoint enum
#[derive(Debug, Clone)]
pub enum NasaEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // NEO (Near Earth Objects) ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get Near Earth Objects feed by date range
    NeoFeed,
    /// Get specific asteroid by ID
    NeoLookup(String), // asteroid_id

    // ═══════════════════════════════════════════════════════════════════════
    // DONKI (Space Weather Database Of Notifications, Knowledge, Information)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get Coronal Mass Ejections
    DonkiCme,
    /// Get Geomagnetic Storms
    DonkiGst,
    /// Get Solar Flares
    DonkiFlr,
    /// Get Solar Energetic Particles
    DonkiSep,
    /// Get Interplanetary Shocks
    DonkiIps,

    // ═══════════════════════════════════════════════════════════════════════
    // OTHER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get Astronomy Picture of the Day
    Apod,
    /// Get Earth Polychromatic Imaging Camera (EPIC) natural color images
    EpicNatural,
}

impl NasaEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // NEO
            Self::NeoFeed => "/neo/rest/v1/feed".to_string(),
            Self::NeoLookup(id) => format!("/neo/rest/v1/neo/{}", id),

            // DONKI
            Self::DonkiCme => "/DONKI/CME".to_string(),
            Self::DonkiGst => "/DONKI/GST".to_string(),
            Self::DonkiFlr => "/DONKI/FLR".to_string(),
            Self::DonkiSep => "/DONKI/SEP".to_string(),
            Self::DonkiIps => "/DONKI/IPS".to_string(),

            // Other
            Self::Apod => "/planetary/apod".to_string(),
            Self::EpicNatural => "/EPIC/api/natural".to_string(),
        }
    }
}
