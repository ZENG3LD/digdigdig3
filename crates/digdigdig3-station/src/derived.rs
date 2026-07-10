//! Derived-stream layer for `digdigdig3-station`.
//!
//! A *derived stream* is a Station-internal computation that subscribes to one
//! or more upstream WS-backed streams and emits events of its own type. It runs
//! as a standalone `tokio::spawn` task per `SeriesKey`, sharing the same
//! `DiskStore<T>` / `Series<T>` / `broadcast::channel<Event>` plumbing as
//! regular WS forwarders. Consumers see no difference.
//!
//! Concrete impls shipped in this module:
//!
//! - [`BasisDerived`] — joins `MarkPrice` + `IndexPrice`, emits
//!   `BasisPoint { value = mark − index }`. Rejects pairs skewed > 2 seconds.
//! - [`FundingSettlementDerived`] — monitors `FundingRate`, emits
//!   `FundingSettlementPoint` each time `next_funding_time` advances past the
//!   current wall clock (crossing-detector pattern).
//! - [`TradeToBarDerived`] — subscribes to `Trade` and aggregates individual
//!   trades into OHLCV bars of a fixed interval. Used as a fallback when the
//!   venue's WS does not natively offer the requested `Kind::Kline(interval)`.
//! - [`TradeToTickBarDerived`] — closes a bar every `n` trades.
//! - [`TradeToFootprintDerived`] — time-bucketed OHLCV with per-price buy/sell breakdown.
//! - [`TradeToTickImbalanceDerived`] — López de Prado Tick Imbalance Bar.
//! - [`TradeToVolumeImbalanceDerived`] — López de Prado Volume Imbalance Bar.
//! - [`TradeToRunBarDerived`] — López de Prado Run Bar.
//!
//! ## Kline-sufficient kinds (PURE kline converters — no `Trade` dependency)
//!
//! [`TradeToRangeBarDerived`], [`TradeToVolumeBarDerived`],
//! [`TradeToDollarBarDerived`], [`TradeToRenkoBarDerived`],
//! [`TradeToPnfBarDerived`], [`TradeToKagiBarDerived`],
//! [`TradeToThreeLineBreakDerived`] subscribe ONLY to
//! `Stream::Kline(KlineInterval::new("1m"))` — never `Stream::Trade`. Their
//! price-path/volume math is fully recoverable from 1m klines: cold-start
//! history folds each closed kline into up to 4 synthetic O→H→L→C /
//! O→L→H→C trade legs via [`kline_to_synthetic_trades`]; the live WS kline
//! stream folds into incremental Δ-volume synthetic ticks via
//! [`KlineDeltaState`] (shared by all seven). See
//! [`DerivedStream::seed_kline_baseline`] for how the cold-seed hands its
//! last folded kline's baseline to the live adapter to avoid a
//! double-counted seam.

use crate::data::{
    BarPoint, BasisPoint, FootprintPoint, FundingRatePoint, FundingSettlementPoint,
    KagiSegmentPoint, MarkPricePoint, IndexPricePoint, PnfColumnPoint,
    RenkoBrickPoint, ScalarBarPoint, ThreeLineBreakLinePoint, TpoSessionPoint, TradePoint,
};
use std::collections::VecDeque;
use crate::series::{DataPoint, Kind, TpoSource};
use crate::series::SeriesKey;
use crate::subscription::{Event, Stream};
use digdigdig3::core::websocket::KlineInterval;

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Stateful, pure-computation stream that subscribes to one or more upstream
/// broadcast channels and emits its own `Output` type.
///
/// Implementations are spawned once per `SeriesKey` and run for the lifetime
/// of the derived multiplexer. All state is local to `self` — no shared
/// mutation, no locks.
pub trait DerivedStream: Send + 'static {
    /// Output data point type. Must already implement `DataPoint` and have a
    /// corresponding `EventFrom<Self::Output>` impl in station.rs.
    type Output: DataPoint;

    /// Upstream `Stream` variants this derived stream needs (static set).
    /// Most derived streams have a fixed dep set independent of the
    /// `SeriesKey` and override only this. Order is significant —
    /// `on_upstream_event` receives `dep_idx` matching the index of the
    /// stream in this slice.
    ///
    /// Derived streams whose deps carry a runtime parameter (e.g. a
    /// `Stream::Kline(KlineInterval)` where the interval is read from
    /// `key.kind`) implement [`deps_for_key`] instead and return `&[]`
    /// here.
    fn deps() -> &'static [Stream];

    /// Per-key dependency set, called once at spawn time. The default
    /// returns the static `deps()` cloned into a Vec — implementations
    /// whose deps need a runtime parameter (TPO subscribes to a specific
    /// `Stream::Kline(interval)`) override this.
    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        Self::deps().to_vec()
    }

    /// Construct initial (empty) state for the given key. Called once before
    /// the forwarder loop begins.
    fn new_for_key(key: &SeriesKey) -> Self;

    /// Process one upstream `Event`. Returns `Some(point)` if a derived output
    /// should be emitted, `None` to silently absorb the event.
    ///
    /// `dep_idx` is the index into `Self::deps()` that produced this event,
    /// allowing implementations to branch without repeated pattern-matching.
    ///
    /// This is a **one-output-per-call** contract — some implementations
    /// (e.g. gap-crossing range bars, multi-brick Renko moves) can close
    /// MORE than one output element while folding a single input event. Any
    /// such intermediate, fully-closed elements are queued internally and
    /// must be drained via [`DerivedStream::take_pending`] immediately after
    /// this call — they are NOT re-derivable from the `Option` return alone.
    fn on_upstream_event(&mut self, ev: &Event, dep_idx: usize) -> Option<Self::Output>;

    /// Drain any fully-closed output elements that were produced as a
    /// side-effect of the most recent [`DerivedStream::on_upstream_event`]
    /// call but could not fit through its single `Option` return slot.
    ///
    /// Default: always empty — the vast majority of `DerivedStream` impls
    /// emit at most one output per input event and never populate a pending
    /// queue. Implementations whose fold loop can close a CHAIN of elements
    /// within one call (gap-crossing range bars, multi-brick Renko moves)
    /// override this to drain their internal `pending: Vec<Self::Output>`
    /// buffer. Callers MUST call this after every `on_upstream_event` (and
    /// after every event fed inside `seed_from_events`) and emit/broadcast
    /// the drained elements BEFORE the `Option` return value — pending
    /// elements are the earlier, now-closed part of the chain; the `Option`
    /// return is always the latest (possibly still in-progress) element.
    fn take_pending(&mut self) -> Vec<Self::Output> {
        Vec::new()
    }

    /// Seed internal state from a batch of upstream events at spawn time, before
    /// the live event loop begins. Returns all derived outputs emitted during the
    /// seed pass so callers can broadcast them (cold-start window visible to
    /// consumers immediately).
    ///
    /// `dep_idx` matches the index in `Self::deps()`. Default implementation
    /// turns replay-mode ON for the duration of the pass (see
    /// [`DerivedStream::set_replay`]), feeds each event through
    /// `on_upstream_event`, draining [`DerivedStream::take_pending`] after
    /// every event so no chained intermediate element is lost during
    /// cold-seed / rewarm, then appends the final in-progress snapshot via
    /// [`DerivedStream::flush_replay`] before turning replay mode back off —
    /// works for all trade-derived impls without any additional code. Machines
    /// that override `set_replay`/`flush_replay` get the quadratic-snapshot
    /// suppression for free through this default; machines that don't
    /// (the no-op defaults) behave exactly as before this contract was added.
    fn seed_from_events(&mut self, events: &[Event], dep_idx: usize) -> Vec<Self::Output> {
        self.set_replay(true);
        let mut out = Vec::with_capacity(events.len());
        for e in events {
            let final_output = self.on_upstream_event(e, dep_idx);
            out.extend(self.take_pending());
            out.extend(final_output);
        }
        out.extend(self.flush_replay());
        self.set_replay(false);
        out
    }

    /// Toggle replay mode. `on = true` is set for the duration of a
    /// history-fold pass (cold-seed / rewarm — see the default
    /// [`DerivedStream::seed_from_events`]); `on = false` restores normal
    /// live behavior.
    ///
    /// ## Semantics when ON
    ///
    /// [`DerivedStream::on_upstream_event`] still updates ALL internal state
    /// exactly as in live mode — every trade/bar is folded, every anchor /
    /// EMA / accumulator advances identically. The ONLY difference is the
    /// **emission** side: the expensive per-event in-progress snapshot
    /// (`Some(...)` on every consumed event, needed live so a bar visibly
    /// grows on screen) is suppressed. `on_upstream_event` returns `Some(...)`
    /// ONLY when an output element actually CLOSES/rolls (bar closes, brick
    /// completes, bucket rolls, line breaks, segment completes) — the exact
    /// same elements [`DerivedStream::take_pending`] already carries for
    /// chained closes today. Replay mode does not change take_pending's
    /// contract at all; it only trims the `Option` return's live-only
    /// in-progress payload.
    ///
    /// Default: no-op — implementations that already emit at most a cheap
    /// clone per event (or already return `None` on non-terminal events)
    /// don't need to override this; the default `seed_from_events` calling
    /// `set_replay(true)`/`set_replay(false)` around them is harmless.
    /// Implementations whose in-progress snapshot is expensive to build
    /// (full price-row scan, whole-vector clone) override this to flip an
    /// internal `replay: bool` flag that their `on_upstream_event`/fold
    /// helpers check before constructing that snapshot.
    fn set_replay(&mut self, _on: bool) {}

    /// Return the CURRENT in-progress snapshot — the same value the last
    /// live emission would have carried — WITHOUT mutating machine state
    /// (clone-out only, never advances any accumulator/anchor/bucket). Used
    /// by the caller after a replay-mode fold pass to cap the replayed
    /// array with the live edge, since replay-mode suppressed that snapshot
    /// on every non-closing event.
    ///
    /// Default: `None` — implementations that never suppress an in-progress
    /// snapshot (the `set_replay` no-op default) have nothing to flush.
    fn flush_replay(&mut self) -> Option<Self::Output> {
        None
    }

    /// Hand the cold-seed's last-folded-kline baseline to the live
    /// delta-tick adapter, so the first live WS kline update for the SAME
    /// `open_time` computes a correct `Δvolume` against the seed's already-
    /// folded volume instead of double-counting or under-counting.
    ///
    /// Called once, immediately after [`DerivedStream::new_for_key`] and
    /// before any live event is fed, ONLY when the cold-seed produced at
    /// least one kline bar. Kline-sufficient kinds
    /// (Renko/PnF/Kagi/3LB/Range/Volume/Dollar) override this to forward
    /// into their embedded [`KlineDeltaState::seed_baseline`]. Every other
    /// derived kind keeps the no-op default.
    fn seed_kline_baseline(&mut self, _open_time: i64, _volume: f64, _close: f64) {}
}

// ---------------------------------------------------------------------------
// KlineDeltaState — shared live Δ-tick adapter for the kline-sufficient
// kinds (Renko / PnF / Kagi / Three-Line-Break / Range / Volume / Dollar)
// ---------------------------------------------------------------------------

/// Live-folding adapter shared by every kline-sufficient derived kind.
///
/// The WS `Stream::Kline("1m")` upstream delivers repeated `Event::Bar`
/// updates for the SAME in-progress bar (same `open_time`, growing
/// `volume`, moving `close`), then a new `open_time` once that bar closes.
/// This adapter folds that stream into synthetic `Event::Trade` ticks with
/// NO rollback, per the doctrine:
///
/// - Same `open_time` as the last seen bar → emit ONE synthetic trade:
///   `price = point.close`, `quantity = max(point.volume - last_seen_volume, 0)`,
///   `side` inferred from `point.close` vs the last close. A zero/negative
///   delta (duplicate frame, or a corrective re-send with lower volume)
///   emits nothing.
/// - New `open_time` (bar rolled) → the previous bar is final and is NEVER
///   re-folded (its path was already folded tick-by-tick as it happened).
///   The new bar starts its own Δ-accounting from a zero volume baseline,
///   so the first observed update on the new bar contributes its own
///   already-accrued volume as one synthetic trade — never the delta
///   against the old bar's volume.
/// - `last_close` carries across a rollover (price-continuity for side
///   inference); `last_seen_volume` does not.
///
/// ## Cold-seed seam
///
/// [`KlineDeltaState::seed_baseline`] primes `last_open_time` /
/// `last_seen_volume` / `last_close` from the cold-seed's last folded
/// kline (see [`DerivedStream::seed_kline_baseline`]) BEFORE any live event
/// arrives, so a live update sharing that same `open_time` computes a
/// correct incremental delta instead of re-folding the whole bar.
///
/// ## Timestamps
///
/// No wall-clock read (wasm-safe, deterministic). Each synthetic tick's
/// `ts_ms` is `point.open_time + offset`, where `offset` is a per-bar
/// monotonically increasing counter clamped to `[0, interval_ms)` — same
/// spacing family as [`kline_to_synthetic_trades`], generalized to an
/// unbounded number of live ticks per bar instead of a fixed 4 legs.
pub struct KlineDeltaState {
    /// `open_time` of the last observed bar. `i64::MIN` = unseeded.
    last_open_time: i64,
    /// Cumulative volume already accounted for within `last_open_time`.
    last_seen_volume: f64,
    /// Last observed close price (carried across rollovers for side inference).
    last_close: f64,
    /// Per-bar monotonic tick offset, reset to 0 on every rollover.
    tick_offset: i64,
}

impl KlineDeltaState {
    pub fn new() -> Self {
        Self {
            last_open_time: i64::MIN,
            last_seen_volume: 0.0,
            last_close: 0.0,
            tick_offset: 0,
        }
    }

    /// Prime the baseline from the cold-seed's last folded kline bar. Must
    /// be called before the first live event — see
    /// [`DerivedStream::seed_kline_baseline`].
    pub fn seed_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.last_open_time = open_time;
        self.last_seen_volume = volume;
        self.last_close = close;
        self.tick_offset = 0;
    }

    /// Fold one `Event::Bar` update into at most one synthetic
    /// `Event::Trade`. Returns `None` when there is nothing new to report
    /// (duplicate frame / non-positive delta) or when `interval_ms` cannot
    /// be resolved from `timeframe`.
    fn to_delta_trade(
        &mut self,
        bar: &BarPoint,
        timeframe: &KlineInterval,
        exchange: digdigdig3::core::types::ExchangeId,
        symbol: &str,
    ) -> Option<Event> {
        let interval_ms = interval_to_ms(timeframe.as_str()).unwrap_or(60_000).max(1);

        if self.last_open_time == i64::MIN {
            // First event ever seen with no cold-seed baseline handed over —
            // establish the baseline without emitting (nothing to delta
            // against yet; avoids treating the entire in-progress bar's
            // volume-to-date as a single synthetic trade on cold start).
            self.last_open_time = bar.open_time;
            self.last_seen_volume = bar.volume;
            self.last_close = bar.close;
            self.tick_offset = 0;
            return None;
        }

        let delta_volume = if bar.open_time == self.last_open_time {
            (bar.volume - self.last_seen_volume).max(0.0)
        } else if bar.open_time > self.last_open_time {
            // Bar rolled — previous bar is final, never re-folded. Baseline
            // for the new bar starts at zero volume.
            self.last_open_time = bar.open_time;
            self.tick_offset = 0;
            bar.volume.max(0.0)
        } else {
            // Stale/out-of-order bar (older open_time) — ignore.
            return None;
        };

        if delta_volume <= 0.0 {
            self.last_seen_volume = bar.volume;
            self.last_close = bar.close;
            return None;
        }

        let side = u8::from(bar.close < self.last_close);
        let offset = self.tick_offset.clamp(0, interval_ms - 1);
        self.tick_offset += 1;

        self.last_seen_volume = bar.volume;
        self.last_close = bar.close;

        Some(Event::Trade {
            exchange,
            symbol: symbol.to_string(),
            point: TradePoint {
                ts_ms: bar.open_time + offset,
                price: bar.close,
                quantity: delta_volume,
                side,
                trade_id_hash: 0,
            },
        })
    }
}

// ---------------------------------------------------------------------------
// BasisDerived
// ---------------------------------------------------------------------------

/// Joins `MarkPrice` (dep index 0) and `IndexPrice` (dep index 1), emitting
/// `BasisPoint { value = mark − index }` on every qualifying update.
///
/// Pairs with timestamps more than `max_skew_ms` apart are rejected. Default
/// skew threshold is 2 000 ms — 10× the typical 200 ms inter-channel lag on
/// Binance `markPrice@1s` / `indexPrice@1s`.
pub struct BasisDerived {
    /// Most recent (ts_ms, mark_price) from the MarkPrice upstream.
    last_mark: Option<(i64, f64)>,
    /// Most recent (ts_ms, index_price) from the IndexPrice upstream.
    last_index: Option<(i64, f64)>,
    /// Maximum allowed age difference between the two sides (milliseconds).
    max_skew_ms: i64,
}

impl BasisDerived {
    /// Direct constructor: `max_skew_ms` is the maximum allowed timestamp
    /// gap between the `MarkPrice` and `IndexPrice` sides of a pair
    /// (default via `new_for_key` is 2 000 ms).
    pub fn new(max_skew_ms: i64) -> Self {
        Self {
            last_mark: None,
            last_index: None,
            max_skew_ms,
        }
    }
}

impl DerivedStream for BasisDerived {
    type Output = BasisPoint;

    fn deps() -> &'static [Stream] {
        &[Stream::MarkPrice, Stream::IndexPrice]
    }

    fn new_for_key(_key: &SeriesKey) -> Self {
        Self::new(2_000)
    }

    fn on_upstream_event(&mut self, ev: &Event, dep_idx: usize) -> Option<BasisPoint> {
        match dep_idx {
            0 => {
                if let Event::MarkPrice { point, .. } = ev {
                    self.last_mark = Some((point.ts_ms, point.mark));
                }
            }
            1 => {
                if let Event::IndexPrice { point, .. } = ev {
                    self.last_index = Some((point.ts_ms, point.price));
                }
            }
            _ => return None,
        }

        let (mark_ts, mark) = self.last_mark?;
        let (idx_ts,  idx)  = self.last_index?;

        if (mark_ts - idx_ts).abs() > self.max_skew_ms {
            return None;
        }

        let now_ms = mark_ts.max(idx_ts);
        Some(BasisPoint {
            ts_ms: now_ms,
            value: mark - idx,
            mark,
            index: idx,
        })
    }
}

// ---------------------------------------------------------------------------
// interval_to_ms
// ---------------------------------------------------------------------------

/// Convert a [`KlineInterval`] string (e.g. `"1s"`, `"3m"`, `"2h"`, `"1d"`,
/// `"1w"`) to its duration in milliseconds.
///
/// Returns `None` for any unrecognised string — the caller must handle that
/// case (typically by refusing to spawn the aggregator and returning a
/// `StationError::StreamNotSupported`).
///
/// Handled intervals (Binance / common exchange convention):
/// `1s 3s 5s 10s 15s 30s 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w`
pub fn interval_to_ms(interval: &str) -> Option<i64> {
    // Fast path for the common case: one-or-two digit number + single letter.
    let bytes = interval.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let unit = *bytes.last()?;
    // Parse the numeric prefix.
    let n_str = std::str::from_utf8(&bytes[..bytes.len().saturating_sub(1)]).ok()?;
    let n: i64 = n_str.parse().ok()?;
    if n <= 0 {
        return None;
    }
    const SEC: i64 = 1_000;
    const MIN: i64 = 60 * SEC;
    const HOUR: i64 = 60 * MIN;
    const DAY: i64 = 24 * HOUR;
    match unit {
        b's' => Some(n * SEC),
        b'm' => Some(n * MIN),
        b'h' | b'H' => Some(n * HOUR),
        b'd' | b'D' => Some(n * DAY),
        b'w' | b'W' => Some(n * 7 * DAY),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// TradeToBarDerived
// ---------------------------------------------------------------------------

/// Aggregates individual `Trade` events into OHLCV [`BarPoint`] bars for a
/// fixed `interval` (e.g. `"1m"`, `"5m"`, `"1h"`).
///
/// Used as a **fallback** when the venue's WS does not natively offer the
/// requested `Kind::Kline(interval)`. Station spawns this derived stream
/// instead of a WS forwarder, and consumers receive the same `Event::Bar`
/// events as they would from a native kline channel.
///
/// ## Bar semantics
///
/// Bars are aligned to UTC epoch boundaries: `bucket_start = (ts_ms /
/// interval_ms) * interval_ms`. Each trade either starts a new bar or
/// updates the current one in-place. The bar is emitted (as a partial /
/// open bar) on **every trade**, so consumers see live intra-bar updates.
/// When a trade from the next bucket arrives the previous bar is implicitly
/// closed (its last emitted state is the final OHLCV). Empty intervals
/// produce no bar — identical to HyperLiquid's native kline behaviour.
///
/// ## Construction via `new_for_key`
///
/// If `key.kind` is not `Kind::Kline(iv)`, or if `interval_to_ms` returns
/// `None` for the interval string, `interval_ms` is set to `0`. With
/// `interval_ms == 0` no buckets can ever form and `on_upstream_event`
/// always returns `None` — safe and panic-free.
pub struct TradeToBarDerived {
    /// Bucket width in milliseconds. `0` means "disabled" (unknown interval).
    interval_ms: i64,
    /// The currently-open (partial) bar, if any.
    current: Option<BarPoint>,
    /// Bucket start of `current` (ms). `0` when `current` is `None`.
    current_bucket_start: i64,
}

impl TradeToBarDerived {
    /// Direct constructor: `interval_ms` is the bucket width in
    /// milliseconds (`0` disables the aggregator — see struct docs).
    pub fn new(interval_ms: i64) -> Self {
        Self { interval_ms, current: None, current_bucket_start: 0 }
    }
}

impl DerivedStream for TradeToBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] {
        &[Stream::Trade]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let interval_ms = match &key.kind {
            Kind::Kline(iv) => interval_to_ms(iv.as_str()).unwrap_or(0),
            _ => 0,
        };
        Self::new(interval_ms)
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        // Disabled aggregator (unknown interval) or non-Trade event.
        if self.interval_ms == 0 {
            return None;
        }
        let Event::Trade { point, .. } = ev else { return None };

        let bucket_start = (point.ts_ms / self.interval_ms) * self.interval_ms;

        if self.current.is_none() || bucket_start > self.current_bucket_start {
            // Open a new bar.
            let bar = BarPoint {
                open_time: bucket_start,
                open:  point.price,
                high:  point.price,
                low:   point.price,
                close: point.price,
                volume:       point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            };
            self.current = Some(bar.clone());
            self.current_bucket_start = bucket_start;
            Some(bar)
        } else {
            // Update current bar in-place.
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            Some(bar.clone())
        }
    }
}

// ---------------------------------------------------------------------------
// TradeToRangeBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into OHLCV [`BarPoint`] bars triggered by price movement.
///
/// ## Bar semantics (canonical TRUE RANGE bars)
///
/// `range` (from `Kind::RangeBar(r)`) is stored as `r / 1e8` internally.
/// A bar closes the moment its running `(high − low)`, INCLUDING the current
/// trade, would reach `range` — i.e. the trigger is the bar's own price
/// envelope width, not the distance from `open`. `open` may sit anywhere
/// inside the closed bar's `[low, high]` envelope (it is NOT required to sit
/// on the boundary) — this is the key difference from the older
/// distance-from-open rule.
///
/// - **Chained open**: every new bar's `open` = the previous bar's `close`
///   exactly (no tick-size drift — `Kind::RangeBar` carries no tick-size
///   parameter, so the "±1 tick" variant is not applicable here; the plain
///   chained-open scheme is used).
/// - **Closed bar geometry**: `high − low == range` EXACTLY. `close` is the
///   boundary price that was breached (whichever extreme — high or low —
///   the closing trade just extended to complete the envelope). `open`
///   stays the bar's real open (unchanged by the close) and may therefore
///   sit anywhere inside `[low, high]` — the body (`|close − open|`) can be
///   strictly less than `range` whenever `open` is not itself one of the
///   two envelope edges. An in-progress (still-open) bar keeps tracking its
///   organically-visited high/low as trades arrive; the exact-range pin
///   only applies at the instant a bar closes.
/// - **Overshoot / gap chaining**: a single incoming trade whose price would
///   widen the envelope by MORE than the remaining room to `range` chains
///   into subsequent bars — same loop shape as
///   [`TradeToRenkoBarDerived::fold_trade`]. Each chained bar opens at the
///   previous chained bar's pinned close and spans exactly `range` in the
///   direction of travel, walking the overshoot until the remainder is
///   fully absorbed by a final still-open bar. `on_upstream_event` only has
///   ONE `Option` return slot, so every fully-CLOSED intermediate bar in the
///   chain is pushed to an internal `pending: Vec<BarPoint>` queue as it
///   closes — callers MUST drain it via [`DerivedStream::take_pending`]
///   immediately after every call (and `seed_from_events` does this
///   automatically during cold-start) or those bars are permanently lost.
///   Each chained bar (pending or final) has its own strictly monotonic
///   `open_time` via the scheme below.
///
/// ## Monotonic `open_time`
///
/// Range/tick/volume bars can close in the same millisecond. `Series::upsert_by_ts`
/// keys on `open_time`, so two distinct bars with equal ms would collapse.
/// To prevent this, `open_time` is `max(first_trade_ts, last_emitted_open_time + 1)`.
/// This guarantees strict monotonicity across all emitted bars without altering
/// the semantic meaning of `open_time` (it remains the timestamp of the first
/// trade in the bar, or 1 ms later if that timestamp collides). Every chained
/// intermediate bar in a gap-crossing consumes its own `+1` tick, so a chain
/// of N bars from one trade still yields N strictly increasing `open_time`s.
///
/// ## Emit semantics
///
/// The current open bar is emitted on **every trade** (same as `TradeToBarDerived`),
/// so consumers see live intra-bar updates via upsert.
///
/// ## Disabled guard
///
/// If `range == 0` (key kind is not `RangeBar` or param is zero), `on_upstream_event`
/// always returns `None` — safe and panic-free.
pub struct TradeToRangeBarDerived {
    /// Minimum price movement to close bar, in native price units (`param / 1e8`).
    /// `0.0` means disabled.
    range: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// The `open_time` of the last bar that was finalized (emitted as a closed bar).
    /// Used for monotonic open_time collision avoidance.
    last_emitted_open_time: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Fully-closed intermediate bars produced by a gap-crossing chain
    /// within the most recent `fold_trade` call, queued for
    /// [`DerivedStream::take_pending`] — see the "Gap chaining" doc above.
    pending: Vec<BarPoint>,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. When `true`,
    /// [`Self::fold_trade`] still updates `current`/`pending` exactly as in
    /// live mode, but returns `Some(..)` from `on_upstream_event` only when
    /// this call closed at least one bar (chain roll), not on every organic
    /// in-bar update.
    replay: bool,
}

impl TradeToRangeBarDerived {
    /// Direct constructor: `range` is the minimum price movement to close
    /// a bar, in native price units (`0.0` disables the aggregator).
    pub fn new(range: f64) -> Self {
        Self { range, current: None, last_emitted_open_time: 0, kline_state: KlineDeltaState::new(), pending: Vec::new(), replay: false }
    }
}

impl DerivedStream for TradeToRangeBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let range = match &key.kind {
            Kind::RangeBar(r) if *r > 0 => *r as f64 / 1e8,
            _ => 0.0,
        };
        Self::new(range)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn take_pending(&mut self) -> Vec<BarPoint> {
        std::mem::take(&mut self.pending)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.range == 0.0 { return None; }
        match ev {
            // Cold-seed / rewarm: 4-leg synthetic trades from
            // `kline_to_synthetic_trades` — fed directly.
            Event::Trade { point, .. } => self.fold_trade(point),
            // Live WS path: fold the kline update into a Δ-volume synthetic
            // tick via the shared adapter, then feed it the same way.
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToRangeBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<BarPoint> {
        // First trade ever — open the seed bar, nothing to chain against.
        // Always returned (even in replay mode) — this is the FIRST bar of
        // the whole series, semantically a "roll" from nothing.
        if self.current.is_none() {
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            let bar = BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            };
            self.current = Some(bar.clone());
            return Some(bar);
        }

        let mut closed_any = false;

        // True-range trigger loop — closes the current bar the moment its
        // running `(high − low)` INCLUDING `point.price` would reach
        // `range`, independent of where `open` sits inside the envelope.
        // A single trade extends at most one side of the envelope (it is
        // one price point compared against both extremes), so each
        // iteration only needs to check the side actually being widened.
        // Overshoot beyond the pinned boundary re-enters the loop as a
        // fresh single-point envelope on the newly-opened chained bar —
        // same chaining shape as `TradeToRenkoBarDerived::fold_trade`. The
        // crossing trade's volume is credited to the LAST bar it lands in
        // (the `width < range` branch below) — intermediate chained bars
        // close with zero volume of their own, same convention as the
        // pre-true-range implementation.
        loop {
            let bar = self.current.as_mut().unwrap_or_else(|| unreachable!("current is always Some inside the loop"));

            let widens_up   = point.price > bar.high;
            let widens_down = point.price < bar.low;
            let candidate_high = if widens_up   { point.price } else { bar.high };
            let candidate_low  = if widens_down { point.price } else { bar.low  };
            let width = candidate_high - candidate_low;

            if width < self.range {
                // Envelope does not yet reach `range` — absorb organically,
                // no close. `open` is left untouched (it may already sit
                // anywhere inside `[low, high]`).
                if widens_up   { bar.high = point.price; }
                if widens_down { bar.low  = point.price; }
                bar.close = point.price;
                bar.volume       += point.quantity;
                bar.quote_volume += point.price * point.quantity;
                bar.trades_count += 1;
                break;
            }

            // Envelope breaches `range` on this trade. Pin the breached
            // side to exactly `low + range` (up) / `high − range` (down) —
            // canonical closed-bar geometry: `high − low == range` exactly,
            // `open` is untouched (may be anywhere inside `[low, high]`,
            // so the body can be strictly narrower than `range`).
            let (close_price, pinned_high, pinned_low) = if widens_up {
                let ph = bar.low + self.range;
                (ph, ph, bar.low)
            } else {
                let pl = bar.high - self.range;
                (pl, bar.high, pl)
            };
            bar.high = pinned_high;
            bar.low  = pinned_low;
            bar.close = close_price;

            // This bar is now fully CLOSED (pinned high/low/close) — queue
            // it for `take_pending` before superseding `self.current` with
            // the next chained bar. Without this, the intermediate bar
            // would be silently discarded.
            if let Some(closed) = self.current.take() {
                self.pending.push(closed);
            }
            closed_any = true;

            // Open the next chained bar at the pinned close — canonical
            // chained-open (no tick-size data available at this machine).
            let new_open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = new_open_time;
            self.current = Some(BarPoint {
                open_time:    new_open_time,
                open:         close_price,
                high:         close_price,
                low:          close_price,
                close:        close_price,
                volume:       0.0,
                quote_volume: 0.0,
                trades_count: 0,
            });
            // Continue the loop against the SAME `point.price` — a large
            // single-trade gap can chain through several full-range bars
            // before the remainder falls under `range`.
        }

        // Replay mode: suppress the in-progress-snapshot clone unless this
        // call actually closed at least one bar (chain roll) — the closed
        // bar(s) are already queued in `pending`; the freshly-opened
        // `current` returned here would otherwise be re-emitted on every
        // single organic in-bar update during history replay.
        if self.replay && !closed_any {
            return None;
        }
        self.current.clone()
    }
}

// ---------------------------------------------------------------------------
// TradeToTickBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into OHLCV [`BarPoint`] bars triggered by trade count.
///
/// ## Bar semantics
///
/// Closes a bar every `n` trades (from `Kind::TickBar(n)`). The `n`-th trade
/// belongs to the **closing** bar.
///
/// See [`TradeToRangeBarDerived`] for the monotonic `open_time` scheme and
/// intra-bar emit semantics — both apply here.
pub struct TradeToTickBarDerived {
    /// Trades per bar. `0` means disabled.
    n: u32,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// Trades accumulated in the current bar.
    count: u32,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToTickBarDerived {
    /// Direct constructor: `n` is the number of trades per bar (`0`
    /// disables the aggregator).
    pub fn new(n: u32) -> Self {
        Self { n, current: None, count: 0, last_emitted_open_time: 0, replay: false }
    }
}

impl DerivedStream for TradeToTickBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let n = match &key.kind {
            Kind::TickBar(n) if *n > 0 => *n,
            _ => 0,
        };
        Self::new(n)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.n == 0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        if self.current.is_none() {
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
            self.count = 1;
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            self.count += 1;
        }

        let bar = self.current.clone()?;

        // Roll if we hit n trades.
        let rolled = self.count >= self.n;
        if rolled {
            // Next bar will be opened on the next trade.
            self.current = None;
            self.count = 0;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToVolumeBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into OHLCV [`BarPoint`] bars triggered by cumulative volume.
///
/// ## Bar semantics
///
/// `threshold` (from `Kind::VolumeBar(v)`) is stored as `v / 1e8` internally.
/// The bar closes when `cumulative_volume >= threshold`. The trade that crosses
/// the threshold belongs to the **closing** bar (no volume carry-over to the
/// next bar — document: partial fills that split across bars are not tracked).
///
/// ## Open semantics (canonical — same as tick bars)
///
/// `open` is the FIRST TRADE's real price of the bar — genuine gaps between
/// consecutive bars are allowed (a volume bar is not price-path-derived like
/// range/Renko bars, so there is nothing to chain: the threshold is purely
/// volume-based). This mirrors [`TradeToTickBarDerived`].
///
/// See [`TradeToRangeBarDerived`] for monotonic `open_time` and emit semantics.
pub struct TradeToVolumeBarDerived {
    /// Volume threshold in native units (`param / 1e8`). `0.0` means disabled.
    threshold: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToVolumeBarDerived {
    /// Direct constructor: `threshold` is the cumulative volume (native
    /// units) that closes a bar (`0.0` disables the aggregator).
    pub fn new(threshold: f64) -> Self {
        Self { threshold, current: None, last_emitted_open_time: 0, kline_state: KlineDeltaState::new(), replay: false }
    }
}

impl DerivedStream for TradeToVolumeBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let threshold = match &key.kind {
            Kind::VolumeBar(v) if *v > 0 => *v as f64 / 1e8,
            _ => 0.0,
        };
        Self::new(threshold)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.threshold == 0.0 { return None; }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToVolumeBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<BarPoint> {
        if self.current.is_none() {
            // Canonical open: the first trade's real price — genuine gaps
            // between bars are allowed, same as tick bars.
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
        }

        let bar = self.current.clone()?;

        // Roll if volume crossed threshold (crossing trade is in the closing bar).
        let rolled = bar.volume >= self.threshold;
        if rolled {
            self.current = None;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar — same convention as `TradeToRangeBarDerived`.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToFootprintDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into time-bucketed [`FootprintPoint`] bars with
/// per-price buy/sell volume breakdown.
///
/// ## Time bucketing
///
/// Uses `KlineInterval` (from `Kind::Footprint(iv)`) aligned to UTC epoch:
/// `bucket_start = (ts_ms / interval_ms) * interval_ms`. Same as `TradeToBarDerived`.
///
/// ## Per-price levels
///
/// Each trade's price maps to a `(buy_vol, sell_vol)` entry in a `BTreeMap`
/// (ordered by price). Side `0` = Buy, `1` = Sell (matches `TradePoint::side`).
///
/// ## Level grouping (`group_step`)
///
/// `group_step` (price units per row) controls level granularity:
/// - `0.0` (default) — raw/no grouping. The level key is the trade's exact
///   price, keyed by its raw `f64` bit-pattern
///   (`u64::from_le_bytes(price.to_le_bytes())`) for exact equality on
///   repeated same-price trades without float precision loss.
/// - `> 0.0` — the level key is `floor(price / group_step) * group_step`
///   (the row's lower boundary), so trades within the same `group_step`-wide
///   price band land in one row. Buy/sell attribution is preserved per row.
///
/// ## Emit semantics
///
/// The current open footprint is emitted on **every trade** via `upsert_by_ts`
/// (same contract as `TradeToBarDerived`). On bucket roll, the new bar is emitted.
///
/// ## Disabled guard
///
/// `interval_ms == 0` → always returns `None`.
pub struct TradeToFootprintDerived {
    /// Bucket width in milliseconds. `0` = disabled.
    interval_ms: i64,
    /// Price units per level row. `0.0` = raw/no grouping (default).
    group_step: f64,
    /// Current bucket start.
    current_bucket_start: i64,
    /// OHLCV of the current bucket.
    current_ohlcv: Option<(f64, f64, f64, f64, f64)>, // open, high, low, close, volume
    /// Per-price(-group) accumulator: level_key_bits → (buy_vol, sell_vol).
    levels: std::collections::BTreeMap<u64, (f64, f64)>,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. While `true`,
    /// [`DerivedStream::on_upstream_event`] still folds every trade into
    /// `current_ohlcv` / `levels` exactly as live, but skips the
    /// `build_point()` full-snapshot construction (POC-equivalent per-price
    /// row scan + levels-vector clone) on every non-rolling trade — this is
    /// the P0 perf fix: that scan is O(levels) and was previously paid on
    /// EVERY trade during replay, making cold-seed O(rows × trades).
    replay: bool,
}

impl TradeToFootprintDerived {
    fn price_bits(price: f64) -> u64 {
        u64::from_le_bytes(price.to_le_bytes())
    }

    /// Level key for a trade's price: raw price when `group_step == 0.0`,
    /// otherwise the row's lower boundary `floor(price / group_step) *
    /// group_step`.
    fn level_key(&self, price: f64) -> u64 {
        if self.group_step > 0.0 {
            let grouped = (price / self.group_step).floor() * self.group_step;
            Self::price_bits(grouped)
        } else {
            Self::price_bits(price)
        }
    }

    fn build_point(&self, open_time: i64) -> FootprintPoint {
        let (open, high, low, close, volume) = self.current_ohlcv.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));
        // Convert BTreeMap (ordered by price bits, which for positive f64 matches
        // ascending numeric order) to sorted-by-price Vec.
        let levels: Vec<(f64, f64, f64)> = self.levels.iter().map(|(bits, (buy, sell))| {
            let price = f64::from_le_bytes(bits.to_le_bytes());
            (price, *buy, *sell)
        }).collect();
        FootprintPoint { open_time, open, high, low, close, volume, levels }
    }
}

impl TradeToFootprintDerived {
    /// Direct constructor: `interval_ms` is the time-bucket width in
    /// milliseconds (`0` disables the aggregator). `group_step` is the price
    /// units per level row (`0.0` = raw/no grouping — matches the pre-grouping
    /// behavior).
    pub fn new(interval_ms: i64, group_step: f64) -> Self {
        Self {
            interval_ms,
            group_step,
            current_bucket_start: 0,
            current_ohlcv: None,
            levels: std::collections::BTreeMap::new(),
            replay: false,
        }
    }

    /// Back-compat constructor: `interval_ms` only, `group_step = 0.0`
    /// (raw/no grouping) — identical to the pre-grouping behavior.
    pub fn new_ungrouped(interval_ms: i64) -> Self {
        Self::new(interval_ms, 0.0)
    }
}

impl DerivedStream for TradeToFootprintDerived {
    type Output = FootprintPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let interval_ms = match &key.kind {
            Kind::Footprint(iv) => interval_to_ms(iv.as_str()).unwrap_or(0),
            _ => 0,
        };
        Self::new(interval_ms, 0.0)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<FootprintPoint> {
        if self.current_ohlcv.is_none() {
            return None;
        }
        Some(self.build_point(self.current_bucket_start))
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<FootprintPoint> {
        if self.interval_ms == 0 {
            if !self.replay {
                eprintln!("[dig3-fp-derived] interval_ms=0 → return None (disabled guard)");
            }
            return None;
        }
        let Event::Trade { point, .. } = ev else {
            // Not a Trade event — log first 3 only so logs don't explode.
            return None
        };

        let bucket_start = (point.ts_ms / self.interval_ms) * self.interval_ms;
        let rolled = self.current_ohlcv.is_none() || bucket_start > self.current_bucket_start;

        if rolled {
            // Roll to new bucket. Replay mode still emits here — this IS the
            // "element closes/rolls" case the contract requires (the
            // PREVIOUS bucket just finalized), so we snapshot it via
            // `build_point` BEFORE clearing state for the new bucket.
            let closed_prev = if self.replay && self.current_ohlcv.is_some() {
                Some(self.build_point(self.current_bucket_start))
            } else {
                None
            };

            self.current_bucket_start = bucket_start;
            self.current_ohlcv = Some((point.price, point.price, point.price, point.price, point.quantity));
            self.levels.clear();
            // First level entry.
            let key = self.level_key(point.price);
            let entry = self.levels.entry(key).or_insert((0.0, 0.0));
            if point.side == 0 { entry.0 += point.quantity; } else { entry.1 += point.quantity; }
            if !self.replay {
                eprintln!(
                    "[dig3-fp-derived] BUCKET ROLL ts_ms={} bucket_start={} price={:.2} side={} qty={:.6}",
                    point.ts_ms, bucket_start, point.price, point.side, point.quantity,
                );
            }

            // Replay mode: the just-closed PREVIOUS bucket is the terminal
            // element for this call — return it instead of the freshly
            // opened (single-trade) new bucket. The new bucket's own final
            // snapshot will be returned on ITS roll (or via `flush_replay`
            // at the end of the pass).
            if let Some(prev) = closed_prev {
                return Some(prev);
            }
        } else {
            // Update current bucket.
            let ohlcv = self.current_ohlcv.as_mut()?;
            if point.price > ohlcv.1 { ohlcv.1 = point.price; } // high
            if point.price < ohlcv.2 { ohlcv.2 = point.price; } // low
            ohlcv.3 = point.price; // close
            ohlcv.4 += point.quantity; // volume
            let key = self.level_key(point.price);
            let entry = self.levels.entry(key).or_insert((0.0, 0.0));
            if point.side == 0 { entry.0 += point.quantity; } else { entry.1 += point.quantity; }
        }

        // Replay mode: suppress the expensive full-snapshot build
        // (`build_point` scans every price row) on every non-rolling trade —
        // this is the P0 perf fix. The bucket roll case above already
        // returned early.
        if self.replay {
            return None;
        }

        let pt = self.build_point(self.current_bucket_start);
        // Log every 100th tick + every bucket roll so we see live cluster growth.
        if rolled || self.levels.len() % 100 == 0 {
            eprintln!(
                "[dig3-fp-derived] emit FootprintPoint open_time={} levels.len()={} vol={:.4} ohlc=[o={:.2} h={:.2} l={:.2} c={:.2}]",
                pt.open_time, pt.levels.len(), pt.volume, pt.open, pt.high, pt.low, pt.close,
            );
        }
        Some(pt)
    }
}

// ---------------------------------------------------------------------------
// TradeToRenkoBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into fixed-height Renko bricks.
///
/// ## Brick semantics
///
/// `box_size` (from `Kind::RenkoBar(b, _)`) is stored as `b / 1e8`. A new
/// brick is emitted whenever the trade price moves at least one full
/// `box_size` in the current direction.
///
/// ## Reversal back-step (canonical, TradingView rule)
///
/// A reversal brick spans FROM THE OPEN of the last placed brick, one box
/// (× `reversal_count`) in the opposite direction — price must travel back
/// to the last brick's `open`, then `reversal_count` more full boxes beyond
/// it, before the reversal fires. Equivalently: `reversal_count + 1` boxes
/// of opposing movement measured from the last brick's CLOSE. When it
/// fires, the new brick's far edge is pinned to `last_open ∓ box_size`
/// (mirrors the box direction) — NOT one box off from the last brick's
/// close-adjacent edge. Continuation bricks (same direction as `last_dir`)
/// are unaffected — they still fire at one full `box_size` of same-direction
/// movement from the running anchor.
///
/// A single trade can close MULTIPLE bricks in one `fold_trade` call (a
/// large price jump walks the anchor grid brick-by-brick). Only the LAST
/// brick closed is returned via the `Option`; every earlier brick closed in
/// the same call is queued in `pending` and MUST be drained via
/// [`DerivedStream::take_pending`] immediately after — otherwise it is
/// silently lost.
///
/// ## Output shape (`BarPoint`)
///
/// Up bricks: `open = brick_bottom, close = brick_top`. Down bricks:
/// `open = brick_top, close = brick_bottom`. `high = max`, `low = min`
/// of the brick's price span. `volume` accumulates trade volume across
/// the bar but is reset to 0 between bricks (we cannot legitimately
/// split a single trade's volume across two bricks).
///
/// ## Monotonic open_time
///
/// Same scheme as [`TradeToRangeBarDerived`].
///
/// ## Disabled guard
///
/// `box_size == 0.0` ⇒ always `None`.
pub struct TradeToRenkoBarDerived {
    /// Brick price height in native units. `0.0` ⇒ disabled.
    box_size: f64,
    /// Bricks required to flip direction (default 2 = TradeStation classic).
    reversal_count: u8,
    /// Anchor price — the price level the current direction was last at.
    /// New bricks emit when `|trade.price − anchor| ≥ box_size`.
    anchor: f64,
    /// Direction of the last emitted brick: `Some(true)` = up, `Some(false)`
    /// = down, `None` = no brick yet (seed phase).
    last_dir: Option<bool>,
    /// Volume accumulator for the in-progress brick.
    vol_acc: f64,
    /// Trade-count accumulator.
    tcount_acc: u64,
    /// Running MIN of every trade price observed while the current brick is
    /// forming (reset when a brick starts forming). Combined with
    /// `intra_max`, feeds the emitted brick's `wick` — the adverse extreme
    /// reached beyond the brick body while it was in progress.
    intra_min: f64,
    /// Running MAX of every trade price observed while the current brick is
    /// forming. See `intra_min`.
    intra_max: f64,
    /// Monotonic open_time guard — same as range/tick/volume bars.
    last_emitted_open_time: i64,
    /// True once the grid anchor has been established (either by the
    /// first-trade floor-snap or by [`Self::preset_grid_anchor`]). Blocks
    /// the floor-snap from re-firing when a rewarm-fold preset the anchor
    /// explicitly (a preset anchor may legitimately be `0.0`).
    seeded: bool,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Fully-closed intermediate bricks produced by a multi-brick move
    /// within the most recent `fold_trade` call, queued for
    /// [`DerivedStream::take_pending`] — mirrors [`TradeToRangeBarDerived::pending`].
    pending: Vec<RenkoBrickPoint>,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. Renko's
    /// `fold_trade` already returns `Some(..)` ONLY when a brick actually
    /// closes this call (organic in-brick absorption returns `None`
    /// unconditionally, live or replay) — so this flag exists solely to
    /// gate [`DerivedStream::flush_replay`]'s in-progress snapshot build.
    replay: bool,
}

impl TradeToRenkoBarDerived {
    /// Direct constructor: `box_size` is the brick price height in native
    /// units (`0.0` disables the aggregator); `reversal_count` is the
    /// number of boxes required to flip direction (floored at 1).
    pub fn new(box_size: f64, reversal_count: u8) -> Self {
        Self {
            box_size,
            reversal_count: reversal_count.max(1),
            anchor: 0.0,
            last_dir: None,
            vol_acc: 0.0,
            tcount_acc: 0,
            intra_min: f64::INFINITY,
            intra_max: f64::NEG_INFINITY,
            last_emitted_open_time: 0,
            seeded: false,
            kline_state: KlineDeltaState::new(),
            pending: Vec::new(),
            replay: false,
        }
    }

    /// Build a snapshot of the CURRENTLY-FORMING brick from the running
    /// accumulators, without closing it — used only by
    /// [`DerivedStream::flush_replay`]. Mirrors the closed-brick shape a
    /// live emission would eventually carry once the brick actually rolls;
    /// `open_time` here is provisional (the accumulator has no anchor
    /// timestamp until close), stamped as `last_emitted_open_time + 1` so it
    /// stays monotonic with whatever closed brick preceded it.
    fn in_progress_snapshot(&self) -> Option<RenkoBrickPoint> {
        if !self.seeded || self.tcount_acc == 0 {
            return None;
        }
        let up = !matches!(self.last_dir, Some(false));
        let (bottom, top) = if up {
            (self.anchor, self.anchor + self.box_size)
        } else {
            (self.anchor - self.box_size, self.anchor)
        };
        let wick = if up {
            if self.intra_min < bottom { bottom - self.intra_min } else { 0.0 }
        } else if self.intra_max > top { self.intra_max - top } else { 0.0 };
        Some(RenkoBrickPoint {
            open_time: self.last_emitted_open_time + 1,
            bottom,
            top,
            up,
            volume: self.vol_acc,
            trades_count: self.tcount_acc,
            wick,
        })
    }
}

impl DerivedStream for TradeToRenkoBarDerived {
    type Output = RenkoBrickPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (box_size, reversal_count) = match &key.kind {
            Kind::RenkoBar(b, r) if *b > 0 => (*b as f64 / 1e8, (*r).max(1)),
            _ => (0.0, 1),
        };
        Self::new(box_size, reversal_count)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn take_pending(&mut self) -> Vec<RenkoBrickPoint> {
        std::mem::take(&mut self.pending)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<RenkoBrickPoint> {
        self.in_progress_snapshot()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<RenkoBrickPoint> {
        if self.box_size == 0.0 { return None; }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToRenkoBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<RenkoBrickPoint> {
        // Seed anchor on the very first trade — skipped when the anchor
        // was already preset via `preset_grid_anchor` (rewarm-fold path).
        if !self.seeded {
            self.anchor = (point.price / self.box_size).floor() * self.box_size;
            self.seeded = true;
        }

        self.vol_acc += point.quantity;
        self.tcount_acc += 1;
        if point.price < self.intra_min { self.intra_min = point.price; }
        if point.price > self.intra_max { self.intra_max = point.price; }

        let mut last_emitted: Option<RenkoBrickPoint> = None;

        loop {
            let up_target = self.anchor + self.box_size;
            let down_target = self.anchor - self.box_size;

            if point.price >= up_target {
                if matches!(self.last_dir, Some(false)) {
                    // Reversal (down → up): the last brick was a down brick,
                    // whose OPEN is its `top` edge = `anchor + box_size`.
                    // Canonical back-step: price must travel back to that
                    // open, then `reversal_count` more full boxes beyond it.
                    let last_open = self.anchor + self.box_size;
                    let needed = last_open + self.box_size * self.reversal_count as f64;
                    if point.price < needed { break; }
                    self.anchor += self.box_size;
                }
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                let bottom = self.anchor;
                // Adverse extreme for an up brick = anything BELOW its
                // bottom edge reached while it was forming.
                let wick = if self.intra_min < bottom { bottom - self.intra_min } else { 0.0 };
                let brick = RenkoBrickPoint {
                    open_time,
                    bottom,
                    top: self.anchor + self.box_size,
                    up: true,
                    volume: self.vol_acc,
                    trades_count: self.tcount_acc,
                    wick,
                };
                self.vol_acc = 0.0;
                self.tcount_acc = 0;
                self.intra_min = f64::INFINITY;
                self.intra_max = f64::NEG_INFINITY;
                self.anchor += self.box_size;
                self.last_dir = Some(true);
                // A previously-held brick from an earlier loop iteration is
                // now fully superseded — queue it so it is not lost.
                if let Some(prior) = last_emitted.replace(brick) {
                    self.pending.push(prior);
                }
                continue;
            }

            if point.price <= down_target {
                if matches!(self.last_dir, Some(true)) {
                    // Reversal (up → down): the last brick was an up brick,
                    // whose OPEN is its `bottom` edge = `anchor − box_size`.
                    // Canonical back-step: price must travel back to that
                    // open, then `reversal_count` more full boxes beyond it.
                    let last_open = self.anchor - self.box_size;
                    let needed = last_open - self.box_size * self.reversal_count as f64;
                    if point.price > needed { break; }
                    self.anchor -= self.box_size;
                }
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                let top = self.anchor;
                // Adverse extreme for a down brick = anything ABOVE its top
                // edge reached while it was forming.
                let wick = if self.intra_max > top { self.intra_max - top } else { 0.0 };
                let brick = RenkoBrickPoint {
                    open_time,
                    bottom: self.anchor - self.box_size,
                    top,
                    up: false,
                    volume: self.vol_acc,
                    trades_count: self.tcount_acc,
                    wick,
                };
                self.vol_acc = 0.0;
                self.tcount_acc = 0;
                self.intra_min = f64::INFINITY;
                self.intra_max = f64::NEG_INFINITY;
                self.anchor -= self.box_size;
                self.last_dir = Some(false);
                // A previously-held brick from an earlier loop iteration is
                // now fully superseded — queue it so it is not lost.
                if let Some(prior) = last_emitted.replace(brick) {
                    self.pending.push(prior);
                }
                continue;
            }

            break;
        }

        last_emitted
    }
}

// ---------------------------------------------------------------------------
// TradeToPnfBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Point-and-Figure (X/O) columns.
///
/// ## Column semantics
///
/// `box_size` from `Kind::PnfBar(b, _)` stored as `b / 1e8`.
/// `reversal_count` = boxes required to start a new opposite-direction
/// column (industry default = 3, TradeStation classic).
///
/// ## Output (`PnfColumnPoint`)
///
/// Emits one column point per trade — the **current** column, updated
/// in-place. The column rolls (direction flips) when reversal_count ×
/// box_size of opposing movement is hit; the new column's first box is
/// placed one box away from the prior column's terminal box (mplfinance
/// convention).
///
/// `column_id` strictly monotonic per-Derived-instance — caller groups
/// trades into columns via column_id, doesn't need to diff `(top, bot)`.
///
/// ## Disabled guard: `box_size == 0.0`.
pub struct TradeToPnfBarDerived {
    box_size: f64,
    reversal_count: u8,
    /// Currently-open column, if any.
    cur: Option<PnfColumnPoint>,
    /// Monotonic column id counter; first column = 1.
    next_column_id: u64,
    /// Monotonic open_time guard.
    last_emitted_open_time: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. PnF has no
    /// `pending` queue (a reversal simply supersedes the old column, it is
    /// never retained) — "closes/rolls" for this machine means "a NEW
    /// column started" (seed or reversal); same-column top/bottom
    /// extensions are the in-progress-snapshot growth suppressed in replay.
    replay: bool,
}

impl TradeToPnfBarDerived {
    /// Direct constructor: `box_size` is the box price height in native
    /// units (`0.0` disables the aggregator); `reversal_count` is the
    /// number of boxes required to start a new opposite-direction column.
    pub fn new(box_size: f64, reversal_count: u8) -> Self {
        Self {
            box_size,
            reversal_count,
            cur: None,
            next_column_id: 1,
            last_emitted_open_time: 0,
            kline_state: KlineDeltaState::new(),
            replay: false,
        }
    }

}

impl DerivedStream for TradeToPnfBarDerived {
    type Output = PnfColumnPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (box_size, reversal_count) = match &key.kind {
            Kind::PnfBar(b, r) if *b > 0 => (*b as f64 / 1e8, (*r).max(1)),
            _ => (0.0, 3),
        };
        Self::new(box_size, reversal_count)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<PnfColumnPoint> {
        self.cur.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<PnfColumnPoint> {
        if self.box_size == 0.0 { return None; }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToPnfBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<PnfColumnPoint> {
        // Seed the first column from the first trade as an X column,
        // floor-snapped to the box grid.
        if self.cur.is_none() {
            let bottom = (point.price / self.box_size).floor() * self.box_size;
            let top = bottom + self.box_size;
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.cur = Some(PnfColumnPoint {
                open_time,
                column_id: self.next_column_id,
                is_x: true,
                bottom,
                top,
                volume: point.quantity,
                trades_count: 1,
            });
            self.next_column_id += 1;
            return self.cur.clone();
        }

        let col = self.cur.as_mut().unwrap();
        col.volume += point.quantity;
        col.trades_count += 1;

        if col.is_x {
            // Extend up if price clears next box top. Same column — not a
            // roll — suppressed in replay mode (the in-progress snapshot
            // this represents is superseded by the next real close/flush).
            if point.price >= col.top + self.box_size {
                let steps = ((point.price - col.top) / self.box_size).floor() as i64;
                col.top += steps as f64 * self.box_size;
                return if self.replay { None } else { self.cur.clone() };
            }
            // Reverse to O column if price drops `reversal × box_size` from top.
            if point.price <= col.top - self.box_size * self.reversal_count as f64 {
                // New O column starts one box below the prior X top.
                let new_top = col.top - self.box_size;
                let drop_steps = (((col.top - self.box_size) - point.price)
                    / self.box_size).floor() as i64 + 1;
                let new_bot = new_top - drop_steps.max(1) as f64 * self.box_size;
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                self.cur = Some(PnfColumnPoint {
                    open_time,
                    column_id: self.next_column_id,
                    is_x: false,
                    bottom: new_bot,
                    top: new_top,
                    volume: 0.0,
                    trades_count: 0,
                });
                self.next_column_id += 1;
                // Reversal — a new column started, always emitted (this IS
                // the "roll" event for PnF, live or replay).
                return self.cur.clone();
            }
        } else {
            // Extend down if price clears next box bottom. Same column —
            // suppressed in replay mode, see the up-extend branch above.
            if point.price <= col.bottom - self.box_size {
                let steps = ((col.bottom - point.price) / self.box_size).floor() as i64;
                col.bottom -= steps as f64 * self.box_size;
                return if self.replay { None } else { self.cur.clone() };
            }
            // Reverse to X column if price rises `reversal × box_size` from bottom.
            if point.price >= col.bottom + self.box_size * self.reversal_count as f64 {
                let new_bot = col.bottom + self.box_size;
                let rise_steps = ((point.price - (col.bottom + self.box_size))
                    / self.box_size).floor() as i64 + 1;
                let new_top = new_bot + rise_steps.max(1) as f64 * self.box_size;
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                self.cur = Some(PnfColumnPoint {
                    open_time,
                    column_id: self.next_column_id,
                    is_x: true,
                    bottom: new_bot,
                    top: new_top,
                    volume: 0.0,
                    trades_count: 0,
                });
                self.next_column_id += 1;
                return self.cur.clone();
            }
        }

        // Neither an extension nor a reversal — trade landed inside the
        // existing column's box with no boundary change. In replay mode
        // this is the purest in-progress no-op and is suppressed.
        if self.replay {
            return None;
        }
        self.cur.clone()
    }
}

// ---------------------------------------------------------------------------
// TradeToKagiBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Kagi segments.
///
/// ## Segment semantics
///
/// `reversal` from `Kind::KagiBar(r)` stored as `r / 1e8`. Direction stays
/// the same as long as price keeps moving in it. A reversal of at least
/// `reversal` in price closes the current segment and opens a new one in
/// the opposite direction.
///
/// ## Mid-segment thickness split (canonical)
///
/// Yang/yin flips at the PRICE where a still-forming segment crosses the
/// previous swing extreme — the prior shoulder (highest high across closed
/// up segments) for an up segment, the prior waist (lowest low across
/// closed down segments) for a down segment — not retroactively at the
/// segment's eventual close. When the live segment's running extreme
/// crosses that threshold mid-formation, the machine SPLITS it: the
/// sub-segment `[seg_start → crossing_price]` closes with the OLD
/// thickness (queued via [`DerivedStream::take_pending`] — see below), and
/// a new live sub-segment `[crossing_price → …]` continues with the
/// FLIPPED thickness. Both pieces share the same `up` direction — only
/// `yang` differs. `KagiSegmentPoint` needs no new fields for this: each
/// emitted piece already carries its own `start_price` / `end_price` /
/// `yang`.
///
/// ## Output (`KagiSegmentPoint`)
///
/// Two flavours:
/// 1. Vertical segment (or split sub-segment): `is_connector = false`,
///    `start_price`, `end_price`, `yang: bool`.
/// 2. Horizontal connector: `is_connector = true`, emitted at the close
///    of each segment (price = the segment's terminal price).
///
/// A mid-segment split can close ONE sub-segment within a single
/// `fold_trade` call — like the other chaining machines
/// ([`TradeToRangeBarDerived`], [`TradeToRenkoBarDerived`]), the earlier
/// (now-closed) piece is queued to `pending` and MUST be drained via
/// [`DerivedStream::take_pending`] immediately after every call, or it is
/// silently lost. The live (still-forming) piece is returned via the normal
/// `Option`.
///
/// ## Disabled guard: `reversal == 0.0`.
pub struct TradeToKagiBarDerived {
    reversal: f64,
    /// Anchor price for the current direction — the segment's running
    /// extreme (highest price reached in an up leg, lowest in a down leg).
    anchor: f64,
    /// True once the seed direction has been determined.
    seeded: bool,
    /// Current direction.
    up: bool,
    /// Last seen shoulder (highest high across closed UP segments).
    last_shoulder: f64,
    /// Last seen waist (lowest low across closed DOWN segments).
    last_waist: f64,
    /// True start price of the CURRENTLY-FORMING sub-segment piece (reset
    /// to the crossing price on a mid-segment split, and to the reversal
    /// trigger price on a direction flip).
    seg_start: f64,
    /// Thickness of the currently-forming sub-segment piece. Flips exactly
    /// once, in place, the moment the live segment crosses the prior
    /// shoulder/waist mid-formation.
    current_yang: bool,
    last_emitted_open_time: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Fully-closed sub-segment produced by a mid-segment thickness split
    /// within the most recent `fold_trade` call, queued for
    /// [`DerivedStream::take_pending`] — mirrors
    /// [`TradeToRangeBarDerived::pending`].
    pending: Vec<KagiSegmentPoint>,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. Kagi's
    /// `fold_trade`/`extend_and_maybe_split` already return `Some(..)` ONLY
    /// on a genuine roll (seed segment, mid-segment split, direction-flip
    /// close) — organic same-direction extension with no crossing already
    /// returns `None` unconditionally. This flag exists solely to gate
    /// [`DerivedStream::flush_replay`]'s in-progress snapshot build.
    replay: bool,
}

impl TradeToKagiBarDerived {
    /// Direct constructor: `reversal` is the minimum opposing price move
    /// (native units) that closes the current segment (`0.0` disables the
    /// aggregator).
    pub fn new(reversal: f64) -> Self {
        Self {
            reversal,
            anchor: 0.0,
            seeded: false,
            up: true,
            last_shoulder: f64::NEG_INFINITY,
            last_waist: f64::INFINITY,
            seg_start: 0.0,
            current_yang: true,
            last_emitted_open_time: 0,
            kline_state: KlineDeltaState::new(),
            pending: Vec::new(),
            replay: false,
        }
    }

    /// Build a snapshot of the CURRENTLY-FORMING (sub-)segment from the
    /// running anchor state, without closing it — used only by
    /// [`DerivedStream::flush_replay`]. Mirrors the live piece
    /// `extend_and_maybe_split` would have carried on the next trade.
    fn in_progress_snapshot(&self) -> Option<KagiSegmentPoint> {
        if !self.seeded {
            return None;
        }
        Some(KagiSegmentPoint {
            open_time: self.last_emitted_open_time + 1,
            start_price: self.seg_start,
            end_price: self.anchor,
            up: self.up,
            yang: self.current_yang,
            is_connector: false,
        })
    }
}

impl DerivedStream for TradeToKagiBarDerived {
    type Output = KagiSegmentPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let reversal = match &key.kind {
            Kind::KagiBar(r) if *r > 0 => *r as f64 / 1e8,
            _ => 0.0,
        };
        Self::new(reversal)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn take_pending(&mut self) -> Vec<KagiSegmentPoint> {
        std::mem::take(&mut self.pending)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<KagiSegmentPoint> {
        self.in_progress_snapshot()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<KagiSegmentPoint> {
        if self.reversal == 0.0 { return None; }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToKagiBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<KagiSegmentPoint> {
        // Seed phase: wait for the first significant move ≥ reversal to
        // determine direction. Do NOT pre-assume up.
        if !self.seeded {
            if self.anchor == 0.0 {
                self.anchor = point.price;
                return None;
            }
            if (point.price - self.anchor).abs() >= self.reversal {
                self.up = point.price > self.anchor;
                if self.up { self.last_waist = self.anchor; }
                else { self.last_shoulder = self.anchor; }
                self.seeded = true;
                // Emit the seed segment.
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                let prev_anchor = self.anchor;
                self.anchor = point.price;
                // The seed segment is a single closed piece (hardcoded
                // `yang: true`, no prior shoulder/waist to compare against
                // yet). The NEXT (active-phase) live piece starts thin — it
                // hasn't crossed anything yet, same convention as every
                // post-reversal leg.
                self.seg_start = point.price;
                self.current_yang = false;
                return Some(KagiSegmentPoint {
                    open_time,
                    start_price: prev_anchor,
                    end_price: point.price,
                    up: self.up,
                    yang: true,
                    is_connector: false,
                });
            }
            // Track anchor to most extreme yet-seen price so seed direction
            // is from the actual swing, not the first trade noise.
            if self.up && point.price > self.anchor { self.anchor = point.price; }
            if !self.up && point.price < self.anchor { self.anchor = point.price; }
            return None;
        }

        // Active phase — continuation in the current direction.
        if self.up && point.price > self.anchor {
            return self.extend_and_maybe_split(point);
        }
        if !self.up && point.price < self.anchor {
            return self.extend_and_maybe_split(point);
        }

        // Candidate reversal: price moved opposite by ≥ reversal.
        let against = if self.up { self.anchor - point.price }
                      else { point.price - self.anchor };
        if against < self.reversal { return None; }

        // Close current (sub-)segment piece — its start is `seg_start`
        // (either the segment's true start, or the crossing price if a
        // mid-segment split already happened on this leg) and its
        // thickness is whatever `current_yang` was left at by the last
        // mid-segment split (or the leg's initial thickness if it never
        // crossed).
        // Persist extremum — this must happen BEFORE computing the fresh
        // seed shoulder/waist for the NEW direction below.
        if self.up { self.last_shoulder = self.last_shoulder.max(self.anchor); }
        else { self.last_waist = self.last_waist.min(self.anchor); }

        let close_ts = point.ts_ms.max(self.last_emitted_open_time + 1);
        self.last_emitted_open_time = close_ts;
        let closed = KagiSegmentPoint {
            open_time: close_ts,
            start_price: self.seg_start,
            end_price: self.anchor,
            up: self.up,
            yang: self.current_yang,
            is_connector: false,
        };
        // Flip direction; new anchor is the trade that triggered the flip.
        // The new live piece starts thin (not yet crossed the — freshly
        // updated — shoulder/waist above): matches the pre-split formula
        // (`anchor > last_shoulder` / `anchor < last_waist`) evaluated at
        // the leg's own start, which is always `false` there. It flips to
        // `true` only if/when it later crosses mid-segment.
        self.up = !self.up;
        self.anchor = point.price;
        self.seg_start = point.price;
        self.current_yang = false;

        Some(closed)
    }

    /// Continuation trade that extends the live segment further in its
    /// current direction. Checks whether the running extreme crosses the
    /// prior shoulder (up) / waist (down) THIS trade — if so, splits the
    /// live piece: the sub-segment up to the crossing price closes (queued
    /// to `pending`) with the OLD thickness, and a new live sub-segment
    /// continues from the crossing price with the FLIPPED thickness.
    fn extend_and_maybe_split(&mut self, point: &TradePoint) -> Option<KagiSegmentPoint> {
        let threshold = if self.up { self.last_shoulder } else { self.last_waist };
        let already_flipped = if self.up { self.anchor > threshold } else { self.anchor < threshold };
        let now_flipped = if self.up { point.price > threshold } else { point.price < threshold };

        if !already_flipped && now_flipped && threshold.is_finite() {
            // Mid-segment crossing — split at the threshold price.
            let split_ts = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = split_ts;
            let closed_piece = KagiSegmentPoint {
                open_time: split_ts,
                start_price: self.seg_start,
                end_price: threshold,
                up: self.up,
                yang: self.current_yang,
                is_connector: false,
            };
            self.pending.push(closed_piece);

            // New live piece continues from the crossing price with the
            // flipped thickness — same polarity convention the pre-split
            // code used at close time (up: crossed shoulder ⇒ yang=true;
            // down: crossed waist ⇒ yang=true — preserved verbatim, not
            // re-derived, since fixing the classical yin/yang polarity is
            // out of scope for this change).
            self.current_yang = true;
            self.seg_start = threshold;
            self.anchor = point.price;

            let live_open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = live_open_time;
            return Some(KagiSegmentPoint {
                open_time: live_open_time,
                start_price: self.seg_start,
                end_price: point.price,
                up: self.up,
                yang: self.current_yang,
                is_connector: false,
            });
        }

        // No crossing this trade — organic extension, no emit (unchanged
        // from the pre-split behavior).
        self.anchor = point.price;
        None
    }
}

// ---------------------------------------------------------------------------
// TradeToCvdLineDerived
// ---------------------------------------------------------------------------

/// Running Cumulative Volume Delta line — scalar series indexed by trade
/// timestamp. Each trade adds `+qty` if buyer-aggressor (side==0), `-qty`
/// otherwise.
///
/// Disabled when `key.kind != Kind::CvdLine` (always-on otherwise).
///
/// ## Replay mode: intentionally NOT overridden
///
/// Every trade produces its own permanent, independent `ScalarBarPoint`
/// (its own monotonic `ts_ms`, never revisited or superseded by a later
/// event) — unlike the bar/brick/segment machines above, CVD has no
/// "in-progress, will be superseded" element at all. Each per-event sample
/// IS a closed element in its own right (the running total AT that
/// timestamp), so there is nothing for [`DerivedStream::set_replay`] to
/// suppress and nothing for [`DerivedStream::flush_replay`] to add beyond
/// what the last `on_upstream_event` call already returned. Keeps the
/// default no-op impls.
pub struct TradeToCvdLineDerived {
    enabled: bool,
    cvd: f64,
    last_emitted_ts: i64,
}

impl TradeToCvdLineDerived {
    /// Direct constructor: `enabled` gates whether the machine emits
    /// (mirrors the `key.kind == Kind::CvdLine` check `new_for_key` uses).
    /// Pass `true` for standalone client-side use.
    pub fn new(enabled: bool) -> Self {
        Self { enabled, cvd: 0.0, last_emitted_ts: 0 }
    }
}

impl DerivedStream for TradeToCvdLineDerived {
    type Output = ScalarBarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let enabled = matches!(key.kind, Kind::CvdLine);
        Self::new(enabled)
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<ScalarBarPoint> {
        if !self.enabled { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        let signed = if point.side == 0 { point.quantity } else { -point.quantity };
        self.cvd += signed;
        let ts_ms = point.ts_ms.max(self.last_emitted_ts + 1);
        self.last_emitted_ts = ts_ms;
        Some(ScalarBarPoint { ts_ms, value: self.cvd })
    }
}

// ---------------------------------------------------------------------------
// TPO — shared session-accumulator state
// ---------------------------------------------------------------------------

/// State machine shared between the two TPO derived flavours
/// (`TpoFromKline1mDerived` and `TpoFromTradeDerived`). The bucket
/// extension API is feed-source-agnostic — both flavours call
/// [`TpoSessionState::extend_bucket`] with `(ts_ms, high, low)` after
/// session-roll detection.
///
/// Maintains a ring of closed sessions alongside the current one.
/// `max_history_sessions` (default 10) caps the ring — approximately
/// two trading weeks of daily profiles. On session close, the final
/// snapshot is appended to `history` before the current session is
/// cleared. The oldest entry is evicted when the ring is full.
pub struct TpoSessionState {
    freq_minutes: u16,
    value_area_pct: f64,
    /// Current session's start (UTC midnight ms).
    session_date_ms: i64,
    /// Letter buckets: each bucket = freq_minutes window, recording
    /// (high, low) of the source events that fell in it.
    buckets: Vec<(f64, f64)>,
    /// Closes / mid-prices accumulated this session (for tick-size
    /// detection in the kline flavour, recent trade prices in the trade
    /// flavour).
    samples: Vec<f64>,
    /// Detected tick size for current session (0.0 = unset).
    tick_size: f64,
    /// Session-wide high/low.
    session_high: f64,
    session_low: f64,
    /// Ring of closed-session final snapshots, ordered oldest → newest.
    /// Capped at `max_history_sessions`.
    history: VecDeque<TpoSessionPoint>,
    /// Maximum number of closed sessions retained in `history`.
    max_history_sessions: usize,
}

impl TpoSessionState {
    fn new(freq_minutes: u16) -> Self {
        Self {
            freq_minutes,
            value_area_pct: 0.70,
            session_date_ms: 0,
            buckets: Vec::new(),
            samples: Vec::new(),
            tick_size: 0.0,
            session_high: f64::NEG_INFINITY,
            session_low: f64::INFINITY,
            history: VecDeque::new(),
            max_history_sessions: 10,
        }
    }

    fn day_start_ms(ts_ms: i64) -> i64 {
        const MS_PER_DAY: i64 = 86_400_000;
        (ts_ms / MS_PER_DAY) * MS_PER_DAY
    }

    fn bucket_idx(&self, ts_ms: i64) -> usize {
        let into_session_ms = ts_ms - self.session_date_ms;
        (into_session_ms / (self.freq_minutes as i64 * 60_000)).max(0) as usize
    }

    /// Rolls the session if `ts_ms` fell on a new UTC day. Returns the
    /// FINALIZED snapshot of the just-closed session when a roll happened
    /// AND that session had data (`None` on no-roll, or on a roll with an
    /// empty/first session — mirrors the pre-existing `has_data` guard that
    /// gated `history` population).
    fn maybe_roll_session(&mut self, ts_ms: i64) -> Option<TpoSessionPoint> {
        let bar_day = Self::day_start_ms(ts_ms);
        let mut closed_point = None;
        if bar_day != self.session_date_ms {
            // Snapshot the closing session before clearing, provided there
            // is meaningful data (at least one finite bucket).
            let has_data = self.buckets.iter().any(|(h, l)| h.is_finite() && l.is_finite());
            if has_data && self.session_date_ms != 0 {
                let closed = self.build_point();
                self.history.push_back(closed.clone());
                if self.history.len() > self.max_history_sessions {
                    self.history.pop_front();
                }
                closed_point = Some(closed);
            }
            self.session_date_ms = bar_day;
            self.buckets.clear();
            self.samples.clear();
            self.tick_size = 0.0;
            self.session_high = f64::NEG_INFINITY;
            self.session_low = f64::INFINITY;
        }
        closed_point
    }

    /// Feed one (ts_ms, high, low, price_sample) tuple into the current
    /// session. `price_sample` is used for tick-size auto-detection
    /// (kline mode: close; trade mode: trade price). Returns the FINALIZED
    /// snapshot of the just-closed session if this event rolled the
    /// session (see [`Self::maybe_roll_session`]).
    fn extend_bucket(&mut self, ts_ms: i64, high: f64, low: f64, price_sample: f64) -> Option<TpoSessionPoint> {
        let rolled = self.maybe_roll_session(ts_ms);

        if high > self.session_high { self.session_high = high; }
        if low < self.session_low { self.session_low = low; }
        self.samples.push(price_sample);
        if self.tick_size == 0.0 && self.samples.len() >= 10 {
            self.tick_size = self.detect_tick_size();
        }

        let bi = self.bucket_idx(ts_ms);
        while self.buckets.len() <= bi {
            self.buckets.push((f64::NEG_INFINITY, f64::INFINITY));
        }
        let (bhigh, blow) = &mut self.buckets[bi];
        if high > *bhigh { *bhigh = high; }
        if low < *blow { *blow = low; }

        rolled
    }

    /// `true` once at least one data point has been folded into the
    /// current (still-open) session — used by [`DerivedStream::flush_replay`]
    /// to decide whether there is anything meaningful to snapshot.
    fn has_current_session_data(&self) -> bool {
        self.buckets.iter().any(|(h, l)| h.is_finite() && l.is_finite())
    }

    fn detect_tick_size(&self) -> f64 {
        if self.samples.len() < 2 { return 0.0; }
        let mut min_step = f64::INFINITY;
        for w in self.samples.windows(2) {
            let d = (w[1] - w[0]).abs();
            if d > 0.0 && d < min_step { min_step = d; }
        }
        if min_step.is_finite() && min_step > 0.0 {
            min_step
        } else {
            ((self.session_high - self.session_low) / 200.0).max(0.01)
        }
    }

    fn build_point(&self) -> TpoSessionPoint {
        let alphabet: Vec<char> = ('A'..='Z').chain('a'..='z').collect();
        let tick = if self.tick_size > 0.0 {
            self.tick_size
        } else {
            ((self.session_high - self.session_low) / 200.0).max(0.01)
        };

        // Build the price-level → letters map.
        let n_rows = (((self.session_high - self.session_low) / tick).ceil() as usize + 1).max(1);
        let mut rows: Vec<(f64, Vec<char>)> = (0..n_rows)
            .map(|i| (self.session_low + i as f64 * tick, Vec::<char>::new()))
            .collect();

        for (bi, &(bhigh, blow)) in self.buckets.iter().enumerate() {
            if !bhigh.is_finite() || !blow.is_finite() { continue; }
            let letter = alphabet[bi % 52];
            for (price, letters) in rows.iter_mut() {
                if *price >= blow && *price < bhigh {
                    letters.push(letter);
                }
            }
        }

        // POC = row with most letters; tie → closest to midpoint.
        let max_count = rows.iter().map(|(_, l)| l.len()).max().unwrap_or(0);
        let mid = (self.session_high + self.session_low) * 0.5;
        let poc_price = rows.iter()
            .filter(|(_, l)| l.len() == max_count)
            .min_by(|(p1, _), (p2, _)| {
                (p1 - mid).abs().partial_cmp(&(p2 - mid).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(p, _)| *p)
            .unwrap_or(mid);

        // Value Area: greedy 70% expansion from POC.
        let total: usize = rows.iter().map(|(_, l)| l.len()).sum();
        let target = (total as f64 * self.value_area_pct).ceil() as usize;
        let poc_idx = rows.iter().position(|(p, _)| (*p - poc_price).abs() < tick * 0.5)
            .unwrap_or(rows.len() / 2);
        let mut acc = rows[poc_idx].1.len();
        let mut va_lo_idx = poc_idx;
        let mut va_hi_idx = poc_idx;
        while acc < target && (va_lo_idx > 0 || va_hi_idx + 1 < rows.len()) {
            let above_n = if va_hi_idx + 1 < rows.len() { rows[va_hi_idx + 1].1.len() } else { 0 };
            let below_n = if va_lo_idx > 0 { rows[va_lo_idx - 1].1.len() } else { 0 };
            if above_n >= below_n && va_hi_idx + 1 < rows.len() {
                va_hi_idx += 1;
                acc += above_n;
            } else if va_lo_idx > 0 {
                va_lo_idx -= 1;
                acc += below_n;
            } else if va_hi_idx + 1 < rows.len() {
                va_hi_idx += 1;
                acc += above_n;
            } else {
                break;
            }
        }
        let vah_price = rows[va_hi_idx].0;
        let val_price = rows[va_lo_idx].0;

        let row_letters: Vec<(f64, Vec<char>)> = rows.into_iter()
            .filter(|(_, l)| !l.is_empty())
            .collect();

        TpoSessionPoint {
            open_time: self.session_date_ms,
            tick_size: tick,
            session_high: self.session_high,
            session_low: self.session_low,
            poc_price,
            vah_price,
            val_price,
            rows: row_letters,
        }
    }
}

// ---------------------------------------------------------------------------
// TpoFromKline1mDerived  (canonical: sivamgr / py-market-profile)
// ---------------------------------------------------------------------------

/// TPO Market Profile aggregator sourced from 1-minute klines.
///
/// Subscribes to `Stream::Kline(1m)` and feeds each bar's `(high, low,
/// close)` into the shared [`TpoSessionState`]. Sessions roll on UTC
/// date change. Emits the current session's profile snapshot on every
/// incoming bar (live intraday view via Series upsert on
/// `open_time = session_date_ms`).
///
/// Subscribing to a specific `Stream::Kline(interval)` requires a
/// per-key dep set — see [`DerivedStream::deps_for_key`].
pub struct TpoFromKline1mDerived {
    state: TpoSessionState,
    /// Replay-mode flag — see [`DerivedStream::set_replay`]. While `true`,
    /// the expensive `TpoSessionState::build_point` (POC + 70% value-area
    /// scan over every price row) is invoked ONLY when `extend_bucket`
    /// reports a session roll — never on every incoming bar. This is the
    /// P0 perf fix: a 100k+-row session × 1M replayed bars was previously
    /// quadratic (full scan on EVERY bar).
    replay: bool,
}

impl TpoFromKline1mDerived {
    /// Direct constructor: `freq_minutes` is the letter-bucket width in
    /// minutes (industry default 30).
    pub fn new(freq_minutes: u16) -> Self {
        Self { state: TpoSessionState::new(freq_minutes), replay: false }
    }
}

impl DerivedStream for TpoFromKline1mDerived {
    type Output = TpoSessionPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let freq_minutes = match &key.kind {
            Kind::TpoProfile(f, TpoSource::Kline1m) if *f > 0 => *f,
            _ => 30,
        };
        Self::new(freq_minutes)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<TpoSessionPoint> {
        if !self.state.has_current_session_data() {
            return None;
        }
        Some(self.state.build_point())
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<TpoSessionPoint> {
        let Event::Bar { point, .. } = ev else { return None };
        let rolled = self.state.extend_bucket(point.open_time, point.high, point.low, point.close);
        if self.replay {
            // Suppress the per-bar full-snapshot rebuild; only the just-
            // CLOSED session (finalized by the roll) is worth the scan.
            return rolled;
        }
        Some(self.state.build_point())
    }
}

// ---------------------------------------------------------------------------
// TpoFromTradeDerived  (live tick path, no kline backfill needed)
// ---------------------------------------------------------------------------

/// TPO Market Profile aggregator sourced directly from the live trade
/// stream. Subscribes to `Stream::Trade`. Each trade extends the
/// (high, low) of its letter-period bucket using the trade price as
/// both high and low (degenerate point — bucket extremes grow as more
/// trades arrive). Tick-size auto-detection samples raw trade prices.
///
/// Faster cold start (no need to wait for 1m bars to backfill) and
/// finer-grain bucket extremes than the kline path; trade-off is more
/// upstream events on high-volume symbols.
pub struct TpoFromTradeDerived {
    state: TpoSessionState,
    /// Replay-mode flag — see [`DerivedStream::set_replay`] and the
    /// identical field on [`TpoFromKline1mDerived`]. Even more critical
    /// here: trade volume on a busy symbol is orders of magnitude higher
    /// than 1m-bar volume, so the per-event `build_point` scan this
    /// suppresses is the dominant replay cost for this machine.
    replay: bool,
}

impl TpoFromTradeDerived {
    /// Direct constructor: `freq_minutes` is the letter-bucket width in
    /// minutes (industry default 30).
    pub fn new(freq_minutes: u16) -> Self {
        Self { state: TpoSessionState::new(freq_minutes), replay: false }
    }
}

impl DerivedStream for TpoFromTradeDerived {
    type Output = TpoSessionPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let freq_minutes = match &key.kind {
            Kind::TpoProfile(f, TpoSource::TradeBucket) if *f > 0 => *f,
            _ => 30,
        };
        Self::new(freq_minutes)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<TpoSessionPoint> {
        if !self.state.has_current_session_data() {
            return None;
        }
        Some(self.state.build_point())
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<TpoSessionPoint> {
        let Event::Trade { point, .. } = ev else { return None };
        // A single trade is a degenerate `[price, price]` bar; the
        // bucket's high/low grow as subsequent trades arrive.
        let rolled = self.state.extend_bucket(point.ts_ms, point.price, point.price, point.price);
        if self.replay {
            return rolled;
        }
        Some(self.state.build_point())
    }
}

// ---------------------------------------------------------------------------
// FundingSettlementDerived
// ---------------------------------------------------------------------------

/// Monitors the `FundingRate` stream (dep index 0) and emits a
/// `FundingSettlementPoint` each time the exchange's `next_funding_time`
/// boundary is crossed — i.e., the current wall clock has passed the
/// previously-declared settlement time AND the exchange has advanced its
/// `next_funding_time` pointer to a new period.
///
/// ## Crossing logic
///
/// Fires when BOTH:
/// 1. `point.ts_ms >= self.last_next_funding_time` — time has passed the
///    previously-seen boundary.
/// 2. `point.next_funding_time_ms != self.last_next_funding_time` — exchange
///    advanced its pointer, which only happens after the settlement window
///    closes on the exchange side.
///
/// `settled_rate` carries `self.last_rate` (the rate from the *previous*
/// event), not the new rate — the rate active *during* the settled period is
/// the one from before the boundary crossed.
///
/// ## Exchanges where `next_funding_time_ms == 0`
///
/// HyperLiquid, Deribit, dYdX do not emit a settlement time on the wire.
/// All their events pass through the `new_nft == 0` guard and are silently
/// absorbed. The derived stream idles for those venues.
pub struct FundingSettlementDerived {
    /// Last seen `next_funding_time_ms`. 0 = uninitialized.
    last_next_funding_time: i64,
    /// Funding rate carried from the previous event (the one active during the
    /// *just-settled* period).
    last_rate: f64,
}

impl FundingSettlementDerived {
    /// Direct constructor. Takes no parameters — all state starts
    /// uninitialized, identical to `new_for_key`.
    pub fn new() -> Self {
        Self {
            last_next_funding_time: 0,
            last_rate: 0.0,
        }
    }
}

impl DerivedStream for FundingSettlementDerived {
    type Output = FundingSettlementPoint;

    fn deps() -> &'static [Stream] {
        &[Stream::FundingRate]
    }

    fn new_for_key(_key: &SeriesKey) -> Self {
        Self::new()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<FundingSettlementPoint> {
        let Event::FundingRate { point, .. } = ev else { return None };

        let new_nft  = point.next_funding_time_ms;
        let new_rate = point.rate;
        let now_ms   = point.ts_ms;

        // Guard: no settlement time available on this wire frame.
        if new_nft == 0 {
            self.last_rate = new_rate;
            return None;
        }

        // First event: initialize state, no emission.
        if self.last_next_funding_time == 0 {
            self.last_next_funding_time = new_nft;
            self.last_rate = new_rate;
            return None;
        }

        // Crossing condition: time passed boundary AND boundary advanced.
        let output = if now_ms >= self.last_next_funding_time
            && new_nft != self.last_next_funding_time
        {
            Some(FundingSettlementPoint {
                ts_ms:           now_ms,
                settled_rate:    self.last_rate,
                settlement_time: self.last_next_funding_time,
            })
        } else {
            None
        };

        // Always advance state.
        self.last_next_funding_time = new_nft;
        self.last_rate = new_rate;

        output
    }
}

// ---------------------------------------------------------------------------
// Suppress unused-import warnings for the concrete point types used above
// ---------------------------------------------------------------------------
// These are consumed via Event destructuring in the impls; the compiler
// may flag them as unused if it doesn't see direct type mentions.
const _: fn() = || {
    let _ = std::mem::size_of::<MarkPricePoint>();
    let _ = std::mem::size_of::<IndexPricePoint>();
    let _ = std::mem::size_of::<FundingRatePoint>();
    let _ = std::mem::size_of::<TradePoint>();
    let _ = std::mem::size_of::<BarPoint>();
    let _ = std::mem::size_of::<FootprintPoint>();
};

// ---------------------------------------------------------------------------
// TradeToDollarBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Dollar Bars (López de Prado, AFML ch.2).
///
/// ## Bar semantics
///
/// A bar closes when the running sum of `trade.price * trade.quantity` (dollar
/// value) since the last close ≥ `dollar_threshold`. The crossing trade is
/// included in the closing bar (no dollar carry-over to the next bar).
///
/// ## EMA not required
///
/// Dollar bars use a fixed threshold (unlike imbalance / run bars). No EMA
/// warm-up needed — the first bar closes as soon as `Σ(price×qty) ≥ threshold`.
///
/// ## Monotonic `open_time`
///
/// Same scheme as [`TradeToRangeBarDerived`].
///
/// ## Open semantics (canonical — same as tick bars)
///
/// `open` is the FIRST TRADE's real price of the bar — genuine gaps between
/// consecutive bars are allowed (a dollar bar is not price-path-derived like
/// range/Renko bars, so there is nothing to chain: the threshold is purely
/// dollar-value-based). This mirrors [`TradeToTickBarDerived`] and
/// [`TradeToVolumeBarDerived`].
///
/// ## Disabled guard
///
/// `threshold == 0.0` ⇒ always `None`.
pub struct TradeToDollarBarDerived {
    /// Dollar threshold in native units (`dollar_threshold` as f64). `0.0` = disabled.
    threshold: f64,
    /// Accumulated dollar value in the current bar.
    cumulative_dollars: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToDollarBarDerived {
    /// Direct constructor: `threshold` is the cumulative dollar value
    /// (`price × quantity` summed) that closes a bar (`0.0` disables the
    /// aggregator).
    pub fn new(threshold: f64) -> Self {
        Self { threshold, cumulative_dollars: 0.0, current: None, last_emitted_open_time: 0, kline_state: KlineDeltaState::new(), replay: false }
    }
}

impl DerivedStream for TradeToDollarBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[] }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let threshold = match &key.kind {
            Kind::DollarBar { dollar_threshold } if *dollar_threshold > 0 => *dollar_threshold as f64,
            _ => 0.0,
        };
        Self::new(threshold)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.threshold == 0.0 { return None; }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToDollarBarDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<BarPoint> {
        // Open bar if none.
        if self.current.is_none() {
            // Canonical open: the first trade's real price — genuine gaps
            // between bars are allowed, same as tick bars.
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
            self.cumulative_dollars = point.price * point.quantity;
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            self.cumulative_dollars += point.price * point.quantity;
        }

        let bar = self.current.clone()?;

        // Close bar when cumulative dollar value crosses threshold.
        let rolled = self.cumulative_dollars >= self.threshold;
        if rolled {
            self.current = None;
            self.cumulative_dollars = 0.0;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToTickImbalanceDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Tick Imbalance Bars (López de Prado, AFML ch.2).
///
/// ## Algorithm
///
/// Running imbalance: `θ = Σ b_t` where `b_t = +1` for buyer-initiated trades
/// (side=0) and `-1` for seller-initiated trades (side=1).
///
/// Bar closes when `|θ| ≥ E[|θ|] × E[T]`.
///
/// `E[T]` and `E[|θ|]` are updated via EMA after each bar closes:
/// - `E[T] = alpha * T_prev + (1 - alpha) * E[T]` (tick count per bar)
/// - `E[|θ|] = alpha * |θ_prev| + (1 - alpha) * E[|θ|]`
///
/// ## EMA seed
///
/// Initial `E[T] = min_ticks as f64`, `E[|θ|] = min_ticks as f64 * 0.5`.
/// With `alpha=0.20`, this means the first few bars will close at roughly
/// `min_ticks * 0.5` imbalance — a reasonable warm-up floor.
///
/// ## Disabled guard
///
/// `min_ticks == 0` ⇒ always `None`.
pub struct TradeToTickImbalanceDerived {
    /// EMA smoothing factor. `0.0` ⇒ disabled.
    alpha: f64,
    /// Sanity floor tick count (also used as EMA seed).
    min_ticks: u32,
    /// Running tick imbalance for the current bar.
    theta: f64,
    /// Number of ticks accumulated in the current bar.
    tick_count: u32,
    /// EMA of `|theta|` at bar close (updated after each closed bar).
    expected_theta_abs: f64,
    /// EMA of tick count per bar (updated after each closed bar).
    expected_ticks: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToTickImbalanceDerived {
    /// Direct constructor: `alpha` is the EMA smoothing factor (`0.0`
    /// disables the aggregator); `min_ticks` is the sanity-floor tick
    /// count, also used to seed `E[T]` and `E[|θ|]` (`E[T] = min_ticks`,
    /// `E[|θ|] = min_ticks × 0.5`) — identical seeding to `new_for_key`.
    pub fn new(alpha: f64, min_ticks: u32) -> Self {
        let expected_ticks = min_ticks as f64;
        let expected_theta_abs = min_ticks as f64 * 0.5;
        Self {
            alpha, min_ticks, theta: 0.0, tick_count: 0,
            expected_theta_abs, expected_ticks,
            current: None, last_emitted_open_time: 0, replay: false,
        }
    }

    /// Dynamic close threshold: `E[|θ|] × E[T]`, floored at `min_ticks / 2`.
    fn close_threshold(&self) -> f64 {
        let dyn_threshold = self.expected_theta_abs * self.expected_ticks;
        dyn_threshold.max(self.min_ticks as f64 * 0.5)
    }
}

impl DerivedStream for TradeToTickImbalanceDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (alpha, min_ticks) = match &key.kind {
            Kind::TickImbalanceBar { alpha_x100, min_ticks } if *min_ticks > 0 => {
                (*alpha_x100 as f64 / 100.0, *min_ticks)
            }
            _ => (0.0, 0),
        };
        Self::new(alpha, min_ticks)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.alpha == 0.0 || self.min_ticks == 0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        // b_t: side=0 (Buy) → +1, side=1 (Sell) → -1.
        let bt = if point.side == 0 { 1.0f64 } else { -1.0f64 };

        // Open bar if none.
        if self.current.is_none() {
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
            self.theta = bt;
            self.tick_count = 1;
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            self.theta += bt;
            self.tick_count += 1;
        }

        let bar = self.current.clone()?;

        // Close bar when |θ| ≥ threshold.
        let rolled = self.theta.abs() >= self.close_threshold();
        if rolled {
            // Update EMAs from the closed bar.
            let t = self.tick_count as f64;
            let theta_abs = self.theta.abs();
            self.expected_ticks = self.alpha * t + (1.0 - self.alpha) * self.expected_ticks;
            self.expected_theta_abs = self.alpha * theta_abs + (1.0 - self.alpha) * self.expected_theta_abs;
            // Reset bar state.
            self.current = None;
            self.theta = 0.0;
            self.tick_count = 0;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToVolumeImbalanceDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Volume Imbalance Bars (López de Prado, AFML ch.2).
///
/// ## Algorithm
///
/// Same as [`TradeToTickImbalanceDerived`] but uses signed volume instead of unit ticks:
/// `θ = Σ b_t × trade.quantity` where `b_t = +1` for buyer-initiated, `-1` for seller.
///
/// Bar closes when `|θ| ≥ E[|θ|] × E[T]`.
///
/// ## EMA seed
///
/// Same convention as [`TradeToTickImbalanceDerived`].
pub struct TradeToVolumeImbalanceDerived {
    alpha: f64,
    min_ticks: u32,
    /// Running signed-volume imbalance for the current bar.
    theta: f64,
    tick_count: u32,
    expected_theta_abs: f64,
    expected_ticks: f64,
    current: Option<BarPoint>,
    last_emitted_open_time: i64,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToVolumeImbalanceDerived {
    /// Direct constructor: `alpha` is the EMA smoothing factor (`0.0`
    /// disables the aggregator); `min_ticks` is the sanity-floor tick
    /// count, also used to seed `E[T]` and `E[|θ|]` (`E[T] = min_ticks`,
    /// `E[|θ|] = min_ticks × 0.5`) — identical seeding to `new_for_key`.
    pub fn new(alpha: f64, min_ticks: u32) -> Self {
        let expected_ticks = min_ticks as f64;
        // EMA seed for signed-volume: assume average trade size ≈ 1.0, so
        // E[|θ|] seed = min_ticks * 0.5 × 1.0 = min_ticks * 0.5.
        let expected_theta_abs = min_ticks as f64 * 0.5;
        Self {
            alpha, min_ticks, theta: 0.0, tick_count: 0,
            expected_theta_abs, expected_ticks,
            current: None, last_emitted_open_time: 0, replay: false,
        }
    }

    fn close_threshold(&self) -> f64 {
        let dyn_threshold = self.expected_theta_abs * self.expected_ticks;
        dyn_threshold.max(self.min_ticks as f64 * 0.5)
    }
}

impl DerivedStream for TradeToVolumeImbalanceDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (alpha, min_ticks) = match &key.kind {
            Kind::VolumeImbalanceBar { alpha_x100, min_ticks } if *min_ticks > 0 => {
                (*alpha_x100 as f64 / 100.0, *min_ticks)
            }
            _ => (0.0, 0),
        };
        Self::new(alpha, min_ticks)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.alpha == 0.0 || self.min_ticks == 0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        let bt_vol = if point.side == 0 { point.quantity } else { -point.quantity };

        if self.current.is_none() {
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
            self.theta = bt_vol;
            self.tick_count = 1;
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            self.theta += bt_vol;
            self.tick_count += 1;
        }

        let bar = self.current.clone()?;

        let rolled = self.theta.abs() >= self.close_threshold();
        if rolled {
            let t = self.tick_count as f64;
            let theta_abs = self.theta.abs();
            self.expected_ticks = self.alpha * t + (1.0 - self.alpha) * self.expected_ticks;
            self.expected_theta_abs = self.alpha * theta_abs + (1.0 - self.alpha) * self.expected_theta_abs;
            self.current = None;
            self.theta = 0.0;
            self.tick_count = 0;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToRunBarDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Run Bars (López de Prado, AFML ch.2).
///
/// ## Algorithm
///
/// Track the length of consecutive buy-runs and sell-runs. A run is the longest
/// streak of same-side ticks at the current bar boundary:
/// - When `side == 0` (Buy) and the last side was also Buy → increment `run_buy_len`.
/// - When `side == 1` (Sell) and the last side was also Sell → increment `run_sell_len`.
/// - On a side flip → reset the opposite run to 0.
///
/// Bar closes when `max(run_buy_len, run_sell_len) ≥ E[run] × E[T]`.
///
/// `E[run]` and `E[T]` are updated via EMA after each bar closes.
///
/// ## EMA seed
///
/// `E[T] = min_ticks`, `E[run] = min_ticks × 0.25` (a run is expected to be
/// about 1/4 of the bar's tick count on a balanced market).
///
/// ## Disabled guard
///
/// `min_ticks == 0` ⇒ always `None`.
pub struct TradeToRunBarDerived {
    alpha: f64,
    min_ticks: u32,
    /// Length of the current buy run (consecutive buy ticks).
    run_buy_len: u32,
    /// Length of the current sell run (consecutive sell ticks).
    run_sell_len: u32,
    /// Side of the last trade processed (0 = Buy, 1 = Sell, u8::MAX = none).
    last_side: u8,
    /// EMA of `max(run_buy, run_sell)` at bar close.
    expected_run: f64,
    /// EMA of tick count per bar.
    expected_ticks: f64,
    /// Number of ticks in the current bar.
    tick_count: u32,
    current: Option<BarPoint>,
    last_emitted_open_time: i64,
    /// Replay-mode flag — see [`DerivedStream::set_replay`].
    replay: bool,
}

impl TradeToRunBarDerived {
    /// Direct constructor: `alpha` is the EMA smoothing factor (`0.0`
    /// disables the aggregator); `min_ticks` is the sanity-floor tick
    /// count, also used to seed `E[T]` and `E[run]` (`E[T] = min_ticks`,
    /// `E[run] = min_ticks × 0.25`) — identical seeding to `new_for_key`.
    pub fn new(alpha: f64, min_ticks: u32) -> Self {
        let expected_ticks = min_ticks as f64;
        // EMA seed: expected run ≈ 25% of bar tick count on a balanced market.
        let expected_run = min_ticks as f64 * 0.25;
        Self {
            alpha, min_ticks, run_buy_len: 0, run_sell_len: 0, last_side: u8::MAX,
            expected_run, expected_ticks, tick_count: 0,
            current: None, last_emitted_open_time: 0, replay: false,
        }
    }

    fn close_threshold(&self) -> f64 {
        let dyn_threshold = self.expected_run * self.expected_ticks;
        // Floor: at least 2 ticks of run length.
        dyn_threshold.max(2.0)
    }
}

impl DerivedStream for TradeToRunBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (alpha, min_ticks) = match &key.kind {
            Kind::RunBar { alpha_x100, min_ticks } if *min_ticks > 0 => {
                (*alpha_x100 as f64 / 100.0, *min_ticks)
            }
            _ => (0.0, 0),
        };
        Self::new(alpha, min_ticks)
    }

    fn set_replay(&mut self, on: bool) {
        self.replay = on;
    }

    fn flush_replay(&mut self) -> Option<BarPoint> {
        self.current.clone()
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.alpha == 0.0 || self.min_ticks == 0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        // Update run lengths.
        if point.side == 0 {
            // Buy tick.
            if self.last_side == 0 {
                self.run_buy_len = self.run_buy_len.saturating_add(1);
            } else {
                self.run_buy_len = 1;
                self.run_sell_len = 0;
            }
        } else {
            // Sell tick.
            if self.last_side == 1 {
                self.run_sell_len = self.run_sell_len.saturating_add(1);
            } else {
                self.run_sell_len = 1;
                self.run_buy_len = 0;
            }
        }
        self.last_side = point.side;

        // Open bar if none.
        if self.current.is_none() {
            let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
            self.last_emitted_open_time = open_time;
            self.current = Some(BarPoint {
                open_time,
                open:        point.price,
                high:        point.price,
                low:         point.price,
                close:       point.price,
                volume:      point.quantity,
                quote_volume: point.price * point.quantity,
                trades_count: 1,
            });
            self.tick_count = 1;
        } else {
            let bar = self.current.as_mut()?;
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            self.tick_count += 1;
        }

        let bar = self.current.clone()?;

        let max_run = self.run_buy_len.max(self.run_sell_len) as f64;
        let rolled = max_run >= self.close_threshold();
        if rolled {
            let t = self.tick_count as f64;
            self.expected_ticks = self.alpha * t + (1.0 - self.alpha) * self.expected_ticks;
            self.expected_run = self.alpha * max_run + (1.0 - self.alpha) * self.expected_run;
            // Reset bar state but NOT run lengths — runs carry across bar boundaries.
            self.current = None;
            self.tick_count = 0;
        }

        // Replay mode: suppress the in-progress snapshot unless this trade
        // just closed the bar.
        if self.replay && !rolled {
            return None;
        }
        Some(bar)
    }
}

// ---------------------------------------------------------------------------
// TradeToThreeLineBreakDerived
// ---------------------------------------------------------------------------

/// Aggregates `Trade` events into Three Line Break (san-sen-ashi) lines.
///
/// ## Algorithm (Steve Nison, "Beyond Candlesticks" ch. 6)
///
/// State tracks the last `lines_back` (default 3) closed lines. Each trade
/// updates the "pending" close price and tries to emit a new line:
///
/// * No lines yet — the first line is printed as soon as the close differs
///   from the seed open (up if `close > open`, down if `close < open`).
/// * Current direction **up** (last line is up):
///   - New UP line if `close > max(close of last N lines)`.
///   - Reversal (DOWN line) if `close < min(low of last N lines)` where
///     `low = min(open, close)` per line.
/// * Symmetric for **down**.
///
/// ## Output (`ThreeLineBreakLinePoint`)
///
/// `open` = previous line's close (or seed open for the very first line).
/// `close` = trade price that triggered the line.
/// `high = max(open, close)`, `low = min(open, close)` (no wicks by
/// construction — the algorithm only uses closes for line boundaries).
/// `direction`: 0 = up, 1 = down.
/// `volume`: cumulative trade quantity during the line.
///
/// ## Disabled guard
///
/// `lines_back == 0` ⇒ always `None`.
pub struct TradeToThreeLineBreakDerived {
    /// How many recent lines to compare against for breakout / reversal.
    lines_back: usize,
    /// Ring of the last `lines_back` closed lines.
    last_lines: std::collections::VecDeque<ThreeLineBreakLinePoint>,
    /// Open price of the line currently forming. `None` until first trade.
    current_open: Option<f64>,
    /// Timestamp of the trade that set `current_open`.
    current_open_ts: i64,
    /// Running close price — updated every trade.
    current_close: f64,
    /// Volume accumulator for the in-progress line.
    current_volume: f64,
    /// Monotonic open_ts guard (same scheme as range/tick bars).
    last_emitted_open_ts: i64,
    /// Live Δ-tick adapter over the `Stream::Kline("1m")` upstream — see
    /// [`KlineDeltaState`]. Kline-sufficient kind: no `Stream::Trade` dependency.
    kline_state: KlineDeltaState,
}

impl TradeToThreeLineBreakDerived {
    /// Direct constructor: `lines_back` is how many recent lines to
    /// compare against for breakout / reversal (industry default 3).
    pub fn new(lines_back: usize) -> Self {
        Self {
            lines_back,
            last_lines: std::collections::VecDeque::with_capacity(lines_back + 1),
            current_open: None,
            current_open_ts: 0,
            current_close: 0.0,
            current_volume: 0.0,
            last_emitted_open_ts: 0,
            kline_state: KlineDeltaState::new(),
        }
    }

    /// Build a snapshot of the CURRENTLY-FORMING line from the running
    /// open/close/volume accumulators, without closing it — used only by
    /// [`DerivedStream::flush_replay`]. `on_upstream_event` already returns
    /// `Some(..)` ONLY when a line genuinely closes (via `emit_line`) —
    /// organic no-break trades return `None` unconditionally, live or
    /// replay — so this machine needs no `replay` flag of its own.
    fn in_progress_snapshot(&self) -> Option<ThreeLineBreakLinePoint> {
        let open = self.current_open?;
        Some(ThreeLineBreakLinePoint {
            ts_open: self.current_open_ts.max(self.last_emitted_open_ts + 1),
            ts_close: self.current_open_ts,
            open,
            close: self.current_close,
            volume: self.current_volume,
            direction: u8::from(self.current_close < open),
        })
    }

    /// Emit a line and update state.
    ///
    /// Returns the closed `ThreeLineBreakLinePoint`.
    fn emit_line(&mut self, direction: u8, ts_close: i64) -> ThreeLineBreakLinePoint {
        let open = self.current_open.unwrap_or(self.current_close);
        let close = self.current_close;
        let ts_open = self.current_open_ts.max(self.last_emitted_open_ts + 1);
        self.last_emitted_open_ts = ts_open;

        let point = ThreeLineBreakLinePoint {
            ts_open,
            ts_close,
            open,
            close,
            volume: self.current_volume,
            direction,
        };

        // Push to ring; evict oldest if at capacity.
        self.last_lines.push_back(point.clone());
        if self.last_lines.len() > self.lines_back {
            self.last_lines.pop_front();
        }

        // Next line opens from this line's close.
        self.current_open = Some(close);
        self.current_open_ts = ts_close;
        self.current_volume = 0.0;

        point
    }
}

impl DerivedStream for TradeToThreeLineBreakDerived {
    type Output = ThreeLineBreakLinePoint;

    fn deps() -> &'static [Stream] {
        &[]
    }

    fn deps_for_key(_key: &SeriesKey) -> Vec<Stream> {
        vec![Stream::Kline(KlineInterval::new("1m"))]
    }

    fn new_for_key(key: &SeriesKey) -> Self {
        let lines_back = match &key.kind {
            Kind::ThreeLineBreak { lines_back } if *lines_back > 0 => *lines_back as usize,
            _ => 3,
        };
        Self::new(lines_back)
    }

    fn seed_kline_baseline(&mut self, open_time: i64, volume: f64, close: f64) {
        self.kline_state.seed_baseline(open_time, volume, close);
    }

    fn flush_replay(&mut self) -> Option<ThreeLineBreakLinePoint> {
        self.in_progress_snapshot()
    }

    fn on_upstream_event(
        &mut self,
        ev: &Event,
        _dep_idx: usize,
    ) -> Option<ThreeLineBreakLinePoint> {
        if self.lines_back == 0 {
            return None;
        }
        match ev {
            Event::Trade { point, .. } => self.fold_trade(point),
            Event::Bar { point: bar, timeframe, exchange, symbol } => {
                let synth = self.kline_state.to_delta_trade(bar, timeframe, *exchange, symbol)?;
                let Event::Trade { point, .. } = &synth else { unreachable!("to_delta_trade always returns Event::Trade") };
                self.fold_trade(point)
            }
            _ => None,
        }
    }
}

impl TradeToThreeLineBreakDerived {
    fn fold_trade(&mut self, point: &TradePoint) -> Option<ThreeLineBreakLinePoint> {
        // Seed: record the very first trade as current open.
        if self.current_open.is_none() {
            self.current_open = Some(point.price);
            self.current_open_ts = point.ts_ms;
            self.current_close = point.price;
            self.current_volume += point.quantity;
            return None;
        }

        self.current_close = point.price;
        self.current_volume += point.quantity;

        // First line — no prior lines yet.
        if self.last_lines.is_empty() {
            let seed_open = self.current_open.unwrap_or(self.current_close);
            if (self.current_close - seed_open).abs() < f64::EPSILON {
                return None; // price hasn't moved yet
            }
            let direction = if self.current_close > seed_open { 0u8 } else { 1u8 };
            return Some(self.emit_line(direction, point.ts_ms));
        }

        // Compute the high and low of the last N lines.
        // high of a line = max(open, close); low = min(open, close).
        let max_high = self
            .last_lines
            .iter()
            .map(|l| l.open.max(l.close))
            .fold(f64::NEG_INFINITY, f64::max);
        let min_low = self
            .last_lines
            .iter()
            .map(|l| l.open.min(l.close))
            .fold(f64::INFINITY, f64::min);

        let last_dir = self.last_lines.back().map(|l| l.direction).unwrap_or(0);

        match last_dir {
            // Last line was UP → look for continuation up or reversal down.
            0 => {
                if self.current_close > max_high {
                    return Some(self.emit_line(0, point.ts_ms));
                }
                if self.current_close < min_low {
                    return Some(self.emit_line(1, point.ts_ms));
                }
            }
            // Last line was DOWN → look for continuation down or reversal up.
            _ => {
                if self.current_close < min_low {
                    return Some(self.emit_line(1, point.ts_ms));
                }
                if self.current_close > max_high {
                    return Some(self.emit_line(0, point.ts_ms));
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Kline-approx synthetic trade path (deep seed for price-path-triggered
// derived kinds: Renko / PnF / Kagi / Three-Line-Break)
// ---------------------------------------------------------------------------

/// Convert one 1m kline bar into up to 4 synthetic `Event::Trade`s tracing
/// the bar's O→H→L→C price path, for feeding into a price-path-triggered
/// derived state machine (`TradeToRenkoBarDerived` / `TradeToPnfBarDerived` /
/// `TradeToKagiBarDerived` / `TradeToThreeLineBreakDerived`) as a lossy-but-
/// honest approximation of the real tick path from weeks-old history (which
/// aggTrades cannot cover at that depth — see design doc "Seed strategy per
/// kind").
///
/// Path rule (non-standard-bars-spec):
/// - Bullish (`close >= open`): Open → Low → High → Close.
/// - Bearish (`close < open`): Open → High → Low → Close.
/// - 4 timestamps evenly spaced across `[open_time, open_time + interval_ms)`.
/// - Volume split evenly across the 4 synthetic legs (`bar.volume / 4`).
/// - `side` is a best-effort direction guess (0 = buy on rising leg, 1 = sell
///   on falling leg) — price-path derived kinds don't consume `side`, so
///   this is cosmetic only.
///
/// Returns exactly 4 events (interval_ms > 0), or an empty Vec if
/// `interval_ms <= 0` (malformed caller input — never happens for the "1m"
/// caller in `station.rs`, guarded defensively here).
pub fn kline_to_synthetic_trades(
    bar: &BarPoint,
    interval_ms: i64,
    exchange: digdigdig3::core::types::ExchangeId,
    symbol: &str,
) -> Vec<Event> {
    if interval_ms <= 0 {
        return Vec::new();
    }
    let bullish = bar.close >= bar.open;
    let path: [f64; 4] = if bullish {
        [bar.open, bar.low, bar.high, bar.close]
    } else {
        [bar.open, bar.high, bar.low, bar.close]
    };
    let leg_volume = bar.volume / 4.0;
    let step_ms = interval_ms / 4;

    (0..4)
        .map(|i| {
            let price = path[i];
            let side = if i == 0 {
                u8::from(!bullish)
            } else {
                u8::from(path[i] < path[i - 1])
            };
            Event::Trade {
                exchange,
                symbol: symbol.to_string(),
                point: TradePoint {
                    ts_ms: bar.open_time + step_ms * i as i64,
                    price,
                    quantity: leg_volume,
                    side,
                    trade_id_hash: 0,
                },
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests (inside module — need access to pub(crate) types)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use digdigdig3::core::types::ExchangeId;
    use digdigdig3::core::types::AccountType;
    use digdigdig3::core::websocket::KlineInterval;

    // Helper constructors for Event variants.
    fn mark_price_event(ts_ms: i64, mark: f64) -> Event {
        Event::MarkPrice {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: MarkPricePoint { ts_ms, mark, index: f64::NAN },
        }
    }

    fn index_price_event(ts_ms: i64, price: f64) -> Event {
        Event::IndexPrice {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: IndexPricePoint { ts_ms, price },
        }
    }

    fn funding_rate_event(ts_ms: i64, rate: f64, next_funding_time_ms: i64) -> Event {
        Event::FundingRate {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: FundingRatePoint { ts_ms, rate, next_funding_time_ms },
        }
    }

    fn test_key() -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT", crate::series::Kind::Basis)
    }

    // --- BasisDerived ---

    #[test]
    fn basis_no_emit_until_both_sides_seen() {
        let mut d = BasisDerived::new_for_key(&test_key());
        // Only mark — no emit.
        let r = d.on_upstream_event(&mark_price_event(1000, 50_000.0), 0);
        assert!(r.is_none(), "MarkPrice alone must not emit");
        // Only index — no emit (mark cached, but this is dep_idx=1 first call).
        let mut d2 = BasisDerived::new_for_key(&test_key());
        let r2 = d2.on_upstream_event(&index_price_event(1000, 49_990.0), 1);
        assert!(r2.is_none(), "IndexPrice alone must not emit");
    }

    #[test]
    fn basis_emits_on_paired_events() {
        let mut d = BasisDerived::new_for_key(&test_key());
        let r1 = d.on_upstream_event(&mark_price_event(1000, 50_000.0), 0);
        assert!(r1.is_none());
        let r2 = d.on_upstream_event(&index_price_event(1200, 49_990.0), 1);
        let p = r2.expect("should emit after both sides seen");
        assert!((p.value - 10.0).abs() < 1e-9, "value = mark - index = 10.0");
        assert_eq!(p.mark, 50_000.0);
        assert_eq!(p.index, 49_990.0);
        assert_eq!(p.ts_ms, 1200); // max(1000, 1200)
    }

    #[test]
    fn basis_skew_rejection() {
        let mut d = BasisDerived::new_for_key(&test_key());
        d.on_upstream_event(&mark_price_event(0, 50_000.0), 0);
        // Index arrives 3 seconds later — skew > 2000 ms.
        let r = d.on_upstream_event(&index_price_event(3000, 49_990.0), 1);
        assert!(r.is_none(), "stale pair must be rejected (skew > 2000 ms)");
    }

    #[test]
    fn basis_emits_on_each_update_once_seeded() {
        let mut d = BasisDerived::new_for_key(&test_key());
        d.on_upstream_event(&mark_price_event(1000, 50_000.0), 0);
        d.on_upstream_event(&index_price_event(1001, 49_990.0), 1);
        // Third event — MarkPrice update within skew.
        let r = d.on_upstream_event(&mark_price_event(1002, 50_010.0), 0);
        let p = r.expect("should emit after update when both seeded");
        assert!((p.value - 20.0).abs() < 1e-9, "updated mark=50010, index=49990 → 20.0");
    }

    #[test]
    fn basis_value_correct() {
        let mut d = BasisDerived::new_for_key(&test_key());
        d.on_upstream_event(&mark_price_event(100, 50_000.0), 0);
        let p = d.on_upstream_event(&index_price_event(100, 49_990.0), 1).unwrap();
        assert!((p.value - 10.0).abs() < 1e-9);
        assert_eq!(p.mark, 50_000.0);
        assert_eq!(p.index, 49_990.0);
    }

    // --- FundingSettlementDerived ---

    fn fs_key() -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT", crate::series::Kind::FundingSettlement)
    }

    #[test]
    fn settlement_no_emit_on_first_event() {
        let mut d = FundingSettlementDerived::new_for_key(&fs_key());
        let r = d.on_upstream_event(&funding_rate_event(500, 0.0001, 1000), 0);
        assert!(r.is_none(), "first event must only initialize state");
    }

    #[test]
    fn settlement_no_emit_if_nft_unchanged() {
        let mut d = FundingSettlementDerived::new_for_key(&fs_key());
        // Seed state: nft=1000.
        d.on_upstream_event(&funding_rate_event(500, 0.0001, 1000), 0);
        // Second event: ts still before nft, nft unchanged → no emit.
        let r = d.on_upstream_event(&funding_rate_event(800, 0.0001, 1000), 0);
        assert!(r.is_none(), "no crossing: nft unchanged and ts < nft");
    }

    #[test]
    fn settlement_emit_on_crossing() {
        let mut d = FundingSettlementDerived::new_for_key(&fs_key());
        // Seed: nft=1000, rate=0.0001.
        d.on_upstream_event(&funding_rate_event(500, 0.0001, 1000), 0);
        // Crossing: ts=1001 >= nft=1000, nft advanced to 2000.
        let r = d.on_upstream_event(&funding_rate_event(1001, 0.0002, 2000), 0);
        let p = r.expect("must emit on crossing");
        assert_eq!(p.ts_ms, 1001);
        assert!((p.settled_rate - 0.0001).abs() < 1e-12, "settled_rate must be from PREVIOUS event");
        assert_eq!(p.settlement_time, 1000);
    }

    #[test]
    fn settlement_no_emit_when_nft_zero() {
        let mut d = FundingSettlementDerived::new_for_key(&fs_key());
        let r = d.on_upstream_event(&funding_rate_event(1000, 0.0001, 0), 0);
        assert!(r.is_none(), "nft=0 must be silently absorbed");
    }

    #[test]
    fn settlement_rate_is_from_previous_event() {
        let mut d = FundingSettlementDerived::new_for_key(&fs_key());
        // Seed with rate=0.05.
        d.on_upstream_event(&funding_rate_event(500, 0.05, 1000), 0);
        // Trigger crossing with new_rate=0.03.
        let p = d.on_upstream_event(&funding_rate_event(1001, 0.03, 2000), 0).unwrap();
        assert!((p.settled_rate - 0.05).abs() < 1e-12, "settled_rate must be 0.05 (from prior event), not 0.03");
    }

    // -----------------------------------------------------------------------
    // interval_to_ms unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn interval_to_ms_known_intervals() {
        assert_eq!(interval_to_ms("1s"),  Some(1_000));
        assert_eq!(interval_to_ms("3s"),  Some(3_000));
        assert_eq!(interval_to_ms("5s"),  Some(5_000));
        assert_eq!(interval_to_ms("10s"), Some(10_000));
        assert_eq!(interval_to_ms("15s"), Some(15_000));
        assert_eq!(interval_to_ms("30s"), Some(30_000));
        assert_eq!(interval_to_ms("1m"),  Some(60_000));
        assert_eq!(interval_to_ms("3m"),  Some(3 * 60_000));
        assert_eq!(interval_to_ms("5m"),  Some(5 * 60_000));
        assert_eq!(interval_to_ms("15m"), Some(15 * 60_000));
        assert_eq!(interval_to_ms("30m"), Some(30 * 60_000));
        assert_eq!(interval_to_ms("1h"),  Some(3_600_000));
        assert_eq!(interval_to_ms("2h"),  Some(2 * 3_600_000));
        assert_eq!(interval_to_ms("4h"),  Some(4 * 3_600_000));
        assert_eq!(interval_to_ms("6h"),  Some(6 * 3_600_000));
        assert_eq!(interval_to_ms("8h"),  Some(8 * 3_600_000));
        assert_eq!(interval_to_ms("12h"), Some(12 * 3_600_000));
        assert_eq!(interval_to_ms("1d"),  Some(86_400_000));
        assert_eq!(interval_to_ms("3d"),  Some(3 * 86_400_000));
        assert_eq!(interval_to_ms("1w"),  Some(7 * 86_400_000));
    }

    #[test]
    fn interval_to_ms_unknown() {
        assert!(interval_to_ms("").is_none());
        assert!(interval_to_ms("1x").is_none());
        assert!(interval_to_ms("abc").is_none());
        assert!(interval_to_ms("0m").is_none());
        assert!(interval_to_ms("-1m").is_none());
    }

    // -----------------------------------------------------------------------
    // TradeToBarDerived unit tests
    // -----------------------------------------------------------------------

    fn kline_key(interval: &str) -> SeriesKey {
        SeriesKey::new(
            ExchangeId::Binance,
            AccountType::FuturesCross,
            "BTCUSDT",
            crate::series::Kind::Kline(KlineInterval::new(interval)),
        )
    }

    fn trade_event(ts_ms: i64, price: f64, quantity: f64) -> Event {
        Event::Trade {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: crate::data::TradePoint {
                ts_ms,
                price,
                quantity,
                side: 0,
                trade_id_hash: 0,
            },
        }
    }

    /// `Event::Bar` fixture for the kline-sufficient kinds' live Δ-tick
    /// adapter tests (Renko/PnF/Kagi/3LB/Range/Volume/Dollar) — a WS 1m
    /// kline update with `open` and `high`/`low` pinned to `close` (the
    /// folds under test only read `open_time` / `close` / `volume` via
    /// `KlineDeltaState::to_delta_trade`).
    fn bar_event(open_time: i64, close: f64, volume: f64) -> Event {
        Event::Bar {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            timeframe: KlineInterval::new("1m"),
            point: BarPoint {
                open_time,
                open: close,
                high: close,
                low: close,
                close,
                volume,
                quote_volume: close * volume,
                trades_count: 1,
            },
        }
    }

    /// Trades within the same 1m bucket produce one bar whose OHLCV reflects
    /// all trades; a trade in the next bucket opens a new bar.
    #[test]
    fn trade_to_bar_bucketing_1m() {
        let key = kline_key("1m");
        let interval_ms = 60_000_i64;
        let mut d = TradeToBarDerived::new_for_key(&key);

        // t=0 — first trade, opens bucket [0, 60000).
        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0)
            .expect("first trade must emit bar");
        assert_eq!(p1.open_time, 0);
        assert_eq!(p1.open, 100.0);
        assert_eq!(p1.high, 100.0);
        assert_eq!(p1.low,  100.0);
        assert_eq!(p1.close, 100.0);
        assert!((p1.volume - 1.0).abs() < 1e-12);
        assert_eq!(p1.trades_count, 1);

        // t=30_000 — same bucket, higher price.
        let p2 = d.on_upstream_event(&trade_event(30_000, 120.0, 2.0), 0)
            .expect("second trade must emit updated bar");
        assert_eq!(p2.open_time, 0, "same bucket — open_time must not change");
        assert_eq!(p2.open,  100.0, "open must be first trade price");
        assert_eq!(p2.high,  120.0, "high must update to 120");
        assert_eq!(p2.low,   100.0, "low stays at 100");
        assert_eq!(p2.close, 120.0, "close is most recent price");
        assert!((p2.volume - 3.0).abs() < 1e-12);
        assert_eq!(p2.trades_count, 2);

        // t=59_999 — still same bucket, lower price.
        let p3 = d.on_upstream_event(&trade_event(59_999, 90.0, 0.5), 0)
            .expect("third trade must emit");
        assert_eq!(p3.open_time, 0);
        assert_eq!(p3.low, 90.0, "new minimum");
        assert_eq!(p3.close, 90.0);
        assert_eq!(p3.trades_count, 3);

        // t=interval_ms — new bucket, resets bar.
        let p4 = d.on_upstream_event(&trade_event(interval_ms, 200.0, 5.0), 0)
            .expect("trade in new bucket must emit fresh bar");
        assert_eq!(p4.open_time, interval_ms, "new bar starts at next bucket boundary");
        assert_eq!(p4.open,  200.0);
        assert_eq!(p4.high,  200.0);
        assert_eq!(p4.low,   200.0);
        assert_eq!(p4.close, 200.0);
        assert!((p4.volume - 5.0).abs() < 1e-12);
        assert_eq!(p4.trades_count, 1);
    }

    /// A 1-second interval buckets at the correct ms boundary.
    #[test]
    fn trade_to_bar_sub_second_1s() {
        let key = kline_key("1s");
        let mut d = TradeToBarDerived::new_for_key(&key);

        let p1 = d.on_upstream_event(&trade_event(0, 50.0, 1.0), 0).unwrap();
        assert_eq!(p1.open_time, 0);

        // t=500ms — still inside bucket [0, 1000).
        let p2 = d.on_upstream_event(&trade_event(500, 60.0, 1.0), 0).unwrap();
        assert_eq!(p2.open_time, 0, "same 1s bucket");
        assert_eq!(p2.high, 60.0);

        // t=1000ms — new bucket.
        let p3 = d.on_upstream_event(&trade_event(1_000, 55.0, 1.0), 0).unwrap();
        assert_eq!(p3.open_time, 1_000, "second 1s bucket starts at 1000ms");
        assert_eq!(p3.open, 55.0);
    }

    /// Open must be first price, high=max, low=min, close=last across a sequence.
    #[test]
    fn trade_to_bar_ohlc_correctness() {
        let key = kline_key("5m");
        let mut d = TradeToBarDerived::new_for_key(&key);
        let bucket = 0_i64; // all inside [0, 5*60000)

        let prices = [300.0_f64, 100.0, 500.0, 200.0, 400.0];
        let qty    = [1.0_f64; 5];
        let ts     = [0_i64, 10_000, 20_000, 30_000, 40_000];

        let mut last = None;
        for i in 0..5 {
            last = d.on_upstream_event(&trade_event(ts[i], prices[i], qty[i]), 0);
        }
        let bar = last.unwrap();
        assert_eq!(bar.open_time, bucket);
        assert_eq!(bar.open,  300.0, "open = first price");
        assert_eq!(bar.high,  500.0, "high = max");
        assert_eq!(bar.low,   100.0, "low  = min");
        assert_eq!(bar.close, 400.0, "close = last price");
        let expected_vol: f64 = qty.iter().sum();
        assert!((bar.volume - expected_vol).abs() < 1e-9, "volume = sum of quantities");
        let expected_qvol: f64 = prices.iter().zip(qty.iter()).map(|(p, q)| p * q).sum();
        assert!((bar.quote_volume - expected_qvol).abs() < 1e-9);
        assert_eq!(bar.trades_count, 5);
    }

    /// `new_for_key` with a non-Kline kind sets `interval_ms=0` and never emits.
    #[test]
    fn trade_to_bar_non_kline_key_safe() {
        let key = SeriesKey::new(
            ExchangeId::Binance,
            AccountType::FuturesCross,
            "BTCUSDT",
            crate::series::Kind::Trade,
        );
        let mut d = TradeToBarDerived::new_for_key(&key);
        // Must never panic, must always return None.
        let r = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0);
        assert!(r.is_none(), "non-Kline key → interval_ms=0 → no emission");
    }

    /// `new_for_key` with an unknown interval string also sets `interval_ms=0`.
    #[test]
    fn trade_to_bar_unknown_interval_safe() {
        let key = kline_key("99x"); // not a valid interval
        let mut d = TradeToBarDerived::new_for_key(&key);
        let r = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0);
        assert!(r.is_none(), "unknown interval → interval_ms=0 → no emission");
    }

    // -----------------------------------------------------------------------
    // seed_from_events (Task A) unit tests
    // -----------------------------------------------------------------------

    /// seed_from_events primes state AND returns emitted bars.
    #[test]
    fn seed_from_events_primes_state_and_returns_bars() {
        let key = kline_key("1m");
        let mut d = TradeToBarDerived::new_for_key(&key);

        // Two trades in the same 1m bucket.
        let evs = vec![
            trade_event(0, 100.0, 1.0),
            trade_event(30_000, 120.0, 2.0),
        ];
        let emitted = d.seed_from_events(&evs, 0);

        // Both trades emitted (intra-bar updates).
        assert_eq!(emitted.len(), 2, "one bar emission per trade");
        let last = &emitted[1];
        assert_eq!(last.open, 100.0, "open = first trade");
        assert_eq!(last.high, 120.0, "high = second trade");
        assert_eq!(last.trades_count, 2);

        // State is primed: a new trade in the same bucket updates correctly.
        let cont = d.on_upstream_event(&trade_event(59_000, 110.0, 0.5), 0).unwrap();
        assert_eq!(cont.trades_count, 3, "live trade after seed updates state");
    }

    /// REGRESSION (fd58108): cold-seed and rewarm feed `kline_to_synthetic_trades`'
    /// 4-leg `Event::Trade` output directly through `seed_from_events` —
    /// NOT `Event::Bar`. `on_upstream_event` must accept `Event::Trade`
    /// unconditionally (dispatched straight to `fold_trade`) or the entire
    /// cold-seed/rewarm history path silently drops every leg (this is
    /// exactly the "history is empty" regression fd58108 introduced by
    /// requiring `Event::Bar` at the top of `on_upstream_event`).
    ///
    /// `seed_from_events`' default now wraps the fold in replay mode (see
    /// [`DerivedStream::set_replay`]) and appends [`DerivedStream::flush_replay`]
    /// at the end — feeding a single trade that only opens the seed bar (no
    /// roll) emits it once via the "first bar ever" exception AND once more
    /// via the trailing flush, so `emitted.len()` is 2, not 1.
    #[test]
    fn seed_from_events_range_bar_state_primed() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // Seed: open a bar at 100.0 via a synthetic Trade leg (cold-seed shape).
        let evs = vec![trade_event(0, 100.0, 1.0)];
        let emitted = d.seed_from_events(&evs, 0);
        assert_eq!(emitted.len(), 2, "seed-open bar (first-bar-ever) + trailing flush_replay snapshot");
        assert_eq!(emitted[0].open, 100.0);
        assert_eq!(emitted[1].open, 100.0, "flush_replay carries the same still-open bar");

        // State is primed — a further synthetic Trade leg crosses the range
        // and opens a new bar.
        let live = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        assert_eq!(live.open, 101.0, "new bar at crossing price");
    }

    /// Live WS path: repeated `Event::Bar` updates with growing volume fold
    /// into Δ-tick bars via `KlineDeltaState` (unchanged behavior from
    /// fd58108 — only the Trade-leg acceptance alongside it was missing).
    #[test]
    fn range_bar_live_bar_path_produces_delta_tick_bars() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        assert!(d.on_upstream_event(&bar_event(0, 100.0, 1.0), 0).is_none(), "baseline-only, no emit");

        let p1 = d.on_upstream_event(&bar_event(0, 100.0, 1.5), 0).unwrap();
        assert_eq!(p1.open, 100.0);

        // Close moves to 100.5, Δvol=1.0 — within range.
        let p2 = d.on_upstream_event(&bar_event(0, 100.5, 2.5), 0).unwrap();
        assert_eq!(p2.open_time, p1.open_time, "same bar");
        assert_eq!(p2.high, 100.5, "high updated");
        assert_eq!(p2.close, 100.5);
        assert_eq!(p2.trades_count, 2);

        // Crossing range on a live Bar update opens a new bar, chained from
        // the previous bar's pinned close (100.0 + range 1.0 = 101.0), not
        // the raw delta-tick price (101.5).
        let p3 = d.on_upstream_event(&bar_event(0, 101.5, 3.5), 0).unwrap();
        assert_eq!(p3.open, 101.0, "new bar opens at pinned prev close");
        assert_eq!(p3.high, 101.5, "in-progress bar absorbs the remainder");
    }

    /// seed_from_events with empty slice → no output, no side effects.
    #[test]
    fn seed_from_events_empty_slice_noop() {
        let key = kline_key("1m");
        let mut d = TradeToBarDerived::new_for_key(&key);
        let emitted = d.seed_from_events(&[], 0);
        assert!(emitted.is_empty());
        // Subsequent live trade opens a fresh bar.
        let p = d.on_upstream_event(&trade_event(0, 50.0, 1.0), 0).unwrap();
        assert_eq!(p.trades_count, 1);
    }

    // -----------------------------------------------------------------------
    // Helper for mechanical bar aggregator keys
    // -----------------------------------------------------------------------

    fn range_bar_key(range_fixed: u64) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::RangeBar(range_fixed))
    }

    fn tick_bar_key(n: u32) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::TickBar(n))
    }

    fn volume_bar_key(vol_fixed: u64) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::VolumeBar(vol_fixed))
    }

    fn dollar_bar_key(dollar_threshold: u64) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::DollarBar { dollar_threshold })
    }

    fn renko_bar_key(box_size_fixed: u64, reversal_count: u8) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::RenkoBar(box_size_fixed, reversal_count))
    }

    fn footprint_key(interval: &str) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::Footprint(KlineInterval::new(interval)))
    }

    fn trade_event_side(ts_ms: i64, price: f64, quantity: f64, side: u8) -> Event {
        Event::Trade {
            exchange: ExchangeId::Binance,
            symbol: "BTCUSDT".to_string(),
            point: crate::data::TradePoint { ts_ms, price, quantity, side, trade_id_hash: 0 },
        }
    }

    // -----------------------------------------------------------------------
    // TradeToRangeBarDerived tests
    // -----------------------------------------------------------------------

    /// Trades within range stay in one bar. Fed as `Event::Trade` — the
    /// shape both the cold-seed and rewarm paths use.
    #[test]
    fn range_bar_stays_in_bar_while_within_range() {
        // range = $1.00 = 1_0000_0000 fixed-point
        let key = range_bar_key(100_000_000);
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert_eq!(p1.open, 100.0);

        // Move 0.5 — within range.
        let p2 = d.on_upstream_event(&trade_event(1, 100.5, 1.0), 0).unwrap();
        assert_eq!(p2.open_time, p1.open_time, "same bar");
        assert_eq!(p2.open, 100.0, "open unchanged");
        assert_eq!(p2.high, 100.5, "high updated");
        assert_eq!(p2.close, 100.5);
        assert_eq!(p2.trades_count, 2);
    }

    /// Crossing range opens a new bar, chained from the previous bar's close
    /// — canonical range bars: open = prev_close, NOT the crossing trade's
    /// raw price (no tick-size data at this machine).
    #[test]
    fn range_bar_rolls_on_crossing() {
        // range = $1.00
        let key = range_bar_key(100_000_000);
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Exactly $1 movement — crosses.
        let p = d.on_upstream_event(&trade_event(10, 101.0, 2.0), 0).unwrap();
        // New bar chained from bar1's close (100.0 + range = 101.0 exactly
        // here, since the crossing trade landed precisely on the boundary),
        // and the crossing trade's volume lands in this new bar.
        assert_eq!(p.open, 101.0, "new bar opens at prev bar's pinned close");
        assert_eq!(p.trades_count, 1, "crossing trade counted in new bar");
    }

    /// A trade landing exactly ON the boundary chains through to a new bar
    /// whose open is pinned to the previous bar's close — proves the
    /// chained-open scheme (open = prev_close) independent of the trade's
    /// raw price, using a non-trivial base price where prev_close and the
    /// crossing trade's raw price would coincide only by the pin, not by
    /// accident.
    #[test]
    fn range_bar_open_is_prev_close_not_raw_crossing_price() {
        let key = range_bar_key(50_000_000); // $0.50 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Drift inside the bar first (doesn't move `open`).
        d.on_upstream_event(&trade_event(1, 100.2, 1.0), 0).unwrap();
        // Cross with a trade whose raw price OVERSHOOTS the boundary —
        // canonical range bars pin the close to open+range (100.5), not the
        // raw trade price (100.7).
        let p = d.on_upstream_event(&trade_event(2, 100.7, 1.0), 0).unwrap();
        assert_eq!(p.open, 100.5, "new bar opens at pinned prev close, not raw trade price 100.7");
    }

    /// OHLC correctness across two bars.
    #[test]
    fn range_bar_ohlc_correct() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // Bar 1: open 100, go up to 100.9, then cross with 101.
        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(1, 100.9, 1.0), 0).unwrap();
        let bar1_last = d.on_upstream_event(&trade_event(2, 100.4, 0.5), 0).unwrap();
        // bar1 still open (max deviation = 0.9 < 1.0)
        assert_eq!(bar1_last.open, 100.0);
        assert_eq!(bar1_last.high, 100.9);
        assert_eq!(bar1_last.low,  100.0);
        assert_eq!(bar1_last.close, 100.4);
    }

    /// Two bars closing at the same ms get distinct monotonic open_times.
    #[test]
    fn range_bar_monotonic_open_time_collision() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // ts=0: open bar1 at 100.0.
        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        let ot1 = p1.open_time;

        // ts=0: cross at 101.0 — bar1 closes, bar2 opens. Same ms!
        let p2 = d.on_upstream_event(&trade_event(0, 101.0, 1.0), 0).unwrap();
        assert_ne!(p2.open_time, ot1, "bar2 must not share open_time with bar1");
        assert!(p2.open_time > ot1, "bar2 open_time must be strictly greater");
    }

    /// Zero range param → no emission.
    #[test]
    fn range_bar_zero_range_safe() {
        let key = range_bar_key(0);
        let mut d = TradeToRangeBarDerived::new_for_key(&key);
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
    }

    /// A CLOSED bar's `|high - low|` is exactly `range` — pinned to the
    /// boundary the bar closed on, not the raw crossing trade's overshoot.
    #[test]
    fn range_bar_closed_high_low_pinned_to_exact_range() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Overshoot the boundary by $0.30 — closed bar1's high must still
        // pin to exactly 101.0 (open 100.0 + range 1.0), not 101.3.
        let p = d.on_upstream_event(&trade_event(1, 101.3, 1.0), 0).unwrap();
        // `p` is the NEW (chained) bar, opened at the pinned close 101.0.
        assert_eq!(p.open, 101.0, "chained bar opens at pinned close, not raw overshoot price");
        // The new bar absorbed the remaining 0.3 organically (still open).
        assert_eq!(p.high, 101.3);
        assert_eq!(p.close, 101.3);
    }

    /// A single incoming trade that moves the price by more than one full
    /// `range` from the current bar's open must emit a CHAIN of
    /// intermediate closed bars, each opening at the previous chained
    /// bar's pinned close, walking the gap — same loop shape as
    /// `TradeToRenkoBarDerived`. Gap of 3.5 ranges ⇒ 3 fully-closed chained
    /// bars (drained via `take_pending`) + 1 final in-progress bar absorbing
    /// the 0.5 remainder (returned via the `Option`).
    #[test]
    fn range_bar_gap_crossing_chains_intermediate_bars() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // Seed bar1 at 100.0.
        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert!(d.take_pending().is_empty(), "seed trade closes nothing");

        // Single trade jumps straight to 103.5 — a 3.5-range gap from
        // bar1's open (100.0). Expect the chain to walk:
        //   bar1: 100.0 -> 101.0 (closed, pinned)
        //   bar2: 101.0 -> 102.0 (closed, pinned)
        //   bar3: 102.0 -> 103.0 (closed, pinned)
        //   bar4: 103.0 -> 103.5 (in-progress, absorbs remainder)
        let last = d.on_upstream_event(&trade_event(1, 103.5, 2.0), 0).unwrap();
        let pending = d.take_pending();

        // The 3 fully-closed intermediate bars are drained via
        // `take_pending` — NOT silently lost. The final (still-open) bar is
        // returned via the `Option` as before.
        assert_eq!(pending.len(), 3, "3 fully-closed chained bars queued as pending");
        assert_eq!(pending[0].open, 100.0);
        assert_eq!(pending[0].close, 101.0);
        assert_eq!(pending[0].high, 101.0);
        assert_eq!(pending[0].low, 100.0);
        assert_eq!(pending[1].open, 101.0);
        assert_eq!(pending[1].close, 102.0);
        assert_eq!(pending[2].open, 102.0);
        assert_eq!(pending[2].close, 103.0);
        // Chained-open stitching: each pending bar's open == the previous
        // one's close, and the final bar's open continues the chain.
        assert_eq!(pending[1].open, pending[0].close);
        assert_eq!(pending[2].open, pending[1].close);
        assert_eq!(last.open, pending[2].close);
        // Strictly monotonic open_time across the whole chain.
        assert!(pending[0].open_time < pending[1].open_time);
        assert!(pending[1].open_time < pending[2].open_time);
        assert!(pending[2].open_time < last.open_time);

        assert_eq!(last.open, 103.0, "final chained bar opens at 3rd pinned close");
        assert_eq!(last.high, 103.5, "in-progress bar absorbs the 0.5 remainder");
        assert_eq!(last.low, 103.0);
        assert_eq!(last.close, 103.5);
        assert_eq!(last.volume, 2.0, "crossing trade's volume credited to the bar it lands in");
        assert_eq!(last.trades_count, 1);

        // A trade that stays within the final bar's range confirms it is
        // genuinely still open (chained-open scheme intact) rather than
        // having silently closed too.
        let p2 = d.on_upstream_event(&trade_event(2, 103.6, 0.5), 0).unwrap();
        assert!(d.take_pending().is_empty(), "in-range trade closes nothing");
        assert_eq!(p2.open_time, last.open_time, "same in-progress bar");
        assert_eq!(p2.open, 103.0);
        assert_eq!(p2.high, 103.6);
        assert_eq!(p2.trades_count, 2);
    }

    /// Downward gap crossing chains the same way, mirrored direction.
    #[test]
    fn range_bar_gap_crossing_downward_chains() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Drop 2.5 ranges in one trade: 100.0 -> 97.5.
        let last = d.on_upstream_event(&trade_event(1, 97.5, 1.0), 0).unwrap();
        let pending = d.take_pending();
        // Chain: bar1 100.0->99.0 (closed, pending), bar2 99.0->98.0
        // (closed, pending), bar3 98.0->97.5 (in-progress, returned).
        assert_eq!(pending.len(), 2, "2 fully-closed chained bars queued as pending");
        assert_eq!(pending[0].open, 100.0);
        assert_eq!(pending[0].close, 99.0);
        assert_eq!(pending[1].open, 99.0);
        assert_eq!(pending[1].close, 98.0);
        assert_eq!(last.open, 98.0);
        assert_eq!(last.low, 97.5);
        assert_eq!(last.high, 98.0);
        assert_eq!(last.close, 97.5);
    }

    /// Canon (a): wandering within `< range` stays in ONE open bar, no
    /// close — the trigger is the running envelope `(high − low)`, not
    /// distance from `open`.
    #[test]
    fn range_bar_true_range_wander_within_envelope_no_close() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Wander up and down but never let (high - low) reach $1.
        d.on_upstream_event(&trade_event(1, 100.4, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(2, 100.1, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(3,  99.9, 1.0), 0).unwrap();
        let last = d.on_upstream_event(&trade_event(4, 100.3, 1.0), 0).unwrap();

        assert!(d.take_pending().is_empty(), "no bar ever closes while envelope < range");
        assert_eq!(last.open, 100.0, "open untouched — still the seed bar");
        assert_eq!(last.high, 100.4);
        assert_eq!(last.low,   99.9);
        assert!((last.high - last.low) < 1.0, "envelope width must stay under range");
        assert_eq!(last.close, 100.3);
    }

    /// Canon (b): `open` sits MID-ENVELOPE (not on an edge) and the LOW
    /// side is what ultimately breaches `range` — the closed bar's body
    /// (`|close − open|`) is strictly less than `range`, and
    /// `high − low == range` exactly regardless of where `open` sits.
    #[test]
    fn range_bar_true_range_open_mid_envelope_breach_on_low() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // Seed at 100.0, then widen the envelope UP first so `open` (100.0)
        // ends up sitting in the middle of the eventual [low, high] span
        // rather than pinned to one edge.
        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(1, 100.4, 1.0), 0).unwrap(); // high=100.4, low=100.0, width=0.4
        // Now breach on the LOW side: price 99.6 -> candidate_low=99.6,
        // candidate_high=100.4, width=0.8 < 1.0 -> still organic absorb.
        d.on_upstream_event(&trade_event(2, 99.6, 1.0), 0).unwrap(); // high=100.4, low=99.6, width=0.8
        // Final breach: price 99.4 -> candidate_low=99.4, candidate_high=100.4,
        // width=1.0 >= range -> closes. Pinned low = high - range = 99.4.
        let closed_via_chain = d.on_upstream_event(&trade_event(3, 99.4, 1.0), 0).unwrap();
        let pending = d.take_pending();

        // The breach lands exactly on the boundary (no overshoot), so the
        // bar that closes is queued as pending and a fresh single-point bar
        // is opened and immediately returned via the `Option`.
        assert_eq!(pending.len(), 1, "exactly one bar closes on this trade");
        let closed = &pending[0];
        assert_eq!(closed.open, 100.0, "open is the real bar open, untouched by the close");
        assert_eq!(closed.high, 100.4);
        assert_eq!(closed.low,  99.4);
        assert!((closed.high - closed.low - 1.0).abs() < 1e-12, "H-L == range exactly");
        assert_eq!(closed.close, 99.4, "close = the boundary price that was breached (low side)");
        assert!((closed.close - closed.open).abs() < 1.0, "body can be < range since open is mid-envelope");

        assert_eq!(closed_via_chain.open, 99.4, "new bar opens at the pinned close");
    }

    /// Canon (c): a big single-trade jump chains N envelope-exact bars —
    /// re-confirms the chain shape still holds under the true-range rule
    /// (width-triggered, not distance-from-open).
    #[test]
    fn range_bar_true_range_big_jump_chains_envelope_exact_bars() {
        let key = range_bar_key(50_000_000); // $0.50 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 10.0, 1.0), 0).unwrap();
        // Jump straight up 2.2 ranges: 10.0 -> 11.1.
        let last = d.on_upstream_event(&trade_event(1, 11.1, 1.0), 0).unwrap();
        let pending = d.take_pending();

        assert_eq!(pending.len(), 2, "2 fully-closed envelope-exact bars queued as pending");
        for p in &pending {
            assert!((p.high - p.low - 0.5).abs() < 1e-12, "every chained bar spans exactly range");
        }
        assert_eq!(pending[0].open, 10.0);
        assert_eq!(pending[0].close, 10.5);
        assert_eq!(pending[1].open, 10.5);
        assert_eq!(pending[1].close, 11.0);
        assert_eq!(last.open, 11.0);
        assert_eq!(last.close, 11.1);
        assert!((last.high - last.low) < 0.5, "final in-progress bar hasn't closed yet");
    }

    /// Canon (d): `open == prev_close` is preserved across the chained-open
    /// scheme under the true-range rule (unchanged from the pre-fix
    /// contract — only the CLOSE trigger changed, not the open-chaining).
    #[test]
    fn range_bar_true_range_open_equals_prev_close_preserved() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Trade at 101.0 immediately closes bar1 (100 -> 101, queued to
        // pending) and returns bar2 (a fresh single-point bar opened at
        // the pinned close 101.0) via the Option.
        let bar2 = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        let bar1_pending = d.take_pending();
        assert_eq!(bar1_pending.len(), 1);
        assert_eq!(bar2.open, bar1_pending[0].close, "bar2's open == bar1's close");
        let bar2_open = bar2.open;

        // Trade at 102.0 closes bar2 (101 -> 102, queued to pending) and
        // returns bar3 (opened at the pinned close 102.0) via the Option.
        let bar3 = d.on_upstream_event(&trade_event(2, 102.0, 1.0), 0).unwrap();
        let pending = d.take_pending();
        assert_eq!(pending.len(), 1, "bar2 closed as part of this call (queued as pending)");
        assert_eq!(pending[0].open, bar2_open);
        assert_eq!(bar3.open, pending[0].close, "chained open == prev bar's close");
    }

    // -----------------------------------------------------------------------
    // TradeToRenkoBarDerived tests
    // -----------------------------------------------------------------------

    /// A single trade that crosses several brick boundaries in one call must
    /// emit ALL closed bricks — the earlier ones via `take_pending`, the
    /// last one via the `Option` return. Box size $1, first trade seeds the
    /// anchor floor-snapped to 100.0; second trade jumps to 103.4 (crosses 3
    /// full up-bricks: 100->101, 101->102, 102->103; remainder 0.4 stays
    /// in-progress with no 4th brick since Renko never returns partial
    /// bricks — the anchor simply holds at 103.0 until the next crossing).
    #[test]
    fn renko_multi_brick_move_queues_intermediates_as_pending() {
        let key = renko_bar_key(100_000_000, 1); // $1 box, reversal=1
        let mut d = TradeToRenkoBarDerived::new_for_key(&key);

        // Seed anchor — first trade never emits a brick (anchor snap only).
        let seed = d.on_upstream_event(&trade_event(0, 100.2, 1.0), 0);
        assert!(seed.is_none(), "seed trade only floor-snaps the anchor");
        assert!(d.take_pending().is_empty());

        // Jump to 103.4 — crosses 3 full box boundaries (100->101->102->103).
        let last = d.on_upstream_event(&trade_event(1, 103.4, 5.0), 0)
            .expect("multi-brick move must emit at least the last brick");
        let pending = d.take_pending();

        assert_eq!(pending.len(), 2, "2 earlier bricks queued as pending, 3rd returned via Option");
        assert_eq!(pending[0].bottom, 100.0);
        assert_eq!(pending[0].top, 101.0);
        assert!(pending[0].up);
        assert_eq!(pending[1].bottom, 101.0);
        assert_eq!(pending[1].top, 102.0);
        assert!(pending[1].up);
        assert_eq!(last.bottom, 102.0);
        assert_eq!(last.top, 103.0);
        assert!(last.up);

        // Strictly monotonic open_time across the whole chain.
        assert!(pending[0].open_time < pending[1].open_time);
        assert!(pending[1].open_time < last.open_time);

        // `vol_acc` accumulates across calls and is credited whole to the
        // FIRST brick closed within this call, then reset to zero — so the
        // remaining bricks in the same chain close with zero volume of
        // their own (pre-existing Renko convention, unchanged by the
        // pending-queue fix). Total accounted-for volume across the whole
        // chain still equals the seed (1.0) + crossing trade (5.0) = 6.0.
        assert_eq!(pending[0].volume, 6.0, "accumulated vol_acc lands on the first brick closed");
        assert_eq!(pending[1].volume, 0.0);
        assert_eq!(last.volume, 0.0);
        let total_volume: f64 = pending.iter().map(|b| b.volume).sum::<f64>() + last.volume;
        assert_eq!(total_volume, 6.0, "no volume lost across the chain");
    }

    /// Canonical TradingView reversal back-step: up-up-reversal-down.
    /// Box $1, reversal_count=1. Two up bricks place the anchor at 102.0
    /// (brick2 = `[101, 102]`, `open` = its `bottom` = 101.0). The reversal
    /// must NOT fire merely by reaching brick2's open (101.0) — canon
    /// requires traveling ONE MORE full box beyond it (to 100.0) before the
    /// down brick fires. Once it fires, the reversal brick spans
    /// `[last_open − box, last_open]` = `[100.0, 101.0]` — exactly one box
    /// from brick2's open, NOT one box off from brick2's close-adjacent
    /// edge (102.0 → the pre-fix bug would have produced `[100,101]` only
    /// by also firing one box too early, at price 101.0 instead of 100.0).
    #[test]
    fn renko_reversal_back_step_up_up_reversal_down_exact_coords() {
        let key = renko_bar_key(100_000_000, 1); // $1 box, reversal_count=1
        let mut d = TradeToRenkoBarDerived::new_for_key(&key);

        // Seed anchor at exactly 100.0 (floor-snap of 100.0 is 100.0).
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());

        // Up brick 1: 100.0 -> 101.0. bottom=100.0, top=101.0, open=bottom=100.0.
        let up1 = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        assert_eq!(up1.bottom, 100.0);
        assert_eq!(up1.top, 101.0);
        assert!(up1.up);

        // Up brick 2: 101.0 -> 102.0. bottom=101.0, top=102.0, open=bottom=101.0.
        let up2 = d.on_upstream_event(&trade_event(2, 102.0, 1.0), 0).unwrap();
        assert_eq!(up2.bottom, 101.0);
        assert_eq!(up2.top, 102.0);
        assert!(up2.up);

        // Price falls to EXACTLY brick2's open (101.0) — must NOT reverse
        // yet (canon requires one more full box beyond the open, to 100.0).
        // The pre-fix bug fired here.
        let no_reversal = d.on_upstream_event(&trade_event(3, 101.0, 1.0), 0);
        assert!(no_reversal.is_none(), "reaching the last brick's open alone must not trigger reversal");
        assert!(d.take_pending().is_empty());

        // Price falls to exactly 100.0 — one full box beyond brick2's open
        // (101.0) — NOW the reversal fires.
        let down = d.on_upstream_event(&trade_event(4, 100.0, 1.0), 0)
            .expect("one box beyond the last brick's open must trigger the reversal");
        assert!(!down.up, "reversal brick is a down brick");
        assert_eq!(down.bottom, 100.0, "reversal brick = [last_open - box, last_open]");
        assert_eq!(down.top, 101.0, "reversal brick's top = brick2's open (101.0), not its close (102.0)");
    }

    /// An up brick that dips BELOW its eventual bottom edge while forming
    /// (before recovering and crossing the top boundary) stamps a non-zero
    /// `wick` = the adverse excursion distance. A brick that closes cleanly
    /// (never adverse) stamps `wick == 0.0`.
    #[test]
    fn renko_brick_wick_tracks_adverse_intra_brick_excursion() {
        let key = renko_bar_key(100_000_000, 1); // $1 box, reversal_count=1
        let mut d = TradeToRenkoBarDerived::new_for_key(&key);

        // Seed anchor at exactly 100.0.
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
        // Dip to 99.7 BEFORE the brick closes — adverse excursion below the
        // eventual bottom (100.0) of the up brick that will close later.
        assert!(d.on_upstream_event(&trade_event(1, 99.7, 1.0), 0).is_none());
        // Now cross up_target (101.0) — up brick closes: bottom=100.0, top=101.0.
        let up = d.on_upstream_event(&trade_event(2, 101.0, 1.0), 0).unwrap();
        assert_eq!(up.bottom, 100.0);
        assert_eq!(up.top, 101.0);
        assert!(up.up);
        assert!((up.wick - 0.3).abs() < 1e-9, "wick = bottom(100.0) - intra_min(99.7) = 0.3, got {}", up.wick);

        // Next brick closes cleanly with no adverse excursion -> wick == 0.0.
        let up2 = d.on_upstream_event(&trade_event(3, 102.0, 1.0), 0).unwrap();
        assert_eq!(up2.bottom, 101.0);
        assert_eq!(up2.top, 102.0);
        assert_eq!(up2.wick, 0.0, "no adverse excursion on this brick");
    }

    // -----------------------------------------------------------------------
    // TradeToPnfBarDerived tests — extension off-by-one verdict (item 6)
    // -----------------------------------------------------------------------

    /// DECISIVE UNIT TRACE — extension off-by-one verdict.
    ///
    /// Canon: a column of X with top `T` (top edge of the last placed box),
    /// price reaching EXACTLY `T + box_size` must append exactly ONE new X
    /// box (new top = `T + box_size`), not require 2 boxes of travel.
    ///
    /// VERDICT: current semantics are ALREADY CORRECT. The extension gate
    /// is `price >= col.top + box_size` (an inclusive lower-bound check,
    /// not `price > col.top + box_size`), and `steps =
    /// floor((price - top) / box_size)`. At `price == top + box_size`
    /// exactly: `steps = floor(box_size / box_size) = floor(1) = 1`, so
    /// `top` advances by exactly `1 * box_size` — ONE new box, no delay.
    /// There is no off-by-one here; `top` is a genuine lower-bound trigger,
    /// not an exclusive one. This test pins that verdict as a regression
    /// guard.
    #[test]
    fn pnf_extension_exactly_one_box_at_price_equals_top_plus_box_size() {
        let mut d = TradeToPnfBarDerived::new(1.0, 3); // $1 box, reversal=3

        // Seed the first X column: floor-snap(100.2) -> bottom=100.0, top=101.0.
        let seed = d.on_upstream_event(&trade_event(0, 100.2, 1.0), 0).unwrap();
        assert!(seed.is_x);
        assert_eq!(seed.bottom, 100.0);
        assert_eq!(seed.top, 101.0);

        // Price reaches EXACTLY top(101.0) + box_size(1.0) = 102.0.
        let extended = d.on_upstream_event(&trade_event(1, 102.0, 1.0), 0).unwrap();
        assert!(extended.is_x, "still the same X column, extended not reversed");
        assert_eq!(extended.bottom, 100.0, "bottom unchanged by extension");
        assert_eq!(extended.top, 102.0, "top advances by EXACTLY one box (101.0 -> 102.0)");
    }

    /// A larger jump extends by the correct NUMBER of boxes (not just one),
    /// confirming `steps = floor((price - top) / box_size)` generalizes
    /// beyond the single-box decisive trace above.
    #[test]
    fn pnf_extension_multi_box_jump_advances_correct_step_count() {
        let mut d = TradeToPnfBarDerived::new(1.0, 3); // $1 box, reversal=3

        d.on_upstream_event(&trade_event(0, 100.2, 1.0), 0).unwrap(); // bottom=100.0, top=101.0

        // Jump to 104.0 -> 3 full boxes beyond top(101.0): steps = floor(3.0/1.0) = 3.
        let extended = d.on_upstream_event(&trade_event(1, 104.0, 1.0), 0).unwrap();
        assert!(extended.is_x);
        assert_eq!(extended.top, 104.0, "top advances by exactly 3 boxes (101 -> 104)");
    }

    // -----------------------------------------------------------------------
    // TradeToKagiBarDerived tests — mid-segment thickness split (item 7)
    // -----------------------------------------------------------------------

    /// An up-swing crossing the previous shoulder mid-way emits a
    /// thin-then-thick pair with the split EXACTLY at the shoulder price —
    /// mirrored here for a DOWN-swing crossing the previous WAIST (same
    /// mechanic, opposite direction, easier to seed deterministically).
    ///
    /// Sequence (reversal = $2.0):
    /// - Seed: 100 -> 103 (up, seed segment, hardcoded thick).
    /// - Continue up 103 -> 110 (no shoulder exists yet, no split possible;
    ///   `last_shoulder` becomes 110 once this leg closes).
    /// - Reverse down: 110 -> 106 (against = 4 >= 2) — closes the up leg
    ///   [103 -> 110], thin (never crossed anything, since `last_shoulder`
    ///   was `-inf` for its entire life).
    /// - Continue down: 106 -> 95. This CROSSES `last_waist` (100, set at
    ///   seed time) mid-leg — split fires exactly at 100.0: sub-segment
    ///   [106 -> 100] closes THIN (queued to `pending`), live sub-segment
    ///   [100 -> 95] continues THICK.
    /// - Continue down further to 90, then reverse up to 94 (against = 4)
    ///   — closes the down sub-segment [100 -> 90], still THICK (the
    ///   thickness set at the split persists for the rest of the leg).
    #[test]
    fn kagi_down_swing_crossing_prior_waist_mid_segment_splits_thin_then_thick() {
        let mut d = TradeToKagiBarDerived::new(2.0); // reversal = $2.0

        // Seed: 100 -> 103 (up). Sets last_waist = 100.0.
        let seed = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0);
        assert!(seed.is_none(), "first trade only primes the anchor");
        let seed = d.on_upstream_event(&trade_event(1, 103.0, 1.0), 0).unwrap();
        assert_eq!(seed.start_price, 100.0);
        assert_eq!(seed.end_price, 103.0);
        assert!(seed.up);
        assert!(seed.yang, "seed segment is hardcoded thick");
        assert!(d.take_pending().is_empty());

        // Continue up: 103 -> 110 (no shoulder yet -> no split).
        assert!(d.on_upstream_event(&trade_event(2, 105.0, 1.0), 0).is_none());
        assert!(d.on_upstream_event(&trade_event(3, 110.0, 1.0), 0).is_none());
        assert!(d.take_pending().is_empty());

        // Reverse down: 110 -> 106 (against = 4 >= reversal 2) — closes the
        // up leg [103 -> 110], thin (never crossed a shoulder — there was
        // none yet).
        let up_leg = d.on_upstream_event(&trade_event(4, 106.0, 1.0), 0).unwrap();
        assert_eq!(up_leg.start_price, 103.0);
        assert_eq!(up_leg.end_price, 110.0);
        assert!(up_leg.up);
        assert!(!up_leg.yang, "up leg never crossed a shoulder (none existed yet) -> thin");
        assert!(d.take_pending().is_empty());

        // Continue down: 106 -> 95. Crosses last_waist (100.0) mid-leg —
        // split fires EXACTLY at 100.0.
        let live_piece = d.on_upstream_event(&trade_event(5, 95.0, 1.0), 0)
            .expect("mid-segment split emits the new live piece via the Option");
        let pending = d.take_pending();
        assert_eq!(pending.len(), 1, "the pre-crossing sub-segment is queued as pending");

        let closed_thin = &pending[0];
        assert_eq!(closed_thin.start_price, 106.0, "pre-crossing sub-segment starts where the down leg began");
        assert_eq!(closed_thin.end_price, 100.0, "split lands EXACTLY at the waist price");
        assert!(!closed_thin.up, "down direction");
        assert!(!closed_thin.yang, "pre-crossing sub-segment keeps the leg's original (thin) thickness");

        assert_eq!(live_piece.start_price, 100.0, "live piece starts exactly at the crossing price");
        assert_eq!(live_piece.end_price, 95.0, "live piece extends to the current trade");
        assert!(!live_piece.up);
        assert!(live_piece.yang, "post-crossing live piece is thick");

        // Continue down further, then reverse — the down sub-segment that
        // closes keeps the FLIPPED (thick) thickness set at the split.
        assert!(d.on_upstream_event(&trade_event(6, 90.0, 1.0), 0).is_none());
        let down_subseg = d.on_upstream_event(&trade_event(7, 94.0, 1.0), 0).unwrap();
        assert_eq!(down_subseg.start_price, 100.0, "closing sub-segment's start = the crossing price");
        assert_eq!(down_subseg.end_price, 90.0);
        assert!(!down_subseg.up);
        assert!(down_subseg.yang, "thickness set at the mid-segment split persists for the rest of the leg");
    }

    /// Zero reversal → no emission (disabled guard, unaffected by the
    /// mid-segment split machinery).
    #[test]
    fn kagi_zero_reversal_safe() {
        let mut d = TradeToKagiBarDerived::new(0.0);
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
        assert!(d.take_pending().is_empty());
    }

    // -----------------------------------------------------------------------
    // TradeToTickBarDerived tests
    // -----------------------------------------------------------------------

    /// Every n trades rolls a new bar.
    #[test]
    fn tick_bar_rolls_every_n() {
        let n = 3u32;
        let key = tick_bar_key(n);
        let mut d = TradeToTickBarDerived::new_for_key(&key);

        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert_eq!(p1.trades_count, 1);
        let p2 = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        assert_eq!(p2.trades_count, 2);
        let p3 = d.on_upstream_event(&trade_event(2, 99.0, 1.0), 0).unwrap();
        assert_eq!(p3.trades_count, 3, "3rd trade completes bar");

        // 4th trade opens new bar.
        let p4 = d.on_upstream_event(&trade_event(3, 102.0, 2.0), 0).unwrap();
        assert_eq!(p4.trades_count, 1, "first trade in new bar");
        assert_eq!(p4.open, 102.0, "new bar open = 4th trade price");
    }

    /// OHLC across one complete bar.
    #[test]
    fn tick_bar_ohlc_correct() {
        let key = tick_bar_key(3);
        let mut d = TradeToTickBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 200.0, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(1,  50.0, 1.0), 0).unwrap();
        let last = d.on_upstream_event(&trade_event(2, 150.0, 1.0), 0).unwrap();

        assert_eq!(last.open,  200.0);
        assert_eq!(last.high,  200.0);
        assert_eq!(last.low,    50.0);
        assert_eq!(last.close, 150.0);
        assert!((last.volume - 3.0).abs() < 1e-12);
    }

    /// n=0 → no emission.
    #[test]
    fn tick_bar_zero_n_safe() {
        let key = tick_bar_key(0);
        let mut d = TradeToTickBarDerived::new_for_key(&key);
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
    }

    // -----------------------------------------------------------------------
    // TradeToVolumeBarDerived tests
    // -----------------------------------------------------------------------

    /// Crossing threshold rolls a bar; crossing trade is in the closing bar.
    /// Fed as `Event::Trade` — the shape both cold-seed and rewarm use.
    #[test]
    fn volume_bar_rolls_on_threshold() {
        // threshold = 2.0 volume = 200_000_000 fixed-point
        let key = volume_bar_key(200_000_000);
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);

        // Trade 1: vol=1.0 — cumulative 1.0 < 2.0 threshold.
        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert!((p1.volume - 1.0).abs() < 1e-12);

        // Trade 2: vol=1.0 — cumulative 2.0 >= 2.0 → roll.
        let p2 = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        assert!((p2.volume - 2.0).abs() < 1e-12, "crossing trade in closing bar");
        assert_eq!(p2.close, 101.0, "close = crossing trade price");

        // Trade 3: opens new bar at its OWN real price (102.0) — canon:
        // volume bars open at the first trade's real price, genuine gaps
        // allowed (no chaining to the previous bar's close).
        let p3 = d.on_upstream_event(&trade_event(2, 102.0, 0.5), 0).unwrap();
        assert_eq!(p3.open, 102.0, "new bar opens at its own first trade's real price");
        assert_eq!(p3.high, 102.0);
        assert_eq!(p3.low, 102.0);
        assert_eq!(p3.close, 102.0);
        assert_ne!(p3.open_time, p2.open_time);
    }

    /// OHLC across one complete volume bar.
    #[test]
    fn volume_bar_ohlc_correct() {
        let key = volume_bar_key(300_000_000); // threshold = 3.0
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        d.on_upstream_event(&trade_event(1, 200.0, 1.0), 0).unwrap();
        let last = d.on_upstream_event(&trade_event(2,  50.0, 1.0), 0).unwrap();

        assert_eq!(last.open,  100.0);
        assert_eq!(last.high,  200.0);
        assert_eq!(last.low,    50.0);
        assert_eq!(last.close,  50.0);
        assert!((last.volume - 3.0).abs() < 1e-12);
    }

    /// Canon: `open` is the FIRST TRADE's real price of the bar — genuine
    /// gaps between consecutive bars are allowed (no chained-open pinning
    /// to the previous bar's close, since a volume bar is not
    /// price-path-derived).
    #[test]
    fn volume_bar_open_is_first_trade_real_price_genuine_gap_allowed() {
        let key = volume_bar_key(100_000_000); // threshold = 1.0
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);

        // Bar 1: single trade closes it immediately (vol=1.0 >= threshold).
        let bar1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert_eq!(bar1.close, 100.0);

        // Bar 2's first trade lands at a completely different price (105.0)
        // — a genuine gap from bar1's close (100.0), same as tick bars.
        let bar2 = d.on_upstream_event(&trade_event(1, 105.0, 1.0), 0).unwrap();
        assert_eq!(bar2.open, 105.0, "open = the bar's own first trade's real price");
        assert_eq!(bar2.high, 105.0);
        assert_eq!(bar2.low, 105.0);
        assert_eq!(bar2.close, 105.0);
    }

    /// Zero threshold → no emission.
    #[test]
    fn volume_bar_zero_threshold_safe() {
        let key = volume_bar_key(0);
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
    }

    /// Live WS path: repeated `Event::Bar` updates with growing volume fold
    /// into Δ-tick bars via `KlineDeltaState`, rolling on threshold exactly
    /// as the Trade-fed path does.
    #[test]
    fn volume_bar_live_bar_path_produces_delta_tick_bars() {
        // threshold = 2.0 volume = 200_000_000 fixed-point
        let key = volume_bar_key(200_000_000);
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);

        d.on_upstream_event(&bar_event(0, 100.0, 0.0), 0); // baseline, vol=0

        // Δvol=1.0 — cumulative 1.0 < 2.0 threshold.
        let p1 = d.on_upstream_event(&bar_event(0, 100.0, 1.0), 0).unwrap();
        assert!((p1.volume - 1.0).abs() < 1e-12);

        // Δvol=1.0 — cumulative 2.0 >= 2.0 → roll.
        let p2 = d.on_upstream_event(&bar_event(0, 101.0, 2.0), 0).unwrap();
        assert!((p2.volume - 2.0).abs() < 1e-12, "crossing tick in closing bar");
        assert_eq!(p2.close, 101.0, "close = crossing tick price");

        // New open_time (bar rolled on the kline side too): baseline resets
        // to zero volume, so the first tick on the new bar reports its own
        // already-accrued volume (0.5) — opens the new derived bar at its
        // OWN synthetic-tick price (102.0), canon (no chaining).
        let p3 = d.on_upstream_event(&bar_event(60_000, 102.0, 0.5), 0).unwrap();
        assert_eq!(p3.open, 102.0, "new bar opens at its own first synthetic tick's price");
        assert_eq!(p3.high, 102.0);
        assert_ne!(p3.open_time, p2.open_time);
    }

    /// `folds` module Э0 contract: a direct-constructed
    /// `TradeToVolumeBarDerived::new(threshold)` must produce output
    /// identical to a `new_for_key`-constructed one when fed the same
    /// chronological event sequence — proves the constructor delegation
    /// introduced no behavior drift.
    #[test]
    fn volume_bar_direct_new_matches_new_for_key() {
        // threshold = 2.0 volume = 200_000_000 fixed-point
        let key = volume_bar_key(200_000_000);
        let mut via_key = TradeToVolumeBarDerived::new_for_key(&key);
        let mut via_new = TradeToVolumeBarDerived::new(2.0);

        let events = [
            trade_event(0, 100.0, 1.0),
            trade_event(1, 101.0, 1.0), // crosses threshold — rolls
            trade_event(2, 102.0, 0.5), // opens new bar
            trade_event(3, 103.0, 2.0), // crosses again — rolls
        ];

        for ev in &events {
            let out_key = via_key.on_upstream_event(ev, 0);
            let out_new = via_new.on_upstream_event(ev, 0);
            match (out_key, out_new) {
                (Some(a), Some(b)) => {
                    assert_eq!(a.open_time, b.open_time);
                    assert_eq!(a.open, b.open);
                    assert_eq!(a.high, b.high);
                    assert_eq!(a.low, b.low);
                    assert_eq!(a.close, b.close);
                    assert!((a.volume - b.volume).abs() < 1e-12);
                    assert!((a.quote_volume - b.quote_volume).abs() < 1e-12);
                    assert_eq!(a.trades_count, b.trades_count);
                }
                (None, None) => {}
                other => panic!("emission mismatch between new_for_key and new: {other:?}"),
            }
        }
    }

    // -----------------------------------------------------------------------
    // TradeToDollarBarDerived tests
    // -----------------------------------------------------------------------

    /// Crossing dollar threshold rolls a bar; crossing trade is in the
    /// closing bar (same convention as `TradeToVolumeBarDerived`).
    #[test]
    fn dollar_bar_rolls_on_threshold() {
        // threshold = $200 dollar value.
        let key = dollar_bar_key(200);
        let mut d = TradeToDollarBarDerived::new_for_key(&key);

        // Trade 1: price=100, qty=1.0 → dollar value 100 < 200 threshold.
        let p1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert_eq!(p1.close, 100.0);

        // Trade 2: price=100, qty=1.0 → cumulative dollar value 200 >= 200 → roll.
        let p2 = d.on_upstream_event(&trade_event(1, 100.0, 1.0), 0).unwrap();
        assert_eq!(p2.close, 100.0, "crossing trade in closing bar");
    }

    /// Canon: `open` is the FIRST TRADE's real price of the bar — genuine
    /// gaps between consecutive bars are allowed (no chained-open pinning
    /// to the previous bar's close, since a dollar bar is not
    /// price-path-derived).
    #[test]
    fn dollar_bar_open_is_first_trade_real_price_genuine_gap_allowed() {
        let key = dollar_bar_key(100); // threshold = $100
        let mut d = TradeToDollarBarDerived::new_for_key(&key);

        // Bar 1: single trade closes it immediately (100*1.0=100 >= threshold).
        let bar1 = d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        assert_eq!(bar1.close, 100.0);

        // Bar 2's first trade lands at a completely different price (110.0)
        // — a genuine gap from bar1's close (100.0).
        let bar2 = d.on_upstream_event(&trade_event(1, 110.0, 1.0), 0).unwrap();
        assert_eq!(bar2.open, 110.0, "open = the bar's own first trade's real price");
        assert_eq!(bar2.high, 110.0);
        assert_eq!(bar2.low, 110.0);
        assert_eq!(bar2.close, 110.0);
    }

    /// Zero threshold → no emission.
    #[test]
    fn dollar_bar_zero_threshold_safe() {
        let key = dollar_bar_key(0);
        let mut d = TradeToDollarBarDerived::new_for_key(&key);
        assert!(d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).is_none());
    }

    // -----------------------------------------------------------------------
    // TradeToFootprintDerived tests
    // -----------------------------------------------------------------------

    /// Per-level buy/sell accumulate correctly by side.
    #[test]
    fn footprint_per_level_buy_sell() {
        let key = footprint_key("1m");
        let mut d = TradeToFootprintDerived::new_for_key(&key);

        // Two buys at 100.0, one sell at 100.0.
        d.on_upstream_event(&trade_event_side(0, 100.0, 1.5, 0), 0); // buy 1.5
        d.on_upstream_event(&trade_event_side(1, 100.0, 0.5, 1), 0); // sell 0.5
        let p = d.on_upstream_event(&trade_event_side(2, 100.0, 1.0, 0), 0).unwrap(); // buy 1.0

        assert_eq!(p.levels.len(), 1, "one unique price level");
        let (price, buy, sell) = p.levels[0];
        assert!((price - 100.0).abs() < 1e-12);
        assert!((buy  -  2.5 ).abs() < 1e-12, "buy = 1.5 + 1.0");
        assert!((sell -  0.5 ).abs() < 1e-12);
    }

    /// Bucket roll resets levels and OHLC.
    #[test]
    fn footprint_bucket_roll_resets() {
        let key = footprint_key("1m");
        let interval_ms = 60_000_i64;
        let mut d = TradeToFootprintDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event_side(0, 100.0, 1.0, 0), 0);
        // Trade in next bucket.
        let p = d.on_upstream_event(&trade_event_side(interval_ms, 200.0, 2.0, 1), 0).unwrap();
        assert_eq!(p.open_time, interval_ms, "new bucket");
        assert_eq!(p.open, 200.0, "reset to new bucket open");
        assert_eq!(p.levels.len(), 1, "only new bucket level");
        let (_, buy, sell) = p.levels[0];
        assert!((buy - 0.0).abs() < 1e-12);
        assert!((sell - 2.0).abs() < 1e-12);
    }

    /// OHLC accumulates correctly across bucket.
    #[test]
    fn footprint_ohlc_correct() {
        let key = footprint_key("1m");
        let mut d = TradeToFootprintDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event_side(0, 100.0, 1.0, 0), 0);
        d.on_upstream_event(&trade_event_side(1, 200.0, 1.0, 1), 0);
        let p = d.on_upstream_event(&trade_event_side(2,  50.0, 1.0, 0), 0).unwrap();

        assert_eq!(p.open,   100.0);
        assert_eq!(p.high,   200.0);
        assert_eq!(p.low,     50.0);
        assert_eq!(p.close,   50.0);
        assert!((p.volume - 3.0).abs() < 1e-12);
    }

    /// Multiple price levels are sorted by price (BTreeMap ordering of positive f64 bits).
    #[test]
    fn footprint_levels_sorted_by_price() {
        let key = footprint_key("1m");
        let mut d = TradeToFootprintDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event_side(0, 300.0, 1.0, 0), 0);
        d.on_upstream_event(&trade_event_side(1, 100.0, 1.0, 0), 0);
        let p = d.on_upstream_event(&trade_event_side(2, 200.0, 1.0, 1), 0).unwrap();

        assert_eq!(p.levels.len(), 3);
        // Prices should be sorted ascending.
        let prices: Vec<f64> = p.levels.iter().map(|(pr, _, _)| *pr).collect();
        assert!(prices[0] < prices[1] && prices[1] < prices[2],
            "levels must be sorted ascending: {:?}", prices);
    }

    /// Unknown interval → disabled.
    #[test]
    fn footprint_unknown_interval_safe() {
        let key = footprint_key("99x");
        let mut d = TradeToFootprintDerived::new_for_key(&key);
        assert!(d.on_upstream_event(&trade_event_side(0, 100.0, 1.0, 0), 0).is_none());
    }

    /// `group_step` > 0.0: trades at 100.01 / 100.02 / 100.03 with
    /// `group_step = 0.05` all land in ONE row keyed at the row's lower
    /// boundary (100.00) — buy/sell attribution preserved per row.
    #[test]
    fn footprint_group_step_bins_trades_into_one_row() {
        // 60_000 ms interval ("1m"), group_step = 0.05.
        let mut d = TradeToFootprintDerived::new(60_000, 0.05);

        d.on_upstream_event(&trade_event_side(0, 100.01, 1.0, 0), 0); // buy 1.0
        d.on_upstream_event(&trade_event_side(1, 100.02, 0.5, 1), 0); // sell 0.5
        let p = d.on_upstream_event(&trade_event_side(2, 100.03, 2.0, 0), 0).unwrap(); // buy 2.0

        assert_eq!(p.levels.len(), 1, "all three trades group into one row");
        let (price, buy, sell) = p.levels[0];
        assert!((price - 100.0).abs() < 1e-9, "row keyed at the lower boundary 100.00, got {price}");
        assert!((buy  - 3.0).abs() < 1e-9, "buy = 1.0 + 2.0");
        assert!((sell - 0.5).abs() < 1e-9);
    }

    /// `group_step == 0.0` (default / `new_ungrouped`) preserves the
    /// pre-grouping raw-price behavior — each distinct trade price is its
    /// own row.
    #[test]
    fn footprint_group_step_zero_is_raw_ungrouped() {
        let mut d = TradeToFootprintDerived::new_ungrouped(60_000);

        d.on_upstream_event(&trade_event_side(0, 100.01, 1.0, 0), 0);
        d.on_upstream_event(&trade_event_side(1, 100.02, 0.5, 1), 0);
        let p = d.on_upstream_event(&trade_event_side(2, 100.03, 2.0, 0), 0).unwrap();

        assert_eq!(p.levels.len(), 3, "raw granularity: each distinct price is its own row");
    }

    /// `group_step` bins trades that straddle a row boundary into
    /// DIFFERENT rows — proves the grouping is a genuine floor-division,
    /// not an accidental single-bucket collapse.
    #[test]
    fn footprint_group_step_respects_row_boundaries() {
        let mut d = TradeToFootprintDerived::new(60_000, 0.05);

        d.on_upstream_event(&trade_event_side(0, 100.03, 1.0, 0), 0); // row 100.00
        let p = d.on_upstream_event(&trade_event_side(1, 100.06, 1.0, 0), 0).unwrap(); // row 100.05

        assert_eq!(p.levels.len(), 2, "trades on either side of the 0.05 boundary land in different rows");
        let prices: Vec<f64> = p.levels.iter().map(|(pr, _, _)| *pr).collect();
        assert!((prices[0] - 100.00).abs() < 1e-9);
        assert!((prices[1] - 100.05).abs() < 1e-9);
    }

    // -----------------------------------------------------------------------
    // kline_to_synthetic_trades
    // -----------------------------------------------------------------------

    fn sample_bar(open: f64, high: f64, low: f64, close: f64, volume: f64) -> BarPoint {
        BarPoint {
            open_time: 1_000,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: f64::NAN,
            trades_count: 0,
        }
    }

    #[test]
    fn kline_to_synthetic_trades_bullish_path_is_o_l_h_c() {
        let bar = sample_bar(100.0, 120.0, 90.0, 110.0, 40.0);
        let events = kline_to_synthetic_trades(&bar, 60_000, ExchangeId::Binance, "BTCUSDT");
        assert_eq!(events.len(), 4);
        let prices: Vec<f64> = events.iter().map(|e| {
            let Event::Trade { point, .. } = e else { panic!("expected Event::Trade") };
            point.price
        }).collect();
        assert_eq!(prices, vec![100.0, 90.0, 120.0, 110.0], "bullish bar must trace O->L->H->C");
    }

    #[test]
    fn kline_to_synthetic_trades_bearish_path_is_o_h_l_c() {
        let bar = sample_bar(110.0, 120.0, 90.0, 100.0, 40.0);
        let events = kline_to_synthetic_trades(&bar, 60_000, ExchangeId::Binance, "BTCUSDT");
        assert_eq!(events.len(), 4);
        let prices: Vec<f64> = events.iter().map(|e| {
            let Event::Trade { point, .. } = e else { panic!("expected Event::Trade") };
            point.price
        }).collect();
        assert_eq!(prices, vec![110.0, 120.0, 90.0, 100.0], "bearish bar must trace O->H->L->C");
    }

    #[test]
    fn kline_to_synthetic_trades_timestamps_evenly_spaced_within_bar() {
        let bar = sample_bar(100.0, 120.0, 90.0, 110.0, 40.0);
        let events = kline_to_synthetic_trades(&bar, 60_000, ExchangeId::Binance, "BTCUSDT");
        let ts: Vec<i64> = events.iter().map(|e| {
            let Event::Trade { point, .. } = e else { panic!("expected Event::Trade") };
            point.ts_ms
        }).collect();
        assert_eq!(ts, vec![1_000, 16_000, 31_000, 46_000]);
        // All 4 timestamps stay inside [open_time, open_time + interval_ms).
        assert!(ts.iter().all(|t| *t >= bar.open_time && *t < bar.open_time + 60_000));
    }

    #[test]
    fn kline_to_synthetic_trades_volume_split_evenly() {
        let bar = sample_bar(100.0, 120.0, 90.0, 110.0, 40.0);
        let events = kline_to_synthetic_trades(&bar, 60_000, ExchangeId::Binance, "BTCUSDT");
        for e in &events {
            let Event::Trade { point, .. } = e else { panic!("expected Event::Trade") };
            assert!((point.quantity - 10.0).abs() < 1e-12, "each leg should carry volume/4");
        }
    }

    #[test]
    fn kline_to_synthetic_trades_zero_interval_returns_empty() {
        let bar = sample_bar(100.0, 120.0, 90.0, 110.0, 40.0);
        assert!(kline_to_synthetic_trades(&bar, 0, ExchangeId::Binance, "BTCUSDT").is_empty());
    }

    #[test]
    fn kline_to_synthetic_trades_carries_exchange_and_symbol() {
        let bar = sample_bar(100.0, 120.0, 90.0, 110.0, 40.0);
        let events = kline_to_synthetic_trades(&bar, 60_000, ExchangeId::Bybit, "ETHUSDT");
        for e in &events {
            let Event::Trade { exchange, symbol, .. } = e else { panic!("expected Event::Trade") };
            assert_eq!(*exchange, ExchangeId::Bybit);
            assert_eq!(symbol, "ETHUSDT");
        }
    }

    // -----------------------------------------------------------------------
    // KlineDeltaState — live Δ-tick adapter tests
    // -----------------------------------------------------------------------

    fn extract_trade(ev: &Event) -> &TradePoint {
        let Event::Trade { point, .. } = ev else { panic!("expected Event::Trade") };
        point
    }

    /// Same open_time repeated updates → correct Δvolume ticks, no rollback,
    /// no double-count of already-reported volume.
    #[test]
    fn kline_delta_state_same_open_time_produces_correct_deltas() {
        let mut st = KlineDeltaState::new();
        let tf = KlineInterval::new("1m");

        // First event ever seen (no cold-seed baseline handed over) only
        // establishes the baseline — no synthetic trade.
        let bar0 = BarPoint { open_time: 0, open: 100.0, high: 100.0, low: 100.0, close: 100.0, volume: 1.0, quote_volume: 100.0, trades_count: 1 };
        assert!(st.to_delta_trade(&bar0, &tf, ExchangeId::Binance, "BTCUSDT").is_none());

        // Same open_time, volume grew 1.0 -> 2.5: Δ = 1.5.
        let bar1 = BarPoint { open_time: 0, open: 100.0, high: 101.0, low: 100.0, close: 101.0, volume: 2.5, quote_volume: 250.0, trades_count: 2 };
        let ev1 = st.to_delta_trade(&bar1, &tf, ExchangeId::Binance, "BTCUSDT").expect("volume grew — must emit");
        let t1 = extract_trade(&ev1);
        assert!((t1.quantity - 1.5).abs() < 1e-12, "Δvolume = 2.5 - 1.0 = 1.5");
        assert_eq!(t1.price, 101.0, "price = bar.close");
        assert_eq!(t1.side, 0, "close rose vs previous close — buy side");

        // Same open_time, volume grew again 2.5 -> 4.0: Δ = 1.5, price falls.
        let bar2 = BarPoint { open_time: 0, open: 100.0, high: 101.0, low: 99.0, close: 99.0, volume: 4.0, quote_volume: 396.0, trades_count: 3 };
        let ev2 = st.to_delta_trade(&bar2, &tf, ExchangeId::Binance, "BTCUSDT").expect("volume grew — must emit");
        let t2 = extract_trade(&ev2);
        assert!((t2.quantity - 1.5).abs() < 1e-12);
        assert_eq!(t2.side, 1, "close fell vs previous close — sell side");

        // Duplicate frame (same open_time, same volume) → no emission, no
        // double-count.
        assert!(st.to_delta_trade(&bar2, &tf, ExchangeId::Binance, "BTCUSDT").is_none(), "zero delta must not emit");
    }

    /// New open_time (bar rolled) → previous bar is final and never
    /// re-folded; the new bar's baseline resets to zero volume so its first
    /// observed update reports its OWN accrued volume, not a delta against
    /// the old bar.
    #[test]
    fn kline_delta_state_rollover_resets_baseline_no_double_count() {
        let mut st = KlineDeltaState::new();
        let tf = KlineInterval::new("1m");

        let bar_a0 = BarPoint { open_time: 0, open: 100.0, high: 100.0, low: 100.0, close: 100.0, volume: 5.0, quote_volume: 500.0, trades_count: 1 };
        assert!(st.to_delta_trade(&bar_a0, &tf, ExchangeId::Binance, "BTCUSDT").is_none(), "baseline only");

        let bar_a1 = BarPoint { open_time: 0, open: 100.0, high: 102.0, low: 100.0, close: 102.0, volume: 9.0, quote_volume: 900.0, trades_count: 2 };
        let ev_a1 = st.to_delta_trade(&bar_a1, &tf, ExchangeId::Binance, "BTCUSDT").expect("must emit");
        assert!((extract_trade(&ev_a1).quantity - 4.0).abs() < 1e-12, "Δvolume within bar A = 9-5 = 4");

        // Bar rolls to a NEW open_time — bar A (final volume 9.0) is never
        // re-folded. Bar B's first observed update carries volume=1.2 —
        // must be reported as 1.2 (its own accrued volume), NOT as
        // 1.2 - 9.0 (which would be negative / would imply double-counting
        // bar A's volume against bar B).
        let bar_b0 = BarPoint { open_time: 60_000, open: 102.0, high: 103.0, low: 102.0, close: 103.0, volume: 1.2, quote_volume: 123.6, trades_count: 1 };
        let ev_b0 = st.to_delta_trade(&bar_b0, &tf, ExchangeId::Binance, "BTCUSDT").expect("new bar's first update must emit");
        let t_b0 = extract_trade(&ev_b0);
        assert!((t_b0.quantity - 1.2).abs() < 1e-12, "new bar reports its own accrued volume, not a delta vs bar A");
        assert!(t_b0.ts_ms >= 60_000, "ts_ms must fall within the new bar's window");

        // Continuing bar B accrues correctly against the reset baseline.
        let bar_b1 = BarPoint { open_time: 60_000, open: 102.0, high: 103.0, low: 102.0, close: 103.0, volume: 2.0, quote_volume: 206.0, trades_count: 2 };
        let ev_b1 = st.to_delta_trade(&bar_b1, &tf, ExchangeId::Binance, "BTCUSDT").expect("must emit");
        assert!((extract_trade(&ev_b1).quantity - 0.8).abs() < 1e-12, "Δvolume within bar B = 2.0-1.2 = 0.8");
    }

    /// `seed_baseline` primes the adapter exactly as the cold-seed→live seam
    /// requires: a live update sharing the seeded `open_time` computes a
    /// delta against the seed's already-folded volume instead of re-folding
    /// (double-counting) the whole bar.
    #[test]
    fn kline_delta_state_seed_baseline_prevents_seam_double_count() {
        let mut st = KlineDeltaState::new();
        let tf = KlineInterval::new("1m");

        // Cold-seed folded this bar's volume up to 7.0 already (as either a
        // 4-leg synthetic path for a closed bar, or as the unclosed-bar
        // baseline handoff — either way, 7.0 units are already accounted).
        st.seed_baseline(120_000, 7.0, 105.0);

        // Live WS delivers the SAME open_time with more volume accrued.
        let bar = BarPoint { open_time: 120_000, open: 105.0, high: 106.0, low: 105.0, close: 106.0, volume: 9.5, quote_volume: 1000.0, trades_count: 3 };
        let ev = st.to_delta_trade(&bar, &tf, ExchangeId::Binance, "BTCUSDT").expect("must emit");
        let t = extract_trade(&ev);
        assert!((t.quantity - 2.5).abs() < 1e-12, "Δvolume against the SEEDED baseline = 9.5 - 7.0 = 2.5, not the full 9.5");
    }

    // -----------------------------------------------------------------------
    // Replay-mode equivalence tests
    //
    // Core guarantee: feeding the same fixture stream two ways —
    // (a) replay OFF, collecting every emission (take_pending + Option) and
    //     last-wins-merging by the machine's identity key, vs
    // (b) replay ON, collecting only closes+pending, then flush_replay —
    // must produce byte-identical FINAL arrays.
    // -----------------------------------------------------------------------

    fn tpo_key(freq_minutes: u16, source: TpoSource) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::FuturesCross, "BTCUSDT",
            crate::series::Kind::TpoProfile(freq_minutes, source))
    }

    /// Feed `events` through `d` with replay OFF, collecting every
    /// emission (pending, drained first, then the `Option` return) in
    /// order, exactly as a live consumer would see them.
    fn collect_live<D: DerivedStream>(d: &mut D, events: &[Event]) -> Vec<D::Output> {
        let mut out = Vec::new();
        for e in events {
            let last = d.on_upstream_event(e, 0);
            out.extend(d.take_pending());
            out.extend(last);
        }
        out
    }

    /// Feed `events` through `d` with replay ON: closes/pending only during
    /// the fold, then one `flush_replay` at the end for the final
    /// in-progress element.
    fn collect_replay<D: DerivedStream>(d: &mut D, events: &[Event]) -> Vec<D::Output> {
        d.set_replay(true);
        let mut out = Vec::new();
        for e in events {
            let last = d.on_upstream_event(e, 0);
            out.extend(d.take_pending());
            out.extend(last);
        }
        out.extend(d.flush_replay());
        d.set_replay(false);
        out
    }

    /// Last-wins merge by identity key, preserving the FIRST-SEEN key order
    /// (matches how a `Series::upsert_by_ts` consumer would end up with one
    /// slot per key, latest value wins, ordered by first appearance).
    fn last_wins_merge<T: Clone, K: Eq + std::hash::Hash + Clone>(
        items: &[T],
        key_of: impl Fn(&T) -> K,
    ) -> Vec<T> {
        let mut order: Vec<K> = Vec::new();
        let mut map: std::collections::HashMap<K, T> = std::collections::HashMap::new();
        for item in items {
            let k = key_of(item);
            if !map.contains_key(&k) {
                order.push(k.clone());
            }
            map.insert(k, item.clone());
        }
        order.into_iter().map(|k| map.remove(&k).expect("key was just inserted")).collect()
    }

    /// Generates a deterministic multi-bar-crossing trade fixture: N trades
    /// walking price in a sawtooth pattern so range/renko/tick-style
    /// machines close multiple elements (including gap-crossing chains).
    fn sawtooth_trades(n: usize, start_ts: i64) -> Vec<Event> {
        let mut evs = Vec::with_capacity(n);
        let mut price = 100.0_f64;
        for i in 0..n {
            // Oscillate with growing amplitude so both organic single-box
            // closes AND multi-box gap-crossing chains occur.
            let step = match i % 6 {
                0 => 0.3,
                1 => 0.3,
                2 => -0.2,
                3 => 2.5,   // gap-crossing jump
                4 => -0.1,
                _ => -1.8,  // gap-crossing jump the other way
            };
            price += step;
            evs.push(trade_event_side(
                start_ts + i as i64,
                price,
                1.0 + (i % 5) as f64 * 0.25,
                (i % 2) as u8,
            ));
        }
        evs
    }

    #[test]
    fn replay_equivalence_range_bar() {
        let key = range_bar_key(100_000_000); // $1 range
        let events = sawtooth_trades(300, 0);

        let mut live = TradeToRangeBarDerived::new_for_key(&key);
        let live_out = collect_live(&mut live, &events);
        let live_merged = last_wins_merge(&live_out, |b| b.open_time);

        let mut replay = TradeToRangeBarDerived::new_for_key(&key);
        let replay_out = collect_replay(&mut replay, &events);
        let replay_merged = last_wins_merge(&replay_out, |b| b.open_time);

        assert_eq!(live_merged.len(), replay_merged.len(), "same number of distinct bars");
        for (l, r) in live_merged.iter().zip(replay_merged.iter()) {
            assert_eq!(l.open_time, r.open_time);
            assert_eq!(l.open, r.open);
            assert_eq!(l.high, r.high);
            assert_eq!(l.low, r.low);
            assert_eq!(l.close, r.close);
            assert!((l.volume - r.volume).abs() < 1e-9);
            assert_eq!(l.trades_count, r.trades_count);
        }
    }

    /// Renko never emits an in-progress-brick snapshot in LIVE mode (only
    /// closes) — so `flush_replay`'s trailing in-progress snapshot is
    /// genuinely EXTRA information beyond anything live mode ever showed,
    /// not a duplicate of an existing live element. The core equivalence
    /// guarantee (closed elements match 1:1) is checked against the CLOSED
    /// bricks only; the flushed in-progress brick is checked separately for
    /// internal consistency (its own bottom/top/up must be sane), matching
    /// the "cap with the live edge" purpose the contract describes rather
    /// than claiming byte-identity with a live element that never existed.
    #[test]
    fn replay_equivalence_renko_bar() {
        let key = renko_bar_key(50_000_000, 2); // $0.50 box, reversal=2
        let events = sawtooth_trades(300, 0);

        let mut live = TradeToRenkoBarDerived::new_for_key(&key);
        let live_out = collect_live(&mut live, &events);
        let live_merged = last_wins_merge(&live_out, |b| b.open_time);

        let mut replay = TradeToRenkoBarDerived::new_for_key(&key);
        replay.set_replay(true);
        let mut replay_closed = Vec::new();
        for e in &events {
            let last = replay.on_upstream_event(e, 0);
            replay_closed.extend(replay.take_pending());
            replay_closed.extend(last);
        }
        let flushed = replay.flush_replay();
        replay.set_replay(false);
        let replay_merged = last_wins_merge(&replay_closed, |b| b.open_time);

        assert_eq!(live_merged.len(), replay_merged.len(), "same number of distinct CLOSED bricks");
        for (l, r) in live_merged.iter().zip(replay_merged.iter()) {
            assert_eq!(l.open_time, r.open_time);
            assert_eq!(l.bottom, r.bottom);
            assert_eq!(l.top, r.top);
            assert_eq!(l.up, r.up);
            assert!((l.volume - r.volume).abs() < 1e-9);
            assert_eq!(l.trades_count, r.trades_count);
            assert!((l.wick - r.wick).abs() < 1e-9);
        }

        // flush_replay's in-progress snapshot (the sawtooth fixture always
        // leaves a brick mid-formation) must be internally consistent.
        if let Some(f) = flushed {
            assert!(f.top > f.bottom, "in-progress brick must have a valid span");
            if f.up {
                assert_eq!(f.top - f.bottom, 0.5, "box_size = $0.50");
            } else {
                assert_eq!(f.top - f.bottom, 0.5);
            }
        }
    }

    #[test]
    fn replay_equivalence_tick_bar() {
        let key = tick_bar_key(7);
        let events = sawtooth_trades(300, 0);

        let mut live = TradeToTickBarDerived::new_for_key(&key);
        let live_out = collect_live(&mut live, &events);
        let live_merged = last_wins_merge(&live_out, |b| b.open_time);

        let mut replay = TradeToTickBarDerived::new_for_key(&key);
        let replay_out = collect_replay(&mut replay, &events);
        let replay_merged = last_wins_merge(&replay_out, |b| b.open_time);

        assert_eq!(live_merged.len(), replay_merged.len(), "same number of distinct bars");
        for (l, r) in live_merged.iter().zip(replay_merged.iter()) {
            assert_eq!(l.open_time, r.open_time);
            assert_eq!(l.open, r.open);
            assert_eq!(l.high, r.high);
            assert_eq!(l.low, r.low);
            assert_eq!(l.close, r.close);
            assert!((l.volume - r.volume).abs() < 1e-9);
            assert_eq!(l.trades_count, r.trades_count);
        }
    }

    #[test]
    fn replay_equivalence_footprint() {
        let key = footprint_key("1m");
        // Trades spanning several 1m buckets, with repeats at the same
        // price to exercise the levels accumulator.
        let mut events = Vec::new();
        let mut price = 100.0_f64;
        for bucket in 0..5_i64 {
            let base_ts = bucket * 60_000;
            for i in 0..40_i64 {
                price += if i % 3 == 0 { 0.1 } else { -0.05 };
                events.push(trade_event_side(base_ts + i * 1000, price, 1.0, (i % 2) as u8));
            }
        }

        let mut live = TradeToFootprintDerived::new_for_key(&key);
        let live_out = collect_live(&mut live, &events);
        let live_merged = last_wins_merge(&live_out, |p: &FootprintPoint| p.open_time);

        let mut replay = TradeToFootprintDerived::new_for_key(&key);
        let replay_out = collect_replay(&mut replay, &events);
        let replay_merged = last_wins_merge(&replay_out, |p: &FootprintPoint| p.open_time);

        assert_eq!(live_merged.len(), replay_merged.len(), "same number of distinct buckets");
        for (l, r) in live_merged.iter().zip(replay_merged.iter()) {
            assert_eq!(l.open_time, r.open_time);
            assert_eq!(l.open, r.open);
            assert_eq!(l.high, r.high);
            assert_eq!(l.low, r.low);
            assert_eq!(l.close, r.close);
            assert!((l.volume - r.volume).abs() < 1e-9);
            // Levels: same set of (price, buy, sell) triples (order is
            // insertion order into a BTreeMap keyed by price bits — both
            // paths fold the SAME trades in the SAME order, so the vecs
            // must match element-wise).
            assert_eq!(l.levels.len(), r.levels.len(), "bucket {}", l.open_time);
            for (lv_l, lv_r) in l.levels.iter().zip(r.levels.iter()) {
                assert!((lv_l.0 - lv_r.0).abs() < 1e-9);
                assert!((lv_l.1 - lv_r.1).abs() < 1e-9);
                assert!((lv_l.2 - lv_r.2).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn replay_equivalence_tpo_from_trade() {
        let key = tpo_key(30, TpoSource::TradeBucket);
        // Trades spanning 3 UTC days so the session rolls twice. Starts at
        // day offset 10 (NOT day 0 / ts_ms=0) — `TpoSessionState` uses
        // `session_date_ms == 0` as its "uninitialized" sentinel, so a
        // session that legitimately starts at Unix epoch 0 would collide
        // with that sentinel and silently skip its own roll-history
        // snapshot (pre-existing quirk, unrelated to the replay contract
        // under test here — sidestepped rather than masked).
        let mut events = Vec::new();
        let mut price = 100.0_f64;
        const DAY: i64 = 86_400_000;
        for day in 10..13_i64 {
            for i in 0..60_i64 {
                price += if i % 4 == 0 { 0.5 } else { -0.2 };
                events.push(trade_event_side(day * DAY + i * 60_000, price, 1.0, (i % 2) as u8));
            }
        }

        let mut live = TpoFromTradeDerived::new_for_key(&key);
        let live_out = collect_live(&mut live, &events);
        let live_merged = last_wins_merge(&live_out, |p: &TpoSessionPoint| p.open_time);

        let mut replay = TpoFromTradeDerived::new_for_key(&key);
        let replay_out = collect_replay(&mut replay, &events);
        let replay_merged = last_wins_merge(&replay_out, |p: &TpoSessionPoint| p.open_time);

        assert_eq!(live_merged.len(), replay_merged.len(), "same number of distinct sessions");
        for (l, r) in live_merged.iter().zip(replay_merged.iter()) {
            assert_eq!(l.open_time, r.open_time);
            assert!((l.tick_size - r.tick_size).abs() < 1e-9);
            assert!((l.session_high - r.session_high).abs() < 1e-9);
            assert!((l.session_low - r.session_low).abs() < 1e-9);
            assert!((l.poc_price - r.poc_price).abs() < 1e-9);
            assert!((l.vah_price - r.vah_price).abs() < 1e-9);
            assert!((l.val_price - r.val_price).abs() < 1e-9);
            assert_eq!(l.rows.len(), r.rows.len(), "session {}", l.open_time);
            for (row_l, row_r) in l.rows.iter().zip(r.rows.iter()) {
                assert!((row_l.0 - row_r.0).abs() < 1e-9);
                assert_eq!(row_l.1, row_r.1, "same letters at price {}", row_l.0);
            }
        }
    }

    /// Perf-shape guarantee for TPO: while replay is ON, feeding N trades
    /// within ONE still-open bucket/session must emit ZERO `Some(..)`
    /// results (no per-event `build_point` scan) until the session actually
    /// rolls or `flush_replay` is called. Counts allocations by counting
    /// `Some` returns from `on_upstream_event` directly (not via the
    /// `collect_replay` helper, so we can assert on the mid-stream count).
    #[test]
    fn tpo_replay_zero_emissions_until_roll_or_flush() {
        let key = tpo_key(30, TpoSource::TradeBucket);
        let mut d = TpoFromTradeDerived::new_for_key(&key);
        d.set_replay(true);

        let mut price = 100.0_f64;
        let mut emit_count = 0usize;
        // 5_000 trades, all within day 0 — session never rolls.
        for i in 0..5_000_i64 {
            price += if i % 7 == 0 { 0.2 } else { -0.05 };
            let ev = trade_event_side(i * 1000, price, 1.0, (i % 2) as u8);
            if d.on_upstream_event(&ev, 0).is_some() {
                emit_count += 1;
            }
            // TPO has no pending queue — take_pending stays empty, but call
            // it anyway to mirror the real call contract.
            assert!(d.take_pending().is_empty());
        }
        assert_eq!(emit_count, 0, "no bucket roll happened — zero emissions expected in replay mode");

        // flush_replay produces the one final in-progress snapshot, and
        // does NOT mutate state (clone-out only) — calling it again returns
        // the same non-empty snapshot rather than `None`.
        let flushed = d.flush_replay();
        assert!(flushed.is_some(), "flush_replay must return the in-progress session snapshot");
        let flushed_again = d.flush_replay();
        assert!(flushed_again.is_some(), "flush_replay is non-mutating — session data is still there");
        assert_eq!(flushed.unwrap().open_time, flushed_again.unwrap().open_time);
    }

    /// Perf-shape guarantee for TPO with a session roll mid-stream: exactly
    /// ONE emission occurs (at the roll), not one per trade.
    ///
    /// Starts at day offset 10 (not day 0 / ts_ms=0) — see the comment on
    /// [`replay_equivalence_tpo_from_trade`] about the `session_date_ms==0`
    /// sentinel collision this sidesteps.
    #[test]
    fn tpo_replay_single_emission_on_session_roll() {
        let key = tpo_key(30, TpoSource::TradeBucket);
        let mut d = TpoFromTradeDerived::new_for_key(&key);
        d.set_replay(true);

        const DAY: i64 = 86_400_000;
        let base = 10 * DAY;
        let mut price = 100.0_f64;
        let mut emit_count = 0usize;
        // Day 10: 2_000 trades (no roll). Day 11: 1 trade (triggers the roll).
        for i in 0..2_000_i64 {
            price += 0.1;
            let ev = trade_event_side(base + i * 1000, price, 1.0, (i % 2) as u8);
            if d.on_upstream_event(&ev, 0).is_some() {
                emit_count += 1;
            }
        }
        assert_eq!(emit_count, 0, "no roll yet");

        let roll_ev = trade_event_side(base + DAY + 1_000, price + 1.0, 1.0, 0);
        let rolled = d.on_upstream_event(&roll_ev, 0);
        assert!(rolled.is_some(), "session roll must emit the finalized PREVIOUS session");
        emit_count += 1;

        // 2_000 more trades in the new (day 11) session — still zero further
        // emissions until the NEXT roll or a flush.
        for i in 0..2_000_i64 {
            price += 0.1;
            let ev = trade_event_side(base + DAY + 2_000 + i * 1000, price, 1.0, (i % 2) as u8);
            if d.on_upstream_event(&ev, 0).is_some() {
                emit_count += 1;
            }
        }
        assert_eq!(emit_count, 1, "exactly one emission total: the single session roll");
    }
}
