//! CBR connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Central Bank of Russia (CBR) connector
///
/// Provides access to Russian economic data including:
/// - Key interest rate
/// - Currency exchange rates
/// - Precious metal prices
/// - International reserves
/// - Monetary indicators
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::cbr::CbrConnector;
///
/// let connector = CbrConnector::new();
///
/// // Get current key rate
/// let key_rates = connector.get_key_rate().await?;
///
/// // Get daily exchange rates
/// let rates = connector.get_daily_rates(None).await?;
///
/// // Get currency list
/// let currencies = connector.get_currency_list().await?;
/// ```
pub struct CbrConnector {
    client: Client,
    auth: CbrAuth,
    endpoints: CbrEndpoints,
}

impl CbrConnector {
    /// Create new CBR connector
    ///
    /// CBR API is public and doesn't require authentication
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: CbrAuth::new(),
            endpoints: CbrEndpoints::default(),
        }
    }

    /// Create connector from environment (same as new() for CBR)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to CBR JSON API
    async fn get_json(
        &self,
        endpoint: CbrEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
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

        CbrParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request to CBR XML API
    async fn get_xml(
        &self,
        endpoint: CbrEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<String> {
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

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // JSON API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get current and historical key rate
    ///
    /// Returns the CBR key interest rate with historical data.
    /// This is the main monetary policy rate of the Central Bank of Russia.
    pub async fn get_key_rate(&self) -> ExchangeResult<Vec<KeyRate>> {
        let params = HashMap::new();
        let response = self.get_json(CbrEndpoint::KeyRate, params).await?;
        CbrParser::parse_key_rate(&response)
    }

    /// Get daily exchange rates (JSON format)
    ///
    /// # Arguments
    /// - `date` - Optional date in YYYY-MM-DD format. If None, returns today's rates.
    ///
    /// Returns exchange rates for all foreign currencies against RUB.
    pub async fn get_daily_rates(&self, _date: Option<&str>) -> ExchangeResult<DailyRates> {
        let params = HashMap::new();
        let response = self.get_json(CbrEndpoint::DailyJson, params).await?;
        CbrParser::parse_daily_json(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // XML API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get daily exchange rates (XML format)
    ///
    /// # Arguments
    /// - `date` - Optional date in DD/MM/YYYY format. If None, returns today's rates.
    ///
    /// Returns exchange rates for all foreign currencies against RUB.
    pub async fn get_daily_rates_xml(&self, date: Option<&str>) -> ExchangeResult<DailyRates> {
        let mut params = HashMap::new();
        if let Some(d) = date {
            params.insert("date_req".to_string(), format_date_cbr(d));
        }

        let response = self.get_xml(CbrEndpoint::DailyXml, params).await?;
        CbrParser::parse_daily_xml(&response)
    }

    /// Get list of all currencies
    ///
    /// Returns metadata for all currencies tracked by CBR.
    pub async fn get_currency_list(&self) -> ExchangeResult<Vec<Currency>> {
        let mut params = HashMap::new();
        params.insert("d".to_string(), "0".to_string());

        let response = self.get_xml(CbrEndpoint::CurrencyList, params).await?;
        CbrParser::parse_currency_list(&response)
    }

    /// Get historical exchange rate for a specific currency
    ///
    /// # Arguments
    /// - `currency_code` - Currency ID (e.g., "R01235" for USD)
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns time series of exchange rates for the specified period.
    pub async fn get_exchange_rate_dynamic(
        &self,
        currency_code: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<RatePoint>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));
        params.insert("VAL_NM_RQ".to_string(), currency_code.to_string());

        let response = self.get_xml(CbrEndpoint::ExchangeRateDynamic, params).await?;
        CbrParser::parse_rate_dynamic(&response)
    }

    /// Get precious metal prices
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns prices for gold, silver, platinum, and palladium.
    pub async fn get_metal_prices(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<MetalPrice>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        let response = self.get_xml(CbrEndpoint::MetalPrices, params).await?;
        CbrParser::parse_metal_prices(&response)
    }

    /// Get repo auction rates
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns repo auction rate data.
    pub async fn get_repo_rates(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ReserveData>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        let response = self.get_xml(CbrEndpoint::RepoRates, params).await?;
        CbrParser::parse_value_records(&response, "stavka")
    }

    /// Get international reserves
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns Russia's international reserve data.
    pub async fn get_international_reserves(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ReserveData>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        let response = self.get_xml(CbrEndpoint::InternationalReserves, params).await?;
        CbrParser::parse_value_records(&response, "Ostat")
    }

    /// Get monetary base data
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns monetary base indicators.
    pub async fn get_monetary_base(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ReserveData>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        let response = self.get_xml(CbrEndpoint::MonetaryBase, params).await?;
        CbrParser::parse_value_records(&response, "M0")
    }

    /// Get interbank rates (MKR)
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns Moscow interbank lending rate data.
    pub async fn get_interbank_rates(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ReserveData>> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        let response = self.get_xml(CbrEndpoint::InterbankRates, params).await?;
        CbrParser::parse_value_records(&response, "stavka")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C6 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get RUONIA overnight index average rate
    ///
    /// RUONIA (Ruble Overnight Index Average) is a benchmark rate based on
    /// actual overnight deposit transactions between Russian banks.
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns historical RUONIA rate data as XML string.
    pub async fn get_ruonia_rate(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        self.get_xml(CbrEndpoint::RuoniaRate, params).await
    }

    /// Get CBR deposit rates (depozit operations)
    ///
    /// Returns CBR deposit auction and overnight deposit rates.
    ///
    /// # Arguments
    /// - `start_date` - Start date in DD/MM/YYYY or YYYY-MM-DD format
    /// - `end_date` - End date in DD/MM/YYYY or YYYY-MM-DD format
    ///
    /// Returns deposit rate data as XML string.
    pub async fn get_deposit_rates(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("date_req1".to_string(), format_date_cbr(start_date));
        params.insert("date_req2".to_string(), format_date_cbr(end_date));

        self.get_xml(CbrEndpoint::DepositRates, params).await
    }

    /// Get refinancing rate (key rate) historical series
    ///
    /// The refinancing rate (now the key rate) is the main monetary policy
    /// instrument of the Central Bank of Russia.
    ///
    /// Returns the same data as `get_key_rate` but intended for historical
    /// analysis. Uses JSON API endpoint.
    ///
    /// # Returns
    /// Vector of key rate records with effective dates and rate values.
    pub async fn get_refinancing_rate_history(&self) -> ExchangeResult<Vec<super::parser::KeyRate>> {
        let params = HashMap::new();
        let response = self.get_json(CbrEndpoint::RefinancingRateHistory, params).await?;
        super::parser::CbrParser::parse_key_rate(&response)
    }
}

impl Default for CbrConnector {
    fn default() -> Self {
        Self::new()
    }
}
