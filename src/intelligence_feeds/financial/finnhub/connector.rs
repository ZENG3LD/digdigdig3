//! Finnhub connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Finnhub API connector
///
/// Provides access to real-time stock data, market news, and financial information.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::finnhub::FinnhubConnector;
///
/// let connector = FinnhubConnector::from_env();
///
/// // Get real-time quote
/// let quote = connector.get_quote("AAPL").await?;
///
/// // Get historical candles
/// let candles = connector.get_candles("AAPL", "D", 1640000000, 1650000000).await?;
///
/// // Search for symbols
/// let results = connector.search_symbols("Apple").await?;
/// ```
pub struct FinnhubConnector {
    client: Client,
    auth: FinnhubAuth,
    endpoints: FinnhubEndpoints,
    _testnet: bool,
}

impl FinnhubConnector {
    /// Create new Finnhub connector with authentication
    pub fn new(auth: FinnhubAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: FinnhubEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `FINNHUB_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(FinnhubAuth::from_env())
    }

    /// Internal: Make GET request to Finnhub API
    async fn get(
        &self,
        endpoint: FinnhubEndpoint,
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
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for Finnhub API errors
        FinnhubParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STOCK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get real-time quote data for a symbol
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
    ///
    /// # Returns
    /// Quote with current price, change, high, low, etc.
    pub async fn get_quote(&self, symbol: &str) -> ExchangeResult<Quote> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        let response = self.get(FinnhubEndpoint::Quote, params).await?;
        FinnhubParser::parse_quote(&response)
    }

    /// Get candles/OHLC data for a symbol
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    /// - `resolution` - Time resolution: "1", "5", "15", "30", "60", "D", "W", "M"
    /// - `from` - UNIX timestamp (seconds)
    /// - `to` - UNIX timestamp (seconds)
    ///
    /// # Returns
    /// Vector of candles with OHLC data
    pub async fn get_candles(
        &self,
        symbol: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> ExchangeResult<Vec<Candle>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("resolution".to_string(), resolution.to_string());
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        let response = self.get(FinnhubEndpoint::StockCandles, params).await?;
        FinnhubParser::parse_candles(&response)
    }

    /// Search for symbols by query
    ///
    /// # Arguments
    /// - `query` - Search query (e.g., "Apple", "AAPL")
    ///
    /// # Returns
    /// Vector of matching symbols with descriptions
    pub async fn search_symbols(&self, query: &str) -> ExchangeResult<Vec<SearchResult>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        let response = self.get(FinnhubEndpoint::SymbolSearch, params).await?;
        FinnhubParser::parse_search_results(&response)
    }

    /// Get company profile information
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Company profile with name, industry, market cap, etc.
    pub async fn get_company_profile(&self, symbol: &str) -> ExchangeResult<CompanyProfile> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        let response = self.get(FinnhubEndpoint::CompanyProfile, params).await?;
        FinnhubParser::parse_company_profile(&response)
    }

    /// Get company peers (similar companies)
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Vector of peer symbols
    pub async fn get_peers(&self, symbol: &str) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        let response = self.get(FinnhubEndpoint::CompanyPeers, params).await?;
        FinnhubParser::parse_peers(&response)
    }

    /// Get financial statements
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    /// - `statement` - Statement type: "bs" (balance sheet), "ic" (income), "cf" (cash flow)
    /// - `freq` - Frequency: "annual" or "quarterly"
    ///
    /// # Returns
    /// Raw JSON with financial data
    pub async fn get_financials(
        &self,
        symbol: &str,
        statement: &str,
        freq: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("statement".to_string(), statement.to_string());
        params.insert("freq".to_string(), freq.to_string());

        self.get(FinnhubEndpoint::Financials, params).await
    }

    /// Get basic financial metrics
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    /// - `metric` - Metric type: "all" or specific metric
    ///
    /// # Returns
    /// Raw JSON with financial metrics
    pub async fn get_basic_financials(
        &self,
        symbol: &str,
        metric: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("metric".to_string(), metric.to_string());

        self.get(FinnhubEndpoint::BasicFinancials, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get market news by category
    ///
    /// # Arguments
    /// - `category` - News category: "general", "forex", "crypto", "merger"
    ///
    /// # Returns
    /// Vector of news articles
    pub async fn get_market_news(&self, category: &str) -> ExchangeResult<Vec<NewsArticle>> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), category.to_string());

        let response = self.get(FinnhubEndpoint::MarketNews, params).await?;
        FinnhubParser::parse_news(&response)
    }

    /// Get company-specific news
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    /// - `from` - Start date (YYYY-MM-DD)
    /// - `to` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of news articles
    pub async fn get_company_news(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
    ) -> ExchangeResult<Vec<NewsArticle>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        let response = self.get(FinnhubEndpoint::CompanyNews, params).await?;
        FinnhubParser::parse_news(&response)
    }

    /// Get market status for an exchange
    ///
    /// # Arguments
    /// - `exchange` - Exchange code (e.g., "US", "UK")
    ///
    /// # Returns
    /// Raw JSON with market status
    pub async fn get_market_status(&self, exchange: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), exchange.to_string());

        self.get(FinnhubEndpoint::MarketStatus, params).await
    }

    /// Get earnings calendar
    ///
    /// # Arguments
    /// - `from` - Start date (YYYY-MM-DD)
    /// - `to` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Raw JSON with earnings calendar
    pub async fn get_earnings_calendar(
        &self,
        from: &str,
        to: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        self.get(FinnhubEndpoint::EarningsCalendar, params).await
    }

    /// Get IPO calendar
    ///
    /// # Arguments
    /// - `from` - Start date (YYYY-MM-DD)
    /// - `to` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Raw JSON with IPO calendar
    pub async fn get_ipo_calendar(&self, from: &str, to: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        self.get(FinnhubEndpoint::IpoCalendar, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FOREX ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get forex exchange rates
    ///
    /// # Arguments
    /// - `base` - Base currency (e.g., "USD")
    ///
    /// # Returns
    /// Raw JSON with exchange rates
    pub async fn get_forex_rates(&self, base: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("base".to_string(), base.to_string());

        self.get(FinnhubEndpoint::ForexRates, params).await
    }

    /// Get forex candles
    ///
    /// # Arguments
    /// - `symbol` - Forex pair (e.g., "OANDA:EUR_USD")
    /// - `resolution` - Time resolution: "1", "5", "15", "30", "60", "D", "W", "M"
    /// - `from` - UNIX timestamp (seconds)
    /// - `to` - UNIX timestamp (seconds)
    ///
    /// # Returns
    /// Vector of candles with OHLC data
    pub async fn get_forex_candles(
        &self,
        symbol: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> ExchangeResult<Vec<Candle>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("resolution".to_string(), resolution.to_string());
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        let response = self.get(FinnhubEndpoint::ForexCandles, params).await?;
        FinnhubParser::parse_candles(&response)
    }

    /// Get forex symbols for an exchange
    ///
    /// # Arguments
    /// - `exchange` - Exchange name (e.g., "oanda", "fxcm")
    ///
    /// # Returns
    /// Raw JSON with forex symbols
    pub async fn get_forex_symbols(&self, exchange: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), exchange.to_string());

        self.get(FinnhubEndpoint::ForexSymbols, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CRYPTO ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get crypto candles
    ///
    /// # Arguments
    /// - `symbol` - Crypto pair (e.g., "BINANCE:BTCUSDT")
    /// - `resolution` - Time resolution: "1", "5", "15", "30", "60", "D", "W", "M"
    /// - `from` - UNIX timestamp (seconds)
    /// - `to` - UNIX timestamp (seconds)
    ///
    /// # Returns
    /// Vector of candles with OHLC data
    pub async fn get_crypto_candles(
        &self,
        symbol: &str,
        resolution: &str,
        from: i64,
        to: i64,
    ) -> ExchangeResult<Vec<Candle>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("resolution".to_string(), resolution.to_string());
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());

        let response = self.get(FinnhubEndpoint::CryptoCandles, params).await?;
        FinnhubParser::parse_candles(&response)
    }

    /// Get crypto symbols for an exchange
    ///
    /// # Arguments
    /// - `exchange` - Exchange name (e.g., "binance", "coinbase")
    ///
    /// # Returns
    /// Raw JSON with crypto symbols
    pub async fn get_crypto_symbols(&self, exchange: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), exchange.to_string());

        self.get(FinnhubEndpoint::CryptoSymbols, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECONOMIC ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get economic calendar
    ///
    /// # Returns
    /// Raw JSON with economic events
    pub async fn get_economic_calendar(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(FinnhubEndpoint::EconomicCalendar, params).await
    }

    /// Get list of countries
    ///
    /// # Returns
    /// Raw JSON with country codes
    pub async fn get_country_list(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(FinnhubEndpoint::CountryList, params).await
    }

    /// Get economic data by code
    ///
    /// # Arguments
    /// - `code` - Economic indicator code
    ///
    /// # Returns
    /// Raw JSON with economic data
    pub async fn get_economic_data(&self, code: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("code".to_string(), code.to_string());

        self.get(FinnhubEndpoint::EconomicData, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SENTIMENT ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get social sentiment for a stock
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Raw JSON with social sentiment data
    pub async fn get_social_sentiment(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        self.get(FinnhubEndpoint::SocialSentiment, params).await
    }

    /// Get insider transactions
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Raw JSON with insider transaction data
    pub async fn get_insider_transactions(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        self.get(FinnhubEndpoint::InsiderTransactions, params).await
    }

    /// Get insider sentiment
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Raw JSON with insider sentiment data
    pub async fn get_insider_sentiment(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        self.get(FinnhubEndpoint::InsiderSentiment, params).await
    }

    /// Get analyst recommendation trends
    ///
    /// # Arguments
    /// - `symbol` - Stock symbol (e.g., "AAPL")
    ///
    /// # Returns
    /// Raw JSON with recommendation trends
    pub async fn get_recommendation_trends(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));

        self.get(FinnhubEndpoint::RecommendationTrends, params).await
    }
}
