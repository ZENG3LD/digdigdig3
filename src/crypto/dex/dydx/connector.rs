//! # dYdX v4 Connector
//!
//! Реализация всех core трейтов для dYdX v4 Indexer API.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные (read-only via Indexer)
//! - `Account` - информация об аккаунте (read-only via Indexer)
//! - `Positions` - perpetual futures позиции (read-only via Indexer)
//!
//! ## Limitations
//! - Текущая реализация: только Indexer API (read-only)
//! - Trading операции требуют Node API (gRPC) - будущая реализация

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook, Balance, AccountInfo,
    Position, FundingRate,
    Order, OrderSide, OrderStatus, OrderType, TimeInForce,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::SimpleRateLimiter;
use crate::core::types::{ConnectorStats, SymbolInfo};

use super::endpoints::{DydxUrls, DydxEndpoint, format_symbol, map_kline_interval};
use super::auth::DydxAuth;
use super::parser::DydxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// dYdX v4 коннектор
pub struct DydxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (не используется для Indexer API)
    auth: DydxAuth,
    /// URL'ы (mainnet/testnet)
    urls: DydxUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (conservative guard: 100 req/10s)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl DydxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            DydxUrls::TESTNET
        } else {
            DydxUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = DydxAuth::new(credentials.as_ref())?;

        // Conservative guard: 100 requests per 10 seconds
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
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

    /// GET запрос к Indexer API
    async fn get(
        &self,
        endpoint: DydxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.indexer_rest;
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in &params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        // Build query string from remaining params
        let mut query_params: Vec<String> = Vec::new();
        for (key, value) in &params {
            if !path.contains(value) {
                query_params.push(format!("{}={}", key, value));
            }
        }

        let query = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);
        let headers = self.auth.sign_request("GET", &path, "");

        self.http.get_with_headers(&url, &HashMap::new(), &headers).await
    }

    /// Извлечь data field или вернуть весь response
    fn _unwrap_response(&self, response: Value) -> Value {
        response
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DydxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Dydx
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_count(), limiter.max_requests())
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::FuturesCross, AccountType::FuturesIsolated]
    }
}

#[async_trait]
impl MarketData for DydxConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;
        DydxParser::parse_price(&response, &market)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;
        DydxParser::parse_ticker(&response, &market)
    }

    async fn get_orderbook(&self, symbol: Symbol, _depth: Option<u16>, _account_type: AccountType) -> ExchangeResult<OrderBook> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());

        let response = self.get(DydxEndpoint::Orderbook, params).await?;
        DydxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let market = format_symbol(&symbol.base, &symbol.quote, _account_type);
        let resolution = map_kline_interval(interval);

        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());
        params.insert("resolution".to_string(), resolution.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }
        if let Some(et) = end_time {
            if let Some(dt) = chrono::DateTime::from_timestamp_millis(et) {
                params.insert("toISO".to_string(), dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
            }
        }

        let response = self.get(DydxEndpoint::Candles, params).await?;
        DydxParser::parse_klines(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(DydxEndpoint::ServerTime, HashMap::new()).await?;
        if response.get("epoch").is_some() {
            Ok(())
        } else {
            Err(ExchangeError::Api {
                code: 0,
                message: "Ping failed".to_string(),
            })
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        let infos = markets.iter().map(|(ticker, data)| {
            // dYdX uses "BTC-USD" format
            let parts: Vec<&str> = ticker.splitn(2, '-').collect();
            let base = parts.first().copied().unwrap_or(ticker).to_string();
            let quote = parts.get(1).copied().unwrap_or("USD").to_string();

            let status = data.get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("ACTIVE")
                .to_string();

            // Parse step size / tick size for precision hints
            let step_size = data.get("stepSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = data.get("minOrderSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            SymbolInfo {
                symbol: ticker.clone(),
                base_asset: base,
                quote_asset: quote,
                status,
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: min_notional,
                max_quantity: None,
                step_size,
                min_notional: None,
            }
        }).collect();

        Ok(infos)
    }
}

#[async_trait]
impl Account for DydxConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        // Note: Requires address parameter
        // Placeholder implementation - требует address
        Err(ExchangeError::NotSupported(
            "get_balance requires address parameter in dYdX. Use get_subaccount_balances instead.".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Requires address - placeholder
        Err(ExchangeError::NotSupported(
            "get_account_info requires address parameter in dYdX".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

#[async_trait]
impl Positions for DydxConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "get_positions requires address and subaccountNumber parameters in dYdX. Use get_subaccount_positions instead.".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "get_positions requires address and subaccountNumber parameters in dYdX. Use get_subaccount_positions instead.".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "get_positions requires address and subaccountNumber parameters in dYdX. Use get_subaccount_positions instead.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING (Read-only via Indexer; write operations require Node gRPC)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for DydxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        // dYdX v4 order placement requires Cosmos SDK gRPC (MsgPlaceOrder).
        // The Indexer REST API is read-only; write operations go through validator
        // nodes via gRPC/Protobuf and require a signed Cosmos transaction.
        // This is beyond the REST-only scope of this connector.
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 order placement requires Cosmos gRPC (Node API). \
             The Indexer REST API is read-only. Implement via gRPC MsgPlaceOrder.".to_string()
        ))
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        // dYdX v4 order cancellation also requires Node gRPC (MsgCancelOrder).
        let _ = req;
        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 order cancellation requires Cosmos gRPC (Node API). \
             The Indexer REST API is read-only.".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(DydxEndpoint::SpecificOrder, params).await?;
        DydxParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Requires address + subaccountNumber — not available in the generic trait call.
        // Return UnsupportedOperation with a helpful message.
        let _ = symbol;
        Err(ExchangeError::UnsupportedOperation(
            "dYdX open orders require address and subaccountNumber. \
             Use get_orders_for_subaccount() instead.".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Also requires address + subaccountNumber.
        Err(ExchangeError::UnsupportedOperation(
            "dYdX order history requires address and subaccountNumber. \
             Use get_orders_for_subaccount() instead.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS
// ═══════════════════════════════════════════════════════════════════════════════

impl DydxConnector {
    /// Получить balances для конкретного subaccount
    pub async fn get_subaccount_balances(
        &self,
        address: &str,
        subaccount_number: u32,
    ) -> ExchangeResult<Vec<Balance>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), subaccount_number.to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        DydxParser::parse_balances(&response)
    }

    /// Получить positions для конкретного subaccount
    pub async fn get_subaccount_positions(
        &self,
        address: &str,
        subaccount_number: u32,
    ) -> ExchangeResult<Vec<Position>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), subaccount_number.to_string());

        let response = self.get(DydxEndpoint::PerpetualPositions, params).await?;
        DydxParser::parse_positions(&response)
    }

    /// Получить market info (для clobPairId mapping)
    pub async fn get_market_info(&self, ticker: &str) -> ExchangeResult<Value> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        markets.get(ticker)
            .cloned()
            .ok_or_else(|| ExchangeError::Parse(format!("Market {} not found", ticker)))
    }

    /// Получить orders для конкретного subaccount (read-only via Indexer)
    pub async fn get_orders_for_subaccount(
        &self,
        address: &str,
        subaccount_number: u32,
        ticker: Option<&str>,
        status: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), subaccount_number.to_string());
        if let Some(t) = ticker {
            params.insert("ticker".to_string(), t.to_string());
        }
        if let Some(s) = status {
            params.insert("status".to_string(), s.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let response = self.get(DydxEndpoint::Orders, params).await?;
        DydxParser::parse_orders(&response)
    }

    /// Получить все markets
    pub async fn get_all_markets(&self) -> ExchangeResult<HashMap<String, Value>> {
        let response = self.get(DydxEndpoint::PerpetualMarkets, HashMap::new()).await?;

        let markets = response.get("markets")
            .and_then(|m| m.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing markets".to_string()))?;

        Ok(markets.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
}
