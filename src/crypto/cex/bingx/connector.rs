//! # BingX Connector
//!
//! Реализация всех core трейтов для BingX.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Optional трейты
//! - `CancelAll` - отмена всех ордеров
//! - `AmendOrder` - изменение ордера (swap only)

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
    Position, FundingRate, MarginType,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    TimeInForce, AmendRequest,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::{CancelAll, AmendOrder};
use crate::core::types::{ConnectorStats, CancelAllResponse};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BingxUrls, BingxEndpoint, format_symbol, map_kline_interval};
use super::auth::BingxAuth;
use super::parser::BingxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// BingX коннектор
pub struct BingxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BingxAuth>,
    /// URL'ы
    urls: BingxUrls,
    /// Rate limiter для market data (100 req/10s)
    market_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BingxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        // BingX doesn't have a public testnet, always use mainnet
        let urls = BingxUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BingxAuth::new)
            .transpose()?;

        // BingX rate limit: 100 requests per 10 seconds (shared pool)
        let market_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            market_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, false).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.market_limiter.lock().expect("Mutex poisoned");
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

    /// GET запрос
    async fn get(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request(&mut params)
        } else {
            HashMap::new()
        };

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build form body
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.post(&url, &json!({}), &headers).await?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build query string
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// Check BingX response for API errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
        if code != 0 {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: msg.to_string(),
            });
        }
        Ok(())
    }

    /// Build a minimal returned Order after placing
    fn build_placed_order(
        order_id: String,
        client_order_id: Option<String>,
        symbol: &Symbol,
        side: OrderSide,
        order_type: OrderType,
        price: Option<Price>,
        quantity: Quantity,
    ) -> Order {
        Order {
            id: order_id,
            client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            status: crate::core::OrderStatus::New,
            price,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BingxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BingX
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.market_limiter.lock() {
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
        false // BingX doesn't have public testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
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
impl MarketData for BingxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use get_ticker and extract the last_price
        let ticker = self.get_ticker(symbol, account_type).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotDepth,
            _ => BingxEndpoint::SwapDepth,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotKlines,
            _ => BingxEndpoint::SwapKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1440).to_string());
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotTickerBookTicker,
            _ => BingxEndpoint::SwapTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // BingX doesn't have dedicated ping endpoint, use symbols as health check
        let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;

        // Check response is valid
        if response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) == 0 {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;
                BingxParser::parse_spot_exchange_info(&response)
            }
            _ => {
                let response = self.get(BingxEndpoint::SwapContracts, HashMap::new(), AccountType::FuturesCross).await?;
                BingxParser::parse_swap_exchange_info(&response)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BingxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let endpoint = if is_futures { BingxEndpoint::SwapOrder } else { BingxEndpoint::SpotOrder };
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), formatted_symbol);
        params.insert("side".to_string(), side_str.to_string());

        match req.order_type {
            OrderType::Market => {
                params.insert("type".to_string(), "MARKET".to_string());
                if is_futures {
                    params.insert("quantity".to_string(), quantity.to_string());
                } else {
                    // BingX Spot: buy uses quoteOrderQty, sell uses quantity
                    match side {
                        OrderSide::Buy => {
                            params.insert("quoteOrderQty".to_string(), quantity.to_string());
                        }
                        OrderSide::Sell => {
                            params.insert("quantity".to_string(), quantity.to_string());
                        }
                    }
                }
            }

            OrderType::Limit { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                if is_futures {
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
            }

            OrderType::PostOnly { price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "PostOnly is not documented for BingX Spot".to_string()
                    ));
                }
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("timeInForce".to_string(), "PostOnly".to_string());
            }

            OrderType::Ioc { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                if let Some(p) = price {
                    params.insert("price".to_string(), p.to_string());
                }
                params.insert("timeInForce".to_string(), "IOC".to_string());
            }

            OrderType::Fok { price } => {
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("timeInForce".to_string(), "FOK".to_string());
            }

            OrderType::StopMarket { stop_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopMarket is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "STOP_MARKET".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("stopPrice".to_string(), stop_price.to_string());
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopLimit is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "STOP".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), limit_price.to_string());
                params.insert("stopPrice".to_string(), stop_price.to_string());
                params.insert("timeInForce".to_string(), "GTC".to_string());
            }

            OrderType::TrailingStop { callback_rate, activation_price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "TrailingStop is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                params.insert("type".to_string(), "TRAILING_STOP_MARKET".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                // priceRate is the trailing distance as a percentage
                params.insert("priceRate".to_string(), callback_rate.to_string());
                if let Some(act_price) = activation_price {
                    params.insert("activationPrice".to_string(), act_price.to_string());
                }
            }

            OrderType::ReduceOnly { price } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly is only supported for BingX Swap (futures)".to_string()
                    ));
                }
                let (type_str, price_val) = if let Some(p) = price {
                    ("LIMIT", Some(p))
                } else {
                    ("MARKET", None)
                };
                params.insert("type".to_string(), type_str.to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("reduceOnly".to_string(), "true".to_string());
                if let Some(p) = price_val {
                    params.insert("price".to_string(), p.to_string());
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                if !is_futures {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Bracket orders are only supported for BingX Swap (futures)".to_string()
                    ));
                }
                // BingX Swap supports embedded TP and SL via takeProfit/stopLoss JSON objects
                // However, the current transport sends form-encoded params, not JSON body.
                // BingX Swap order API typically accepts query params with JSON strings.
                // For bracket orders we use a limit/market entry with embedded TP/SL.
                let type_str = if price.is_some() { "LIMIT" } else { "MARKET" };
                params.insert("type".to_string(), type_str.to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                if let Some(p) = price {
                    params.insert("price".to_string(), p.to_string());
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                }
                // Encode TP/SL as JSON params — BingX accepts these as JSON-encoded strings
                let tp_json = json!({
                    "type": "TAKE_PROFIT_MARKET",
                    "stopPrice": take_profit.to_string(),
                    "price": "0",
                    "workingType": "MARK_PRICE"
                });
                let sl_json = json!({
                    "type": "STOP_MARKET",
                    "stopPrice": stop_loss.to_string(),
                    "price": "0",
                    "workingType": "MARK_PRICE"
                });
                params.insert("takeProfit".to_string(), tp_json.to_string());
                params.insert("stopLoss".to_string(), sl_json.to_string());
            }

            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on BingX", req.order_type)
                ));
            }
        }

        let response = self.post(endpoint, params, account_type).await?;
        self.check_response(&response)?;

        // Extract order from response
        let data = response.get("data").cloned().unwrap_or(json!({}));
        let order_data = data.get("order").cloned()
            .or_else(|| Some(data.clone()))
            .unwrap_or(json!({}));
        let order_id = order_data.get("orderId")
            .and_then(|v| v.as_str().map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_default();
        let client_order_id = order_data.get("clientOrderId")
            .and_then(|v| v.as_str())
            .map(String::from);

        let price_for_order = match &req.order_type {
            OrderType::Limit { price } | OrderType::PostOnly { price } | OrderType::Fok { price } => Some(*price),
            OrderType::Ioc { price } => *price,
            OrderType::StopLimit { limit_price, .. } => Some(*limit_price),
            _ => None,
        };

        Ok(PlaceOrderResponse::Simple(
            Self::build_placed_order(order_id, client_order_id, &symbol, side, req.order_type, price_for_order, quantity)
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

                let endpoint = if is_futures { BingxEndpoint::SwapOrder } else { BingxEndpoint::SpotOrder };
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.delete(endpoint, params, account_type).await?;
                self.check_response(&response)?;
                BingxParser::parse_order(&response, &symbol.to_string())
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope — use CancelAll trait for all/bySymbol on BingX", req.scope)
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
            _ => BingxEndpoint::SwapOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_order(&response, &symbol.to_string())
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

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOpenOrders,
            _ => BingxEndpoint::SwapOpenOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let mut params = HashMap::new();

        if is_futures {
            // Futures AllOrders requires symbol
            let sym = filter.symbol.as_ref()
                .ok_or_else(|| ExchangeError::InvalidRequest(
                    "BingX Swap order history requires a symbol".to_string()
                ))?;
            params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
            if let Some(start) = filter.start_time {
                params.insert("startTime".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTime".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.to_string());
            }
            let response = self.get(BingxEndpoint::SwapAllOrders, params, account_type).await?;
            BingxParser::parse_orders(&response)
        } else {
            // Spot history orders
            if let Some(ref sym) = filter.symbol {
                params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
            }
            if let Some(start) = filter.start_time {
                params.insert("startTime".to_string(), start.to_string());
            }
            if let Some(end) = filter.end_time {
                params.insert("endTime".to_string(), end.to_string());
            }
            if let Some(limit) = filter.limit {
                params.insert("limit".to_string(), limit.to_string());
            }
            let response = self.get(BingxEndpoint::SpotHistoryOrders, params, account_type).await?;
            BingxParser::parse_orders(&response)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BingxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotBalance,
            _ => BingxEndpoint::SwapBalance,
        };

        let params = HashMap::new();
        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => BingxParser::parse_balances(&response),
            _ => BingxParser::parse_swap_balance(&response),
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default BingX fees, should query from API
            taker_commission: 0.1,
            balances,
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // BingX provides commission rates via dedicated endpoints
        // Try spot commission rate first — works for both spot and futures context
        let params = HashMap::new();
        let response = self.get(BingxEndpoint::SpotCommissionRate, params, AccountType::Spot).await;

        if let Ok(response) = response {
            self.check_response(&response)?;
            if let Some(data) = response.get("data") {
                let maker = data.get("makerCommissionRate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| data.get("maker").and_then(|v| v.as_f64()))
                    .unwrap_or(0.001);
                let taker = data.get("takerCommissionRate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| data.get("taker").and_then(|v| v.as_f64()))
                    .unwrap_or(0.001);
                return Ok(FeeInfo {
                    maker_rate: maker,
                    taker_rate: taker,
                    symbol: _symbol.map(String::from),
                    tier: None,
                });
            }
        }

        // Fallback to default BingX fees
        Ok(FeeInfo {
            maker_rate: 0.001,
            taker_rate: 0.001,
            symbol: _symbol.map(String::from),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BingxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(BingxEndpoint::SwapPositions, params, account_type).await?;
        BingxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let symbol_str = symbol;
        let sym = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));

        let response = self.get(BingxEndpoint::SwapFundingRate, params, account_type).await?;

        // Parse funding rate from response
        let data = response.get("data").cloned().unwrap_or(json!({}));
        let rate = data.get("fundingRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| data.get("fundingRate").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        let next_time = data.get("nextFundingTime")
            .and_then(|v| v.as_i64());

        Ok(FundingRate {
            symbol: symbol_str.to_string(),
            rate,
            next_funding_time: next_time,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Leverage not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), "LONG".to_string()); // BingX requires side
                params.insert("leverage".to_string(), leverage.to_string());

                let response = self.post(BingxEndpoint::SwapLeverage, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "MarginMode not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                let margin_type_str = match margin_type {
                    MarginType::Cross => "CROSSED",
                    MarginType::Isolated => "ISOLATED",
                };
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("marginType".to_string(), margin_type_str.to_string());

                let response = self.post(BingxEndpoint::SwapMarginType, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }
                // BingX: place a market order with closePosition=true
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), "SELL".to_string()); // for long position
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("closePosition".to_string(), "true".to_string());

                let response = self.post(BingxEndpoint::SwapOrder, params, account_type).await?;
                self.check_response(&response)?;
                Ok(())
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on BingX", req)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BingxConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        match scope {
            CancelScope::All { ref symbol } => {
                if is_futures {
                    // Swap cancel-all requires symbol
                    let sym = symbol.as_ref()
                        .ok_or_else(|| ExchangeError::InvalidRequest(
                            "BingX Swap cancel-all requires a symbol".to_string()
                        ))?;
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                    let response = self.delete(BingxEndpoint::SwapCancelAllOrders, params, account_type).await?;
                    self.check_response(&response)?;
                    Ok(CancelAllResponse {
                        cancelled_count: 0, // BingX doesn't return count in cancel-all response
                        failed_count: 0,
                        details: vec![],
                    })
                } else {
                    let sym = symbol.as_ref()
                        .ok_or_else(|| ExchangeError::InvalidRequest(
                            "BingX Spot cancel-all requires a symbol".to_string()
                        ))?;
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                    let response = self.delete(BingxEndpoint::SpotCancelAllOrders, params, account_type).await?;
                    self.check_response(&response)?;
                    Ok(CancelAllResponse {
                        cancelled_count: 0,
                        failed_count: 0,
                        details: vec![],
                    })
                }
            }

            CancelScope::BySymbol { ref symbol } => {
                let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted_symbol);

                let endpoint = if is_futures {
                    BingxEndpoint::SwapCancelAllOrders
                } else {
                    BingxEndpoint::SpotCancelAllOrders
                };

                let response = self.delete(endpoint, params, account_type).await?;
                self.check_response(&response)?;

                Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                "cancel_all_orders only supports All and BySymbol scopes".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (Swap only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BingxConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let symbol = &req.symbol;
        let account_type = req.account_type;
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        if !is_futures {
            return Err(ExchangeError::UnsupportedOperation(
                "AmendOrder is only supported for BingX Swap (futures). Spot requires cancel+replace.".to_string()
            ));
        }

        // POST /openApi/swap/v1/trade/amend
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), req.order_id.clone());

        if let Some(new_price) = req.fields.price {
            params.insert("price".to_string(), new_price.to_string());
        }
        if let Some(new_qty) = req.fields.quantity {
            params.insert("quantity".to_string(), new_qty.to_string());
        }
        if let Some(trigger) = req.fields.trigger_price {
            params.insert("stopPrice".to_string(), trigger.to_string());
        }

        let response = self.post(BingxEndpoint::SwapAmend, params, account_type).await?;
        self.check_response(&response)?;

        // Fetch the updated order
        self.get_order(&symbol.to_string(), &req.order_id, account_type).await
    }
}
