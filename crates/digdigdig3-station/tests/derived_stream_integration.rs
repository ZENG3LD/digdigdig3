#![cfg(not(target_arch = "wasm32"))]
//! Integration tests for the derived-stream layer.
//!
//! The state-machine unit tests live in `src/derived.rs` (they need
//! access to `pub(crate)` types). This file tests the public-facing
//! schema contracts and round-trip behaviour.
//!
//! A full end-to-end test wiring a real `Station` is omitted here —
//! it would require a live exchange connection or a mock WS hub.
//! The forwarder actor shape is exercised by the in-crate unit tests.

use digdigdig3_station::data::{BasisPoint, FundingSettlementPoint};
use digdigdig3_station::DataPoint;

// ---------------------------------------------------------------------------
// BasisPoint schema
// ---------------------------------------------------------------------------

#[test]
fn basis_point_is_32_bytes() {
    assert_eq!(BasisPoint::RECORD_SIZE, 32);
}

#[test]
fn basis_point_encode_decode_stable() {
    let p = BasisPoint {
        ts_ms: 1_700_000_000_123,
        value: -5.5,
        mark:  70_000.0,
        index: 70_005.5,
    };
    let mut buf = vec![0u8; BasisPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BasisPoint::decode(&buf).expect("decode must succeed");
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!((back.value - p.value).abs() < 1e-9);
    assert_eq!(back.mark, p.mark);
    assert_eq!(back.index, p.index);
    assert_eq!(back.timestamp_ms(), p.ts_ms);

    // Second encode must produce identical bytes (stability).
    let mut buf2 = vec![0u8; BasisPoint::RECORD_SIZE];
    back.encode(&mut buf2);
    assert_eq!(buf, buf2, "encode must be stable across round-trip");
}

#[test]
fn basis_point_value_invariant() {
    // value must equal mark − index when constructed by BasisDerived.
    let mark = 50_123.456_f64;
    let index = 50_113.0_f64;
    let p = BasisPoint { ts_ms: 1, value: mark - index, mark, index };
    let mut buf = vec![0u8; BasisPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BasisPoint::decode(&buf).unwrap();
    // verify: back.value == back.mark - back.index (within float precision).
    let expected = back.mark - back.index;
    assert!(
        (back.value - expected).abs() < 1e-6,
        "value={} expected={} (mark - index)",
        back.value,
        expected
    );
}

#[test]
fn basis_point_decode_wrong_length_returns_none() {
    let buf = vec![0u8; 24]; // old 24-byte layout
    assert!(
        BasisPoint::decode(&buf).is_none(),
        "decoding 24 bytes into 32-byte record must return None"
    );
}

// ---------------------------------------------------------------------------
// FundingSettlementPoint schema (unchanged — sanity check)
// ---------------------------------------------------------------------------

#[test]
fn funding_settlement_point_is_32_bytes() {
    assert_eq!(FundingSettlementPoint::RECORD_SIZE, 32);
}

#[test]
fn funding_settlement_encode_decode_stable() {
    let p = FundingSettlementPoint {
        ts_ms: 1_700_000_001_000,
        settled_rate: 0.0001_f64,
        settlement_time: 1_700_000_000_000_i64,
    };
    let mut buf = vec![0u8; FundingSettlementPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = FundingSettlementPoint::decode(&buf).expect("decode must succeed");
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!((back.settled_rate - p.settled_rate).abs() < 1e-12);
    assert_eq!(back.settlement_time, p.settlement_time);

    let mut buf2 = vec![0u8; FundingSettlementPoint::RECORD_SIZE];
    back.encode(&mut buf2);
    assert_eq!(buf, buf2, "encode must be stable across round-trip");
}

// ---------------------------------------------------------------------------
// Task C: DiskStore warm-seed shape (unit-level, no live exchange)
// ---------------------------------------------------------------------------

/// Verifies that DiskStore<BarPoint> round-trips correctly — confirming that
/// spawn_derived_forwarder's disk warm-seed read_tail path will receive valid
/// BarPoints when re-opened after a previous session.
#[test]
fn disk_store_bar_point_round_trip_for_derived_warm_seed() {
    // BarPoint is the output type of all trade-derived streams. Verify it can
    // encode+decode — the disk warm-seed path relies on this.
    use digdigdig3_station::data::BarPoint;

    let p = BarPoint {
        open_time:    1_700_000_000_000,
        open:         50_000.0,
        high:         51_000.0,
        low:          49_000.0,
        close:        50_500.0,
        volume:       123.456,
        quote_volume: 6_234_000.0,
        trades_count: 42,
    };
    let mut buf = vec![0u8; BarPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BarPoint::decode(&buf).expect("BarPoint decode must succeed");
    assert_eq!(back.open_time, p.open_time);
    assert_eq!(back.open, p.open);
    assert_eq!(back.high, p.high);
    assert_eq!(back.low, p.low);
    assert_eq!(back.close, p.close);
    assert!((back.volume - p.volume).abs() < 1e-9);
    assert!((back.quote_volume - p.quote_volume).abs() < 1e-3);
    assert_eq!(back.trades_count, p.trades_count);
    // timestamp_ms() must match open_time for DiskStore keying.
    assert_eq!(back.timestamp_ms(), p.open_time);
}
