//! WHO GHO connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{WhoParser, WhoIndicator, WhoDataPoint, WhoCountry, WhoRegion};

/// WHO GHO (Global Health Observatory) connector
///
/// Provides access to 1000+ health indicators from the World Health Organization.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::who::WhoConnector;
///
/// let connector = WhoConnector::new();
///
/// // Get all indicators
/// let indicators = connector.get_indicators().await?;
///
/// // Get life expectancy for USA in 2020
/// let data = connector.get_life_expectancy("USA", Some(2020)).await?;
///
/// // Get all countries
/// let countries = connector.get_countries().await?;
/// ```
pub struct WhoConnector {
    client: Client,
    auth: WhoAuth,
    endpoints: WhoEndpoints,
    _testnet: bool,
}

impl WhoConnector {
    /// Create new WHO GHO connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: WhoAuth::new(),
            endpoints: WhoEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment (no-op for WHO, no auth needed)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to WHO GHO API
    async fn get(
        &self,
        endpoint: WhoEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for WHO)
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
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for WHO API errors
        WhoParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WHO-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all indicators
    ///
    /// Returns a list of all available health indicators.
    pub async fn get_indicators(&self) -> ExchangeResult<Vec<WhoIndicator>> {
        let params = HashMap::new();
        let response = self.get(WhoEndpoint::Indicators, params).await?;
        WhoParser::parse_indicators(&response)
    }

    /// Get indicator data
    ///
    /// # Arguments
    /// - `indicator_code` - Indicator code (e.g., "WHOSIS_000001")
    /// - `country` - Optional country code (e.g., "USA")
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Vector of data points for the indicator
    pub async fn get_indicator_data(
        &self,
        indicator_code: &str,
        country: Option<&str>,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        let mut params = HashMap::new();

        // Build OData filter
        let mut filters = Vec::new();
        if let Some(c) = country {
            filters.push(format!("SpatialDim eq '{}'", c));
        }
        if let Some(y) = year {
            filters.push(format!("TimeDim eq {}", y));
        }

        if !filters.is_empty() {
            params.insert("$filter".to_string(), filters.join(" and "));
        }

        let response = self
            .get(WhoEndpoint::IndicatorData(indicator_code.to_string()), params)
            .await?;
        WhoParser::parse_data_points(&response)
    }

    /// Get list of countries
    pub async fn get_countries(&self) -> ExchangeResult<Vec<WhoCountry>> {
        let params = HashMap::new();
        let response = self.get(WhoEndpoint::Countries, params).await?;
        WhoParser::parse_countries(&response)
    }

    /// Get list of regions
    pub async fn get_regions(&self) -> ExchangeResult<Vec<WhoRegion>> {
        let params = HashMap::new();
        let response = self.get(WhoEndpoint::Regions, params).await?;
        WhoParser::parse_regions(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS FOR COMMON INDICATORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get life expectancy at birth (WHOSIS_000001)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_life_expectancy(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("WHOSIS_000001", Some(country), year)
            .await
    }

    /// Get infant mortality rate (MDG_0000000001)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_infant_mortality(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("MDG_0000000001", Some(country), year)
            .await
    }

    /// Get obesity rate (NCD_BMI_30A)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_obesity_rate(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("NCD_BMI_30A", Some(country), year)
            .await
    }

    /// Get tobacco use (TOBACCO_0000000192)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_tobacco_use(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("TOBACCO_0000000192", Some(country), year)
            .await
    }

    /// Get air pollution deaths (AIR_7)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_air_pollution_deaths(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("AIR_7", Some(country), year)
            .await
    }

    /// Get health expenditure as % of GDP (GHED_CHEGDP_SHA2011)
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "USA")
    /// - `year` - Optional year filter
    pub async fn get_health_expenditure(
        &self,
        country: &str,
        year: Option<i64>,
    ) -> ExchangeResult<Vec<WhoDataPoint>> {
        self.get_indicator_data("GHED_CHEGDP_SHA2011", Some(country), year)
            .await
    }
}

impl Default for WhoConnector {
    fn default() -> Self {
        Self::new()
    }
}
