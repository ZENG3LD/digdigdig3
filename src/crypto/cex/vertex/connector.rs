//! # Vertex Protocol Connector
//!
//! ⚠️ **IMPORTANT: SERVICE PERMANENTLY SHUT DOWN** ⚠️
//!
//! Vertex Protocol was acquired by Ink Foundation (Kraken-backed L2) and
//! completely shut down on **August 14, 2025**.
//!
//! **Timeline:**
//! - July 8, 2025: Acquisition announced
//! - August 14, 2025: Complete service termination
//! - All endpoints offline (gateway.prod.vertexprotocol.com, etc.)
//!
//! **Status:** This connector is kept for reference only and will not work.
//!
//! **Alternatives:**
//! - GMX (Arbitrum perpetuals DEX)
//! - dYdX V4 (standalone L1)
//! - Hyperliquid (perpetuals L1)
//!
//! See: research/vertex/ENDPOINTS_DEEP_RESEARCH.md for full details
//!
//! ---
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - futures positions
//!
//! ## Extended Methods
//! Additional Vertex-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{VertexUrls, VertexEndpoint, format_symbol, map_kline_interval};
use super::auth::{VertexAuth, TimeInForce, to_x18};
use super::parser::VertexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Vertex Protocol connector
pub struct VertexConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<VertexAuth>,
    /// URL's (mainnet/testnet)
    urls: VertexUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (100 requests per 10 seconds)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl VertexConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            VertexUrls::TESTNET
        } else {
            VertexUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Create auth if credentials provided
        let auth = credentials.as_ref().map(|creds| {
            let chain_id = if testnet { 421613 } else { 42161 };
            let verifying_contract = if testnet {
                "0x0000000000000000000000000000000000000000".to_string() // Testnet contract
            } else {
                "0x0000000000000000000000000000000000000000".to_string() // Mainnet contract
            };

            VertexAuth::new(creds, chain_id, verifying_contract, None)
        }).transpose()?;

        // Initialize rate limiter: 100 weight per 10 seconds
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector only for public methods
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: VertexEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = self.urls.rest;
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

        let response = self.http.get(&url, &HashMap::new()).await?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: VertexEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait(1).await;

        let base_url = if endpoint == VertexEndpoint::Candlesticks
            || endpoint == VertexEndpoint::ProductSnapshots
            || endpoint == VertexEndpoint::FundingRate {
            self.urls.indexer
        } else {
            self.urls.rest
        };

        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let response = self.http.post(&url, &body, &HashMap::new()).await?;
        Ok(response)
    }

    /// Query request (POST to /query endpoint)
    async fn query(&self, query_type: &str, params: Value) -> ExchangeResult<Value> {
        let mut body = params.as_object()
            .cloned()
            .unwrap_or_else(|| serde_json::Map::new());

        body.insert("type".to_string(), json!(query_type));

        self.post(VertexEndpoint::AllProducts, json!(body)).await
    }

    /// Execute request (POST to /execute endpoint with signature)
    async fn execute(&self, tx: Value) -> ExchangeResult<Value> {
        let body = json!({
            "tx": tx
        });

        self.post(VertexEndpoint::Execute, body).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Vertex-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all products (spot + perps)
    pub async fn get_all_products(&self) -> ExchangeResult<Value> {
        self.query("all_products", json!({})).await
    }

    /// Get product ID for symbol
    pub async fn get_product_id(&self, symbol: &str) -> ExchangeResult<u32> {
        let products = self.get_all_products().await?;
        let data = VertexParser::extract_data(&products)?;

        // Search in spot_products and perp_products
        if let Some(spot) = data.get("spot_products").and_then(|v| v.as_array()) {
            for product in spot {
                if let Some(symbol_str) = product.get("symbol").and_then(|s| s.as_str()) {
                    if symbol_str == symbol {
                        return product.get("product_id")
                            .and_then(|id| id.as_u64())
                            .map(|n| n as u32)
                            .ok_or_else(|| ExchangeError::Parse("Missing product_id".to_string()));
                    }
                }
            }
        }

        if let Some(perps) = data.get("perp_products").and_then(|v| v.as_array()) {
            for product in perps {
                if let Some(symbol_str) = product.get("symbol").and_then(|s| s.as_str()) {
                    if symbol_str == symbol {
                        return product.get("product_id")
                            .and_then(|id| id.as_u64())
                            .map(|n| n as u32)
                            .ok_or_else(|| ExchangeError::Parse("Missing product_id".to_string()));
                    }
                }
            }
        }

        Err(ExchangeError::Parse(format!("Product not found: {}", symbol)))
    }

    /// Cancel all orders for a product
    pub async fn cancel_all_orders(&self, product_id: u32) -> ExchangeResult<String> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let nonce = auth.generate_nonce();
        let signature = auth.sign_cancel_products(vec![product_id], nonce).await?;

        let tx = json!({
            "cancel_product_orders": {
                "sender": auth.get_sender(),
                "productIds": [product_id],
                "nonce": nonce.to_string(),
            },
            "signature": signature,
        });

        let response = self.execute(tx).await?;
        Ok(response.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for VertexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Vertex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(limiter) = self.rate_limiter.lock() {
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for VertexConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let product_id = self.get_product_id(&formatted).await?;

        let params = json!({
            "product_id": product_id,
        });

        let response = self.query("market_price", params).await?;
        VertexParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let product_id = self.get_product_id(&formatted).await?;

        let params = json!({
            "product_id": product_id,
        });

        let response = self.query("market_liquidity", params).await?;
        VertexParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let product_id = self.get_product_id(&formatted).await?;

        let granularity = map_kline_interval(interval);

        let max_time = end_time
            .map(|t| (t / 1000) as u64)
            .unwrap_or_else(|| (crate::core::timestamp_millis() / 1000) as u64);

        let count = limit.unwrap_or(1000).min(1000);

        let body = json!({
            "candlesticks": {
                "product_id": product_id,
                "granularity": granularity,
                "max_time": max_time,
                "limit": count,
            }
        });

        let response = self.post(VertexEndpoint::Candlesticks, body).await?;
        VertexParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let product_id = self.get_product_id(&formatted).await?;

        let params = json!({
            "product_id": product_id,
        });

        let response = self.query("market_price", params).await?;
        VertexParser::parse_ticker(&response, &formatted)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.query("status", json!({})).await?;
        if response.get("status").and_then(|s| s.as_str()) == Some("success") {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for VertexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                // Vertex doesn't have true market orders - use IOC limit order at extreme price
                        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                        let product_id = self.get_product_id(&formatted).await?;
                
                        let auth = self.auth.as_ref()
                            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                
                        // Get current price to determine extreme price
                        let current_price = self.get_price(symbol.clone(), account_type).await?;
                
                        // Use extreme price to ensure immediate fill
                        let price = match side {
                            OrderSide::Buy => current_price * 1.1, // 10% above market
                            OrderSide::Sell => current_price * 0.9, // 10% below market
                        };
                
                        let price_x18 = to_x18(price);
                        let amount_x18 = to_x18(match side {
                            OrderSide::Buy => quantity,
                            OrderSide::Sell => -quantity,
                        });
                
                        let nonce = auth.generate_nonce();
                        let expiration = auth.generate_expiration(60, TimeInForce::Ioc);
                
                        let (signature, _digest) = auth.sign_order(
                            product_id,
                            &price_x18,
                            &amount_x18,
                            expiration,
                            nonce,
                        ).await?;
                
                        let tx = json!({
                            "place_order": {
                                "sender": auth.get_sender(),
                                "priceX18": price_x18,
                                "amount": amount_x18,
                                "expiration": expiration.to_string(),
                                "nonce": nonce.to_string(),
                            },
                            "signature": signature,
                        });
                
                        let response = self.execute(tx).await?;
                        let order_id = VertexParser::parse_order_id(&response)?;
                
                        Ok(Order {
                            id: order_id,
                            client_order_id: None,
                            symbol: formatted,
                            side,
                            order_type: OrderType::Market,
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
                            time_in_force: crate::core::TimeInForce::Ioc,
                        })
            }
            OrderType::Limit { price } => {
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                        let product_id = self.get_product_id(&formatted).await?;
                
                        let auth = self.auth.as_ref()
                            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                
                        let price_x18 = to_x18(price);
                        let amount_x18 = to_x18(match side {
                            OrderSide::Buy => quantity,
                            OrderSide::Sell => -quantity,
                        });
                
                        let nonce = auth.generate_nonce();
                        let expiration = auth.generate_expiration(86400, TimeInForce::Gtc); // 24h validity
                
                        let (signature, _digest) = auth.sign_order(
                            product_id,
                            &price_x18,
                            &amount_x18,
                            expiration,
                            nonce,
                        ).await?;
                
                        let tx = json!({
                            "place_order": {
                                "sender": auth.get_sender(),
                                "priceX18": price_x18,
                                "amount": amount_x18,
                                "expiration": expiration.to_string(),
                                "nonce": nonce.to_string(),
                            },
                            "signature": signature,
                        });
                
                        let response = self.execute(tx).await?;
                        let order_id = VertexParser::parse_order_id(&response)?;
                
                        Ok(Order {
                            id: order_id,
                            client_order_id: None,
                            symbol: formatted,
                            side,
                            order_type: OrderType::Limit { price: 0.0 },
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
                            time_in_force: crate::core::TimeInForce::Gtc,
                        })
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
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let nonce = auth.generate_nonce();

            // Parse product_id from order_id if needed
            // For now, use empty product_ids and just the digest
            let signature = auth.sign_cancel(vec![], vec![order_id.to_string()], nonce).await?;

            let tx = json!({
                "cancel_orders": {
                    "sender": auth.get_sender(),
                    "productIds": [],
                    "digests": [order_id],
                    "nonce": nonce.to_string(),
                },
                "signature": signature,
            });

            let response = self.execute(tx).await?;
            let _ = response;

            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: "".to_string(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit { price: 0.0 },
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
                time_in_force: crate::core::TimeInForce::Gtc,
            })
    
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

        // Get all orders and filter
        let orders = self.get_open_orders(Some(symbol), account_type).await?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::Parse("Order not found".to_string()))
    
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

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let mut params = json!({
            "sender": auth.get_sender(),
        });

        if let Some(sym) = symbol {
            let formatted = format_symbol(&sym.base, &sym.quote, account_type);
            let product_id = self.get_product_id(&formatted).await?;
            params.as_object_mut().expect("Value is a JSON object").insert("product_id".to_string(), json!(product_id));
        }

        let response = self.query("subaccount_orders", params).await?;
        VertexParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for VertexConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let params = json!({
            "sender": auth.get_sender(),
        });

        let response = self.query("subaccount_info", params).await?;
        VertexParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.02, // 2 bps default
            taker_commission: 0.04, // 4 bps default
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
impl Positions for VertexConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let params = json!({
            "sender": auth.get_sender(),
        });

        let response = self.query("subaccount_info", params).await?;
        VertexParser::parse_positions(&response)
    
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

        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let product_id = self.get_product_id(&formatted).await?;

        let body = json!({
            "funding_rate": {
                "product_id": product_id,
            }
        });

        let response = self.post(VertexEndpoint::FundingRate, body).await?;
        VertexParser::parse_funding_rate(&response, &formatted)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                // Vertex uses cross-margin by default, leverage is dynamic
                Err(ExchangeError::UnsupportedOperation(
                "Vertex uses dynamic cross-margin, leverage cannot be set manually".to_string()
                ))
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}
