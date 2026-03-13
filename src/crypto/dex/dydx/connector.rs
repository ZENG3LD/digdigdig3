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
    Order, OrderRequest, CancelRequest,
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
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        // dYdX balances are per-subaccount; address is stored in credentials.
        // If credentials contain an address, use subaccount 0 by default.
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_balance requires a dYdX address. Provide it via Credentials::new(address, \"\").".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), "0".to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        let mut balances = DydxParser::parse_balances(&response)?;

        // Filter by asset if requested
        if let Some(asset_filter) = &query.asset {
            balances.retain(|b| b.asset.eq_ignore_ascii_case(asset_filter));
        }

        Ok(balances)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_account_info requires a dYdX address.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccount_number".to_string(), "0".to_string());

        let response = self.get(DydxEndpoint::SpecificSubaccount, params).await?;
        let balances = DydxParser::parse_balances(&response)?;

        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0,   // dYdX fees vary; fills endpoint needed
            taker_commission: 0.0005, // dYdX default taker: 0.05%
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // dYdX does not expose a dedicated fee-schedule endpoint.
        // We approximate fees from fills: compute effective rate as fee/size.
        // Without an address we return the published default schedule.
        let (maker_rate, taker_rate) = if let Some(address) = self.auth.address() {
            let mut params = HashMap::new();
            params.insert("address".to_string(), address.to_string());
            params.insert("subaccountNumber".to_string(), "0".to_string());
            params.insert("limit".to_string(), "10".to_string());
            if let Some(sym) = symbol {
                params.insert("ticker".to_string(), sym.to_string());
            }

            match self.get(DydxEndpoint::Fills, params).await {
                Ok(response) => {
                    let fills = response.get("fills")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    if fills.is_empty() {
                        // No fills — use published defaults
                        (0.0, 0.0005)
                    } else {
                        // Compute effective fee rate from recent fills
                        let mut total_fee = 0.0f64;
                        let mut total_value = 0.0f64;
                        let mut maker_count = 0usize;
                        let mut taker_count = 0usize;

                        for fill in &fills {
                            let size: f64 = fill.get("size")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let price: f64 = fill.get("price")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let fee: f64 = fill.get("fee")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0.0);
                            let liquidity = fill.get("liquidity")
                                .and_then(|v| v.as_str())
                                .unwrap_or("TAKER");

                            total_fee += fee.abs();
                            total_value += size * price;
                            if liquidity == "MAKER" { maker_count += 1; } else { taker_count += 1; }
                        }

                        let effective_rate = if total_value > 0.0 { total_fee / total_value } else { 0.0005 };

                        // Estimate maker/taker split from liquidity counts
                        let total = (maker_count + taker_count) as f64;
                        if total == 0.0 {
                            (0.0, effective_rate)
                        } else {
                            let maker_share = maker_count as f64 / total;
                            let taker_share = taker_count as f64 / total;
                            // Maker rate is typically negative (rebate) or zero on dYdX
                            let implied_taker = if taker_share > 0.0 { effective_rate / taker_share } else { 0.0005 };
                            let implied_maker = if maker_share > 0.0 { -(effective_rate * 0.1) } else { 0.0 };
                            (implied_maker, implied_taker)
                        }
                    }
                }
                Err(_) => (0.0, 0.0005), // Fallback to published defaults
            }
        } else {
            // No credentials — published default fee schedule
            // dYdX v4: maker rebate ~ -0.011%, taker fee ~ 0.050%
            (-0.00011, 0.0005)
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
impl Positions for DydxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_positions requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());
        params.insert("status".to_string(), "OPEN".to_string());

        if let Some(sym) = &query.symbol {
            // dYdX symbol format: BTC-USD
            let market = format!("{}-USD", sym.base.to_uppercase());
            params.insert("market".to_string(), market);
        }

        let response = self.get(DydxEndpoint::PerpetualPositions, params).await?;
        DydxParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Normalize symbol: "BTC" → "BTC-USD", "BTC-USD" → "BTC-USD"
        let market = if symbol.contains('-') {
            symbol.to_uppercase()
        } else {
            format!("{}-USD", symbol.to_uppercase())
        };

        let mut params = HashMap::new();
        params.insert("market".to_string(), market.clone());
        params.insert("limit".to_string(), "1".to_string());

        let response = self.get(DydxEndpoint::HistoricalFunding, params).await?;
        let mut funding = DydxParser::parse_funding_rate(&response)?;

        // Override symbol with the normalized market ticker
        funding.symbol = market;
        Ok(funding)
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "dYdX v4 position modification (leverage, margin mode) requires Cosmos gRPC (Node API). \
             The Indexer REST API is read-only.".to_string()
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
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_open_orders requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());
        params.insert("status".to_string(), "OPEN".to_string());

        if let Some(sym) = symbol {
            // Normalize to dYdX format
            let market = if sym.contains('-') {
                sym.to_uppercase()
            } else {
                format!("{}-USD", sym.to_uppercase())
            };
            params.insert("ticker".to_string(), market);
        }

        let response = self.get(DydxEndpoint::Orders, params).await?;
        // Orders endpoint returns an array directly
        DydxParser::parse_orders(&response)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let address = self.auth.address()
            .ok_or_else(|| ExchangeError::Auth(
                "dYdX get_order_history requires a dYdX address in credentials.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("subaccountNumber".to_string(), "0".to_string());

        if let Some(sym) = &filter.symbol {
            let market = format!("{}-USD", sym.base.to_uppercase());
            params.insert("ticker".to_string(), market);
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }
        // Filter to non-open orders (filled, canceled)
        params.insert("returnLatestOrders".to_string(), "true".to_string());

        let response = self.get(DydxEndpoint::Orders, params).await?;
        let mut orders = DydxParser::parse_orders(&response)?;

        // Apply time filters if provided
        if let Some(start) = filter.start_time {
            orders.retain(|o| o.created_at >= start);
        }
        if let Some(end) = filter.end_time {
            orders.retain(|o| o.created_at <= end);
        }

        Ok(orders)
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

    /// Get transfers between two subaccounts
    ///
    /// Returns transfers from `source_subaccount_number` to `recipient_subaccount_number`
    /// for the given `address`.
    pub async fn get_transfers_between(
        &self,
        address: &str,
        source_subaccount_number: u32,
        recipient_subaccount_number: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("sourceSubaccountNumber".to_string(), source_subaccount_number.to_string());
        params.insert("recipientSubaccountNumber".to_string(), recipient_subaccount_number.to_string());
        self.get(DydxEndpoint::TransfersBetween, params).await
    }

    /// Get asset positions for a parent subaccount number
    ///
    /// Returns asset positions across all child subaccounts under the given parent.
    pub async fn get_parent_asset_positions(
        &self,
        address: &str,
        parent_subaccount_number: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("parentSubaccountNumber".to_string(), parent_subaccount_number.to_string());
        self.get(DydxEndpoint::ParentAssetPositions, params).await
    }

    /// Get transfers for a parent subaccount number
    pub async fn get_parent_transfers(
        &self,
        address: &str,
        parent_subaccount_number: u32,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        params.insert("parentSubaccountNumber".to_string(), parent_subaccount_number.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(DydxEndpoint::ParentTransfers, params).await
    }

    /// Get MegaVault historical PnL
    ///
    /// Returns historical profit and loss data for the dYdX MegaVault.
    pub async fn get_megavault_pnl(
        &self,
        resolution: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(r) = resolution {
            params.insert("resolution".to_string(), r.to_string());
        }
        self.get(DydxEndpoint::MegaVaultPnl, params).await
    }

    /// Get MegaVault positions
    ///
    /// Returns current positions held in the dYdX MegaVault.
    pub async fn get_megavault_positions(&self) -> ExchangeResult<Value> {
        self.get(DydxEndpoint::MegaVaultPositions, HashMap::new()).await
    }

    /// Get historical PnL for all individual vaults
    ///
    /// Returns historical PnL data for all vaults (not just the MegaVault).
    pub async fn get_all_vaults_pnl(
        &self,
        resolution: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(r) = resolution {
            params.insert("resolution".to_string(), r.to_string());
        }
        self.get(DydxEndpoint::AllVaultsPnl, params).await
    }

    /// Get affiliate program metadata for an address
    pub async fn get_affiliate_metadata(&self, address: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), address.to_string());
        self.get(DydxEndpoint::AffiliateMetadata, params).await
    }

    /// Get affiliate address info for a referral code
    pub async fn get_affiliate_address(&self, referral_code: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("referralCode".to_string(), referral_code.to_string());
        self.get(DydxEndpoint::AffiliateAddress, params).await
    }
}
