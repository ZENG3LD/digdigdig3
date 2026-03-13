//! IMF API endpoints

/// Base URLs for IMF API
pub struct ImfEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for ImfEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "http://dataservices.imf.org/REST/SDMX_JSON.svc",
            ws_base: None, // IMF does not support WebSocket
        }
    }
}

/// IMF API endpoint enum
#[derive(Debug, Clone)]
pub enum ImfEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // CORE DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get all available dataflows (datasets)
    Dataflow,
    /// Get compact data for a database with dimensions
    CompactData { database_id: String, dimensions: String },
    /// Get data structure definition for a database
    DataStructure { database_id: String },
    /// Get code list for a dimension
    CodeList { code_list_id: String, database_id: String },
    /// Get generic data format
    GenericData { database_id: String, dimensions: String },

    // ═══════════════════════════════════════════════════════════════════════
    // C6 ADDITIONS — WEO (World Economic Outlook) endpoints
    // ═══════════════════════════════════════════════════════════════════════
    /// List all WEO indicators (series codes)
    WeoIndicators,
    /// List all WEO country codes
    WeoCountries,
    /// List all WEO regional aggregates
    WeoRegions,
    /// Fetch WEO forecast data: requires database_id="WEO"
    WeoData { dimensions: String },
}

impl ImfEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Dataflow => "/Dataflow".to_string(),
            Self::CompactData { database_id, dimensions } => {
                format!("/CompactData/{}/{}", database_id, dimensions)
            }
            Self::DataStructure { database_id } => {
                format!("/DataStructure/{}", database_id)
            }
            Self::CodeList { code_list_id, database_id } => {
                format!("/CodeList/{}_{}", code_list_id, database_id)
            }
            Self::GenericData { database_id, dimensions } => {
                format!("/GenericData/{}/{}", database_id, dimensions)
            }

            // C6 additions
            Self::WeoIndicators => "/CodeList/CONCEPT_WEO".to_string(),
            Self::WeoCountries => "/CodeList/ISO_WEO".to_string(),
            Self::WeoRegions => "/CodeList/REGIONGROUP_WEO".to_string(),
            Self::WeoData { dimensions } => {
                format!("/CompactData/WEO/{}", dimensions)
            }
        }
    }
}

/// Format dimension string for IMF API
///
/// IMF uses dot-separated dimensions: {freq}.{country}.{indicator}
/// Example: "A.US.NGDP_RPCH" = Annual US GDP growth
pub fn format_dimensions(freq: &str, country: &str, indicator: &str) -> String {
    format!("{}.{}.{}", freq, country, indicator)
}

/// Parse dimension string from IMF response
///
/// Returns (freq, country, indicator)
pub fn parse_dimensions(dimensions: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = dimensions.split('.').collect();
    if parts.len() >= 3 {
        Some((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
    } else {
        None
    }
}
