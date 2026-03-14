//! # Gemini Connector
//!
//! Реализация всех core трейтов для Gemini.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## Extended методы
//! Дополнительные Gemini-специфичные методы как методы структуры.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CancelAllResponse, CancelAll, CustodialFunds,
    DepositAddress, WithdrawResponse, FundsRecord,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::types::{WithdrawRequest, FundsHistoryFilter, FundsRecordType};
use crate::core::utils::SimpleRateLimiter;
use crate::core::utils::PrecisionCache;

use super::endpoints::{GeminiUrls, GeminiEndpoint, format_symbol, normalize_symbol, map_kline_interval};
use super::auth::GeminiAuth;
use super::parser::GeminiParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Gemini коннектор
pub struct GeminiConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<GeminiAuth>,
    /// URL'ы (mainnet/testnet)
    urls: GeminiUrls,
    /// Testnet mode
    testnet: bool,
    /// Public rate limiter (120 req/min = 2 req/sec)
    public_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Private rate limiter (600 req/min = 10 req/sec)
    private_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl GeminiConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            GeminiUrls::TESTNET
        } else {
            GeminiUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(GeminiAuth::new)
            .transpose()?;

        // Initialize rate limiters: public 120 req/min, private 600 req/min
        let public_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(120, Duration::from_secs(60))
        ));
        let private_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(600, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            public_limiter,
            private_limiter,
            precision: PrecisionCache::new(),
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, is_private: bool) {
        let limiter = if is_private {
            &self.private_limiter
        } else {
            &self.public_limiter
        };

        loop {
            let wait_time = {
                let mut lim = limiter.lock().expect("Mutex poisoned");
                if lim.try_acquire() {
                    return;
                }
                lim.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: GeminiEndpoint,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(endpoint.requires_auth()).await;

        let base_url = self.urls.rest_url(AccountType::Spot);
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        let response = self.http.get(&url, &HashMap::new()).await?;
        GeminiParser::check_error(&response)?;
        Ok(response)
    }

    /// POST запрос (всегда требует auth)
    async fn post(
        &self,
        endpoint: GeminiEndpoint,
        params: HashMap<String, Value>,
        path_params: &[(&str, &str)],
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (POST is always private)
        self.rate_limit_wait(true).await;

        let base_url = self.urls.rest_url(AccountType::Spot);
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&path, params)?;

        // Gemini POST requests have empty body, everything in headers
        let response = self.http.post(&url, &json!({}), &headers).await?;
        GeminiParser::check_error(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for GeminiConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Gemini
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.public_limiter.lock() {
            (lim.current_count(), lim.max_requests())
        } else {
            (0, 0)
        };
        let rate_groups = {
            let pub_stats = self.public_limiter.lock()
                .map(|mut lim| (lim.current_count(), lim.max_requests()))
                .unwrap_or((0, 0));
            let priv_stats = self.private_limiter.lock()
                .map(|mut lim| (lim.current_count(), lim.max_requests()))
                .unwrap_or((0, 0));
            vec![
                ("public".to_string(), pub_stats.0, pub_stats.1),
                ("private".to_string(), priv_stats.0, priv_stats.1),
            ]
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
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for GeminiConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::Ticker,
            &[("symbol", &symbol_str)],
        ).await?;

        let ticker = GeminiParser::parse_ticker(&response, &symbol_str)?;
        Ok(ticker.last_price)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::TickerV2,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_ticker(&response, &symbol_str)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(
            GeminiEndpoint::OrderBook,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
        let time_frame = map_kline_interval(interval);

        // Use DerivativeCandles endpoint for futures
        let endpoint = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            GeminiEndpoint::DerivativeCandles
        } else {
            GeminiEndpoint::Candles
        };

        let response = self.get(
            endpoint,
            &[("symbol", &symbol_str), ("time_frame", time_frame)],
        ).await?;

        GeminiParser::parse_candles(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Gemini doesn't have a dedicated ping endpoint, use symbols as health check
        self.get(GeminiEndpoint::Symbols, &[]).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Fetch all symbols first, then get details for each
        let symbols_response = self.get(GeminiEndpoint::Symbols, &[]).await?;
        let symbols = GeminiParser::parse_symbols(&symbols_response)?;

        let mut result = Vec::with_capacity(symbols.len());

        for symbol_lower in &symbols {
            // Skip non-spot/perpetual symbols (e.g. contain digits like options)
            // Only process lowercase alpha symbols
            if !symbol_lower.chars().all(|c| c.is_alphabetic()) {
                continue;
            }

            match self.get(GeminiEndpoint::SymbolDetails, &[("symbol", symbol_lower)]).await {
                Ok(details) => {
                    if let Some(info) = GeminiParser::parse_symbol_details(&details, symbol_lower) {
                        result.push(info);
                    }
                }
                Err(_) => continue, // Skip symbols where details fetch fails
            }
        }

        self.precision.load_from_symbols(&result);
        Ok(result)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for GeminiConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));

        match req.order_type {
            OrderType::Market => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange market"));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("price".to_string(), json!(self.precision.price(&symbol_str, price)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange limit"));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // Gemini: type="exchange stop limit", stop_price=trigger, price=limit
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("price".to_string(), json!(self.precision.price(&symbol_str, limit_price)));
                params.insert("stop_price".to_string(), json!(self.precision.price(&symbol_str, stop_price)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange stop limit"));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::PostOnly { price } => {
                // Gemini: type="exchange limit" with options=["maker-or-cancel"]
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("price".to_string(), json!(self.precision.price(&symbol_str, price)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange limit"));
                params.insert("options".to_string(), json!(["maker-or-cancel"]));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Ioc { price } => {
                // Gemini: type="exchange limit" with options=["immediate-or-cancel"]
                let limit_price = price.unwrap_or(0.0);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("price".to_string(), json!(self.precision.price(&symbol_str, limit_price)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange limit"));
                params.insert("options".to_string(), json!(["immediate-or-cancel"]));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Fok { price } => {
                // Gemini: type="exchange limit" with options=["fill-or-kill"]
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), json!(symbol_str));
                params.insert("amount".to_string(), json!(self.precision.qty(&symbol_str, quantity)));
                params.insert("price".to_string(), json!(self.precision.price(&symbol_str, price)));
                params.insert("side".to_string(), json!(match side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                }));
                params.insert("type".to_string(), json!("exchange limit"));
                params.insert("options".to_string(), json!(["fill-or-kill"]));

                let response = self.post(GeminiEndpoint::NewOrder, params, &[]).await?;
                GeminiParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Gemini uses /v1/mytrades (PastTrades) for trade history
        let mut params = HashMap::new();

        // Add symbol filter if provided
        if let Some(ref symbol) = filter.symbol {
            let symbol_str = normalize_symbol(&format_symbol(&symbol.base, &symbol.quote, account_type));
            params.insert("symbol".to_string(), json!(symbol_str));
        }

        // Limit trades returned (max 500 per Gemini docs)
        let limit = filter.limit.unwrap_or(50).min(500);
        params.insert("limit_trades".to_string(), json!(limit));

        // Timestamp filter
        if let Some(since) = filter.start_time {
            params.insert("timestamp".to_string(), json!(since / 1000)); // convert ms to sec
        }

        let response = self.post(GeminiEndpoint::PastTrades, params, &[]).await?;
        GeminiParser::parse_past_trades(&response)
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let mut params = HashMap::new();
                params.insert("order_id".to_string(), json!(order_id.parse::<i64>().unwrap_or(0)));

                let response = self.post(GeminiEndpoint::CancelOrder, params, &[]).await?;
                GeminiParser::parse_order(&response)
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
        let mut params = HashMap::new();
        params.insert("order_id".to_string(), json!(order_id.parse::<i64>().unwrap_or(0)));

        let response = self.post(GeminiEndpoint::OrderStatus, params, &[]).await?;
        GeminiParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let response = self.post(GeminiEndpoint::ActiveOrders, HashMap::new(), &[]).await?;
        GeminiParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for GeminiConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let response = self.post(GeminiEndpoint::Balances, HashMap::new(), &[]).await?;
        GeminiParser::parse_balances(&response)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Gemini doesn't have a specific account info endpoint
        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0,
            taker_commission: 0.0,
            balances: vec![],
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Use /v1/notionalvolume which returns API fee tier in basis points
        let response = self.post(GeminiEndpoint::NotionalVolume, HashMap::new(), &[]).await?;
        GeminiParser::parse_notional_volume_fees(&response, symbol)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for GeminiConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let response = self.post(GeminiEndpoint::Positions, HashMap::new(), &[]).await?;
        GeminiParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let sym = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let symbol_str = normalize_symbol(&format_symbol(&sym.base, &sym.quote, AccountType::FuturesCross));

        let response = self.get(
            GeminiEndpoint::FundingAmount,
            &[("symbol", &symbol_str)],
        ).await?;

        GeminiParser::parse_funding_rate(&response)
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { .. } => {
                // Gemini doesn't have a set leverage endpoint
                Err(ExchangeError::NotSupported("Set leverage not supported by Gemini".to_string()))
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for GeminiConnector {
    async fn cancel_all_orders(
        &self,
        _scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        // Gemini /v1/order/cancel/all cancels all session orders globally.
        // There is no per-symbol cancel-all in the REST API.
        let response = self.post(GeminiEndpoint::CancelAllOrders, HashMap::new(), &[]).await?;

        // Response: {"result":"ok","details":{"cancelledOrders":[...],"cancelRejects":[...]}}
        let cancelled_count = response
            .get("details")
            .and_then(|d| d.get("cancelledOrders"))
            .and_then(|arr| arr.as_array())
            .map(|arr| arr.len() as u32)
            .unwrap_or(0);

        let failed_count = response
            .get("details")
            .and_then(|d| d.get("cancelRejects"))
            .and_then(|arr| arr.as_array())
            .map(|arr| arr.len() as u32)
            .unwrap_or(0);

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count,
            details: vec![],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for GeminiConnector {
    /// Get a new deposit address for an asset.
    ///
    /// Endpoint: POST /v1/deposit/{currency}/newAddress
    /// The `network` parameter is used as the currency path segment if provided.
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let currency = network.unwrap_or(asset).to_lowercase();
        let params = HashMap::new();

        let response = self.post(
            GeminiEndpoint::NewDepositAddress,
            params,
            &[("network", &currency)],
        ).await?;

        // Response: {"currency": "BTC", "address": "...", "label": "...", "timestamp": ...}
        let address = response.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing address in deposit address response".to_string()))?
            .to_string();

        let created_at = response.get("timestamp")
            .and_then(|v| v.as_i64());

        Ok(DepositAddress {
            address,
            tag: None, // Gemini doesn't return a tag/memo for standard addresses
            network: Some(currency),
            asset: asset.to_string(),
            created_at,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// Endpoint: POST /v1/withdraw/{currency}
    /// Params: address, amount
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let currency = req.asset.to_lowercase();

        let mut params = HashMap::new();
        params.insert("address".to_string(), json!(req.address));
        params.insert("amount".to_string(), json!(req.amount.to_string()));

        let response = self.post(
            GeminiEndpoint::Withdraw,
            params,
            &[("currency", &currency)],
        ).await?;

        // Response: {"destination": "...", "amount": "...", "txHash": "...", "withdrawalId": "..."}
        // or on error: {"result": "error", "reason": "..."}
        let withdraw_id = response.get("withdrawalId")
            .or_else(|| response.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tx_hash = response.get("txHash")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash,
        })
    }

    /// Get deposit and/or withdrawal history via the transfers endpoint.
    ///
    /// Endpoint: POST /v1/transfers
    /// Both deposits and withdrawals are returned by the same endpoint.
    /// Filtered client-side by `type` field ("Deposit" or "Withdrawal").
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut params = HashMap::new();

        if let Some(limit) = filter.limit {
            params.insert("limit_transfers".to_string(), json!(limit.min(50u32)));
        }

        if let Some(start) = filter.start_time {
            // Gemini uses Unix timestamp in seconds
            params.insert("timestamp".to_string(), json!(start / 1000));
        }

        let response = self.post(GeminiEndpoint::Transfers, params, &[]).await?;

        // Response is an array of transfer objects:
        // {"type": "Deposit"|"Withdrawal", "status": "...", "timestampms": ...,
        //  "eid": ..., "currency": "...", "amount": "...",
        //  "destination": "...", "txHash": "...", "feeAmount": "..."}
        let records = if let Some(arr) = response.as_array() {
            arr.iter().filter_map(|item| {
                let obj = item.as_object()?;

                let transfer_type = obj.get("type")?.as_str()?;
                let currency = obj.get("currency")?.as_str().unwrap_or("").to_uppercase();

                // Filter by asset if specified
                if let Some(ref asset_filter) = filter.asset {
                    if !currency.eq_ignore_ascii_case(asset_filter) {
                        return None;
                    }
                }

                let id = obj.get("eid").and_then(|v| v.as_i64()).map(|v| v.to_string())
                    .or_else(|| obj.get("eventId").and_then(|v| v.as_str()).map(|s| s.to_string()))
                    .unwrap_or_default();
                let amount_str = obj.get("amount").and_then(|v| v.as_str()).unwrap_or("0");
                let amount = amount_str.parse::<f64>().unwrap_or(0.0);
                let timestamp = obj.get("timestampms").and_then(|v| v.as_i64()).unwrap_or(0);
                let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                let tx_hash = obj.get("txHash")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                match transfer_type {
                    "Deposit" | "deposit" => {
                        if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
                            Some(FundsRecord::Deposit {
                                id,
                                asset: currency,
                                amount,
                                tx_hash,
                                network: None,
                                status,
                                timestamp,
                            })
                        } else {
                            None
                        }
                    }
                    "Withdrawal" | "withdrawal" => {
                        if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
                            let address = obj.get("destination")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let fee_str = obj.get("feeAmount").and_then(|v| v.as_str()).unwrap_or("0");
                            let fee = fee_str.parse::<f64>().ok().filter(|&f| f > 0.0);

                            Some(FundsRecord::Withdrawal {
                                id,
                                asset: currency,
                                amount,
                                fee,
                                address,
                                tag: None,
                                tx_hash,
                                network: None,
                                status,
                                timestamp,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }).collect()
        } else {
            vec![]
        };

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Gemini-специфичные)
// ═══════════════════════════════════════════════════════════════════════════════

impl GeminiConnector {
    /// Get all available symbols
    pub async fn get_symbols(&self) -> ExchangeResult<Vec<String>> {
        let response = self.get(GeminiEndpoint::Symbols, &[]).await?;
        GeminiParser::parse_symbols(&response)
    }

    /// Get notional volume and fee information
    pub async fn get_notional_volume(&self) -> ExchangeResult<Value> {
        self.post(GeminiEndpoint::NotionalVolume, HashMap::new(), &[]).await
    }

    /// Get funding payment history for perpetuals
    pub async fn get_funding_payments(
        &self,
        since: Option<i64>,
        to: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();

        if let Some(s) = since {
            params.insert("since".to_string(), json!(s));
        }
        if let Some(t) = to {
            params.insert("to".to_string(), json!(t));
        }

        self.post(GeminiEndpoint::FundingPayments, params, &[]).await
    }

    /// Get margin account summary
    pub async fn get_margin_info(&self) -> ExchangeResult<Value> {
        self.post(GeminiEndpoint::MarginAccount, HashMap::new(), &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = GeminiConnector::public(false).await.unwrap();
        assert_eq!(connector.exchange_id(), ExchangeId::Gemini);
        assert!(!connector.is_testnet());
    }

    #[test]
    fn test_format_symbol() {
        let symbol = format_symbol("BTC", "USD", AccountType::Spot);
        assert_eq!(symbol, "btcusd");

        let symbol = format_symbol("ETH", "USD", AccountType::FuturesCross);
        assert_eq!(symbol, "ethgusdperp");
    }
}
