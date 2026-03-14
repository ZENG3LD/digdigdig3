//! # Lighter Connector
//!
//! Implementation of core traits for Lighter DEX.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data (PUBLIC - Phase 1)
//! - `Trading` - Trading operations (STUB - Phase 3)
//! - `Account` - Account information (STUB - Phase 2)
//! - `Positions` - Futures positions (STUB - Phase 2)
//!
//! ## Implementation Status
//! - Phase 1 (Current): Public market data only
//! - Phase 2: Account data with auth tokens
//! - Phase 3: Trading with transaction signing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "k256-signing")]
use hex;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, Balance, AccountInfo,
    Position, FundingRate, PublicTrade,
    OrderRequest, CancelRequest,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::{ConnectorStats, SymbolInfo};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{LighterUrls, LighterEndpoint, map_kline_interval, format_symbol, symbol_to_market_id};
use super::auth::LighterAuth;
use super::parser::LighterParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Lighter DEX connector
pub struct LighterConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    _auth: Option<LighterAuth>,
    /// URLs (mainnet/testnet)
    urls: LighterUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (10,000 weight per minute)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl LighterConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            LighterUrls::TESTNET
        } else {
            LighterUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(LighterAuth::new)
            .transpose()?;

        // Initialize rate limiter: 10,000 weight per minute
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(10_000, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            _auth: auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            LighterUrls::TESTNET
        } else {
            LighterUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?;
        let auth = Some(LighterAuth::public_only());

        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(10_000, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            _auth: auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
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

    /// GET request
    async fn get(
        &self,
        endpoint: LighterEndpoint,
        params: HashMap<String, String>,
        weight: u32,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();

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

        // Lighter uses query params for auth, not headers (for most endpoints)
        let headers = HashMap::new();

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST request (used by trading methods — called from `#[cfg(feature = "k256-signing")]` code)
    #[cfg_attr(not(feature = "k256-signing"), allow(dead_code))]
    async fn post(
        &self,
        endpoint: LighterEndpoint,
        body: Value,
        weight: u32,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(weight).await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let _auth = self._auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
    }

    /// Fetch the next available nonce for the authenticated account.
    ///
    /// Lighter requires a unique, monotonically increasing nonce per transaction.
    /// The nonce is obtained from `GET /api/v1/nextNonce` and must be passed in
    /// the transaction payload before signing.
    #[cfg_attr(not(feature = "k256-signing"), allow(dead_code))]
    async fn fetch_next_nonce(&self, account_index: u64) -> ExchangeResult<u64> {
        let mut params = HashMap::new();
        params.insert("account_index".to_string(), account_index.to_string());

        let response = self.get(LighterEndpoint::NextNonce, params, 100).await?;

        response.get("nonce")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(
                "Missing or invalid 'nonce' field in nextNonce response".to_string()
            ))
    }

    /// Check response for errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        LighterParser::check_success(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET ID CONVERSION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get market_id for symbol using static mapping.
    ///
    /// Uses the shared `symbol_to_market_id` mapping from endpoints.rs.
    /// The `OrderBookDetails` REST endpoint is geo-blocked by CloudFront,
    /// so we rely on a static lookup instead.
    async fn get_market_id(&self, symbol: &str, _account_type: AccountType) -> ExchangeResult<u16> {
        // Extract base asset: "BTC" -> "BTC", "BTC/USDC" -> "BTC", "ETH/USDC" -> "ETH"
        let base = symbol.split('/').next().unwrap_or(symbol);
        symbol_to_market_id(base).ok_or_else(|| {
            ExchangeError::InvalidRequest(format!(
                "Unknown Lighter market for symbol '{}' (base: '{}')", symbol, base
            ))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for LighterConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Lighter
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
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
impl MarketData for LighterConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let market_id = self.get_market_id(&formatted_symbol, account_type).await?;

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());

        let response = self.get(LighterEndpoint::OrderBookDetails, params, 300).await?;
        LighterParser::parse_price(&response)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let market_id = self.get_market_id(&formatted_symbol, account_type).await?;

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());

        let response = self.get(LighterEndpoint::OrderBookDetails, params, 300).await?;
        LighterParser::parse_ticker(&response)
    }

    async fn get_orderbook(&self, symbol: Symbol, _depth: Option<u16>, account_type: AccountType) -> ExchangeResult<OrderBook> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let market_id = self.get_market_id(&formatted_symbol, account_type).await?;

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());

        let response = self.get(LighterEndpoint::OrderBookOrders, params, 300).await?;
        LighterParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let market_id = self.get_market_id(&formatted_symbol, account_type).await?;

        let bars = limit.unwrap_or(500).min(500) as u64;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let end_ms = end_time.map(|t| t as u64).unwrap_or(now_ms);

        let interval_ms = interval_to_ms(interval);
        // Use 2x buffer so we always get at least `bars` candles back
        let start_ms = end_ms.saturating_sub(interval_ms * bars * 2);

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());
        params.insert("resolution".to_string(), map_kline_interval(interval).to_string());
        params.insert("count_back".to_string(), bars.to_string());
        params.insert("end_timestamp".to_string(), end_ms.to_string());
        params.insert("start_timestamp".to_string(), start_ms.to_string());

        let response = self.get(LighterEndpoint::Candlesticks, params, 300).await?;
        LighterParser::parse_klines(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get(LighterEndpoint::Status, HashMap::new(), 300).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // The OrderBookDetails endpoint is geo-blocked by CloudFront.
        // Build the symbol list from the static market ID mapping instead.
        let known_symbols: &[(&str, u16)] = &[
            ("ETH", 0),
            ("BTC", 1),
            ("SOL", 2),
            ("ARB", 3),
            ("OP", 4),
            ("DOGE", 5),
            ("MATIC", 6),
            ("AVAX", 7),
            ("LINK", 8),
            ("SUI", 9),
            ("1000PEPE", 10),
            ("WIF", 11),
            ("SEI", 12),
            ("AAVE", 13),
            ("NEAR", 14),
            ("WLD", 15),
            ("FTM", 16),
            ("BONK", 17),
            ("APT", 19),
            ("BNB", 25),
        ];

        let is_spot = matches!(account_type, AccountType::Spot);

        let infos = known_symbols.iter().map(|(base, _market_id)| {
            let (symbol, quote_asset) = if is_spot {
                (format!("{}/USDC", base), "USDC".to_string())
            } else {
                (base.to_string(), "USDC".to_string())
            };
            SymbolInfo {
                symbol,
                base_asset: base.to_string(),
                quote_asset,
                status: "TRADING".to_string(),
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
                min_notional: None,
            }
        }).collect();

        Ok(infos)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for LighterConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        #[cfg(feature = "k256-signing")]
        {
            return self.place_order_signed(req).await;
        }

        #[cfg(not(feature = "k256-signing"))]
        {
            let _ = req;
            Err(ExchangeError::UnsupportedOperation(
                "Lighter order placement requires ECDSA transaction signing. \
                 Enable the 'k256-signing' feature flag to activate this.".to_string()
            ))
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        #[cfg(feature = "k256-signing")]
        {
            return self.cancel_order_signed(req).await;
        }

        #[cfg(not(feature = "k256-signing"))]
        {
            let _ = req;
            Err(ExchangeError::UnsupportedOperation(
                "Lighter order cancellation requires ECDSA transaction signing. \
                 Enable the 'k256-signing' feature flag to activate this.".to_string()
            ))
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Lighter does not have a GET-by-order-id endpoint for active orders.
        // Inactive (filled/cancelled) orders can be queried via accountInactiveOrders,
        // but that endpoint requires account_index and returns a list, not a single order.
        // For now, query all inactive orders and find by id.
        let account_index = self._auth.as_ref()
            .and_then(|a| a.account_index())
            .ok_or_else(|| ExchangeError::Auth(
                "Lighter get_order requires account_index in credentials passphrase JSON.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("account_index".to_string(), account_index.to_string());
        params.insert("limit".to_string(), "100".to_string());

        let response = self.get(LighterEndpoint::AccountInactiveOrders, params, 100).await?;
        let orders = LighterParser::parse_orders(&response)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::NotFound(format!("Order {} not found", order_id)))
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let account_index = self._auth.as_ref()
            .and_then(|a| a.account_index())
            .ok_or_else(|| ExchangeError::Auth(
                "Lighter get_open_orders requires account_index in credentials passphrase JSON.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("account_index".to_string(), account_index.to_string());

        // Optional market filter
        if let Some(sym) = symbol {
            if let Ok(market_id) = self.get_market_id(sym, account_type).await {
                params.insert("market_id".to_string(), market_id.to_string());
            }
        }

        // Auth token is currently optional on the server side per Lighter API docs
        // (both `auth` query param and `Authorization` header are `required: false`).
        // TODO: pass auth token once ECgFp5+Poseidon2 signing is implemented.

        let response = self.get(LighterEndpoint::AccountActiveOrders, params, 300).await?;
        let orders = LighterParser::parse_open_orders(&response)?;
        Ok(orders)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let account_index = self._auth.as_ref()
            .and_then(|a| a.account_index())
            .ok_or_else(|| ExchangeError::Auth(
                "Lighter get_order_history requires account_index in credentials passphrase JSON.".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("account_index".to_string(), account_index.to_string());

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        // Resolve market_id from symbol filter
        if let Some(sym) = &filter.symbol {
            let symbol_str = sym.base.as_str();
            if let Ok(market_id) = self.get_market_id(symbol_str, account_type).await {
                params.insert("market_id".to_string(), market_id.to_string());
            }
        }

        let response = self.get(LighterEndpoint::AccountInactiveOrders, params, 100).await?;
        let mut orders = LighterParser::parse_orders(&response)?;

        // Apply time filters
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
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for LighterConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        // Lighter account data is available via GET /api/v1/account
        // Query by account_index (from credentials) or l1_address.
        let (by_field, value) = self.resolve_account_query()?;

        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);

        let response = self.get(LighterEndpoint::Account, params, 3000).await?;
        let mut balances = LighterParser::parse_balance(&response)?;

        // Filter by asset if requested
        if let Some(asset_filter) = &query.asset {
            balances.retain(|b| b.asset.eq_ignore_ascii_case(asset_filter));
        }

        Ok(balances)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let (by_field, value) = self.resolve_account_query()?;

        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);

        let response = self.get(LighterEndpoint::Account, params, 3000).await?;

        let balances = LighterParser::parse_balance(&response)?;

        // Extract fees from the first available order_book
        let fees_response = self.get(LighterEndpoint::OrderBooks, HashMap::new(), 300).await;
        let (maker_commission, taker_commission) = if let Ok(fee_resp) = fees_response {
            let book = fee_resp.get("order_books")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .cloned();
            if let Some(b) = book {
                let maker = b.get("maker_fee")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let taker = b.get("taker_fee")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0001);
                (maker, taker)
            } else {
                (0.0, 0.0001)
            }
        } else {
            (0.0, 0.0001)
        };

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission,
            taker_commission,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Fetch fee schedule from the OrderBooks metadata endpoint.
        // The endpoint returns maker_fee and taker_fee per market.
        let mut params = HashMap::new();

        // If a symbol is given, resolve it to a market_id for a targeted request.
        if let Some(sym) = symbol {
            if let Ok(market_id) = self.get_market_id(sym, AccountType::FuturesCross).await {
                params.insert("market_id".to_string(), market_id.to_string());
            }
        }

        let response = self.get(LighterEndpoint::OrderBooks, params, 300).await?;

        // Parse first order book entry for fees (or global defaults).
        let order_books = response
            .get("order_books")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .cloned();

        let (maker_rate, taker_rate) = if let Some(book) = order_books {
            let maker = book.get("maker_fee")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let taker = book.get("taker_fee")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0001); // Lighter default taker: 0.01%
            (maker, taker)
        } else {
            // Lighter published defaults: maker 0%, taker 0.01%
            (0.0, 0.0001)
        };

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for LighterConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let (by_field, value) = self.resolve_account_query()?;

        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);

        let response = self.get(LighterEndpoint::Account, params, 3000).await?;
        let mut positions = LighterParser::parse_positions(&response)?;

        // Filter by symbol if requested
        if let Some(sym) = &query.symbol {
            let base = sym.base.to_uppercase();
            positions.retain(|p| p.symbol.to_uppercase() == base);
        }

        Ok(positions)
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
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

        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross)
        };
        let market_id = self.get_market_id(&formatted_symbol, AccountType::FuturesCross).await?;

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());

        let response = self.get(LighterEndpoint::Fundings, params, 300).await?;
        let mut funding = LighterParser::parse_funding_rate(&response)?;
        funding.symbol = symbol.to_string();
        Ok(funding)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { .. } => {
                // Lighter uses margin fractions set per-market at the protocol level.
                // There is no REST endpoint to change per-account leverage.
                Err(ExchangeError::UnsupportedOperation(
                    "Lighter does not support per-account leverage changes via REST. \
                     Leverage is controlled by initial margin fraction set at the market level.".to_string()
                ))
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} is not supported on Lighter", req)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Lighter-specific)
// ═══════════════════════════════════════════════════════════════════════════════

impl LighterConnector {
    /// Resolve the `by`/`value` query params for the `/api/v1/account` endpoint.
    ///
    /// Lighter supports lookup by:
    /// - `"index"` + numeric account_index
    /// - `"l1_address"` + Ethereum address
    ///
    /// This picks whichever credential field is available.
    fn resolve_account_query(&self) -> ExchangeResult<(String, String)> {
        let auth = self._auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth(
                "Lighter account queries require credentials (account_index or l1_address).".to_string()
            ))?;

        if let Some(idx) = auth.account_index() {
            return Ok(("index".to_string(), idx.to_string()));
        }

        if let Some(addr) = auth.l1_address() {
            return Ok(("l1_address".to_string(), addr.to_string()));
        }

        Err(ExchangeError::Auth(
            "Lighter account queries require either account_index or l1_address in credentials. \
             Pass them via Credentials::new(\"\", \"\").with_passphrase(r#\"{\"account_index\": 1}\"#).".to_string()
        ))
    }
}

impl LighterConnector {
    /// Get recent trades for a market
    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        account_type: AccountType,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<PublicTrade>> {
        let market_id = self.get_market_id(symbol, account_type).await?;

        let mut params = HashMap::new();
        params.insert("market_id".to_string(), market_id.to_string());
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LighterEndpoint::RecentTrades, params, 600).await?;
        LighterParser::parse_trades(&response)
    }

    /// Get exchange statistics
    pub async fn get_exchange_stats(&self) -> ExchangeResult<Value> {
        let response = self.get(LighterEndpoint::ExchangeStats, HashMap::new(), 300).await?;
        Ok(response)
    }

    /// Get current blockchain height
    pub async fn get_current_height(&self) -> ExchangeResult<i64> {
        let response = self.get(LighterEndpoint::CurrentHeight, HashMap::new(), 300).await?;
        response.get("height")
            .and_then(|h| h.as_i64())
            .ok_or_else(|| ExchangeError::Parse("Missing height field".to_string()))
    }

    /// Get all trading pairs
    pub async fn get_trading_pairs(&self, account_type: AccountType) -> ExchangeResult<Vec<String>> {
        let params = {
            let mut p = HashMap::new();
            let filter = match account_type {
                AccountType::Spot => "spot",
                _ => "perp",
            };
            p.insert("filter".to_string(), filter.to_string());
            p
        };

        let response = self.get(LighterEndpoint::OrderBookDetails, params, 300).await?;
        LighterParser::parse_trading_pairs(&response)
    }

    /// Get latest funding rates for all markets (or a specific market)
    ///
    /// Returns the current funding rate per market. Corresponds to
    /// `GET /api/v1/funding-rates`.
    pub async fn get_funding_rates(
        &self,
        market_id: Option<u16>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(id) = market_id {
            params.insert("market_id".to_string(), id.to_string());
        }
        self.get(LighterEndpoint::FundingRates, params, 300).await
    }

    /// Get exchange-level aggregate metrics
    ///
    /// Returns global statistics such as total volume, open interest, and
    /// number of active accounts. Corresponds to `GET /api/v1/exchangeMetrics`.
    pub async fn get_exchange_metrics(&self) -> ExchangeResult<Value> {
        self.get(LighterEndpoint::ExchangeMetrics, HashMap::new(), 300).await
    }

    /// Get account-level trading limits
    ///
    /// Returns order size limits, position limits, and other account caps.
    /// Corresponds to `GET /api/v1/accountLimits`.
    pub async fn get_account_limits(&self) -> ExchangeResult<Value> {
        let (by_field, value) = self.resolve_account_query()?;
        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);
        self.get(LighterEndpoint::AccountLimits, params, 300).await
    }

    /// Get account metadata (tier, settings, referral info)
    ///
    /// Corresponds to `GET /api/v1/accountMetadata`.
    pub async fn get_account_metadata(&self) -> ExchangeResult<Value> {
        let (by_field, value) = self.resolve_account_query()?;
        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);
        self.get(LighterEndpoint::AccountMetadata, params, 300).await
    }

    /// Get per-position funding payment history
    ///
    /// Returns historical funding payments for each open or recently closed
    /// position. Corresponds to `GET /api/v1/positionFunding`.
    pub async fn get_position_funding(
        &self,
        market_id: Option<u16>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let (by_field, value) = self.resolve_account_query()?;
        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);
        if let Some(id) = market_id {
            params.insert("market_id".to_string(), id.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(LighterEndpoint::PositionFunding, params, 300).await
    }

    /// Get liquidation history for the account
    ///
    /// Returns a list of past liquidation events. Corresponds to
    /// `GET /api/v1/liquidations`.
    pub async fn get_liquidations(
        &self,
        market_id: Option<u16>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let (by_field, value) = self.resolve_account_query()?;
        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);
        if let Some(id) = market_id {
            params.insert("market_id".to_string(), id.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(LighterEndpoint::Liquidations, params, 300).await
    }

    /// Get pending withdrawal delay information
    ///
    /// Returns the remaining delay period before queued withdrawals can be
    /// finalised on-chain. Corresponds to `GET /api/v1/withdrawalDelays`.
    pub async fn get_withdrawal_delays(&self) -> ExchangeResult<Value> {
        let (by_field, value) = self.resolve_account_query()?;
        let mut params = HashMap::new();
        params.insert("by".to_string(), by_field);
        params.insert("value".to_string(), value);
        self.get(LighterEndpoint::WithdrawalDelays, params, 300).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIGNED TRADING METHODS (k256-signing feature)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "k256-signing")]
impl LighterConnector {
    /// Place an order on Lighter using secp256k1 ECDSA signing (tx_type = 14).
    ///
    /// # Flow
    ///
    /// 1. Resolve account_index and market_id from credentials / symbol.
    /// 2. Fetch the next nonce from `GET /api/v1/nextNonce`.
    /// 3. Build the L2CreateOrder tx fields.
    /// 4. Compute a deterministic 32-byte hash over the canonical tx payload.
    /// 5. Sign the hash with the API key private key (secp256k1 ECDSA via k256).
    /// 6. POST the signed payload to `POST /api/v1/sendTx`.
    /// 7. Parse the response and return a `PlaceOrderResponse::Simple`.
    ///
    /// # Signing Note
    ///
    /// Lighter's native L2 protocol uses ECgFp5+Poseidon2 over the Goldilocks
    /// field, not secp256k1 ECDSA. This implementation uses secp256k1 signing
    /// as a bridge for testing and EVM-compatible tooling. The hash construction
    /// (`build_create_order_hash`) is SHA-256 based — production use requires
    /// porting the ECgFp5/Poseidon2 primitives from the Lighter TypeScript SDK.
    pub(crate) async fn place_order_signed(
        &self,
        req: OrderRequest,
    ) -> ExchangeResult<PlaceOrderResponse> {
        let auth = self._auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth(
                "Authentication required for place_order. \
                 Provide credentials with api_key_index, account_index, and api_secret.".to_string()
            ))?;

        let account_index = auth.account_index()
            .ok_or_else(|| ExchangeError::Auth(
                "account_index required in credentials passphrase JSON for Lighter order placement. \
                 Example: Credentials::new(\"\", \"<private_key_hex>\").with_passphrase(r#\"{\"account_index\": 1, \"api_key_index\": 0}\"#)".to_string()
            ))?;

        // Resolve market_id from the order symbol
        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, req.account_type);
        let market_id = self.get_market_id(&symbol_str, req.account_type).await?;

        // Fetch nonce
        let nonce = self.fetch_next_nonce(account_index).await?;

        // Determine order direction (Lighter: is_ask = true for sells)
        let is_ask = matches!(req.side, crate::core::OrderSide::Sell);

        // Extract price and quantity from the order type
        let (price_str, order_type_code) = match &req.order_type {
            crate::core::OrderType::Limit { price } => {
                (format!("{:.8}", price), 0u8) // 0 = limit
            }
            crate::core::OrderType::Market => {
                // Market orders use price = 0 on Lighter
                ("0".to_string(), 1u8) // 1 = market
            }
            crate::core::OrderType::PostOnly { price } => {
                (format!("{:.8}", price), 2u8) // 2 = post-only
            }
            crate::core::OrderType::Ioc { price } => {
                let p = price.unwrap_or(0.0);
                (format!("{:.8}", p), 3u8) // 3 = IOC
            }
            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Lighter only supports Limit, Market, PostOnly, and IOC order types.".to_string()
                ));
            }
        };

        let base_amount_str = format!("{:.8}", req.quantity);

        // Time-in-force code (Lighter: 0=GTC, 1=IOC, 2=FOK, 3=GTB)
        let tif_code: u8 = match req.time_in_force {
            crate::core::TimeInForce::Gtc => 0,
            crate::core::TimeInForce::Ioc => 1,
            crate::core::TimeInForce::Fok => 2,
            crate::core::TimeInForce::GoodTilBlock { .. } => 3,
            _ => 0,
        };

        // Build hash and sign
        let tx_hash = auth.build_create_order_hash(
            account_index,
            market_id,
            is_ask,
            &base_amount_str,
            &price_str,
            nonce,
            order_type_code,
            tif_code,
        );

        let signature_bytes = auth.sign_l2_transaction(&tx_hash)?;
        let signature_hex = hex::encode(&signature_bytes);

        // Build the tx_info payload
        let tx_info = serde_json::json!({
            "account_index": account_index,
            "market_id": market_id,
            "is_ask": is_ask,
            "base_amount": base_amount_str,
            "price": price_str,
            "nonce": nonce,
            "order_type": order_type_code,
            "time_in_force": tif_code,
            "signature": signature_hex,
        });

        let body = serde_json::json!({
            "tx_type": 14,
            "tx_info": tx_info,
        });

        let response = self.post(LighterEndpoint::SendTx, body, 100).await?;

        // Parse the returned order from the response
        let order_index = response.get("order_index")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let order_type_parsed = match &req.order_type {
            crate::core::OrderType::Limit { price } => crate::core::OrderType::Limit { price: *price },
            crate::core::OrderType::Market => crate::core::OrderType::Market,
            crate::core::OrderType::PostOnly { price } => crate::core::OrderType::PostOnly { price: *price },
            crate::core::OrderType::Ioc { price } => crate::core::OrderType::Ioc { price: *price },
            other => other.clone(),
        };

        let order = crate::core::Order {
            id: order_index.to_string(),
            client_order_id: req.client_order_id.clone(),
            symbol: symbol_str,
            side: req.side,
            order_type: order_type_parsed,
            status: crate::core::types::OrderStatus::New,
            price: match &req.order_type {
                crate::core::OrderType::Limit { price } => Some(*price),
                crate::core::OrderType::PostOnly { price } => Some(*price),
                crate::core::OrderType::Ioc { price } => *price,
                _ => None,
            },
            stop_price: None,
            quantity: req.quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: now_ms,
            updated_at: None,
            time_in_force: req.time_in_force,
        };

        Ok(PlaceOrderResponse::Simple(order))
    }

    /// Cancel an order on Lighter using secp256k1 ECDSA signing (tx_type = 15).
    ///
    /// # Flow
    ///
    /// 1. Extract order_index from `CancelScope::Single { order_id }`.
    /// 2. Resolve account_index and market_id.
    /// 3. Fetch the next nonce.
    /// 4. Build hash and sign the L2CancelOrder transaction.
    /// 5. POST to `POST /api/v1/sendTx`.
    ///
    /// Only `CancelScope::Single` is supported — Lighter cancels one order per
    /// signed transaction. For bulk cancellation, use `cancel_order` in a loop
    /// or use the batch endpoint via `POST /api/v1/sendTxBatch`.
    pub(crate) async fn cancel_order_signed(
        &self,
        req: CancelRequest,
    ) -> ExchangeResult<crate::core::Order> {
        let auth = self._auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth(
                "Authentication required for cancel_order.".to_string()
            ))?;

        let account_index = auth.account_index()
            .ok_or_else(|| ExchangeError::Auth(
                "account_index required in credentials passphrase JSON for Lighter order cancellation.".to_string()
            ))?;

        // Extract order_index from the cancel scope
        let order_id_str = match &req.scope {
            crate::core::types::CancelScope::Single { order_id } => order_id.clone(),
            other => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!(
                        "Lighter cancel_order only supports CancelScope::Single. \
                         Got: {:?}. Each Lighter cancel requires a signed transaction per order.",
                        other
                    )
                ));
            }
        };

        let order_index: u64 = order_id_str.parse().map_err(|_| {
            ExchangeError::InvalidRequest(format!(
                "Lighter order_id must be a numeric order_index, got '{}'", order_id_str
            ))
        })?;

        // Resolve market_id from optional symbol hint
        let market_id = if let Some(sym) = &req.symbol {
            let sym_str = format_symbol(&sym.base, &sym.quote, req.account_type);
            self.get_market_id(&sym_str, req.account_type).await?
        } else {
            return Err(ExchangeError::InvalidRequest(
                "Lighter cancel_order requires a symbol hint to determine market_id. \
                 Set CancelRequest::symbol to the symbol of the order being cancelled.".to_string()
            ));
        };

        // Fetch nonce
        let nonce = self.fetch_next_nonce(account_index).await?;

        // Build hash and sign
        let tx_hash = auth.build_cancel_order_hash(account_index, market_id, order_index, nonce);
        let signature_bytes = auth.sign_l2_transaction(&tx_hash)?;
        let signature_hex = hex::encode(&signature_bytes);

        let tx_info = serde_json::json!({
            "account_index": account_index,
            "market_id": market_id,
            "order_index": order_index,
            "nonce": nonce,
            "signature": signature_hex,
        });

        let body = serde_json::json!({
            "tx_type": 15,
            "tx_info": tx_info,
        });

        let _response = self.post(LighterEndpoint::SendTx, body, 100).await?;

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let symbol_str = req.symbol
            .as_ref()
            .map(|s| format_symbol(&s.base, &s.quote, req.account_type))
            .unwrap_or_default();

        Ok(crate::core::Order {
            id: order_id_str,
            client_order_id: None,
            symbol: symbol_str,
            side: crate::core::types::OrderSide::Buy, // Unknown at cancel time
            order_type: crate::core::OrderType::Limit { price: 0.0 },
            status: crate::core::types::OrderStatus::Canceled,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: now_ms,
            updated_at: Some(now_ms),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a kline interval string to milliseconds.
///
/// Used to compute the `start_timestamp` for the `/api/v1/candles` endpoint.
fn interval_to_ms(interval: &str) -> u64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "15m" => 900_000,
        "30m" => 1_800_000,
        "1h" => 3_600_000,
        "4h" => 14_400_000,
        "12h" => 43_200_000,
        "1d" => 86_400_000,
        "1w" => 604_800_000,
        _ => 3_600_000, // default 1h
    }
}
