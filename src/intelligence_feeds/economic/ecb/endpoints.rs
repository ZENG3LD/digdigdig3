//! ECB API endpoints

/// Base URLs for ECB API
pub struct EcbEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for EcbEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://data-api.ecb.europa.eu/service",
            ws_base: None, // ECB does not support WebSocket
        }
    }
}

/// ECB API endpoint enum
#[derive(Debug, Clone)]
pub enum EcbEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data for a specific dataflow and key
    Data,

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// List all available dataflows from ECB
    Dataflows,
    /// Get specific dataflow metadata
    Dataflow,
    /// Get data structure definition
    DataStructure,
    /// Get codelist
    CodeList,
    /// Get concept scheme
    ConceptScheme,
}

impl EcbEndpoint {
    /// Get endpoint path
    ///
    /// Note: Some endpoints require parameters to be inserted into the path
    /// (e.g., dataflow ID). These return templates that need formatting.
    pub fn path(&self) -> &'static str {
        match self {
            // Data
            Self::Data => "/data",

            // Structure
            Self::Dataflows => "/dataflow/ECB",
            Self::Dataflow => "/dataflow/ECB/{id}/latest",
            Self::DataStructure => "/datastructure/ECB/{id}",
            Self::CodeList => "/codelist/ECB/{id}",
            Self::ConceptScheme => "/conceptscheme/ECB/{id}",
        }
    }

    /// Build full data endpoint path with dataflow and key
    pub fn data_path(dataflow: &str, key: &str) -> String {
        format!("/data/{}/{}", dataflow, key)
    }

    /// Build full dataflow endpoint path with ID
    pub fn dataflow_path(id: &str) -> String {
        format!("/dataflow/ECB/{}/latest", id)
    }

    /// Build full data structure endpoint path with ID
    pub fn datastructure_path(id: &str) -> String {
        format!("/datastructure/ECB/{}", id)
    }

    /// Build full codelist endpoint path with ID
    pub fn codelist_path(id: &str) -> String {
        format!("/codelist/ECB/{}", id)
    }

    /// Build full concept scheme endpoint path with ID
    pub fn conceptscheme_path(id: &str) -> String {
        format!("/conceptscheme/ECB/{}", id)
    }
}

/// Format dataflow key for ECB API
///
/// ECB uses SDMX keys like "D.USD.EUR.SP00.A" for exchange rates
/// Keys follow the structure defined by each dataflow's Data Structure Definition (DSD)
///
/// For compatibility with the Symbol type, we'll use:
/// - base = dataflow_id
/// - quote = key (or empty for all keys)
pub fn _format_dataflow_key(symbol: &crate::core::types::Symbol) -> (String, String) {
    // For ECB, the "base" field contains the dataflow ID
    // The "quote" field contains the key (or empty for default)
    let dataflow = symbol.base.to_uppercase();
    let key = if symbol.quote.is_empty() {
        "all".to_string()
    } else {
        symbol.quote.clone()
    };
    (dataflow, key)
}

/// Parse dataflow ID and key from ECB response to domain Symbol
///
/// ECB dataflow IDs become the "base" field, keys become "quote"
pub fn _parse_dataflow_key(dataflow: &str, key: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(dataflow, key)
}
