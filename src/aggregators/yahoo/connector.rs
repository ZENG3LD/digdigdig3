//! Yahoo Finance connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Yahoo Finance connector
pub struct YahooFinanceConnector {
    client: Client,
    auth: YahooFinanceAuth,
    urls: YahooFinanceUrls,
}

impl YahooFinanceConnector {
    /// Create new connector (no authentication needed for most endpoints)
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: YahooFinanceAuth::new(),
            urls: YahooFinanceUrls::default(),
        }
    }

    /// Create connector with authentication for historical downloads
    pub fn with_auth(auth: YahooFinanceAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            urls: YahooFinanceUrls::default(),
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: YahooFinanceAuth::from_env(),
            urls: YahooFinanceUrls::default(),
        }
    }

    /// Get mutable reference to auth (for updating cookie/crumb)
    pub fn auth_mut(&mut self) -> &mut YahooFinanceAuth {
        &mut self.auth
    }

    /// Obtain crumb from Yahoo Finance
    ///
    /// This requires visiting Yahoo Finance first to get a valid cookie.
    /// Call this method after setting a cookie via `auth_mut().set_cookie(...)`.
    pub async fn obtain_crumb(&mut self) -> ExchangeResult<String> {
        if self.auth.cookie.is_none() {
            return Err(ExchangeError::Auth(
                "Cookie required to obtain crumb. Visit https://finance.yahoo.com first.".to_string()
            ));
        }

        let url = YahooFinanceEndpoint::GetCrumb.url(self.urls.rest_base, None);
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to get crumb: {}", e)))?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            return Err(ExchangeError::Api {
                code: status_code,
                message: format!("Failed to get crumb: HTTP {}", status_code)
            });
        }

        let crumb_text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read crumb: {}", e)))?;

        let crumb = YahooFinanceParser::parse_crumb(&crumb_text)?;
        self.auth.set_crumb(&crumb);

        Ok(crumb)
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: YahooFinanceEndpoint,
        symbol: Option<&str>,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = endpoint.url(self.urls.rest_base, symbol);

        // Add authentication headers and query params
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        if endpoint.requires_crumb() {
            self.auth.sign_query(&mut params);
        }

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check for rate limiting
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ExchangeError::RateLimit);
        }

        if !response.status().is_success() {
            let status = response.status();
            let status_code = status.as_u16() as i32;
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status_code,
                message: format!("HTTP {} - {}", status, body)
            });
        }

        let json = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors in response
        YahooFinanceParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Get quote for a symbol
    ///
    /// Uses chart endpoint instead of quote endpoint (quote returns 401 as of Jan 2026).
    /// Chart endpoint provides the same data in response.chart.result[0].meta
    async fn get_quote_internal(&self, yahoo_symbol: &str) -> ExchangeResult<serde_json::Value> {
        // Use chart endpoint instead of quote endpoint (quote endpoint returns 401)
        // Chart response includes current price in meta.regularMarketPrice
        self.get(YahooFinanceEndpoint::Chart, Some(yahoo_symbol), HashMap::new()).await
    }
}

impl Default for YahooFinanceConnector {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for YahooFinanceConnector {
    fn exchange_name(&self) -> &'static str {
        "yahoo_finance"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::YahooFinance
    }

    fn is_testnet(&self) -> bool {
        false // Yahoo Finance has no testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Yahoo Finance is a data provider only (treat as Spot for compatibility)
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement what makes sense)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for YahooFinanceConnector {
    /// Get current price
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let yahoo_symbol = format_symbol(&symbol.base, &symbol.quote);
        let response = self.get_quote_internal(&yahoo_symbol).await?;
        YahooFinanceParser::parse_price(&response)
    }

    /// Get ticker (24h stats)
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let yahoo_symbol = format_symbol(&symbol.base, &symbol.quote);
        let response = self.get_quote_internal(&yahoo_symbol).await?;
        YahooFinanceParser::parse_ticker(&response, &yahoo_symbol)
    }

    /// Get orderbook
    ///
    /// Yahoo Finance does NOT provide orderbook data.
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance does not provide orderbook data - data feed only".to_string(),
        ))
    }

    /// Get klines/candles
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let yahoo_symbol = format_symbol(&symbol.base, &symbol.quote);
        let yahoo_interval = map_chart_interval(interval);

        let mut params = HashMap::new();
        params.insert("interval".to_string(), yahoo_interval.to_string());

        // Use range parameter for simplicity (Yahoo prefers this over period1/period2 for recent data)
        if let Some(lim) = limit {
            // Map limit to range
            let range = match interval {
                "1m" | "2m" | "5m" => format!("{}d", (lim as f64 / 390.0).ceil()), // ~390 1m candles per day
                "15m" => format!("{}d", (lim as f64 / 26.0).ceil()),                // ~26 15m candles per day
                "30m" => format!("{}d", (lim as f64 / 13.0).ceil()),                // ~13 30m candles per day
                "1h" => format!("{}d", (lim as f64 / 6.5).ceil()),                  // ~6.5 1h candles per day
                "1d" => format!("{}d", lim),
                "1wk" => format!("{}mo", (lim as f64 / 4.0).ceil()),
                "1mo" => format!("{}mo", lim),
                _ => format!("{}d", lim),
            };
            params.insert("range".to_string(), range);
        } else {
            // Default range
            params.insert("range".to_string(), "1mo".to_string());
        }

        let response = self
            .get(
                YahooFinanceEndpoint::Chart,
                Some(&yahoo_symbol),
                params,
            )
            .await?;

        YahooFinanceParser::parse_klines(&response)
    }

    /// Ping (check connection)
    async fn ping(&self) -> ExchangeResult<()> {
        // Yahoo Finance doesn't have a dedicated ping endpoint
        // Try to get market summary as a health check
        self.get(YahooFinanceEndpoint::MarketSummary, None, HashMap::new())
            .await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for YahooFinanceConnector {
    async fn market_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - trading not supported".to_string(),
        ))
    }

    async fn limit_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - trading not supported".to_string(),
        ))
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - trading not supported".to_string(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - trading not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for YahooFinanceConnector {
    async fn get_balance(&self, _asset: Option<Asset>, _account_type: AccountType) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - account operations not supported".to_string(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - account operations not supported".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (UnsupportedOperation - data provider only)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for YahooFinanceConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance is a data provider - position tracking not supported".to_string(),
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - Yahoo Finance is not a derivatives platform".to_string(),
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Leverage not applicable - Yahoo Finance is a data provider".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Yahoo-specific, not from traits)
// ═══════════════════════════════════════════════════════════════════════════

impl YahooFinanceConnector {
    /// Get market summary (major indices)
    pub async fn get_market_summary(&self) -> ExchangeResult<serde_json::Value> {
        self.get(YahooFinanceEndpoint::MarketSummary, None, HashMap::new())
            .await
    }

    /// Search for symbols
    ///
    /// # Parameters
    /// - `query`: Search query (e.g., "apple", "btc")
    /// - `quotes_count`: Max number of quotes to return (default: 10)
    pub async fn search_symbols(
        &self,
        query: &str,
        quotes_count: Option<u16>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert(
            "quotesCount".to_string(),
            quotes_count.unwrap_or(10).to_string(),
        );
        params.insert("enableFuzzyQuery".to_string(), "true".to_string());

        self.get(YahooFinanceEndpoint::Search, None, params).await
    }

    /// Get quote summary with specific modules
    ///
    /// # Parameters
    /// - `symbol`: Yahoo symbol (e.g., "AAPL", "BTC-USD")
    /// - `modules`: Comma-separated module names (see `quote_summary_modules`)
    ///
    /// # Example
    /// ```ignore
    /// let data = connector.get_quote_summary("AAPL", "assetProfile,financialData").await?;
    /// ```
    pub async fn get_quote_summary(
        &self,
        symbol: &str,
        modules: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("modules".to_string(), modules.to_string());

        self.get(YahooFinanceEndpoint::QuoteSummary, Some(symbol), params)
            .await
    }

    /// Get asset profile (company information)
    pub async fn get_asset_profile(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        self.get_quote_summary(symbol, quote_summary_modules::ASSET_PROFILE)
            .await
    }

    /// Get financial data (key metrics)
    pub async fn get_financial_data(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        self.get_quote_summary(symbol, quote_summary_modules::FINANCIAL_DATA)
            .await
    }

    /// Get earnings data
    pub async fn get_earnings(&self, symbol: &str) -> ExchangeResult<serde_json::Value> {
        self.get_quote_summary(symbol, quote_summary_modules::EARNINGS)
            .await
    }

    /// Get options chain
    ///
    /// # Parameters
    /// - `symbol`: Underlying symbol (e.g., "AAPL")
    /// - `expiration_date`: Optional Unix timestamp for specific expiration
    pub async fn get_options_chain(
        &self,
        symbol: &str,
        expiration_date: Option<i64>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(date) = expiration_date {
            params.insert("date".to_string(), date.to_string());
        }

        self.get(YahooFinanceEndpoint::Options, Some(symbol), params)
            .await
    }

    /// Download historical data as CSV (requires authentication)
    ///
    /// This endpoint requires cookie and crumb authentication.
    /// Use `obtain_crumb()` first to set up authentication.
    pub async fn download_history_csv(
        &self,
        symbol: &str,
        period1: i64,
        period2: i64,
        interval: &str,
    ) -> ExchangeResult<String> {
        if !self.auth.has_download_auth() {
            return Err(ExchangeError::Auth(
                "Cookie and crumb required for historical download".to_string(),
            ));
        }

        let mut params = HashMap::new();
        params.insert("period1".to_string(), period1.to_string());
        params.insert("period2".to_string(), period2.to_string());
        params.insert("interval".to_string(), interval.to_string());
        params.insert("events".to_string(), "history".to_string());

        // Add crumb to query params
        self.auth.sign_query(&mut params);

        let url = YahooFinanceEndpoint::DownloadHistory.url(self.urls.rest_base, Some(symbol));

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        request = request.query(&params);

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            return Err(ExchangeError::Api {
                code: status_code,
                message: format!("HTTP {} - download failed", status_code)
            });
        }

        response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read CSV: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_creation() {
        let connector = YahooFinanceConnector::new();
        assert_eq!(connector.exchange_name(), "yahoo_finance");
        assert_eq!(connector.exchange_id(), ExchangeId::YahooFinance);
    }

    #[test]
    fn test_supported_account_types() {
        let connector = YahooFinanceConnector::new();
        let types = connector.supported_account_types();
        assert_eq!(types, vec![AccountType::Spot]);
    }
}
