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

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType,Balance, AccountInfo,
    Position, FundingRate, PublicTrade,
    OrderRequest, CancelRequest, CancelScope,
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

    /// POST request (for trading - Phase 3)
    async fn _post(
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

        // TODO Phase 3: Implement transaction signing
        let headers = HashMap::new();

        let response = self.http.post(&url, &body, &headers).await?;
        self.check_response(&response)?;
        Ok(response)
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
// TRADING (Stubs for Phase 3)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for LighterConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let _symbol = req.symbol.clone();
        let _side = req.side;
        let _quantity = req.quantity;
        let _account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                Err(ExchangeError::UnsupportedOperation("Trading not yet implemented (Phase 3)".to_string()))
            }
            OrderType::Limit { price: _price } => {
                Err(ExchangeError::UnsupportedOperation("Trading not yet implemented (Phase 3)".to_string()))
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
            CancelScope::Single { order_id: ref _order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            Err(ExchangeError::UnsupportedOperation("Trading not yet implemented (Phase 3)".to_string()))

            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        Err(ExchangeError::UnsupportedOperation("Trading not yet implemented (Phase 3)".to_string()))
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        Err(ExchangeError::UnsupportedOperation("Trading not yet implemented (Phase 3)".to_string()))
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT (Stubs for Phase 2)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for LighterConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::NotSupported("Account data not yet implemented (Phase 2)".to_string()))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::NotSupported("Account data not yet implemented (Phase 2)".to_string()))
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
// POSITIONS (Stubs for Phase 2)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for LighterConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        Err(ExchangeError::NotSupported("Positions not yet implemented (Phase 2)".to_string()))
    
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
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                Err(ExchangeError::NotSupported("Leverage not yet implemented (Phase 2)".to_string()))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (Lighter-specific)
// ═══════════════════════════════════════════════════════════════════════════════

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
