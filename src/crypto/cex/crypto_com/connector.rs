//! # Crypto.com Connector
//!
//! Implementation of all core traits for Crypto.com Exchange.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Futures positions

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicI64, Ordering}};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{CryptoComUrls, CryptoComEndpoint, format_symbol, account_type_to_instrument, map_kline_interval};
use super::auth::CryptoComAuth;
use super::parser::CryptoComParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Crypto.com connector
pub struct CryptoComConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<CryptoComAuth>,
    /// URLs (mainnet/testnet)
    urls: CryptoComUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (100 requests per second)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Request ID counter
    request_id: Arc<AtomicI64>,
}

impl CryptoComConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            CryptoComUrls::TESTNET
        } else {
            CryptoComUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(CryptoComAuth::new)
            .transpose()?;

        // Initialize rate limiter: 100 requests per second
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            request_id: Arc::new(AtomicI64::new(1)),
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get next request ID
    fn next_id(&self) -> i64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

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

    /// Make API request
    async fn request(
        &self,
        endpoint: CryptoComEndpoint,
        params: Value,
    ) -> ExchangeResult<Value> {
        // Rate limiting
        self.rate_limit_wait().await;

        let method = endpoint.method();
        let base_url = self.urls.rest_url();
        let url = format!("{}/{}", base_url, method);

        let response = if endpoint.requires_auth() {
            // Private endpoints use POST with JSON body
            let id = self.next_id();
            let nonce = CryptoComAuth::generate_nonce();

            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let signature = auth.sign_request(method, id, &params, nonce);

            let mut body = json!({
                "id": id,
                "method": method,
                "nonce": nonce,
                "api_key": auth.api_key(),
                "sig": signature
            });

            // Add params if not empty
            if !params.is_null() && params.as_object().is_some_and(|o| !o.is_empty()) {
                body["params"] = params;
            }

            let headers = HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
            ]);

            self.http.post(&url, &body, &headers).await?
        } else {
            // Public endpoints use GET with query parameters
            let mut query_url = url;

            if let Some(obj) = params.as_object() {
                if !obj.is_empty() {
                    let query_string: Vec<String> = obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_str().map(|s| format!("{}={}", k, s))
                                .or_else(|| v.as_i64().map(|n| format!("{}={}", k, n)))
                                .or_else(|| v.as_u64().map(|n| format!("{}={}", k, n)))
                                .or_else(|| v.as_f64().map(|n| format!("{}={}", k, n)))
                        })
                        .collect();

                    if !query_string.is_empty() {
                        query_url = format!("{}?{}", query_url, query_string.join("&"));
                    }
                }
            }

            let headers = HashMap::new();
            self.http.get(&query_url, &headers).await?
        };

        CryptoComParser::check_response(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CryptoComConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::CryptoCom
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_count(), lim.max_requests())
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for CryptoComConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetTickers, params).await?;
        CryptoComParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let mut params = json!({
            "instrument_name": instrument_name
        });

        if let Some(d) = depth {
            params["depth"] = json!(d);
        }

        let response = self.request(CryptoComEndpoint::GetBook, params).await?;
        CryptoComParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);
        let timeframe = map_kline_interval(interval);

        let mut params = json!({
            "instrument_name": instrument_name,
            "timeframe": timeframe,
            "count": limit.unwrap_or(300).min(300)
        });

        if let Some(end_ts) = end_time {
            params["end_ts"] = json!(end_ts);
        }

        let response = self.request(CryptoComEndpoint::GetCandlestick, params).await?;
        CryptoComParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetTickers, params).await?;
        CryptoComParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.request(CryptoComEndpoint::GetInstruments, json!({})).await?;
        CryptoComParser::check_response(&response)
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.request(CryptoComEndpoint::GetInstruments, json!({})).await?;
        CryptoComParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for CryptoComConnector {
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name,
            "side": match side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            },
            "type": "MARKET",
            "quantity": quantity.to_string(),
        });

        let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
        let order_id = CryptoComParser::parse_order_id(&response)?;

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            status: crate::core::OrderStatus::New,
            price: None,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name,
            "side": match side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            },
            "type": "LIMIT",
            "quantity": quantity.to_string(),
            "price": price.to_string(),
            "time_in_force": "GOOD_TILL_CANCEL",
        });

        let response = self.request(CryptoComEndpoint::CreateOrder, params).await?;
        let order_id = CryptoComParser::parse_order_id(&response)?;

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
            status: crate::core::OrderStatus::New,
            price: Some(price),
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    async fn cancel_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name,
            "order_id": order_id,
        });

        let response = self.request(CryptoComEndpoint::CancelOrder, params).await?;
        CryptoComParser::check_response(&response)?;

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side: OrderSide::Buy, // Unknown
            order_type: OrderType::Limit,
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
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let params = json!({
            "order_id": order_id,
        });

        let response = self.request(CryptoComEndpoint::GetOrderDetail, params).await?;
        CryptoComParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = json!({});

        if let Some(s) = symbol {
            let instrument_type = account_type_to_instrument(account_type);
            let instrument_name = format_symbol(&s.base, &s.quote, instrument_type);
            params["instrument_name"] = json!(instrument_name);
        }

        let response = self.request(CryptoComEndpoint::GetOpenOrders, params).await?;
        CryptoComParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for CryptoComConnector {
    async fn get_balance(
        &self,
        _asset: Option<crate::core::Asset>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>> {
        let params = json!({});
        let response = self.request(CryptoComEndpoint::UserBalance, params).await?;
        CryptoComParser::parse_balances(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(None, account_type).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.075, // Default Crypto.com fee
            taker_commission: 0.075,
            balances,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for CryptoComConnector {
    async fn get_positions(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot".to_string()
                ));
            }
            _ => {}
        }

        let mut params = json!({});

        if let Some(s) = symbol {
            let instrument_type = account_type_to_instrument(account_type);
            let instrument_name = format_symbol(&s.base, &s.quote, instrument_type);
            params["instrument_name"] = json!(instrument_name);
        }

        let response = self.request(CryptoComEndpoint::GetPositions, params).await?;
        CryptoComParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot".to_string()
                ));
            }
            _ => {}
        }

        let instrument_type = account_type_to_instrument(account_type);
        let instrument_name = format_symbol(&symbol.base, &symbol.quote, instrument_type);

        let params = json!({
            "instrument_name": instrument_name
        });

        let response = self.request(CryptoComEndpoint::GetValuations, params).await?;
        CryptoComParser::parse_funding_rate(&response)
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        let params = json!({
            "leverage": leverage.to_string()
        });

        let response = self.request(CryptoComEndpoint::ChangeAccountLeverage, params).await?;
        CryptoComParser::check_response(&response)
    }
}
