//! Eurostat connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Eurostat (European Statistical Office) connector
///
/// Provides access to European economic and social statistics.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::eurostat::EurostatConnector;
///
/// let connector = EurostatConnector::new();
///
/// // Get GDP data for Germany
/// let mut filters = HashMap::new();
/// filters.insert("geo".to_string(), "DE".to_string());
/// filters.insert("time".to_string(), "2024".to_string());
/// let dataset = connector.get_dataset("nama_10_gdp", Some(filters)).await?;
///
/// // Get table of contents
/// let toc = connector.get_toc().await?;
/// ```
pub struct EurostatConnector {
    client: Client,
    auth: EurostatAuth,
    endpoints: EurostatEndpoints,
}

impl EurostatConnector {
    /// Create new Eurostat connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: EurostatAuth::new(),
            endpoints: EurostatEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for Eurostat, no auth needed)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to Eurostat API
    async fn get(
        &self,
        endpoint: EurostatEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for Eurostat)
        self.auth.sign_query(&mut params);

        // Always request JSON format
        params.insert("format".to_string(), "JSON".to_string());
        params.insert("lang".to_string(), "EN".to_string());

        let (base, path) = endpoint.path_and_base();

        let base_url = match base {
            EndpointBase::Statistics => self.endpoints.statistics_base,
            EndpointBase::Sdmx => self.endpoints.sdmx_base,
            EndpointBase::Catalogue => self.endpoints.catalogue_base,
        };

        let url = format!("{}{}", base_url, path);

        let response = self
            .client
            .get(&url)
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

        // Check for Eurostat API errors
        EurostatParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EUROSTAT-SPECIFIC METHODS (Statistics API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get dataset observations
    ///
    /// # Arguments
    /// - `dataset_code` - Dataset code (e.g., "nama_10_gdp", "prc_hicp_midx")
    /// - `filters` - Optional filters as key=value pairs (e.g., geo=DE, time=2024)
    ///
    /// # Returns
    /// Dataset in JSON-stat v2 format
    ///
    /// # Example filters
    /// ```ignore
    /// let mut filters = HashMap::new();
    /// filters.insert("geo".to_string(), "DE".to_string());  // Germany
    /// filters.insert("time".to_string(), "2024".to_string()); // Year 2024
    /// ```
    pub async fn get_dataset(
        &self,
        dataset_code: &str,
        filters: Option<HashMap<String, String>>,
    ) -> ExchangeResult<EurostatDataset> {
        let params = filters.unwrap_or_default();

        let endpoint = EurostatEndpoint::Data {
            dataset_code: dataset_code.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        EurostatParser::parse_dataset(&response)
    }

    /// Get dataset label (metadata)
    ///
    /// # Arguments
    /// - `dataset_code` - Dataset code
    ///
    /// # Returns
    /// Dataset label/title
    pub async fn get_dataset_label(&self, dataset_code: &str) -> ExchangeResult<EurostatLabel> {
        let params = HashMap::new();

        let endpoint = EurostatEndpoint::Label {
            dataset_code: dataset_code.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        EurostatParser::parse_label(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SDMX API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// List all dataflows
    ///
    /// Returns list of available dataflows in SDMX format
    pub async fn list_dataflows(&self) -> ExchangeResult<Vec<EurostatDataflow>> {
        let params = HashMap::new();
        let response = self.get(EurostatEndpoint::ListDataflows, params).await?;
        EurostatParser::parse_dataflows(&response)
    }

    /// Get specific dataflow metadata
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID
    pub async fn get_dataflow(&self, dataflow_id: &str) -> ExchangeResult<Vec<EurostatDataflow>> {
        let params = HashMap::new();
        let endpoint = EurostatEndpoint::Dataflow {
            dataflow_id: dataflow_id.to_string(),
        };
        let response = self.get(endpoint, params).await?;
        EurostatParser::parse_dataflows(&response)
    }

    /// Get data via SDMX API
    ///
    /// # Arguments
    /// - `dataflow_id` - Dataflow ID
    /// - `key` - SDMX key (e.g., "A.DE.EUR")
    pub async fn get_data_sdmx(
        &self,
        dataflow_id: &str,
        key: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let endpoint = EurostatEndpoint::DataSdmx {
            dataflow_id: dataflow_id.to_string(),
            key: key.to_string(),
        };
        self.get(endpoint, params).await
    }

    /// Get datastructure definition
    ///
    /// # Arguments
    /// - `dsd_id` - Data Structure Definition ID
    pub async fn get_datastructure(&self, dsd_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let endpoint = EurostatEndpoint::Datastructure {
            dsd_id: dsd_id.to_string(),
        };
        self.get(endpoint, params).await
    }

    /// Get codelist
    ///
    /// # Arguments
    /// - `codelist_id` - Codelist ID
    pub async fn get_codelist(&self, codelist_id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let endpoint = EurostatEndpoint::Codelist {
            codelist_id: codelist_id.to_string(),
        };
        self.get(endpoint, params).await
    }

    /// Get concept scheme
    ///
    /// # Arguments
    /// - `id` - Concept scheme ID
    pub async fn get_concept_scheme(&self, id: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        let endpoint = EurostatEndpoint::ConceptScheme {
            id: id.to_string(),
        };
        self.get(endpoint, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CATALOGUE API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get table of contents
    ///
    /// Returns hierarchical catalog of available datasets
    pub async fn get_toc(&self) -> ExchangeResult<Vec<EurostatTocEntry>> {
        let params = HashMap::new();
        let response = self.get(EurostatEndpoint::TableOfContents, params).await?;
        EurostatParser::parse_toc(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get GDP data
    ///
    /// Convenience method for nama_10_gdp dataset
    pub async fn get_gdp(
        &self,
        geo: Option<&str>,
        time: Option<&str>,
    ) -> ExchangeResult<EurostatDataset> {
        let mut filters = HashMap::new();
        if let Some(g) = geo {
            filters.insert("geo".to_string(), g.to_string());
        }
        if let Some(t) = time {
            filters.insert("time".to_string(), t.to_string());
        }
        self.get_dataset("nama_10_gdp", Some(filters)).await
    }

    /// Get CPI/HICP data
    ///
    /// Convenience method for prc_hicp_midx dataset
    pub async fn get_hicp(
        &self,
        geo: Option<&str>,
        time: Option<&str>,
    ) -> ExchangeResult<EurostatDataset> {
        let mut filters = HashMap::new();
        if let Some(g) = geo {
            filters.insert("geo".to_string(), g.to_string());
        }
        if let Some(t) = time {
            filters.insert("time".to_string(), t.to_string());
        }
        self.get_dataset("prc_hicp_midx", Some(filters)).await
    }

    /// Get unemployment data
    ///
    /// Convenience method for une_rt_m dataset
    pub async fn get_unemployment(
        &self,
        geo: Option<&str>,
        time: Option<&str>,
    ) -> ExchangeResult<EurostatDataset> {
        let mut filters = HashMap::new();
        if let Some(g) = geo {
            filters.insert("geo".to_string(), g.to_string());
        }
        if let Some(t) = time {
            filters.insert("time".to_string(), t.to_string());
        }
        self.get_dataset("une_rt_m", Some(filters)).await
    }
}

impl Default for EurostatConnector {
    fn default() -> Self {
        Self::new()
    }
}
