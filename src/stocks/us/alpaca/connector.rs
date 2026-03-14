//! Alpaca connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use serde_json::{json, Value};

use crate::core::types::*;
use crate::core::traits::*;
use crate::core::utils::precision::PrecisionCache;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

// ─────────────────────────────────────────────────────────────────────────────
// Helper: map TimeInForce → Alpaca time_in_force string
// ─────────────────────────────────────────────────────────────────────────────

fn tif_str(tif: TimeInForce) -> &'static str {
    match tif {
        TimeInForce::Gtc => "gtc",
        TimeInForce::Ioc => "ioc",
        TimeInForce::Fok => "fok",
        TimeInForce::PostOnly => "gtc", // Alpaca has no post-only TIF — caller should not reach here
        TimeInForce::Gtd { .. } => "gtc", // Unsupported, caller guards this
        TimeInForce::GoodTilBlock { .. } => "gtc",
    }
}

/// Alpaca connector
///
/// Supports both market data and trading operations for US stocks.
pub struct AlpacaConnector {
    client: Client,
    auth: AlpacaAuth,
    endpoints: AlpacaEndpoints,
    testnet: bool,
    feed: DataFeed,
    precision: PrecisionCache,
}

/// Data feed selection
#[derive(Debug, Clone, Copy, Default)]
pub enum DataFeed {
    /// IEX exchange only (~2.5% market volume) - FREE
    #[default]
    Iex,
    /// All US exchanges (100% market volume) - PAID
    Sip,
}

impl AlpacaConnector {
    /// Create new connector (paper trading by default)
    pub fn new(auth: AlpacaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AlpacaEndpoints::paper(),
            testnet: true, // Paper trading is testnet
            feed: DataFeed::Iex,
            precision: PrecisionCache::new(),
        }
    }

    /// Create connector with custom environment
    pub fn with_env(auth: AlpacaAuth, live: bool) -> Self {
        let endpoints = if live {
            AlpacaEndpoints::live()
        } else {
            AlpacaEndpoints::paper()
        };

        Self {
            client: Client::new(),
            auth,
            endpoints,
            testnet: !live,
            feed: DataFeed::Iex,
            precision: PrecisionCache::new(),
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(AlpacaAuth::from_env())
    }

    /// Create public crypto-only connector (no API keys required)
    ///
    /// This connector can access crypto market data without authentication.
    /// All crypto endpoints on Alpaca work without API keys.
    ///
    /// Limitations:
    /// - Only crypto symbols work (e.g., BTC/USD, ETH/USD)
    /// - Stock data, trading, and account operations require auth
    pub fn crypto_only() -> Self {
        Self {
            client: Client::new(),
            auth: AlpacaAuth::none(),
            endpoints: AlpacaEndpoints::live(),
            testnet: false,
            feed: DataFeed::Iex,
            precision: PrecisionCache::new(),
        }
    }

    /// Set data feed (IEX free vs SIP paid)
    pub fn with_feed(mut self, feed: DataFeed) -> Self {
        self.feed = feed;
        self
    }

    /// Internal: Make GET request to Trading API
    async fn get_trading(
        &self,
        endpoint: AlpacaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.trading_base, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Internal: Make POST request to Trading API
    async fn post_trading(
        &self,
        endpoint: AlpacaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.trading_base, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add JSON body
        request = request.json(&body);

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Internal: Make DELETE request to Trading API
    async fn delete_trading(
        &self,
        endpoint: AlpacaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.trading_base, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.delete(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Internal: DELETE that accepts 207 Multi-Status (cancel-all returns 207)
    async fn delete_trading_multi(
        &self,
        endpoint: AlpacaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.trading_base, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.delete(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        // 200 OK, 207 Multi-Status are both acceptable for cancel-all
        if !status.is_success() && status.as_u16() != 207 {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        // Body may be empty on full success
        let text = response.text().await
            .map_err(|e| ExchangeError::Network(format!("Response read failed: {}", e)))?;

        if text.is_empty() {
            Ok(Value::Array(vec![]))
        } else {
            serde_json::from_str(&text)
                .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
        }
    }

    /// Internal: PATCH request to Trading API (amend order)
    async fn patch_trading(
        &self,
        endpoint: AlpacaEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.trading_base, path);

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.patch(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        request = request.json(&body);

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Internal: Make GET request to Market Data API
    async fn get_market_data(
        &self,
        endpoint: AlpacaEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let (path, _) = endpoint.path();
        let url = format!("{}{}", self.endpoints.market_data_base, path);

        // Add feed parameter for stock data (not needed for crypto)
        if !path.contains("/crypto/") {
            let feed_str = match self.feed {
                DataFeed::Iex => "iex",
                DataFeed::Sip => "sip",
            };
            params.insert("feed".to_string(), feed_str.to_string());
        }

        let mut headers = HashMap::new();
        // Only add auth headers if we have credentials
        if self.auth.has_credentials() {
            self.auth.sign_headers(&mut headers);
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

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: error_text,
            });
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Watchlists
    // ═══════════════════════════════════════════════════════════════════

    /// List all watchlists — `GET /v2/watchlists`
    pub async fn get_watchlists(&self) -> ExchangeResult<Value> {
        self.get_trading(AlpacaEndpoint::Watchlists, HashMap::new()).await
    }

    /// Create watchlist — `POST /v2/watchlists`
    ///
    /// `name` is the unique watchlist name; `symbols` is an optional list of tickers.
    pub async fn create_watchlist(&self, name: &str, symbols: Vec<&str>) -> ExchangeResult<Value> {
        let body = json!({
            "name": name,
            "symbols": symbols,
        });
        self.post_trading(AlpacaEndpoint::Watchlists, body).await
    }

    /// Get watchlist by ID — `GET /v2/watchlists/{id}`
    pub async fn get_watchlist(&self, id: &str) -> ExchangeResult<Value> {
        self.get_trading(AlpacaEndpoint::WatchlistById(id.to_string()), HashMap::new()).await
    }

    /// Update watchlist — `PUT /v2/watchlists/{id}`
    ///
    /// Replaces the name and/or symbols list of an existing watchlist.
    pub async fn update_watchlist(
        &self,
        id: &str,
        name: Option<&str>,
        symbols: Option<Vec<&str>>,
    ) -> ExchangeResult<Value> {
        let mut body = json!({});
        if let Some(n) = name {
            body["name"] = json!(n);
        }
        if let Some(s) = symbols {
            body["symbols"] = json!(s);
        }
        let (path, _) = AlpacaEndpoint::WatchlistById(id.to_string()).path();
        let url = format!("{}{}", self.endpoints.trading_base, path);
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        let mut request = self.client.put(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        request = request.json(&body);
        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api { code: status.as_u16() as i32, message: error_text });
        }
        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Delete watchlist — `DELETE /v2/watchlists/{id}`
    pub async fn delete_watchlist(&self, id: &str) -> ExchangeResult<Value> {
        self.delete_trading(AlpacaEndpoint::WatchlistById(id.to_string()), HashMap::new()).await
    }

    /// Add symbol to watchlist — `POST /v2/watchlists/{id}`
    pub async fn add_symbol_to_watchlist(&self, id: &str, symbol: &str) -> ExchangeResult<Value> {
        let body = json!({ "symbol": symbol });
        self.post_trading(AlpacaEndpoint::WatchlistAddSymbol(id.to_string()), body).await
    }

    // ═══════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Positions
    // ═══════════════════════════════════════════════════════════════════

    /// Close all open positions — `DELETE /v2/positions`
    ///
    /// Pass `cancel_orders: true` to cancel open orders before liquidating.
    pub async fn close_all_positions(&self, cancel_orders: bool) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("cancel_orders".to_string(), cancel_orders.to_string());
        self.delete_trading_multi(AlpacaEndpoint::Positions, params).await
    }

    /// Close a single position — `DELETE /v2/positions/{symbol_or_asset_id}`
    ///
    /// Optionally pass `qty` or `percentage` to partially close.
    pub async fn close_position(
        &self,
        symbol_or_asset_id: &str,
        qty: Option<f64>,
        percentage: Option<f64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(q) = qty {
            params.insert("qty".to_string(), q.to_string());
        }
        if let Some(p) = percentage {
            params.insert("percentage".to_string(), p.to_string());
        }
        self.delete_trading(
            AlpacaEndpoint::PositionBySymbol(symbol_or_asset_id.to_string()),
            params,
        ).await
    }

    // ═══════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Account Configurations
    // ═══════════════════════════════════════════════════════════════════

    /// Get account configurations — `GET /v2/account/configurations`
    pub async fn get_account_configurations(&self) -> ExchangeResult<Value> {
        self.get_trading(AlpacaEndpoint::AccountConfigurations, HashMap::new()).await
    }

    /// Update account configurations — `PATCH /v2/account/configurations`
    ///
    /// Pass a JSON object with only the configuration fields to update.
    pub async fn patch_account_configurations(&self, config: Value) -> ExchangeResult<Value> {
        self.patch_trading(AlpacaEndpoint::AccountConfigurations, config).await
    }

    // ═══════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Options Chain
    // ═══════════════════════════════════════════════════════════════════

    /// Get options chain for a symbol — `GET /v1beta1/options/chain`
    ///
    /// `underlying_symbol` is required (e.g. `"AAPL"`).
    /// Optional `params`: `expiration_date`, `strike_price_gte`, `strike_price_lte`, `type`.
    pub async fn get_options_chain(
        &self,
        underlying_symbol: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let mut full_params = params;
        full_params.insert("underlying_symbol".to_string(), underlying_symbol.to_string());
        self.get_market_data(AlpacaEndpoint::OptionsChain, full_params).await
    }

    // ═══════════════════════════════════════════════════════════════════
    // EXTENDED METHODS — Screener
    // ═══════════════════════════════════════════════════════════════════

    /// Most active stocks screener — `GET /v1beta1/screener/most-actives`
    ///
    /// Optional `params`: `by` ("trades" or "volume"), `top` (count, default 10).
    pub async fn get_most_actives(&self, params: HashMap<String, String>) -> ExchangeResult<Value> {
        self.get_market_data(AlpacaEndpoint::MostActives, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for AlpacaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Alpaca
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Alpaca is primarily a stock broker - use Spot as the account type
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for AlpacaConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let symbol_str = format_symbol(&symbol);

        // Use snapshot endpoint to get latest price
        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());

        // Determine if this is a crypto symbol (has "/" in it)
        let is_crypto = symbol_str.contains('/');
        let endpoint = if is_crypto {
            AlpacaEndpoint::CryptoSnapshots
        } else {
            AlpacaEndpoint::StockSnapshots
        };

        let response = self.get_market_data(endpoint, params).await?;

        // For crypto, the response has a "snapshots" wrapper
        let snapshots = if is_crypto {
            response.get("snapshots")
                .ok_or_else(|| ExchangeError::Parse("Missing 'snapshots' field for crypto".to_string()))?
        } else {
            &response
        };

        // Extract snapshot for this symbol
        let snapshot = snapshots
            .get(&symbol_str)
            .ok_or_else(|| ExchangeError::Parse(format!("No snapshot for {}", symbol_str)))?;

        AlpacaParser::parse_price(snapshot)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Orderbook only available for crypto
        let symbol_str = format_symbol(&symbol);

        // Check if this is a crypto symbol (has "/" in it)
        if !symbol_str.contains('/') {
            return Err(ExchangeError::UnsupportedOperation(
                "Orderbook only available for crypto, not stocks".to_string()
            ));
        }

        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());

        let response = self.get_market_data(AlpacaEndpoint::CryptoOrderbooks, params).await?;

        AlpacaParser::parse_orderbook(&response, &symbol_str)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = format_symbol(&symbol);

        // Map interval to Alpaca format
        // Supported: 1Min, 5Min, 15Min, 30Min, 1Hour, 4Hour, 1Day, 1Week
        let timeframe = match interval {
            "1m" => "1Min",
            "5m" => "5Min",
            "15m" => "15Min",
            "30m" => "30Min",
            "1h" => "1Hour",
            "4h" => "4Hour",
            "1d" => "1Day",
            "1w" => "1Week",
            other => other, // Pass through as-is
        };

        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());
        params.insert("timeframe".to_string(), timeframe.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        } else {
            params.insert("limit".to_string(), "1000".to_string()); // Default limit
        }

        // Determine if this is a crypto symbol (has "/" in it)
        let is_crypto = symbol_str.contains('/');
        let endpoint = if is_crypto {
            AlpacaEndpoint::CryptoBars
        } else {
            AlpacaEndpoint::StockBars
        };

        let response = self.get_market_data(endpoint, params).await?;

        AlpacaParser::parse_klines(&response, &symbol_str)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol);

        // Use snapshot endpoint
        let mut params = HashMap::new();
        params.insert("symbols".to_string(), symbol_str.clone());

        // Determine if this is a crypto symbol (has "/" in it)
        let is_crypto = symbol_str.contains('/');
        let endpoint = if is_crypto {
            AlpacaEndpoint::CryptoSnapshots
        } else {
            AlpacaEndpoint::StockSnapshots
        };

        let response = self.get_market_data(endpoint, params).await?;

        // For crypto, the response has a "snapshots" wrapper
        let snapshots = if is_crypto {
            response.get("snapshots")
                .ok_or_else(|| ExchangeError::Parse("Missing 'snapshots' field for crypto".to_string()))?
        } else {
            &response
        };

        // Extract snapshot for this symbol
        let snapshot = snapshots
            .get(&symbol_str)
            .ok_or_else(|| ExchangeError::Parse(format!("No snapshot for {}", symbol_str)))?;

        AlpacaParser::parse_ticker(snapshot, &symbol_str)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // If no auth, use crypto endpoint which works without API keys
        if !self.auth.has_credentials() {
            let mut params = HashMap::new();
            params.insert("symbols".to_string(), "BTC/USD".to_string());
            self.get_market_data(AlpacaEndpoint::CryptoSnapshots, params).await?;
            Ok(())
        } else {
            // Use clock endpoint as ping for authenticated connections
            self.get_trading(AlpacaEndpoint::Clock, HashMap::new()).await?;
            Ok(())
        }
    }

    /// Get exchange info — returns list of active, tradable US equity assets
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "active".to_string());
        params.insert("tradable".to_string(), "true".to_string());
        params.insert("asset_class".to_string(), "us_equity".to_string());

        let response = self.get_trading(AlpacaEndpoint::Assets, params).await?;

        // Response is an array of asset objects
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of assets".to_string()))?;

        let infos: Vec<SymbolInfo> = arr.iter().filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let tradable = item.get("tradable").and_then(|v| v.as_bool()).unwrap_or(false);
            if !tradable {
                return None;
            }
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active")
                .to_uppercase();

            // Alpaca provides price_increment (tick size) and min_trade_increment (qty step)
            // Both are string-encoded floats in the assets response
            let tick_size = item.get("price_increment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|&v| v > 0.0);
            let step_size = item.get("min_trade_increment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|&v| v > 0.0)
                .or(Some(1.0));

            Some(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: "USD".to_string(),
                status,
                price_precision: 2,
                quantity_precision: 0,
                min_quantity: Some(1.0),
                max_quantity: None,
                tick_size,
                step_size,
                min_notional: None,
            })
        }).collect();

        self.precision.load_from_symbols(&infos);

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for AlpacaConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol_str = format_symbol(&req.symbol);
        let side_str = req.side.as_str().to_lowercase();
        let qty_str = self.precision.qty(&symbol_str, req.quantity);
        let client_oid = req.client_order_id.clone();

        // Build base body fields shared by most order types
        let mut base = serde_json::Map::new();
        base.insert("symbol".to_string(), json!(symbol_str));
        base.insert("qty".to_string(), json!(qty_str));
        base.insert("side".to_string(), json!(side_str));

        if let Some(coid) = client_oid {
            base.insert("client_order_id".to_string(), json!(coid));
        }

        let body: Value = match req.order_type {
            OrderType::Market => {
                base.insert("type".to_string(), json!("market"));
                base.insert("time_in_force".to_string(), json!("day"));
                Value::Object(base)
            }

            OrderType::Limit { price } => {
                let tif = tif_str(req.time_in_force);
                base.insert("type".to_string(), json!("limit"));
                base.insert("time_in_force".to_string(), json!(tif));
                base.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, price)));
                Value::Object(base)
            }

            OrderType::StopMarket { stop_price } => {
                let tif = tif_str(req.time_in_force);
                base.insert("type".to_string(), json!("stop"));
                base.insert("time_in_force".to_string(), json!(tif));
                base.insert("stop_price".to_string(), json!(self.precision.price(&symbol_str, stop_price)));
                Value::Object(base)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                let tif = tif_str(req.time_in_force);
                base.insert("type".to_string(), json!("stop_limit"));
                base.insert("time_in_force".to_string(), json!(tif));
                base.insert("stop_price".to_string(), json!(self.precision.price(&symbol_str, stop_price)));
                base.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, limit_price)));
                Value::Object(base)
            }

            OrderType::TrailingStop { callback_rate, activation_price: _ } => {
                // Alpaca accepts trail_percent (percentage offset).
                // activation_price has no Alpaca equivalent — ignored.
                base.insert("type".to_string(), json!("trailing_stop"));
                base.insert("time_in_force".to_string(), json!("gtc"));
                base.insert("trail_percent".to_string(), json!(callback_rate.to_string()));
                Value::Object(base)
            }

            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // Alpaca OCO: limit take-profit + stop(-limit) stop-loss, linked via order_class=oco
                // The side refers to the closing leg — entry must already be open.
                base.insert("type".to_string(), json!("limit"));
                base.insert("time_in_force".to_string(), json!("gtc"));
                base.insert("order_class".to_string(), json!("oco"));
                base.insert("take_profit".to_string(), json!({ "limit_price": self.precision.price(&symbol_str, price) }));
                let sl_obj = if let Some(slp) = stop_limit_price {
                    json!({ "stop_price": self.precision.price(&symbol_str, stop_price), "limit_price": self.precision.price(&symbol_str, slp) })
                } else {
                    json!({ "stop_price": self.precision.price(&symbol_str, stop_price) })
                };
                base.insert("stop_loss".to_string(), sl_obj);
                Value::Object(base)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // Entry type: market if price is None, limit otherwise
                if let Some(entry_price) = price {
                    base.insert("type".to_string(), json!("limit"));
                    base.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, entry_price)));
                } else {
                    base.insert("type".to_string(), json!("market"));
                }
                base.insert("time_in_force".to_string(), json!("gtc"));
                base.insert("order_class".to_string(), json!("bracket"));
                base.insert("take_profit".to_string(), json!({ "limit_price": self.precision.price(&symbol_str, take_profit) }));
                // Alpaca requires at minimum a stop_price in stop_loss object
                base.insert("stop_loss".to_string(), json!({ "stop_price": self.precision.price(&symbol_str, stop_loss) }));
                Value::Object(base)
            }

            OrderType::Ioc { price } => {
                base.insert("time_in_force".to_string(), json!("ioc"));
                if let Some(p) = price {
                    base.insert("type".to_string(), json!("limit"));
                    base.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, p)));
                } else {
                    base.insert("type".to_string(), json!("market"));
                }
                Value::Object(base)
            }

            OrderType::Fok { price } => {
                base.insert("type".to_string(), json!("limit"));
                base.insert("time_in_force".to_string(), json!("fok"));
                base.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, price)));
                Value::Object(base)
            }

            // Alpaca has no post-only flag for equities
            OrderType::PostOnly { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "PostOnly orders are not supported on Alpaca (US equities broker)".to_string()
                ));
            }

            // Alpaca does not support iceberg orders
            OrderType::Iceberg { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Iceberg orders are not supported on Alpaca".to_string()
                ));
            }

            // Alpaca does not support algorithmic TWAP
            OrderType::Twap { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "TWAP orders are not supported on Alpaca".to_string()
                ));
            }

            // Alpaca has no GTD — only day / gtc
            OrderType::Gtd { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "GTD orders are not supported on Alpaca (use GTC or day instead)".to_string()
                ));
            }

            // Alpaca is a stock broker — no futures, no reduce-only
            OrderType::ReduceOnly { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "ReduceOnly orders are not supported on Alpaca (stock broker, no futures)".to_string()
                ));
            }

            OrderType::Oto { .. } | OrderType::ConditionalPlan { .. } | OrderType::DcaRecurring { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Oto/ConditionalPlan/DcaRecurring orders are not supported on Alpaca".to_string()
                ));
            }
        };

        let response = self.post_trading(AlpacaEndpoint::Orders, body).await?;

        // Bracket and OCO orders return an order that embeds legs.
        // We parse the primary (outer) order; legs are accessible via nested field.
        AlpacaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let endpoint = AlpacaEndpoint::OrderById(order_id.clone());
                let response = self.delete_trading(endpoint, HashMap::new()).await?;
                AlpacaParser::parse_order(&response)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported directly on Trading trait — use CancelAll trait", req.scope)
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Alpaca lookup by order ID — symbol not required
        let endpoint = AlpacaEndpoint::OrderById(order_id.to_string());
        let response = self.get_trading(endpoint, HashMap::new()).await?;
        AlpacaParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "open".to_string());
        params.insert("nested".to_string(), "true".to_string());
        params.insert("limit".to_string(), "500".to_string());

        if let Some(sym) = symbol {
            // Alpaca uses "symbols" query param for filtering by ticker
            params.insert("symbols".to_string(), sym.to_uppercase());
        }

        let response = self.get_trading(AlpacaEndpoint::Orders, params).await?;
        AlpacaParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "closed".to_string());
        params.insert("nested".to_string(), "true".to_string());

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        } else {
            params.insert("limit".to_string(), "500".to_string());
        }

        if let Some(start_ms) = filter.start_time {
            // Alpaca expects RFC-3339 for "after" param
            let dt = chrono::DateTime::from_timestamp_millis(start_ms)
                .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH.into());
            params.insert("after".to_string(), dt.to_rfc3339());
        }

        if let Some(end_ms) = filter.end_time {
            let dt = chrono::DateTime::from_timestamp_millis(end_ms)
                .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH.into());
            params.insert("until".to_string(), dt.to_rfc3339());
        }

        if let Some(sym) = filter.symbol {
            params.insert("symbols".to_string(), format_symbol(&sym));
        }

        // direction=desc gives newest first
        params.insert("direction".to_string(), "desc".to_string());

        let response = self.get_trading(AlpacaEndpoint::Orders, params).await?;
        AlpacaParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for AlpacaConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let response = self.get_trading(AlpacaEndpoint::Account, HashMap::new()).await?;
        let balances = AlpacaParser::parse_balance(&response)?;

        if let Some(asset_filter) = query.asset {
            Ok(balances.into_iter().filter(|b| b.asset == asset_filter).collect())
        } else {
            Ok(balances)
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get_trading(AlpacaEndpoint::Account, HashMap::new()).await?;
        AlpacaParser::parse_account_info(&response, account_type)
    }

    /// Alpaca is commission-free — returns zero rates rather than UnsupportedOperation
    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Ok(FeeInfo {
            maker_rate: 0.0,
            taker_rate: 0.0,
            symbol: symbol.map(|s| s.to_string()),
            tier: Some("commission-free".to_string()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for AlpacaConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        if let Some(sym) = query.symbol {
            let symbol_str = format_symbol(&sym);
            let endpoint = AlpacaEndpoint::PositionBySymbol(symbol_str);
            let response = self.get_trading(endpoint, HashMap::new()).await?;
            let position = AlpacaParser::parse_position(&response)?;
            Ok(vec![position])
        } else {
            let response = self.get_trading(AlpacaEndpoint::Positions, HashMap::new()).await?;
            AlpacaParser::parse_positions(&response)
        }
    }

    /// Alpaca is a stock broker — funding rates are not applicable
    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Funding rates are not available on Alpaca (stock broker, no perpetual futures)".to_string()
        ))
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { symbol, .. } => {
                // Alpaca natively supports closing a single position via DELETE /v2/positions/{symbol}
                let symbol_str = format_symbol(&symbol);
                let endpoint = AlpacaEndpoint::PositionBySymbol(symbol_str);
                // DELETE /v2/positions/{symbol} returns the closing order
                self.delete_trading(endpoint, HashMap::new()).await?;
                Ok(())
            }

            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SetLeverage is not supported on Alpaca (stock broker, no leveraged futures positions)".to_string()
                ))
            }

            PositionModification::SetMarginMode { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SetMarginMode is not supported on Alpaca".to_string()
                ))
            }

            PositionModification::AddMargin { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "AddMargin is not supported on Alpaca".to_string()
                ))
            }

            PositionModification::RemoveMargin { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "RemoveMargin is not supported on Alpaca".to_string()
                ))
            }

            PositionModification::SetTpSl { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SetTpSl is not supported on Alpaca positions directly — use Bracket orders instead".to_string()
                ))
            }

            PositionModification::SwitchPositionMode { .. } | PositionModification::MovePositions { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SwitchPositionMode/MovePositions not supported on Alpaca".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPTIONAL TRAIT: CancelAll
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for AlpacaConnector {
    /// Cancel all open orders via DELETE /v2/orders
    ///
    /// Alpaca returns HTTP 207 Multi-Status with per-order sub-statuses.
    /// The native endpoint cancels all orders atomically.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let mut params = HashMap::new();

        // If scoped to a symbol, pass it as query param
        match &scope {
            CancelScope::BySymbol { symbol } => {
                params.insert("symbols".to_string(), format_symbol(symbol));
            }
            CancelScope::All { symbol: Some(sym) } => {
                params.insert("symbols".to_string(), format_symbol(sym));
            }
            CancelScope::All { symbol: None } => {
                // Cancel all — no filter
            }
            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    "CancelAll only supports CancelScope::All and CancelScope::BySymbol".to_string()
                ));
            }
        }

        // DELETE /v2/orders returns 207 Multi-Status array
        let response = self.delete_trading_multi(AlpacaEndpoint::Orders, params).await?;

        AlpacaParser::parse_cancel_all(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPTIONAL TRAIT: AmendOrder
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for AlpacaConnector {
    /// Amend an open order via PATCH /v2/orders/{order_id}
    ///
    /// Alpaca creates a new order atomically and cancels the original.
    /// The returned order has a new exchange-assigned ID.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let mut body = serde_json::Map::new();
        let symbol_str = format_symbol(&req.symbol);

        if let Some(qty) = req.fields.quantity {
            body.insert("qty".to_string(), json!(self.precision.qty(&symbol_str, qty)));
        }

        if let Some(price) = req.fields.price {
            body.insert("limit_price".to_string(), json!(self.precision.price(&symbol_str, price)));
        }

        if let Some(trigger) = req.fields.trigger_price {
            body.insert("stop_price".to_string(), json!(self.precision.price(&symbol_str, trigger)));
        }

        if body.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "AmendRequest must specify at least one field to change (qty, price, or trigger_price)".to_string()
            ));
        }

        let endpoint = AlpacaEndpoint::OrderById(req.order_id);
        let response = self.patch_trading(endpoint, Value::Object(body)).await?;
        AlpacaParser::parse_order(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Alpaca-specific)
// ═══════════════════════════════════════════════════════════════════════════

impl AlpacaConnector {
    /// Get list of all tradable assets
    pub async fn get_assets(&self) -> ExchangeResult<Vec<String>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "active".to_string());
        params.insert("tradable".to_string(), "true".to_string());

        let response = self.get_trading(AlpacaEndpoint::Assets, params).await?;
        AlpacaParser::parse_symbols(&response)
    }

    /// Get market clock (is market open?)
    pub async fn get_clock(&self) -> ExchangeResult<Value> {
        self.get_trading(AlpacaEndpoint::Clock, HashMap::new()).await
    }

    /// Get trading calendar
    pub async fn get_calendar(&self, start: Option<&str>, end: Option<&str>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }

        self.get_trading(AlpacaEndpoint::Calendar, params).await
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(&self) -> ExchangeResult<Vec<Order>> {
        let response = self.delete_trading(AlpacaEndpoint::Orders, HashMap::new()).await?;
        AlpacaParser::parse_orders(&response)
    }

    /// Get news
    pub async fn get_news(&self, symbols: Option<Vec<String>>, limit: Option<u32>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();

        if let Some(syms) = symbols {
            params.insert("symbols".to_string(), syms.join(","));
        }

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        self.get_market_data(AlpacaEndpoint::News, params).await
    }
}
