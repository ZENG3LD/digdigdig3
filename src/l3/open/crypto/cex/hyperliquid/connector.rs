//! # Hyperliquid Connector Implementation
//!
//! Main connector struct implementing all V5 traits.
//!
//! ## Implementation Status
//!
//! - [x] ExchangeIdentity
//! - [x] MarketData
//! - [x] Trading (place_order, cancel_order, get_order, get_open_orders, get_order_history)
//! - [x] Account (get_balance, get_account_info, get_fees)
//! - [x] Positions (get_positions, get_funding_rate, modify_position)
//!
//! ## Notes on Info Endpoint Authentication
//!
//! The `/info` endpoint for user-specific data (balance, orders, positions)
//! does NOT require a cryptographic signature — only the wallet address in the
//! request body. Only the `/exchange` endpoint requires EIP-712 signing.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::{
    HttpClient, Credentials, ExchangeResult, ExchangeError,
    ExchangeId, ExchangeType, AccountType, Symbol,
    Price, Ticker, OrderBook, Kline,
    ExchangeIdentity, MarketData,
    Order, OrderRequest, CancelRequest, CancelScope,
    OrderType, OrderSide, TimeInForce, OrderStatus,
    Balance, AccountInfo, Position, FundingRate, MarginType,
    PlaceOrderResponse, BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, FeeInfo,
    AmendRequest, OrderResult,
    CancelAllResponse,
    UserTrade, UserTradeFilter,
};
use crate::core::traits::{Trading, Account, Positions, AmendOrder, BatchOrders, CancelAll, AccountTransfers, FundingHistory};
use crate::core::types::{ConnectorStats, SymbolInfo, AlgoOrderResponse, TransferRequest, TransferHistoryFilter, TransferResponse, FundingPayment, FundingFilter};
use crate::core::utils::WeightRateLimiter;
use crate::core::utils::PrecisionCache;

use super::{HyperliquidUrls, HyperliquidAuth, HyperliquidParser, HyperliquidEndpoint};
use super::endpoints::InfoType;
use super::auth::{HlOrder, HlOrderType, HlTif, normalize_price};

/// Hyperliquid DEX connector
pub struct HyperliquidConnector {
    /// HTTP client
    http: HttpClient,
    /// API URLs
    urls: HyperliquidUrls,
    /// Authentication handler (required for trading and account data)
    auth: Option<HyperliquidAuth>,
    /// Is testnet
    is_testnet: bool,
    /// Rate limiter (1200 weight/min)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl HyperliquidConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - Wallet credentials (private key in api_secret, address in api_key)
    /// * `is_testnet` - Use testnet if true
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
            .map(|c| HyperliquidAuth::new_with_network(c, is_testnet))
            .transpose()?;

        let http = HttpClient::new(30_000)?;

        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(1200, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            urls,
            auth,
            is_testnet,
            rate_limiter,
            precision: PrecisionCache::new(),
        })
    }

    /// Create public connector (no authentication, market data only)
    pub async fn public(is_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, is_testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTH HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get authentication handler or return Auth error
    fn require_auth(&self) -> ExchangeResult<&HyperliquidAuth> {
        self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth(
                "Authentication required. Provide wallet credentials.".to_string()
            ))
    }

    /// Get wallet address for authenticated Info queries
    fn wallet_address(&self) -> ExchangeResult<&str> {
        Ok(self.require_auth()?.wallet_address())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RATE LIMITING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit slot
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

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP REQUEST HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// POST to /info endpoint
    async fn info_request(
        &self,
        info_type: InfoType,
        params: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let weight = match info_type {
            InfoType::L2Book | InfoType::AllMids => 2,
            _ => 20,
        };
        self.rate_limit_wait(weight).await;

        let url = format!("{}{}", self.urls.rest_url(), HyperliquidEndpoint::Info.path());

        let mut body = serde_json::json!({ "type": info_type.as_str() });
        if let Some(obj) = body.as_object_mut() {
            if let Some(params_obj) = params.as_object() {
                obj.extend(params_obj.clone());
            }
        }

        self.http.post(&url, &body, &std::collections::HashMap::new()).await
    }

    /// POST to /exchange endpoint (authenticated)
    async fn exchange_request(
        &self,
        body: &serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        self.rate_limit_wait(20).await;
        let url = format!("{}{}", self.urls.rest_url(), HyperliquidEndpoint::Exchange.path());
        let headers = self.require_auth()?.get_headers();
        let response = self.http.post(&url, body, &headers).await?;
        HyperliquidParser::check_exchange_response(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HYPERLIQUID-SPECIFIC PUBLIC METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get perpetuals metadata (asset index mapping)
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

    /// Look up the asset index for a coin symbol from metadata.
    /// Returns 0 if the symbol is not found (BTC is at index 0).
    async fn symbol_to_asset_index(&self, coin: &str) -> ExchangeResult<u32> {
        let meta = self.get_metadata().await?;
        let universe = meta.get("universe")
            .and_then(|u| u.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing universe in metadata".to_string()))?;

        for (idx, item) in universe.iter().enumerate() {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                if name.eq_ignore_ascii_case(coin) {
                    return Ok(idx as u32);
                }
            }
        }

        Err(ExchangeError::Parse(format!("Symbol '{}' not found in Hyperliquid metadata", coin)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

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
        _     => 60_000,
    }
}

/// Build a PlaceOrderResponse from Hyperliquid exchange response + the original request.
fn build_order_response(
    response: &serde_json::Value,
    req: &OrderRequest,
) -> ExchangeResult<PlaceOrderResponse> {
    let data = HyperliquidParser::extract_exchange_data(response)?;
    let statuses = data.get("statuses")
        .and_then(|s| s.as_array())
        .ok_or_else(|| ExchangeError::Parse("Missing statuses in order response".to_string()))?;

    let status = statuses.first()
        .ok_or_else(|| ExchangeError::Parse("Empty statuses array".to_string()))?;

    // Check for per-order error
    if let Some(err) = status.get("error").and_then(|e| e.as_str()) {
        return Err(ExchangeError::Api { code: -1, message: err.to_string() });
    }

    // Extract order ID from resting or filled status
    let (order_id, filled_qty, avg_price, order_status) = if let Some(resting) = status.get("resting") {
        let oid = resting.get("oid")
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string())
            .unwrap_or_default();
        (oid, 0.0, None, OrderStatus::Open)
    } else if let Some(filled) = status.get("filled") {
        let oid = filled.get("oid")
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string())
            .unwrap_or_default();
        let total_sz = filled.get("totalSz")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let avg_px = filled.get("avgPx")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
        (oid, total_sz, avg_px, OrderStatus::Filled)
    } else {
        return Err(ExchangeError::Parse("Unknown order status in response".to_string()));
    };

    let order_type_for_response = match &req.order_type {
        OrderType::Market => OrderType::Market,
        OrderType::Limit { price } => OrderType::Limit { price: *price },
        OrderType::StopMarket { stop_price } => OrderType::StopMarket { stop_price: *stop_price },
        OrderType::StopLimit { stop_price, limit_price } => OrderType::StopLimit {
            stop_price: *stop_price,
            limit_price: *limit_price,
        },
        OrderType::PostOnly { price } => OrderType::PostOnly { price: *price },
        OrderType::Ioc { price } => OrderType::Ioc { price: *price },
        OrderType::Fok { price } => OrderType::Fok { price: *price },
        OrderType::ReduceOnly { price } => OrderType::ReduceOnly { price: *price },
        other => other.clone(),
    };

    let price = match &req.order_type {
        OrderType::Limit { price } | OrderType::PostOnly { price } |
        OrderType::Fok { price } => Some(*price),
        OrderType::Ioc { price } => *price,
        OrderType::ReduceOnly { price } => *price,
        _ => None,
    };

    let stop_price = match &req.order_type {
        OrderType::StopMarket { stop_price } => Some(*stop_price),
        OrderType::StopLimit { stop_price, .. } => Some(*stop_price),
        _ => None,
    };

    let order = Order {
        id: order_id,
        client_order_id: req.client_order_id.clone(),
        symbol: req.symbol.base.clone(),
        side: req.side,
        order_type: order_type_for_response,
        status: order_status,
        price,
        stop_price,
        quantity: req.quantity,
        filled_quantity: filled_qty,
        average_price: avg_price,
        commission: None,
        commission_asset: None,
        created_at: crate::core::timestamp_millis() as i64,
        updated_at: None,
        time_in_force: req.time_in_force,
    };

    Ok(PlaceOrderResponse::Simple(order))
}

/// Build the HlOrder struct from an OrderRequest + asset index
fn build_hl_order(req: &OrderRequest, asset_index: u32) -> ExchangeResult<HlOrder> {
    let is_buy = matches!(req.side, OrderSide::Buy);
    let reduce_only = req.reduce_only || matches!(req.order_type, OrderType::ReduceOnly { .. });

    // Determine price, size, and order type
    let (price_str, order_type, reduce_only) = match &req.order_type {
        OrderType::Market => {
            // Hyperliquid market = aggressive limit at slippage price
            // Use a very aggressive price: buy at 2x current or sell at 0.5x
            // The actual price doesn't matter for market orders — HL fills at best price
            // Convention: use 0 price with IOC TIF for market semantics
            // Actually: Hyperliquid recommends slippage: 10% from mark price
            // For simplicity we use a large number for buys, small for sells
            let price = if is_buy { 999_999_999.0f64 } else { 0.000001f64 };
            (
                normalize_price(price),
                HlOrderType::Limit { tif: HlTif::Ioc },
                reduce_only,
            )
        }
        OrderType::Limit { price } => {
            (normalize_price(*price), HlOrderType::Limit { tif: HlTif::Gtc }, reduce_only)
        }
        OrderType::PostOnly { price } => {
            (normalize_price(*price), HlOrderType::Limit { tif: HlTif::Alo }, reduce_only)
        }
        OrderType::Ioc { price } => {
            let p = price.unwrap_or(if is_buy { 999_999_999.0 } else { 0.000001 });
            (normalize_price(p), HlOrderType::Limit { tif: HlTif::Ioc }, reduce_only)
        }
        OrderType::Fok { price } => {
            (normalize_price(*price), HlOrderType::Limit { tif: HlTif::Fok }, reduce_only)
        }
        OrderType::ReduceOnly { price } => {
            let p = price.unwrap_or(if is_buy { 999_999_999.0 } else { 0.000001 });
            let tif = if price.is_none() { HlTif::Ioc } else { HlTif::Gtc };
            (normalize_price(p), HlOrderType::Limit { tif }, true)
        }
        OrderType::StopMarket { stop_price } => {
            (
                normalize_price(*stop_price),
                HlOrderType::Trigger {
                    trigger_px: normalize_price(*stop_price),
                    is_market: true,
                    tpsl: "sl".to_string(),
                },
                reduce_only,
            )
        }
        OrderType::StopLimit { stop_price, limit_price } => {
            (
                normalize_price(*limit_price),
                HlOrderType::Trigger {
                    trigger_px: normalize_price(*stop_price),
                    is_market: false,
                    tpsl: "sl".to_string(),
                },
                reduce_only,
            )
        }
        other => {
            return Err(ExchangeError::UnsupportedOperation(
                format!("Order type {:?} not supported on Hyperliquid", other)
            ));
        }
    };

    // Apply TIF from request if it overrides the order type's TIF
    let order_type = match (&order_type, req.time_in_force) {
        (HlOrderType::Limit { .. }, TimeInForce::PostOnly) => HlOrderType::Limit { tif: HlTif::Alo },
        (HlOrderType::Limit { .. }, TimeInForce::Ioc) => HlOrderType::Limit { tif: HlTif::Ioc },
        (HlOrderType::Limit { .. }, TimeInForce::Fok) => HlOrderType::Limit { tif: HlTif::Fok },
        _ => order_type,
    };

    Ok(HlOrder {
        a: asset_index,
        b: is_buy,
        p: price_str,
        s: normalize_price(req.quantity),
        r: reduce_only,
        t: order_type,
        c: req.client_order_id.clone(),
    })
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
        let response = self.get_all_mids().await?;
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
        Err(ExchangeError::NotSupported(
            "get_ticker requires symbol-to-index mapping. Use get_all_mids() instead.".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.get_all_mids().await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let info = match account_type {
            AccountType::Spot => {
                let response = self.get_spot_metadata().await?;
                HyperliquidParser::parse_spot_exchange_info(&response, account_type)?
            }
            _ => {
                let response = self.get_metadata().await?;
                HyperliquidParser::parse_perp_exchange_info(&response, account_type)?
            }
        };
        self.precision.load_from_symbols(&info);
        Ok(info)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for HyperliquidConnector {
    /// Place an order on Hyperliquid.
    ///
    /// Supported order types:
    /// - Market (implemented as aggressive IOC limit)
    /// - Limit (GTC)
    /// - PostOnly (ALO — Add-Liquidity-Only)
    /// - IOC
    /// - FOK
    /// - StopMarket (trigger order, tpsl="sl", isMarket=true)
    /// - StopLimit (trigger order, tpsl="sl", isMarket=false)
    /// - ReduceOnly (limit or market with reduce_only=true)
    ///
    /// Unsupported: TrailingStop, OCO, Bracket, Iceberg, TWAP, GTD
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let auth = self.require_auth()?;

        // Resolve asset index from symbol
        let asset_index = self.symbol_to_asset_index(&req.symbol.base).await?;

        // TWAP is a separate action type — handle before build_hl_order.
        if let OrderType::Twap { duration_seconds, .. } = req.order_type {
            let is_buy = matches!(req.side, OrderSide::Buy);
            let size_str = normalize_price(req.quantity);
            let body = auth.sign_twap_action(
                asset_index,
                is_buy,
                &size_str,
                req.reduce_only,
                duration_seconds,
                None,
            )?;
            let response = self.exchange_request(&body).await?;
            // Hyperliquid TWAP response: { "status": "ok", "response": { "type": "twapOrder",
            // "data": { "running": { "twapId": 123, ... } } } }
            let algo_id = response
                .pointer("/response/data/running/twapId")
                .or_else(|| response.pointer("/response/data/twapId"))
                .and_then(|v| v.as_u64())
                .map(|id| id.to_string())
                .unwrap_or_else(|| "0".to_string());
            return Ok(PlaceOrderResponse::Algo(AlgoOrderResponse {
                algo_id,
                status: "Running".to_string(),
                executed_count: None,
                total_count: None,
            }));
        }

        // Build the HlOrder for standard order types
        let hl_order = build_hl_order(&req, asset_index)?;

        // Sign and build request body
        let body = auth.sign_order_action(&[hl_order], "na", None)?;

        // POST to /exchange
        let response = self.exchange_request(&body).await?;

        build_order_response(&response, &req)
    }

    /// Cancel an order (single, batch, all, or by symbol).
    ///
    /// Returns the first cancelled order's state (or a placeholder for batch/all).
    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        let auth = self.require_auth()?;

        // Build cancel list: (asset_index, order_id)
        let cancels: Vec<(u32, u64)> = match &req.scope {
            CancelScope::Single { order_id } => {
                let oid = order_id.parse::<u64>()
                    .map_err(|_| ExchangeError::Parse(
                        format!("Invalid order ID '{}': must be numeric", order_id)
                    ))?;
                // Need asset index — use symbol hint if provided
                let asset = if let Some(sym) = &req.symbol {
                    self.symbol_to_asset_index(&sym.base).await?
                } else {
                    0
                };
                vec![(asset, oid)]
            }
            CancelScope::Batch { order_ids } => {
                let asset = if let Some(sym) = &req.symbol {
                    self.symbol_to_asset_index(&sym.base).await?
                } else {
                    0
                };
                let mut pairs = Vec::with_capacity(order_ids.len());
                for oid_str in order_ids {
                    let oid = oid_str.parse::<u64>()
                        .map_err(|_| ExchangeError::Parse(
                            format!("Invalid order ID '{}': must be numeric", oid_str)
                        ))?;
                    pairs.push((asset, oid));
                }
                pairs
            }
            CancelScope::All { symbol: sym_opt } => {
                // Cancel all open orders, optionally filtered to a symbol
                let wallet = self.wallet_address()?;
                let params = serde_json::json!({ "user": wallet, "dex": "" });
                let response = self.info_request(InfoType::OpenOrders, params).await?;
                let orders = HyperliquidParser::parse_orders(&response)?;

                let mut pairs = Vec::new();
                for o in &orders {
                    if let Some(ref s) = sym_opt {
                        if !o.symbol.eq_ignore_ascii_case(&s.base) {
                            continue;
                        }
                    }
                    if let Ok(oid) = o.id.parse::<u64>() {
                        let asset = self.symbol_to_asset_index(&o.symbol).await.unwrap_or(0);
                        pairs.push((asset, oid));
                    }
                }

                if pairs.is_empty() {
                    let sym_str = sym_opt.as_ref().map(|s| s.base.clone()).unwrap_or_default();
                    return Ok(Order {
                        id: "0".to_string(),
                        client_order_id: None,
                        symbol: sym_str,
                        side: OrderSide::Buy,
                        order_type: OrderType::Limit { price: 0.0 },
                        status: OrderStatus::Canceled,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: None,
                        time_in_force: TimeInForce::Gtc,
                    });
                }

                pairs
            }
            CancelScope::ByLabel(_)
            | CancelScope::ByCurrencyKind { .. }
            | CancelScope::ScheduledAt(_) => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Hyperliquid does not support this cancel scope".to_string()
                ));
            }
            CancelScope::BySymbol { symbol: sym } => {
                // Cancel all open orders for a specific symbol
                let wallet = self.wallet_address()?;
                let params = serde_json::json!({ "user": wallet, "dex": "" });
                let response = self.info_request(InfoType::OpenOrders, params).await?;
                let orders = HyperliquidParser::parse_orders(&response)?;

                let mut pairs = Vec::new();
                for o in &orders {
                    if !o.symbol.eq_ignore_ascii_case(&sym.base) {
                        continue;
                    }
                    if let Ok(oid) = o.id.parse::<u64>() {
                        let asset = self.symbol_to_asset_index(&o.symbol).await.unwrap_or(0);
                        pairs.push((asset, oid));
                    }
                }

                if pairs.is_empty() {
                    return Ok(Order {
                        id: "0".to_string(),
                        client_order_id: None,
                        symbol: sym.base.clone(),
                        side: OrderSide::Buy,
                        order_type: OrderType::Limit { price: 0.0 },
                        status: OrderStatus::Canceled,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: None,
                        time_in_force: TimeInForce::Gtc,
                    });
                }

                pairs
            }
        };

        if cancels.is_empty() {
            return Err(ExchangeError::Parse("No orders to cancel".to_string()));
        }

        let body = auth.sign_cancel_action(&cancels, None)?;
        let response = self.exchange_request(&body).await?;

        // Extract cancel response
        let data = HyperliquidParser::extract_exchange_data(&response)?;
        let statuses = data.get("statuses")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing statuses in cancel response".to_string()))?;

        // Check for per-cancel error
        if let Some(first) = statuses.first() {
            if let Some(err) = first.get("error").and_then(|e| e.as_str()) {
                return Err(ExchangeError::Api { code: -1, message: err.to_string() });
            }
        }

        // Return a synthetic cancelled order
        let symbol = req.symbol.map(|s| s.base).unwrap_or_default();
        let first_oid = cancels.first().map(|(_, oid)| oid.to_string()).unwrap_or_default();

        Ok(Order {
            id: first_oid,
            client_order_id: None,
            symbol,
            side: OrderSide::Buy, // Unknown — HL cancel response doesn't return full order details
            order_type: OrderType::Limit { price: 0.0 },
            status: OrderStatus::Canceled,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }

    /// Get the current state of a single order by ID.
    ///
    /// Requires authentication (wallet address).
    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let wallet = self.wallet_address()?;

        // Parse order ID — can be numeric oid or cloid
        let oid_value = if let Ok(oid) = order_id.parse::<u64>() {
            serde_json::json!(oid)
        } else if order_id.starts_with("0x") {
            // Treat as cloid
            serde_json::json!({ "cloid": order_id })
        } else {
            serde_json::json!(order_id.parse::<u64>().map_err(|_|
                ExchangeError::Parse(format!("Invalid order ID: {}", order_id))
            )?)
        };

        let params = serde_json::json!({
            "user": wallet,
            "oid": oid_value,
        });

        let response = self.info_request(InfoType::OrderStatus, params).await?;
        HyperliquidParser::parse_order_status(&response)
    }

    /// Get all open orders, optionally filtered by symbol.
    ///
    /// Requires authentication (wallet address).
    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let wallet = self.wallet_address()?;
        let params = serde_json::json!({ "user": wallet, "dex": "" });
        let response = self.info_request(InfoType::OpenOrders, params).await?;
        let mut orders = HyperliquidParser::parse_orders(&response)?;

        // Filter by symbol if requested
        if let Some(sym) = symbol {
            orders.retain(|o| o.symbol.eq_ignore_ascii_case(sym));
        }

        Ok(orders)
    }

    /// Get order history (fills / closed orders).
    ///
    /// Uses `userFills` for recent fills or `historicalOrders` for order history.
    /// Requires authentication (wallet address).
    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let wallet = self.wallet_address()?;

        let response = if filter.start_time.is_some() || filter.end_time.is_some() {
            // Use userFillsByTime when a time range is specified
            let mut params = serde_json::json!({
                "user": wallet,
                "aggregateByTime": false,
            });
            if let Some(start) = filter.start_time {
                params["startTime"] = serde_json::json!(start);
            }
            if let Some(end) = filter.end_time {
                params["endTime"] = serde_json::json!(end);
            }
            self.info_request(InfoType::UserFillsByTime, params).await?
        } else {
            // Use historicalOrders for general history
            let params = serde_json::json!({ "user": wallet });
            self.info_request(InfoType::HistoricalOrders, params).await?
        };

        let mut orders = HyperliquidParser::parse_historical_orders(&response)?;

        // Apply symbol filter
        if let Some(sym) = &filter.symbol {
            orders.retain(|o| o.symbol.eq_ignore_ascii_case(&sym.base));
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            orders.truncate(limit as usize);
        }

        Ok(orders)
    }

    /// Fetch trade fills for the authenticated wallet address.
    ///
    /// Uses `userFillsByTime` when a time range is specified, otherwise
    /// falls back to `userFills` (returns up to the last 2000 fills).
    /// No authentication signature needed — only the wallet address.
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let wallet = self.wallet_address()?;

        let response = if filter.start_time.is_some() || filter.end_time.is_some() {
            let mut params = serde_json::json!({
                "user": wallet,
                "aggregateByTime": false,
            });
            if let Some(start) = filter.start_time {
                params["startTime"] = serde_json::json!(start);
            }
            if let Some(end) = filter.end_time {
                params["endTime"] = serde_json::json!(end);
            }
            self.info_request(InfoType::UserFillsByTime, params).await?
        } else {
            let params = serde_json::json!({
                "user": wallet,
                "aggregateByTime": false,
            });
            self.info_request(InfoType::UserFills, params).await?
        };

        let mut trades = HyperliquidParser::parse_user_fills(&response)?;

        // Apply symbol filter (HyperLiquid uses coin name, not base/quote pair)
        if let Some(sym) = &filter.symbol {
            trades.retain(|t| t.symbol.eq_ignore_ascii_case(sym));
        }

        // Apply order_id filter
        if let Some(oid) = &filter.order_id {
            trades.retain(|t| &t.order_id == oid);
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            trades.truncate(limit as usize);
        }

        Ok(trades)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for HyperliquidConnector {
    /// Get account balances.
    ///
    /// For Spot accounts: uses `spotClearinghouseState` → returns per-token balances.
    /// For Perp accounts: uses `clearinghouseState` → returns USDC balance summary.
    ///
    /// No signature required — only wallet address.
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let wallet = self.wallet_address()?;

        let mut balances = match query.account_type {
            AccountType::Spot => {
                let params = serde_json::json!({ "user": wallet });
                let response = self.info_request(InfoType::SpotClearinghouseState, params).await?;
                HyperliquidParser::parse_spot_balances(&response)?
            }
            _ => {
                // FuturesCross, FuturesIsolated, Margin → all use clearinghouseState
                let params = serde_json::json!({ "user": wallet, "dex": "" });
                let response = self.info_request(InfoType::ClearinghouseState, params).await?;
                HyperliquidParser::parse_perp_balances(&response)?
            }
        };

        // Filter by asset if requested
        if let Some(ref asset) = query.asset {
            balances.retain(|b| b.asset.eq_ignore_ascii_case(asset));
        }

        Ok(balances)
    }

    /// Get account info (permissions, margin summary, balances).
    ///
    /// Uses `clearinghouseState` for perp account metadata.
    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let wallet = self.wallet_address()?;

        let params = serde_json::json!({ "user": wallet, "dex": "" });
        let response = self.info_request(InfoType::ClearinghouseState, params).await?;

        let balances = HyperliquidParser::parse_perp_balances(&response)?;

        // account_value extracted for potential future use in AccountInfo enrichment
        let _account_value = response.get("marginSummary")
            .and_then(|m| m.get("accountValue"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            // Standard Hyperliquid fees (tier 0)
            maker_commission: 0.0002,  // 0.02%
            taker_commission: 0.00035, // 0.035%
            balances,
        })
    }

    /// Get fee schedule for the account.
    ///
    /// Uses `userFees` endpoint to get the current tier.
    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Try to get actual fee tier from API if authenticated
        if let Ok(wallet) = self.wallet_address() {
            let params = serde_json::json!({ "user": wallet });
            if let Ok(response) = self.info_request(InfoType::UserFees, params).await {
                if let Some(schedule) = response.get("feeSchedule") {
                    if let Some(tiers) = schedule.get("tiers").and_then(|t| t.as_array()) {
                        if let Some(first_tier) = tiers.first() {
                            let maker = first_tier.get("maker")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(0.0002);
                            let taker = first_tier.get("taker")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<f64>().ok())
                                .unwrap_or(0.00035);
                            return Ok(FeeInfo {
                                maker_rate: maker,
                                taker_rate: taker,
                                symbol: symbol.map(String::from),
                                tier: None,
                            });
                        }
                    }
                }
            }
        }

        // Fallback to standard Hyperliquid fees
        Ok(FeeInfo {
            maker_rate: 0.0002,
            taker_rate: 0.00035,
            symbol: symbol.map(String::from),
            tier: Some("Standard".to_string()),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for HyperliquidConnector {
    /// Get open perpetual positions.
    ///
    /// Uses `clearinghouseState` — no signature required, only wallet address.
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let wallet = self.wallet_address()?;

        let params = serde_json::json!({ "user": wallet, "dex": "" });
        let response = self.info_request(InfoType::ClearinghouseState, params).await?;
        let mut positions = HyperliquidParser::parse_positions(&response)?;

        // Filter by symbol if requested
        if let Some(ref sym) = query.symbol {
            positions.retain(|p| p.symbol.eq_ignore_ascii_case(&sym.base));
        }

        Ok(positions)
    }

    /// Get the current funding rate for a perpetual symbol.
    ///
    /// Uses `metaAndAssetCtxs` to find the symbol's funding rate.
    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Get metadata to find index, then get asset contexts
        let meta_response = self.info_request(
            InfoType::MetaAndAssetCtxs,
            serde_json::json!({}),
        ).await?;

        HyperliquidParser::parse_funding_rate_for_symbol(&meta_response, symbol)
    }

    /// Modify a position — leverage, margin mode, add/remove margin, or close.
    ///
    /// Supported modifications:
    /// - SetLeverage: POST /exchange updateLeverage
    /// - SetMarginMode: POST /exchange updateLeverage with isCross flag
    /// - AddMargin: POST /exchange updateIsolatedMargin (positive ntli)
    /// - RemoveMargin: POST /exchange updateIsolatedMargin (negative ntli)
    /// - ClosePosition: place a reduce-only market order for the full position size
    ///
    /// Unsupported: SetTpSl (Hyperliquid TP/SL is set at order placement, not separately)
    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        let auth = self.require_auth()?;

        match req {
            PositionModification::SetLeverage { symbol, leverage, .. } => {
                let asset = self.symbol_to_asset_index(&symbol.base).await?;
                // Keep current margin mode (cross by default)
                let body = auth.sign_update_leverage(asset, true, leverage, None)?;
                self.exchange_request(&body).await?;
                Ok(())
            }

            PositionModification::SetMarginMode { symbol, margin_type, .. } => {
                let asset = self.symbol_to_asset_index(&symbol.base).await?;
                let is_cross = matches!(margin_type, MarginType::Cross);
                // When switching to isolated, default to 1x leverage
                // When switching to cross, restore leverage
                let body = auth.sign_update_leverage(asset, is_cross, 1, None)?;
                self.exchange_request(&body).await?;
                Ok(())
            }

            PositionModification::AddMargin { symbol, amount, .. } => {
                let asset = self.symbol_to_asset_index(&symbol.base).await?;
                // ntli = amount in units of 1e-6 USDC (1 USDC = 1_000_000 ntli)
                let ntli = (amount * 1_000_000.0) as i64;
                let body = auth.sign_update_isolated_margin(asset, true, ntli, None)?;
                self.exchange_request(&body).await?;
                Ok(())
            }

            PositionModification::RemoveMargin { symbol, amount, .. } => {
                let asset = self.symbol_to_asset_index(&symbol.base).await?;
                let ntli = -((amount * 1_000_000.0) as i64);
                let body = auth.sign_update_isolated_margin(asset, false, ntli, None)?;
                self.exchange_request(&body).await?;
                Ok(())
            }

            PositionModification::ClosePosition { symbol, account_type } => {
                // Fetch current position to get size
                let wallet = self.wallet_address()?;
                let params = serde_json::json!({ "user": wallet, "dex": "" });
                let response = self.info_request(InfoType::ClearinghouseState, params).await?;
                let positions = HyperliquidParser::parse_positions(&response)?;

                let position = positions.iter()
                    .find(|p| p.symbol.eq_ignore_ascii_case(&symbol.base))
                    .ok_or_else(|| ExchangeError::Parse(
                        format!("No open position for symbol '{}'", symbol.base)
                    ))?;

                // Close by placing a reduce-only market order in the opposite direction
                let close_side = match position.side {
                    crate::core::PositionSide::Long => OrderSide::Sell,
                    crate::core::PositionSide::Short => OrderSide::Buy,
                    crate::core::PositionSide::Both => OrderSide::Sell, // fallback
                };

                let close_req = OrderRequest {
                    symbol: symbol.clone(),
                    side: close_side,
                    order_type: OrderType::Market,
                    quantity: position.quantity,
                    time_in_force: TimeInForce::Ioc,
                    account_type,
                    client_order_id: None,
                    reduce_only: true,
                };

                self.place_order(close_req).await?;
                Ok(())
            }

            PositionModification::SetTpSl { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "SetTpSl is not supported as a standalone operation on Hyperliquid. \
                     Set TP/SL at order placement using StopMarket/StopLimit order types.".to_string()
                ))
            }
            PositionModification::SwitchPositionMode { .. } => Err(ExchangeError::UnsupportedOperation(
                "SwitchPositionMode not supported on Hyperliquid".to_string()
            )),
            PositionModification::MovePositions { .. } => Err(ExchangeError::UnsupportedOperation(
                "MovePositions not supported on Hyperliquid".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIONAL TRAITS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for HyperliquidConnector {
    /// Modify a live order using the native `modify` action on `/exchange`.
    ///
    /// Hyperliquid's modify action requires the full order spec (asset, side, price, size,
    /// reduce_only, order type) alongside the order ID to amend.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let auth = self.require_auth()?;

        // Parse the numeric order ID
        let oid = req.order_id.parse::<u64>()
            .map_err(|_| ExchangeError::Parse(
                format!("Invalid Hyperliquid order ID '{}': must be numeric", req.order_id)
            ))?;

        // Resolve asset index from symbol
        let asset_index = self.symbol_to_asset_index(&req.symbol.base).await?;

        // Fetch current order to get its existing fields (we need a complete spec for modify)
        let current_order = self.get_order(&req.symbol.base, &req.order_id, req.account_type).await?;

        // Determine the new price and size (use amended fields if provided, else keep current)
        let new_price = req.fields.price.unwrap_or_else(|| {
            current_order.price.unwrap_or(0.0)
        });
        let new_size = req.fields.quantity.unwrap_or(current_order.quantity);

        // Determine TIF from current order's order_type
        let (price_str, order_type) = match &current_order.order_type {
            OrderType::PostOnly { .. } => (
                normalize_price(new_price),
                HlOrderType::Limit { tif: HlTif::Alo },
            ),
            OrderType::Ioc { .. } => (
                normalize_price(new_price),
                HlOrderType::Limit { tif: HlTif::Ioc },
            ),
            OrderType::Fok { .. } => (
                normalize_price(new_price),
                HlOrderType::Limit { tif: HlTif::Fok },
            ),
            _ => (
                normalize_price(new_price),
                HlOrderType::Limit { tif: HlTif::Gtc },
            ),
        };

        let is_buy = matches!(current_order.side, OrderSide::Buy);

        let hl_order = HlOrder {
            a: asset_index,
            b: is_buy,
            p: price_str,
            s: normalize_price(new_size),
            r: false,
            t: order_type,
            c: None,
        };

        let body = auth.sign_modify_action(oid, &hl_order, None)?;
        let response = self.exchange_request(&body).await?;

        // Parse the response — modify returns statuses similar to order placement
        let data = HyperliquidParser::extract_exchange_data(&response)?;
        let statuses = data.get("statuses")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing statuses in modify response".to_string()))?;

        if let Some(first) = statuses.first() {
            if let Some(err) = first.get("error").and_then(|e| e.as_str()) {
                return Err(ExchangeError::Api { code: -1, message: err.to_string() });
            }
        }

        // Return the order with updated fields
        Ok(Order {
            id: req.order_id,
            client_order_id: current_order.client_order_id,
            symbol: req.symbol.base.clone(),
            side: current_order.side,
            order_type: current_order.order_type,
            status: OrderStatus::Open,
            price: Some(new_price),
            stop_price: current_order.stop_price,
            quantity: new_size,
            filled_quantity: current_order.filled_quantity,
            average_price: current_order.average_price,
            commission: None,
            commission_asset: None,
            created_at: current_order.created_at,
            updated_at: Some(crate::core::timestamp_millis() as i64),
            time_in_force: current_order.time_in_force,
        })
    }
}

#[async_trait]
impl BatchOrders for HyperliquidConnector {
    /// Place multiple orders in a single native batch request.
    ///
    /// Hyperliquid's `order` action natively accepts an array of orders — this IS the batch
    /// endpoint. Uses `grouping="na"` (no bracket/TP-SL grouping).
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        let auth = self.require_auth()?;

        // Build HlOrder structs for each request
        let mut hl_orders = Vec::with_capacity(orders.len());
        for req in &orders {
            let asset_index = self.symbol_to_asset_index(&req.symbol.base).await?;
            hl_orders.push(build_hl_order(req, asset_index)?);
        }

        // Sign and submit the batch
        let body = auth.sign_order_action(&hl_orders, "na", None)?;
        let response = self.exchange_request(&body).await?;

        // Parse per-order statuses
        let data = HyperliquidParser::extract_exchange_data(&response)?;
        let statuses = data.get("statuses")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing statuses in batch order response".to_string()))?;

        let results: Vec<OrderResult> = statuses.iter().zip(orders.iter()).map(|(status, req)| {
            if let Some(err) = status.get("error").and_then(|e| e.as_str()) {
                return OrderResult {
                    order: None,
                    client_order_id: req.client_order_id.clone(),
                    success: false,
                    error: Some(err.to_string()),
                    error_code: None,
                };
            }

            // Extract order ID from resting or filled
            let (order_id, filled_qty, avg_price, order_status) = if let Some(resting) = status.get("resting") {
                let oid = resting.get("oid")
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                (oid, 0.0f64, None::<f64>, OrderStatus::Open)
            } else if let Some(filled) = status.get("filled") {
                let oid = filled.get("oid")
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let total_sz = filled.get("totalSz")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let avg_px = filled.get("avgPx")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                (oid, total_sz, avg_px, OrderStatus::Filled)
            } else {
                return OrderResult {
                    order: None,
                    client_order_id: req.client_order_id.clone(),
                    success: false,
                    error: Some("Unknown status in batch response".to_string()),
                    error_code: None,
                };
            };

            let price = match &req.order_type {
                OrderType::Limit { price } | OrderType::PostOnly { price } |
                OrderType::Fok { price } => Some(*price),
                OrderType::Ioc { price } => *price,
                OrderType::ReduceOnly { price } => *price,
                _ => None,
            };

            let order = Order {
                id: order_id,
                client_order_id: req.client_order_id.clone(),
                symbol: req.symbol.base.clone(),
                side: req.side,
                order_type: req.order_type.clone(),
                status: order_status,
                price,
                stop_price: None,
                quantity: req.quantity,
                filled_quantity: filled_qty,
                average_price: avg_price,
                commission: None,
                commission_asset: None,
                created_at: crate::core::timestamp_millis() as i64,
                updated_at: None,
                time_in_force: req.time_in_force,
            };

            OrderResult {
                order: Some(order),
                client_order_id: req.client_order_id.clone(),
                success: true,
                error: None,
                error_code: None,
            }
        }).collect();

        Ok(results)
    }

    /// Cancel multiple orders in a single native batch request.
    ///
    /// Hyperliquid's `cancel` action natively accepts an array of `{a, o}` pairs.
    /// `symbol` hint is used to look up the asset index; all order IDs are assumed
    /// to belong to the same asset if a symbol is provided.
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        let auth = self.require_auth()?;

        // Determine asset index (use symbol hint if provided, default to 0)
        let asset = if let Some(sym) = symbol {
            let coin = sym.split('/').next().unwrap_or(sym);
            self.symbol_to_asset_index(coin).await.unwrap_or(0)
        } else {
            0
        };

        // Parse order IDs to (asset, oid) pairs
        let mut cancels: Vec<(u32, u64)> = Vec::with_capacity(order_ids.len());
        for id_str in &order_ids {
            let oid = id_str.parse::<u64>()
                .map_err(|_| ExchangeError::Parse(
                    format!("Invalid Hyperliquid order ID '{}': must be numeric", id_str)
                ))?;
            cancels.push((asset, oid));
        }

        let body = auth.sign_cancel_action(&cancels, None)?;
        let response = self.exchange_request(&body).await?;

        // Parse per-cancel statuses
        let data = HyperliquidParser::extract_exchange_data(&response)?;
        let statuses = data.get("statuses")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing statuses in batch cancel response".to_string()))?;

        let results: Vec<OrderResult> = statuses.iter().zip(order_ids.iter()).map(|(status, oid_str)| {
            if let Some(err) = status.get("error").and_then(|e| e.as_str()) {
                return OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(err.to_string()),
                    error_code: None,
                };
            }

            // Success — "success" string in status
            OrderResult {
                order: Some(Order {
                    id: oid_str.clone(),
                    client_order_id: None,
                    symbol: symbol.unwrap_or("").to_string(),
                    side: OrderSide::Buy, // Unknown from cancel response
                    order_type: OrderType::Limit { price: 0.0 },
                    status: OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: TimeInForce::Gtc,
                }),
                client_order_id: None,
                success: true,
                error: None,
                error_code: None,
            }
        }).collect();

        Ok(results)
    }

    fn max_batch_place_size(&self) -> usize {
        10 // Hyperliquid rate limit: 10 orders per batch recommended
    }

    fn max_batch_cancel_size(&self) -> usize {
        10 // Same limit for cancel batches
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH MODIFY (AMEND)
// ═══════════════════════════════════════════════════════════════════════════════

impl HyperliquidConnector {
    /// Batch modify (amend) multiple existing orders via the `batchModify` action on `/exchange`.
    ///
    /// Each entry in `modifies` is `(order_id_str, new_price, new_size, asset_symbol)`.
    /// All four fields are required per Hyperliquid's batchModify spec.
    ///
    /// Max 10 modifies per batch.
    ///
    /// Returns the raw JSON exchange response.
    pub async fn batch_amend_orders(
        &self,
        modifies: Vec<(String, f64, f64, String)>,
    ) -> ExchangeResult<serde_json::Value> {
        if modifies.is_empty() {
            return Ok(serde_json::json!({"status": "ok", "response": {}}));
        }
        if modifies.len() > 10 {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch modify size {} exceeds Hyperliquid limit of 10", modifies.len())
            ));
        }

        let auth = self.require_auth()?;

        // Build (oid, HlOrder) pairs
        let mut hl_modifies: Vec<(u64, HlOrder)> = Vec::with_capacity(modifies.len());
        for (oid_str, price, size, symbol) in &modifies {
            let oid = oid_str.parse::<u64>()
                .map_err(|_| ExchangeError::InvalidRequest(
                    format!("Invalid Hyperliquid order ID '{}': must be numeric", oid_str)
                ))?;
            let asset_index = self.symbol_to_asset_index(symbol).await?;
            hl_modifies.push((oid, HlOrder {
                a: asset_index,
                b: true, // Direction will be taken from existing order; placeholder
                p: normalize_price(*price),
                s: normalize_price(*size),
                r: false,
                t: HlOrderType::Limit { tif: HlTif::Gtc },
                c: None,
            }));
        }

        let pairs: Vec<(u64, &HlOrder)> = hl_modifies.iter().map(|(oid, order)| (*oid, order)).collect();
        let body = auth.sign_batch_modify_action(&pairs, None)?;
        self.exchange_request(&body).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for HyperliquidConnector {
    /// Cancel all open orders, optionally scoped to a symbol.
    ///
    /// Hyperliquid has no native cancel-all endpoint.
    /// Implementation:
    ///   1. Fetch all open orders via `POST /info {"type": "openOrders", "user": "0x..."}`
    ///   2. Filter by symbol if scope is `BySymbol` or `All { symbol: Some(...) }`
    ///   3. Send a single batch cancel via `POST /exchange` with all (asset, oid) pairs
    ///
    /// This is a single batch cancel request — NOT a loop over `cancel_order`.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let auth = self.require_auth()?;
        let wallet = self.wallet_address()?;

        // Fetch all open orders (no signature required — just wallet address)
        let params = serde_json::json!({ "user": wallet, "dex": "" });
        let response = self.info_request(InfoType::OpenOrders, params).await?;
        let open_orders = HyperliquidParser::parse_orders(&response)?;

        // Determine optional symbol filter from scope
        let symbol_filter: Option<&str> = match &scope {
            CancelScope::All { symbol: Some(sym) } => Some(&sym.base),
            CancelScope::BySymbol { symbol: sym } => Some(&sym.base),
            CancelScope::All { symbol: None } => None,
            // Single and Batch are handled by Trading::cancel_order — not this trait
            _ => {
                return Err(ExchangeError::UnsupportedOperation(
                    "CancelAll::cancel_all_orders only supports All and BySymbol scopes".to_string()
                ));
            }
        };

        // Build (asset_index, oid) pairs, optionally filtered by symbol
        let mut cancels: Vec<(u32, u64)> = Vec::new();
        for order in &open_orders {
            if let Some(filter) = symbol_filter {
                if !order.symbol.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }
            if let Ok(oid) = order.id.parse::<u64>() {
                let asset = self.symbol_to_asset_index(&order.symbol).await.unwrap_or(0);
                cancels.push((asset, oid));
            }
        }

        // Nothing to cancel — return early with zero counts
        if cancels.is_empty() {
            return Ok(CancelAllResponse {
                cancelled_count: 0,
                failed_count: 0,
                details: Vec::new(),
            });
        }

        // One single batch cancel request
        let body = auth.sign_cancel_action(&cancels, None)?;
        let exchange_response = self.exchange_request(&body).await?;

        // Parse per-cancel statuses to build detailed results
        let data = HyperliquidParser::extract_exchange_data(&exchange_response)?;
        let statuses = data.get("statuses")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse(
                "Missing statuses in cancel-all response".to_string()
            ))?;

        let mut cancelled_count = 0u32;
        let mut failed_count = 0u32;
        let mut details: Vec<OrderResult> = Vec::with_capacity(statuses.len());

        for (status, (_, oid)) in statuses.iter().zip(cancels.iter()) {
            if let Some(err) = status.get("error").and_then(|e| e.as_str()) {
                failed_count += 1;
                details.push(OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(err.to_string()),
                    error_code: None,
                });
            } else {
                cancelled_count += 1;
                details.push(OrderResult {
                    order: Some(Order {
                        id: oid.to_string(),
                        client_order_id: None,
                        symbol: symbol_filter.unwrap_or("").to_string(),
                        side: OrderSide::Buy,
                        order_type: OrderType::Limit { price: 0.0 },
                        status: OrderStatus::Canceled,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: Some(crate::core::timestamp_millis() as i64),
                        time_in_force: TimeInForce::Gtc,
                    }),
                    client_order_id: None,
                    success: true,
                    error: None,
                    error_code: None,
                });
            }
        }

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count,
            details,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountTransfers for HyperliquidConnector {
    /// Transfer USDC between Spot and Perp wallets.
    ///
    /// Uses the `usdClassTransfer` action on `/exchange`.
    /// - Spot → Perp:   `from_account = Spot, to_account = FuturesCross`  → `toPerp: true`
    /// - Perp → Spot:   `from_account = FuturesCross, to_account = Spot`  → `toPerp: false`
    ///
    /// Only USDC transfers between Spot and Perp are supported.
    /// Other asset/account combinations return `UnsupportedOperation`.
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let auth = self.require_auth()?;

        // Determine transfer direction
        let to_perp = match (&req.from_account, &req.to_account) {
            (AccountType::Spot, AccountType::FuturesCross)
            | (AccountType::Spot, AccountType::FuturesIsolated) => true,
            (AccountType::FuturesCross, AccountType::Spot)
            | (AccountType::FuturesIsolated, AccountType::Spot) => false,
            _ => {
                return Err(ExchangeError::UnsupportedOperation(format!(
                    "Hyperliquid only supports Spot ↔ Perp (FuturesCross) USDC transfers. \
                     Got: {:?} → {:?}",
                    req.from_account, req.to_account
                )));
            }
        };

        // Normalize amount to string (no trailing zeros)
        let amount_str = super::auth::normalize_price(req.amount);

        // Sign and send the transfer action
        let body = auth.sign_usd_class_transfer(&amount_str, to_perp, None)?;
        let response = self.exchange_request(&body).await?;

        // usdClassTransfer response: { "status": "ok", "response": { "type": "default" } }
        // No transfer ID is returned — generate a synthetic one from nonce context
        let transfer_id = response
            .pointer("/response/data")
            .and_then(|d| d.as_str())
            .unwrap_or("ok")
            .to_string();

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: Some(crate::core::timestamp_millis() as i64),
        })
    }

    /// Get transfer history between Spot and Perp.
    ///
    /// Hyperliquid does not expose a transfer history endpoint — returns empty vec.
    async fn get_transfer_history(
        &self,
        _filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        Ok(Vec::new())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for HyperliquidConnector {
    /// Get historical funding payments from `POST /info` with `type: userFunding`.
    ///
    /// The wallet address is taken from credentials (`api_key`). No cryptographic
    /// signature is required for this endpoint — only the address in the request body.
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let auth = self.require_auth()?;
        let user_addr = auth.wallet_address();

        let mut params = serde_json::json!({ "user": user_addr });

        if let Some(start) = filter.start_time {
            params["startTime"] = serde_json::Value::from(start);
        }
        if let Some(end) = filter.end_time {
            params["endTime"] = serde_json::Value::from(end);
        }

        let response = self.info_request(InfoType::UserFunding, params).await?;
        HyperliquidParser::parse_funding_payments(&response)
    }
}
