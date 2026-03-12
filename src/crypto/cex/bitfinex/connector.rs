//! # Bitfinex Connector
//!
//! Implementation of all core traits for Bitfinex API v2.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data endpoints
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Margin/futures positions

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::{CancelAll, AmendOrder};
use crate::core::types::{
    ConnectorStats, CancelAllResponse, OrderResult, AmendRequest,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BitfinexUrls, BitfinexEndpoint, format_symbol, build_candle_key};
use super::auth::BitfinexAuth;
use super::parser::BitfinexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitfinex connector
pub struct BitfinexConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods only)
    auth: Option<BitfinexAuth>,
    /// URLs (mainnet)
    urls: BitfinexUrls,
    /// Rate limiter (conservative: 10 requests per 60 seconds)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitfinexConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        let urls = BitfinexUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BitfinexAuth::new)
            .transpose()?;

        // Bitfinex rate limit: 90 requests per 60 seconds (matches registry rpm)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(90, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, _testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("lock");
                if limiter.try_acquire() {
                    return;
                }
                limiter.time_until_ready()
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: BitfinexEndpoint,
        path_params: &[(&str, &str)],
        query_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(endpoint.requires_auth());
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        // Build query string
        let query = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        let response = self.http.get(&url, &HashMap::new()).await?;
        BitfinexParser::check_error(&response)?;
        Ok(response)
    }

    /// POST request (authenticated)
    async fn post(
        &self,
        endpoint: BitfinexEndpoint,
        path_params: &[(&str, &str)],
        body: Value,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(true); // Always use auth URL for POST
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Get auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        // API path without /v2 prefix (auth expects "v2/auth/r/wallets" not "/v2/auth/r/wallets")
        let api_path = path.trim_start_matches('/');
        let body_str = body.to_string();
        let headers = auth.sign_request(api_path, &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        BitfinexParser::check_error(&response)?;
        Ok(response)
    }

    /// Build a minimal Order struct from known fields after place_order
    fn make_order(
        &self,
        id: String,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        quantity: Quantity,
        price: Option<Price>,
        stop_price: Option<Price>,
    ) -> Order {
        Order {
            id,
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type,
            status: crate::core::OrderStatus::New,
            price,
            stop_price,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        }
    }

    /// Format symbol helper
    fn fmt_symbol(symbol: &Symbol, account_type: AccountType) -> String {
        if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        }
    }

    /// Determine if account type is derivatives (margin/futures)
    fn is_derivatives(account_type: AccountType) -> bool {
        matches!(account_type, AccountType::Margin | AccountType::FuturesCross | AccountType::FuturesIsolated)
    }

    /// Get order type string prefix ("EXCHANGE " for spot, "" for margin/futures)
    fn order_type_prefix(account_type: AccountType) -> &'static str {
        if Self::is_derivatives(account_type) {
            ""
        } else {
            "EXCHANGE "
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitfinexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitfinex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_count(), lim.max_requests())
        } else {
            (0, 0)
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups: Vec::new(),
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        false // Bitfinex doesn't have a public testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BitfinexConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

        let response = self.get(
            BitfinexEndpoint::Ticker,
            &[("symbol", &formatted_symbol)],
            HashMap::new(),
        ).await?;

        let ticker = BitfinexParser::parse_ticker(&response, &formatted_symbol)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

        // Use P0 precision (highest aggregation) for best performance
        let response = self.get(
            BitfinexEndpoint::Orderbook,
            &[("symbol", &formatted_symbol), ("precision", "P0")],
            HashMap::new(),
        ).await?;

        BitfinexParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);
        let candle_key = build_candle_key(&formatted_symbol, interval);

        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(10000).to_string());
        }
        // Don't use sort=1 — it returns data from 2013. Default (newest-first) + parser.reverse() is correct.

        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }

        let response = self.get(
            BitfinexEndpoint::Candles,
            &[("candle", &candle_key)],
            params,
        ).await?;

        BitfinexParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

        let response = self.get(
            BitfinexEndpoint::Ticker,
            &[("symbol", &formatted_symbol)],
            HashMap::new(),
        ).await?;

        BitfinexParser::parse_ticker(&response, &formatted_symbol)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(
            BitfinexEndpoint::PlatformStatus,
            &[],
            HashMap::new(),
        ).await?;

        // Platform status returns [1] for operative, [0] for maintenance
        if let Some(arr) = response.as_array() {
            if !arr.is_empty() {
                if let Some(status) = arr[0].as_i64() {
                    if status == 1 {
                        return Ok(());
                    }
                }
            }
        }

        Err(ExchangeError::Network("Platform in maintenance".to_string()))
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Use Bitfinex v1 symbols_details endpoint (returns array with pair info)
        // Note: v1 is still supported and returns more detail than v2 conf endpoints
        self.rate_limit_wait().await;
        let url = "https://api.bitfinex.com/v1/symbols_details";
        let response = self.http.get(url, &HashMap::new()).await?;
        BitfinexParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitfinexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);
        let prefix = Self::order_type_prefix(account_type);

        // Amount: positive=buy, negative=sell
        let amount = match side {
            OrderSide::Buy => quantity,
            OrderSide::Sell => -quantity,
        };

        match req.order_type {
            OrderType::Market => {
                let body = json!({
                    "type": format!("{}MARKET", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": price.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // Bitfinex: EXCHANGE STOP (triggers market at stop_price)
                let body = json!({
                    "type": format!("{}STOP", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": stop_price.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Bitfinex: EXCHANGE STOP LIMIT
                let body = json!({
                    "type": format!("{}STOP LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": limit_price.to_string(),
                    "price_aux_limit": stop_price.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::TrailingStop { callback_rate, activation_price: _ } => {
                // Bitfinex: EXCHANGE TRAILING STOP
                // trail_pct is callback_rate in percent
                let body = json!({
                    "type": format!("{}TRAILING STOP", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": callback_rate.to_string(), // trail distance as % string
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::PostOnly { price } => {
                // Bitfinex: EXCHANGE LIMIT with flags = 4096 (POST_ONLY)
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": price.to_string(),
                    "flags": 4096,
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // Bitfinex: EXCHANGE IOC
                let price_val = price.unwrap_or(0.0);
                let body = json!({
                    "type": format!("{}IOC", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": price_val.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Fok { price } => {
                // Bitfinex: EXCHANGE FOK
                let body = json!({
                    "type": format!("{}FOK", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": price.to_string(),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Bitfinex: EXCHANGE LIMIT with max_show parameter
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "price": price.to_string(),
                    "meta": {
                        "max_show": display_quantity.to_string(),
                    },
                    "flags": 64, // Hidden flag
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::ReduceOnly { price } => {
                // Bitfinex: LIMIT with reduce-only flag (only valid for margin/futures)
                if !Self::is_derivatives(account_type) {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly not supported for Spot".to_string()
                    ));
                }
                let order_type_str = if price.is_some() { "LIMIT" } else { "MARKET" };
                let mut body = json!({
                    "type": order_type_str,
                    "symbol": formatted_symbol,
                    "amount": amount.to_string(),
                    "flags": 1024, // REDUCE_ONLY flag
                });
                if let Some(p) = price {
                    body["price"] = json!(p.to_string());
                }
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let id = order_id.parse::<i64>()
                    .map_err(|_| ExchangeError::InvalidRequest("Invalid order ID".to_string()))?;

                let body = json!({ "id": id });
                let response = self.post(BitfinexEndpoint::CancelOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response)
            }

            CancelScope::Batch { ref order_ids } => {
                // Bitfinex: POST /auth/w/order/cancel/multi with {"id": [...]}
                let ids: Vec<i64> = order_ids.iter()
                    .filter_map(|id| id.parse::<i64>().ok())
                    .collect();

                if ids.is_empty() {
                    return Err(ExchangeError::InvalidRequest("No valid order IDs".to_string()));
                }

                let body = json!({ "id": ids });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                // Return a placeholder — Bitfinex multi-cancel returns notifications, not a single order
                Ok(Order {
                    id: ids[0].to_string(),
                    client_order_id: None,
                    symbol: req.symbol.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported — use CancelAll trait for All/BySymbol", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut body = json!({});

        if let Some(sym) = &filter.symbol {
            // sym is already a Symbol struct
            let formatted_symbol = Self::fmt_symbol(sym, _account_type);
            body["symbol"] = json!(formatted_symbol);
        }

        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(limit) = filter.limit {
            body["limit"] = json!(limit.min(2500));
        }

        let response = self.post(BitfinexEndpoint::OrderHistory, &[], body).await?;
        BitfinexParser::parse_orders(&response)
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

        let body = json!({ "symbol": formatted_symbol });

        let response = self.post(
            BitfinexEndpoint::ActiveOrdersBySymbol,
            &[("symbol", &formatted_symbol)],
            body,
        ).await?;

        let orders = BitfinexParser::parse_orders(&response)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::Parse(format!("Order {} not found", order_id)))
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let symbol: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = if let Some(s) = symbol {
            let formatted_symbol = Self::fmt_symbol(&s, account_type);
            self.post(
                BitfinexEndpoint::ActiveOrdersBySymbol,
                &[("symbol", &formatted_symbol)],
                json!({}),
            ).await?
        } else {
            self.post(
                BitfinexEndpoint::ActiveOrders,
                &[],
                json!({}),
            ).await?
        };

        BitfinexParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitfinexConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let response = self.post(
            BitfinexEndpoint::Wallets,
            &[],
            json!({}),
        ).await?;

        BitfinexParser::parse_balances(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1,  // Default Bitfinex fees
            taker_commission: 0.2,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Bitfinex: POST /auth/r/trades/hist returns trades with fee info
        // Use a recent trade to get actual fee rates, or return defaults from account summary
        let body = json!({ "limit": 1 });
        let response = self.post(BitfinexEndpoint::TradeHistory, &[], body).await?;

        // Try to extract fee from most recent trade
        if let Some(arr) = response.as_array() {
            if let Some(trade) = arr.first() {
                if let Some(trade_arr) = trade.as_array() {
                    // Trade array format: [ID, PAIR, MTS_CREATE, ORDER_ID, EXEC_AMOUNT, EXEC_PRICE, ORDER_TYPE, ORDER_PRICE, MAKER, FEE, FEE_CURRENCY]
                    if trade_arr.len() > 9 {
                        let fee = trade_arr[9].as_f64().unwrap_or(0.0).abs();
                        let amount = trade_arr[4].as_f64().unwrap_or(1.0).abs();
                        let price = trade_arr[5].as_f64().unwrap_or(1.0);
                        let rate = if amount * price > 0.0 { fee / (amount * price) } else { 0.002 };

                        return Ok(FeeInfo {
                            maker_rate: rate,
                            taker_rate: rate,
                            symbol: symbol.map(|s| s.to_string()),
                            tier: None,
                        });
                    }
                }
            }
        }

        // Fallback: default Bitfinex fees
        Ok(FeeInfo {
            maker_rate: 0.001,  // 0.1%
            taker_rate: 0.002,  // 0.2%
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BitfinexConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let account_type = query.account_type;

        if account_type == AccountType::Spot {
            return Err(ExchangeError::UnsupportedOperation(
                "Positions not supported for Spot".to_string()
            ));
        }

        let response = self.post(
            BitfinexEndpoint::Positions,
            &[],
            json!({}),
        ).await?;

        BitfinexParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<crate::core::FundingRate> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        // Bitfinex doesn't have a direct funding rate endpoint for perpetuals
        // Would need to implement via derivatives API or funding book
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate endpoint not implemented for Bitfinex".to_string()
        ))
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { account_type, .. } => {
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Leverage not supported for Spot".to_string()
                    ));
                }
                // Bitfinex handles leverage via order flags, not a separate endpoint
                Err(ExchangeError::UnsupportedOperation(
                    "Set leverage not available - use order flags instead".to_string()
                ))
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BitfinexConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { symbol: None } => {
                // Cancel all orders across all symbols
                let body = json!({ "all": 1 });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                Ok(CancelAllResponse {
                    cancelled_count: 0, // Bitfinex doesn't return count
                    failed_count: 0,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // Cancel all orders for a specific symbol
                let formatted_symbol = Self::fmt_symbol(&sym, _account_type);
                let body = json!({ "symbol": formatted_symbol });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BitfinexConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let id = req.order_id.parse::<i64>()
            .map_err(|_| ExchangeError::InvalidRequest("Invalid order ID".to_string()))?;

        let mut body = json!({ "id": id });

        if let Some(price) = req.fields.price {
            body["price"] = json!(price.to_string());
        }
        if let Some(qty) = req.fields.quantity {
            // For Bitfinex, amount sign determines buy/sell — preserve original sign
            body["amount"] = json!(qty.to_string());
        }
        if let Some(stop_price) = req.fields.trigger_price {
            body["price_aux_limit"] = json!(stop_price.to_string());
        }

        let response = self.post(BitfinexEndpoint::UpdateOrder, &[], body).await?;
        BitfinexParser::parse_submit_order(&response)
    }
}
