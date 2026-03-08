//! World Bank connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    WorldBankParser, IndicatorObservation, IndicatorMetadata, IndicatorInfo,
    Country, CountryInfo, Topic, TopicInfo, Source, SourceInfo,
    IncomeLevel, LendingType,
};

/// World Bank connector
///
/// Provides access to World Bank economic data covering 200+ countries and territories.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::worldbank::WorldBankConnector;
///
/// let connector = WorldBankConnector::new();
///
/// // Get GDP data for USA from 2020-2023
/// let data = connector.get_indicator_data("US", "NY.GDP.MKTP.CD", Some("2020"), Some("2023")).await?;
///
/// // Search for GDP indicators
/// let indicators = connector.search_indicators("GDP", None, Some(10)).await?;
///
/// // Get indicator metadata
/// let metadata = connector.get_indicator_metadata("NY.GDP.MKTP.CD").await?;
/// ```
pub struct WorldBankConnector {
    client: Client,
    auth: WorldBankAuth,
    endpoints: WorldBankEndpoints,
}

impl WorldBankConnector {
    /// Create new World Bank connector
    ///
    /// No authentication required - World Bank API is completely free
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: WorldBankAuth::new(),
            endpoints: WorldBankEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for World Bank)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to World Bank API
    async fn get(
        &self,
        endpoint: WorldBankEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for World Bank)
        self.auth.sign_query(&mut params);

        // Always request JSON format
        params.insert("format".to_string(), "json".to_string());

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

        // Check for World Bank API errors
        WorldBankParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INDICATOR DATA METHODS (Core)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get indicator data for a country
    ///
    /// This is the CORE endpoint for retrieving economic time series data.
    ///
    /// # Arguments
    /// - `country` - Country code (ISO 3166-1 alpha-2 or alpha-3, or "all")
    /// - `indicator` - Indicator code (e.g., "NY.GDP.MKTP.CD")
    /// - `start_year` - Optional start year (e.g., "2020")
    /// - `end_year` - Optional end year (e.g., "2023")
    ///
    /// # Returns
    /// Vector of observations with date and value
    ///
    /// # Example
    /// ```ignore
    /// // Get GDP for USA from 2020-2023
    /// let data = connector.get_indicator_data("US", "NY.GDP.MKTP.CD", Some("2020"), Some("2023")).await?;
    /// ```
    pub async fn get_indicator_data(
        &self,
        country: &str,
        indicator: &str,
        start_year: Option<&str>,
        end_year: Option<&str>,
    ) -> ExchangeResult<Vec<IndicatorObservation>> {
        let mut params = HashMap::new();

        // Add date range if provided
        if let (Some(start), Some(end)) = (start_year, end_year) {
            params.insert("date".to_string(), format!("{}:{}", start, end));
        } else if let Some(start) = start_year {
            params.insert("date".to_string(), format!("{}:{}", start, "9999"));
        } else if let Some(end) = end_year {
            params.insert("date".to_string(), format!("1900:{}", end));
        }

        // Request large page size to get all data
        params.insert("per_page".to_string(), "1000".to_string());

        let response = self
            .get(
                WorldBankEndpoint::IndicatorData {
                    country: country.to_string(),
                    indicator: indicator.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_indicator_data(&response)
    }

    /// Get multiple indicators for a country at once
    ///
    /// # Arguments
    /// - `country` - Country code
    /// - `indicators` - Semicolon-separated indicator codes (e.g., "NY.GDP.MKTP.CD;FP.CPI.TOTL.ZG")
    /// - `start_year` - Optional start year
    /// - `end_year` - Optional end year
    pub async fn get_multiple_indicators(
        &self,
        country: &str,
        indicators: &str,
        start_year: Option<&str>,
        end_year: Option<&str>,
    ) -> ExchangeResult<Vec<IndicatorObservation>> {
        let mut params = HashMap::new();

        if let (Some(start), Some(end)) = (start_year, end_year) {
            params.insert("date".to_string(), format!("{}:{}", start, end));
        }

        params.insert("per_page".to_string(), "1000".to_string());

        let response = self
            .get(
                WorldBankEndpoint::MultiIndicatorData {
                    country: country.to_string(),
                    indicators: indicators.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_indicator_data(&response)
    }

    /// Get indicator metadata
    ///
    /// Returns detailed information about an indicator including description,
    /// source organization, and topics.
    ///
    /// # Example
    /// ```ignore
    /// let metadata = connector.get_indicator_metadata("NY.GDP.MKTP.CD").await?;
    /// println!("Name: {}", metadata.name);
    /// println!("Description: {:?}", metadata.source_note);
    /// ```
    pub async fn get_indicator_metadata(&self, indicator_id: &str) -> ExchangeResult<IndicatorMetadata> {
        let params = HashMap::new();

        let response = self
            .get(
                WorldBankEndpoint::Indicator {
                    id: indicator_id.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_indicator_metadata(&response)
    }

    /// Search for indicators matching keywords
    ///
    /// # Arguments
    /// - `query` - Search keywords
    /// - `page` - Optional page number (default: 1)
    /// - `per_page` - Optional results per page (default: 50, max: 1000)
    ///
    /// # Returns
    /// Vector of indicator info matching the search
    ///
    /// # Example
    /// ```ignore
    /// let results = connector.search_indicators("GDP", None, Some(10)).await?;
    /// ```
    pub async fn search_indicators(
        &self,
        query: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> ExchangeResult<Vec<IndicatorInfo>> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), "2".to_string()); // World Development Indicators

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }

        // World Bank uses special search parameter
        params.insert("prefix".to_string(), "y".to_string());
        params.insert("qterm".to_string(), query.to_string());

        let response = self.get(WorldBankEndpoint::IndicatorSearch, params).await?;
        WorldBankParser::parse_indicator_list(&response)
    }

    /// List all indicators (paginated)
    ///
    /// # Arguments
    /// - `page` - Page number (default: 1)
    /// - `per_page` - Results per page (default: 50, max: 1000)
    pub async fn list_indicators(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> ExchangeResult<Vec<IndicatorInfo>> {
        let mut params = HashMap::new();

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }

        let response = self.get(WorldBankEndpoint::Indicators, params).await?;
        WorldBankParser::parse_indicator_list(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COUNTRY METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get country metadata
    ///
    /// # Arguments
    /// - `country_code` - ISO 3166-1 alpha-2 or alpha-3 code (e.g., "US", "USA")
    pub async fn get_country(&self, country_code: &str) -> ExchangeResult<Country> {
        let params = HashMap::new();

        let response = self
            .get(
                WorldBankEndpoint::Country {
                    code: country_code.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_country(&response)
    }

    /// List all countries (paginated)
    ///
    /// # Arguments
    /// - `page` - Optional page number (default: 1)
    /// - `per_page` - Optional results per page (default: 50, max: 1000)
    pub async fn list_countries(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> ExchangeResult<Vec<CountryInfo>> {
        let mut params = HashMap::new();

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }

        let response = self.get(WorldBankEndpoint::Countries, params).await?;
        WorldBankParser::parse_country_list(&response)
    }

    /// Get countries by income level
    ///
    /// # Arguments
    /// - `income_level` - Income level code (e.g., "HIC", "UMC", "LMC", "LIC")
    pub async fn get_country_by_income(&self, income_level: &str) -> ExchangeResult<Vec<CountryInfo>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "500".to_string());

        let response = self
            .get(
                WorldBankEndpoint::IncomeCountries {
                    level: income_level.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_country_list(&response)
    }

    /// Get countries by lending type
    ///
    /// # Arguments
    /// - `lending_type` - Lending type code (e.g., "IBD", "IDB", "IDX", "LNX")
    pub async fn get_country_by_lending(&self, lending_type: &str) -> ExchangeResult<Vec<CountryInfo>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "500".to_string());

        let response = self
            .get(
                WorldBankEndpoint::LendingCountries {
                    lending_type: lending_type.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_country_list(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CLASSIFICATION METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get topic metadata
    ///
    /// # Arguments
    /// - `topic_id` - Topic ID (e.g., "1", "3", "8")
    pub async fn get_topic(&self, topic_id: &str) -> ExchangeResult<Topic> {
        let params = HashMap::new();

        let response = self
            .get(
                WorldBankEndpoint::Topic {
                    id: topic_id.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_topic(&response)
    }

    /// List all topics
    pub async fn list_topics(&self) -> ExchangeResult<Vec<TopicInfo>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "100".to_string());

        let response = self.get(WorldBankEndpoint::Topics, params).await?;
        WorldBankParser::parse_topic_list(&response)
    }

    /// Get indicators for a specific topic
    ///
    /// # Arguments
    /// - `topic_id` - Topic ID
    /// - `page` - Optional page number
    /// - `per_page` - Optional results per page
    pub async fn get_indicator_by_topic(
        &self,
        topic_id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> ExchangeResult<Vec<IndicatorInfo>> {
        let mut params = HashMap::new();

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }

        let response = self
            .get(
                WorldBankEndpoint::TopicIndicators {
                    topic_id: topic_id.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_indicator_list(&response)
    }

    /// Get source metadata
    ///
    /// # Arguments
    /// - `source_id` - Source ID (e.g., "2" for World Development Indicators)
    pub async fn get_source(&self, source_id: &str) -> ExchangeResult<Source> {
        let params = HashMap::new();

        let response = self
            .get(
                WorldBankEndpoint::Source {
                    id: source_id.to_string(),
                },
                params,
            )
            .await?;

        WorldBankParser::parse_source(&response)
    }

    /// List all sources
    pub async fn list_sources(&self) -> ExchangeResult<Vec<SourceInfo>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "100".to_string());

        let response = self.get(WorldBankEndpoint::Sources, params).await?;
        WorldBankParser::parse_source_list(&response)
    }

    /// Get all income levels
    pub async fn get_income_levels(&self) -> ExchangeResult<Vec<IncomeLevel>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "50".to_string());

        let response = self.get(WorldBankEndpoint::IncomeLevels, params).await?;
        WorldBankParser::parse_income_levels(&response)
    }

    /// Get all lending types
    pub async fn get_lending_types(&self) -> ExchangeResult<Vec<LendingType>> {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "50".to_string());

        let response = self.get(WorldBankEndpoint::LendingTypes, params).await?;
        WorldBankParser::parse_lending_types(&response)
    }

    /// Ping (check connection)
    pub async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to get topics (lightweight endpoint)
        let _ = self.list_topics().await?;
        Ok(())
    }
}

impl Default for WorldBankConnector {
    fn default() -> Self {
        Self::new()
    }
}
