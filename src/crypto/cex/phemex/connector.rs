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
    ExchangeId, ExchangeType, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType,Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::GroupRateLimiter;

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

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(PhemexEndpoint::Products, HashMap::new(), AccountType::Spot).await?;
        PhemexParser::parse_exchange_info(&response)
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

        match req.order_type {
            OrderType::Market => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                        let side_str = match side {
                            OrderSide::Buy => "Buy",
                            OrderSide::Sell => "Sell",
                        };
                
                        let body = match account_type {
                            AccountType::Spot => {
                                // Spot market order
                                json!({
                                    "symbol": symbol_str,
                                    "side": side_str,
                                    "ordType": "Market",
                                    "qtyType": "ByBase",
                                    "baseQtyEv": scale_value(quantity, self.default_value_scale),
                                })
                            }
                            _ => {
                                // Contract market order
                                json!({
                                    "symbol": symbol_str,
                                    "side": side_str,
                                    "orderQty": quantity as i64,
                                    "ordType": "Market",
                                })
                            }
                        };
                
                        let endpoint = match account_type {
                            AccountType::Spot => PhemexEndpoint::SpotCreateOrder,
                            _ => PhemexEndpoint::ContractCreateOrder,
                        };
                
                        let response = self.post(endpoint, body, account_type).await?;
                        PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let symbol_str = format_symbol(&symbol.base, &symbol.quote, account_type);
                        let side_str = match side {
                            OrderSide::Buy => "Buy",
                            OrderSide::Sell => "Sell",
                        };
                
                        let body = match account_type {
                            AccountType::Spot => {
                                // Spot order
                                json!({
                                    "symbol": symbol_str,
                                    "side": side_str,
                                    "ordType": "Limit",
                                    "qtyType": "ByBase",
                                    "baseQtyEv": scale_value(quantity, self.default_value_scale),
                                    "priceEp": scale_price(price, self.default_price_scale),
                                    "timeInForce": "GoodTillCancel",
                                })
                            }
                            _ => {
                                // Contract order
                                json!({
                                    "symbol": symbol_str,
                                    "side": side_str,
                                    "orderQty": quantity as i64,
                                    "ordType": "Limit",
                                    "priceEp": scale_price(price, self.default_price_scale),
                                    "timeInForce": "GoodTillCancel",
                                })
                            }
                        };
                
                        let endpoint = match account_type {
                            AccountType::Spot => PhemexEndpoint::SpotCreateOrder,
                            _ => PhemexEndpoint::ContractCreateOrder,
                        };
                
                        let response = self.post(endpoint, body, account_type).await?;
                        PhemexParser::parse_order(&response, &symbol_str, self.default_price_scale).map(PlaceOrderResponse::Simple)
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
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
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
            maker_commission: 0.0,  // Default fee, should be fetched from API
            taker_commission: 0.0,  // Default fee, should be fetched from API
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
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
        // Parse symbol string into Symbol struct
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

                // For Phemex, leverage depends on margin mode
                // leverageEr: positive = isolated, zero/negative = cross
                // Convert leverage to Er format (ratioScale = 8)
                // For cross margin, use 0. For isolated, calculate based on leverage percentage
                let leverage_er = if account_type == AccountType::FuturesIsolated {
                ((leverage as f64 / 100.0) * 100_000_000.0) as i64
                } else {
                0i64 // Cross margin
                };

                let body = json!({
                "symbol": symbol_str,
                "leverageEr": leverage_er,
                });

                let _response = self.put(PhemexEndpoint::SetLeverage, HashMap::new(), body, account_type).await?;
                Ok(())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_public_connector() {
        let connector = PhemexConnector::public(false).await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        let mut group_limiter = GroupRateLimiter::new();
        group_limiter.add_group("OTHERS", 100, Duration::from_secs(60));
        group_limiter.add_group("CONTRACT", 500, Duration::from_secs(60));
        group_limiter.add_group("SPOTORDER", 500, Duration::from_secs(60));

        let connector = PhemexConnector {
            http: HttpClient::new(30_000).unwrap(),
            auth: None,
            urls: PhemexUrls::MAINNET,
            testnet: false,
            rate_limiter: Arc::new(Mutex::new(group_limiter)),
            default_price_scale: 4,
            default_value_scale: 8,
        };

        assert_eq!(connector.exchange_id(), ExchangeId::Phemex);
        assert!(!connector.is_testnet());
        assert_eq!(connector.exchange_type(), ExchangeType::Cex);
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
