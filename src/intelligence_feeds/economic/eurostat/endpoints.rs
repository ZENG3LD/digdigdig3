//! Eurostat API endpoints

/// Base URLs for Eurostat API
pub struct EurostatEndpoints {
    pub statistics_base: &'static str,
    pub sdmx_base: &'static str,
    pub catalogue_base: &'static str,
}

impl Default for EurostatEndpoints {
    fn default() -> Self {
        Self {
            statistics_base: "https://ec.europa.eu/eurostat/api/dissemination/statistics/1.0",
            sdmx_base: "https://ec.europa.eu/eurostat/api/dissemination/sdmx/2.1",
            catalogue_base: "https://ec.europa.eu/eurostat/api/dissemination/catalogue",
        }
    }
}

/// Eurostat API endpoint enum
#[derive(Debug, Clone)]
pub enum EurostatEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // STATISTICS API (Main data access)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get dataset observations - /data/{dataset_code}
    Data { dataset_code: String },

    /// Get dataset label metadata - /label/{dataset_code}
    Label { dataset_code: String },

    // ═══════════════════════════════════════════════════════════════════════
    // SDMX API (Metadata and structure)
    // ═══════════════════════════════════════════════════════════════════════
    /// List all dataflows - /dataflow/ESTAT/all/latest
    ListDataflows,

    /// Get specific dataflow - /dataflow/ESTAT/{id}/latest
    Dataflow { dataflow_id: String },

    /// Get data via SDMX - /data/{dataflow_id}/{key}
    DataSdmx { dataflow_id: String, key: String },

    /// Get datastructure definition - /datastructure/ESTAT/{id}
    Datastructure { dsd_id: String },

    /// Get codelist - /codelist/ESTAT/{id}
    Codelist { codelist_id: String },

    /// Get concept scheme - /conceptscheme/ESTAT/{id}
    ConceptScheme { id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // CATALOGUE API (Table of contents)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get table of contents - /toc
    TableOfContents,
}

impl EurostatEndpoint {
    /// Get endpoint path and which base URL to use
    pub fn path_and_base(&self) -> (EndpointBase, String) {
        match self {
            // Statistics API
            Self::Data { dataset_code } => (
                EndpointBase::Statistics,
                format!("/data/{}", dataset_code),
            ),
            Self::Label { dataset_code } => (
                EndpointBase::Statistics,
                format!("/label/{}", dataset_code),
            ),

            // SDMX API
            Self::ListDataflows => (
                EndpointBase::Sdmx,
                "/dataflow/ESTAT/all/latest".to_string(),
            ),
            Self::Dataflow { dataflow_id } => (
                EndpointBase::Sdmx,
                format!("/dataflow/ESTAT/{}/latest", dataflow_id),
            ),
            Self::DataSdmx { dataflow_id, key } => (
                EndpointBase::Sdmx,
                format!("/data/{}/{}", dataflow_id, key),
            ),
            Self::Datastructure { dsd_id } => (
                EndpointBase::Sdmx,
                format!("/datastructure/ESTAT/{}", dsd_id),
            ),
            Self::Codelist { codelist_id } => (
                EndpointBase::Sdmx,
                format!("/codelist/ESTAT/{}", codelist_id),
            ),
            Self::ConceptScheme { id } => (
                EndpointBase::Sdmx,
                format!("/conceptscheme/ESTAT/{}", id),
            ),

            // Catalogue API
            Self::TableOfContents => (
                EndpointBase::Catalogue,
                "/toc".to_string(),
            ),
        }
    }
}

/// Which base URL to use for an endpoint
#[derive(Debug, Clone, Copy)]
pub enum EndpointBase {
    Statistics,
    Sdmx,
    Catalogue,
}

/// Format dataset code for Eurostat API
///
/// Eurostat uses dataset codes like "nama_10_gdp", "prc_hicp_midx"
/// For compatibility with Symbol type, we use base = dataset_code
pub fn _format_dataset_code(symbol: &crate::core::types::Symbol) -> String {
    symbol.base.to_lowercase()
}

/// Parse dataset code from response to domain Symbol
pub fn _parse_dataset_code(dataset_code: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(dataset_code, "")
}
