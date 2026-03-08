//! BIS connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    BisParser, SdmxObservation, SdmxDataflow, SdmxDataStructure,
    SdmxCodelist, SdmxConceptScheme, SdmxAvailability,
};

/// BIS (Bank for International Settlements) connector
///
/// Provides access to international banking and financial statistics via SDMX API.
///
/// # Key Dataflows
/// - `WS_CBPOL` - Central bank policy rates
/// - `WS_XRU` - US dollar exchange rates
/// - `WS_EER` - Effective exchange rates
/// - `WS_LONG_CPI` - Long consumer price indices
/// - `WS_SPP` - Residential property prices
/// - `WS_CREDIT` - Credit statistics
/// - `WS_DSR` - Debt service ratios
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::bis::BisConnector;
///
/// let connector = BisConnector::new();
///
/// // Get central bank policy rates
/// let observations = connector.get_data("WS_CBPOL", "all", None, None).await?;
///
/// // List all dataflows
/// let dataflows = connector.list_dataflows().await?;
/// ```
pub struct BisConnector {
    client: Client,
    _auth: BisAuth,
    endpoints: BisEndpoints,
}

impl BisConnector {
    /// Create new BIS connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: BisAuth::new(),
            endpoints: BisEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for BIS - public API)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to BIS API with SDMX JSON format
    async fn get(
        &self,
        endpoint: BisEndpoint,
        params: HashMap<String, String>,
        accept_type: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .header("Accept", accept_type)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for SDMX errors
        BisParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data for a specific dataflow with key filter
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID (e.g., "WS_CBPOL", "WS_XRU")
    /// - `key` - Dimension key filter (e.g., "US", "all")
    /// - `start_period` - Optional start period (e.g., "2020-01")
    /// - `end_period` - Optional end period (e.g., "2024-01")
    ///
    /// # Returns
    /// Vector of SDMX observations
    pub async fn get_data(
        &self,
        dataflow: &str,
        key: &str,
        start_period: Option<&str>,
        end_period: Option<&str>,
    ) -> ExchangeResult<Vec<SdmxObservation>> {
        let mut params = HashMap::new();
        params.insert("dimensionAtObservation".to_string(), "AllDimensions".to_string());

        if let Some(start) = start_period {
            params.insert("startPeriod".to_string(), start.to_string());
        }
        if let Some(end) = end_period {
            params.insert("endPeriod".to_string(), end.to_string());
        }

        let endpoint = BisEndpoint::Data {
            dataflow: dataflow.to_string(),
            key: key.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.data+json").await?;
        BisParser::parse_data_observations(&response)
    }

    /// Get all data for a dataflow
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID (e.g., "WS_CBPOL")
    ///
    /// # Returns
    /// Vector of all observations for the dataflow
    pub async fn get_data_all(&self, dataflow: &str) -> ExchangeResult<Vec<SdmxObservation>> {
        let params = HashMap::new();

        let endpoint = BisEndpoint::DataAll {
            dataflow: dataflow.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.data+json").await?;
        BisParser::parse_data_observations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// List all available dataflows
    ///
    /// # Returns
    /// Vector of dataflow metadata
    pub async fn list_dataflows(&self) -> ExchangeResult<Vec<SdmxDataflow>> {
        let params = HashMap::new();
        let response = self.get(BisEndpoint::Dataflows, params, "application/vnd.sdmx.structure+json").await?;
        BisParser::parse_dataflows(&response)
    }

    /// Get specific dataflow metadata
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID (e.g., "WS_CBPOL")
    ///
    /// # Returns
    /// Dataflow metadata
    pub async fn get_dataflow(&self, dataflow_id: &str) -> ExchangeResult<SdmxDataflow> {
        let params = HashMap::new();
        let endpoint = BisEndpoint::Dataflow {
            dataflow_id: dataflow_id.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.structure+json").await?;
        BisParser::parse_dataflow(&response)
    }

    /// Get data structure definition
    ///
    /// # Arguments
    /// - `dsd_id` - Data structure definition ID
    ///
    /// # Returns
    /// Data structure metadata
    pub async fn get_datastructure(&self, dsd_id: &str) -> ExchangeResult<SdmxDataStructure> {
        let params = HashMap::new();
        let endpoint = BisEndpoint::DataStructure {
            dsd_id: dsd_id.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.structure+json").await?;
        BisParser::parse_data_structure(&response)
    }

    /// Get codelist (dimension values)
    ///
    /// # Arguments
    /// - `codelist_id` - Codelist ID
    ///
    /// # Returns
    /// Codelist with available codes
    pub async fn get_codelist(&self, codelist_id: &str) -> ExchangeResult<SdmxCodelist> {
        let params = HashMap::new();
        let endpoint = BisEndpoint::Codelist {
            codelist_id: codelist_id.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.structure+json").await?;
        BisParser::parse_codelist(&response)
    }

    /// Get concept scheme
    ///
    /// # Arguments
    /// - `id` - Concept scheme ID
    ///
    /// # Returns
    /// Concept scheme metadata
    pub async fn get_concept_scheme(&self, id: &str) -> ExchangeResult<SdmxConceptScheme> {
        let params = HashMap::new();
        let endpoint = BisEndpoint::ConceptScheme {
            id: id.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.structure+json").await?;
        BisParser::parse_concept_scheme(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AVAILABILITY ENDPOINT
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data availability for a dataflow and key
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID
    /// - `key` - Dimension key filter
    ///
    /// # Returns
    /// Availability information including time period coverage
    pub async fn get_availability(
        &self,
        dataflow: &str,
        key: &str,
    ) -> ExchangeResult<SdmxAvailability> {
        let params = HashMap::new();
        let endpoint = BisEndpoint::Availability {
            dataflow: dataflow.to_string(),
            key: key.to_string(),
        };

        let response = self.get(endpoint, params, "application/vnd.sdmx.data+json").await?;
        BisParser::parse_availability(&response)
    }
}

impl Default for BisConnector {
    fn default() -> Self {
        Self::new()
    }
}
