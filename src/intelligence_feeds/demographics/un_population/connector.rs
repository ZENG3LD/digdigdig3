//! UN Population connector implementation

use reqwest::Client;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::{UnPopEndpoint, UnPopEndpoints, format_params};
use super::auth::UnPopAuth;
use super::parser::{UnPopParser, UnPopLocation, UnPopIndicator, UnPopDataPoint};

/// UN Population connector
///
/// Provides access to demographic data from the United Nations Population Division.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::un_population::UnPopConnector;
///
/// let connector = UnPopConnector::new(false);
///
/// // Get all locations
/// let locations = connector.get_locations().await?;
///
/// // Get world population data
/// let world_pop = connector.get_world_population(Some(2000), Some(2023)).await?;
///
/// // Get fertility rate for USA
/// let fertility = connector.get_fertility_rate(840, Some(2010), Some(2020)).await?;
/// ```
pub struct UnPopConnector {
    client: Client,
    _auth: UnPopAuth,
    endpoints: UnPopEndpoints,
    _testnet: bool,
}

impl UnPopConnector {
    /// Create new UN Population connector
    pub fn new(testnet: bool) -> Self {
        Self {
            client: Client::new(),
            _auth: UnPopAuth::new(),
            endpoints: UnPopEndpoints::new(testnet),
            _testnet: testnet,
        }
    }

    /// Internal: Make GET request to UN Population API
    async fn get(
        &self,
        endpoint: UnPopEndpoint,
        params: Vec<(String, String)>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = self.endpoints.url(&endpoint);

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

        // Check for API errors
        UnPopParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MAIN API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all locations (countries and regions)
    ///
    /// # Returns
    /// Vector of locations with id, name, and ISO codes
    pub async fn get_locations(&self) -> ExchangeResult<Vec<UnPopLocation>> {
        let response = self.get(UnPopEndpoint::Locations, vec![]).await?;
        UnPopParser::parse_locations(&response)
    }

    /// Get all available indicators
    ///
    /// # Returns
    /// Vector of indicators with id, name, and description
    pub async fn get_indicators(&self) -> ExchangeResult<Vec<UnPopIndicator>> {
        let response = self.get(UnPopEndpoint::Indicators, vec![]).await?;
        UnPopParser::parse_indicators(&response)
    }

    /// Get indicator data for a specific location
    ///
    /// # Arguments
    /// - `location_id` - Location ID (e.g., 900 for World, 840 for USA)
    /// - `indicator_id` - Indicator ID (e.g., 49 for PopTotal, 47 for PopGrowthRate)
    /// - `start_year` - Optional start year
    /// - `end_year` - Optional end year
    ///
    /// # Returns
    /// Vector of data points with year and value
    pub async fn get_indicator_data(
        &self,
        location_id: u32,
        indicator_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        let params = format_params(start_year, end_year, None, None);
        let endpoint = UnPopEndpoint::LocationIndicatorData { location_id, indicator_id };
        let response = self.get(endpoint, params).await?;
        let parsed = UnPopParser::parse_data_points(&response)?;
        Ok(parsed.data)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS (using get_indicator_data internally)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get total population data
    ///
    /// Uses indicator 49 (PopTotal)
    pub async fn get_population(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 49, start_year, end_year).await
    }

    /// Get population growth rate
    ///
    /// Uses indicator 47 (PopGrowthRate)
    pub async fn get_population_growth(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 47, start_year, end_year).await
    }

    /// Get life expectancy at birth
    ///
    /// Uses indicator 68 (LifeExpectancyAtBirth)
    pub async fn get_life_expectancy(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 68, start_year, end_year).await
    }

    /// Get fertility rate
    ///
    /// Uses indicator 19 (FertilityRate)
    pub async fn get_fertility_rate(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 19, start_year, end_year).await
    }

    /// Get infant mortality rate
    ///
    /// Uses indicator 22 (InfantMortalityRate)
    pub async fn get_infant_mortality(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 22, start_year, end_year).await
    }

    /// Get median age
    ///
    /// Uses indicator 67 (MedianAge)
    pub async fn get_median_age(
        &self,
        location_id: u32,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_indicator_data(location_id, 67, start_year, end_year).await
    }

    /// Get world population data
    ///
    /// Convenience method using location ID 900 (World)
    pub async fn get_world_population(
        &self,
        start_year: Option<u32>,
        end_year: Option<u32>,
    ) -> ExchangeResult<Vec<UnPopDataPoint>> {
        self.get_population(900, start_year, end_year).await
    }
}
