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
        }
    }
}
