//! Deutsche Bundesbank API endpoints
//!
//! SDMX-based REST API for German economic and financial statistics

/// Base URLs for Bundesbank API
pub struct BundesbankEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for BundesbankEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.statistiken.bundesbank.de/rest",
            ws_base: None, // Bundesbank does not support WebSocket
        }
    }
}

/// Bundesbank API endpoint enum (SDMX REST)
#[derive(Debug, Clone)]
pub enum BundesbankEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get time series data by dataflow and key
    /// GET /data/{dataflow}/{key}?startPeriod=&endPeriod=&detail=dataonly&format=jsondata
    Data {
        dataflow: String,
        key: String,
    },

    /// Get time series data by time series IDs (POST)
    /// POST /data/tsIdList with body: tsIds={id1},{id2},...
    DataByTsId,

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// List all available dataflows
    /// GET /dataflow/BBK?format=jsondata
    ListDataflows,

    /// Get specific dataflow metadata
    /// GET /dataflow/BBK/{id}?format=jsondata
    Dataflow { id: String },

    /// Get data structure definition
    /// GET /datastructure/BBK/{id}?format=jsondata
    DataStructure { id: String },

    /// Get codelist (dimension values)
    /// GET /codelist/BBK/{id}?format=jsondata
    Codelist { id: String },

    /// Get concept scheme (metadata concepts)
    /// GET /conceptscheme/BBK/{id}?format=jsondata
    ConceptScheme { id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINT (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get metadata for a specific dataflow and key
    /// GET /metadata/dataflow/BBK/{dataflow}/{key}?format=jsondata
    Metadata {
        dataflow: String,
        key: String,
    },
}

impl BundesbankEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Data
            Self::Data { dataflow, key } => {
                format!("/data/{}/{}", dataflow, key)
            }
            Self::DataByTsId => "/data/tsIdList".to_string(),

            // Structure
            Self::ListDataflows => "/dataflow/BBK".to_string(),
            Self::Dataflow { id } => format!("/dataflow/BBK/{}", id),
            Self::DataStructure { id } => format!("/datastructure/BBK/{}", id),
            Self::Codelist { id } => format!("/codelist/BBK/{}", id),
            Self::ConceptScheme { id } => format!("/conceptscheme/BBK/{}", id),

            // Metadata
            Self::Metadata { dataflow, key } => {
                format!("/metadata/dataflow/BBK/{}/{}", dataflow, key)
            }
        }
    }
}

/// Common Bundesbank dataflow IDs
pub mod dataflows {
    /// Exchange rates
    pub const EXCHANGE_RATES: &str = "BBEX3";

    /// Securities statistics
    pub const SECURITIES: &str = "BBSIS";

    /// Financial market data
    pub const FINANCIAL_MARKETS: &str = "BBFID";

    /// Banking statistics
    pub const BANKING: &str = "BBK01";

    /// Investment funds
    pub const INVESTMENT_FUNDS: &str = "BBK_IVF";

    /// MFI (Monetary Financial Institutions) statistics
    pub const MFI_STATISTICS: &str = "BBMFI";
}

/// Format time series key for Bundesbank API
///
/// Bundesbank uses SDMX key syntax: dimension values separated by dots
/// Example: "D.EUR.USD.BB.AC.C04" for daily EUR/USD exchange rate
///
/// Use "+" for wildcard (all values) or specific dimension value
pub fn format_ts_key(dimensions: &[&str]) -> String {
    dimensions.join(".")
}

/// Format period for SDMX API (YYYY-MM-DD or YYYY-MM or YYYY)
pub fn format_period(date: &str) -> String {
    date.to_string()
}
