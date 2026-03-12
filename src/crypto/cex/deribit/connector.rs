//! # Deribit Connector
//!
//! Implementation of core traits for Deribit exchange.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Futures/options positions
//!
//! ## JSON-RPC Request Pattern
//! All requests use POST with JSON-RPC 2.0 format

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
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
use crate::core::types::{ConnectorStats, SymbolInfo};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::DecayingRateLimiter;

use super::endpoints::{DeribitUrls, DeribitMethod, format_symbol};
use super::auth::DeribitAuth;
use super::parser::DeribitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Deribit connector
pub struct DeribitConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Arc<Mutex<Option<DeribitAuth>>>,
    /// URLs (mainnet/testnet)
    urls: DeribitUrls,
    /// Testnet mode
    testnet: bool,
    /// Request counter for JSON-RPC ID
    request_id: Arc<Mutex<u64>>,
    /// Rate limiter (Deribit credit system: max 10000 credits, refill 10000/s, cost 500/request)
    rate_limiter: Arc<Mutex<DecayingRateLimiter>>,
}

impl DeribitConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            DeribitUrls::TESTNET
        } else {
            DeribitUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(DeribitAuth::new)
            .transpose()?;

        // Deribit rate limit: 10000 credits max, refills at 10000/s, each request costs 500
        let rate_limiter = Arc::new(Mutex::new(
            DecayingRateLimiter::new(10000.0, 10000.0)
        ));

        let connector = Self {
            http,
            auth: Arc::new(Mutex::new(auth)),
            urls,
            testnet,
            request_id: Arc::new(Mutex::new(1)),
            rate_limiter,
        };

        // Authenticate if we have credentials
        if credentials.is_some() {
            connector.authenticate().await?;
        }

        Ok(connector)
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTHENTICATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Authenticate using client credentials grant
    async fn authenticate(&self) -> ExchangeResult<()> {
        let params = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            let auth = auth_guard.as_ref()
                .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

            // Use client signature (more secure than client credentials)
            auth.client_signature_params()
        };

        let response = self.rpc_call(DeribitMethod::Auth, params).await?;
        let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

        // Store tokens
        let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
        if let Some(auth) = auth_guard.as_mut() {
            auth.store_tokens(access_token, refresh_token, expires_in);
        }

        Ok(())
    }

    /// Refresh access token
    async fn _refresh_token(&self) -> ExchangeResult<()> {
        let params = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            let auth = auth_guard.as_ref()
                .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

            auth.refresh_token_params()?
        };

        let response = self.rpc_call(DeribitMethod::Auth, params).await?;
        let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

        // Store new tokens
        let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
        if let Some(auth) = auth_guard.as_mut() {
            auth.store_tokens(access_token, refresh_token, expires_in);
        }

        Ok(())
    }

    /// Ensure we have a valid access token (non-recursive version)
    async fn ensure_authenticated(&self) -> ExchangeResult<()> {
        let needs_refresh = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            if let Some(auth) = auth_guard.as_ref() {
                !auth.has_valid_token()
            } else {
                return Err(ExchangeError::Auth("No credentials configured".to_string()));
            }
        };

        if needs_refresh {
            // Directly refresh without going through ensure_authenticated again
            let params = {
                let auth_guard = self.auth.lock().expect("Mutex poisoned");
                let auth = auth_guard.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

                auth.refresh_token_params()?
            };

            let response = self.rpc_call_internal(DeribitMethod::Auth, params).await?;
            let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

            // Store new tokens
            let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
            if let Some(auth) = auth_guard.as_mut() {
                auth.store_tokens(access_token, refresh_token, expires_in);
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RATE LIMITING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary (cost = 500 credits per request)
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(500.0) {
                    return;
                }
                limiter.time_until_ready(500.0)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Internal RPC call without auth check (to avoid recursion)
    async fn rpc_call_internal(
        &self,
        method: DeribitMethod,
        params: HashMap<String, Value>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let id = self.next_id();
        let url = self.urls.rest_url();

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.method(),
            "params": params,
        });

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Make request (all Deribit requests use POST)
        let response = self.http.post(url, &request, &headers).await?;

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // JSON-RPC HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get next request ID
    fn next_id(&self) -> u64 {
        let mut id = self.request_id.lock().expect("Mutex poisoned");
        let current = *id;
        *id += 1;
        current
    }

    /// Make JSON-RPC call
    async fn rpc_call(
        &self,
        method: DeribitMethod,
        params: HashMap<String, Value>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let id = self.next_id();
        let url = self.urls.rest_url();

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.method(),
            "params": params,
        });

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Add Authorization header for private methods
        if method.requires_auth() {
            self.ensure_authenticated().await?;

            let auth_header = {
                let auth_guard = self.auth.lock().expect("Mutex poisoned");
                auth_guard.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?
                    .auth_header()?
            };

            headers.insert("Authorization".to_string(), auth_header);
        }

        // Make request (all Deribit requests use POST)
        let response = self.http.post(url, &request, &headers).await?;

        // Check for JSON-RPC errors (handled by parser)
        Ok(response)
    }

    /// Currency from symbol for Deribit
    fn _currency_from_symbol(symbol: &Symbol) -> String {
        // For Deribit, use base currency (BTC, ETH, SOL, etc.)
        symbol.base.to_uppercase()
    }

    /// Instrument name from symbol
    fn instrument_from_symbol(symbol: &Symbol, account_type: AccountType) -> String {
        format_symbol(&symbol.base, &symbol.quote, account_type)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DeribitConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Deribit
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_level() as u32, lim.max_level() as u32)
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
            AccountType::Spot,          // Limited spot trading
            AccountType::FuturesCross,  // Inverse and linear perpetuals/futures
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
impl MarketData for DeribitConnector {
    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));

        let response = self.rpc_call(DeribitMethod::Ticker, params).await?;
        DeribitParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));
        if let Some(d) = depth {
            params.insert("depth".to_string(), json!(d));
        }

        let response = self.rpc_call(DeribitMethod::GetOrderBook, params).await?;
        DeribitParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let (resolution, interval_ms): (&str, u64) = match interval {
            "1m"  => ("1",   60_000),
            "3m"  => ("3",   180_000),
            "5m"  => ("5",   300_000),
            "15m" => ("15",  900_000),
            "30m" => ("30",  1_800_000),
            "1h"  => ("60",  3_600_000),
            "2h"  => ("120", 7_200_000),
            "4h"  => ("240", 14_400_000),
            "6h"  => ("360", 21_600_000),
            "12h" => ("720", 43_200_000),
            "1d" | "1D" => ("1D", 86_400_000),
            other => return Err(ExchangeError::Parse(format!("Unsupported interval: {}", other))),
        };

        let count = limit.unwrap_or(2000).min(10000) as u64;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let end_ms = end_time.map(|t| t as u64).unwrap_or(now_ms);
        let start_ms = end_ms.saturating_sub(count * interval_ms);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));
        params.insert("start_timestamp".to_string(), json!(start_ms));
        params.insert("end_timestamp".to_string(), json!(end_ms));
        params.insert("resolution".to_string(), json!(resolution));

        let response = self.rpc_call(DeribitMethod::GetTradingviewChartData, params).await?;
        DeribitParser::parse_klines(&response, interval_ms)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));

        let response = self.rpc_call(DeribitMethod::Ticker, params).await?;
        DeribitParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use test method for ping
        let params = HashMap::new();
        let _response = self.rpc_call(DeribitMethod::Test, params).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Fetch instruments for major currencies: BTC, ETH, SOL, USDC
        let currencies = ["BTC", "ETH", "SOL", "USDC"];
        let mut all_symbols = Vec::new();

        for currency in &currencies {
            let mut params = HashMap::new();
            params.insert("currency".to_string(), json!(currency));
            params.insert("expired".to_string(), json!(false));

            match self.rpc_call(DeribitMethod::GetInstruments, params).await {
                Ok(response) => {
                    match DeribitParser::parse_exchange_info(&response) {
                        Ok(mut symbols) => all_symbols.append(&mut symbols),
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(all_symbols)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for DeribitConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let instrument_name = Self::instrument_from_symbol(&symbol, account_type);
                
                        let method = match side {
                            OrderSide::Buy => DeribitMethod::Buy,
                            OrderSide::Sell => DeribitMethod::Sell,
                        };
                
                        let mut params = HashMap::new();
                        params.insert("instrument_name".to_string(), json!(instrument_name));
                        params.insert("amount".to_string(), json!(quantity));
                        params.insert("type".to_string(), json!("market"));
                
                        let response = self.rpc_call(method, params).await?;
                        DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let instrument_name = Self::instrument_from_symbol(&symbol, account_type);
                
                        let method = match side {
                            OrderSide::Buy => DeribitMethod::Buy,
                            OrderSide::Sell => DeribitMethod::Sell,
                        };
                
                        let mut params = HashMap::new();
                        params.insert("instrument_name".to_string(), json!(instrument_name));
                        params.insert("amount".to_string(), json!(quantity));
                        params.insert("type".to_string(), json!("limit"));
                        params.insert("price".to_string(), json!(price));
                
                        let response = self.rpc_call(method, params).await?;
                        DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
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

            let _ = (symbol, account_type); // Not needed for Deribit
            let mut params = HashMap::new();
            params.insert("order_id".to_string(), json!(order_id));

            let response = self.rpc_call(DeribitMethod::Cancel, params).await?;
            DeribitParser::parse_order(&response, "")
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let _ = (symbol, account_type); // Not needed for Deribit
        let mut params = HashMap::new();
        params.insert("order_id".to_string(), json!(order_id));

        let response = self.rpc_call(DeribitMethod::GetOrderState, params).await?;
        DeribitParser::parse_order(&response, "")
    
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

        if let Some(sym) = symbol {
            let instrument_name = Self::instrument_from_symbol(&sym, account_type);
            params.insert("instrument_name".to_string(), json!(instrument_name));
            let response = self.rpc_call(DeribitMethod::GetOpenOrdersByInstrument, params).await?;
            DeribitParser::parse_orders(&response)
        } else {
            // Get all open orders (no specific instrument)
            let response = self.rpc_call(DeribitMethod::GetOpenOrders, params).await?;
            DeribitParser::parse_orders(&response)
        }
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for DeribitConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let _account_type = query.account_type;
        // Determine currency
        let currency = asset.map(|a| a.to_uppercase()).unwrap_or_else(|| "BTC".to_string());

        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(currency));
        params.insert("extended".to_string(), json!(false));

        let response = self.rpc_call(DeribitMethod::GetAccountSummary, params).await?;
        DeribitParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Get account summary for BTC (main currency on Deribit)
        let balances = self.get_balance(BalanceQuery { asset: Some("BTC".to_string()), account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // Deribit has dynamic fees
            taker_commission: 0.0,
            balances,
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
impl Positions for DeribitConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Deribit uses dynamic leverage".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Deribit uses dynamic leverage".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Deribit uses dynamic leverage".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_from_symbol() {
        let symbol = Symbol::new("BTC", "PERPETUAL");
        assert_eq!(DeribitConnector::_currency_from_symbol(&symbol), "BTC");

        let symbol = Symbol::new("eth", "usd");
        assert_eq!(DeribitConnector::_currency_from_symbol(&symbol), "ETH");
    }

    #[test]
    fn test_instrument_from_symbol() {
        let symbol = Symbol::new("BTC", "USD");
        let instrument = DeribitConnector::instrument_from_symbol(&symbol, AccountType::FuturesCross);
        assert_eq!(instrument, "BTC-PERPETUAL");

        let symbol = Symbol::new("SOL", "USDC");
        let instrument = DeribitConnector::instrument_from_symbol(&symbol, AccountType::FuturesCross);
        assert_eq!(instrument, "SOL_USDC-PERPETUAL");
    }
}
