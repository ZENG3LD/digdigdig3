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



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════


