//! Derived-stream layer for `digdigdig3-station`.
//!
//! A *derived stream* is a Station-internal computation that subscribes to one
//! or more upstream WS-backed streams and emits events of its own type. It runs
//! as a standalone `tokio::spawn` task per `SeriesKey`, sharing the same
//! `DiskStore<T>` / `Series<T>` / `broadcast::channel<Event>` plumbing as
//! regular WS forwarders. Consumers see no difference.
//!
//! Two concrete impls ship in this module:
//!
//! - [`BasisDerived`] — joins `MarkPrice` + `IndexPrice`, emits
//!   `BasisPoint { value = mark − index }`. Rejects pairs skewed > 2 seconds.
//! - [`FundingSettlementDerived`] — monitors `FundingRate`, emits
//!   `FundingSettlementPoint` each time `next_funding_time` advances past the
//!   current wall clock (crossing-detector pattern).
//!
//! The trait is `pub(crate)` — it is a Station-internal abstraction. External
//! code never needs to name `DerivedStream` directly; users simply call
//! `SubscriptionSet::add(…, [Stream::Basis])` as usual.

use crate::data::{BasisPoint, FundingRatePoint, FundingSettlementPoint, MarkPricePoint, IndexPricePoint};
use crate::series::DataPoint;
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
};

// ---------------------------------------------------------------------------
// Unit tests (inside module — need access to pub(crate) types)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use digdigdig3::core::types::ExchangeId;
    use digdigdig3::core::types::AccountType;

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
}
