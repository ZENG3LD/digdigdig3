//! # Upbit Connector
//!
//! Реализация всех core трейтов для Upbit.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//!
//! ## Note
//! Upbit only supports Spot trading (no Futures).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType,Balance, AccountInfo,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::GroupRateLimiter;

use super::endpoints::{UpbitUrls, UpbitEndpoint, format_symbol, map_kline_interval};
use super::auth::{UpbitAuth, json_to_query_string};
use super::parser::UpbitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Upbit коннектор
pub struct UpbitConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<UpbitAuth>,
    /// URL'ы (регион)
    urls: UpbitUrls,
    /// Rate limiter with groups: market (10/s), account (30/s), order (8/s)
    rate_limiter: Arc<Mutex<GroupRateLimiter>>,
}

impl UpbitConnector {
    /// Создать новый коннектор
    /// region: "kr"/"korea" (Korea, KRW markets), "sg" (Singapore), "id" (Indonesia), "th" (Thailand)
    pub async fn new(credentials: Option<Credentials>, region: &str) -> ExchangeResult<Self> {
        let urls = match region {
            "kr" | "korea" => UpbitUrls::KOREA,
            "sg" | "singapore" => UpbitUrls::SINGAPORE,
            "id" => UpbitUrls::INDONESIA,
            "th" => UpbitUrls::THAILAND,
            _ => UpbitUrls::KOREA, // Default to Korea (KRW markets)
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(UpbitAuth::new)
            .transpose()?;

        // Initialize group rate limiter per Upbit API limits
        let mut group_limiter = GroupRateLimiter::new();
        group_limiter.add_group("market", 10, Duration::from_secs(1));
        group_limiter.add_group("account", 30, Duration::from_secs(1));
        group_limiter.add_group("order", 8, Duration::from_secs(1));
        let rate_limiter = Arc::new(Mutex::new(group_limiter));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Создать коннектор только для публичных методов (Korea region, KRW markets)
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None, "kr").await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Upbit response headers
    ///
    /// Upbit reports: Remaining-Req = "group=market; min=99; sec=9"
    /// Parse the group name and `sec=XX` remaining-per-second value,
    /// then compute used = group_max - remaining and call update_from_server.
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let header_val = match headers
            .get("Remaining-Req")
            .and_then(|v| v.to_str().ok())
        {
            Some(s) => s.to_string(),
            None => return,
        };

        // Parse group name from "group=market; min=99; sec=9"
        let group_name = header_val
            .split(';')
            .find(|part| part.trim().starts_with("group="))
            .and_then(|part| part.trim().strip_prefix("group="))
            .map(|s| s.trim().to_string());

        // Parse sec=XX (remaining per second)
        let remaining_sec = header_val
            .split(';')
            .find(|part| part.trim().starts_with("sec="))
            .and_then(|part| part.trim().strip_prefix("sec="))
            .and_then(|v| v.trim().parse::<u32>().ok());

        if let (Some(group), Some(remaining)) = (group_name, remaining_sec) {
            // Determine group max from known limits
            let group_max = match group.as_str() {
                "market" => 10u32,
                "account" => 30u32,
                "order" => 8u32,
                _ => return,
            };
            let used = group_max.saturating_sub(remaining);
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(&group, used);
            }
        }
    }

    /// Wait for rate limit if needed, routing to the appropriate group
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
        endpoint: UpbitEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Route to appropriate rate limit group
        let group = if endpoint.requires_auth() { "account" } else { "market" };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(account_type);
        let mut path = endpoint.path().to_string();

        // For CandlesMinutes, add unit to path
        if endpoint == UpbitEndpoint::CandlesMinutes {
            if let Some(unit) = params.get("unit") {
                path = format!("{}/{}", path, unit);
            }
        }

        // Build query string
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<_> = params.iter()
                .filter(|(k, _)| *k != "unit") // Skip unit param for CandlesMinutes
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish()
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &path, Some(&query_string))?
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
        endpoint: UpbitEndpoint,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("order", 1).await;

        let base_url = self.urls.rest;
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        // Convert JSON body to query string for signing
        let body_str = body.to_string();
        let query_string = json_to_query_string(&body_str)?;
        let headers = auth.sign_request("POST", path, Some(&query_string))?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: UpbitEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait("order", 1).await;

        let base_url = self.urls.rest;
        let path = endpoint.path();

        // Build query string
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<_> = params.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish()
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("DELETE", path, Some(&query_string))?;

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Upbit-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить список всех торговых пар
    pub async fn get_trading_pairs(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;

        if let Some(arr) = response.as_array() {
            Ok(arr.iter()
                .filter_map(|v| v.get("market").and_then(|m| m.as_str()).map(String::from))
                .collect())
        } else {
            Ok(vec![])
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for UpbitConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Upbit
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
        false
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }
}

#[async_trait]
impl MarketData for UpbitConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);

        let response = self.get(UpbitEndpoint::Tickers, params, account_type).await?;
        UpbitParser::parse_price(&response)
    }

    async fn get_orderbook(&self, symbol: Symbol, _depth: Option<u16>, account_type: AccountType) -> ExchangeResult<OrderBook> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);

        let response = self.get(UpbitEndpoint::Orderbook, params, account_type).await?;
        UpbitParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let (endpoint, unit) = map_kline_interval(interval);

        let mut params = HashMap::new();
        params.insert("market".to_string(), upbit_symbol);
        if let Some(u) = unit {
            params.insert("unit".to_string(), u.to_string());
        }
        if let Some(l) = limit {
            params.insert("count".to_string(), l.min(200).to_string());
        }
        if let Some(et) = end_time {
            if let Some(dt) = chrono::DateTime::from_timestamp_millis(et) {
                params.insert("to".to_string(), dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
            }
        }

        let response = self.get(endpoint, params, account_type).await?;
        UpbitParser::parse_klines(&response)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let upbit_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("markets".to_string(), upbit_symbol);

        let response = self.get(UpbitEndpoint::Tickers, params, account_type).await?;
        UpbitParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Upbit doesn't have a dedicated ping endpoint, so we'll just call the server time endpoint
        self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /v1/market/all returns all markets
        let response = self.get(UpbitEndpoint::TradingPairs, HashMap::new(), AccountType::Spot).await?;
        UpbitParser::parse_exchange_info(&response)
    }
}

#[async_trait]
impl Trading for UpbitConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let upbit_symbol = if let Some(raw) = symbol.raw() {
                            raw.to_string()
                        } else {
                            format_symbol(&symbol.base, &symbol.quote, account_type)
                        };
                
                        // Upbit order types: "price" (market buy with total spend), "market" (market sell)
                        let (ord_type, side_str) = match side {
                            OrderSide::Buy => ("price", "bid"),
                            OrderSide::Sell => ("market", "ask"),
                        };
                
                        let mut body = json!({
                            "market": upbit_symbol,
                            "side": side_str,
                            "ord_type": ord_type,
                        });
                
                        // Market buy: quantity is total amount to spend
                        // Market sell: quantity is volume to sell
                        match side {
                            OrderSide::Buy => {
                                body["price"] = json!(quantity.to_string());
                            },
                            OrderSide::Sell => {
                                body["volume"] = json!(quantity.to_string());
                            },
                        }
                
                        let response = self.post(UpbitEndpoint::CreateOrder, body, account_type).await?;
                        UpbitParser::parse_order(&response, &upbit_symbol).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let upbit_symbol = if let Some(raw) = symbol.raw() {
                            raw.to_string()
                        } else {
                            format_symbol(&symbol.base, &symbol.quote, account_type)
                        };
                
                        let side_str = match side {
                            OrderSide::Buy => "bid",
                            OrderSide::Sell => "ask",
                        };
                
                        let body = json!({
                            "market": upbit_symbol,
                            "side": side_str,
                            "ord_type": "limit",
                            "volume": quantity.to_string(),
                            "price": price.to_string(),
                        });
                
                        let response = self.post(UpbitEndpoint::CreateOrder, body, account_type).await?;
                        UpbitParser::parse_order(&response, &upbit_symbol).map(PlaceOrderResponse::Simple)
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
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?;
                let account_type = req.account_type;

                let upbit_symbol = if let Some(raw) = symbol.raw() {
                    raw.to_string()
                } else {
                    format_symbol(&symbol.base, &symbol.quote, account_type)
                };
                let mut params = HashMap::new();
                params.insert("uuid".to_string(), order_id.to_string());

                let response = self.delete(UpbitEndpoint::CancelOrder, params, account_type).await?;
                UpbitParser::parse_order(&response, &upbit_symbol)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(&self, symbol: &str, order_id: &str, account_type: AccountType) -> ExchangeResult<Order> {
        let parts: Vec<&str> = symbol.split('/').collect();
        let sym = if parts.len() == 2 {
            crate::core::Symbol::new(parts[0], parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };
        let upbit_symbol = if let Some(raw) = sym.raw() {
            raw.to_string()
        } else {
            format_symbol(&sym.base, &sym.quote, account_type)
        };
        let mut params = HashMap::new();
        params.insert("uuid".to_string(), order_id.to_string());

        let response = self.get(UpbitEndpoint::GetOrder, params, account_type).await?;
        UpbitParser::parse_order(&response, &upbit_symbol)
    }

    async fn get_open_orders(&self, symbol: Option<&str>, account_type: AccountType) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("state".to_string(), "wait".to_string());

        if let Some(s) = symbol {
            let parts: Vec<&str> = s.split('/').collect();
            let sym = if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            };
            let upbit_symbol = if let Some(raw) = sym.raw() {
                raw.to_string()
            } else {
                format_symbol(&sym.base, &sym.quote, account_type)
            };
            params.insert("market".to_string(), upbit_symbol);
        }

        let response = self.get(UpbitEndpoint::ListOrders, params, account_type).await?;
        UpbitParser::parse_orders(&response)
    }
}

#[async_trait]
impl Account for UpbitConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;
        let response = self.get(UpbitEndpoint::Balances, HashMap::new(), account_type).await?;
        let balances = UpbitParser::parse_balances(&response)?;

        // Filter by asset if provided
        if let Some(asset_name) = asset {
            Ok(balances.into_iter()
                .filter(|b| b.asset.eq_ignore_ascii_case(&asset_name))
                .collect())
        } else {
            Ok(balances)
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            balances,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.05, // Upbit default maker commission 0.05%
            taker_commission: 0.05, // Upbit default taker commission 0.05%
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}
