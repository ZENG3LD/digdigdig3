//! # Raydium Connector Implementation
//!
//! Market data connector for Raydium DEX on Solana.
//!
//! ## Implementation Status
//!
//! - [x] ExchangeIdentity
//! - [x] MarketData (basic public data)
//! - [ ] Trading (not implemented - requires Solana wallet integration)
//! - [ ] Account (not implemented - DEX architecture)
//! - [ ] WebSocket (not available - use gRPC instead)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{
    HttpClient, ExchangeResult, ExchangeError,
    ExchangeId, ExchangeType, AccountType, Symbol,
    Price, Ticker, OrderBook, Kline,
    ExchangeIdentity, MarketData,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::{RaydiumUrls, RaydiumAuth, RaydiumParser, RaydiumEndpoint};

/// Raydium DEX connector
pub struct RaydiumConnector {
    /// HTTP client
    http: HttpClient,
    /// API URLs
    urls: RaydiumUrls,
    /// Authentication handler (no-op for Raydium)
    _auth: RaydiumAuth,
    /// Is devnet
    is_testnet: bool,
    /// Rate limiter (10 req/s conservative)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl RaydiumConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `is_testnet` - Use devnet (true) or mainnet (false)
    ///
    /// Note: Credentials not needed - Raydium APIs are public
    pub async fn new(is_testnet: bool) -> ExchangeResult<Self> {
        let urls = if is_testnet {
            RaydiumUrls::DEVNET
        } else {
            RaydiumUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout
        let auth = RaydiumAuth::new();

        // Raydium rate limit: 12 req/60s (conservative, ~1 req/5s)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(12, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            urls,
            _auth: auth,
            is_testnet,
            rate_limiter,
        })
    }

    /// Wait for rate limit if necessary
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

    /// Make GET request to endpoint
    async fn get_request(
        &self,
        endpoint: RaydiumEndpoint,
        params: &HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        self.rate_limit_wait().await;

        let url = endpoint.url(&self.urls);

        self.http.get(&url, params).await
    }

    /// Get pool data by mint pair
    async fn get_pool_by_mints(
        &self,
        mint_a: &str,
        mint_b: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("mint1".to_string(), mint_a.to_string());
        params.insert("mint2".to_string(), mint_b.to_string());

        self.get_request(RaydiumEndpoint::PoolByMint, &params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for RaydiumConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Raydium
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

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Dex
    }

    fn is_testnet(&self) -> bool {
        self.is_testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for RaydiumConnector {
    async fn get_price(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Price> {
        // For Raydium, symbol.base and symbol.quote are mint addresses
        let mut params = HashMap::new();
        params.insert("mints".to_string(), symbol.base.clone());

        let response = self.get_request(RaydiumEndpoint::MintPrice, &params).await?;
        let price_value = RaydiumParser::parse_price(&response, &symbol.base)?;

        Ok(price_value)
    }

    async fn get_ticker(&self, symbol: Symbol, _account_type: AccountType) -> ExchangeResult<Ticker> {
        let response = self.get_pool_by_mints(&symbol.base, &symbol.quote).await?;
        let data = RaydiumParser::extract_data(&response)?;

        // Get first pool from array
        let pool = data.as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No pools found for pair".to_string()))?;

        let mut ticker = RaydiumParser::parse_ticker(pool)?;
        ticker.symbol = symbol.to_string();

        Ok(ticker)
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _limit: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // Raydium is a pure AMM (Automated Market Maker) using constant product formula (x*y=k).
        // AMM pools do not have traditional orderbooks with discrete bid/ask levels.
        //
        // Historical Note: Raydium v4 originally integrated with Serum/OpenBook orderbooks,
        // but this integration has been deprecated. Modern Raydium pools are pure AMMs.
        //
        // Alternative: To simulate market depth, query pool reserves via Solana RPC
        // and calculate prices at different swap amounts using the AMM formula.
        Err(ExchangeError::UnsupportedOperation(
            "Orderbooks not supported - Raydium is a pure AMM. Use get_price() or query pool reserves.".to_string()
        ))
    }

    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::NotSupported(
            "Raydium API does not provide kline data".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Ping by fetching version endpoint
        let params = HashMap::new();
        let _response = self.get_request(RaydiumEndpoint::Version, &params).await?;
        Ok(())
    }
}
