//! Twelvedata connector implementation
//!
//! DATA PROVIDER ONLY - no trading capabilities.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use serde_json::Value;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Twelvedata connector
///
/// Multi-asset data provider supporting stocks, forex, crypto, ETFs, commodities, and indices.
/// This is a DATA PROVIDER ONLY - no trading/order execution capabilities.
pub struct TwelvedataConnector {
    client: Client,
    auth: TwelvedataAuth,
    urls: TwelvedataUrls,
}

impl TwelvedataConnector {
    /// Create new connector with API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: TwelvedataAuth::new(api_key),
            urls: TwelvedataUrls::default(),
        }
    }

    /// Create connector from environment variable (TWELVEDATA_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: TwelvedataAuth::from_env(),
            urls: TwelvedataUrls::default(),
        }
    }

    /// Create connector with demo API key (for testing only)
    ///
    /// WARNING: Demo key has severe rate limits and limited functionality.
    /// Use only for initial testing.
    pub fn demo() -> Self {
        Self {
            client: Client::new(),
            auth: TwelvedataAuth::demo(),
            urls: TwelvedataUrls::default(),
        }
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: TwelvedataEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let url = format!("{}{}", self.urls.rest, endpoint.path());

        // Add API key to headers (recommended method)
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        // If no auth in headers, try query param (fallback)
        if !self.auth.has_credentials() || headers.is_empty() {
            self.auth.add_query_param(&mut params);
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

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        // Check status code
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return match status.as_u16() {
                401 => Err(ExchangeError::Auth("Invalid API key".to_string())),
                403 => Err(ExchangeError::PermissionDenied(
                    "Endpoint requires higher tier plan".to_string(),
                )),
                429 => Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: "Rate limit exceeded".to_string(),
                }),
                _ => Err(ExchangeError::Http(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    error_text
                ))),
            };
        }

        // Parse JSON response
        response
            .json::<Value>()
            .await
            .map_err(|e| ExchangeError::Parse(e.to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Real-time & Complex Data
    // ═══════════════════════════════════════════════════════════════════════════

    /// Real-time price — `GET /price`
    ///
    /// Simpler than `Quote` — returns only the current price for one or more symbols.
    /// `symbol` can be a comma-separated list (e.g. `"AAPL,MSFT,GOOGL"`).
    pub async fn get_realtime_price(&self, symbol: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        self.get(TwelvedataEndpoint::RealTimePrice, params).await
    }

    /// Complex data — `GET /complex_data`
    ///
    /// Batch multiple instruments and/or technical indicators in a single API call.
    /// `params` should include at minimum `symbols` and optionally `intervals`, indicators, etc.
    pub async fn get_complex_data(&self, params: HashMap<String, String>) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::ComplexData, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Fund Reference Data
    // ═══════════════════════════════════════════════════════════════════════════

    /// Mutual funds list — `GET /mutual_funds/list`
    ///
    /// Optional params: `symbol`, `exchange`, `country`, `fund_type`, `show_plan`, `country_fund`.
    pub async fn get_mutual_funds_list(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::MutualFundsList, params).await
    }

    /// Bonds list — `GET /bonds/list`
    ///
    /// Optional params: `symbol`, `exchange`, `country`, `type`, `bond_type`.
    pub async fn get_bonds_list(
        &self,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::BondsList, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ExchangeIdentity for TwelvedataConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Twelvedata
    }

    fn exchange_name(&self) -> &'static str {
        "Twelvedata"
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DataProvider
    }

    fn is_testnet(&self) -> bool {
        self.auth.is_demo()
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Twelvedata is a data provider - account type doesn't apply
        // But we return Spot to maintain compatibility
        vec![AccountType::Spot]
    }
}

#[async_trait]
impl MarketData for TwelvedataConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));

        let response = self.get(TwelvedataEndpoint::Price, params).await?;
        TwelvedataParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Twelvedata is a stocks data provider - no orderbook depth available
        Err(ExchangeError::UnsupportedOperation(
            "Orderbook not available from Twelvedata (stocks/data provider)".to_string(),
        ))
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));
        params.insert("interval".to_string(), map_interval(interval).to_string());

        if let Some(outputsize) = limit {
            params.insert("outputsize".to_string(), outputsize.to_string());
        }

        let response = self.get(TwelvedataEndpoint::TimeSeries, params).await?;
        TwelvedataParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));

        let response = self.get(TwelvedataEndpoint::Quote, params).await?;
        TwelvedataParser::parse_ticker(&response, &symbol.to_string())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Test connection with a simple price request for a known symbol
        let symbol = Symbol::new("AAPL", "USD");
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol));

        self.get(TwelvedataEndpoint::Price, params).await?;
        Ok(())
    }

    /// Get exchange info — returns list of US stocks from Twelvedata
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("country".to_string(), "United States".to_string());
        params.insert("show_plan".to_string(), "false".to_string());

        let response = self.get(TwelvedataEndpoint::Stocks, params).await?;

        // Response: {"data": [{symbol, name, currency, exchange, ...}], "status": "ok"}
        let data = response.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing data array in stocks response".to_string()))?;

        let infos = data.iter().filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let currency = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_uppercase();

            Some(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: currency,
                status: "TRADING".to_string(),
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
                step_size: Some(1.0),
                min_notional: None,
            })
        }).collect();

        Ok(infos)
    }
}

#[async_trait]
impl Trading for TwelvedataConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no trading capabilities".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no trading capabilities".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no trading capabilities".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no trading capabilities".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no trading capabilities".to_string()
        ))
    }
}

#[async_trait]
impl Account for TwelvedataConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no account/balance information".to_string(),
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no account information".to_string(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no account/balance information".to_string()
        ))
    }
}

#[async_trait]
impl Positions for TwelvedataConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no positions (no trading)".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no positions (no trading)".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Twelvedata is a data provider - no positions (no trading)".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Provider-specific)
// ═══════════════════════════════════════════════════════════════════════════════

impl TwelvedataConnector {
    /// Search symbols across all asset classes
    ///
    /// Returns list of matching symbols with metadata.
    pub async fn symbol_search(&self, query: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), query.to_string());

        self.get(TwelvedataEndpoint::SymbolSearch, params).await
    }

    /// Get list of all available stocks
    pub async fn get_stocks(&self) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::Stocks, HashMap::new())
            .await
    }

    /// Get list of all forex pairs
    pub async fn get_forex_pairs(&self) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::ForexPairs, HashMap::new())
            .await
    }

    /// Get list of all cryptocurrencies
    pub async fn get_cryptocurrencies(&self) -> ExchangeResult<Value> {
        self.get(TwelvedataEndpoint::Cryptocurrencies, HashMap::new())
            .await
    }

    /// Get market state (open/closed) for an exchange
    pub async fn market_state(&self, exchange: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("exchange".to_string(), exchange.to_string());

        self.get(TwelvedataEndpoint::MarketState, params).await
    }

    /// Get technical indicator: RSI
    pub async fn rsi(
        &self,
        symbol: &Symbol,
        interval: &str,
        time_period: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("interval".to_string(), map_interval(interval).to_string());
        params.insert("time_period".to_string(), time_period.to_string());

        self.get(TwelvedataEndpoint::Rsi, params).await
    }

    /// Get technical indicator: MACD
    pub async fn macd(&self, symbol: &Symbol, interval: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(symbol));
        params.insert("interval".to_string(), map_interval(interval).to_string());

        self.get(TwelvedataEndpoint::Macd, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_connector() {
        let connector = TwelvedataConnector::new("test_api_key");
        assert_eq!(connector.exchange_name(), "Twelvedata");
        assert_eq!(connector.exchange_type(), ExchangeType::DataProvider);
    }

    #[test]
    fn test_demo_connector() {
        let connector = TwelvedataConnector::demo();
        assert!(connector.is_testnet());
    }

    #[tokio::test]
    async fn test_trading_unsupported() {
        let connector = TwelvedataConnector::demo();
        let symbol = Symbol::new("AAPL", "USD");

        let result = connector
            .market_order(symbol, OrderSide::Buy, 1.0, AccountType::Spot)
            .await;

        assert!(matches!(
            result,
            Err(ExchangeError::UnsupportedOperation(_))
        ));
    }
}
