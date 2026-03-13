//! UNHCR connector implementation

use reqwest::Client;
use serde_json::Value;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::{UnhcrEndpoint, UnhcrEndpoints, format_params};
use super::auth::UnhcrAuth;
use super::parser::{UnhcrParser, UnhcrPopulationData, UnhcrCountry};

/// UNHCR connector
///
/// Provides access to refugee and displaced population statistics from the UNHCR.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::unhcr::UnhcrConnector;
///
/// let connector = UnhcrConnector::new(false);
///
/// // Get all countries
/// let countries = connector.get_countries().await?;
///
/// // Get population data for 2023
/// let pop_2023 = connector.get_population(Some(2023), None, None, None).await?;
///
/// // Get asylum decisions
/// let decisions = connector.get_asylum_decisions(Some(2023), None).await?;
/// ```
pub struct UnhcrConnector {
    client: Client,
    _auth: UnhcrAuth,
    endpoints: UnhcrEndpoints,
    _testnet: bool,
}

impl UnhcrConnector {
    /// Create new UNHCR connector
    pub fn new(testnet: bool) -> Self {
        Self {
            client: Client::new(),
            _auth: UnhcrAuth::new(),
            endpoints: UnhcrEndpoints::new(testnet),
            _testnet: testnet,
        }
    }

    /// Internal: Make GET request to UNHCR API
    async fn get(
        &self,
        endpoint: UnhcrEndpoint,
        params: Vec<(String, String)>,
    ) -> ExchangeResult<Value> {
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

        let json: Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors
        UnhcrParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MAIN API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get refugee population statistics
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country_origin` - Optional country of origin filter
    /// - `country_asylum` - Optional country of asylum filter
    /// - `limit` - Optional limit (page size)
    ///
    /// # Returns
    /// Vector of population data records
    pub async fn get_population(
        &self,
        year: Option<u32>,
        country_origin: Option<&str>,
        country_asylum: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<UnhcrPopulationData>> {
        let params = format_params(year, country_origin, country_asylum, None, limit);
        let response = self.get(UnhcrEndpoint::Population, params).await?;
        UnhcrParser::parse_population(&response)
    }

    /// Get all countries
    ///
    /// # Returns
    /// Vector of countries with id, name, and ISO3 code
    pub async fn get_countries(&self) -> ExchangeResult<Vec<UnhcrCountry>> {
        let response = self.get(UnhcrEndpoint::Countries, vec![]).await?;
        UnhcrParser::parse_countries(&response)
    }

    /// Get asylum decisions data
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country` - Optional country filter
    ///
    /// # Returns
    /// Vector of asylum decision records as JSON values
    pub async fn get_asylum_decisions(
        &self,
        year: Option<u32>,
        country: Option<&str>,
    ) -> ExchangeResult<Vec<Value>> {
        let params = format_params(year, country, None, None, None);
        let response = self.get(UnhcrEndpoint::AsylumDecisions, params).await?;
        UnhcrParser::parse_json_array(&response)
    }

    /// Get demographic breakdowns
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country_origin` - Optional country of origin filter
    /// - `country_asylum` - Optional country of asylum filter
    ///
    /// # Returns
    /// Vector of demographic records as JSON values
    pub async fn get_demographics(
        &self,
        year: Option<u32>,
        country_origin: Option<&str>,
        country_asylum: Option<&str>,
    ) -> ExchangeResult<Vec<Value>> {
        let params = format_params(year, country_origin, country_asylum, None, None);
        let response = self.get(UnhcrEndpoint::Demographics, params).await?;
        UnhcrParser::parse_json_array(&response)
    }

    /// Get durable solutions data (resettlement, returns)
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country` - Optional country filter
    ///
    /// # Returns
    /// Vector of solution records as JSON values
    pub async fn get_solutions(
        &self,
        year: Option<u32>,
        country: Option<&str>,
    ) -> ExchangeResult<Vec<Value>> {
        let params = format_params(year, country, None, None, None);
        let response = self.get(UnhcrEndpoint::Solutions, params).await?;
        UnhcrParser::parse_json_array(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get asylum application data
    ///
    /// Asylum applications track submissions (not decisions). This is distinct
    /// from `get_asylum_decisions` which tracks case outcomes.
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country_origin` - Optional country of origin filter
    /// - `country_asylum` - Optional country of asylum (where application was filed)
    /// - `limit` - Optional page size
    ///
    /// # Returns
    /// Vector of asylum application records as JSON values
    pub async fn get_asylum_applications(
        &self,
        year: Option<u32>,
        country_origin: Option<&str>,
        country_asylum: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Value>> {
        let params = format_params(year, country_origin, country_asylum, None, limit);
        let response = self.get(UnhcrEndpoint::AsylumApplications, params).await?;
        UnhcrParser::parse_json_array(&response)
    }
}
