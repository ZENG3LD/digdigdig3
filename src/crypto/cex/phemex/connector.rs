//! # Phemex Connector
//!
//! Реализация всех core трейтов для Phemex.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## CRITICAL: Value Scaling
//! Phemex uses integer representation with scale factors.
//! Always fetch /public/products first to get scale factors!

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
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
    CancelAll, AmendOrder, AccountTransfers, CustodialFunds, SubAccounts,
    UserTrade, UserTradeFilter,
};
use crate::core::types::{
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult, SubAccount,
};
use crate::core::types::SymbolInfo;
use crate::core::types::ConnectorStats;
use crate::core::utils::GroupRateLimiter;
use crate::core::utils::PrecisionCache;

use super::endpoints::{PhemexUrls, PhemexEndpoint, format_symbol, map_kline_interval, scale_price, scale_value};
use super::auth::PhemexAuth;
use super::parser::PhemexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Phemex коннектор
pub struct PhemexConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<PhemexAuth>,
    /// URL'ы (mainnet/testnet)
    urls: PhemexUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter with per-group pools (OTHERS/CONTRACT/SPOTORDER)
    rate_limiter: Arc<Mutex<GroupRateLimiter>>,
    /// Default price scale (used when scale is not provided)
    default_price_scale: u8,
    /// Default value scale
    default_value_scale: u8,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl PhemexConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            PhemexUrls::TESTNET
        } else {
            PhemexUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(PhemexAuth::new)
            .transpose()?;

        // Initialize grouped rate limiter per Phemex API docs
        let mut group_limiter = GroupRateLimiter::new();
        group_limiter.add_group("OTHERS", 100, Duration::from_secs(60));
        group_limiter.add_group("CONTRACT", 500, Duration::from_secs(60));
        group_limiter.add_group("SPOTORDER", 500, Duration::from_secs(60));
        let rate_limiter = Arc::new(Mutex::new(group_limiter));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            default_price_scale: 4, // BTCUSD default
            default_value_scale: 8, // BTC default
            precision: PrecisionCache::new(),
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    /// Get price scale for symbol based on account type
    /// Spot symbols use priceScale=8, Contract symbols use priceScale=4
    fn get_price_scale(&self, account_type: AccountType) -> u8 {
        match account_type {
            AccountType::Spot => 8,
            _ => 4,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Phemex response headers
    ///
    /// Phemex reports: x-ratelimit-remaining-<GROUP> = remaining (e.g. x-ratelimit-remaining-others)
    /// Parse the group name from the header and update the matching group.
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let prefix = "x-ratelimit-remaining-";
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            if let Some(group_lower) = header_name.strip_prefix(prefix) {
                // Map header suffix to group name
                let group = match group_lower {
                    "contract" => "CONTRACT",
                    "spotorder" => "SPOTORDER",
                    _ => "OTHERS",
                };
                if let Some(remaining) = value.to_str().ok().and_then(|s| s.parse::<u32>().ok()) {
                    // Determine max for this group to compute used = max - remaining
                    let max = if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.group_stats(group).map(|(_, m)| m).unwrap_or(100)
                    } else {
                        continue;
                    };
                    let used = max.saturating_sub(remaining);
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.update_from_server(group, used);
                    }
                }
            }
        }
    }

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, group: &str, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(group, weight) {
                    return;
                }
                limiter.time_until_ready(group, weight)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.get_weighted(endpoint, params, _account_type, 1).await
    }

    /// GET запрос with explicit weight (for endpoints that cost more, e.g. klines = 10)
    async fn get_weighted(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
        weight: u32,
    ) -> ExchangeResult<Value> {
        // Route to group based on whether endpoint is authenticated (order mgmt) or public (market data)
        let group = if endpoint.requires_auth() { "CONTRACT" } else { "OTHERS" };
        self.rate_limit_wait(group, weight).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request(path, &query, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: PhemexEndpoint,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST is always order/account — use SPOTORDER for Spot, CONTRACT otherwise
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request(path, "", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// PUT запрос
    async fn put(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // PUT is always for order/position management
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();

        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request(path, &query, &body_str);

        self.http.put(&url, &body, &headers).await
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // DELETE is always for order cancellation
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();

        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(path, &query, "");

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for PhemexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Phemex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max, rate_groups) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            let (used, max) = limiter.primary_stats();
            let groups = limiter.all_stats()
                .into_iter()
                .map(|(name, cur, mx)| (name.to_string(), cur, mx))
                .collect();
            (used, max, groups)
        } else {
            (0, 0, Vec::new())
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups,
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
impl MarketData for PhemexConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let ticker = self.get_ticker(symbol, account_type).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotOrderbook,
            _ => PhemexEndpoint::ContractOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_orderbook(&response, price_scale)
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
            AccountType::Spot => PhemexEndpoint::SpotKlines,
            _ => PhemexEndpoint::ContractKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("resolution".to_string(), map_kline_interval(interval).to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        // Phemex v2 requires BOTH from + to together.
        // "to" alone is silently ignored.
        if let Some(et) = end_time {
            let interval_s = interval_to_secs(interval) as i64;
            let count = limit.unwrap_or(500) as i64;
            let to_s = et / 1000;
            let from_s = to_s - count * interval_s;
            params.insert("from".to_string(), from_s.to_string());
            params.insert("to".to_string(), to_s.to_string());
        }

        // Kline endpoint has weight=10 per Phemex API docs
        let response = self.get_weighted(endpoint, params, account_type, 10).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_klines(&response, price_scale)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotTicker24h,
            _ => PhemexEndpoint::ContractTicker24h,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_ticker(&response, price_scale, account_type)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _response = self.get(PhemexEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(PhemexEndpoint::Products, HashMap::new(), AccountType::Spot).await?;
        let info = PhemexParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&info);
        Ok(info)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for PhemexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side {
            OrderSide::Buy => "Buy",
            OrderSide::Sell => "Sell",
        };

        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotCreateOrder,
            _ => PhemexEndpoint::ContractCreateOrder,
        };

        match req.order_type {
            OrderType::Market => {
                let body = match account_type {
                    AccountType::Spot => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Market",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                    }),
                    _ => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Market",
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let body = match account_type {
                    AccountType::Spot => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Limit",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "GoodTillCancel",
                    }),
                    _ => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Limit",
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "GoodTillCancel",
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // Phemex ordType="Stop" for stop-market (contract only)
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopMarket not supported for Spot on Phemex".to_string()
                    ));
                }
                let body = json!({
                    "symbol": symbol_str,
                    "side": side_str,
                    "orderQty": quantity as i64,
                    "ordType": "Stop",
                    "stopPxEp": scale_price(stop_price, self.default_price_scale),
                    "triggerType": "ByLastPrice",
                });
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Phemex ordType="StopLimit"
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "StopLimit not supported for Spot on Phemex".to_string()
                    ));
                }
                let body = json!({
                    "symbol": symbol_str,
                    "side": side_str,
                    "orderQty": quantity as i64,
                    "ordType": "StopLimit",
                    "priceEp": scale_price(limit_price, self.default_price_scale),
                    "stopPxEp": scale_price(stop_price, self.default_price_scale),
                    "triggerType": "ByLastPrice",
                    "timeInForce": "GoodTillCancel",
                });
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::PostOnly { price } => {
                // Phemex PostOnly: Limit with timeInForce="PostOnly"
                let body = match account_type {
                    AccountType::Spot => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Limit",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "PostOnly",
                    }),
                    _ => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Limit",
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "PostOnly",
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // ImmediateOrCancel — if price is Some use Limit IOC, else Market
                let body = match (account_type, price) {
                    (AccountType::Spot, Some(p)) => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Limit",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                        "priceEp": scale_price(p, self.default_price_scale),
                        "timeInForce": "ImmediateOrCancel",
                    }),
                    (AccountType::Spot, None) => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Market",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                        "timeInForce": "ImmediateOrCancel",
                    }),
                    (_, Some(p)) => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Limit",
                        "priceEp": scale_price(p, self.default_price_scale),
                        "timeInForce": "ImmediateOrCancel",
                    }),
                    (_, None) => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Market",
                        "timeInForce": "ImmediateOrCancel",
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::Fok { price } => {
                // FillOrKill — Limit with timeInForce="FillOrKill"
                let body = match account_type {
                    AccountType::Spot => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "ordType": "Limit",
                        "qtyType": "ByBase",
                        "baseQtyEv": scale_value(quantity, self.default_value_scale),
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "FillOrKill",
                    }),
                    _ => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Limit",
                        "priceEp": scale_price(price, self.default_price_scale),
                        "timeInForce": "FillOrKill",
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::ReduceOnly { price } => {
                // Contract only: Limit or Market with reduceOnly=true
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly not supported for Spot on Phemex".to_string()
                    ));
                }
                let body = match price {
                    Some(p) => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Limit",
                        "priceEp": scale_price(p, self.default_price_scale),
                        "timeInForce": "GoodTillCancel",
                        "reduceOnly": true,
                    }),
                    None => json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": "Market",
                        "reduceOnly": true,
                    }),
                };
                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // Phemex native Bracket order — ordType 11.
                // This is a first-class API type: entry + TP (ordType 12) + SL (ordType 14)
                // submitted as a single request on the standard contract order endpoint.
                // Contract only (Spot does not support bracket orders).
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Bracket orders not supported for Spot on Phemex".to_string()
                    ));
                }

                let tp_price_ep = scale_price(take_profit, self.default_price_scale);
                let sl_price_ep = scale_price(stop_loss, self.default_price_scale);

                let body = if let Some(p) = price {
                    json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": 11,                       // Bracket
                        "priceEp": scale_price(p, self.default_price_scale),
                        "timeInForce": "GoodTillCancel",
                        "takeProfitEp": tp_price_ep,
                        "stopLossEp": sl_price_ep,
                    })
                } else {
                    json!({
                        "symbol": symbol_str,
                        "side": side_str,
                        "orderQty": quantity as i64,
                        "ordType": 11,                       // Bracket (market entry)
                        "takeProfitEp": tp_price_ep,
                        "stopLossEp": sl_price_ep,
                    })
                };

                let response = self.post(endpoint, body, account_type).await?;
                PhemexParser::parse_bracket_order(&response, &symbol_str, self.default_price_scale)
                    .map(|b| PlaceOrderResponse::Bracket(Box::new(b)))
            }

            // Unsupported order types
            OrderType::TrailingStop { .. }
            | OrderType::Oco { .. }
            | OrderType::Iceberg { .. }
            | OrderType::Twap { .. }
            | OrderType::Gtd { .. }
            | OrderType::Oto { .. }
            | OrderType::ConditionalPlan { .. }
            | OrderType::DcaRecurring { .. } => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on Phemex", req.order_type)
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
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("orderID".to_string(), order_id.to_string());

                let endpoint = match account_type {
                    AccountType::Spot => PhemexEndpoint::SpotCancelOrder,
                    _ => PhemexEndpoint::ContractCancelOrder,
                };

                let response = self.delete(endpoint, params, account_type).await?;
                PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale)
            }

            CancelScope::All { ref symbol } => {
                // Use cancel-all endpoint, optionally filtered by symbol
                let account_type = req.account_type;
                let mut params = HashMap::new();

                if let Some(sym) = symbol {
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                }

                let endpoint = match account_type {
                    AccountType::Spot => PhemexEndpoint::SpotCancelAllOrders,
                    _ => PhemexEndpoint::ContractCancelAllOrders,
                };

                let _response = self.delete(endpoint, params, account_type).await?;
                // Return a minimal placeholder order (exchange returns list, not single)
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
                // Cancel all orders for a specific symbol
                let account_type = req.account_type;
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());

                let endpoint = match account_type {
                    AccountType::Spot => PhemexEndpoint::SpotCancelAllOrders,
                    _ => PhemexEndpoint::ContractCancelAllOrders,
                };

                let _response = self.delete(endpoint, params, account_type).await?;
                Ok(Order {
                    id: "cancel_by_symbol".to_string(),
                    client_order_id: None,
                    symbol: symbol_str,
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
                "Batch cancel not supported via cancel_order on Phemex; use CancelAll trait".to_string()
            )),
            CancelScope::ByLabel(_)
            | CancelScope::ByCurrencyKind { .. }
            | CancelScope::ScheduledAt(_) => Err(ExchangeError::UnsupportedOperation(
                "Phemex does not support this cancel scope".to_string()
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        // Only available for contracts
        if account_type == AccountType::Spot {
            return Err(ExchangeError::UnsupportedOperation(
                "Spot get_order not supported by Phemex API".to_string()
            ));
        }

        let mut params = HashMap::new();
        params.insert("orderID".to_string(), order_id.to_string());

        let response = self.get(PhemexEndpoint::ContractGetOrder, params, account_type).await?;
        PhemexParser::parse_order(&response, "", self.default_price_scale)
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

        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotOpenOrders,
            _ => PhemexEndpoint::ContractOpenOrders,
        };

        let response = self.get(endpoint, params, account_type).await?;
        PhemexParser::parse_orders(&response, self.default_price_scale)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // GET /exchange/order/list (requires symbol for contract)
        let mut params = HashMap::new();

        if let Some(ref sym) = filter.symbol {
            params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
        }

        if let Some(start) = filter.start_time {
            params.insert("start".to_string(), (start / 1000).to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), (end / 1000).to_string());
        }

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(200).to_string());
        }

        let response = self.get(PhemexEndpoint::ContractClosedOrders, params, account_type).await?;
        PhemexParser::parse_orders(&response, self.default_price_scale)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = HashMap::new();

        if let Some(sym) = &filter.symbol {
            // Accept "BTC/USDT" slash-separated or already formatted "BTCUSDT"
            let formatted = if sym.contains('/') {
                let parts: Vec<&str> = sym.split('/').collect();
                if parts.len() == 2 {
                    format_symbol(parts[0], parts[1], account_type)
                } else {
                    sym.to_uppercase()
                }
            } else {
                sym.to_uppercase()
            };
            params.insert("symbol".to_string(), formatted);
        }

        // Phemex expects seconds for start/end, not milliseconds
        if let Some(start_ms) = filter.start_time {
            params.insert("start".to_string(), (start_ms / 1000).to_string());
        }
        if let Some(end_ms) = filter.end_time {
            params.insert("end".to_string(), (end_ms / 1000).to_string());
        }
        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(200).to_string());
        }

        // Use the dedicated trade history endpoint
        let response = self.get(PhemexEndpoint::TradeHistory, params, account_type).await?;
        PhemexParser::parse_user_trades(&response, self.default_price_scale)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for PhemexConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let mut params = HashMap::new();

        match &scope {
            CancelScope::All { symbol } => {
                if let Some(sym) = symbol {
                    params.insert("symbol".to_string(), format_symbol(&sym.base, &sym.quote, account_type));
                }
            }
            CancelScope::BySymbol { symbol } => {
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
            }
            _ => return Err(ExchangeError::InvalidRequest(
                "cancel_all_orders requires CancelScope::All or BySymbol".to_string()
            )),
        }

        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotCancelAllOrders,
            _ => PhemexEndpoint::ContractCancelAllOrders,
        };

        let _response = self.delete(endpoint, params, account_type).await?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // Phemex doesn't return count in cancel-all response
            failed_count: 0,
            details: vec![],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for PhemexConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let symbol = req.symbol.clone();
        let account_type = req.account_type;
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotAmendOrder,
            _ => PhemexEndpoint::ContractAmendOrder,
        };

        let mut body = json!({
            "symbol": symbol_str,
            "orderID": req.order_id,
        });

        if let Some(price) = req.fields.price {
            body["priceEp"] = json!(scale_price(price, self.default_price_scale));
        }

        if let Some(qty) = req.fields.quantity {
            match account_type {
                AccountType::Spot => {
                    body["baseQtyEv"] = json!(scale_value(qty, self.default_value_scale));
                }
                _ => {
                    body["orderQty"] = json!(qty as i64);
                }
            }
        }

        if let Some(trigger) = req.fields.trigger_price {
            body["stopPxEp"] = json!(scale_price(trigger, self.default_price_scale));
        }

        let response = self.put(endpoint, HashMap::new(), body, account_type).await?;
        PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for PhemexConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;
        match account_type {
            AccountType::Spot => {
                let response = self.get(PhemexEndpoint::SpotWallets, HashMap::new(), account_type).await?;
                let mut balances = PhemexParser::parse_spot_balances(&response, self.default_value_scale)?;

                // Filter by asset if provided
                if let Some(a) = asset {
                    balances.retain(|b| b.asset == a);
                }

                Ok(balances)
            }
            _ => {
                // Contract account
                let mut params = HashMap::new();

                // Use provided asset or default to BTC
                let currency = asset.as_deref().unwrap_or("BTC");
                params.insert("currency".to_string(), currency.to_string());

                let response = self.get(PhemexEndpoint::ContractAccount, params, account_type).await?;
                PhemexParser::parse_contract_account(&response, self.default_value_scale)
            }
        }
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            balances,
            can_trade: true,
            can_withdraw: false,
            can_deposit: false,
            maker_commission: 0.0,
            taker_commission: 0.0,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Phemex doesn't expose a public fee endpoint; return standard fee tiers.
        // Maker: 0.01%, Taker: 0.06% (standard tier)
        Ok(FeeInfo {
            maker_rate: 0.0001,
            taker_rate: 0.0006,
            symbol: symbol.map(String::from),
            tier: Some("standard".to_string()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for PhemexConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        let mut params = HashMap::new();
        params.insert("currency".to_string(), "BTC".to_string());

        let response = self.get(PhemexEndpoint::Positions, params, account_type).await?;
        let mut positions = PhemexParser::parse_positions(&response, self.default_price_scale, self.default_value_scale)?;

        // Filter by symbol if provided
        if let Some(s) = symbol {
            let symbol_str = format_symbol(&s.base, &s.quote, account_type);
            positions.retain(|p| p.symbol == symbol_str);
        }

        Ok(positions)
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

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(PhemexEndpoint::FundingRateHistory, params, account_type).await?;
        PhemexParser::parse_funding_rate(&response)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                // leverageEr: positive = isolated, zero = cross
                let leverage_er = if account_type == AccountType::FuturesIsolated {
                    ((leverage as f64 / 100.0) * 100_000_000.0) as i64
                } else {
                    0i64
                };

                let body = json!({
                    "symbol": symbol_str,
                    "leverageEr": leverage_er,
                });

                let _response = self.put(PhemexEndpoint::SetLeverage, HashMap::new(), body, account_type).await?;
                Ok(())
            }

            PositionModification::SetMarginMode { ref symbol, ref margin_type, account_type } => {
                let symbol = symbol.clone();
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                // Phemex uses leverageEr: 0 = cross, positive = isolated
                // Toggle by setting leverage to 0 (cross) or a default (10x isolated)
                let leverage_er = match margin_type {
                    MarginType::Cross => 0i64,
                    MarginType::Isolated => 1_000_000i64, // ~10x default
                };

                let body = json!({
                    "symbol": symbol_str,
                    "leverageEr": leverage_er,
                });

                let _response = self.put(PhemexEndpoint::SetLeverage, HashMap::new(), body, account_type).await?;
                Ok(())
            }

            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                // POST /positions/assign — assign balance to isolated margin
                let body = json!({
                    "symbol": symbol_str,
                    "posBalanceEv": scale_value(amount, self.default_value_scale),
                    "add": true,
                });

                let _response = self.post(PhemexEndpoint::AssignBalance, body, account_type).await?;
                Ok(())
            }

            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                // POST /positions/assign with negative amount
                let body = json!({
                    "symbol": symbol_str,
                    "posBalanceEv": scale_value(amount, self.default_value_scale),
                    "add": false,
                });

                let _response = self.post(PhemexEndpoint::AssignBalance, body, account_type).await?;
                Ok(())
            }

            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                // Close position: market order with reduceOnly=true for the full position qty
                // Phemex doesn't have a dedicated close-position endpoint;
                // use a market sell/buy with reduceOnly=true
                // We use a very large quantity — exchange will cap at position size
                let body = json!({
                    "symbol": symbol_str,
                    "side": "Sell",
                    "orderQty": 999999999i64,
                    "ordType": "Market",
                    "reduceOnly": true,
                });

                let _response = self.post(PhemexEndpoint::ContractCreateOrder, body, account_type).await?;
                Ok(())
            }

            PositionModification::SetTpSl { .. } => {
                // Phemex supports TP/SL via order placement with ordType="TakeProfitLimit" or "Stop"
                // For simplicity, we return UnsupportedOperation as Phemex SetTpSl
                // requires separate orders for TP and SL (no unified endpoint)
                Err(ExchangeError::UnsupportedOperation(
                    "SetTpSl not supported as a single operation on Phemex; place separate TP/SL orders".to_string()
                ))
            }
            PositionModification::SwitchPositionMode { .. } => Err(ExchangeError::UnsupportedOperation(
                "SwitchPositionMode not supported on Phemex".to_string()
            )),
            PositionModification::MovePositions { .. } => Err(ExchangeError::UnsupportedOperation(
                "MovePositions not supported on Phemex".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountTransfers for PhemexConnector {
    /// Transfer between Spot and Futures account.
    ///
    /// Endpoint: POST /assets/transfer
    /// Params: amountEv (×10^8), currency, moveOp (1=spot→futures, 2=futures→spot)
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let move_op = match (&req.from_account, &req.to_account) {
            (AccountType::Spot, AccountType::FuturesCross)
            | (AccountType::Spot, AccountType::FuturesIsolated) => 1i64,
            _ => 2i64, // futures → spot
        };

        let amount_ev = scale_value(req.amount, self.default_value_scale);

        let body = serde_json::json!({
            "amountEv": amount_ev,
            "currency": req.asset,
            "moveOp": move_op,
        });

        let response = self.post(PhemexEndpoint::Transfer, body, AccountType::Spot).await?;

        // Phemex transfer response: {"code": 0, "msg": "", "data": {"linkKey": "..."}
        let transfer_id = response.get("data")
            .and_then(|d| d.get("linkKey"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string());

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: None,
        })
    }

    /// Transfer history is not available on Phemex as a standard endpoint.
    /// Returns empty vec rather than an error to keep callers functional.
    async fn get_transfer_history(
        &self,
        _filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        Ok(vec![])
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for PhemexConnector {
    /// Get deposit address for an asset on a given network.
    ///
    /// Endpoint: GET /exchange/wallets/v2/depositAddress
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), asset.to_string());
        if let Some(chain) = network {
            params.insert("chainName".to_string(), chain.to_string());
        }

        let response = self.get(PhemexEndpoint::DepositAddress, params, AccountType::Spot).await?;

        // Phemex response: {"code": 0, "data": {"address": "...", "tag": "..."}}
        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        let address = data.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'address' field".to_string()))?
            .to_string();
        let tag = data.get("tag")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let net = data.get("chainName")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| network.map(String::from));

        Ok(DepositAddress {
            address,
            tag,
            network: net,
            asset: asset.to_string(),
            created_at: None,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// Endpoint: POST /exchange/wallets/createWithdraw
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let mut body = serde_json::json!({
            "currency": req.asset,
            "address": req.address,
            "amount": req.amount.to_string(),
        });

        if let Some(chain) = &req.network {
            body["chainName"] = serde_json::json!(chain);
        }
        if let Some(tag) = &req.tag {
            body["addressTag"] = serde_json::json!(tag);
        }

        let response = self.post(PhemexEndpoint::Withdraw, body, AccountType::Spot).await?;

        let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
        let withdraw_id = data.get("id")
            .and_then(|v| v.as_str().map(String::from)
                .or_else(|| v.as_i64().map(|n| n.to_string())))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Get deposit and/or withdrawal history.
    ///
    /// Endpoint (deposits): GET /exchange/wallets/depositList
    /// Endpoint (withdrawals): GET /exchange/wallets/withdrawList
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut records: Vec<FundsRecord> = Vec::new();

        let build_params = |f: &FundsHistoryFilter| {
            let mut p = HashMap::new();
            if let Some(a) = &f.asset {
                p.insert("currency".to_string(), a.clone());
            }
            if let Some(s) = f.start_time {
                p.insert("start".to_string(), s.to_string());
            }
            if let Some(e) = f.end_time {
                p.insert("end".to_string(), e.to_string());
            }
            if let Some(l) = f.limit {
                p.insert("limit".to_string(), l.to_string());
            }
            p
        };

        if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
            let params = build_params(&filter);
            let response = self.get(PhemexEndpoint::DepositList, params, AccountType::Spot).await?;
            if let Some(data) = response.get("data").and_then(|d| d.get("rows")).and_then(|v| v.as_array()) {
                for item in data {
                    let id = item.get("id")
                        .and_then(|v| v.as_str().map(String::from)
                            .or_else(|| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    let asset = item.get("currency").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let amount_ev = item.get("amountEv").and_then(|v| v.as_i64()).unwrap_or(0);
                    let amount = amount_ev as f64 / 10f64.powi(self.default_value_scale as i32);
                    let tx_hash = item.get("txHash").and_then(|v| v.as_str()).map(String::from);
                    let network = item.get("chainName").and_then(|v| v.as_str()).map(String::from);
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                    let timestamp = item.get("submitTime").and_then(|v| v.as_i64()).unwrap_or(0);

                    records.push(FundsRecord::Deposit {
                        id, asset, amount, tx_hash, network, status, timestamp,
                    });
                }
            }
        }

        if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
            let params = build_params(&filter);
            let response = self.get(PhemexEndpoint::WithdrawList, params, AccountType::Spot).await?;
            if let Some(data) = response.get("data").and_then(|d| d.get("rows")).and_then(|v| v.as_array()) {
                for item in data {
                    let id = item.get("id")
                        .and_then(|v| v.as_str().map(String::from)
                            .or_else(|| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    let asset = item.get("currency").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let amount_ev = item.get("amountEv").and_then(|v| v.as_i64()).unwrap_or(0);
                    let amount = amount_ev as f64 / 10f64.powi(self.default_value_scale as i32);
                    let fee_ev = item.get("feeEv").and_then(|v| v.as_i64());
                    let fee = fee_ev.map(|f| f as f64 / 10f64.powi(self.default_value_scale as i32));
                    let address = item.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let tag = item.get("addressTag").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from);
                    let tx_hash = item.get("txHash").and_then(|v| v.as_str()).map(String::from);
                    let network = item.get("chainName").and_then(|v| v.as_str()).map(String::from);
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                    let timestamp = item.get("submitTime").and_then(|v| v.as_i64()).unwrap_or(0);

                    records.push(FundsRecord::Withdrawal {
                        id, asset, amount, fee, address, tag, tx_hash, network, status, timestamp,
                    });
                }
            }
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SubAccounts for PhemexConnector {
    /// Perform a sub-account operation.
    ///
    /// - Create: POST /phemex-user/users/children
    /// - List: GET /phemex-user/users/children
    /// - Transfer: POST /assets/universal-transfer
    /// - GetBalance: UnsupportedOperation
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::Create { label } => {
                let body = serde_json::json!({
                    "childUsername": label,
                });

                let response = self.post(PhemexEndpoint::SubAccountCreate, body, AccountType::Spot).await?;

                let data = response.get("data").cloned().unwrap_or(serde_json::json!({}));
                let id = data.get("userId")
                    .and_then(|v| v.as_str().map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string())));

                Ok(SubAccountResult {
                    id,
                    name: Some(label),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                let response = self.get(PhemexEndpoint::SubAccountList, HashMap::new(), AccountType::Spot).await?;

                let items = response.get("data")
                    .and_then(|d| d.as_array())
                    .cloned()
                    .unwrap_or_default();

                let accounts = items.iter().map(|item| {
                    let id = item.get("userId")
                        .and_then(|v| v.as_str().map(String::from)
                            .or_else(|| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    let name = item.get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let status = item.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Normal")
                        .to_string();
                    SubAccount { id, name, status }
                }).collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                let amount_ev = scale_value(amount, self.default_value_scale);

                // For master→sub: fromUserId = master (empty string), toUserId = sub
                // For sub→master: fromUserId = sub, toUserId = master (empty string)
                // Phemex requires both IDs to be provided; use empty string for "self"
                let (from_user_id, to_user_id) = if to_sub {
                    ("".to_string(), sub_account_id.clone())
                } else {
                    (sub_account_id.clone(), "".to_string())
                };

                let body = serde_json::json!({
                    "fromUserId": from_user_id,
                    "toUserId": to_user_id,
                    "currency": asset,
                    "amountEv": amount_ev,
                });

                let response = self.post(PhemexEndpoint::SubAccountTransfer, body, AccountType::Spot).await?;

                let transaction_id = response.get("data")
                    .and_then(|d| d.get("linkKey"))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id,
                })
            }

            SubAccountOperation::GetBalance { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "GetBalance for sub-accounts is not available on Phemex".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS — Trade history & derivatives additions
// ═══════════════════════════════════════════════════════════════════════════════

impl PhemexConnector {
    /// Trade history (fills) — `GET /exchange/order/v2/tradingList` (signed)
    ///
    /// Returns the authenticated user's trade execution history.
    /// Optional params: `symbol`, `start`, `end`, `offset`, `limit`, `withCount`.
    pub async fn get_trade_history(
        &self,
        symbol: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }
        if let Some(st) = start_time {
            params.insert("start".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(200).to_string());
        }
        let response = self.get(PhemexEndpoint::TradeHistory, params, account_type).await?;
        Ok(response)
    }

    /// Open interest — `GET /api-data/public/data/open-interest` (public)
    ///
    /// Required param: `symbol` (contract symbol, e.g. `BTCUSD`).
    /// Optional params: `start`, `end`, `limit`.
    pub async fn get_open_interest(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(st) = start_time {
            params.insert("start".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(200).to_string());
        }
        let response = self.get(PhemexEndpoint::OpenInterest, params, AccountType::FuturesCross).await?;
        Ok(response)
    }

    /// Funding rate history — `GET /api-data/public/data/funding-rate-history` (public)
    ///
    /// Required param: `symbol`. Optional: `start`, `end`, `limit`.
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(st) = start_time {
            params.insert("start".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(200).to_string());
        }
        let response = self.get(PhemexEndpoint::FundingRateHistory, params, AccountType::FuturesCross).await?;
        Ok(response)
    }
}

fn interval_to_secs(interval: &str) -> u64 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "4h" => 14400,
        "12h" => 43200,
        "1d" => 86400,
        "1w" => 604800,
        _ => 3600,
    }
}
