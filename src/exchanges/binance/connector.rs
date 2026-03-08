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
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::types::{ConnectorStats, SymbolInfo};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
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
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
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
        BinanceParser::parse_order(&response, &symbol.to_string())
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
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
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("quantity".to_string(), quantity.to_string());
        params.insert("price".to_string(), price.to_string());
        params.insert("timeInForce".to_string(), "GTC".to_string());

        let response = self.post(endpoint, params, account_type).await?;
        BinanceParser::parse_order(&response, &symbol.to_string())
    }

    async fn cancel_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
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

    async fn get_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotGetOrder,
            _ => BinanceEndpoint::FuturesGetOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(endpoint, params, account_type).await?;
        BinanceParser::parse_order(&response, &symbol.to_string())
    }

    async fn get_open_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BinanceEndpoint::SpotOpenOrders,
            _ => BinanceEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
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
    async fn get_balance(
        &self,
        _asset: Option<crate::core::Asset>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BinanceConnector {
    async fn get_positions(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
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
        symbol: Symbol,
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

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("limit".to_string(), "1".to_string());

        let response = self.get(BinanceEndpoint::FundingRate, params, account_type).await?;
        BinanceParser::parse_funding_rate(&response)
    }

    async fn set_leverage(
        &self,
        symbol: Symbol,
        leverage: u32,
        account_type: AccountType,
    ) -> ExchangeResult<()> {
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
        params.insert("leverage".to_string(), leverage.to_string());

        let response = self.post(BinanceEndpoint::FuturesSetLeverage, params, account_type).await?;
        BinanceParser::check_error(&response)?;

        Ok(())
    }
}
