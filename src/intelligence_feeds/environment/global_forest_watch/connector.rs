//! GFW connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    GfwParser, GfwDataset, GfwTreeCoverLoss, GfwTreeCoverGain,
    GfwForestStats, GfwAlert, GfwCountryStats,
};

/// Global Forest Watch (GFW) connector
///
/// Provides access to global forest monitoring data including deforestation,
/// tree cover loss/gain, fire alerts, and forest statistics.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::global_forest_watch::GfwConnector;
///
/// let connector = GfwConnector::from_env();
///
/// // Get tree cover loss for a country
/// let loss = connector.get_tree_cover_loss("BRA", 2015, 2023).await?;
///
/// // Get fire alerts for the last 7 days
/// let fires = connector.get_fire_alerts("IDN", 7).await?;
///
/// // Get forest statistics
/// let stats = connector.get_forest_statistics("COD").await?;
/// ```
pub struct GfwConnector {
    client: Client,
    auth: GfwAuth,
    endpoints: GfwEndpoints,
    _testnet: bool,
}

impl GfwConnector {
    /// Create new GFW connector with authentication
    pub fn new(auth: GfwAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: GfwEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `GFW_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(GfwAuth::from_env())
    }

    /// Internal: Make GET request to GFW API
    async fn get(
        &self,
        endpoint: GfwEndpoint,
        path_params: Option<(&str, &str, &str)>,
        query_params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Build path with parameters
        let path = if let Some((id, dataset_id, version)) = path_params {
            endpoint.with_params(Some(id), Some(dataset_id), Some(version))
        } else {
            endpoint.path()
        };

        let url = format!("{}{}", self.endpoints.rest_base, path);

        // Add authentication headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query parameters
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        let response = request
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

        // Check for GFW API errors
        GfwParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PUBLIC API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of all datasets
    pub async fn get_datasets(&self) -> ExchangeResult<Vec<GfwDataset>> {
        let response = self.get(GfwEndpoint::Datasets, None, HashMap::new()).await?;
        GfwParser::parse_datasets(&response)
    }

    /// Get specific dataset by ID
    pub async fn get_dataset(&self, id: &str) -> ExchangeResult<GfwDataset> {
        let response = self.get(
            GfwEndpoint::Dataset,
            Some((id, "", "")),
            HashMap::new(),
        ).await?;
        GfwParser::parse_dataset(&response)
    }

    /// Query a dataset with SQL
    ///
    /// # Arguments
    /// - `dataset_id` - Dataset identifier
    /// - `version` - Dataset version
    /// - `sql` - SQL query string
    pub async fn query_dataset(
        &self,
        dataset_id: &str,
        version: &str,
        sql: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("sql".to_string(), sql.to_string());

        self.get(
            GfwEndpoint::DatasetQuery,
            Some(("", dataset_id, version)),
            params,
        ).await
    }

    /// Get tree cover loss by country and year range
    ///
    /// # Arguments
    /// - `iso_code` - ISO country code (e.g., "BRA", "IDN", "COD")
    /// - `start_year` - Start year (e.g., 2001)
    /// - `end_year` - End year (e.g., 2023)
    pub async fn get_tree_cover_loss(
        &self,
        iso_code: &str,
        start_year: u32,
        end_year: u32,
    ) -> ExchangeResult<Vec<GfwTreeCoverLoss>> {
        let mut params = HashMap::new();
        params.insert("iso".to_string(), iso_code.to_string());
        params.insert("start_year".to_string(), start_year.to_string());
        params.insert("end_year".to_string(), end_year.to_string());

        let response = self.get(GfwEndpoint::TreeCoverLoss, None, params).await?;
        GfwParser::parse_tree_cover_loss(&response)
    }

    /// Get tree cover gain by country
    ///
    /// # Arguments
    /// - `iso_code` - ISO country code (e.g., "BRA", "IDN", "COD")
    pub async fn get_tree_cover_gain(
        &self,
        iso_code: &str,
    ) -> ExchangeResult<GfwTreeCoverGain> {
        let mut params = HashMap::new();
        params.insert("iso".to_string(), iso_code.to_string());

        let response = self.get(GfwEndpoint::TreeCoverGain, None, params).await?;
        GfwParser::parse_tree_cover_gain(&response)
    }

    /// Get forest statistics by country
    ///
    /// # Arguments
    /// - `iso_code` - ISO country code (e.g., "BRA", "IDN", "COD")
    pub async fn get_forest_statistics(
        &self,
        iso_code: &str,
    ) -> ExchangeResult<GfwForestStats> {
        let mut params = HashMap::new();
        params.insert("iso".to_string(), iso_code.to_string());

        let response = self.get(GfwEndpoint::ForestChangeStatistics, None, params).await?;
        GfwParser::parse_forest_stats(&response)
    }

    /// Get fire alerts for a country
    ///
    /// # Arguments
    /// - `iso_code` - ISO country code (e.g., "BRA", "IDN", "COD")
    /// - `days` - Number of days to look back (e.g., 7, 30)
    pub async fn get_fire_alerts(
        &self,
        iso_code: &str,
        days: u32,
    ) -> ExchangeResult<Vec<GfwAlert>> {
        let mut params = HashMap::new();
        params.insert("iso".to_string(), iso_code.to_string());
        params.insert("days".to_string(), days.to_string());

        let response = self.get(GfwEndpoint::FireAlerts, None, params).await?;
        GfwParser::parse_alerts(&response)
    }

    /// Get deforestation alerts for a country
    ///
    /// # Arguments
    /// - `iso_code` - ISO country code (e.g., "BRA", "IDN", "COD")
    /// - `days` - Number of days to look back (e.g., 7, 30)
    pub async fn get_deforestation_alerts(
        &self,
        iso_code: &str,
        days: u32,
    ) -> ExchangeResult<Vec<GfwAlert>> {
        let mut params = HashMap::new();
        params.insert("iso".to_string(), iso_code.to_string());
        params.insert("days".to_string(), days.to_string());

        let response = self.get(GfwEndpoint::DeforestationAlerts, None, params).await?;
        GfwParser::parse_alerts(&response)
    }

    /// Get global deforestation totals by year range
    ///
    /// # Arguments
    /// - `start_year` - Start year (e.g., 2001)
    /// - `end_year` - End year (e.g., 2023)
    pub async fn get_global_deforestation(
        &self,
        start_year: u32,
        end_year: u32,
    ) -> ExchangeResult<Vec<GfwTreeCoverLoss>> {
        let mut params = HashMap::new();
        params.insert("start_year".to_string(), start_year.to_string());
        params.insert("end_year".to_string(), end_year.to_string());

        let response = self.get(GfwEndpoint::TreeCoverLoss, None, params).await?;
        GfwParser::parse_tree_cover_loss(&response)
    }

    /// Get top countries by deforestation for a specific year
    ///
    /// # Arguments
    /// - `year` - Year to query (e.g., 2023)
    /// - `limit` - Number of top countries to return (e.g., 10)
    pub async fn get_top_deforestation_countries(
        &self,
        year: u32,
        limit: u32,
    ) -> ExchangeResult<Vec<GfwCountryStats>> {
        let mut params = HashMap::new();
        params.insert("year".to_string(), year.to_string());
        params.insert("limit".to_string(), limit.to_string());

        let response = self.get(GfwEndpoint::TreeCoverLoss, None, params).await?;
        GfwParser::parse_country_stats(&response)
    }
}
