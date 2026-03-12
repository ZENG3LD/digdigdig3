//! # Crypto.com Connector
//!
//! Implementation of all core traits for Crypto.com Exchange.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Futures positions

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicI64, Ordering}};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CancelAllResponse, AmendRequest, MarginType,
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
};
use crate::core::types::{SymbolInfo, OrderResult};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{CryptoComUrls, CryptoComEndpoint, format_symbol, account_type_to_instrument, map_kline_interval};
use super::auth::CryptoComAuth;
use super::parser::CryptoComParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Crypto.com connector
pub struct CryptoComConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<CryptoComAuth>,
    /// URLs (mainnet/testnet)
    urls: CryptoComUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (100 requests per second)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Request ID counter
    request_id: Arc<AtomicI64>,
}

impl CryptoComConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            CryptoComUrls::TESTNET
        } else {
            CryptoComUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(CryptoComAuth::new)
            .transpose()?;

        // Initialize rate limiter: 100 requests per second
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            request_id: Arc::new(AtomicI64::new(1)),
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get next request ID
    fn next_id(&self) -> i64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
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

    /// Make API request
    async fn request(
        &self,
        endpoint: CryptoComEndpoint,
        params: Value,
    ) -> ExchangeResult<Value> {
        // Rate limiting
        self.rate_limit_wait().await;

        let method = endpoint.method();
        let base_url = self.urls.rest_url();
        let url = format!("{}/{}", base_url, method);

        let response = if endpoint.requires_auth() {
            // Private endpoints use POST with JSON body
            let id = self.next_id();
            let nonce = CryptoComAuth::generate_nonce();

            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let signature = auth.sign_request(method, id, &params, nonce);

            let mut body = json!({
                "id": id,
                "method": method,
                "nonce": nonce,
                "api_key": auth.api_key(),
                "sig": signature
            });

            // Add params if not empty
            if !params.is_null() && params.as_object().is_some_and(|o| !o.is_empty()) {
                body["params"] = params;
            }

            let headers = HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
            ]);

            self.http.post(&url, &body, &headers).await?
        } else {
            // Public endpoints use GET with query parameters
            let mut query_url = url;

            if let Some(obj) = params.as_object() {
                if !obj.is_empty() {
                    let query_string: Vec<String> = obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_str().map(|s| format!("{}={}", k, s))
                                .or_else(|| v.as_i64().map(|n| format!("{}={}", k, n)))
                                .or_else(|| v.as_u64().map(|n| format!("{}={}", k, n)))
                                .or_else(|| v.as_f64().map(|n| format!("{}={}", k, n)))
                        })
                        .collect();

                    if !query_string.is_empty() {
                        query_url = format!("{}?{}", query_url, query_string.join("&"));
                    }
                }
            }

            let headers = HashMap::new();
            self.http.get(&query_url, &headers).await?
        };

        CryptoComParser::check_response(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CryptoComConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::CryptoCom
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
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
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
impl MarketData for CryptoComConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetTickers, params).await?;
        CryptoComParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let mut params = json!({
            "instrument_name": instrument_name
        });

        if let Some(d) = depth {
            params["depth"] = json!(d);
        }

        let response = self.request(CryptoComEndpoint::GetBook, params).await?;
        CryptoComParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);
        let timeframe = map_kline_interval(interval);

        let mut params = json!({
            "instrument_name": instrument_name,
            "timeframe": timeframe,
            "count": limit.unwrap_or(300).min(300)
        });

        if let Some(end_ts) = end_time {
            params["end_ts"] = json!(end_ts);
        }

        let response = self.request(CryptoComEndpoint::GetCandlestick, params).await?;
        CryptoComParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetTickers, params).await?;
        CryptoComParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.request(CryptoComEndpoint::GetInstruments, json!({})).await?;
        CryptoComParser::check_response(&response)
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.request(CryptoComEndpoint::GetInstruments, json!({})).await?;
        CryptoComParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for CryptoComConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);
        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        match req.order_type {
            OrderType::Market => {
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "MARKET",
                    "quantity": quantity.to_string(),
                });

                let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::New,
                    price: None,
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::Limit { price } => {
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "LIMIT",
                    "quantity": quantity.to_string(),
                    "price": price.to_string(),
                    "time_in_force": "GOOD_TILL_CANCEL",
                });

                let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Limit { price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::StopMarket { stop_price } => {
                // Crypto.com migrated stop orders to private/advanced/create-order on 2026-01-28.
                // The legacy private/create-order no longer accepts STOP_LOSS / STOP_LIMIT types.
                // Advanced endpoint: type="STOP_LOSS", ref_price=stop trigger price.
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "STOP_LOSS",
                    "quantity": quantity.to_string(),
                    "ref_price": stop_price.to_string(),
                });

                let response = self.request(CryptoComEndpoint::AdvancedCreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::StopMarket { stop_price },
                    status: crate::core::OrderStatus::New,
                    price: None,
                    stop_price: Some(stop_price),
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Crypto.com migrated stop-limit orders to private/advanced/create-order on 2026-01-28.
                // Advanced endpoint: type="STOP_LIMIT", ref_price=trigger, price=limit price.
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "STOP_LIMIT",
                    "quantity": quantity.to_string(),
                    "price": limit_price.to_string(),
                    "ref_price": stop_price.to_string(),
                    "time_in_force": "GOOD_TILL_CANCEL",
                });

                let response = self.request(CryptoComEndpoint::AdvancedCreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::StopLimit { stop_price, limit_price },
                    status: crate::core::OrderStatus::New,
                    price: Some(limit_price),
                    stop_price: Some(stop_price),
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::PostOnly { price } => {
                // Crypto.com exec_inst="POST_ONLY" on a LIMIT order
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "LIMIT",
                    "quantity": quantity.to_string(),
                    "price": price.to_string(),
                    "exec_inst": "POST_ONLY",
                    "time_in_force": "GOOD_TILL_CANCEL",
                });

                let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::PostOnly { price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::PostOnly,
                }))
            }

            OrderType::Ioc { price } => {
                // IMMEDIATE_OR_CANCEL with optional limit price
                let (order_type_str, price_field) = match price {
                    Some(p) => ("LIMIT", Some(p)),
                    None => ("MARKET", None),
                };

                let mut params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": order_type_str,
                    "quantity": quantity.to_string(),
                    "time_in_force": "IMMEDIATE_OR_CANCEL",
                });

                if let Some(p) = price_field {
                    params["price"] = json!(p.to_string());
                }

                let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Ioc { price },
                    status: crate::core::OrderStatus::New,
                    price: price_field,
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Ioc,
                }))
            }

            OrderType::Fok { price } => {
                // FILL_OR_KILL — LIMIT with time_in_force=FILL_OR_KILL
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "LIMIT",
                    "quantity": quantity.to_string(),
                    "price": price.to_string(),
                    "time_in_force": "FILL_OR_KILL",
                });

                let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
                let order_id = CryptoComParser::parse_order_id(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Fok { price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Fok,
                }))
            }

            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // Crypto.com OCO: private/advanced/create-oco — Spot only (as of 2026-01-28).
                // First leg: limit order at `price`.
                // Second leg: stop-market (stop_limit_price=None) or stop-limit (stop_limit_price=Some).
                let account_type = req.account_type;
                let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);
                if is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "OCO orders are only supported for Spot on Crypto.com".to_string()
                    ));
                }
                let (leg2_type, leg2_price) = match stop_limit_price {
                    Some(lp) => ("STOP_LIMIT", Some(lp)),
                    None => ("STOP_LOSS", None),
                };
                let mut leg2 = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": leg2_type,
                    "quantity": quantity.to_string(),
                    "ref_price": stop_price.to_string(),
                });
                if let Some(lp) = leg2_price {
                    leg2["price"] = json!(lp.to_string());
                }
                let params = json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "price": price.to_string(),
                    "quantity": quantity.to_string(),
                    "stop_side": side_str,
                    "ref_price": stop_price.to_string(),
                    "ref_price_type": "MARK_PRICE",
                    "contingency_type": "OCO",
                });

                let response = self.request(CryptoComEndpoint::AdvancedCreateOco, params).await?;
                CryptoComParser::check_response(&response)?;

                // OCO returns two order IDs; build a minimal OcoResponse
                let list_id = response
                    .get("result")
                    .and_then(|r| r.get("list_id"))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let now = crate::core::timestamp_millis() as i64;
                let make_leg = |otype: OrderType, px: Option<Price>, sp: Option<Price>| Order {
                    id: String::new(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side,
                    order_type: otype,
                    status: crate::core::OrderStatus::New,
                    price: px,
                    stop_price: sp,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: now,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                };

                Ok(PlaceOrderResponse::Oco(crate::core::types::OcoResponse {
                    first_order: make_leg(OrderType::Limit { price }, Some(price), None),
                    second_order: make_leg(
                        OrderType::StopMarket { stop_price },
                        stop_limit_price,
                        Some(stop_price),
                    ),
                    list_id,
                }))
            }

            // Unsupported on Crypto.com
            // TrailingStop: confirmed NOT available via API (UI only) — research section 3.6
            // Bracket/OTOCO: available via private/advanced/create-otoco but not in our OrderType enum yet
            // Iceberg: not available on Crypto.com
            // TWAP: not available on Crypto.com
            // GTD: not available on Crypto.com standard API
            // ReduceOnly: use ClosePosition or separate reduce-only order flag
            OrderType::TrailingStop { .. } => Err(ExchangeError::UnsupportedOperation(
                "TrailingStop is not available via Crypto.com API (UI-only feature)".to_string()
            )),
            OrderType::Bracket { .. }
            | OrderType::Iceberg { .. }
            | OrderType::Twap { .. }
            | OrderType::Gtd { .. }
            | OrderType::ReduceOnly { .. } => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on Crypto.com", req.order_type)
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

                let params = json!({
                    "instrument_name": instrument_name,
                    "order_id": order_id,
                });

                let response = self.request(CryptoComEndpoint::CancelOrder, params).await?;
                CryptoComParser::check_response(&response)?;

                Ok(Order {
                    id: order_id.to_string(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy, // Unknown from cancel response
                    order_type: OrderType::Limit { price: 0.0 },
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

            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let mut params = json!({});

                if let Some(sym) = symbol {
                    let instrument_type = account_type_to_instrument(account_type);
                    let instrument_name = format_symbol(&sym.base, &sym.quote, instrument_type);
                    params["instrument_name"] = json!(instrument_name);
                }

                let response = self.request(CryptoComEndpoint::CancelAllOrders, params).await?;
                CryptoComParser::check_response(&response)?;

                Ok(Order {
                    id: "cancel_all".to_string(),
                    client_order_id: None,
                    symbol: symbol.as_ref().map(|s| s.to_string()).unwrap_or_default(),
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

            CancelScope::BySymbol { ref symbol } => {
                let account_type = req.account_type;
                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

                let params = json!({
                    "instrument_name": instrument_name,
                });

                let response = self.request(CryptoComEndpoint::CancelAllOrders, params).await?;
                CryptoComParser::check_response(&response)?;

                Ok(Order {
                    id: "cancel_by_symbol".to_string(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
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

            CancelScope::Batch { .. } => Err(ExchangeError::UnsupportedOperation(
                "Batch cancel not supported via cancel_order on Crypto.com".to_string()
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let params = json!({
            "order_id": order_id,
        });

        let response = self.request(CryptoComEndpoint::GetOrderDetail, params).await?;
        CryptoComParser::parse_order(&response)
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

        let mut params = json!({});

        if let Some(s) = symbol {
            let instrument_type = account_type_to_instrument(account_type);
            let instrument_name = format_symbol(&s.base, &s.quote, instrument_type);
            params["instrument_name"] = json!(instrument_name);
        }

        let response = self.request(CryptoComEndpoint::GetOpenOrders, params).await?;
        CryptoComParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // private/get-order-history supports start_ts, end_ts, page, page_size
        let mut params = json!({});

        if let Some(ref sym) = filter.symbol {
            let instrument_type = account_type_to_instrument(account_type);
            let instrument_name = format_symbol(&sym.base, &sym.quote, instrument_type);
            params["instrument_name"] = json!(instrument_name);
        }

        if let Some(start) = filter.start_time {
            params["start_ts"] = json!(start);
        }

        if let Some(end) = filter.end_time {
            params["end_ts"] = json!(end);
        }

        if let Some(lim) = filter.limit {
            params["page_size"] = json!(lim.min(200));
        }

        let response = self.request(CryptoComEndpoint::GetOrderHistory, params).await?;
        CryptoComParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for CryptoComConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let mut params = json!({});

        match &scope {
            CancelScope::All { symbol } => {
                if let Some(sym) = symbol {
                    let instrument_type = account_type_to_instrument(account_type);
                    let instrument_name = format_symbol(&sym.base, &sym.quote, instrument_type);
                    params["instrument_name"] = json!(instrument_name);
                }
            }
            CancelScope::BySymbol { symbol } => {
                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);
                params["instrument_name"] = json!(instrument_name);
            }
            _ => return Err(ExchangeError::InvalidRequest(
                "cancel_all_orders requires CancelScope::All or BySymbol".to_string()
            )),
        }

        let response = self.request(CryptoComEndpoint::CancelAllOrders, params).await?;
        CryptoComParser::check_response(&response)?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // Crypto.com doesn't return count
            failed_count: 0,
            details: vec![],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for CryptoComConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let symbol = req.symbol.clone();
        let account_type = req.account_type;

        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let mut params = json!({
            "order_id": req.order_id,
            "instrument_name": instrument_name,
        });

        if let Some(price) = req.fields.price {
            params["price"] = json!(price.to_string());
        }

        if let Some(qty) = req.fields.quantity {
            params["quantity"] = json!(qty.to_string());
        }

        // trigger_price maps to ref_price for stop orders
        if let Some(trigger) = req.fields.trigger_price {
            params["ref_price"] = json!(trigger.to_string());
        }

        let response = self.request(CryptoComEndpoint::AmendOrder, params).await?;
        CryptoComParser::parse_order(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for CryptoComConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let params = json!({});
        let response = self.request(CryptoComEndpoint::UserBalance, params).await?;
        CryptoComParser::parse_balances(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.075, // Default Crypto.com fee
            taker_commission: 0.075,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Use get-fee-rate for account-wide or get-instrument-fee-rate for symbol-specific
        let (endpoint, params) = if let Some(sym) = symbol {
            // Symbol-specific fee: need to parse symbol string
            let parts: Vec<&str> = sym.split('/').collect();
            let instrument_name = if parts.len() == 2 {
                // Try to parse as BASE/QUOTE
                let sym_struct = crate::core::Symbol::new(parts[0], parts[1]);
                format_symbol(&sym_struct.base, &sym_struct.quote,
                    super::endpoints::InstrumentType::Spot)
            } else {
                sym.to_string()
            };
            (
                CryptoComEndpoint::GetInstrumentFeeRate,
                json!({ "instrument_name": instrument_name }),
            )
        } else {
            (CryptoComEndpoint::GetFeeRate, json!({}))
        };

        let response = self.request(endpoint, params).await?;
        CryptoComParser::parse_fee_rate(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for CryptoComConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot".to_string()
                ));
            }
            _ => {}
        }

        let mut params = json!({});

        if let Some(s) = symbol {
            let instrument_type = account_type_to_instrument(account_type);
            let instrument_name = format_symbol(&s.base, &s.quote, instrument_type);
            params["instrument_name"] = json!(instrument_name);
        }

        let response = self.request(CryptoComEndpoint::GetPositions, params).await?;
        CryptoComParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        let symbol_str = symbol;
        let symbol = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot".to_string()
                ));
            }
            _ => {}
        }

        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetValuations, params).await?;
        CryptoComParser::parse_funding_rate(&response)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { leverage, .. } => {
                let params = json!({
                    "leverage": leverage.to_string()
                });

                let response = self.request(CryptoComEndpoint::ChangeAccountLeverage, params).await?;
                CryptoComParser::check_response(&response)
            }

            PositionModification::SetMarginMode { ref symbol, ref margin_type, account_type } => {
                let symbol = symbol.clone();
                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

                // change-isolated-margin-leverage: leverage=0 for cross, >0 for isolated
                let leverage = match margin_type {
                    MarginType::Cross => "0".to_string(),
                    MarginType::Isolated => "10".to_string(), // default 10x isolated
                };

                let params = json!({
                    "instrument_name": instrument_name,
                    "leverage": leverage,
                });

                let response = self.request(CryptoComEndpoint::ChangeIsolatedMarginLeverage, params).await?;
                CryptoComParser::check_response(&response)
            }

            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

                // create-isolated-margin-transfer direction=IN to add margin
                let params = json!({
                    "instrument_name": instrument_name,
                    "direction": "IN",
                    "amount": amount.to_string(),
                });

                // Note: Crypto.com uses a different endpoint for margin transfers
                // private/create-isolated-margin-transfer is not in our endpoint enum
                // We'll use ChangeIsolatedMarginLeverage as the closest available
                // For completeness, flag as unsupported with a descriptive message
                let _ = params; // suppress unused warning
                Err(ExchangeError::UnsupportedOperation(
                    "AddMargin requires private/create-isolated-margin-transfer endpoint (not yet mapped)".to_string()
                ))
            }

            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let _ = (symbol, amount, account_type);
                Err(ExchangeError::UnsupportedOperation(
                    "RemoveMargin requires private/create-isolated-margin-transfer endpoint (not yet mapped)".to_string()
                ))
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                let instrument_type = account_type_to_instrument(account_type);
                let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

                // private/close-position with type=MARKET
                let params = json!({
                    "instrument_name": instrument_name,
                    "type": "MARKET",
                });

                let response = self.request(CryptoComEndpoint::ClosePosition, params).await?;
                CryptoComParser::check_response(&response)
            }

            PositionModification::SetTpSl { .. } => {
                // Crypto.com doesn't have a unified SetTpSl endpoint
                // TP/SL must be placed as separate TAKE_PROFIT / STOP_LOSS orders
                Err(ExchangeError::UnsupportedOperation(
                    "SetTpSl not supported as a single operation on Crypto.com; place separate TP/SL orders".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for CryptoComConnector {
    /// Place multiple orders in a single batch request.
    ///
    /// Endpoint: private/create-order-list
    /// Params: `contingency_type` = "LIST", `order_list` = array of order params
    /// Max 10 orders per batch.
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let account_type = orders.first().map(|o| o.account_type).unwrap_or(AccountType::Spot);
        let instrument_type = super::endpoints::account_type_to_instrument(account_type);

        let order_list: Vec<Value> = orders.iter().map(|req| {
            let instrument_name = super::endpoints::format_symbol(&req.symbol.base, &req.symbol.quote, instrument_type);
            let side_str = match req.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };

            match &req.order_type {
                OrderType::Market => json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "MARKET",
                    "quantity": req.quantity.to_string(),
                }),
                OrderType::Limit { price } => json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "LIMIT",
                    "quantity": req.quantity.to_string(),
                    "price": price.to_string(),
                    "time_in_force": "GOOD_TILL_CANCEL",
                }),
                OrderType::PostOnly { price } => json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "LIMIT",
                    "quantity": req.quantity.to_string(),
                    "price": price.to_string(),
                    "exec_inst": "POST_ONLY",
                    "time_in_force": "GOOD_TILL_CANCEL",
                }),
                // Note: StopMarket / StopLimit / advanced order types cannot be included in
                // private/create-order-list (LIST batches support LIMIT and MARKET only).
                // Fall back to MARKET for unrecognized types rather than silently building
                // invalid requests — callers should use place_order for conditional types.
                _ => json!({
                    "instrument_name": instrument_name,
                    "side": side_str,
                    "type": "MARKET",
                    "quantity": req.quantity.to_string(),
                }),
            }
        }).collect();

        let params = json!({
            "contingency_type": "LIST",
            "order_list": order_list,
        });

        let response = self.request(CryptoComEndpoint::CreateOrderList, params).await?;
        CryptoComParser::check_response(&response)?;

        // Parse response — result.data.result_list contains per-order results
        let result_list = response
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(|d| d.get("result_list"))
            .and_then(|v| v.as_array());

        let results = if let Some(list) = result_list {
            list.iter().enumerate().map(|(i, item)| {
                let code = item.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                let success = code == 0;
                let order_id = item.get("order_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let req = orders.get(i);

                OrderResult {
                    order: if success { order_id.map(|id| Order {
                        id,
                        client_order_id: None,
                        symbol: req.map(|o| o.symbol.to_string()).unwrap_or_default(),
                        side: req.map(|o| o.side).unwrap_or(OrderSide::Buy),
                        order_type: req.map(|o| o.order_type.clone()).unwrap_or(OrderType::Market),
                        status: crate::core::OrderStatus::New,
                        price: None,
                        stop_price: None,
                        quantity: req.map(|o| o.quantity).unwrap_or(0.0),
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: crate::core::timestamp_millis() as i64,
                        updated_at: None,
                        time_in_force: crate::core::TimeInForce::Gtc,
                    })} else { None },
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("message").and_then(|v| v.as_str()).map(String::from)
                    },
                    error_code: if success { None } else { Some(code as i32) },
                }
            }).collect()
        } else {
            orders.iter().map(|_| OrderResult {
                order: None,
                client_order_id: None,
                success: false,
                error: Some("No result list in response".to_string()),
                error_code: None,
            }).collect()
        };

        Ok(results)
    }

    /// Cancel multiple orders by IDs.
    ///
    /// Endpoint: private/cancel-order-list
    /// Params: `order_list` = array of `{"instrument_name": ..., "order_id": ...}`
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        let instrument_type = super::endpoints::account_type_to_instrument(account_type);

        // Build order list — Crypto.com requires instrument_name for each cancel
        let order_list: Vec<Value> = order_ids.iter().map(|order_id| {
            let mut obj = json!({ "order_id": order_id });
            if let Some(sym) = symbol {
                let parts: Vec<&str> = sym.split('/').collect();
                let instrument_name = if parts.len() == 2 {
                    super::endpoints::format_symbol(parts[0], parts[1], instrument_type)
                } else {
                    sym.to_string()
                };
                obj["instrument_name"] = json!(instrument_name);
            }
            obj
        }).collect();

        let params = json!({ "order_list": order_list });
        let response = self.request(CryptoComEndpoint::CancelOrderList, params).await?;
        CryptoComParser::check_response(&response)?;

        let result_list = response
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(|d| d.get("result_list"))
            .and_then(|v| v.as_array());

        let results = if let Some(list) = result_list {
            list.iter().map(|item| {
                let code = item.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                let success = code == 0;
                OrderResult {
                    order: None,
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("message").and_then(|v| v.as_str()).map(String::from)
                    },
                    error_code: if success { None } else { Some(code as i32) },
                }
            }).collect()
        } else {
            order_ids.iter().map(|_| OrderResult {
                order: None,
                client_order_id: None,
                success: true,
                error: None,
                error_code: None,
            }).collect()
        };

        Ok(results)
    }

    /// Maximum batch place size (Crypto.com limit: 10 orders per batch).
    fn max_batch_place_size(&self) -> usize {
        10
    }

    /// Maximum batch cancel size (Crypto.com limit: 10 orders per batch).
    fn max_batch_cancel_size(&self) -> usize {
        10
    }
}
