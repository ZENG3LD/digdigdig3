//! Derived-stream layer for `digdigdig3-station`.
//!
//! A *derived stream* is a Station-internal computation that subscribes to one
//! or more upstream WS-backed streams and emits events of its own type. It runs
//! as a standalone `tokio::spawn` task per `SeriesKey`, sharing the same
//! `DiskStore<T>` / `Series<T>` / `broadcast::channel<Event>` plumbing as
//! regular WS forwarders. Consumers see no difference.
//!
//! Three concrete impls ship in this module:
//!
//! - [`BasisDerived`] — joins `MarkPrice` + `IndexPrice`, emits
//!   `BasisPoint { value = mark − index }`. Rejects pairs skewed > 2 seconds.
//! - [`FundingSettlementDerived`] — monitors `FundingRate`, emits
//!   `FundingSettlementPoint` each time `next_funding_time` advances past the
//!   current wall clock (crossing-detector pattern).
//! - [`TradeToBarDerived`] — subscribes to `Trade` and aggregates individual
//!   trades into OHLCV bars of a fixed interval. Used as a fallback when the
//!   venue's WS does not natively offer the requested `Kind::Kline(interval)`.

use crate::data::{BarPoint, BasisPoint, FundingRatePoint, FundingSettlementPoint, MarkPricePoint, IndexPricePoint, TradePoint};
use crate::series::{DataPoint, Kind};
use crate::series::SeriesKey;
use crate::subscription::{Event, Stream};

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

    /// Upstream `Stream` variants this derived stream needs. Called once at
    /// spawn time to determine which upstream multiplexers to acquire. Order
    /// is significant — `on_upstream_event` receives `dep_idx` matching the
    /// index of the stream in this slice.
    fn deps() -> &'static [Stream];

    /// Construct initial (empty) state for the given key. Called once before
    /// the forwarder loop begins.
    fn new_for_key(key: &SeriesKey) -> Self;

    /// Process one upstream `Event`. Returns `Some(point)` if a derived output
    /// should be emitted, `None` to silently absorb the event.
    ///
    /// `dep_idx` is the index into `Self::deps()` that produced this event,
    /// allowing implementations to branch without repeated pattern-matching.
    fn on_upstream_event(&mut self, ev: &Event, dep_idx: usize) -> Option<Self::Output>;
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

// Silence dead-code lint: MarkPricePoint + IndexPricePoint are used via
// Event destructuring; FundingRatePoint via FundingSettlementDerived below.
// These use-less imports are needed to keep the type names in scope for docs
// and to avoid the compiler complaining about the fields being unused.
// Actually they are just suppressed at import level — not needed. Removed.

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
};

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
}
