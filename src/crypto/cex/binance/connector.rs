//! # Binance Connector
//!
//! Реализация всех core трейтов для Binance.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    AmendRequest, CancelAllResponse, OrderResult,
    MarginType,
};
use crate::core::types::{ConnectorStats, SymbolInfo};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{BinanceUrls, BinanceEndpoint, format_symbol, map_kline_interval};
use super::auth::BinanceAuth;
use super::parser::BinanceParser;

// Binance endpoint weights (from API docs)
mod weights {
    pub const PING: u32 = 1;
    pub const KLINES: u32 = 2;
    pub const _DEPTH_DEFAULT: u32 = 5;   // limit 100
    pub const _DEPTH_500: u32 = 10;
    pub const TICKER_24H: u32 = 1;
    pub const _ALL_TICKERS: u32 = 40;
    pub const _TRADES: u32 = 10;
    pub const ORDER_BOOK: u32 = 5;
    pub const ACCOUNT: u32 = 10;
    pub const ORDER: u32 = 1;
    pub const DEFAULT: u32 = 1;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Binance коннектор
pub struct BinanceConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BinanceAuth>,
    /// URL'ы (mainnet/testnet)
    urls: BinanceUrls,
    /// Testnet mode
    testnet: bool,
    /// Weight-based rate limiter (6000 weight per minute)
    weight_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl BinanceConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            BinanceUrls::TESTNET
        } else {
            BinanceUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials
            .as_ref()
            .map(BinanceAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/api/v3/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(server_time) = response.get("serverTime").and_then(|t| t.as_i64()) {
                    if let Some(ref mut a) = auth {
                        a.sync_time(server_time);
                    }
                }
            }
        }

        // Initialize weight limiter: 6000 weight per minute
        let weight_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(6000, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            weight_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BINANCE-SPECIFIC PUBLIC METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Fetch up to `total_bars` klines with backward pagination.
    ///
    /// Binance limits to 1000 klines per request. This method chains
    /// multiple requests, walking backward in time from `end_time` (or now
    /// if `None`), until `total_bars` klines are collected or no more data
    /// is available.
    ///
    /// The returned slice is in chronological order (oldest first).
    pub async fn get_klines_paginated(
        &self,
        symbol: Symbol,
        interval: &str,
        total_bars: usize,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Kline>> {
        const LIMIT_PER_REQUEST: usize = 1000;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotKlines,
            _ => BinanceEndpoint::FuturesKlines,
        };

        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
        let interval_str = map_kline_interval(interval).to_string();

        let mut all_klines: Vec<Kline> = Vec::with_capacity(total_bars);
        let mut end_time: Option<i64> = None; // None = latest (now)

        loop {
            let mut params = HashMap::new();
            params.insert("symbol".to_string(), symbol_str.clone());
            params.insert("interval".to_string(), interval_str.clone());
            params.insert("limit".to_string(), LIMIT_PER_REQUEST.to_string());

            if let Some(et) = end_time {
                params.insert("endTime".to_string(), et.to_string());
            }

            let response = self.get(endpoint, params, account_type).await?;
            let batch = BinanceParser::parse_klines(&response)?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len();

            // Use the first bar's open_time - 1ms as the next endTime cursor,
            // so the next request fetches bars strictly before this batch.
            end_time = Some(batch[0].open_time - 1);

            // Prepend the batch to keep chronological order: older data goes first.
            let mut combined = batch;
            combined.append(&mut all_klines);
            all_klines = combined;

            if all_klines.len() >= total_bars {
                break;
            }

            // If the exchange returned fewer bars than the limit, there is no
            // more historical data available.
            if batch_len < LIMIT_PER_REQUEST {
                break;
            }
        }

        // Trim to the requested count, keeping the most recent bars.
        if all_klines.len() > total_bars {
            all_klines = all_klines.split_off(all_klines.len() - total_bars);
        }

        Ok(all_klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RATE LIMITING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary before making a request
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.weight_limiter.lock()
                    .expect("Weight limiter mutex poisoned");
                if limiter.try_acquire(weight) {
                    return;
                }
                limiter.time_until_ready(weight)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Обновить weight limiter из заголовка ответа X-MBX-USED-WEIGHT-1M
    fn update_weight_from_headers(&self, headers: &reqwest::header::HeaderMap) {
        if let Some(weight) = headers
            .get("X-MBX-USED-WEIGHT-1M")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if let Ok(mut limiter) = self.weight_limiter.lock() {
                limiter.update_from_server(weight);
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BinanceEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit check with per-endpoint weights
        let weight = match endpoint {
            BinanceEndpoint::Ping => weights::PING,
            BinanceEndpoint::SpotKlines | BinanceEndpoint::FuturesKlines => weights::KLINES,
            BinanceEndpoint::SpotOrderbook | BinanceEndpoint::FuturesOrderbook => weights::ORDER_BOOK,
            BinanceEndpoint::SpotTicker | BinanceEndpoint::FuturesTicker => weights::TICKER_24H,
            BinanceEndpoint::SpotAccount | BinanceEndpoint::FuturesAccount => weights::ACCOUNT,
            BinanceEndpoint::SpotGetOrder | BinanceEndpoint::FuturesGetOrder => weights::ORDER,
            BinanceEndpoint::SpotOpenOrders | BinanceEndpoint::FuturesOpenOrders => weights::ORDER,
            BinanceEndpoint::FuturesPositions => weights::ACCOUNT,
            BinanceEndpoint::FundingRate => weights::DEFAULT,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth if needed
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

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        BinanceParser::check_error(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BinanceEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST endpoints: order placement/amend = weight 1
        let weight = match endpoint {
            BinanceEndpoint::SpotCreateOrder | BinanceEndpoint::FuturesCreateOrder => weights::ORDER,
            BinanceEndpoint::FuturesSetLeverage => weights::DEFAULT,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Auth required for POST
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let headers = auth.sign_request(&mut params);

        // Build query string (Binance uses query params for POST too)
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // POST with empty body, params in query string
        let (response, resp_headers) = self.http.post_with_response_headers(&url, &json!({}), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        BinanceParser::check_error(&response)?;
        Ok(response)
    }

    /// PUT запрос (for order amend)
    async fn put(
        &self,
        endpoint: BinanceEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(weights::ORDER).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let headers = auth.sign_request(&mut params);

        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        // HttpClient::put does not return headers; use it directly
        let response = self.http.put(&url, &json!({}), &headers).await?;
        BinanceParser::check_error(&response)?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: BinanceEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // DELETE endpoints: cancel order = weight 1
        let weight = match endpoint {
            BinanceEndpoint::SpotCancelOrder | BinanceEndpoint::FuturesCancelOrder => weights::ORDER,
            _ => weights::DEFAULT,
        };
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Auth required for DELETE
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let headers = auth.sign_request(&mut params);

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

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        BinanceParser::check_error(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BinanceConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Binance
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.weight_limiter.lock() {
            (limiter.current_weight(), limiter.max_weight())
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BinanceConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotPrice,
            _ => BinanceEndpoint::FuturesPrice,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotOrderbook,
            _ => BinanceEndpoint::FuturesOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_orderbook(&response)
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
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotKlines,
            _ => BinanceEndpoint::FuturesKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotTicker,
            _ => BinanceEndpoint::FuturesTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(BinanceEndpoint::Ping, HashMap::new(), AccountType::Spot).await?;
        BinanceParser::check_error(&response)
    }

    /// Получить информацию о всех торговых символах биржи
    ///
    /// Returns only symbols with `status == "TRADING"`.
    /// Use `AccountType::Spot` for spot markets, any other for futures.
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let endpoint = match account_type {
            AccountType::Spot => BinanceEndpoint::SpotExchangeInfo,
            _ => BinanceEndpoint::FuturesExchangeInfo,
        };
        let response = self.get(endpoint, HashMap::new(), account_type).await?;
        BinanceParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BinanceConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol;
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), match side {
                    OrderSide::Buy => "BUY".to_string(),
                    OrderSide::Sell => "SELL".to_string(),
                });
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("quantity".to_string(), quantity.to_string());

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }
            OrderType::Limit { price } => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("timeInForce".to_string(), "GTC".to_string());

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::StopMarket { stop_price } => {
                // Spot: no native STOP_MARKET. Futures only.
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "StopMarket not supported on Spot/Margin (Binance Futures only)".to_string()
                        ));
                    }
                    _ => {}
                }

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "STOP_MARKET".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("stopPrice".to_string(), stop_price.to_string());

                if req.reduce_only {
                    params.insert("reduceOnly".to_string(), "true".to_string());
                }
                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(BinanceEndpoint::FuturesCreateOrder, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("stopPrice".to_string(), stop_price.to_string());
                params.insert("price".to_string(), limit_price.to_string());

                // Spot uses STOP_LOSS_LIMIT / TAKE_PROFIT_LIMIT; Futures uses STOP
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        params.insert("type".to_string(), "STOP_LOSS_LIMIT".to_string());
                        params.insert("timeInForce".to_string(), "GTC".to_string());
                    }
                    _ => {
                        params.insert("type".to_string(), "STOP".to_string());
                        params.insert("timeInForce".to_string(), "GTC".to_string());
                    }
                }

                if req.reduce_only {
                    params.insert("reduceOnly".to_string(), "true".to_string());
                }
                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::TrailingStop { callback_rate, activation_price } => {
                // Futures only
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "TrailingStop not supported on Spot/Margin (Binance Futures only)".to_string()
                        ));
                    }
                    _ => {}
                }

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "TRAILING_STOP_MARKET".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("callbackRate".to_string(), callback_rate.to_string());

                if let Some(ap) = activation_price {
                    params.insert("activationPrice".to_string(), ap.to_string());
                }
                if req.reduce_only {
                    params.insert("reduceOnly".to_string(), "true".to_string());
                }
                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(BinanceEndpoint::FuturesCreateOrder, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // Spot only — Binance OCO is not available on Futures
                match account_type {
                    AccountType::Spot | AccountType::Margin => {}
                    _ => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "OCO orders not supported on Futures (Binance Spot only)".to_string()
                        ));
                    }
                }

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("stopPrice".to_string(), stop_price.to_string());

                if let Some(slp) = stop_limit_price {
                    params.insert("stopLimitPrice".to_string(), slp.to_string());
                    params.insert("stopLimitTimeInForce".to_string(), "GTC".to_string());
                }
                if let Some(cid) = &req.client_order_id {
                    params.insert("listClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(BinanceEndpoint::SpotOcoOrder, params, account_type).await?;
                let oco = BinanceParser::parse_oco_response(&response)?;
                Ok(PlaceOrderResponse::Oco(oco))
            }

            OrderType::Bracket { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Bracket orders not natively supported on Binance. Use separate TP/SL orders.".to_string()
                ))
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Spot only — Binance Futures does not support iceberg
                match account_type {
                    AccountType::Spot | AccountType::Margin => {}
                    _ => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Iceberg orders not supported on Futures (Binance Spot only)".to_string()
                        ));
                    }
                }

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("icebergQty".to_string(), display_quantity.to_string());
                params.insert("timeInForce".to_string(), "GTC".to_string());

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(BinanceEndpoint::SpotCreateOrder, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::Twap { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "TWAP orders not supported via standard Binance API".to_string()
                ))
            }

            OrderType::PostOnly { price } => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                // GTX = Post-Only on Binance (Good Till Crossing)
                params.insert("timeInForce".to_string(), "GTX".to_string());

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::Ioc { price } => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("timeInForce".to_string(), "IOC".to_string());

                // Use the provided price, or fall back to a limit order at market
                if let Some(p) = price {
                    params.insert("price".to_string(), p.to_string());
                } else {
                    // IOC with no price — use MARKET type instead
                    params.insert("type".to_string(), "MARKET".to_string());
                    params.remove("timeInForce");
                }

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::Fok { price } => {
                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCreateOrder,
                    _ => BinanceEndpoint::FuturesCreateOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), quantity.to_string());
                params.insert("price".to_string(), price.to_string());
                params.insert("timeInForce".to_string(), "FOK".to_string());

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(endpoint, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }

            OrderType::Gtd { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "GTD (Good-Till-Date) not supported on Binance".to_string()
                ))
            }

            OrderType::ReduceOnly { price } => {
                // Futures only
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ReduceOnly not supported on Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), side.as_str().to_string());
                params.insert("reduceOnly".to_string(), "true".to_string());
                params.insert("quantity".to_string(), quantity.to_string());

                if let Some(p) = price {
                    params.insert("type".to_string(), "LIMIT".to_string());
                    params.insert("price".to_string(), p.to_string());
                    params.insert("timeInForce".to_string(), "GTC".to_string());
                } else {
                    params.insert("type".to_string(), "MARKET".to_string());
                }

                if let Some(cid) = &req.client_order_id {
                    params.insert("newClientOrderId".to_string(), cid.clone());
                }

                let response = self.post(BinanceEndpoint::FuturesCreateOrder, params, account_type).await?;
                let order = BinanceParser::parse_order(&response, &symbol.to_string())?;
                Ok(PlaceOrderResponse::Simple(order))
            }
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?;
                let account_type = req.account_type;

                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCancelOrder,
                    _ => BinanceEndpoint::FuturesCancelOrder,
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.delete(endpoint, params, account_type).await?;
                BinanceParser::parse_order(&response, &symbol.to_string())
            }
            CancelScope::Batch { .. } => {
                // Batch cancel is handled by BatchOrders trait; not available via Trading::cancel_order
                Err(ExchangeError::UnsupportedOperation(
                    "Use BatchOrders::cancel_orders_batch for batch cancellation on Binance".to_string()
                ))
            }
            CancelScope::All { .. } | CancelScope::BySymbol { .. } => {
                // Delegate to CancelAll logic but return a placeholder order since Trading::cancel_order
                // returns a single Order. Users should call CancelAll::cancel_all_orders instead.
                Err(ExchangeError::UnsupportedOperation(
                    "Use CancelAll::cancel_all_orders for cancel-all on Binance".to_string()
                ))
            }
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into base/quote for format_symbol
        let parts: Vec<&str> = symbol.split('/').collect();
        let (base, quote) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (symbol, "")
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotGetOrder,
            _ => BinanceEndpoint::FuturesGetOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), if quote.is_empty() {
            symbol.to_string()
        } else {
            format_symbol(base, quote, account_type)
        });
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_order(&response, symbol)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotOpenOrders,
            _ => BinanceEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            let parts: Vec<&str> = s.split('/').collect();
            let formatted = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                s.to_string()
            };
            params.insert("symbol".to_string(), formatted);
        }

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotAllOrders,
            _ => BinanceEndpoint::FuturesAllOrders,
        };

        let mut params = HashMap::new();

        // Symbol is required for Binance allOrders endpoint
        if let Some(ref sym) = filter.symbol {
            params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
        } else {
            return Err(ExchangeError::InvalidRequest(
                "Symbol is required for get_order_history on Binance".to_string()
            ));
        }

        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BinanceConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.as_deref();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotAccount,
            _ => BinanceEndpoint::FuturesAccount,
        };

        let mut params = HashMap::new();
        // Optionally exclude zero balances
        if matches!(account_type, AccountType::Spot | AccountType::Margin) {
            params.insert("omitZeroBalances".to_string(), "true".to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => BinanceParser::parse_balances(&response),
            _ => BinanceParser::parse_futures_balances(&response),
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotAccount,
            _ => BinanceEndpoint::FuturesAccount,
        };

        let mut params = HashMap::new();
        if matches!(account_type, AccountType::Spot | AccountType::Margin) {
            params.insert("omitZeroBalances".to_string(), "false".to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;

        let balances = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceParser::parse_balances(&response)?,
            _ => BinanceParser::parse_futures_balances(&response)?,
        };

        // Parse commission rates
        let (maker_commission, taker_commission) = if let Some(rates) = response.get("commissionRates") {
            let maker = rates.get("maker")
                .and_then(|m| m.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|r| r * 100.0) // Convert to percentage
                .unwrap_or(0.1);
            let taker = rates.get("taker")
                .and_then(|t| t.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|r| r * 100.0)
                .unwrap_or(0.1);
            (maker, taker)
        } else {
            (0.1, 0.1) // Default
        };

        Ok(AccountInfo {
            account_type,
            can_trade: response.get("canTrade").and_then(|c| c.as_bool()).unwrap_or(true),
            can_withdraw: response.get("canWithdraw").and_then(|c| c.as_bool()).unwrap_or(true),
            can_deposit: response.get("canDeposit").and_then(|c| c.as_bool()).unwrap_or(true),
            maker_commission,
            taker_commission,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Priority order:
        // 1. GET /sapi/v1/asset/tradeFee  — spot per-symbol rates (best accuracy)
        // 2. GET /fapi/v1/commissionRate  — futures per-symbol rates (when symbol given)
        // 3. GET /api/v3/account          — spot account-wide commissionRates fallback
        // 4. GET /fapi/v2/account         — futures feeTier fallback (tier → estimated rates)

        let formatted_symbol = symbol.map(|s| s.replace('/', "").to_uppercase());

        // Attempt 1: Spot /sapi trade fee (per-symbol or account-wide)
        let mut spot_params = HashMap::new();
        if let Some(ref sym) = formatted_symbol {
            spot_params.insert("symbol".to_string(), sym.clone());
        }
        match self.get(BinanceEndpoint::SpotTradeFee, spot_params, AccountType::Spot).await {
            Ok(response) => return BinanceParser::parse_fee_info(&response, symbol),
            Err(_) => {}
        }

        // Attempt 2: Futures /fapi/v1/commissionRate (requires symbol)
        if let Some(ref sym) = formatted_symbol {
            let mut futures_params = HashMap::new();
            futures_params.insert("symbol".to_string(), sym.clone());
            match self.get(
                BinanceEndpoint::FuturesCommissionRate,
                futures_params,
                AccountType::FuturesCross,
            ).await {
                Ok(response) => return BinanceParser::parse_fee_info(&response, symbol),
                Err(_) => {}
            }
        }

        // Attempt 3: Spot account commissionRates
        let mut account_params = HashMap::new();
        account_params.insert("omitZeroBalances".to_string(), "true".to_string());
        match self.get(BinanceEndpoint::SpotAccount, account_params, AccountType::Spot).await {
            Ok(response) => return BinanceParser::parse_fee_info(&response, symbol),
            Err(_) => {}
        }

        // Attempt 4: Futures account feeTier
        let response = self.get(BinanceEndpoint::FuturesAccount, HashMap::new(), AccountType::FuturesCross).await?;
        BinanceParser::parse_fee_info(&response, symbol)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BinanceConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol;
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

        let response = self.get(BinanceEndpoint::FuturesPositions, params, account_type).await?;
        BinanceParser::parse_positions(&response)
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

        // Parse symbol string into parts for format_symbol
        let parts: Vec<&str> = symbol.split('/').collect();
        let formatted = if parts.len() == 2 {
            format_symbol(parts[0], parts[1], account_type)
        } else {
            symbol.to_string()
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), formatted);
        params.insert("limit".to_string(), "1".to_string());

        let response = self.get(BinanceEndpoint::FundingRate, params, account_type).await?;
        BinanceParser::parse_funding_rate(&response)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Leverage not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("leverage".to_string(), leverage.to_string());

                let response = self.post(BinanceEndpoint::FuturesSetLeverage, params, account_type).await?;
                BinanceParser::check_error(&response)?;

                Ok(())
            }

            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetMarginMode not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let margin_type_str = match margin_type {
                    MarginType::Isolated => "ISOLATED",
                    MarginType::Cross => "CROSSED",
                };

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("marginType".to_string(), margin_type_str.to_string());

                let response = self.post(BinanceEndpoint::FuturesSetMarginType, params, account_type).await?;
                BinanceParser::check_error(&response)?;
                Ok(())
            }

            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("amount".to_string(), amount.to_string());
                params.insert("type".to_string(), "1".to_string()); // 1 = add margin

                let response = self.post(BinanceEndpoint::FuturesPositionMargin, params, account_type).await?;
                BinanceParser::check_error(&response)?;
                Ok(())
            }

            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "RemoveMargin not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("amount".to_string(), amount.to_string());
                params.insert("type".to_string(), "2".to_string()); // 2 = remove margin

                let response = self.post(BinanceEndpoint::FuturesPositionMargin, params, account_type).await?;
                BinanceParser::check_error(&response)?;
                Ok(())
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                // Get the open position to find its quantity
                let positions = self.get_positions(PositionQuery {
                    symbol: Some(symbol.clone()),
                    account_type,
                }).await?;

                let position = positions.into_iter().next()
                    .ok_or_else(|| ExchangeError::InvalidRequest(
                        format!("No open position found for {}", symbol)
                    ))?;

                // Place a reduce-only market order in the opposite direction
                let close_side = if position.side == crate::core::PositionSide::Long {
                    OrderSide::Sell
                } else {
                    OrderSide::Buy
                };

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("side".to_string(), close_side.as_str().to_string());
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("quantity".to_string(), position.quantity.to_string());
                params.insert("reduceOnly".to_string(), "true".to_string());

                let response = self.post(BinanceEndpoint::FuturesCreateOrder, params, account_type).await?;
                BinanceParser::check_error(&response)?;
                Ok(())
            }

            PositionModification::SetTpSl { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SetTpSl is not a single native endpoint on Binance. Place separate TP/SL orders.".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders (optionally filtered to a single symbol).
///
/// - Spot: `DELETE /api/v3/openOrders` — requires `symbol` param
/// - Futures: `DELETE /fapi/v1/allOpenOrders` — requires `symbol` param
///
/// Note: Binance requires `symbol` on both endpoints; passing `All` with
/// `symbol = None` is not supported and returns `UnsupportedOperation`.
#[async_trait]
impl CancelAll for BinanceConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let symbol = match &scope {
            CancelScope::All { symbol } => symbol.clone(),
            CancelScope::BySymbol { symbol } => Some(symbol.clone()),
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        };

        let sym = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "Binance cancel-all requires a symbol. Pass CancelScope::BySymbol or CancelScope::All with Some(symbol).".to_string()
        ))?;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotCancelAllOrders,
            _ => BinanceEndpoint::FuturesCancelAllOrders,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));

        let response = self.delete(endpoint, params, account_type).await?;
        BinanceParser::parse_cancel_all_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Modify a live futures order in-place.
///
/// Binance Futures: `PUT /fapi/v1/order`
/// Spot does NOT support amend — this returns `UnsupportedOperation` for Spot/Margin.
#[async_trait]
impl AmendOrder for BinanceConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        match req.account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Amend order not supported on Spot/Margin (Binance Futures only)".to_string()
                ));
            }
            _ => {}
        }

        // At least one field must be changed
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price or quantity must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&req.symbol.base, &req.symbol.quote, account_type));
        params.insert("orderId".to_string(), req.order_id.clone());

        if let Some(price) = req.fields.price {
            params.insert("price".to_string(), price.to_string());
        }
        if let Some(quantity) = req.fields.quantity {
            params.insert("quantity".to_string(), quantity.to_string());
        }
        if let Some(stop_price) = req.fields.trigger_price {
            params.insert("stopPrice".to_string(), stop_price.to_string());
        }

        let response = self.put(BinanceEndpoint::FuturesAmendOrder, params, account_type).await?;
        BinanceParser::parse_order(&response, &req.symbol.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement and cancellation.
///
/// - Futures: `POST /fapi/v1/batchOrders` — max 5 orders per batch
/// - Spot: no native batch endpoint → returns `UnsupportedOperation`
#[async_trait]
impl BatchOrders for BinanceConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        // Detect account type from first order — all orders in batch must be same type
        let account_type = orders[0].account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Batch orders not supported on Spot/Margin (Binance Futures only)".to_string()
                ));
            }
            _ => {}
        }

        if orders.len() > self.max_batch_place_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds Binance limit of {}", orders.len(), self.max_batch_place_size())
            ));
        }

        // Build each order as a JSON object for the batchOrders array
        let batch_orders_json: Vec<serde_json::Value> = orders.iter().map(|req| {
            let mut obj = serde_json::Map::new();
            obj.insert("symbol".to_string(), json!(format_symbol(&req.symbol.base, &req.symbol.quote, account_type)));
            obj.insert("side".to_string(), json!(req.side.as_str()));

            match &req.order_type {
                OrderType::Market => {
                    obj.insert("type".to_string(), json!("MARKET"));
                    obj.insert("quantity".to_string(), json!(req.quantity.to_string()));
                }
                OrderType::Limit { price } => {
                    obj.insert("type".to_string(), json!("LIMIT"));
                    obj.insert("quantity".to_string(), json!(req.quantity.to_string()));
                    obj.insert("price".to_string(), json!(price.to_string()));
                    obj.insert("timeInForce".to_string(), json!("GTC"));
                }
                _ => {
                    // For other types, encode as MARKET (best-effort fallback)
                    obj.insert("type".to_string(), json!("MARKET"));
                    obj.insert("quantity".to_string(), json!(req.quantity.to_string()));
                }
            }

            if req.reduce_only {
                obj.insert("reduceOnly".to_string(), json!("true"));
            }
            if let Some(ref cid) = req.client_order_id {
                obj.insert("newClientOrderId".to_string(), json!(cid));
            }

            serde_json::Value::Object(obj)
        }).collect();

        let batch_json_str = serde_json::to_string(&batch_orders_json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize batch orders: {}", e)))?;

        let mut params = HashMap::new();
        params.insert("batchOrders".to_string(), batch_json_str);

        let response = self.post(BinanceEndpoint::FuturesBatchOrders, params, account_type).await?;
        BinanceParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Batch cancel not supported on Spot/Margin (Binance Futures only)".to_string()
                ));
            }
            _ => {}
        }

        let sym = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "Symbol is required for batch cancel on Binance".to_string()
        ))?;

        // Futures batch cancel: DELETE /fapi/v1/batchOrders with orderIdList param
        let order_ids_json = serde_json::to_string(&order_ids)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize order IDs: {}", e)))?;

        let mut params = HashMap::new();
        // Symbol for batch cancel needs to be formatted — we have it as a raw string
        params.insert("symbol".to_string(), sym.replace('/', "").to_uppercase());
        params.insert("orderIdList".to_string(), order_ids_json);

        let response = self.delete(BinanceEndpoint::FuturesBatchOrders, params, account_type).await?;
        BinanceParser::parse_batch_orders_response(&response)
    }

    fn max_batch_place_size(&self) -> usize {
        5 // Binance Futures limit
    }

    fn max_batch_cancel_size(&self) -> usize {
        10 // Binance Futures limit
    }
}
