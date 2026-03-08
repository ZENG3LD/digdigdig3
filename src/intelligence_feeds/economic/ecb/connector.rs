//! ECB connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{EcbParser, SdmxObservation, SdmxDataflow};

/// ECB (European Central Bank) connector
///
/// Provides access to ECB statistical data via SDMX 2.1 REST API.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ecb::EcbConnector;
///
/// let connector = EcbConnector::new();
///
/// // Get exchange rate data (USD/EUR)
/// let observations = connector.get_data("EXR", "D.USD.EUR.SP00.A", Some("2023-01-01"), Some("2023-12-31")).await?;
///
/// // List available dataflows
/// let dataflows = connector.list_dataflows().await?;
///
/// // Get specific dataflow metadata
/// let dataflow = connector.get_dataflow("EXR").await?;
/// ```
pub struct EcbConnector {
    client: Client,
    _auth: EcbAuth,
    endpoints: EcbEndpoints,
}

impl EcbConnector {
    /// Create new ECB connector
    ///
    /// No authentication required for ECB API
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: EcbAuth::new(),
            endpoints: EcbEndpoints::default(),
        }
    }

    /// Internal: Make GET request to ECB API with JSON data Accept header
    async fn get_data_json(
        &self,
        path: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, path);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Accept", "application/vnd.sdmx.data+json;charset=utf-8;version=1.0.0-wd")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_body),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for SDMX errors
        EcbParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request to ECB API with JSON structure Accept header
    async fn get_structure_json(
        &self,
        path: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, path);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Accept", "application/vnd.sdmx.structure+json;charset=utf-8;version=1.0.0-wd")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_body),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for SDMX errors
        EcbParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECB-SPECIFIC METHODS (Data API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data observations for a dataflow and key
    ///
    /// This is the CORE endpoint for retrieving time series data.
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID (e.g., "EXR", "ICP", "BSI")
    /// - `key` - SDMX key (e.g., "D.USD.EUR.SP00.A" for daily USD/EUR exchange rate)
    /// - `start_period` - Optional start date (YYYY-MM-DD)
    /// - `end_period` - Optional end date (YYYY-MM-DD)
    ///
    /// # Common dataflows:
    /// - **EXR**: Exchange rates (e.g., key "D.USD.EUR.SP00.A")
    /// - **FM**: Financial market data (interest rates)
    /// - **BSI**: Balance sheet items (money supply)
    /// - **ICP**: Index of consumer prices (HICP inflation)
    /// - **MNA**: National accounts (GDP)
    /// - **BOP**: Balance of payments
    /// - **GFS**: Government finance statistics
    /// - **SEC**: Securities statistics
    ///
    /// # Returns
    /// Vector of observations with series key, time period, and value
    pub async fn get_data(
        &self,
        dataflow: &str,
        key: &str,
        start_period: Option<&str>,
        end_period: Option<&str>,
    ) -> ExchangeResult<Vec<SdmxObservation>> {
        let mut params = HashMap::new();
        params.insert("detail".to_string(), "dataonly".to_string());

        if let Some(start) = start_period {
            params.insert("startPeriod".to_string(), start.to_string());
        }
        if let Some(end) = end_period {
            params.insert("endPeriod".to_string(), end.to_string());
        }

        let path = EcbEndpoint::data_path(dataflow, key);
        let response = self.get_data_json(&path, params).await?;
        EcbParser::parse_data_observations(&response)
    }

    /// Get full data (including metadata) for a dataflow and key
    ///
    /// Similar to `get_data` but includes all metadata (attributes, annotations, etc.)
    pub async fn get_data_full(
        &self,
        dataflow: &str,
        key: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("detail".to_string(), "full".to_string());

        let path = EcbEndpoint::data_path(dataflow, key);
        self.get_data_json(&path, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECB-SPECIFIC METHODS (Structure API)
    // ═══════════════════════════════════════════════════════════════════════

    /// List all available dataflows from ECB
    ///
    /// Returns metadata about all data collections available from ECB.
    /// Use this to discover what data is available.
    pub async fn list_dataflows(&self) -> ExchangeResult<Vec<SdmxDataflow>> {
        let params = HashMap::new();
        let response = self.get_structure_json(EcbEndpoint::Dataflows.path(), params).await?;
        EcbParser::parse_dataflows(&response)
    }

    /// Get metadata for a specific dataflow
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID (e.g., "EXR", "ICP")
    ///
    /// Returns detailed metadata about the dataflow including dimensions and structure
    pub async fn get_dataflow(&self, dataflow_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let path = EcbEndpoint::dataflow_path(dataflow_id);
        self.get_structure_json(&path, params).await
    }

    /// Get data structure definition (DSD)
    ///
    /// # Arguments
    /// - `dsd_id` - Data structure definition ID (usually matches dataflow ID)
    ///
    /// Returns the complete structure definition including all dimensions,
    /// attributes, and their allowed values
    pub async fn get_datastructure(&self, dsd_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let path = EcbEndpoint::datastructure_path(dsd_id);
        self.get_structure_json(&path, params).await
    }

    /// Get codelist (allowed values for a dimension)
    ///
    /// # Arguments
    /// - `codelist_id` - Codelist ID (e.g., "CL_FREQ" for frequency, "CL_CURRENCY" for currency)
    ///
    /// Returns all valid codes and their descriptions for the specified codelist
    pub async fn get_codelist(&self, codelist_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let path = EcbEndpoint::codelist_path(codelist_id);
        self.get_structure_json(&path, params).await
    }

    /// Get concept scheme
    ///
    /// # Arguments
    /// - `scheme_id` - Concept scheme ID
    ///
    /// Returns metadata about statistical concepts used in the data
    pub async fn get_concept_scheme(&self, scheme_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let path = EcbEndpoint::conceptscheme_path(scheme_id);
        self.get_structure_json(&path, params).await
    }
}

impl Default for EcbConnector {
    fn default() -> Self {
        Self::new()
    }
}
