//! # Hyperliquid Connector Implementation
//!
//! Main connector struct implementing all V5 traits.
//!
//! ## Implementation Status
//!
//! - [x] ExchangeIdentity
//! - [ ] MarketData (partial - needs full implementation)
//! - [ ] Trading (not implemented - requires EIP-712 signing)
//! - [ ] Account (not implemented - requires EIP-712 signing)
//! - [ ] Positions (not implemented - requires EIP-712 signing)

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{
    HttpClient, Credentials, ExchangeResult, ExchangeError,
    ExchangeId, ExchangeType, AccountType, Symbol,
    Price, Ticker, OrderBook, Kline,
    ExchangeIdentity, MarketData,
    Order, OrderRequest, CancelRequest,
    Balance, AccountInfo, Position, FundingRate,
    PlaceOrderResponse, BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, FeeInfo,
};
use crate::core::traits::{Trading, Account, Positions};
use crate::core::types::{ConnectorStats, SymbolInfo};
use crate::core::utils::WeightRateLimiter;

use super::{HyperliquidUrls, HyperliquidAuth, HyperliquidParser, HyperliquidEndpoint};
use super::endpoints::InfoType;

/// Hyperliquid DEX connector
pub struct HyperliquidConnector {
    /// HTTP client
    http: HttpClient,
    /// API URLs
    urls: HyperliquidUrls,
    /// Authentication handler
    _auth: Option<HyperliquidAuth>,
    /// Is testnet
    is_testnet: bool,
    /// Rate limiter (1200 weight/min, weight=20 for most endpoints, weight=2 for l2Book/allMids)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl HyperliquidConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - Wallet credentials (private key in api_secret field)
    /// * `is_testnet` - Use testnet (true) or mainnet (false)
    pub async fn new(
        credentials: Option<Credentials>,
        is_testnet: bool,
    ) -> ExchangeResult<Self> {
        let urls = if is_testnet {
            HyperliquidUrls::TESTNET
        } else {
            HyperliquidUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(HyperliquidAuth::new)
            .transpose()?;

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Hyperliquid rate limit: 1200 weight/min (weight=20 for most, weight=2 for l2Book/allMids)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(1200, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            urls,
            _auth: auth,
            is_testnet,
            rate_limiter,
        })
    }

    /// Create public connector (no authentication)
    pub async fn public(is_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, is_testnet).await
    }

    /// Wait for rate limit if necessary
    ///
    /// Weight guide: weight=2 for l2Book/allMids, weight=20 for other endpoints.
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

    /// Make POST request to /info endpoint
    async fn info_request(
        &self,
        info_type: InfoType,
        params: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        // l2Book and allMids are lightweight (weight=2); other info endpoints cost weight=20
        let weight = match info_type {
            InfoType::L2Book | InfoType::AllMids => 2,
            _ => 20,
        };
        self.rate_limit_wait(weight).await;

        let url = format!("{}{}", self.urls.rest_url(), HyperliquidEndpoint::Info.path());

        let mut body = serde_json::json!({
            "type": info_type.as_str(),
        });

        // Merge params into body
        if let Some(obj) = body.as_object_mut() {
            if let Some(params_obj) = params.as_object() {
                obj.extend(params_obj.clone());
            }
        }

        self.http.post(&url, &body, &std::collections::HashMap::new()).await
    }

    /// Get metadata (required for symbol to asset ID conversion)
    pub async fn get_metadata(&self) -> ExchangeResult<serde_json::Value> {
        self.info_request(InfoType::Meta, serde_json::json!({"dex": ""})).await
    }

    /// Get spot metadata
    pub async fn get_spot_metadata(&self) -> ExchangeResult<serde_json::Value> {
        self.info_request(InfoType::SpotMeta, serde_json::json!({})).await
    }

    /// Get all mid prices
    pub async fn get_all_mids(&self) -> ExchangeResult<serde_json::Value> {
        self.info_request(InfoType::AllMids, serde_json::json!({"dex": ""})).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert an interval string to its duration in milliseconds.
fn interval_to_ms(interval: &str) -> i64 {
    match interval {
        "1m"  => 60_000,
        "3m"  => 180_000,
        "5m"  => 300_000,
        "15m" => 900_000,
        "30m" => 1_800_000,
        "1h"  => 3_600_000,
        "2h"  => 7_200_000,
        "4h"  => 14_400_000,
        "6h"  => 21_600_000,
        "8h"  => 28_800_000,
        "12h" => 43_200_000,
        "1d" | "1D" => 86_400_000,
        "3d"  => 259_200_000,
        "1w"  => 604_800_000,
        // Fallback: treat unknown as 1 minute so the window is at least sensible
        _     => 60_000,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGEIDENTITY TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for HyperliquidConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::HyperLiquid
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_weight(), lim.max_weight())
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
        self.is_testnet
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
// MARKETDATA TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for HyperliquidConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Get all mids and extract specific symbol
        let response = self.get_all_mids().await?;
        // For Hyperliquid, symbol is just the base (e.g., "BTC")
        HyperliquidParser::parse_price(&response, &symbol.base)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let params = serde_json::json!({
            "coin": &symbol.base,
            "nSigFigs": null,
            "mantissa": null,
        });

        let response = self.info_request(InfoType::L2Book, params).await?;
        HyperliquidParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let now = crate::core::timestamp_millis() as i64;
        let end_ms = end_time.unwrap_or(now);
        let interval_ms = interval_to_ms(interval);
        let count = limit.unwrap_or(5000).min(5000) as i64;
        let start_time = end_ms - count * interval_ms;

        let params = serde_json::json!({
            "req": {
                "coin": &symbol.base,
                "interval": super::endpoints::map_kline_interval(interval),
                "startTime": start_time,
                "endTime": end_ms,
            }
        });

        let response = self.info_request(InfoType::CandleSnapshot, params).await?;
        HyperliquidParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        // Get metaAndAssetCtxs which contains ticker data
        let params = serde_json::json!({});
        let _response = self.info_request(InfoType::MetaAndAssetCtxs, params).await?;

        // TODO: Need to find the index of the symbol from metadata
        // For now, return error indicating incomplete implementation
        Err(ExchangeError::NotSupported(
            "get_ticker needs symbol to index mapping from metadata".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Hyperliquid doesn't have a dedicated ping endpoint
        // Use lightweight allMids request
        self.get_all_mids().await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        match account_type {
            AccountType::Spot => {
                let response = self.get_spot_metadata().await?;
                HyperliquidParser::parse_spot_exchange_info(&response)
            }
            _ => {
                // Perp (FuturesCross and others)
                let response = self.get_metadata().await?;
                HyperliquidParser::parse_perp_exchange_info(&response)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING TRAIT (stub — EIP-712 signing not yet implemented)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for HyperliquidConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid trading requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid trading requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid trading requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid trading requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid trading requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRAIT (stub — requires authenticated info endpoint)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for HyperliquidConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid account data requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid account data requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Hyperliquid standard fees: 0.02% maker, 0.05% taker (no fee query endpoint)
        Ok(FeeInfo {
            maker_rate: 0.0002,
            taker_rate: 0.0005,
            symbol: _symbol.map(String::from),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS TRAIT (stub — requires authenticated info endpoint)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for HyperliquidConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid position data requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Hyperliquid funding rate query not yet implemented".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::NotSupported(
            "Hyperliquid position modification requires EIP-712 wallet signing which is not yet implemented".to_string()
        ))
    }
}
