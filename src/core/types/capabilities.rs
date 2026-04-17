//! # Connector Capabilities
//!
//! Fine-grained capability descriptors for market data, trading, and account operations.
//! These supplement `Features` with per-operation granularity.

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Describes which market data endpoints a connector supports.
#[derive(Debug, Clone, Copy)]
pub struct MarketDataCapabilities {
    /// Supports ping/server-time endpoint
    pub has_ping: bool,
    /// Supports current price endpoint
    pub has_price: bool,
    /// Supports ticker (24h stats) endpoint
    pub has_ticker: bool,
    /// Supports orderbook snapshot endpoint
    pub has_orderbook: bool,
    /// Supports historical kline/candlestick endpoint
    pub has_klines: bool,
    /// Supports exchange info / symbol metadata endpoint
    pub has_exchange_info: bool,
    /// Supports recent public trades endpoint
    pub has_recent_trades: bool,
    /// Supports WebSocket kline/candlestick stream
    pub has_ws_klines: bool,
    /// Supports WebSocket trade stream
    pub has_ws_trades: bool,
    /// Supports WebSocket orderbook stream
    pub has_ws_orderbook: bool,
    /// Supports WebSocket ticker stream
    pub has_ws_ticker: bool,
    /// Supported kline intervals (e.g. &["1m", "5m", "15m", "1h", "4h", "1d"])
    pub supported_intervals: &'static [&'static str],
    /// Maximum klines per single request. None = unknown/unlimited.
    pub max_kline_limit: Option<u16>,
}

impl MarketDataCapabilities {
    /// Full CEX market data (all endpoints, standard intervals, 1000-bar limit).
    pub const fn full_cex() -> Self {
        Self {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            has_recent_trades: true,
            has_ws_klines: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_ticker: true,
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d",
                "1w", "1M",
            ],
            max_kline_limit: Some(1000),
        }
    }

    /// Data provider without recent trades, 500-bar limit.
    pub const fn data_only() -> Self {
        Self {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            has_recent_trades: false,
            has_ws_klines: false,
            has_ws_trades: false,
            has_ws_orderbook: false,
            has_ws_ticker: false,
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d",
                "1w", "1M",
            ],
            max_kline_limit: Some(500),
        }
    }

    /// Minimal capabilities: ping, price and daily klines only.
    pub const fn minimal() -> Self {
        Self {
            has_ping: true,
            has_price: true,
            has_ticker: false,
            has_orderbook: false,
            has_klines: true,
            has_exchange_info: false,
            has_recent_trades: false,
            has_ws_klines: false,
            has_ws_trades: false,
            has_ws_orderbook: false,
            has_ws_ticker: false,
            supported_intervals: &["1d"],
            max_kline_limit: Some(100),
        }
    }

    /// No market data support.
    pub const fn none() -> Self {
        Self {
            has_ping: false,
            has_price: false,
            has_ticker: false,
            has_orderbook: false,
            has_klines: false,
            has_exchange_info: false,
            has_recent_trades: false,
            has_ws_klines: false,
            has_ws_trades: false,
            has_ws_orderbook: false,
            has_ws_ticker: false,
            supported_intervals: &[],
            max_kline_limit: None,
        }
    }

    /// All-true placeholder for connectors that have not yet filled in real caps.
    pub const fn permissive() -> Self {
        Self {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            has_recent_trades: true,
            has_ws_klines: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_ticker: true,
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d",
                "1w", "1M",
            ],
            max_kline_limit: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Describes which order types and trading operations a connector supports.
#[derive(Debug, Clone, Copy)]
pub struct TradingCapabilities {
    /// Supports market orders
    pub has_market_order: bool,
    /// Supports limit orders
    pub has_limit_order: bool,
    /// Supports stop-market (stop-loss market) orders
    pub has_stop_market: bool,
    /// Supports stop-limit orders
    pub has_stop_limit: bool,
    /// Supports trailing-stop orders
    pub has_trailing_stop: bool,
    /// Supports bracket (take-profit + stop-loss combo) orders
    pub has_bracket: bool,
    /// Supports OCO (one-cancels-the-other) orders
    pub has_oco: bool,
    /// Supports amending (modifying) an existing open order
    pub has_amend: bool,
    /// Supports batch order placement/cancellation
    pub has_batch: bool,
    /// Maximum orders per batch request. None = no batch support or unlimited.
    pub max_batch_size: Option<u16>,
    /// Supports cancel-all-open-orders endpoint
    pub has_cancel_all: bool,
    /// Supports fetching user (account) trade history
    pub has_user_trades: bool,
    /// Supports fetching order history (closed/cancelled orders)
    pub has_order_history: bool,
}

impl TradingCapabilities {
    /// Standard full-featured CEX trading (no bracket/oco/trailing, batch of 20).
    pub const fn full_cex() -> Self {
        Self {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,
            has_stop_limit: true,
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            has_amend: true,
            has_batch: true,
            max_batch_size: Some(20),
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
    }

    /// Basic trading: market + limit + cancel-all + history only.
    pub const fn basic() -> Self {
        Self {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: false,
            has_stop_limit: false,
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            has_amend: false,
            has_batch: false,
            max_batch_size: None,
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
    }

    /// No trading support.
    pub const fn none() -> Self {
        Self {
            has_market_order: false,
            has_limit_order: false,
            has_stop_market: false,
            has_stop_limit: false,
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            has_amend: false,
            has_batch: false,
            max_batch_size: None,
            has_cancel_all: false,
            has_user_trades: false,
            has_order_history: false,
        }
    }

    /// All-true placeholder for connectors that have not yet filled in real caps.
    pub const fn permissive() -> Self {
        Self {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,
            has_stop_limit: true,
            has_trailing_stop: true,
            has_bracket: true,
            has_oco: true,
            has_amend: true,
            has_batch: true,
            max_batch_size: None,
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// ORDERBOOK CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Algorithm used to compute the orderbook integrity checksum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// CRC-32 over interleaved top-N bid+ask price:qty strings (OKX/Bitget format).
    Crc32Interleaved,
    /// CRC-32 over asks_string + bids_string with decimal-stripped numeric strings (Kraken format).
    Crc32KrakenFormat,
    /// CRC-32, exact algorithm TBD (used by Crypto.com `cs` field).
    Crc32Generic,
    /// CRC-32 over interleaved top-25 bid+ask with order IDs (Bitfinex R0).
    Crc32BitfinexRaw,
}

/// Describes the checksum coverage and algorithm for a WS orderbook channel.
#[derive(Debug, Clone, Copy)]
pub struct ChecksumInfo {
    /// Algorithm used.
    pub algorithm: ChecksumAlgorithm,
    /// Number of levels per side covered by the checksum (e.g. 10 for Kraken, 25 for OKX/Bitget).
    pub levels_per_side: u32,
    /// Whether the checksum is opt-in (must be enabled via flags, e.g. Bitfinex OB_CHECKSUM).
    pub opt_in: bool,
}

/// Describes one named WebSocket orderbook channel variant.
///
/// Some exchanges expose multiple named channels with distinct depth/speed/update-model
/// characteristics (OKX books vs books5; KuCoin level2 vs level2Depth5; HTX mbp vs depth).
/// Each variant is described here. The ws_manager picks the best-fit channel at subscription
/// time using `ws_channels` instead of raw `ws_depths` / `update_speeds_ms`.
///
/// All fields are `Copy`-safe and use `'static` lifetimes for zero-alloc use.
#[derive(Debug, Clone, Copy)]
pub struct WsBookChannel {
    /// Exchange-specific channel or topic name (e.g. "books5", "mbp.150", "level2Depth50").
    pub name: &'static str,
    /// Fixed depth of this channel. `None` = full book / not constrained to a fixed count.
    pub depth: Option<u32>,
    /// True if this channel delivers full snapshots on every push.
    /// False = delta/incremental (initial snapshot then deltas).
    pub is_snapshot: bool,
    /// Fixed update speed in milliseconds. `None` = event-driven / real-time.
    pub update_speed_ms: Option<u32>,
    /// True if this channel requires elevated account tier / VIP / API key.
    pub requires_auth_tier: bool,
}

impl WsBookChannel {
    pub const fn snapshot(name: &'static str, depth: u32, speed_ms: u32) -> Self {
        Self {
            name,
            depth: Some(depth),
            is_snapshot: true,
            update_speed_ms: Some(speed_ms),
            requires_auth_tier: false,
        }
    }

    pub const fn delta(name: &'static str, depth: Option<u32>, speed_ms: Option<u32>) -> Self {
        Self {
            name,
            depth,
            is_snapshot: false,
            update_speed_ms: speed_ms,
            requires_auth_tier: false,
        }
    }

    pub const fn with_auth_tier(mut self) -> Self {
        self.requires_auth_tier = true;
        self
    }
}

/// Declares what L2/orderbook configurations an exchange supports on WebSocket.
///
/// ## Design notes
/// - All fields use `&'static` slices or `Copy` primitives — zero-allocation, `const`-friendly.
/// - `ws_channels` is the primary field for multi-channel exchanges (OKX, HTX, KuCoin, etc.).
///   When `ws_channels` is non-empty, `ws_depths` and `update_speeds_ms` are best-effort summaries.
/// - `rest_depth_values` overrides `rest_max_depth` when an exchange requires discrete values.
///   An empty `rest_depth_values` with `rest_max_depth = Some(N)` means "any integer up to N".
/// - `checksum` is `None` for exchanges without checksums.
/// - `has_sequence` / `has_prev_sequence` describe gap-detection capability.
///   `has_prev_sequence = true` implies `has_sequence = true`.
#[derive(Debug, Clone, Copy)]
pub struct OrderbookCapabilities {
    // ── Existing fields (preserved, semantics unchanged) ─────────────────────

    /// Valid depth levels for WS orderbook subscription.
    /// Empty = exchange doesn't accept depth parameter (it decides internally).
    pub ws_depths: &'static [u32],
    /// Recommended default depth for WS subscription. None = omit depth.
    pub ws_default_depth: Option<u32>,
    /// Maximum depth available via REST get_orderbook. None = unknown/unlimited.
    pub rest_max_depth: Option<u32>,
    /// Whether the exchange supports full orderbook snapshots on WS.
    pub supports_snapshot: bool,
    /// Whether the exchange supports incremental/delta updates on WS.
    pub supports_delta: bool,
    /// Valid update speed values in milliseconds. Empty = not configurable.
    pub update_speeds_ms: &'static [u32],
    /// Recommended default update speed. None = exchange default.
    pub default_speed_ms: Option<u32>,

    // ── New: named channel variants ──────────────────────────────────────────

    /// Named WS channel variants with distinct depth/speed/model properties.
    /// Empty slice = exchange has a single implicit channel (use ws_depths / update_speeds_ms).
    /// Non-empty = use `WsBookChannel` records for channel selection logic.
    pub ws_channels: &'static [WsBookChannel],

    // ── New: REST depth precision ─────────────────────────────────────────────

    /// Discrete valid values for REST `limit` / `depth` parameter.
    /// Empty = any integer up to `rest_max_depth` is accepted.
    /// Non-empty = ONLY these values are valid (e.g. Binance Futures: 5/10/20/50/100/500/1000).
    pub rest_depth_values: &'static [u32],

    // ── New: checksum ─────────────────────────────────────────────────────────

    /// Checksum info for the primary (or only) channel. None = no checksum.
    pub checksum: Option<ChecksumInfo>,

    // ── New: sequence / gap-detection ────────────────────────────────────────

    /// True = WS messages carry a monotonic sequence/update-ID field.
    pub has_sequence: bool,
    /// True = WS messages carry a PREVIOUS sequence field enabling in-message gap detection.
    /// (e.g. Binance Futures `pu`, OKX `prevSeqId`, Deribit `prev_change_id`).
    pub has_prev_sequence: bool,

    // ── New: price aggregation ────────────────────────────────────────────────

    /// True = exchange supports price-level aggregation/grouping on WS or REST.
    pub supports_aggregation: bool,
    /// Named aggregation tiers or parameter values (e.g. "step0".."step5", "P0".."R0", "none").
    /// Empty = aggregation not available or values are numeric/continuous.
    pub aggregation_levels: &'static [&'static str],
}

impl OrderbookCapabilities {
    /// Permissive default — accepts any depth, both snapshot and delta.
    /// Used as default for connectors that haven't declared capabilities yet.
    pub const fn permissive() -> Self {
        Self {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            rest_depth_values: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }

    /// Pick the best matching WsBookChannel for a requested depth and update model.
    ///
    /// Returns `None` if `ws_channels` is empty (caller should fall back to legacy fields).
    /// Auth-tier channels are always skipped.
    /// When `prefer_delta` is true, delta channels are preferred over snapshots.
    pub fn best_channel(&self, requested_depth: Option<u32>, prefer_delta: bool) -> Option<&WsBookChannel> {
        if self.ws_channels.is_empty() {
            return None;
        }
        // Filter out auth-tier channels
        let public: Vec<&WsBookChannel> = self.ws_channels.iter()
            .filter(|c| !c.requires_auth_tier)
            .collect();
        if public.is_empty() {
            return None;
        }
        // Prefer delta or snapshot channels
        let preferred: Vec<&&WsBookChannel> = public.iter()
            .filter(|c| if prefer_delta { !c.is_snapshot } else { c.is_snapshot })
            .collect();
        let candidates: Vec<&WsBookChannel> = if preferred.is_empty() {
            public
        } else {
            preferred.into_iter().copied().collect()
        };
        // Pick by closest depth: smallest depth >= requested, or largest depth
        candidates.into_iter().min_by_key(|c| {
            match (c.depth, requested_depth) {
                (Some(d), Some(r)) if d >= r => d - r,
                (Some(_), Some(_)) => u32::MAX,
                (None, _) => 0,
                (Some(_), None) => 0,
            }
        })
    }

    /// Pick the closest valid depth for a requested value.
    /// - If ws_depths is empty, returns ws_default_depth (exchange doesn't accept depth param).
    /// - If requested is None, returns ws_default_depth.
    /// - Otherwise finds the smallest valid depth >= requested, or the largest valid depth.
    pub fn clamp_depth(&self, requested: Option<u32>) -> Option<u32> {
        if self.ws_depths.is_empty() {
            return self.ws_default_depth;
        }
        let target = match requested {
            Some(d) => d,
            None => return self.ws_default_depth,
        };
        // Find smallest depth >= target
        let mut best = None;
        for &d in self.ws_depths {
            if d >= target {
                match best {
                    None => best = Some(d),
                    Some(b) if d < b => best = Some(d),
                    _ => {}
                }
            }
        }
        // If nothing >= target, use the largest available
        best.or_else(|| self.ws_depths.iter().copied().max())
    }

    /// Pick the closest valid update speed for a requested value.
    /// Same logic as clamp_depth but for speed.
    pub fn clamp_speed(&self, requested: Option<u32>) -> Option<u32> {
        if self.update_speeds_ms.is_empty() {
            return self.default_speed_ms;
        }
        let target = match requested {
            Some(s) => s,
            None => return self.default_speed_ms,
        };
        let mut best = None;
        for &s in self.update_speeds_ms {
            if s >= target {
                match best {
                    None => best = Some(s),
                    Some(b) if s < b => best = Some(s),
                    _ => {}
                }
            }
        }
        best.or_else(|| self.update_speeds_ms.iter().copied().min())
    }
}

impl Default for OrderbookCapabilities {
    fn default() -> Self {
        Self::permissive()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Describes which account management operations a connector supports.
#[derive(Debug, Clone, Copy)]
pub struct AccountCapabilities {
    /// Supports fetching account balances
    pub has_balances: bool,
    /// Supports fetching full account info (permissions, tier, etc.)
    pub has_account_info: bool,
    /// Supports fetching trading fees / fee schedule
    pub has_fees: bool,
    /// Supports internal fund transfers (spot ↔ futures, sub-account, etc.)
    pub has_transfers: bool,
    /// Supports sub-account management
    pub has_sub_accounts: bool,
    /// Supports on-chain deposit address / withdrawal requests
    pub has_deposit_withdraw: bool,
    /// Supports margin borrowing and repayment
    pub has_margin: bool,
    /// Supports earn / staking products
    pub has_earn_staking: bool,
    /// Supports funding payment history (for perp/futures)
    pub has_funding_history: bool,
    /// Supports full account ledger / transaction log
    pub has_ledger: bool,
    /// Supports instant coin-to-coin conversion (swap)
    pub has_convert: bool,
    /// Supports fetching open positions (futures/perp)
    pub has_positions: bool,
}

impl AccountCapabilities {
    /// Standard full-featured CEX account (no margin/earn/staking, no convert).
    pub const fn full_cex() -> Self {
        Self {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            has_transfers: true,
            has_sub_accounts: false,
            has_deposit_withdraw: true,
            has_margin: false,
            has_earn_staking: false,
            has_funding_history: true,
            has_ledger: true,
            has_convert: false,
            has_positions: true,
        }
    }

    /// Basic account: balances + account info + fees only.
    pub const fn basic() -> Self {
        Self {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            has_transfers: false,
            has_sub_accounts: false,
            has_deposit_withdraw: false,
            has_margin: false,
            has_earn_staking: false,
            has_funding_history: false,
            has_ledger: false,
            has_convert: false,
            has_positions: false,
        }
    }

    /// No account support.
    pub const fn none() -> Self {
        Self {
            has_balances: false,
            has_account_info: false,
            has_fees: false,
            has_transfers: false,
            has_sub_accounts: false,
            has_deposit_withdraw: false,
            has_margin: false,
            has_earn_staking: false,
            has_funding_history: false,
            has_ledger: false,
            has_convert: false,
            has_positions: false,
        }
    }

    /// All-true placeholder for connectors that have not yet filled in real caps.
    pub const fn permissive() -> Self {
        Self {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            has_transfers: true,
            has_sub_accounts: true,
            has_deposit_withdraw: true,
            has_margin: true,
            has_earn_staking: true,
            has_funding_history: true,
            has_ledger: true,
            has_convert: true,
            has_positions: true,
        }
    }
}
