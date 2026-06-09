//! StreamKind — typed enumeration of all known WebSocket stream kinds.

use serde::{Deserialize, Serialize};

/// Typed kline interval.  Inner string is the exchange-canonical form after formatting
/// (e.g. "1m", "1h", "1D").  Equality is by the inner str.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KlineInterval(pub String);

impl KlineInterval {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Full enumeration of all known WebSocket stream kinds across all supported exchanges.
///
/// Variants with `{ interval: KlineInterval }` carry a parameter.
/// All other variants are unit variants (no parameters).
///
/// Partitioned into groups for documentation; the enum itself is flat.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamKind {
    // ── Market (price / ticker) ───────────────────────────────────────────────
    /// Full 24-h rolling ticker (OHLCV + last price + volume)
    Ticker,
    /// Index price feed (spot → perpetual fair value)
    IndexPrice,
    /// Mark price feed (settlement-reference price)
    MarkPrice,
    /// Composite index price (constructed from multiple underlying sources)
    CompositeIndex,

    // ── OrderBook ────────────────────────────────────────────────────────────
    /// Level-2 full-depth snapshot
    Orderbook,
    /// Level-2 incremental delta stream
    OrderbookDelta,
    /// Level-3 per-order (L3) full orderbook
    OrderbookL3,

    // ── Trade ────────────────────────────────────────────────────────────────
    /// Individual public trades (time-and-sales)
    Trade,
    /// Aggregated trades (multiple fills at same price collapsed)
    AggTrade,
    /// Block trade / RFQ event (large off-book transactions)
    BlockTrade,

    // ── Kline (all carry KlineInterval parameter) ─────────────────────────
    /// Standard OHLCV candlestick
    Kline { interval: KlineInterval },
    /// Mark-price candlestick (futures)
    MarkPriceKline { interval: KlineInterval },
    /// Index-price candlestick (futures)
    IndexPriceKline { interval: KlineInterval },
    /// Premium-index candlestick (futures; basis ≈ mark−index)
    PremiumIndexKline { interval: KlineInterval },

    // ── Funding ──────────────────────────────────────────────────────────────
    /// Live funding rate (updates intraperiod)
    FundingRate,
    /// Predicted funding rate before settlement window opens
    PredictedFunding,
    /// Actual funding settlement event (rate + charged amount)
    FundingSettlement,

    // ── Risk / Open Interest / Sentiment ─────────────────────────────────
    /// Open interest snapshot / update
    OpenInterest,
    /// Long/short ratio (market sentiment)
    LongShortRatio,
    /// Insurance fund balance update
    InsuranceFund,
    /// Risk limit tier update (margin tiers)
    RiskLimit,
    /// Basis stream (futures price − spot price)
    Basis,
    /// Forced-liquidation event (public)
    Liquidation,

    // ── Options-specific ─────────────────────────────────────────────────
    /// Option greeks: delta/gamma/vega/theta/rho + IV
    OptionGreeks,
    /// Volatility index (e.g. DVOL on Deribit)
    VolatilityIndex,
    /// Historical realized volatility feed
    HistoricalVolatility,

    // ── Lifecycle / Market Events ─────────────────────────────────────────
    /// Settlement / expiry delivery event
    SettlementEvent,
    /// Auction event (indicative price, crossing state)
    AuctionEvent,
    /// Market warning / trading halt notification
    MarketWarning,

    // ── Private streams (auth-required) ──────────────────────────────────
    /// Order lifecycle events (create/fill/cancel/expire)
    OrderUpdate,
    /// Account balance changes
    BalanceUpdate,
    /// Futures position changes
    PositionUpdate,
}

impl StreamKind {
    /// Returns true if this stream kind requires authentication.
    pub fn is_private(&self) -> bool {
        matches!(self, Self::OrderUpdate | Self::BalanceUpdate | Self::PositionUpdate)
    }

    /// Returns true if this variant carries a kline interval parameter.
    pub fn is_kline(&self) -> bool {
        matches!(
            self,
            Self::Kline { .. }
                | Self::MarkPriceKline { .. }
                | Self::IndexPriceKline { .. }
                | Self::PremiumIndexKline { .. }
        )
    }
}
