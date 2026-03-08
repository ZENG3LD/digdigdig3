//! OECD connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// OECD (Organisation for Economic Co-operation and Development) connector
///
/// Provides access to OECD economic and social statistics via SDMX REST API.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::oecd::OecdConnector;
///
/// let connector = OecdConnector::new();
///
/// // Get GDP data for USA from Quarterly National Accounts
/// let data = connector.get_data("QNA", "USA.GDP.Q.V", None, None).await?;
///
/// // List all dataflows from OECD
/// let dataflows = connector.list_dataflows("OECD").await?;
///
/// // Get specific dataflow metadata
/// let dataflow = connector.get_dataflow("OECD", "QNA").await?;
/// ```
pub struct OecdConnector {
    client: Client,
    auth: OecdAuth,
    endpoints: OecdEndpoints,
}

impl OecdConnector {
    /// Create new OECD connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: OecdAuth::new(),
            endpoints: OecdEndpoints::default(),
        }
    }

    /// Internal: Make GET request to OECD SDMX API
    async fn get(
        &self,
        endpoint: OecdEndpoint,
        mut params: HashMap<String, String>,
        accept_header: &str,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for OECD)
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .header("Accept", accept_header)
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

        // Check for OECD API errors
        OecdParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data with filters
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID (e.g., "QNA", "PRICES_CPI")
    /// - `key` - Dimension key (e.g., "USA.GDP.Q.V" or "all" for all)
    /// - `start_period` - Optional start period (e.g., "2020-Q1")
    /// - `end_period` - Optional end period (e.g., "2023-Q4")
    ///
    /// # Returns
    /// Vector of observations with dimension keys and values
    ///
    /// # Example
    /// ```ignore
    /// // Get USA GDP quarterly data from 2020
    /// let data = connector.get_data("QNA", "USA.GDP.Q.V", Some("2020-Q1"), None).await?;
    /// ```
    pub async fn get_data(
        &self,
        dataflow_id: &str,
        key: &str,
        start_period: Option<&str>,
        end_period: Option<&str>,
    ) -> ExchangeResult<Vec<OecdObservation>> {
        let mut params = HashMap::new();
        params.insert("dimensionAtObservation".to_string(), "AllDimensions".to_string());

        if let Some(start) = start_period {
            params.insert("startPeriod".to_string(), start.to_string());
        }
        if let Some(end) = end_period {
            params.insert("endPeriod".to_string(), end.to_string());
        }

        let endpoint = OecdEndpoint::Data {
            dataflow_id: dataflow_id.to_string(),
            key: key.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.data+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_data(&response)
    }

    /// Get all data for a dataflow
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID (e.g., "QNA")
    ///
    /// # Returns
    /// All available observations for the dataflow
    ///
    /// # Warning
    /// This can return a very large dataset. Use with caution.
    pub async fn get_data_all(&self, dataflow_id: &str) -> ExchangeResult<Vec<OecdObservation>> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::DataAll {
            dataflow_id: dataflow_id.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.data+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get dataflow metadata
    ///
    /// # Arguments
    /// - `agency` - Agency ID (typically "OECD")
    /// - `dataflow_id` - Dataflow ID (e.g., "QNA")
    ///
    /// # Returns
    /// Dataflow metadata including ID, name, and version
    pub async fn get_dataflow(
        &self,
        agency: &str,
        dataflow_id: &str,
    ) -> ExchangeResult<OecdDataflow> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::Dataflow {
            agency: agency.to_string(),
            id: dataflow_id.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_dataflow(&response)
    }

    /// List all dataflows for an agency
    ///
    /// # Arguments
    /// - `agency` - Agency ID (typically "OECD")
    ///
    /// # Returns
    /// List of all available dataflows
    pub async fn list_dataflows(&self, agency: &str) -> ExchangeResult<Vec<OecdDataflow>> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::DataflowList {
            agency: agency.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_dataflows(&response)
    }

    /// Get datastructure definition
    ///
    /// # Arguments
    /// - `agency` - Agency ID (typically "OECD")
    /// - `id` - Datastructure ID
    ///
    /// # Returns
    /// Datastructure definition with dimensions and attributes
    pub async fn get_datastructure(
        &self,
        agency: &str,
        id: &str,
    ) -> ExchangeResult<OecdDatastructure> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::Datastructure {
            agency: agency.to_string(),
            id: id.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_datastructure(&response)
    }

    /// Get codelist
    ///
    /// # Arguments
    /// - `agency` - Agency ID (typically "OECD")
    /// - `id` - Codelist ID
    ///
    /// # Returns
    /// Codelist with all codes and their descriptions
    pub async fn get_codelist(&self, agency: &str, id: &str) -> ExchangeResult<OecdCodelist> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::Codelist {
            agency: agency.to_string(),
            id: id.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_codelist(&response)
    }

    /// Get concept scheme
    ///
    /// # Arguments
    /// - `agency` - Agency ID (typically "OECD")
    /// - `id` - Concept scheme ID
    ///
    /// # Returns
    /// Raw JSON response with concept definitions
    pub async fn get_concept_scheme(
        &self,
        agency: &str,
        id: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::ConceptScheme {
            agency: agency.to_string(),
            id: id.to_string(),
        };

        self.get(
            endpoint,
            params,
            "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
        )
        .await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AVAILABILITY ENDPOINT
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data availability constraints
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID
    /// - `key` - Dimension key (e.g., "USA" or "all")
    ///
    /// # Returns
    /// Availability information with constraints
    pub async fn get_availability(
        &self,
        dataflow_id: &str,
        key: &str,
    ) -> ExchangeResult<OecdAvailability> {
        let params = HashMap::new();

        let endpoint = OecdEndpoint::Availability {
            dataflow_id: dataflow_id.to_string(),
            key: key.to_string(),
        };

        let response = self
            .get(
                endpoint,
                params,
                "application/vnd.sdmx.structure+json;charset=utf-8;version=2",
            )
            .await?;

        OecdParser::parse_availability(&response)
    }
}

impl Default for OecdConnector {
    fn default() -> Self {
        Self::new()
    }
}
