//! IMF connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    ImfParser, Dataflow, ImfSeries, DataStructure, CodeList,
};

/// IMF (International Monetary Fund) connector
///
/// Provides access to international economic and financial data.
///
/// # Key Databases
/// - IFS: International Financial Statistics
/// - BOP: Balance of Payments
/// - DOT: Direction of Trade Statistics
/// - GFS: Government Finance Statistics
/// - WEO: World Economic Outlook
/// - PCPS: Primary Commodity Prices
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::imf::ImfConnector;
///
/// let connector = ImfConnector::new();
///
/// // Get GDP data for US
/// let data = connector.get_data(
///     "IFS",
///     "A.US.NGDP_RPCH",
///     Some("2010"),
///     Some("2023")
/// ).await?;
///
/// // List available datasets
/// let dataflows = connector.list_dataflows().await?;
/// ```
pub struct ImfConnector {
    client: Client,
    auth: ImfAuth,
    endpoints: ImfEndpoints,
}

impl ImfConnector {
    /// Create new IMF connector (no auth required)
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: ImfAuth::new(),
            endpoints: ImfEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for IMF)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to IMF API
    async fn get(
        &self,
        endpoint: ImfEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add auth (no-op for IMF)
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

        // Check for IMF API errors
        ImfParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // IMF-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of all available dataflows (datasets)
    ///
    /// Returns metadata about available databases like IFS, BOP, DOT, etc.
    ///
    /// # Returns
    /// Vector of dataflow metadata (id, name, description)
    pub async fn list_dataflows(&self) -> ExchangeResult<Vec<Dataflow>> {
        let params = HashMap::new();
        let response = self.get(ImfEndpoint::Dataflow, params).await?;
        ImfParser::parse_dataflows(&response)
    }

    /// Get economic data from a specific database
    ///
    /// This is the CORE endpoint for retrieving time series data.
    ///
    /// # Arguments
    /// - `database_id` - Database ID (e.g., "IFS", "BOP", "WEO")
    /// - `dimensions` - Dimension filter (e.g., "A.US.NGDP_RPCH" for annual US GDP growth)
    /// - `start_period` - Optional start period (e.g., "2010", "2010-Q1")
    /// - `end_period` - Optional end period (e.g., "2023", "2023-Q4")
    ///
    /// # Dimension Format
    /// Dot-separated: `{freq}.{country}.{indicator}`
    /// - freq: A (Annual), Q (Quarterly), M (Monthly)
    /// - country: ISO 2-letter code (US, GB, DE, JP, CN, etc.)
    /// - indicator: Indicator code (NGDP_RPCH, PCPI_IX, etc.)
    ///
    /// # Example
    /// ```ignore
    /// // Get annual US GDP growth from 2010 to 2023
    /// let data = connector.get_data(
    ///     "IFS",
    ///     "A.US.NGDP_RPCH",
    ///     Some("2010"),
    ///     Some("2023")
    /// ).await?;
    /// ```
    ///
    /// # Returns
    /// Vector of time series with observations
    pub async fn get_data(
        &self,
        database_id: &str,
        dimensions: &str,
        start_period: Option<&str>,
        end_period: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let mut params = HashMap::new();

        if let Some(start) = start_period {
            params.insert("startPeriod".to_string(), start.to_string());
        }
        if let Some(end) = end_period {
            params.insert("endPeriod".to_string(), end.to_string());
        }

        let endpoint = ImfEndpoint::CompactData {
            database_id: database_id.to_string(),
            dimensions: dimensions.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        ImfParser::parse_compact_data(&response)
    }

    /// Get data structure definition for a database
    ///
    /// Returns dimension definitions and metadata for a database.
    ///
    /// # Arguments
    /// - `database_id` - Database ID (e.g., "IFS", "BOP")
    ///
    /// # Returns
    /// Data structure with dimension definitions
    pub async fn get_data_structure(&self, database_id: &str) -> ExchangeResult<DataStructure> {
        let params = HashMap::new();
        let endpoint = ImfEndpoint::DataStructure {
            database_id: database_id.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        ImfParser::parse_data_structure(&response)
    }

    /// Get code list for a dimension
    ///
    /// Returns available codes (valid values) for a specific dimension.
    ///
    /// # Arguments
    /// - `database_id` - Database ID (e.g., "IFS")
    /// - `code_list_id` - Code list ID (e.g., "CL_INDICATOR_IFS" for indicators)
    ///
    /// # Common Code Lists (IFS)
    /// - CL_INDICATOR_IFS: Available indicators
    /// - CL_AREA_IFS: Available countries/regions
    /// - CL_FREQ: Frequency codes (A, Q, M)
    ///
    /// # Returns
    /// Code list with available values
    pub async fn get_code_list(
        &self,
        database_id: &str,
        code_list_id: &str,
    ) -> ExchangeResult<CodeList> {
        let params = HashMap::new();
        let endpoint = ImfEndpoint::CodeList {
            code_list_id: code_list_id.to_string(),
            database_id: database_id.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        ImfParser::parse_code_list(&response)
    }

    /// Get data in generic format
    ///
    /// Alternative data format with more metadata than CompactData.
    ///
    /// # Arguments
    /// - `database_id` - Database ID (e.g., "IFS")
    /// - `dimensions` - Dimension filter (e.g., "A.US.NGDP_RPCH")
    ///
    /// # Returns
    /// Vector of time series (parsed same as compact data)
    pub async fn get_generic_data(
        &self,
        database_id: &str,
        dimensions: &str,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let params = HashMap::new();
        let endpoint = ImfEndpoint::GenericData {
            database_id: database_id.to_string(),
            dimensions: dimensions.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        // GenericData uses same structure as CompactData for series
        ImfParser::parse_compact_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get GDP growth data for a country
    ///
    /// Convenience method for getting real GDP growth (NGDP_RPCH) from IFS.
    ///
    /// # Arguments
    /// - `country` - ISO 2-letter country code (e.g., "US", "GB", "CN")
    /// - `start` - Optional start year (e.g., "2010")
    /// - `end` - Optional end year (e.g., "2023")
    ///
    /// # Returns
    /// Vector of time series with GDP growth data
    pub async fn get_gdp_growth(
        &self,
        country: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let dimensions = format!("A.{}.NGDP_RPCH", country.to_uppercase());
        self.get_data("IFS", &dimensions, start, end).await
    }

    /// Get CPI (inflation) data for a country
    ///
    /// Convenience method for getting Consumer Price Index (PCPI_IX) from IFS.
    ///
    /// # Arguments
    /// - `country` - ISO 2-letter country code
    /// - `start` - Optional start period
    /// - `end` - Optional end period
    ///
    /// # Returns
    /// Vector of time series with CPI data
    pub async fn get_cpi(
        &self,
        country: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let dimensions = format!("M.{}.PCPI_IX", country.to_uppercase());
        self.get_data("IFS", &dimensions, start, end).await
    }

    /// Get interest rate data for a country
    ///
    /// Convenience method for getting policy rate (FPOLM_PA) from IFS.
    ///
    /// # Arguments
    /// - `country` - ISO 2-letter country code
    /// - `start` - Optional start period
    /// - `end` - Optional end period
    ///
    /// # Returns
    /// Vector of time series with interest rate data
    pub async fn get_interest_rate(
        &self,
        country: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let dimensions = format!("M.{}.FPOLM_PA", country.to_uppercase());
        self.get_data("IFS", &dimensions, start, end).await
    }

    /// Get exchange rate data
    ///
    /// Convenience method for getting exchange rate to USD (ENDA_XDC_USD_RATE) from IFS.
    ///
    /// # Arguments
    /// - `country` - ISO 2-letter country code
    /// - `start` - Optional start period
    /// - `end` - Optional end period
    ///
    /// # Returns
    /// Vector of time series with exchange rate data
    pub async fn get_exchange_rate(
        &self,
        country: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        let dimensions = format!("M.{}.ENDA_XDC_USD_RATE", country.to_uppercase());
        self.get_data("IFS", &dimensions, start, end).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C6 ADDITIONS — WEO (World Economic Outlook) endpoints
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all WEO indicator codes (concepts)
    ///
    /// Returns the full list of World Economic Outlook series codes such as
    /// NGDP_RPCH (real GDP growth), PCPI (CPI), LUR (unemployment), etc.
    ///
    /// # Returns
    /// Code list as raw JSON — parse keys for indicator codes and descriptions.
    pub async fn get_weo_indicators(&self) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, ImfEndpoint::WeoIndicators.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Get all WEO country codes (ISO codes used by the WEO database)
    ///
    /// Returns country codes and names as used in the WEO dataset.
    ///
    /// # Returns
    /// Code list as raw JSON.
    pub async fn get_weo_countries(&self) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, ImfEndpoint::WeoCountries.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Get all WEO regional aggregates
    ///
    /// Returns region group codes such as Emerging Market Economies,
    /// Advanced Economies, etc.
    ///
    /// # Returns
    /// Code list as raw JSON.
    pub async fn get_weo_regions(&self) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, ImfEndpoint::WeoRegions.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Fetch WEO forecast data
    ///
    /// Uses the WEO database (World Economic Outlook) to retrieve IMF forecasts.
    ///
    /// # Arguments
    /// - `country` - Country code (e.g., "US", "DE") or empty string for all
    /// - `indicator` - WEO indicator code (e.g., "NGDP_RPCH", "PCPI", "LUR")
    /// - `start` - Optional start year (e.g., "2020")
    /// - `end` - Optional end year (e.g., "2026")
    ///
    /// # Returns
    /// Vector of IMF series data points with forecast values
    pub async fn get_weo_data(
        &self,
        country: &str,
        indicator: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> ExchangeResult<Vec<ImfSeries>> {
        // WEO uses annual frequency "A"
        let dimensions = format!("A.{}.{}", country.to_uppercase(), indicator.to_uppercase());
        self.get_data("WEO", &dimensions, start, end).await
    }
}

impl Default for ImfConnector {
    fn default() -> Self {
        Self::new()
    }
}
