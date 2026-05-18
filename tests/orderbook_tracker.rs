//! Unit tests for OrderBookTracker.

use digdigdig3::core::orderbook::{OrderBookError, OrderBookTracker};
use digdigdig3::core::types::{OrderBook, OrderBookLevel, OrderbookDelta};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn dec(v: f64) -> Decimal {
    Decimal::from_f64(v).expect("test value must convert to Decimal")
}

fn make_snapshot(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> OrderBook {
    OrderBook {
        bids: bids.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
        asks: asks.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
        timestamp: 1_700_000_000_000,
        last_update_id: Some(100),
        sequence: None,
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
    }
}

fn make_delta(
    bids: &[(f64, f64)],
    asks: &[(f64, f64)],
    prev_update_id: Option<u64>,
    last_update_id: Option<u64>,
) -> OrderbookDelta {
    OrderbookDelta {
        bids: bids.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
        asks: asks.iter().map(|&(p, s)| OrderBookLevel::new(p, s)).collect(),
        timestamp: 1_700_000_001_000,
        prev_update_id,
        last_update_id,
        first_update_id: None,
        event_time: None,
        checksum: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot populates book; delta updates levels; top N correct.
#[test]
fn snapshot_then_delta_consistent() {
    let mut tracker = OrderBookTracker::new("BTCUSDT");

    let snapshot = make_snapshot(
        &[(50000.0, 1.0), (49900.0, 2.0), (49800.0, 3.0), (49700.0, 4.0), (49600.0, 5.0)],
        &[(50100.0, 1.0), (50200.0, 2.0), (50300.0, 3.0), (50400.0, 4.0), (50500.0, 5.0)],
    );
    tracker.apply_snapshot(&snapshot).unwrap();

    // Delta: update one bid, add one ask
    let delta = make_delta(
        &[(49900.0, 2.5)],     // update bid qty
        &[(50600.0, 6.0)],     // add new ask level
        Some(100),
        Some(101),
    );
    tracker.apply_delta(&delta).unwrap();

    let bids = tracker.top_bids(5);
    assert_eq!(bids.len(), 5);
    // Best bid still 50000
    assert_eq!(bids[0].0, dec(50000.0));
    // 49900 updated to 2.5
    assert_eq!(bids[1].0, dec(49900.0));
    assert_eq!(bids[1].1, dec(2.5));

    let asks = tracker.top_asks(5);
    assert_eq!(asks.len(), 5);
    assert_eq!(asks[0].0, dec(50100.0));

    // 6th ask should be the new 50600 level
    let all_asks = tracker.top_asks(6);
    assert_eq!(all_asks.len(), 6);
    assert_eq!(all_asks[5].0, dec(50600.0));
    assert_eq!(all_asks[5].1, dec(6.0));

    assert_eq!(tracker.last_update_id(), Some(101));
}

/// Delta with size 0.0 removes that price level.
#[test]
fn delta_zero_qty_removes_level() {
    let mut tracker = OrderBookTracker::new("ETHUSDT");
    let snapshot = make_snapshot(
        &[(3000.0, 1.0), (2990.0, 2.0)],
        &[(3010.0, 1.0), (3020.0, 2.0)],
    );
    tracker.apply_snapshot(&snapshot).unwrap();

    let delta = make_delta(
        &[(2990.0, 0.0)], // remove 2990 bid
        &[(3010.0, 0.0)], // remove 3010 ask
        Some(100),
        Some(101),
    );
    tracker.apply_delta(&delta).unwrap();

    let (bid_depth, ask_depth) = tracker.depth();
    assert_eq!(bid_depth, 1); // only 3000 remains
    assert_eq!(ask_depth, 1); // only 3020 remains

    let bids = tracker.top_bids(5);
    assert_eq!(bids[0].0, dec(3000.0));
    let asks = tracker.top_asks(5);
    assert_eq!(asks[0].0, dec(3020.0));
}

/// Deliberate sequence gap returns SequenceGap error.
#[test]
fn sequence_gap_detected() {
    let mut tracker = OrderBookTracker::new("BTCUSDT");
    let snapshot = make_snapshot(&[(50000.0, 1.0)], &[(50100.0, 1.0)]);
    tracker.apply_snapshot(&snapshot).unwrap();
    // last_update_id = 100

    // Send delta with prev_update_id = 99 instead of 100
    let delta = make_delta(&[], &[], Some(99), Some(101));
    let err = tracker.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrderBookError::SequenceGap { last: 100, got: 99 }));

    // Book unchanged after gap error
    assert_eq!(tracker.last_update_id(), Some(100));
}

/// BBO, mid, spread calculations on known book.
#[test]
fn bbo_and_mid_and_spread() {
    let mut tracker = OrderBookTracker::new("BTCUSDT");
    let snapshot = make_snapshot(
        &[(50000.0, 1.0), (49900.0, 2.0)],
        &[(50100.0, 1.0), (50200.0, 2.0)],
    );
    tracker.apply_snapshot(&snapshot).unwrap();

    let (best_bid, best_ask) = tracker.bbo().unwrap();
    assert_eq!(best_bid, dec(50000.0));
    assert_eq!(best_ask, dec(50100.0));

    let mid = tracker.mid().unwrap();
    // (50000 + 50100) / 2 = 50050
    assert_eq!(mid, dec(50050.0));

    let spread = tracker.spread().unwrap();
    // 50100 - 50000 = 100
    assert_eq!(spread, dec(100.0));
}

/// Symbol mismatch — tracker validates symbol on deltas if we add that check.
/// (OrderBook has no symbol field; we skip snapshot mismatch.)
/// Verify that a SymbolMismatch error is NOT produced for snapshots (no symbol field).
#[test]
fn empty_book_handling() {
    let tracker = OrderBookTracker::new("BTCUSDT");
    assert!(tracker.top_bids(5).is_empty());
    assert!(tracker.top_asks(5).is_empty());
    assert!(tracker.bbo().is_none());
    assert!(tracker.mid().is_none());
    assert!(tracker.spread().is_none());
    assert_eq!(tracker.total_bid_volume(), Decimal::ZERO);
    assert_eq!(tracker.total_ask_volume(), Decimal::ZERO);
    assert_eq!(tracker.depth(), (0, 0));
    assert!(!tracker.has_snapshot());
}

/// Delta applied before snapshot returns NoSnapshot.
#[test]
fn delta_before_snapshot_errors() {
    let mut tracker = OrderBookTracker::new("SOLUSDT");
    let delta = make_delta(&[(100.0, 1.0)], &[], None, Some(1));
    let err = tracker.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrderBookError::NoSnapshot));
}

/// reset() clears the book and allows re-apply.
#[test]
fn reset_clears_state() {
    let mut tracker = OrderBookTracker::new("BTCUSDT");
    tracker.apply_snapshot(&make_snapshot(&[(50000.0, 1.0)], &[(50100.0, 1.0)])).unwrap();
    tracker.reset();

    assert!(!tracker.has_snapshot());
    assert_eq!(tracker.depth(), (0, 0));
    assert!(tracker.bbo().is_none());

    // Can apply snapshot again after reset
    tracker.apply_snapshot(&make_snapshot(&[(49000.0, 2.0)], &[(49100.0, 2.0)])).unwrap();
    let (b, _) = tracker.bbo().unwrap();
    assert_eq!(b, dec(49000.0));
}

/// total_bid_volume / total_ask_volume sum correctly.
#[test]
fn total_volume_correct() {
    let mut tracker = OrderBookTracker::new("BTCUSDT");
    tracker.apply_snapshot(&make_snapshot(
        &[(50000.0, 1.0), (49900.0, 2.0), (49800.0, 3.0)],
        &[(50100.0, 4.0), (50200.0, 5.0)],
    )).unwrap();

    assert_eq!(tracker.total_bid_volume(), dec(6.0));  // 1+2+3
    assert_eq!(tracker.total_ask_volume(), dec(9.0)); // 4+5
}
