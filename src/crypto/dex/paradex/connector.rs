//! # Paradex Connector
//!
//! Реализация всех core трейтов для Paradex.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Note
//! Paradex работает только с perpetual futures (no spot trading).
//! Все методы используют JWT authentication.

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
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, OrderStatus, Balance, AccountInfo,
    Position, FundingRate, TimeInForce,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    UserTrade, UserTradeFilter,
};
use crate::core::{AmendRequest, CancelAllResponse, OrderResult};
use crate::core::types::AlgoOrderResponse;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
};
use crate::core::utils::GroupRateLimiter;
use crate::core::utils::precision::PrecisionCache;
use crate::core::types::{ConnectorStats, SymbolInfo};

use super::endpoints::{ParadexUrls, ParadexEndpoint, format_symbol, map_kline_resolution};
use super::auth::ParadexAuth;
use super::parser::ParadexParser;

#[cfg(feature = "onchain-starknet")]
use crate::core::chain::StarkNetProvider;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Paradex коннектор
pub struct ParadexConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (JWT-based)
    auth: Arc<ParadexAuth>,
    /// URL'ы (mainnet/testnet)
    urls: ParadexUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (grouped: public=1500/60s, orders=17250/60s, private_gets=600/60s)
    rate_limiter: Arc<Mutex<GroupRateLimiter>>,
    /// Per-symbol precision cache (populated from get_exchange_info)
    precision: PrecisionCache,
    /// Optional StarkNet chain provider for on-chain operations (invoke, call, nonce).
    ///
    /// When set, connectors can use `starknet_provider` to read nonces or call
    /// contracts directly on StarkNet L2 without going through the Paradex REST API.
    /// This is an advanced use-case; the Paradex REST API is sufficient for all
    /// trading operations without this provider.
    #[cfg(feature = "onchain-starknet")]
    starknet_provider: Option<Arc<StarkNetProvider>>,
}

impl ParadexConnector {
    /// Создать новый коннектор
    ///
    /// ВАЖНО: Credentials должны содержать JWT token в api_key поле.
    /// Для полной реализации нужна интеграция со StarkNet signing.
    pub async fn new(credentials: Credentials, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            ParadexUrls::TESTNET
        } else {
            ParadexUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = Arc::new(ParadexAuth::new(&credentials)?);

        // Sync time with server
        let base_url = urls.rest_url();
        let url = format!("{}/system/time", base_url);
        if let Ok(response) = http.get(&url, &HashMap::new()).await {
            if let Some(server_time) = response.get("server_time").and_then(|t| t.as_i64()) {
                auth.sync_time(server_time).await;
            }
        }

        // Paradex rate limits grouped by endpoint category
        let mut rl = GroupRateLimiter::new();
        rl.add_group("public", 1500, Duration::from_secs(60));
        rl.add_group("orders", 17250, Duration::from_secs(60));
        rl.add_group("private_gets", 600, Duration::from_secs(60));
        let rate_limiter = Arc::new(Mutex::new(rl));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            precision: PrecisionCache::new(),
            #[cfg(feature = "onchain-starknet")]
            starknet_provider: None,
        })
    }

    /// Создать коннектор только для публичных методов (без auth)
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        let credentials = Credentials::new("", ""); // Empty credentials
        Self::new(credentials, testnet).await
    }

    /// Attach a [`StarkNetProvider`] for direct on-chain operations.
    ///
    /// This is an optional extension. The Paradex REST API is fully functional
    /// without a StarkNet provider. Use this when you need to:
    /// - Read nonces directly from StarkNet (bypassing REST API latency)
    /// - Call StarkNet contracts (e.g. token balances, position queries)
    /// - Broadcast StarkNet transactions without going through Paradex
    ///
    /// The provider is shared via `Arc` so it can be reused across connectors.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use digdigdig3::core::chain::StarkNetProvider;
    ///
    /// let connector = ParadexConnector::new(credentials, false)
    ///     .await?
    ///     .with_starknet_provider(Arc::new(StarkNetProvider::mainnet()));
    /// ```
    #[cfg(feature = "onchain-starknet")]
    pub fn with_starknet_provider(mut self, provider: Arc<StarkNetProvider>) -> Self {
        self.starknet_provider = Some(provider);
        self
    }

    /// Get the attached [`StarkNetProvider`], if any.
    ///
    /// Returns `None` if no provider was attached via [`with_starknet_provider`].
    ///
    /// [`with_starknet_provider`]: ParadexConnector::with_starknet_provider
    #[cfg(feature = "onchain-starknet")]
    pub fn starknet_provider(&self) -> Option<&Arc<StarkNetProvider>> {
        self.starknet_provider.as_ref()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    ///
    /// Groups: "public" (GET public endpoints), "orders" (POST/DELETE orders), "private_gets" (GET private endpoints)
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

    /// Update rate limiter from Paradex response headers.
    ///
    /// Paradex reports: `x-ratelimit-remaining`, `x-ratelimit-limit`, `x-ratelimit-window`.
    /// The `group` parameter identifies which limiter to update (the caller knows this from context).
    fn update_rate_from_headers(&self, headers: &HeaderMap, group: &str) {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        if let (Some(remaining), Some(limit)) = (remaining, limit) {
            let used = limit.saturating_sub(remaining);
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(group, used);
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: ParadexEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        let group = if endpoint.requires_auth() { "private_gets" } else { "public" };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters (e.g., {market}, {order_id})
        for (key, value) in &params {
            if path.contains(&format!("{{{}}}", key)) {
                path = path.replace(&format!("{{{}}}", key), value);
            }
        }

        // Build query string (exclude path parameters)
        let query_params: HashMap<_, _> = params.iter()
            .filter(|(k, _)| !path.contains(&format!("{{{}}}", k)))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let query = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);
        let full_path = format!("{}{}", path, query);

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            self.auth.sign_request("GET", &full_path, "").await?
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, group);
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: ParadexEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers (Paradex POST endpoints требуют JWT)
        let body_str = body.to_string();
        let headers = self.auth.sign_request("POST", path, &body_str).await?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers, "orders");
        self.check_response(&response)?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: ParadexEndpoint,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let headers = self.auth.sign_request("DELETE", &path, "").await?;

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, "orders");
        self.check_response(&response)?;
        Ok(response)
    }

    /// PUT запрос (для modify order)
    async fn _put(
        &self,
        endpoint: ParadexEndpoint,
        path_params: &[(&str, &str)],
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let body_str = body.to_string();
        let headers = self.auth.sign_request("PUT", &path, &body_str).await?;

        let response = self.http.put(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Проверить response на ошибки
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        // Paradex error format: {"error": "ERROR_CODE", "message": "description"}
        if let Some(error) = response.get("error") {
            let code = error.as_str().unwrap_or("UNKNOWN");
            let message = response.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            return Err(ExchangeError::Api {
                code: -1,
                message: format!("{}: {}", code, message),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Paradex-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все символы (markets)
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(ParadexEndpoint::Markets, HashMap::new()).await?;
        ParadexParser::parse_symbols(&response)
    }

    /// Получить все маркеты с summary data
    pub async fn get_markets_summary(&self, market: Option<String>) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        if let Some(m) = market {
            params.insert("market".to_string(), m);
        } else {
            params.insert("market".to_string(), "ALL".to_string());
        }

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;

        // Parse all tickers from results array
        let results = ParadexParser::extract_results(&response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        Ok(arr.iter()
            .filter_map(|item| {
                // Create a wrapper to use parse_ticker
                let wrapper = json!({"results": [item]});
                ParadexParser::parse_ticker(&wrapper).ok()
            })
            .collect())
    }

    /// Получить account summary
    pub async fn get_account_summary(&self) -> ExchangeResult<Value> {
        self.get(ParadexEndpoint::Account, HashMap::new()).await
    }

    /// Отменить все ордера
    pub async fn cancel_all_orders(&self, symbol: Option<Symbol>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("market".to_string(), format_symbol(&s.base, &s.quote, AccountType::FuturesCross));
        }

        self.delete(ParadexEndpoint::CancelAllOrders, &[]).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORE TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for ParadexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Paradex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max, rate_groups) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            let (u, m) = limiter.primary_stats();
            let groups = limiter.all_stats()
                .into_iter()
                .map(|(name, cur, max)| (name.to_string(), cur, max))
                .collect();
            (u, m, groups)
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
        vec![AccountType::FuturesCross] // Paradex только perpetual futures
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex // Paradex is a DEX
    }
}

#[async_trait]
impl MarketData for ParadexConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str);

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;
        Ok(ParadexParser::parse_price(&response)?)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str.clone());
        if let Some(d) = depth {
            params.insert("depth".to_string(), d.to_string());
        }

        let response = self.get(ParadexEndpoint::Orderbook, params).await?;
        ParadexParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str);
        params.insert("resolution".to_string(), map_kline_resolution(interval).to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(ParadexEndpoint::Klines, params).await?;
        ParadexParser::parse_klines(&response)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol_str);

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;
        ParadexParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(ParadexEndpoint::SystemState, HashMap::new()).await?;

        // Check if system is operational
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "operational" {
                return Ok(());
            }
        }

        Err(ExchangeError::Network("System not operational".to_string()))
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(ParadexEndpoint::Markets, HashMap::new()).await?;

        // Paradex markets endpoint returns {"results": [...]} where each entry has
        // "symbol", "base_currency", "quote_currency", "price_tick_size",
        // "order_size_increment", "min_notional", "max_order_size", "status".
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array in markets response".to_string()))?;

        let infos: Vec<SymbolInfo> = results.iter().filter_map(|item| {
            let sym = item.get("symbol").and_then(|v| v.as_str())?.to_string();

            // base_currency / quote_currency are returned directly by the API
            let base = item.get("base_currency")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    // Fallback: split "BTC-USD-PERP" on first '-'
                    sym.splitn(2, '-').next().unwrap_or(&sym)
                })
                .to_string();

            let quote = item.get("quote_currency")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    // Fallback: take middle segment of "BTC-USD-PERP"
                    let parts: Vec<&str> = sym.splitn(3, '-').collect();
                    if parts.len() > 1 { parts[1] } else { "USD" }
                })
                .to_string();

            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("TRADING")
                .to_string();

            // Paradex provides "price_tick_size" as a decimal string e.g. "0.1"
            let tick_size = item.get("price_tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // "order_size_increment" maps to step_size
            let step_size = item.get("order_size_increment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = item.get("min_notional")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let max_quantity = item.get("max_order_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            Some(SymbolInfo {
                symbol: sym,
                base_asset: base,
                quote_asset: quote,
                status,
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity,
                tick_size,
                step_size,
                min_notional,
            })
        }).collect();

        self.precision.load_from_symbols(&infos);
        Ok(infos)
    }
}

#[async_trait]
impl Trading for ParadexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        match req.order_type {
            OrderType::Market => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "MARKET",
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "IOC",
                    // NOTE: full production use requires StarkNet signature + signature_timestamp
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "GTC",
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::PostOnly { price } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "POST_ONLY",
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                let mut body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "MARKET",
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "IOC",
                });
                // If price specified, treat as limit IOC
                if let Some(p) = price {
                    body["type"] = json!("LIMIT");
                    body["price"] = json!(self.precision.price(&symbol_str, p));
                }

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Fok { price } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "FOK",
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "STOP_MARKET",
                    "trigger_price": self.precision.price(&symbol_str, stop_price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "IOC",
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "STOP_LIMIT",
                    "trigger_price": self.precision.price(&symbol_str, stop_price),
                    "price": self.precision.price(&symbol_str, limit_price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "GTC",
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::ReduceOnly { price } => {
                let (order_type_str, price_val) = match price {
                    Some(p) => ("LIMIT", p),
                    None => ("MARKET", 0.0),
                };
                let mut body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": order_type_str,
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": if price.is_some() { "GTC" } else { "IOC" },
                    "reduce_only": true,
                });
                if price.is_some() {
                    body["price"] = json!(self.precision.price(&symbol_str, price_val));
                }

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Gtd { price, expire_time } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, price),
                    "size": self.precision.qty(&symbol_str, quantity),
                    "instruction": "GTC",
                    "expiry": expire_time,
                });

                let response = self.post(ParadexEndpoint::CreateOrder, body).await?;
                ParadexParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            // TWAP algo order via POST /v1/algo/orders.
            // Paradex TWAP: sub-orders every 30 seconds, market sub-order type only.
            // Duration must be 30–86400 seconds.
            OrderType::Twap { duration_seconds, .. } => {
                let body = json!({
                    "market": symbol_str,
                    "side": side_str,
                    "size": self.precision.qty(&symbol_str, quantity),
                    "algo_type": "TWAP",
                    // Duration in seconds; clamp to [30, 86400] per Paradex spec.
                    "duration": duration_seconds.clamp(30, 86400),
                });

                let response = self.post(ParadexEndpoint::CreateAlgoOrder, body).await?;

                // Paradex returns { "id": "<algo_id>", "status": "...", ... }
                let algo_id = response
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = response
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("RUNNING")
                    .to_string();

                Ok(PlaceOrderResponse::Algo(AlgoOrderResponse {
                    algo_id,
                    status,
                    executed_count: None,
                    total_count: None,
                }))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on Paradex", req.order_type)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();

        if let Some(symbol) = &filter.symbol {
            params.insert("market".to_string(), format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross));
        }
        if let Some(limit) = filter.limit {
            params.insert("page_size".to_string(), limit.to_string());
        }
        if let Some(start) = filter.start_time {
            params.insert("start_unix_timestamp".to_string(), (start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end_unix_timestamp".to_string(), (end / 1000).to_string());
        }

        let response = self.get(ParadexEndpoint::OrdersHistory, params).await?;
        ParadexParser::parse_orders(&response)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                self.delete(ParadexEndpoint::CancelOrder, &[("order_id", order_id)]).await?;

                // Return a minimal cancelled order stub.
                Ok(Order {
                    id: order_id.to_string(),
                    client_order_id: None,
                    symbol: req.symbol
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit { price: 0.0 },
                    status: OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: TimeInForce::Gtc,
                })
            }

            CancelScope::All { ref symbol } => {
                self.cancel_all_orders(symbol.clone()).await?;

                // Return a synthetic "cancelled" placeholder order.
                Ok(Order {
                    id: "cancel-all".to_string(),
                    client_order_id: None,
                    symbol: String::new(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: TimeInForce::Gtc,
                })
            }

            CancelScope::BySymbol { ref symbol } => {
                self.cancel_all_orders(Some(symbol.clone())).await?;

                Ok(Order {
                    id: "cancel-by-symbol".to_string(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: TimeInForce::Gtc,
                })
            }

            CancelScope::Batch { .. } => Err(ExchangeError::UnsupportedOperation(
                "Batch cancel not supported on Paradex; use CancelAll/BySymbol instead".to_string()
            )),

            CancelScope::ByLabel(_) | CancelScope::ByCurrencyKind { .. } | CancelScope::ScheduledAt(_) => {
                Err(ExchangeError::UnsupportedOperation(
                    "ByLabel/ByCurrencyKind/ScheduledAt cancel scopes not supported on Paradex".to_string()
                ))
            }
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

        let mut params = HashMap::new();
        params.insert("order_id".to_string(), order_id.to_string());

        let response = self.get(ParadexEndpoint::GetOrder, params).await?;
        ParadexParser::parse_order(&response)
    
    }

    async fn get_open_orders(&self, symbol: Option<&str>, account_type: AccountType) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            let parts: Vec<&str> = s.split('/').collect();
            let symbol_str = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                s.to_string()
            };
            params.insert("market".to_string(), symbol_str);
        }

        let response = self.get(ParadexEndpoint::OpenOrders, params).await?;
        ParadexParser::parse_orders(&response)
    }

    /// Get user fill history via `GET /v1/fills`.
    ///
    /// Paradex fills are individual trade executions. Supports filtering by
    /// market, time range, and result page size (max 100 per page).
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = HashMap::new();

        // Market filter (Paradex symbol format: BTC-USD-PERP)
        if let Some(sym) = &filter.symbol {
            // Accept both raw market names ("BTC-USD-PERP") and slash-separated
            // ("BTC/USD"). If the caller passes slash-separated we convert.
            let market = if sym.contains('/') {
                let parts: Vec<&str> = sym.splitn(2, '/').collect();
                format!("{}-{}-PERP", parts[0].to_uppercase(), parts[1].to_uppercase())
            } else {
                sym.clone()
            };
            params.insert("market".to_string(), market);
        }

        // Order ID filter
        if let Some(order_id) = &filter.order_id {
            params.insert("order_id".to_string(), order_id.clone());
        }

        // Time range — Paradex uses Unix seconds for start/end
        if let Some(start_ms) = filter.start_time {
            params.insert("start_unix_timestamp".to_string(), (start_ms / 1000).to_string());
        }
        if let Some(end_ms) = filter.end_time {
            params.insert("end_unix_timestamp".to_string(), (end_ms / 1000).to_string());
        }

        // Page size (max 100)
        if let Some(limit) = filter.limit {
            params.insert("page_size".to_string(), limit.min(100).to_string());
        }

        let response = self.get(ParadexEndpoint::Fills, params).await?;
        ParadexParser::parse_fills(&response)
    }
}

#[async_trait]
impl Account for ParadexConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(ParadexEndpoint::Balances, HashMap::new()).await?;
        ParadexParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(ParadexEndpoint::Account, HashMap::new()).await?;
        ParadexParser::parse_account_info(&response)
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Paradex exposes fee_config in the /markets endpoint (fee_config.api_fees).
        let mut params = HashMap::new();
        if let Some(sym) = symbol {
            params.insert("market".to_string(), sym.to_string());
        }

        let response = self.get(ParadexEndpoint::Markets, params).await?;

        // Parse first market's fee_config.api_fees
        let results = response.get("results")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .cloned();

        let (maker_rate, taker_rate) = if let Some(market) = results {
            let fee_config = market.get("fee_config");
            let api_fees = fee_config.and_then(|fc| fc.get("api_fees"));

            let maker = api_fees
                .and_then(|af| af.get("maker"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let taker = api_fees
                .and_then(|af| af.get("taker"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0003); // Paradex default taker ~0.03%

            (maker, taker)
        } else {
            (0.0, 0.0003)
        };

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

#[async_trait]
impl Positions for ParadexConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let mut params = HashMap::new();
        if let Some(symbol) = &query.symbol {
            params.insert("market".to_string(), format_symbol(&symbol.base, &symbol.quote, query.account_type));
        }

        let response = self.get(ParadexEndpoint::Positions, params).await?;
        ParadexParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Paradex provides funding rate via markets summary (next_funding_rate field).
        let mut params = HashMap::new();
        params.insert("market".to_string(), symbol.to_string());

        let response = self.get(ParadexEndpoint::MarketsSummary, params).await?;
        let mut rate = ParadexParser::parse_funding_rate(&response)?;
        rate.symbol = symbol.to_string();
        Ok(rate)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { symbol, account_type } => {
                // Close position by placing a reduce-only market order opposite to current side.
                // Paradex does not have a dedicated "close position" endpoint —
                // the standard approach is a reduce-only market order.
                // We signal this via the order instruction "REDUCE_ONLY".
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                let body = json!({
                    "market": symbol_str,
                    "type": "MARKET",
                    "instruction": "REDUCE_ONLY",
                    // NOTE: side and size must be determined from current position.
                    // Without knowing position side we cannot fill them here automatically.
                    // Callers should use place_order(ReduceOnly) directly for full control.
                });
                let _ = self.post(ParadexEndpoint::CreateOrder, body).await?;
                Ok(())
            }

            PositionModification::SetTpSl { symbol, take_profit, stop_loss, account_type } => {
                // Paradex TP/SL is set via separate conditional orders (STOP_MARKET with reduce_only).
                // Place them if provided.
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);

                if let Some(tp) = take_profit {
                    let body = json!({
                        "market": symbol_str,
                        "type": "TAKE_PROFIT_MARKET",
                        "trigger_price": self.precision.price(&symbol_str, tp),
                        "instruction": "IOC",
                        "reduce_only": true,
                    });
                    let _ = self.post(ParadexEndpoint::CreateOrder, body).await?;
                }

                if let Some(sl) = stop_loss {
                    let body = json!({
                        "market": symbol_str,
                        "type": "STOP_MARKET",
                        "trigger_price": self.precision.price(&symbol_str, sl),
                        "instruction": "IOC",
                        "reduce_only": true,
                    });
                    let _ = self.post(ParadexEndpoint::CreateOrder, body).await?;
                }

                Ok(())
            }

            // Paradex manages leverage and margin mode automatically at the account level.
            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Paradex manages leverage automatically based on margin mode".to_string()
                ))
            }
            PositionModification::SetMarginMode { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Paradex uses cross-margin by default; isolated margin is per-market configuration".to_string()
                ))
            }
            PositionModification::AddMargin { .. } | PositionModification::RemoveMargin { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Paradex uses auto-margin management; manual margin add/remove not supported".to_string()
                ))
            }

            PositionModification::SwitchPositionMode { .. } | PositionModification::MovePositions { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SwitchPositionMode/MovePositions not supported on Paradex".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIONAL TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for ParadexConnector {
    /// Cancel all open orders, optionally filtered to a single market.
    ///
    /// Uses `DELETE /v1/orders` — a native Paradex endpoint.
    /// Returns aggregate counts; Paradex does not return per-order detail on this endpoint.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { .. } => {
                // DELETE /orders — cancel all open orders across all markets
                self.delete(ParadexEndpoint::CancelAllOrders, &[]).await?;
                Ok(CancelAllResponse {
                    cancelled_count: 0, // Paradex does not return count in this response
                    failed_count: 0,
                    details: Vec::new(),
                })
            }
            CancelScope::BySymbol { ref symbol } => {
                // DELETE /orders?market=BTC-USD-PERP — cancel all for one market
                // Paradex accepts an optional `market` query param on the cancel-all endpoint.
                // We need to pass it as a query string on the DELETE request.
                // The existing `delete()` helper builds the path only; for the market filter
                // we append it directly to the URL by using a custom request.
                self.rate_limit_wait("orders", 1).await;

                let base_url = self.urls.rest_url();
                let market_str = format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross);
                let path = format!("{}?market={}", ParadexEndpoint::CancelAllOrders.path(), market_str);
                let url = format!("{}{}", base_url, path);
                let headers = self.auth.sign_request("DELETE", &path, "").await?;

                let (response, resp_headers) = self.http
                    .delete_with_response_headers(&url, &std::collections::HashMap::new(), &headers)
                    .await?;
                self.update_rate_from_headers(&resp_headers, "orders");
                self.check_response(&response)?;

                Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: Vec::new(),
                })
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "CancelAll only accepts CancelScope::All or CancelScope::BySymbol".to_string(),
            )),
        }
    }
}

#[async_trait]
impl AmendOrder for ParadexConnector {
    /// Modify a live order's price and/or size in-place.
    ///
    /// Uses `PUT /v1/orders/{order_id}` — native Paradex modify endpoint.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let mut body = json!({});

        let amend_symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, req.account_type);
        if let Some(price) = req.fields.price {
            body["price"] = json!(self.precision.price(&amend_symbol_str, price));
        }
        if let Some(qty) = req.fields.quantity {
            body["size"] = json!(self.precision.qty(&amend_symbol_str, qty));
        }
        if let Some(trigger) = req.fields.trigger_price {
            body["trigger_price"] = json!(self.precision.price(&amend_symbol_str, trigger));
        }

        let response = self._put(
            ParadexEndpoint::ModifyOrder,
            &[("order_id", &req.order_id)],
            body,
        )
        .await?;

        ParadexParser::parse_order(&response)
    }
}

#[async_trait]
impl BatchOrders for ParadexConnector {
    /// Place multiple orders in a single `POST /v1/orders/batch` request.
    ///
    /// Paradex batch endpoint accepts up to 10 orders per call.
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        let order_jsons: Vec<serde_json::Value> = orders.iter().map(|req| {
            let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, req.account_type);
            let side_str = match req.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            };
            match &req.order_type {
                OrderType::Market => json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "MARKET",
                    "size": self.precision.qty(&symbol_str, req.quantity),
                    "instruction": "IOC",
                }),
                OrderType::Limit { price } => json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, *price),
                    "size": self.precision.qty(&symbol_str, req.quantity),
                    "instruction": "GTC",
                }),
                OrderType::PostOnly { price } => json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, *price),
                    "size": self.precision.qty(&symbol_str, req.quantity),
                    "instruction": "POST_ONLY",
                }),
                OrderType::Ioc { price } => {
                    let mut o = json!({
                        "market": symbol_str,
                        "side": side_str,
                        "type": "MARKET",
                        "size": self.precision.qty(&symbol_str, req.quantity),
                        "instruction": "IOC",
                    });
                    if let Some(p) = price {
                        o["type"] = json!("LIMIT");
                        o["price"] = json!(self.precision.price(&symbol_str, *p));
                    }
                    o
                }
                OrderType::Fok { price } => json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "LIMIT",
                    "price": self.precision.price(&symbol_str, *price),
                    "size": self.precision.qty(&symbol_str, req.quantity),
                    "instruction": "FOK",
                }),
                _ => json!({
                    "market": symbol_str,
                    "side": side_str,
                    "type": "MARKET",
                    "size": self.precision.qty(&symbol_str, req.quantity),
                    "instruction": "IOC",
                }),
            }
        }).collect();

        let body = json!({ "orders": order_jsons });
        let response = self.post(ParadexEndpoint::CreateOrderBatch, body).await?;

        // Paradex batch response: { "results": [ { "id": "...", "status": "OPEN|REJECTED", ... } ] }
        let results_val = response.get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' in batch place response".to_string()))?;

        let results: Vec<OrderResult> = results_val.iter().zip(orders.iter()).map(|(item, req)| {
            if let Some(err_msg) = item.get("error").and_then(|e| e.as_str()) {
                OrderResult {
                    order: None,
                    client_order_id: req.client_order_id.clone(),
                    success: false,
                    error: Some(err_msg.to_string()),
                    error_code: item.get("error_code").and_then(|c| c.as_i64()).map(|c| c as i32),
                }
            } else {
                match ParadexParser::parse_order(item) {
                    Ok(order) => OrderResult {
                        order: Some(order),
                        client_order_id: req.client_order_id.clone(),
                        success: true,
                        error: None,
                        error_code: None,
                    },
                    Err(e) => OrderResult {
                        order: None,
                        client_order_id: req.client_order_id.clone(),
                        success: false,
                        error: Some(e.to_string()),
                        error_code: None,
                    },
                }
            }
        }).collect();

        Ok(results)
    }

    /// Cancel multiple orders by ID in a single `DELETE /v1/orders/batch` request.
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        self.rate_limit_wait("orders", 1).await;

        let base_url = self.urls.rest_url();
        let path = ParadexEndpoint::CancelOrderBatch.path();
        let url = format!("{}{}", base_url, path);
        let body = json!({ "ids": order_ids });
        let body_str = body.to_string();
        let headers = self.auth.sign_request("DELETE", path, &body_str).await?;

        let response = self.http
            .delete_with_body(&url, &body, &headers)
            .await?;
        self.check_response(&response)?;

        // Paradex batch cancel returns: { "cancelled": ["id1", "id2"], "failed": [...] }
        let cancelled: Vec<String> = response.get("cancelled")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        let failed: Vec<String> = response.get("failed")
            .and_then(|f| f.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        let results: Vec<OrderResult> = order_ids.iter().map(|oid| {
            if cancelled.contains(oid) {
                OrderResult {
                    order: Some(Order {
                        id: oid.clone(),
                        client_order_id: None,
                        symbol: String::new(),
                        side: OrderSide::Buy,
                        order_type: OrderType::Limit { price: 0.0 },
                        status: OrderStatus::Canceled,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: Some(crate::core::timestamp_millis() as i64),
                        time_in_force: TimeInForce::Gtc,
                    }),
                    client_order_id: None,
                    success: true,
                    error: None,
                    error_code: None,
                }
            } else if failed.contains(oid) {
                OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(format!("Failed to cancel order {}", oid)),
                    error_code: None,
                }
            } else {
                // Not mentioned — treat as failed
                OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(format!("Order {} not in cancel response", oid)),
                    error_code: None,
                }
            }
        }).collect();

        Ok(results)
    }

    fn max_batch_place_size(&self) -> usize {
        10 // Paradex batch endpoint limit per call
    }

    fn max_batch_cancel_size(&self) -> usize {
        100 // Paradex batch cancel accepts up to 100 IDs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_public_connector() {
        let connector = ParadexConnector::public(true).await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        let connector = ParadexConnector::public(true);
        // This is async, so we can't test it directly here
        // But we can test the sync methods once we have instance
    }
}
