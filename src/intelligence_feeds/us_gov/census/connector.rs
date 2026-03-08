//! US Census Bureau connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{CensusParser, CensusDataRow, EconomicIndicatorObservation, DatasetInfo};

/// US Census Bureau API connector
///
/// Provides access to economic indicators, demographic data, and other census datasets.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::census::CensusConnector;
///
/// let connector = CensusConnector::from_env();
///
/// // Get retail sales data
/// let retail = connector.get_retail_sales(None).await?;
///
/// // Get housing starts
/// let housing = connector.get_housing_starts(None).await?;
///
/// // Get population data
/// let pop = connector.get_population("2021", "01").await?;
///
/// // Generic dataset query
/// let data = connector.get_data("acs/acs1", "2021", &["B01001_001E"], "state:*", None).await?;
/// ```
pub struct CensusConnector {
    client: Client,
    auth: CensusAuth,
    endpoints: CensusEndpoints,
    _testnet: bool,
}

impl CensusConnector {
    /// Create new Census connector with authentication
    pub fn new(auth: CensusAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: CensusEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `CENSUS_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(CensusAuth::from_env())
    }

    /// Internal: Make GET request to Census API
    async fn get(
        &self,
        endpoint: CensusEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for Census API errors
        CensusParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CENSUS-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get data from any Census dataset (CORE METHOD)
    ///
    /// # Arguments
    /// - `dataset` - Dataset path (e.g., "acs/acs1", "timeseries/eits/advm")
    /// - `year` - Year or vintage (e.g., "2021")
    /// - `variables` - List of variable codes to retrieve (e.g., &["B01001_001E"])
    /// - `geography` - Geography specification (e.g., "state:*", "state:01")
    /// - `predicates` - Optional additional filters (e.g., Some(&[("in", "state:01")]))
    ///
    /// # Example
    /// ```ignore
    /// // Get population for all states in 2021
    /// let data = connector.get_data(
    ///     "acs/acs1",
    ///     "2021",
    ///     &["B01001_001E"],
    ///     "state:*",
    ///     None
    /// ).await?;
    /// ```
    pub async fn get_data(
        &self,
        dataset: &str,
        year: &str,
        variables: &[&str],
        geography: &str,
        predicates: Option<&[(&str, &str)]>,
    ) -> ExchangeResult<Vec<CensusDataRow>> {
        let mut params = HashMap::new();

        // Add variables to get
        params.insert("get".to_string(), variables.join(","));

        // Add geography
        params.insert("for".to_string(), geography.to_string());

        // Add predicates (additional filters)
        if let Some(preds) = predicates {
            for (key, value) in preds {
                params.insert(key.to_string(), value.to_string());
            }
        }

        let endpoint = CensusEndpoint::Dataset {
            year: year.to_string(),
            dataset: dataset.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        CensusParser::parse_dataset(&response)
    }

    /// Get economic indicator time series data
    ///
    /// # Arguments
    /// - `indicator` - Indicator ID (e.g., "advm", "resconst", "ftd")
    /// - `time_slot` - Optional time slot filter (e.g., "2024-01", "2024-Q1")
    ///
    /// # Example
    /// ```ignore
    /// // Get all retail sales data
    /// let data = connector.get_economic_indicators("advm", None).await?;
    ///
    /// // Get retail sales for January 2024
    /// let jan_data = connector.get_economic_indicators("advm", Some("2024-01")).await?;
    /// ```
    pub async fn get_economic_indicators(
        &self,
        indicator: &str,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        let mut params = HashMap::new();

        // Get standard fields
        params.insert(
            "get".to_string(),
            "cell_value,data_type_code,time_slot_id,error_data".to_string(),
        );

        // Category code (usually TOTAL for aggregated data)
        params.insert("category_code".to_string(), "TOTAL".to_string());

        // Geography (usually us:* for national data)
        params.insert("for".to_string(), "us:*".to_string());

        // Optional time slot filter
        if let Some(slot) = time_slot {
            params.insert("time_slot_id".to_string(), slot.to_string());
        }

        let endpoint = CensusEndpoint::EconomicIndicator {
            indicator: indicator.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        CensusParser::parse_economic_indicator(&response)
    }

    /// Get Advance Monthly Retail Sales
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01" for January 2024)
    pub async fn get_retail_sales(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::RETAIL_SALES, time_slot)
            .await
    }

    /// Get New Residential Construction (Housing Starts)
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01")
    pub async fn get_housing_starts(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::HOUSING_STARTS, time_slot)
            .await
    }

    /// Get New Residential Sales (New Home Sales)
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01")
    pub async fn get_new_home_sales(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::NEW_HOME_SALES, time_slot)
            .await
    }

    /// Get U.S. International Trade in Goods and Services
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01")
    pub async fn get_trade_data(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::FOREIGN_TRADE, time_slot)
            .await
    }

    /// Get Manufacturers' Shipments, Inventories, and Orders
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01")
    pub async fn get_manufacturers_shipments(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::MANUFACTURERS_SHIPMENTS, time_slot)
            .await
    }

    /// Get Value of Construction Put in Place
    ///
    /// # Arguments
    /// - `time_slot` - Optional time slot (e.g., "2024-01")
    pub async fn get_construction_spending(
        &self,
        time_slot: Option<&str>,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        self.get_economic_indicators(indicators::CONSTRUCTION_SPENDING, time_slot)
            .await
    }

    /// Get population data from American Community Survey
    ///
    /// # Arguments
    /// - `year` - Year (e.g., "2021")
    /// - `state` - State FIPS code (e.g., "01" for Alabama, "*" for all states)
    ///
    /// # Example
    /// ```ignore
    /// // Get population for Alabama
    /// let pop = connector.get_population("2021", "01").await?;
    ///
    /// // Get population for all states
    /// let all_pop = connector.get_population("2021", "*").await?;
    /// ```
    pub async fn get_population(
        &self,
        year: &str,
        state: &str,
    ) -> ExchangeResult<Vec<CensusDataRow>> {
        self.get_data(
            datasets::ACS1,
            year,
            &["NAME", "B01001_001E"], // NAME and total population
            &format_geography("state", state),
            None,
        )
        .await
    }

    /// Get American Community Survey (ACS) data
    ///
    /// # Arguments
    /// - `year` - Year (e.g., "2021")
    /// - `variables` - List of ACS variable codes (e.g., &["B01001_001E", "B19013_001E"])
    /// - `geography` - Geography specification (e.g., "state:*", "county:*")
    ///
    /// # Example
    /// ```ignore
    /// // Get population and median household income for all states
    /// let data = connector.get_acs_data(
    ///     "2021",
    ///     &["NAME", "B01001_001E", "B19013_001E"],
    ///     "state:*"
    /// ).await?;
    /// ```
    pub async fn get_acs_data(
        &self,
        year: &str,
        variables: &[&str],
        geography: &str,
    ) -> ExchangeResult<Vec<CensusDataRow>> {
        self.get_data(datasets::ACS1, year, variables, geography, None)
            .await
    }

    /// List all available datasets
    ///
    /// # Arguments
    /// - `year` - Optional year to list datasets for
    ///
    /// # Example
    /// ```ignore
    /// // List all datasets
    /// let datasets = connector.list_datasets(None).await?;
    ///
    /// // List datasets for 2021
    /// let datasets_2021 = connector.list_datasets(Some("2021")).await?;
    /// ```
    pub async fn list_datasets(
        &self,
        year: Option<&str>,
    ) -> ExchangeResult<Vec<DatasetInfo>> {
        let endpoint = if let Some(y) = year {
            CensusEndpoint::ListDatasets { year: y.to_string() }
        } else {
            CensusEndpoint::ListDatasetsAll
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        CensusParser::parse_dataset_list(&response)
    }
}
