//! Persistence configuration for the Station builder.
//!
//! Actual disk I/O lives in [`crate::series::DiskStore`], generic over
//! [`crate::series::DataPoint`]. This module carries per-kind depth toggles
//! threaded through the builder.

use serde::{Deserialize, Serialize};

// ─── PersistDepth ────────────────────────────────────────────────────────────

/// How much of a payload to persist per Kind.
///
/// Different consumers want different granularity:
/// - `Compact`: lean record used by the bar-aligned mli/mlq loaders (existing
///   `<Kind>Point` types). Smallest disk footprint. Backward-compatible —
///   existing `.dat` files are produced AND read by this depth.
/// - `Indicators`: enriched record for indicator/feature engineering — adds
///   the modeling-relevant fields the exchange emits (bid/ask qty, weighted
///   avg, open, last qty, mark/index, OI, funding, count). Costs more disk
///   per record but no loss of common signals at replay.
/// - `Full`: every stable wire field on the payload struct. Largest record;
///   intended for historical research persistence where nothing should be
///   dropped at replay.
///
/// Default is `Compact` for every Kind for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistDepth {
    Compact,
    Indicators,
    Full,
}

impl Default for PersistDepth {
    fn default() -> Self {
        PersistDepth::Compact
    }
}

// ─── PersistenceConfig ───────────────────────────────────────────────────────

/// Per-kind persistence toggles.
///
/// `None` means "do not persist this kind". `Some(depth)` means persist at the
/// given depth. Master `enabled` gates everything; individual kinds can still be
/// `Some(_)` but produce no writes when `enabled = false`.
///
/// All fields that previously existed as `bool` now carry
/// `Option<PersistDepth>`. For backward compatibility:
/// - `Default` produces `None` for every kind (same as all-`false` before).
/// - `PersistenceConfig::on()` sets every field to `Some(Compact)` —
///   identical behavior to the previous `true` / all-on path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub trades: Option<PersistDepth>,
    pub agg_trades: Option<PersistDepth>,
    pub klines: Option<PersistDepth>,
    pub tickers: Option<PersistDepth>,
    pub orderbook_snapshots: Option<PersistDepth>,
    pub orderbook_deltas: Option<PersistDepth>,
    pub mark_price: Option<PersistDepth>,
    pub funding_rate: Option<PersistDepth>,
    pub open_interest: Option<PersistDepth>,
    pub liquidations: Option<PersistDepth>,
    // Extended kinds — opt-in, default off.
    pub index_price: Option<PersistDepth>,
    pub block_trade: Option<PersistDepth>,
    pub composite_index: Option<PersistDepth>,
    pub volatility_index: Option<PersistDepth>,
    pub historical_volatility: Option<PersistDepth>,
    pub long_short_ratio: Option<PersistDepth>,
    pub basis: Option<PersistDepth>,
    pub insurance_fund: Option<PersistDepth>,
    pub settlement_event: Option<PersistDepth>,
    pub predicted_funding: Option<PersistDepth>,
    pub funding_settlement: Option<PersistDepth>,
    pub risk_limit: Option<PersistDepth>,
    pub option_greeks: Option<PersistDepth>,
    pub mark_price_kline: Option<PersistDepth>,
    pub index_price_kline: Option<PersistDepth>,
    pub premium_index_kline: Option<PersistDepth>,
    pub market_warning: Option<PersistDepth>,
    pub orderbook_l3: Option<PersistDepth>,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trades: None,
            agg_trades: None,
            klines: None,
            tickers: None,
            orderbook_snapshots: None,
            orderbook_deltas: None,
            mark_price: None,
            funding_rate: None,
            open_interest: None,
            liquidations: None,
            index_price: None,
            block_trade: None,
            composite_index: None,
            volatility_index: None,
            historical_volatility: None,
            long_short_ratio: None,
            basis: None,
            insurance_fund: None,
            settlement_event: None,
            predicted_funding: None,
            funding_settlement: None,
            risk_limit: None,
            option_greeks: None,
            mark_price_kline: None,
            index_price_kline: None,
            premium_index_kline: None,
            market_warning: None,
            orderbook_l3: None,
        }
    }
}

impl PersistenceConfig {
    /// Enable persistence for every supported kind at `Compact` depth.
    ///
    /// Produces identical output to the previous `on()` that set all bools to
    /// `true`. Compact = existing `<Kind>Point` layout, no behavior change.
    pub fn on() -> Self {
        let c = Some(PersistDepth::Compact);
        Self {
            enabled: true,
            trades: c,
            agg_trades: c,
            klines: c,
            tickers: c,
            orderbook_snapshots: c,
            orderbook_deltas: c,
            mark_price: c,
            funding_rate: c,
            open_interest: c,
            liquidations: c,
            // Extended kinds default off even in on() — they were previously
            // guarded by `self.enabled` only, so on() must still enable them.
            index_price: c,
            block_trade: c,
            composite_index: c,
            volatility_index: c,
            historical_volatility: c,
            long_short_ratio: c,
            basis: c,
            insurance_fund: c,
            settlement_event: c,
            predicted_funding: c,
            funding_settlement: c,
            risk_limit: c,
            option_greeks: c,
            mark_price_kline: c,
            index_price_kline: c,
            premium_index_kline: c,
            market_warning: c,
            orderbook_l3: c,
        }
    }

    // Builder methods — accept `Option<PersistDepth>` so callers can do
    // `.tickers(Some(PersistDepth::Indicators))` or `.tickers(None)`.

    pub fn trades(mut self, depth: Option<PersistDepth>) -> Self { self.trades = depth; self }
    pub fn agg_trades(mut self, depth: Option<PersistDepth>) -> Self { self.agg_trades = depth; self }
    pub fn klines(mut self, depth: Option<PersistDepth>) -> Self { self.klines = depth; self }
    pub fn tickers(mut self, depth: Option<PersistDepth>) -> Self { self.tickers = depth; self }
    pub fn orderbook_snapshots(mut self, depth: Option<PersistDepth>) -> Self { self.orderbook_snapshots = depth; self }
    pub fn orderbook_deltas(mut self, depth: Option<PersistDepth>) -> Self { self.orderbook_deltas = depth; self }
    pub fn mark_price(mut self, depth: Option<PersistDepth>) -> Self { self.mark_price = depth; self }
    pub fn funding_rate(mut self, depth: Option<PersistDepth>) -> Self { self.funding_rate = depth; self }
    pub fn open_interest(mut self, depth: Option<PersistDepth>) -> Self { self.open_interest = depth; self }
    pub fn liquidations(mut self, depth: Option<PersistDepth>) -> Self { self.liquidations = depth; self }
    pub fn index_price(mut self, depth: Option<PersistDepth>) -> Self { self.index_price = depth; self }
    pub fn block_trade(mut self, depth: Option<PersistDepth>) -> Self { self.block_trade = depth; self }
    pub fn composite_index(mut self, depth: Option<PersistDepth>) -> Self { self.composite_index = depth; self }
    pub fn volatility_index(mut self, depth: Option<PersistDepth>) -> Self { self.volatility_index = depth; self }
    pub fn historical_volatility(mut self, depth: Option<PersistDepth>) -> Self { self.historical_volatility = depth; self }
    pub fn long_short_ratio(mut self, depth: Option<PersistDepth>) -> Self { self.long_short_ratio = depth; self }
    pub fn basis(mut self, depth: Option<PersistDepth>) -> Self { self.basis = depth; self }
    pub fn insurance_fund(mut self, depth: Option<PersistDepth>) -> Self { self.insurance_fund = depth; self }
    pub fn settlement_event(mut self, depth: Option<PersistDepth>) -> Self { self.settlement_event = depth; self }
    pub fn predicted_funding(mut self, depth: Option<PersistDepth>) -> Self { self.predicted_funding = depth; self }
    pub fn funding_settlement(mut self, depth: Option<PersistDepth>) -> Self { self.funding_settlement = depth; self }
    pub fn risk_limit(mut self, depth: Option<PersistDepth>) -> Self { self.risk_limit = depth; self }
    pub fn option_greeks(mut self, depth: Option<PersistDepth>) -> Self { self.option_greeks = depth; self }
    pub fn mark_price_kline(mut self, depth: Option<PersistDepth>) -> Self { self.mark_price_kline = depth; self }
    pub fn index_price_kline(mut self, depth: Option<PersistDepth>) -> Self { self.index_price_kline = depth; self }
    pub fn premium_index_kline(mut self, depth: Option<PersistDepth>) -> Self { self.premium_index_kline = depth; self }
    pub fn market_warning(mut self, depth: Option<PersistDepth>) -> Self { self.market_warning = depth; self }
    pub fn orderbook_l3(mut self, depth: Option<PersistDepth>) -> Self { self.orderbook_l3 = depth; self }

    /// Should `kind` be persisted given the current config?
    pub fn is_enabled_for(&self, kind: &crate::series::Kind) -> bool {
        self.depth_for(kind).is_some()
    }

    /// The configured depth for `kind`, or `None` if persistence is disabled
    /// for this kind (either master `enabled=false` or the kind's toggle is
    /// `None`).
    pub fn depth_for(&self, kind: &crate::series::Kind) -> Option<PersistDepth> {
        if !self.enabled {
            return None;
        }
        use crate::series::Kind::*;
        match kind {
            Trade => self.trades,
            AggTrade => self.agg_trades,
            Kline(_) => self.klines,
            Ticker => self.tickers,
            Orderbook => self.orderbook_snapshots,
            OrderbookDelta => self.orderbook_deltas,
            MarkPrice => self.mark_price,
            FundingRate => self.funding_rate,
            OpenInterest => self.open_interest,
            Liquidation => self.liquidations,
            IndexPrice => self.index_price,
            BlockTrade => self.block_trade,
            CompositeIndex => self.composite_index,
            VolatilityIndex => self.volatility_index,
            HistoricalVolatility => self.historical_volatility,
            LongShortRatio => self.long_short_ratio,
            Basis => self.basis,
            InsuranceFund => self.insurance_fund,
            SettlementEvent => self.settlement_event,
            PredictedFunding => self.predicted_funding,
            FundingSettlement => self.funding_settlement,
            RiskLimit => self.risk_limit,
            OptionGreeks => self.option_greeks,
            MarkPriceKline(_) => self.mark_price_kline,
            IndexPriceKline(_) => self.index_price_kline,
            PremiumIndexKline(_) => self.premium_index_kline,
            MarketWarning => self.market_warning,
            OrderbookL3 => self.orderbook_l3,
            // Mechanical bar aggregators — follow global enabled state at Compact.
            RangeBar(_) | TickBar(_) | VolumeBar(_) | Footprint(_) => {
                if self.enabled { Some(PersistDepth::Compact) } else { None }
            }
            // Private streams are ephemeral — never persisted.
            OrderUpdate | BalanceUpdate | PositionUpdate => None,
        }
    }
}
