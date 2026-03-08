//! BIS API endpoints

/// Base URLs for BIS SDMX API
pub struct BisEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for BisEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://stats.bis.org/api/v2",
            ws_base: None, // BIS does not support WebSocket
        }
    }
}

/// BIS API endpoint enum
#[derive(Debug, Clone)]
pub enum BisEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data for a specific dataflow with key filter
    Data {
        dataflow: String,
        key: String,
    },
    /// Get all data for a dataflow
    DataAll {
        dataflow: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS (5)
    // ═══════════════════════════════════════════════════════════════════════
    /// List all available dataflows
    Dataflows,
    /// Get specific dataflow metadata
    Dataflow {
        dataflow_id: String,
    },
    /// Get data structure definition
    DataStructure {
        dsd_id: String,
    },
    /// Get codelist (dimension values)
    Codelist {
        codelist_id: String,
    },
    /// Get concept scheme
    ConceptScheme {
        id: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // AVAILABILITY ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data availability for a dataflow and key
    Availability {
        dataflow: String,
        key: String,
    },
}

impl BisEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Data
            Self::Data { dataflow, key } => {
                format!("/data/dataflow/BIS/{}/latest/{}", dataflow, key)
            }
            Self::DataAll { dataflow } => {
                format!("/data/dataflow/BIS/{}/latest/all", dataflow)
            }

            // Structure
            Self::Dataflows => "/structure/dataflow/BIS".to_string(),
            Self::Dataflow { dataflow_id } => {
                format!("/structure/dataflow/BIS/{}/latest", dataflow_id)
            }
            Self::DataStructure { dsd_id } => {
                format!("/structure/datastructure/BIS/{}", dsd_id)
            }
            Self::Codelist { codelist_id } => {
                format!("/structure/codelist/BIS/{}", codelist_id)
            }
            Self::ConceptScheme { id } => {
                format!("/structure/conceptscheme/BIS/{}", id)
            }

            // Availability
            Self::Availability { dataflow, key } => {
                format!("/availability/dataflow/BIS/{}/latest/{}", dataflow, key)
            }
        }
    }
}

/// Format dataflow ID for BIS API
///
/// BIS uses dataflow IDs like "WS_CBPOL", "WS_XRU", etc.
/// This is different from crypto exchanges - there's no base/quote concept.
/// Dataflow IDs are unique identifiers in the BIS database.
///
/// For compatibility with the Symbol type, we'll use:
/// - base = dataflow_id
/// - quote = "" (empty)
pub fn _format_dataflow_id(symbol: &crate::core::types::Symbol) -> String {
    symbol.base.to_uppercase()
}

/// Parse dataflow ID from BIS response to domain Symbol
///
/// BIS dataflow IDs become the "base" field, with empty "quote"
pub fn _parse_dataflow_id(dataflow_id: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(dataflow_id, "")
}
