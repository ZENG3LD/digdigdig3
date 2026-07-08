//! Client-side re-cut library for the unified-data-plane arc
//! (`nemo/docs/mlc/plans/unified-data-plane-2026-07-09.md`, stage Э0).
//!
//! This module is a thin, curated re-export of the pure fold state
//! machines that live in [`crate::derived`]. Station itself uses these
//! machines internally to compute derived chart series (Renko, Point &
//! Figure, Kagi, Three-Line-Break, Range/Volume/Dollar/Tick bars, TPO
//! profiles, imbalance/run bars, footprint, CVD, basis, funding
//! settlement) from upstream `Trade`/`Kline` streams. Exposing them here
//! lets a consumer (e.g. `mylittlechart`) drive the SAME fold math
//! directly over its own client-side raw data layers, instead of
//! reimplementing the bar-construction algorithms — chart types become
//! client-side folds over shared raw series rather than server-computed
//! derived streams.
//!
//! ## Input contract
//!
//! Every fold machine implements [`DerivedStream`] and is fed by calling
//! [`DerivedStream::on_upstream_event`] once per upstream event, in
//! chronological order:
//!
//! - **Real or synthetic trades** — feed [`Event::Trade`]. Use
//!   [`kline_to_synthetic_trades`] to turn a single closed 1-minute
//!   kline bar into up to 4 synthetic trade legs (O→H→L→C / O→L→H→C
//!   price path) for cold-start / history folding when only kline data
//!   is available.
//! - **Live 1-minute kline delta-ticks** — feed [`Event::Bar`] directly
//!   into the "kline-sufficient" machines (`TradeToRangeBarDerived`,
//!   `TradeToVolumeBarDerived`, `TradeToDollarBarDerived`,
//!   `TradeToRenkoBarDerived`, `TradeToPnfBarDerived`,
//!   `TradeToKagiBarDerived`, `TradeToThreeLineBreakDerived`). Each of
//!   these embeds a [`KlineDeltaState`] internally and folds repeated
//!   `Event::Bar` updates for the same in-progress kline into
//!   incremental Δ-volume synthetic trades — the caller never has to
//!   manage `KlineDeltaState` directly, just keep feeding `Event::Bar`.
//!
//! ## Statefulness
//!
//! Every fold machine is a **stateful, single-purpose accumulator**: one
//! machine instance per output series. State (open bar, running
//! imbalance, grid anchor, session buckets, etc.) is entirely local to
//! `self` — no shared mutation, no locks, `Send` but not `Sync`-required.
//! Events MUST be fed in chronological order; there is no rollback or
//! re-ordering support. To recompute a series from scratch, construct a
//! fresh machine via one of the direct `new(...)` constructors (or
//! `new_for_key` if you have a [`crate::series::SeriesKey`] on hand) and
//! replay the full event history from the beginning.
//!
//! ## Construction
//!
//! Two ways to construct a fold machine:
//!
//! - **Direct constructor** — `TradeToVolumeBarDerived::new(threshold)`,
//!   `TradeToRenkoBarDerived::new(box_size, reversal_count)`, etc. Takes
//!   the same parameters `new_for_key` would have extracted from a
//!   [`crate::series::Kind`], with identical defaults/clamps, so a
//!   direct-constructed machine produces byte-identical output to a
//!   `new_for_key`-constructed one fed the same event sequence.
//! - **`new_for_key`** (via the [`DerivedStream`] trait) — extracts
//!   parameters from a [`crate::series::SeriesKey`]'s `kind` field.
//!   Useful when the caller already has a `SeriesKey` (e.g. mirroring a
//!   server-side derived series) rather than raw parameters.
use crate::derived;

/// Stateful, pure-computation stream trait. See [`crate::derived::DerivedStream`]
/// for the full contract (deps, seeding, event folding).
pub use derived::DerivedStream;

/// Live-folding adapter that turns repeated `Event::Bar` updates for the
/// same in-progress 1-minute kline into incremental Δ-volume synthetic
/// `Event::Trade` ticks. Embedded automatically by every kline-sufficient
/// fold machine — most callers never construct this directly.
pub use derived::KlineDeltaState;

/// Converts one closed 1-minute kline bar into up to 4 synthetic
/// `Event::Trade`s tracing the bar's O→H→L→C (or O→L→H→C) price path.
/// Use this to cold-start / backfill a price-path-triggered fold machine
/// (Renko / PnF / Kagi / Three-Line-Break / Range / Volume / Dollar) from
/// historical kline data when tick-level history is unavailable.
pub use derived::kline_to_synthetic_trades;

/// Bucket width → milliseconds conversion for `KlineInterval` strings
/// (`"1s"`, `"3m"`, `"2h"`, `"1d"`, `"1w"`, ...). Used internally by the
/// time-bucketed fold machines (`TradeToBarDerived`, `TradeToFootprintDerived`)
/// and useful to callers who need the same bucket-width semantics.
pub use derived::interval_to_ms;

// ---------------------------------------------------------------------------
// Fold state machines
// ---------------------------------------------------------------------------

/// Joins `MarkPrice` + `IndexPrice` upstream events, emitting
/// `BasisPoint { value = mark − index }`.
pub use derived::BasisDerived;

/// Crossing-detector over the `FundingRate` stream — emits a
/// `FundingSettlementPoint` each time the exchange's `next_funding_time`
/// boundary is crossed.
pub use derived::FundingSettlementDerived;

/// Aggregates `Trade` events into fixed-interval OHLCV bars (fallback
/// kline aggregator).
pub use derived::TradeToBarDerived;

/// Range bar: a new bar opens when price moves at least `range` from the
/// bar's open. Kline-sufficient (feeds off `Event::Bar` via the embedded
/// [`KlineDeltaState`] as well as `Event::Trade`).
pub use derived::TradeToRangeBarDerived;

/// Tick bar: closes a bar every `n` trades.
pub use derived::TradeToTickBarDerived;

/// Volume bar: closes a bar when cumulative volume crosses `threshold`.
/// Kline-sufficient.
pub use derived::TradeToVolumeBarDerived;

/// Dollar bar (López de Prado, AFML ch.2): closes a bar when cumulative
/// dollar value (`price × quantity`) crosses `threshold`. Kline-sufficient.
pub use derived::TradeToDollarBarDerived;

/// Renko brick: fixed-height bricks with directional reversal.
/// Kline-sufficient.
pub use derived::TradeToRenkoBarDerived;

/// Point & Figure (X/O) column aggregator. Kline-sufficient.
pub use derived::TradeToPnfBarDerived;

/// Kagi segment aggregator (yang/yin thickness + reversal threshold).
/// Kline-sufficient.
pub use derived::TradeToKagiBarDerived;

/// Three Line Break (san-sen-ashi) line aggregator. Kline-sufficient.
pub use derived::TradeToThreeLineBreakDerived;

/// Tick Imbalance Bar (López de Prado, AFML ch.2) — EMA-driven dynamic
/// threshold over signed tick imbalance.
pub use derived::TradeToTickImbalanceDerived;

/// Volume Imbalance Bar (López de Prado, AFML ch.2) — EMA-driven dynamic
/// threshold over signed-volume imbalance.
pub use derived::TradeToVolumeImbalanceDerived;

/// Run Bar (López de Prado, AFML ch.2) — EMA-driven dynamic threshold over
/// same-side tick run length.
pub use derived::TradeToRunBarDerived;

/// Time-bucketed OHLCV with per-price buy/sell volume breakdown
/// (footprint chart).
pub use derived::TradeToFootprintDerived;

/// Running Cumulative Volume Delta scalar line.
pub use derived::TradeToCvdLineDerived;

/// TPO Market Profile aggregator sourced from 1-minute klines
/// (canonical sivamgr / py-market-profile algorithm).
pub use derived::TpoFromKline1mDerived;

/// TPO Market Profile aggregator sourced directly from the live trade
/// stream (finer-grain bucket extremes, no kline backfill needed).
pub use derived::TpoFromTradeDerived;
