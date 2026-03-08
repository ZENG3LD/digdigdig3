//! BLS connector implementation

use reqwest::Client;

use crate::core::types::{
    ExchangeError, ExchangeResult,
};

use super::endpoints::*;
use super::auth::*;
use super::parser::{BlsParser, BlsSeries};

/// BLS (Bureau of Labor Statistics) connector
///
/// Provides access to US labor market and economic indicators including:
/// - Consumer Price Index (CPI)
/// - Unemployment Rate
/// - Nonfarm Payrolls
/// - Producer Price Index (PPI)
/// - Job Openings and Labor Turnover Survey (JOLTS)
/// - Employment Cost Index
/// - And 10,000+ other economic series
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::bls::BlsConnector;
///
/// let connector = BlsConnector::from_env();
///
/// // Get CPI data for 2020-2024
/// let cpi = connector.get_cpi("2020", "2024").await?;
///
/// // Get unemployment rate
/// let unemployment = connector.get_unemployment_rate("2020", "2024").await?;
///
/// // Get multiple series at once (up to 50)
/// let series = connector.get_multiple_series(
///     &["CUSR0000SA0", "LNS14000000"],
///     "2023",
///     "2024"
/// ).await?;
/// ```
pub struct BlsConnector {
    client: Client,
    auth: BlsAuth,
    endpoints: BlsEndpoints,
    _testnet: bool,
}

impl BlsConnector {
    /// Create new BLS connector with authentication
    pub fn new(auth: BlsAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: BlsEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `BLS_API_KEY` environment variable (optional)
    ///
    /// Without API key: 25 queries/day, 10 years max range
    /// With API key: 500 queries/day, 20 years max range
    pub fn from_env() -> Self {
        Self::new(BlsAuth::from_env())
    }

    /// Create connector for public access (no API key)
    ///
    /// Rate limits: 25 queries/day, 10 years max range
    pub fn public() -> Self {
        Self::new(BlsAuth::public())
    }

    /// Internal: Make POST request to BLS API v2
    ///
    /// BLS v2 uses POST with JSON body, not GET with query params
    async fn post(
        &self,
        endpoint: BlsEndpoint,
        mut body: serde_json::Map<String, serde_json::Value>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication to body
        self.auth.sign_body(&mut body);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
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

        // Check for BLS API errors
        BlsParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BLS-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get time series data for one or more series
    ///
    /// This is the CORE endpoint for retrieving BLS data.
    ///
    /// # Arguments
    /// - `series_ids` - Array of BLS series IDs (up to 50)
    /// - `start_year` - Start year (YYYY format)
    /// - `end_year` - End year (YYYY format)
    /// - `catalog` - Include catalog metadata (optional)
    /// - `calculations` - Include calculated values (optional)
    /// - `annual_average` - Include annual averages (optional)
    ///
    /// # Returns
    /// Vector of series with their data points
    ///
    /// # Example
    /// ```ignore
    /// let series = connector.get_series_data(
    ///     &["CUSR0000SA0", "LNS14000000"],
    ///     "2020",
    ///     "2024",
    ///     Some(true),
    ///     Some(true),
    ///     Some(true),
    /// ).await?;
    /// ```
    pub async fn get_series_data(
        &self,
        series_ids: &[&str],
        start_year: &str,
        end_year: &str,
        catalog: Option<bool>,
        calculations: Option<bool>,
        annual_average: Option<bool>,
    ) -> ExchangeResult<Vec<BlsSeries>> {
        let mut body = serde_json::Map::new();

        // Series IDs array
        let series_array: Vec<serde_json::Value> = series_ids
            .iter()
            .map(|id| serde_json::Value::String(id.to_string()))
            .collect();
        body.insert("seriesid".to_string(), serde_json::Value::Array(series_array));

        // Date range
        body.insert("startyear".to_string(), serde_json::Value::String(start_year.to_string()));
        body.insert("endyear".to_string(), serde_json::Value::String(end_year.to_string()));

        // Optional parameters
        if let Some(cat) = catalog {
            body.insert("catalog".to_string(), serde_json::Value::Bool(cat));
        }
        if let Some(calc) = calculations {
            body.insert("calculations".to_string(), serde_json::Value::Bool(calc));
        }
        if let Some(avg) = annual_average {
            body.insert("annualaverage".to_string(), serde_json::Value::Bool(avg));
        }

        let response = self.post(BlsEndpoint::TimeSeriesData, body).await?;
        BlsParser::parse_series_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS FOR POPULAR SERIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Consumer Price Index - All Urban Consumers (CPI-U)
    ///
    /// Series: CUSR0000SA0
    /// The most widely used measure of inflation
    pub async fn get_cpi(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[CPI_ALL_URBAN], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Unemployment Rate
    ///
    /// Series: LNS14000000
    /// Civilian unemployment rate (seasonally adjusted)
    pub async fn get_unemployment_rate(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[UNEMPLOYMENT_RATE], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Total Nonfarm Payroll Employment
    ///
    /// Series: CES0000000001
    /// Total nonfarm employment (seasonally adjusted)
    pub async fn get_nonfarm_payrolls(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[NONFARM_PAYROLLS], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Average Hourly Earnings - Total Private
    ///
    /// Series: CES0500000003
    /// Average hourly earnings of all employees, total private (seasonally adjusted)
    pub async fn get_average_hourly_earnings(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[AVG_HOURLY_EARNINGS], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Producer Price Index - Finished Goods
    ///
    /// Series: WPSFD4
    /// PPI for finished goods (not seasonally adjusted)
    pub async fn get_ppi(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[PPI_FINISHED_GOODS], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Employment Cost Index - Total Compensation
    ///
    /// Series: CIU1010000000000A
    /// Employment cost index for total compensation (not seasonally adjusted)
    pub async fn get_employment_cost_index(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[EMPLOYMENT_COST_INDEX], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Productivity - Nonfarm Business Sector
    ///
    /// Series: PRS85006092
    /// Labor productivity (output per hour) for nonfarm business sector
    pub async fn get_productivity(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[PRODUCTIVITY], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Import Price Index - All Commodities
    ///
    /// Series: EIUIR
    /// Import price index for all commodities
    pub async fn get_import_prices(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[IMPORT_PRICES], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Export Price Index - All Commodities
    ///
    /// Series: EIUIQ
    /// Export price index for all commodities
    pub async fn get_export_prices(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[EXPORT_PRICES], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get Job Openings and Labor Turnover Survey (JOLTS)
    ///
    /// Series: JTS000000000000000JOL
    /// Total nonfarm job openings (seasonally adjusted)
    pub async fn get_jolts(&self, start_year: &str, end_year: &str) -> ExchangeResult<BlsSeries> {
        let mut results = self.get_series_data(&[JOLTS_JOB_OPENINGS], start_year, end_year, None, None, None).await?;
        results.pop().ok_or_else(|| ExchangeError::Parse("No data returned".to_string()))
    }

    /// Get multiple series in a single request (up to 50 series)
    ///
    /// More efficient than individual requests when you need multiple series
    ///
    /// # Example
    /// ```ignore
    /// let series = connector.get_multiple_series(
    ///     &["CUSR0000SA0", "LNS14000000", "CES0000000001"],
    ///     "2023",
    ///     "2024"
    /// ).await?;
    /// ```
    pub async fn get_multiple_series(
        &self,
        series_ids: &[&str],
        start_year: &str,
        end_year: &str,
    ) -> ExchangeResult<Vec<BlsSeries>> {
        if series_ids.is_empty() {
            return Err(ExchangeError::InvalidRequest("No series IDs provided".to_string()));
        }
        if series_ids.len() > 50 {
            return Err(ExchangeError::InvalidRequest("Maximum 50 series per request".to_string()));
        }

        self.get_series_data(series_ids, start_year, end_year, None, None, None).await
    }

    /// Get latest numbers for series (most recent data point)
    ///
    /// Note: BLS v2 API doesn't have a specific "latest" endpoint,
    /// so this method requests the last 2 years and filters for latest=true
    pub async fn get_latest_numbers(&self, series_ids: &[&str]) -> ExchangeResult<Vec<BlsSeries>> {
        // Get current year
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ExchangeError::Parse(format!("Time error: {}", e)))?;
        let current_year = 1970 + (now.as_secs() / 31536000);
        let start_year = (current_year - 1).to_string();
        let end_year = current_year.to_string();

        let mut series = self.get_series_data(series_ids, &start_year, &end_year, None, None, None).await?;

        // Filter each series to only include latest data points
        for s in &mut series {
            s.data.retain(|point| point.latest);
        }

        Ok(series)
    }
}

// BLS is a data feed, not a trading exchange — no trait implementations needed.
// Use BLS-specific methods: get_cpi(), get_unemployment_rate(), etc.
