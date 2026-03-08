//! Bank of Korea ECOS connector implementation

use reqwest::Client;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    EcosParser, StatisticData, KeyStatistic, StatisticTable, StatisticItem,
    StatisticWord, StatMeta,
};

/// ECOS (Economic Statistics System) connector
///
/// Provides access to economic data from the Bank of Korea.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ecos::EcosConnector;
///
/// let connector = EcosConnector::from_env();
///
/// // Get GDP data
/// let data = connector.get_statistical_data(
///     "200Y001",  // GDP stat code
///     "Q",        // Quarterly
///     "2020Q1",   // Start date
///     "2023Q4",   // End date
///     None,       // No item codes
/// ).await?;
///
/// // Get key statistics
/// let stats = connector.get_key_statistics().await?;
/// ```
pub struct EcosConnector {
    client: Client,
    auth: EcosAuth,
    endpoints: EcosEndpoints,
}

impl EcosConnector {
    /// Create new ECOS connector with authentication
    pub fn new(auth: EcosAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: EcosEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `ECOS_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(EcosAuth::from_env())
    }

    /// Build URL path for ECOS API request
    ///
    /// ECOS uses path-based parameters:
    /// /{service}/{api_key}/{format}/{lang}/{startCount}/{endCount}/...
    fn build_url_path(
        &self,
        endpoint: &EcosEndpoint,
        path_params: &[&str],
    ) -> ExchangeResult<String> {
        let api_key = self.auth.get_api_key()?;
        let service = endpoint.service_name();

        // Base path structure: /{service}/{api_key}/json/en/{start}/{end}
        let mut path = format!(
            "{}/{}/{}/json/en/1/100",
            self.endpoints.rest_base,
            service,
            api_key
        );

        // Append additional path parameters
        for param in path_params {
            path.push('/');
            path.push_str(param);
        }

        Ok(path)
    }

    /// Internal: Make GET request to ECOS API
    async fn get(
        &self,
        endpoint: EcosEndpoint,
        path_params: &[&str],
    ) -> ExchangeResult<serde_json::Value> {
        let url = self.build_url_path(&endpoint, path_params)?;

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

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for ECOS API errors
        EcosParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECOS-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get statistical data by stat code, cycle, and date range
    ///
    /// # Arguments
    /// - `stat_code` - Statistical code (e.g., "200Y001" for GDP)
    /// - `cycle` - Frequency: "A" (annual), "Q" (quarterly), "M" (monthly), "D" (daily)
    /// - `start_date` - Start date in format matching cycle (e.g., "2020Q1" for quarterly)
    /// - `end_date` - End date in format matching cycle
    /// - `item_codes` - Optional tuple of item codes (item1, item2, item3)
    ///
    /// # Returns
    /// Vector of statistical data points with values
    ///
    /// # Example
    /// ```ignore
    /// // Get quarterly GDP from 2020Q1 to 2023Q4
    /// let data = connector.get_statistical_data(
    ///     "200Y001",
    ///     "Q",
    ///     "2020Q1",
    ///     "2023Q4",
    ///     None,
    /// ).await?;
    /// ```
    pub async fn get_statistical_data(
        &self,
        stat_code: &str,
        cycle: &str,
        start_date: &str,
        end_date: &str,
        item_codes: Option<(&str, &str, &str)>,
    ) -> ExchangeResult<Vec<StatisticData>> {
        let mut params = vec![stat_code, cycle, start_date, end_date];

        // Add item codes if provided
        if let Some((item1, item2, item3)) = item_codes {
            params.push(item1);
            params.push(item2);
            params.push(item3);
        }

        let response = self.get(EcosEndpoint::StatisticSearch, &params).await?;
        EcosParser::parse_statistical_data(&response)
    }

    /// Get list of key statistics
    ///
    /// Returns metadata for important economic indicators tracked by BOK
    pub async fn get_key_statistics(&self) -> ExchangeResult<Vec<KeyStatistic>> {
        let response = self.get(EcosEndpoint::KeyStatisticList, &[]).await?;
        EcosParser::parse_key_statistics(&response)
    }

    /// Get statistical table list by stat code
    ///
    /// # Arguments
    /// - `stat_code` - Statistical code to query
    ///
    /// # Returns
    /// Vector of available statistical tables for the given code
    pub async fn get_stat_table_list(&self, stat_code: &str) -> ExchangeResult<Vec<StatisticTable>> {
        let response = self.get(EcosEndpoint::StatisticTableList, &[stat_code]).await?;
        EcosParser::parse_statistic_tables(&response)
    }

    /// Get statistical item list by stat code
    ///
    /// # Arguments
    /// - `stat_code` - Statistical code to query
    ///
    /// # Returns
    /// Vector of available items/series within the statistical category
    pub async fn get_stat_item_list(&self, stat_code: &str) -> ExchangeResult<Vec<StatisticItem>> {
        let response = self.get(EcosEndpoint::StatisticItemList, &[stat_code]).await?;
        EcosParser::parse_statistic_items(&response)
    }

    /// Search statistics by keyword
    ///
    /// # Arguments
    /// - `word` - Search keyword (in Korean or English)
    ///
    /// # Returns
    /// Vector of statistics matching the search term
    pub async fn get_stat_word(&self, word: &str) -> ExchangeResult<Vec<StatisticWord>> {
        let response = self.get(EcosEndpoint::StatisticWord, &[word]).await?;
        EcosParser::parse_statistic_words(&response)
    }

    /// Get statistical metadata
    ///
    /// # Arguments
    /// - `data_name` - Name of the statistical data to query
    ///
    /// # Returns
    /// Vector of metadata entries for the specified data
    pub async fn get_stat_meta(&self, data_name: &str) -> ExchangeResult<Vec<StatMeta>> {
        let response = self.get(EcosEndpoint::StatMeta, &[data_name]).await?;
        EcosParser::parse_stat_meta(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS FOR COMMON INDICATORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get GDP data (quarterly)
    ///
    /// # Arguments
    /// - `start_date` - Start quarter (e.g., "2020Q1")
    /// - `end_date` - End quarter (e.g., "2023Q4")
    pub async fn get_gdp(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_GDP, CYCLE_QUARTERLY, start_date, end_date, None)
            .await
    }

    /// Get CPI data (monthly)
    ///
    /// # Arguments
    /// - `start_date` - Start month (e.g., "202001")
    /// - `end_date` - End month (e.g., "202312")
    pub async fn get_cpi(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_CPI, CYCLE_MONTHLY, start_date, end_date, None)
            .await
    }

    /// Get policy rate (base rate) data
    ///
    /// # Arguments
    /// - `start_date` - Start date (e.g., "20200101")
    /// - `end_date` - End date (e.g., "20231231")
    pub async fn get_policy_rate(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_POLICY_RATE, CYCLE_DAILY, start_date, end_date, None)
            .await
    }

    /// Get exchange rate data (daily)
    ///
    /// # Arguments
    /// - `start_date` - Start date (e.g., "20200101")
    /// - `end_date` - End date (e.g., "20231231")
    pub async fn get_exchange_rates(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_EXCHANGE_RATES, CYCLE_DAILY, start_date, end_date, None)
            .await
    }

    /// Get employment data (monthly)
    ///
    /// # Arguments
    /// - `start_date` - Start month (e.g., "202001")
    /// - `end_date` - End month (e.g., "202312")
    pub async fn get_employment(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_EMPLOYMENT, CYCLE_MONTHLY, start_date, end_date, None)
            .await
    }

    /// Get money supply data (monthly)
    ///
    /// # Arguments
    /// - `start_date` - Start month (e.g., "202001")
    /// - `end_date` - End month (e.g., "202312")
    pub async fn get_money_supply(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_MONEY_SUPPLY, CYCLE_MONTHLY, start_date, end_date, None)
            .await
    }

    /// Get trade balance data (monthly)
    ///
    /// # Arguments
    /// - `start_date` - Start month (e.g., "202001")
    /// - `end_date` - End month (e.g., "202312")
    pub async fn get_trade_balance(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(STAT_TRADE, CYCLE_MONTHLY, start_date, end_date, None)
            .await
    }

    /// Get industrial production data (monthly)
    ///
    /// # Arguments
    /// - `start_date` - Start month (e.g., "202001")
    /// - `end_date` - End month (e.g., "202312")
    pub async fn get_industrial_production(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<StatisticData>> {
        self.get_statistical_data(
            STAT_INDUSTRIAL_PRODUCTION,
            CYCLE_MONTHLY,
            start_date,
            end_date,
            None,
        )
        .await
    }
}
