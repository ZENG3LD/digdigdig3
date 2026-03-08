//! Alpha Vantage connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Alpha Vantage connector
///
/// Provides access to stocks, forex, crypto, economic indicators, technical indicators, and commodities data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::alpha_vantage::AlphaVantageConnector;
///
/// let connector = AlphaVantageConnector::from_env();
///
/// // Get stock quote
/// let quote = connector.get_quote("IBM").await?;
///
/// // Get economic indicator
/// let gdp = connector.get_gdp().await?;
///
/// // Search symbols
/// let results = connector.search_symbols("microsoft").await?;
/// ```
pub struct AlphaVantageConnector {
    client: Client,
    auth: AlphaVantageAuth,
    endpoints: AlphaVantageEndpoints,
}

impl AlphaVantageConnector {
    /// Create new Alpha Vantage connector with authentication
    pub fn new(auth: AlphaVantageAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AlphaVantageEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `ALPHA_VANTAGE_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(AlphaVantageAuth::from_env())
    }

    /// Internal: Make GET request to Alpha Vantage API
    async fn get(
        &self,
        endpoint: AlphaVantageEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add function parameter
        params.insert("function".to_string(), endpoint.function().to_string());

        // Add API key authentication
        self.auth.sign_query(&mut params);

        let response = self
            .client
            .get(self.endpoints.rest_base)
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

        // Check for Alpha Vantage API errors
        AlphaVantageParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STOCK/EQUITY ENDPOINTS (6)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get real-time quote for a stock symbol
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "IBM", "AAPL", "MSFT")
    pub async fn get_quote(&self, symbol: &str) -> ExchangeResult<GlobalQuote> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::GlobalQuote, params).await?;
        AlphaVantageParser::parse_global_quote(&response)
    }

    /// Get intraday time series (OHLCV data)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    /// - `interval` - Time interval: "1min", "5min", "15min", "30min", "60min"
    pub async fn get_intraday(&self, symbol: &str, interval: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("outputsize".to_string(), "compact".to_string());

        let response = self.get(AlphaVantageEndpoint::TimeSeriesIntraday, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Time Series")
    }

    /// Get daily time series (OHLCV data)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    pub async fn get_daily(&self, symbol: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("outputsize".to_string(), "compact".to_string());

        let response = self.get(AlphaVantageEndpoint::TimeSeriesDaily, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Time Series")
    }

    /// Get weekly time series
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    pub async fn get_weekly(&self, symbol: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::TimeSeriesWeekly, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Weekly Time Series")
    }

    /// Get monthly time series
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    pub async fn get_monthly(&self, symbol: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::TimeSeriesMonthly, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Monthly Time Series")
    }

    /// Search for symbols by keywords
    ///
    /// # Arguments
    /// - `keywords` - Search keywords (e.g., "microsoft", "tesla")
    pub async fn search_symbols(&self, keywords: &str) -> ExchangeResult<Vec<SymbolMatch>> {
        let mut params = HashMap::new();
        params.insert("keywords".to_string(), keywords.to_string());

        let response = self.get(AlphaVantageEndpoint::SymbolSearch, params).await?;
        AlphaVantageParser::parse_symbol_search(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FOREX ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get real-time forex exchange rate
    ///
    /// # Arguments
    /// - `from` - From currency code (e.g., "USD", "EUR")
    /// - `to` - To currency code (e.g., "JPY", "GBP")
    pub async fn get_fx_rate(&self, from: &str, to: &str) -> ExchangeResult<ForexRate> {
        let mut params = HashMap::new();
        params.insert("from_currency".to_string(), from.to_uppercase());
        params.insert("to_currency".to_string(), to.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::CurrencyExchangeRate, params).await?;
        AlphaVantageParser::parse_fx_rate(&response)
    }

    /// Get daily forex time series
    ///
    /// # Arguments
    /// - `from` - From currency code
    /// - `to` - To currency code
    pub async fn get_fx_daily(&self, from: &str, to: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("from_symbol".to_string(), from.to_uppercase());
        params.insert("to_symbol".to_string(), to.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::FxDaily, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Time Series FX")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CRYPTO ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get crypto rating/health score
    ///
    /// # Arguments
    /// - `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    pub async fn get_crypto_rating(&self, symbol: &str) -> ExchangeResult<CryptoRating> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::CryptoRating, params).await?;
        AlphaVantageParser::parse_crypto_rating(&response)
    }

    /// Get daily crypto time series
    ///
    /// # Arguments
    /// - `symbol` - Crypto symbol (e.g., "BTC", "ETH")
    /// - `market` - Market currency (e.g., "USD", "EUR")
    pub async fn get_crypto_daily(&self, symbol: &str, market: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("market".to_string(), market.to_uppercase());

        let response = self.get(AlphaVantageEndpoint::DigitalCurrencyDaily, params).await?;
        AlphaVantageParser::parse_time_series(&response, "Time Series (Digital Currency Daily)")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECONOMIC INDICATORS ENDPOINTS (9)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get real GDP data (quarterly)
    pub async fn get_gdp(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), "quarterly".to_string());

        let response = self.get(AlphaVantageEndpoint::RealGdp, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get real GDP per capita
    pub async fn get_gdp_per_capita(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let params = HashMap::new();

        let response = self.get(AlphaVantageEndpoint::RealGdpPerCapita, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get Treasury yield rates
    ///
    /// # Arguments
    /// - `interval` - Data interval: "daily", "weekly", "monthly"
    /// - `maturity` - Maturity period: "3month", "2year", "5year", "7year", "10year", "30year"
    pub async fn get_treasury_yield(&self, interval: &str, maturity: &str) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), interval.to_string());
        params.insert("maturity".to_string(), maturity.to_string());

        let response = self.get(AlphaVantageEndpoint::TreasuryYield, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get federal funds rate
    ///
    /// # Arguments
    /// - `interval` - Data interval: "daily", "weekly", "monthly"
    pub async fn get_federal_funds_rate(&self, interval: &str) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), interval.to_string());

        let response = self.get(AlphaVantageEndpoint::FederalFundsRate, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get Consumer Price Index
    ///
    /// # Arguments
    /// - `interval` - Data interval: "monthly", "semiannual"
    pub async fn get_cpi(&self, interval: &str) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), interval.to_string());

        let response = self.get(AlphaVantageEndpoint::Cpi, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get inflation rate
    pub async fn get_inflation(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let params = HashMap::new();

        let response = self.get(AlphaVantageEndpoint::Inflation, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get retail sales data
    pub async fn get_retail_sales(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let params = HashMap::new();

        let response = self.get(AlphaVantageEndpoint::RetailSales, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get unemployment rate
    pub async fn get_unemployment(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let params = HashMap::new();

        let response = self.get(AlphaVantageEndpoint::Unemployment, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get nonfarm payroll data
    pub async fn get_nonfarm_payroll(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let params = HashMap::new();

        let response = self.get(AlphaVantageEndpoint::NonfarmPayroll, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TECHNICAL INDICATORS ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Simple Moving Average (SMA)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    /// - `interval` - Time interval: "1min", "5min", "15min", "30min", "60min", "daily", "weekly", "monthly"
    /// - `time_period` - Number of data points (e.g., 20, 50, 200)
    /// - `series_type` - Price type: "close", "open", "high", "low"
    pub async fn get_sma(&self, symbol: &str, interval: &str, time_period: u32, series_type: &str) -> ExchangeResult<Vec<TechnicalIndicatorEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("time_period".to_string(), time_period.to_string());
        params.insert("series_type".to_string(), series_type.to_string());

        let response = self.get(AlphaVantageEndpoint::Sma, params).await?;
        AlphaVantageParser::parse_technical_indicator(&response, "Technical Analysis: SMA")
    }

    /// Get Exponential Moving Average (EMA)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    /// - `interval` - Time interval
    /// - `time_period` - Number of data points
    /// - `series_type` - Price type
    pub async fn get_ema(&self, symbol: &str, interval: &str, time_period: u32, series_type: &str) -> ExchangeResult<Vec<TechnicalIndicatorEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("time_period".to_string(), time_period.to_string());
        params.insert("series_type".to_string(), series_type.to_string());

        let response = self.get(AlphaVantageEndpoint::Ema, params).await?;
        AlphaVantageParser::parse_technical_indicator(&response, "Technical Analysis: EMA")
    }

    /// Get Relative Strength Index (RSI)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    /// - `interval` - Time interval
    /// - `time_period` - Number of data points (typically 14)
    /// - `series_type` - Price type
    pub async fn get_rsi(&self, symbol: &str, interval: &str, time_period: u32, series_type: &str) -> ExchangeResult<Vec<TechnicalIndicatorEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("time_period".to_string(), time_period.to_string());
        params.insert("series_type".to_string(), series_type.to_string());

        let response = self.get(AlphaVantageEndpoint::Rsi, params).await?;
        AlphaVantageParser::parse_technical_indicator(&response, "Technical Analysis: RSI")
    }

    /// Get Moving Average Convergence Divergence (MACD)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol
    /// - `interval` - Time interval
    /// - `series_type` - Price type
    pub async fn get_macd(&self, symbol: &str, interval: &str, series_type: &str) -> ExchangeResult<Vec<TechnicalIndicatorEntry>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("series_type".to_string(), series_type.to_string());

        let response = self.get(AlphaVantageEndpoint::Macd, params).await?;
        AlphaVantageParser::parse_technical_indicator(&response, "Technical Analysis: MACD")
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMMODITIES ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get WTI crude oil prices (monthly)
    pub async fn get_wti(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), "monthly".to_string());

        let response = self.get(AlphaVantageEndpoint::Wti, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get Brent crude oil prices (monthly)
    pub async fn get_brent(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), "monthly".to_string());

        let response = self.get(AlphaVantageEndpoint::Brent, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get natural gas prices (monthly)
    pub async fn get_natural_gas(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), "monthly".to_string());

        let response = self.get(AlphaVantageEndpoint::NaturalGas, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }

    /// Get copper prices (monthly)
    pub async fn get_copper(&self) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let mut params = HashMap::new();
        params.insert("interval".to_string(), "monthly".to_string());

        let response = self.get(AlphaVantageEndpoint::Copper, params).await?;
        AlphaVantageParser::parse_economic_data(&response)
    }
}
