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
//! - [`TradeToRangeBarDerived`] — emits a new [`BarPoint`] when the price
//!   moves ≥ `range` away from the current bar's open price.
//! - [`TradeToTickBarDerived`] — closes a bar every `n` trades.
//! - [`TradeToVolumeBarDerived`] — closes a bar when cumulative volume ≥ threshold.
//! - [`TradeToFootprintDerived`] — time-bucketed OHLCV with per-price buy/sell breakdown.
//! - [`TradeToDollarBarDerived`] — López de Prado dollar bar (close on cumulative dollar volume).
//! - [`TradeToTickImbalanceDerived`] — López de Prado Tick Imbalance Bar.
//! - [`TradeToVolumeImbalanceDerived`] — López de Prado Volume Imbalance Bar.
//! - [`TradeToRunBarDerived`] — López de Prado Run Bar.

use crate::data::{
    BarPoint, BasisPoint, FootprintPoint, FundingRatePoint, FundingSettlementPoint,
    KagiSegmentPoint, MarkPricePoint, IndexPricePoint, PnfColumnPoint,
    RenkoBrickPoint, ScalarBarPoint, TpoSessionPoint, TradePoint,
};
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
pub(crate) trait DerivedStream: Send + 'static {
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
    fn on_upstream_event(&mut self, ev: &Event, dep_idx: usize) -> Option<Self::Output>;

    /// Seed internal state from a batch of upstream events at spawn time, before
    /// the live event loop begins. Returns all derived outputs emitted during the
    /// seed pass so callers can broadcast them (cold-start window visible to
    /// consumers immediately).
    ///
    /// `dep_idx` matches the index in `Self::deps()`. Default implementation
    /// feeds each event through `on_upstream_event` and collects results — works
    /// for all trade-derived impls without any additional code.
    fn seed_from_events(&mut self, events: &[Event], dep_idx: usize) -> Vec<Self::Output> {
        events.iter().filter_map(|e| self.on_upstream_event(e, dep_idx)).collect()
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
pub(crate) struct BasisDerived {
    /// Most recent (ts_ms, mark_price) from the MarkPrice upstream.
    last_mark: Option<(i64, f64)>,
    /// Most recent (ts_ms, index_price) from the IndexPrice upstream.
    last_index: Option<(i64, f64)>,
    /// Maximum allowed age difference between the two sides (milliseconds).
    max_skew_ms: i64,
}

impl DerivedStream for BasisDerived {
    type Output = BasisPoint;

    fn deps() -> &'static [Stream] {
        &[Stream::MarkPrice, Stream::IndexPrice]
    }

    fn new_for_key(_key: &SeriesKey) -> Self {
        Self {
            last_mark: None,
            last_index: None,
            max_skew_ms: 2_000,
        }
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
pub(crate) fn interval_to_ms(interval: &str) -> Option<i64> {
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
pub(crate) struct TradeToBarDerived {
    /// Bucket width in milliseconds. `0` means "disabled" (unknown interval).
    interval_ms: i64,
    /// The currently-open (partial) bar, if any.
    current: Option<BarPoint>,
    /// Bucket start of `current` (ms). `0` when `current` is `None`.
    current_bucket_start: i64,
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
        Self { interval_ms, current: None, current_bucket_start: 0 }
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
/// ## Bar semantics
///
/// `range` (from `Kind::RangeBar(r)`) is stored as `r / 1e8` internally.
/// A new bar opens when `|trade.price − bar_open| >= range`. The crossing trade
/// belongs to the **new** bar (not the closing one).
///
/// ## Monotonic `open_time`
///
/// Range/tick/volume bars can close in the same millisecond. `Series::upsert_by_ts`
/// keys on `open_time`, so two distinct bars with equal ms would collapse.
/// To prevent this, `open_time` is `max(first_trade_ts, last_emitted_open_time + 1)`.
/// This guarantees strict monotonicity across all emitted bars without altering
/// the semantic meaning of `open_time` (it remains the timestamp of the first
/// trade in the bar, or 1 ms later if that timestamp collides).
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
pub(crate) struct TradeToRangeBarDerived {
    /// Minimum price movement to close bar, in native price units (`param / 1e8`).
    /// `0.0` means disabled.
    range: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// The `open_time` of the last bar that was finalized (emitted as a closed bar).
    /// Used for monotonic open_time collision avoidance.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToRangeBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let range = match &key.kind {
            Kind::RangeBar(r) if *r > 0 => *r as f64 / 1e8,
            _ => 0.0,
        };
        Self { range, current: None, last_emitted_open_time: 0 }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.range == 0.0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        if let Some(ref mut bar) = self.current {
            // Check close condition BEFORE updating.
            if (point.price - bar.open).abs() >= self.range {
                // Finalize current bar (no update with crossing trade — it starts the new bar).
                let closed = bar.clone();
                // Emit the closed bar. open_time is already set.

                // Open new bar at crossing trade.
                let new_open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = new_open_time;
                self.current = Some(BarPoint {
                    open_time:   new_open_time,
                    open:        point.price,
                    high:        point.price,
                    low:         point.price,
                    close:       point.price,
                    volume:      point.quantity,
                    quote_volume: point.price * point.quantity,
                    trades_count: 1,
                });
                // Emit the new (open) bar rather than the closed one so callers
                // receive the in-progress state immediately (same contract as
                // TradeToBarDerived). The closed bar was already emitted on the
                // previous trade call that reached bar.close == crossing trade.
                // Returning the new bar gives the consumer one event per trade.
                let _ = closed; // closed bar state committed
                return Some(self.current.clone().unwrap());
            }
            // Update in-place.
            if point.price > bar.high  { bar.high  = point.price; }
            if point.price < bar.low   { bar.low   = point.price; }
            bar.close        = point.price;
            bar.volume       += point.quantity;
            bar.quote_volume += point.price * point.quantity;
            bar.trades_count += 1;
            Some(bar.clone())
        } else {
            // First trade — open a new bar.
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
            Some(bar)
        }
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
pub(crate) struct TradeToTickBarDerived {
    /// Trades per bar. `0` means disabled.
    n: u32,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// Trades accumulated in the current bar.
    count: u32,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToTickBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let n = match &key.kind {
            Kind::TickBar(n) if *n > 0 => *n,
            _ => 0,
        };
        Self { n, current: None, count: 0, last_emitted_open_time: 0 }
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
        if self.count >= self.n {
            // Next bar will be opened on the next trade.
            self.current = None;
            self.count = 0;
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
/// See [`TradeToRangeBarDerived`] for monotonic `open_time` and emit semantics.
pub(crate) struct TradeToVolumeBarDerived {
    /// Volume threshold in native units (`param / 1e8`). `0.0` means disabled.
    threshold: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToVolumeBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let threshold = match &key.kind {
            Kind::VolumeBar(v) if *v > 0 => *v as f64 / 1e8,
            _ => 0.0,
        };
        Self { threshold, current: None, last_emitted_open_time: 0 }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.threshold == 0.0 { return None; }
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
        if bar.volume >= self.threshold {
            self.current = None;
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
/// Price key is the raw `f64` bit-pattern (`u64::from_le_bytes(price.to_le_bytes())`),
/// which gives exact equality on repeated same-price trades without float precision loss.
///
/// ## Emit semantics
///
/// The current open footprint is emitted on **every trade** via `upsert_by_ts`
/// (same contract as `TradeToBarDerived`). On bucket roll, the new bar is emitted.
///
/// ## Disabled guard
///
/// `interval_ms == 0` → always returns `None`.
pub(crate) struct TradeToFootprintDerived {
    /// Bucket width in milliseconds. `0` = disabled.
    interval_ms: i64,
    /// Current bucket start.
    current_bucket_start: i64,
    /// OHLCV of the current bucket.
    current_ohlcv: Option<(f64, f64, f64, f64, f64)>, // open, high, low, close, volume
    /// Per-price accumulator: price_bits → (buy_vol, sell_vol).
    levels: std::collections::BTreeMap<u64, (f64, f64)>,
}

impl TradeToFootprintDerived {
    fn price_bits(price: f64) -> u64 {
        u64::from_le_bytes(price.to_le_bytes())
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

impl DerivedStream for TradeToFootprintDerived {
    type Output = FootprintPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let interval_ms = match &key.kind {
            Kind::Footprint(iv) => interval_to_ms(iv.as_str()).unwrap_or(0),
            _ => 0,
        };
        Self {
            interval_ms,
            current_bucket_start: 0,
            current_ohlcv: None,
            levels: std::collections::BTreeMap::new(),
        }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<FootprintPoint> {
        if self.interval_ms == 0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        let bucket_start = (point.ts_ms / self.interval_ms) * self.interval_ms;

        if self.current_ohlcv.is_none() || bucket_start > self.current_bucket_start {
            // Roll to new bucket.
            self.current_bucket_start = bucket_start;
            self.current_ohlcv = Some((point.price, point.price, point.price, point.price, point.quantity));
            self.levels.clear();
            // First level entry.
            let bits = Self::price_bits(point.price);
            let entry = self.levels.entry(bits).or_insert((0.0, 0.0));
            if point.side == 0 { entry.0 += point.quantity; } else { entry.1 += point.quantity; }
        } else {
            // Update current bucket.
            let ohlcv = self.current_ohlcv.as_mut()?;
            if point.price > ohlcv.1 { ohlcv.1 = point.price; } // high
            if point.price < ohlcv.2 { ohlcv.2 = point.price; } // low
            ohlcv.3 = point.price; // close
            ohlcv.4 += point.quantity; // volume
            let bits = Self::price_bits(point.price);
            let entry = self.levels.entry(bits).or_insert((0.0, 0.0));
            if point.side == 0 { entry.0 += point.quantity; } else { entry.1 += point.quantity; }
        }

        Some(self.build_point(self.current_bucket_start))
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
/// `box_size` in the current direction. Reversal requires
/// `reversal_count × box_size` of opposing movement; when it fires, ONE
/// brick is consumed (mplfinance gap-fill rule) and the remainder emit.
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
pub(crate) struct TradeToRenkoBarDerived {
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
    /// Monotonic open_time guard — same as range/tick/volume bars.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToRenkoBarDerived {
    type Output = RenkoBrickPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (box_size, reversal_count) = match &key.kind {
            Kind::RenkoBar(b, r) if *b > 0 => (*b as f64 / 1e8, (*r).max(1)),
            _ => (0.0, 1),
        };
        Self {
            box_size,
            reversal_count,
            anchor: 0.0,
            last_dir: None,
            vol_acc: 0.0,
            tcount_acc: 0,
            last_emitted_open_time: 0,
        }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<RenkoBrickPoint> {
        if self.box_size == 0.0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

        // Seed anchor on the very first trade.
        if self.last_dir.is_none() && self.anchor == 0.0 {
            self.anchor = (point.price / self.box_size).floor() * self.box_size;
        }

        self.vol_acc += point.quantity;
        self.tcount_acc += 1;

        let mut last_emitted: Option<RenkoBrickPoint> = None;

        loop {
            let up_target = self.anchor + self.box_size;
            let down_target = self.anchor - self.box_size;

            if point.price >= up_target {
                if matches!(self.last_dir, Some(false)) {
                    let needed = self.anchor + self.box_size * self.reversal_count as f64;
                    if point.price < needed { break; }
                    self.anchor += self.box_size;
                }
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                let brick = RenkoBrickPoint {
                    open_time,
                    bottom: self.anchor,
                    top: self.anchor + self.box_size,
                    up: true,
                    volume: self.vol_acc,
                    trades_count: self.tcount_acc,
                };
                self.vol_acc = 0.0;
                self.tcount_acc = 0;
                self.anchor += self.box_size;
                self.last_dir = Some(true);
                last_emitted = Some(brick);
                continue;
            }

            if point.price <= down_target {
                if matches!(self.last_dir, Some(true)) {
                    let needed = self.anchor - self.box_size * self.reversal_count as f64;
                    if point.price > needed { break; }
                    self.anchor -= self.box_size;
                }
                let open_time = point.ts_ms.max(self.last_emitted_open_time + 1);
                self.last_emitted_open_time = open_time;
                let brick = RenkoBrickPoint {
                    open_time,
                    bottom: self.anchor - self.box_size,
                    top: self.anchor,
                    up: false,
                    volume: self.vol_acc,
                    trades_count: self.tcount_acc,
                };
                self.vol_acc = 0.0;
                self.tcount_acc = 0;
                self.anchor -= self.box_size;
                self.last_dir = Some(false);
                last_emitted = Some(brick);
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
pub(crate) struct TradeToPnfBarDerived {
    box_size: f64,
    reversal_count: u8,
    /// Currently-open column, if any.
    cur: Option<PnfColumnPoint>,
    /// Monotonic column id counter; first column = 1.
    next_column_id: u64,
    /// Monotonic open_time guard.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToPnfBarDerived {
    type Output = PnfColumnPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let (box_size, reversal_count) = match &key.kind {
            Kind::PnfBar(b, r) if *b > 0 => (*b as f64 / 1e8, (*r).max(1)),
            _ => (0.0, 3),
        };
        Self {
            box_size,
            reversal_count,
            cur: None,
            next_column_id: 1,
            last_emitted_open_time: 0,
        }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<PnfColumnPoint> {
        if self.box_size == 0.0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

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
            // Extend up if price clears next box top.
            if point.price >= col.top + self.box_size {
                let steps = ((point.price - col.top) / self.box_size).floor() as i64;
                col.top += steps as f64 * self.box_size;
                return self.cur.clone();
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
                return self.cur.clone();
            }
        } else {
            // Extend down if price clears next box bottom.
            if point.price <= col.bottom - self.box_size {
                let steps = ((col.bottom - point.price) / self.box_size).floor() as i64;
                col.bottom -= steps as f64 * self.box_size;
                return self.cur.clone();
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
/// the opposite direction. Yang/yin thickness flips at the prior
/// shoulder (highest high) or waist (lowest low) — when a down segment
/// crosses *below* the prior waist it flips to yin (thin); when an up
/// segment crosses *above* the prior shoulder it flips to yang (thick).
///
/// ## Output (`KagiSegmentPoint`)
///
/// Two flavours:
/// 1. Vertical segment: `is_connector = false`, `start_price`,
///    `end_price`, `yang: bool`.
/// 2. Horizontal connector: `is_connector = true`, emitted at the close
///    of each segment (price = the segment's terminal price).
///
/// ## Disabled guard: `reversal == 0.0`.
pub(crate) struct TradeToKagiBarDerived {
    reversal: f64,
    /// Anchor price for the current direction.
    anchor: f64,
    /// True once the seed direction has been determined.
    seeded: bool,
    /// Current direction.
    up: bool,
    /// Last seen shoulder (highest high across closed segments).
    last_shoulder: f64,
    /// Last seen waist (lowest low across closed segments).
    last_waist: f64,
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToKagiBarDerived {
    type Output = KagiSegmentPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let reversal = match &key.kind {
            Kind::KagiBar(r) if *r > 0 => *r as f64 / 1e8,
            _ => 0.0,
        };
        Self {
            reversal,
            anchor: 0.0,
            seeded: false,
            up: true,
            last_shoulder: f64::NEG_INFINITY,
            last_waist: f64::INFINITY,
            last_emitted_open_time: 0,
        }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<KagiSegmentPoint> {
        if self.reversal == 0.0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

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

        // Active phase.
        if self.up && point.price > self.anchor {
            self.anchor = point.price;
            return None;
        }
        if !self.up && point.price < self.anchor {
            self.anchor = point.price;
            return None;
        }

        // Candidate reversal: price moved opposite by ≥ reversal.
        let against = if self.up { self.anchor - point.price }
                      else { point.price - self.anchor };
        if against < self.reversal { return None; }

        // Close current segment.
        let yang = if self.up {
            // Up segment closing — yang determined by whether anchor cleared
            // the prior shoulder during this segment.
            self.anchor > self.last_shoulder
        } else {
            // Down segment closing — yang (thick) when anchor breached prior
            // waist (a "yin" flip in classical Kagi means the price made a
            // new LOW below the prior waist).
            self.anchor < self.last_waist
        };
        // Persist extremum.
        if self.up { self.last_shoulder = self.last_shoulder.max(self.anchor); }
        else { self.last_waist = self.last_waist.min(self.anchor); }

        let close_ts = point.ts_ms.max(self.last_emitted_open_time + 1);
        self.last_emitted_open_time = close_ts;
        let closed = KagiSegmentPoint {
            open_time: close_ts,
            start_price: if self.up { self.anchor - against - 0.0 } else { self.anchor + against - 0.0 },
            end_price: self.anchor,
            up: self.up,
            yang,
            is_connector: false,
        };
        // Flip direction; new anchor is the trade that triggered the flip.
        self.up = !self.up;
        self.anchor = point.price;

        Some(closed)
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
pub(crate) struct TradeToCvdLineDerived {
    enabled: bool,
    cvd: f64,
    last_emitted_ts: i64,
}

impl DerivedStream for TradeToCvdLineDerived {
    type Output = ScalarBarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let enabled = matches!(key.kind, Kind::CvdLine);
        Self { enabled, cvd: 0.0, last_emitted_ts: 0 }
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
pub(crate) struct TpoSessionState {
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

    fn maybe_roll_session(&mut self, ts_ms: i64) {
        let bar_day = Self::day_start_ms(ts_ms);
        if bar_day != self.session_date_ms {
            self.session_date_ms = bar_day;
            self.buckets.clear();
            self.samples.clear();
            self.tick_size = 0.0;
            self.session_high = f64::NEG_INFINITY;
            self.session_low = f64::INFINITY;
        }
    }

    /// Feed one (ts_ms, high, low, price_sample) tuple into the current
    /// session. `price_sample` is used for tick-size auto-detection
    /// (kline mode: close; trade mode: trade price).
    fn extend_bucket(&mut self, ts_ms: i64, high: f64, low: f64, price_sample: f64) {
        self.maybe_roll_session(ts_ms);

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
pub(crate) struct TpoFromKline1mDerived {
    state: TpoSessionState,
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
        Self { state: TpoSessionState::new(freq_minutes) }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<TpoSessionPoint> {
        let Event::Bar { point, .. } = ev else { return None };
        self.state.extend_bucket(point.open_time, point.high, point.low, point.close);
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
pub(crate) struct TpoFromTradeDerived {
    state: TpoSessionState,
}

impl DerivedStream for TpoFromTradeDerived {
    type Output = TpoSessionPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let freq_minutes = match &key.kind {
            Kind::TpoProfile(f, TpoSource::TradeBucket) if *f > 0 => *f,
            _ => 30,
        };
        Self { state: TpoSessionState::new(freq_minutes) }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<TpoSessionPoint> {
        let Event::Trade { point, .. } = ev else { return None };
        // A single trade is a degenerate `[price, price]` bar; the
        // bucket's high/low grow as subsequent trades arrive.
        self.state.extend_bucket(point.ts_ms, point.price, point.price, point.price);
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
pub(crate) struct FundingSettlementDerived {
    /// Last seen `next_funding_time_ms`. 0 = uninitialized.
    last_next_funding_time: i64,
    /// Funding rate carried from the previous event (the one active during the
    /// *just-settled* period).
    last_rate: f64,
}

impl DerivedStream for FundingSettlementDerived {
    type Output = FundingSettlementPoint;

    fn deps() -> &'static [Stream] {
        &[Stream::FundingRate]
    }

    fn new_for_key(_key: &SeriesKey) -> Self {
        Self {
            last_next_funding_time: 0,
            last_rate: 0.0,
        }
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
/// ## Disabled guard
///
/// `threshold == 0.0` ⇒ always `None`.
pub(crate) struct TradeToDollarBarDerived {
    /// Dollar threshold in native units (`dollar_threshold` as f64). `0.0` = disabled.
    threshold: f64,
    /// Accumulated dollar value in the current bar.
    cumulative_dollars: f64,
    /// Currently-open bar, if any.
    current: Option<BarPoint>,
    /// See [`TradeToRangeBarDerived`] doc.
    last_emitted_open_time: i64,
}

impl DerivedStream for TradeToDollarBarDerived {
    type Output = BarPoint;

    fn deps() -> &'static [Stream] { &[Stream::Trade] }

    fn new_for_key(key: &SeriesKey) -> Self {
        let threshold = match &key.kind {
            Kind::DollarBar { dollar_threshold } if *dollar_threshold > 0 => *dollar_threshold as f64,
            _ => 0.0,
        };
        Self { threshold, cumulative_dollars: 0.0, current: None, last_emitted_open_time: 0 }
    }

    fn on_upstream_event(&mut self, ev: &Event, _dep_idx: usize) -> Option<BarPoint> {
        if self.threshold == 0.0 { return None; }
        let Event::Trade { point, .. } = ev else { return None };

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
        if self.cumulative_dollars >= self.threshold {
            self.current = None;
            self.cumulative_dollars = 0.0;
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
pub(crate) struct TradeToTickImbalanceDerived {
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
}

impl TradeToTickImbalanceDerived {
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
        // EMA seed: E[T] = min_ticks, E[|θ|] = min_ticks × 0.5 so the first
        // bar closes at roughly min_ticks/2 ticks of net imbalance.
        let expected_ticks = min_ticks as f64;
        let expected_theta_abs = min_ticks as f64 * 0.5;
        Self {
            alpha, min_ticks, theta: 0.0, tick_count: 0,
            expected_theta_abs, expected_ticks,
            current: None, last_emitted_open_time: 0,
        }
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
        if self.theta.abs() >= self.close_threshold() {
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
pub(crate) struct TradeToVolumeImbalanceDerived {
    alpha: f64,
    min_ticks: u32,
    /// Running signed-volume imbalance for the current bar.
    theta: f64,
    tick_count: u32,
    expected_theta_abs: f64,
    expected_ticks: f64,
    current: Option<BarPoint>,
    last_emitted_open_time: i64,
}

impl TradeToVolumeImbalanceDerived {
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
        let expected_ticks = min_ticks as f64;
        // EMA seed for signed-volume: assume average trade size ≈ 1.0, so
        // E[|θ|] seed = min_ticks * 0.5 × 1.0 = min_ticks * 0.5.
        let expected_theta_abs = min_ticks as f64 * 0.5;
        Self {
            alpha, min_ticks, theta: 0.0, tick_count: 0,
            expected_theta_abs, expected_ticks,
            current: None, last_emitted_open_time: 0,
        }
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

        if self.theta.abs() >= self.close_threshold() {
            let t = self.tick_count as f64;
            let theta_abs = self.theta.abs();
            self.expected_ticks = self.alpha * t + (1.0 - self.alpha) * self.expected_ticks;
            self.expected_theta_abs = self.alpha * theta_abs + (1.0 - self.alpha) * self.expected_theta_abs;
            self.current = None;
            self.theta = 0.0;
            self.tick_count = 0;
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
pub(crate) struct TradeToRunBarDerived {
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
}

impl TradeToRunBarDerived {
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
        let expected_ticks = min_ticks as f64;
        // EMA seed: expected run ≈ 25% of bar tick count on a balanced market.
        let expected_run = min_ticks as f64 * 0.25;
        Self {
            alpha, min_ticks, run_buy_len: 0, run_sell_len: 0, last_side: u8::MAX,
            expected_run, expected_ticks, tick_count: 0,
            current: None, last_emitted_open_time: 0,
        }
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
        if max_run >= self.close_threshold() {
            let t = self.tick_count as f64;
            self.expected_ticks = self.alpha * t + (1.0 - self.alpha) * self.expected_ticks;
            self.expected_run = self.alpha * max_run + (1.0 - self.alpha) * self.expected_run;
            // Reset bar state but NOT run lengths — runs carry across bar boundaries.
            self.current = None;
            self.tick_count = 0;
        }

        Some(bar)
    }
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

    /// seed_from_events for RangeBar primes last_emitted_open_time so
    /// subsequent live trades produce monotonic open_times.
    #[test]
    fn seed_from_events_range_bar_state_primed() {
        let key = range_bar_key(100_000_000); // $1 range
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        // Seed: open a bar at 100.0.
        let evs = vec![trade_event(0, 100.0, 1.0)];
        let emitted = d.seed_from_events(&evs, 0);
        assert_eq!(emitted.len(), 1);

        // State is primed — live event crosses range and opens new bar.
        let live = d.on_upstream_event(&trade_event(1, 101.0, 1.0), 0).unwrap();
        assert_eq!(live.open, 101.0, "new bar at crossing price");
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

    /// Trades within range stay in one bar.
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

    /// Crossing range opens a new bar.
    #[test]
    fn range_bar_rolls_on_crossing() {
        // range = $1.00
        let key = range_bar_key(100_000_000);
        let mut d = TradeToRangeBarDerived::new_for_key(&key);

        d.on_upstream_event(&trade_event(0, 100.0, 1.0), 0).unwrap();
        // Exactly $1 movement — crosses.
        let p = d.on_upstream_event(&trade_event(10, 101.0, 2.0), 0).unwrap();
        // New bar started at 101.0.
        assert_eq!(p.open, 101.0, "new bar opens at crossing price");
        assert_eq!(p.trades_count, 1, "first trade in new bar");
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

        // Trade 3: opens new bar.
        let p3 = d.on_upstream_event(&trade_event(2, 102.0, 0.5), 0).unwrap();
        assert_eq!(p3.open, 102.0, "new bar");
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

    /// Zero threshold → no emission.
    #[test]
    fn volume_bar_zero_threshold_safe() {
        let key = volume_bar_key(0);
        let mut d = TradeToVolumeBarDerived::new_for_key(&key);
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
}
