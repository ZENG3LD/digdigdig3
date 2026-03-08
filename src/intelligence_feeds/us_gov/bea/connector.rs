//! BEA connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{BeaParser, BeaDataPoint, BeaDataset, BeaParameter, BeaParameterValue};

/// BEA (Bureau of Economic Analysis) connector
///
/// Provides access to U.S. economic data including GDP, national income,
/// international transactions, and regional economic statistics.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::bea::BeaConnector;
///
/// let connector = BeaConnector::from_env();
///
/// // Get list of available datasets
/// let datasets = connector.get_dataset_list().await?;
///
/// // Get GDP data
/// let gdp_data = connector.get_gdp(2024, Some(3)).await?;
///
/// // Get generic NIPA table
/// let data = connector.get_nipa_table("T10101", "Q", 2024).await?;
/// ```
pub struct BeaConnector {
    client: Client,
    auth: BeaAuth,
    endpoints: BeaEndpoints,
    _testnet: bool,
}

impl BeaConnector {
    /// Create new BEA connector with authentication
    pub fn new(auth: BeaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: BeaEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `BEA_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(BeaAuth::from_env())
    }

    /// Internal: Make GET request to BEA API
    async fn get(
        &self,
        endpoint: BeaEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        // Add method parameter
        params.insert("method".to_string(), endpoint.method().to_string());

        // Always request JSON format
        params.insert("ResultFormat".to_string(), "JSON".to_string());

        let url = self.endpoints.rest_base;

        let response = self
            .client
            .get(url)
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

        // Check for BEA API errors
        BeaParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of available datasets
    ///
    /// Returns all available BEA datasets with descriptions.
    ///
    /// # Example
    /// ```ignore
    /// let datasets = connector.get_dataset_list().await?;
    /// for dataset in datasets {
    ///     println!("{}: {}", dataset.name, dataset.description);
    /// }
    /// ```
    pub async fn get_dataset_list(&self) -> ExchangeResult<Vec<BeaDataset>> {
        let params = HashMap::new();
        let response = self.get(BeaEndpoint::GetDatasetList, params).await?;
        BeaParser::parse_dataset_list(&response)
    }

    /// Get parameter list for a dataset
    ///
    /// # Arguments
    /// - `dataset` - Dataset name (e.g., "NIPA", "GDPbyIndustry")
    ///
    /// # Example
    /// ```ignore
    /// let params = connector.get_parameter_list("NIPA").await?;
    /// ```
    pub async fn get_parameter_list(&self, dataset: &str) -> ExchangeResult<Vec<BeaParameter>> {
        let mut params = HashMap::new();
        params.insert("DatasetName".to_string(), dataset.to_string());

        let response = self.get(BeaEndpoint::GetParameterList, params).await?;
        BeaParser::parse_parameter_list(&response)
    }

    /// Get values for a parameter
    ///
    /// # Arguments
    /// - `dataset` - Dataset name
    /// - `param` - Parameter name
    ///
    /// # Example
    /// ```ignore
    /// let values = connector.get_parameter_values("NIPA", "TableName").await?;
    /// ```
    pub async fn get_parameter_values(
        &self,
        dataset: &str,
        param: &str,
    ) -> ExchangeResult<Vec<BeaParameterValue>> {
        let mut params = HashMap::new();
        params.insert("DatasetName".to_string(), dataset.to_string());
        params.insert("ParameterName".to_string(), param.to_string());

        let response = self.get(BeaEndpoint::GetParameterValues, params).await?;
        BeaParser::parse_parameter_values(&response)
    }

    /// Get filtered parameter values
    ///
    /// # Arguments
    /// - `dataset` - Dataset name
    /// - `target_param` - Parameter to get values for
    /// - `filter_param` - Parameter to filter by
    /// - `filter_value` - Value to filter on
    ///
    /// # Example
    /// ```ignore
    /// let values = connector.get_parameter_values_filtered(
    ///     "Regional",
    ///     "GeoFips",
    ///     "TableName",
    ///     "CAINC1"
    /// ).await?;
    /// ```
    pub async fn get_parameter_values_filtered(
        &self,
        dataset: &str,
        target_param: &str,
        filter_param: &str,
        filter_value: &str,
    ) -> ExchangeResult<Vec<BeaParameterValue>> {
        let mut params = HashMap::new();
        params.insert("DatasetName".to_string(), dataset.to_string());
        params.insert("TargetParameter".to_string(), target_param.to_string());
        params.insert(filter_param.to_string(), filter_value.to_string());

        let response = self.get(BeaEndpoint::GetParameterValuesFiltered, params).await?;
        BeaParser::parse_parameter_values(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA RETRIEVAL METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data from a dataset
    ///
    /// Generic method to retrieve data with custom parameters.
    ///
    /// # Arguments
    /// - `dataset` - Dataset name
    /// - `params` - Dataset-specific parameters
    ///
    /// # Example
    /// ```ignore
    /// let mut params = HashMap::new();
    /// params.insert("TableName".to_string(), "T10101".to_string());
    /// params.insert("Frequency".to_string(), "Q".to_string());
    /// params.insert("Year".to_string(), "2024".to_string());
    /// let data = connector.get_data("NIPA", params).await?;
    /// ```
    pub async fn get_data(
        &self,
        dataset: &str,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<Vec<BeaDataPoint>> {
        params.insert("DatasetName".to_string(), dataset.to_string());

        let response = self.get(BeaEndpoint::GetData, params).await?;
        BeaParser::parse_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS (High-level)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get GDP data from NIPA Table 1.1.1
    ///
    /// # Arguments
    /// - `year` - Year (e.g., 2024)
    /// - `quarter` - Optional quarter (1-4). If None, gets annual data.
    ///
    /// # Example
    /// ```ignore
    /// // Get Q3 2024 GDP
    /// let gdp = connector.get_gdp(2024, Some(3)).await?;
    ///
    /// // Get annual 2024 GDP
    /// let gdp_annual = connector.get_gdp(2024, None).await?;
    /// ```
    pub async fn get_gdp(&self, year: u32, quarter: Option<u8>) -> ExchangeResult<Vec<BeaDataPoint>> {
        let frequency = if quarter.is_some() { "Q" } else { "A" };
        self.get_nipa_table("T10101", frequency, year).await
    }

    /// Get GDP by industry data
    ///
    /// # Arguments
    /// - `year` - Year
    /// - `quarter` - Optional quarter (1-4). If None, gets annual data.
    ///
    /// # Example
    /// ```ignore
    /// let data = connector.get_gdp_by_industry(2024, Some(3)).await?;
    /// ```
    pub async fn get_gdp_by_industry(
        &self,
        year: u32,
        quarter: Option<u8>,
    ) -> ExchangeResult<Vec<BeaDataPoint>> {
        let mut params = HashMap::new();
        params.insert("Year".to_string(), year.to_string());

        let frequency = if quarter.is_some() { "Q" } else { "A" };
        params.insert("Frequency".to_string(), frequency.to_string());

        params.insert("TableID".to_string(), "ALL".to_string());

        self.get_data("GDPbyIndustry", params).await
    }

    /// Get personal income data from Regional dataset
    ///
    /// # Arguments
    /// - `year` - Year
    ///
    /// # Example
    /// ```ignore
    /// let income = connector.get_personal_income(2024).await?;
    /// ```
    pub async fn get_personal_income(&self, year: u32) -> ExchangeResult<Vec<BeaDataPoint>> {
        let mut params = HashMap::new();
        params.insert("TableName".to_string(), "CAINC1".to_string());
        params.insert("LineCode".to_string(), "1".to_string());
        params.insert("Year".to_string(), year.to_string());
        params.insert("GeoFips".to_string(), "STATE".to_string());

        self.get_data("Regional", params).await
    }

    /// Get international transactions data
    ///
    /// # Arguments
    /// - `year` - Year
    ///
    /// # Example
    /// ```ignore
    /// let transactions = connector.get_international_transactions(2024).await?;
    /// ```
    pub async fn get_international_transactions(&self, year: u32) -> ExchangeResult<Vec<BeaDataPoint>> {
        let mut params = HashMap::new();
        params.insert("Year".to_string(), year.to_string());
        params.insert("Frequency".to_string(), "A".to_string());
        params.insert("Indicator".to_string(), "BalGds".to_string());

        self.get_data("ITA", params).await
    }

    /// Get fixed assets data
    ///
    /// # Arguments
    /// - `year` - Year
    ///
    /// # Example
    /// ```ignore
    /// let assets = connector.get_fixed_assets(2024).await?;
    /// ```
    pub async fn get_fixed_assets(&self, year: u32) -> ExchangeResult<Vec<BeaDataPoint>> {
        let mut params = HashMap::new();
        params.insert("Year".to_string(), year.to_string());
        params.insert("TableName".to_string(), "FAAt201".to_string());

        self.get_data("FixedAssets", params).await
    }

    /// Get generic NIPA table data
    ///
    /// # Arguments
    /// - `table_name` - Table name (e.g., "T10101", "T10106")
    /// - `frequency` - Frequency: "A" (annual), "Q" (quarterly), "M" (monthly)
    /// - `year` - Year
    ///
    /// # Example
    /// ```ignore
    /// // Get quarterly GDP price index for 2024
    /// let data = connector.get_nipa_table("T10106", "Q", 2024).await?;
    /// ```
    pub async fn get_nipa_table(
        &self,
        table_name: &str,
        frequency: &str,
        year: u32,
    ) -> ExchangeResult<Vec<BeaDataPoint>> {
        let mut params = HashMap::new();
        params.insert("TableName".to_string(), table_name.to_string());
        params.insert("Frequency".to_string(), frequency.to_string());
        params.insert("Year".to_string(), year.to_string());

        self.get_data("NIPA", params).await
    }
}
