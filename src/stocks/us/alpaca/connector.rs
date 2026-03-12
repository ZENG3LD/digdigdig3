//! Alpaca connector implementation

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use serde_json::{json, Value};

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Alpaca connector
///
/// Supports both market data and trading operations for US stocks.
pub struct AlpacaConnector {
    client: Client,
    auth: AlpacaAuth,
    endpoints: AlpacaEndpoints,
    testnet: bool,
    feed: DataFeed,
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

        let infos = arr.iter().filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let tradable = item.get("tradable").and_then(|v| v.as_bool()).unwrap_or(false);
            if !tradable {
                return None;
            }
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active")
                .to_uppercase();

            Some(SymbolInfo {
                symbol: symbol.clone(),
                base_asset: symbol,
                quote_asset: "USD".to_string(),
                status,
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

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for AlpacaConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let _account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let symbol_str = format_symbol(&symbol);
                
                        let body = json!({
                            "symbol": symbol_str,
                            "qty": quantity.to_string(),
                            "side": side.as_str().to_lowercase(),
                            "type": "market",
                            "time_in_force": "day",
                        });
                
                        let response = self.post_trading(AlpacaEndpoint::Orders, body).await?;
                        AlpacaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let symbol_str = format_symbol(&symbol);
                
                        let body = json!({
                            "symbol": symbol_str,
                            "qty": quantity.to_string(),
                            "side": side.as_str().to_lowercase(),
                            "type": "limit",
                            "time_in_force": "gtc",
                            "limit_price": price.to_string(),
                        });
                
                        let response = self.post_trading(AlpacaEndpoint::Orders, body).await?;
                        AlpacaParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "get_order_history not yet implemented".to_string()
        ))
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let endpoint = AlpacaEndpoint::OrderById(order_id.to_string());
            let response = self.delete_trading(endpoint, HashMap::new()).await?;
            AlpacaParser::parse_order(&response)
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let endpoint = AlpacaEndpoint::OrderById(order_id.to_string());
        let response = self.get_trading(endpoint, HashMap::new()).await?;
        AlpacaParser::parse_order(&response)
    
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let symbol_str = symbol;
        let symbol: Option<crate::core::Symbol> = symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let mut params = HashMap::new();
        params.insert("status".to_string(), "open".to_string());

        if let Some(sym) = symbol {
            params.insert("symbols".to_string(), format_symbol(&sym));
        }

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
        let asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get_trading(AlpacaEndpoint::Account, HashMap::new()).await?;
        let balances = AlpacaParser::parse_balance(&response)?;

        // Filter by asset if specified
        if let Some(a) = asset {
            Ok(balances.into_iter().filter(|b| b.asset == a).collect())
        } else {
            Ok(balances)
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get_trading(AlpacaEndpoint::Account, HashMap::new()).await?;
        AlpacaParser::parse_account_info(&response, account_type)
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for AlpacaConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let _account_type = query.account_type;

        if let Some(sym) = symbol {
            // Get specific position
            let symbol_str = format_symbol(&sym);
            let endpoint = AlpacaEndpoint::PositionBySymbol(symbol_str);
            let response = self.get_trading(endpoint, HashMap::new()).await?;
            let position = AlpacaParser::parse_position(&response)?;
            Ok(vec![position])
        } else {
            // Get all positions
            let response = self.get_trading(AlpacaEndpoint::Positions, HashMap::new()).await?;
            AlpacaParser::parse_positions(&response)
        }
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let _symbol_str = _symbol;
        let _symbol = {
            let parts: Vec<&str> = _symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: _symbol_str.to_string(), quote: String::new(), raw: Some(_symbol_str.to_string()) }
            }
        };

        // Alpaca doesn't support funding rates (stocks broker, not futures)
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - Alpaca is a stock broker, not a futures exchange".to_string()
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Alpaca doesn't support leverage settings (stocks broker, not futures)
                Err(ExchangeError::UnsupportedOperation(
                "Leverage not available - Alpaca is a stock broker, not a futures exchange".to_string()
                ))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
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

    /// Close a position by symbol
    pub async fn close_position(&self, symbol: Symbol) -> ExchangeResult<Order> {
        let symbol_str = format_symbol(&symbol);
        let endpoint = AlpacaEndpoint::PositionBySymbol(symbol_str);
        let response = self.delete_trading(endpoint, HashMap::new()).await?;
        AlpacaParser::parse_order(&response)
    }

    /// Close all positions
    pub async fn close_all_positions(&self) -> ExchangeResult<Vec<Order>> {
        let response = self.delete_trading(AlpacaEndpoint::Positions, HashMap::new()).await?;
        AlpacaParser::parse_orders(&response)
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
