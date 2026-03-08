//! Deutsche Bundesbank connector implementation
//!
//! Provides access to German economic and financial statistics via SDMX REST API.

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    BundesbankParser, SdmxObservation, SdmxDataflow, SdmxDataStructure,
    SdmxCodelist, SdmxConceptScheme,
};

/// Deutsche Bundesbank connector
///
/// Provides access to German economic and financial statistics through the
/// Bundesbank's SDMX REST API.
///
/// # Key Dataflows
/// - `BBEX3` - Exchange rates
/// - `BBSIS` - Securities statistics
/// - `BBFID` - Financial market data
/// - `BBK01` - Banking statistics
/// - `BBK_IVF` - Investment funds
/// - `BBMFI` - MFI statistics
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::bundesbank::BundesbankConnector;
///
/// let connector = BundesbankConnector::new();
///
/// // Get exchange rate data
/// let observations = connector.get_data(
///     "BBEX3",
///     "D.EUR.USD.BB.AC.C04",
///     Some("2024-01-01"),
///     Some("2024-12-31")
/// ).await?;
///
/// // List all available dataflows
/// let dataflows = connector.list_dataflows().await?;
/// ```
pub struct BundesbankConnector {
    client: Client,
    auth: BundesbankAuth,
    endpoints: BundesbankEndpoints,
}

impl BundesbankConnector {
    /// Create new Bundesbank connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: BundesbankAuth::new(),
            endpoints: BundesbankEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for Bundesbank)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to Bundesbank API
    async fn get(
        &self,
        endpoint: BundesbankEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // No authentication required for Bundesbank
        self.auth.sign_query(&mut params);

        // Always request JSON format
        params.insert("format".to_string(), "jsondata".to_string());

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Accept", "application/json")
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

        // Check for API errors
        BundesbankParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to Bundesbank API
    async fn post(
        &self,
        endpoint: BundesbankEndpoint,
        body: String,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .query(&[("format", "jsondata")])
            .body(body)
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

        BundesbankParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get time series data by dataflow and key
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID (e.g., "BBEX3" for exchange rates)
    /// - `key` - SDMX key with dimension values (e.g., "D.EUR.USD.BB.AC.C04")
    ///   Use "+" for wildcard to get all values for a dimension
    /// - `start_period` - Optional start period (YYYY-MM-DD, YYYY-MM, or YYYY)
    /// - `end_period` - Optional end period
    ///
    /// # Returns
    /// Vector of observations with period and value
    ///
    /// # Example
    /// ```ignore
    /// // Get daily EUR/USD exchange rates for 2024
    /// let data = connector.get_data(
    ///     "BBEX3",
    ///     "D.EUR.USD.BB.AC.C04",
    ///     Some("2024-01-01"),
    ///     Some("2024-12-31")
    /// ).await?;
    /// ```
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

        let endpoint = BundesbankEndpoint::Data {
            dataflow: dataflow.to_string(),
            key: key.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        BundesbankParser::parse_observations(&response)
    }

    /// Get time series data by time series IDs (POST method)
    ///
    /// This method allows retrieving multiple time series at once by their IDs.
    ///
    /// # Arguments
    /// - `ts_ids` - Slice of time series IDs
    ///
    /// # Returns
    /// Vector of observations
    ///
    /// # Example
    /// ```ignore
    /// let data = connector.get_data_by_tsid(&[
    ///     "BBEX3.D.EUR.USD.BB.AC.C04",
    ///     "BBEX3.D.GBP.USD.BB.AC.C04",
    /// ]).await?;
    /// ```
    pub async fn get_data_by_tsid(&self, ts_ids: &[&str]) -> ExchangeResult<Vec<SdmxObservation>> {
        let body = format!("tsIds={}", ts_ids.join(","));
        let endpoint = BundesbankEndpoint::DataByTsId;
        let response = self.post(endpoint, body).await?;
        BundesbankParser::parse_observations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// List all available dataflows
    ///
    /// Returns a list of all available datasets in the Bundesbank API.
    ///
    /// # Example
    /// ```ignore
    /// let dataflows = connector.list_dataflows().await?;
    /// for df in dataflows {
    ///     println!("{}: {}", df.id, df.name);
    /// }
    /// ```
    pub async fn list_dataflows(&self) -> ExchangeResult<Vec<SdmxDataflow>> {
        let endpoint = BundesbankEndpoint::ListDataflows;
        let response = self.get(endpoint, HashMap::new()).await?;
        BundesbankParser::parse_dataflows(&response)
    }

    /// Get specific dataflow metadata
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID (e.g., "BBEX3", "BBSIS")
    ///
    /// # Returns
    /// Dataflow metadata including ID, name, and description
    pub async fn get_dataflow(&self, dataflow_id: &str) -> ExchangeResult<SdmxDataflow> {
        let endpoint = BundesbankEndpoint::Dataflow {
            id: dataflow_id.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        BundesbankParser::parse_dataflow(&response)
    }

    /// Get data structure definition
    ///
    /// Returns the structure of a dataset, including its dimensions and attributes.
    ///
    /// # Arguments
    /// - `dsd_id` - Data structure definition ID
    ///
    /// # Returns
    /// Data structure metadata
    pub async fn get_datastructure(&self, dsd_id: &str) -> ExchangeResult<SdmxDataStructure> {
        let endpoint = BundesbankEndpoint::DataStructure {
            id: dsd_id.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        BundesbankParser::parse_datastructure(&response)
    }

    /// Get codelist (valid dimension values)
    ///
    /// Returns the list of valid values for a specific dimension.
    ///
    /// # Arguments
    /// - `codelist_id` - Codelist ID
    ///
    /// # Returns
    /// Codelist with all valid codes
    pub async fn get_codelist(&self, codelist_id: &str) -> ExchangeResult<SdmxCodelist> {
        let endpoint = BundesbankEndpoint::Codelist {
            id: codelist_id.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        BundesbankParser::parse_codelist(&response)
    }

    /// Get concept scheme
    ///
    /// Returns metadata concepts used in the data structures.
    ///
    /// # Arguments
    /// - `id` - Concept scheme ID
    ///
    /// # Returns
    /// Concept scheme metadata
    pub async fn get_concept_scheme(&self, id: &str) -> ExchangeResult<SdmxConceptScheme> {
        let endpoint = BundesbankEndpoint::ConceptScheme {
            id: id.to_string(),
        };
        let _response = self.get(endpoint, HashMap::new()).await?;

        // Simplified parsing - full implementation would extract concept details
        Ok(SdmxConceptScheme {
            id: id.to_string(),
            name: "Concept Scheme".to_string(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get metadata for a specific dataflow and key
    ///
    /// Returns detailed metadata about the time series.
    ///
    /// # Arguments
    /// - `dataflow` - Dataflow ID
    /// - `key` - SDMX key
    ///
    /// # Returns
    /// Raw JSON metadata (parsing depends on specific metadata structure)
    pub async fn get_metadata(
        &self,
        dataflow: &str,
        key: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let endpoint = BundesbankEndpoint::Metadata {
            dataflow: dataflow.to_string(),
            key: key.to_string(),
        };
        self.get(endpoint, HashMap::new()).await
    }
}

impl Default for BundesbankConnector {
    fn default() -> Self {
        Self::new()
    }
}
