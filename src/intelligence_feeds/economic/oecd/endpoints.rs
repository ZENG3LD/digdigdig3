//! OECD SDMX REST API endpoints

/// Base URLs for OECD SDMX API
pub struct OecdEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OecdEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://sdmx.oecd.org/public/rest",
            ws_base: None, // OECD does not support WebSocket
        }
    }
}

/// OECD SDMX API endpoint enum
#[derive(Debug, Clone)]
pub enum OecdEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data with filters
    Data { dataflow_id: String, key: String },
    /// Get all data for a dataflow
    DataAll { dataflow_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS (5)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get dataflow by agency and ID
    Dataflow { agency: String, id: String },
    /// List all dataflows for an agency
    DataflowList { agency: String },
    /// Get datastructure definition
    Datastructure { agency: String, id: String },
    /// Get codelist
    Codelist { agency: String, id: String },
    /// Get concept scheme
    ConceptScheme { agency: String, id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // AVAILABILITY ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data availability constraints
    Availability { dataflow_id: String, key: String },
}

impl OecdEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Data endpoints
            Self::Data { dataflow_id, key } => {
                format!("/data/{}/{}", dataflow_id, key)
            }
            Self::DataAll { dataflow_id } => {
                format!("/data/{}/all", dataflow_id)
            }

            // Structure endpoints
            Self::Dataflow { agency, id } => {
                format!("/dataflow/{}/{}", agency, id)
            }
            Self::DataflowList { agency } => {
                format!("/dataflow/{}", agency)
            }
            Self::Datastructure { agency, id } => {
                format!("/datastructure/{}/{}", agency, id)
            }
            Self::Codelist { agency, id } => {
                format!("/codelist/{}/{}", agency, id)
            }
            Self::ConceptScheme { agency, id } => {
                format!("/conceptscheme/{}/{}", agency, id)
            }

            // Availability endpoint
            Self::Availability { dataflow_id, key } => {
                format!("/availableconstraint/{}/{}", dataflow_id, key)
            }
        }
    }
}

/// Common OECD dataflow IDs
pub mod dataflows {
    /// Quarterly National Accounts (GDP data)
    pub const QNA: &str = "QNA";

    /// Consumer Price Index
    pub const PRICES_CPI: &str = "PRICES_CPI";

    /// Composite Leading Indicators (includes unemployment)
    pub const MEI_CLI: &str = "MEI_CLI";

    /// Financial Indicators (interest rates)
    pub const MEI_FIN: &str = "MEI_FIN";

    /// International Trade
    pub const MEI_TRD: &str = "MEI_TRD";

    /// Real Sector (industrial production)
    pub const MEI_REAL: &str = "MEI_REAL";
}

/// Common OECD agencies
pub mod agencies {
    /// Main OECD agency
    pub const OECD: &str = "OECD";
}
