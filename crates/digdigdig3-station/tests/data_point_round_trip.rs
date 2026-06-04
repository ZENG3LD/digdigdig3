#![cfg(not(target_arch = "wasm32"))]
//! Round-trip encode/decode for every DataPoint.

use digdigdig3_station::data::{
    AggTradePoint, BarPoint, BasisPoint, FundingRatePoint, FundingSettlementPoint,
    LiquidationPoint, LongShortRatioPoint, MarkPricePoint, ObDeltaPoint, ObSnapshotPoint,
    OpenInterestPoint, TickerPoint, TradePoint,
};
use digdigdig3_station::DataPoint;

fn rt_bytes_stable<T: DataPoint>(p: T) {
    let mut buf = vec![0u8; T::RECORD_SIZE];
    p.encode(&mut buf);
    let back = T::decode(&buf).expect("decode must succeed");
    let mut buf2 = vec![0u8; T::RECORD_SIZE];
    back.encode(&mut buf2);
    assert_eq!(buf, buf2, "{}: encode → decode → encode unstable", std::any::type_name::<T>());
}

#[test]
fn trade_round_trip() {
    let p = TradePoint {
        ts_ms: 1_700_000_000_123,
        price: 70_123.45,
        quantity: 0.0123,
        side: 1,
        trade_id_hash: 0xDEAD_BEEF_CAFE_F00D,
    };
    let mut buf = vec![0u8; TradePoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = TradePoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert_eq!(back.price, p.price);
    assert_eq!(back.quantity, p.quantity);
    assert_eq!(back.side, p.side);
    assert_eq!(back.trade_id_hash, p.trade_id_hash);
    assert_eq!(p.timestamp_ms(), 1_700_000_000_123);
}

#[test]
fn bar_round_trip() {
    let p = BarPoint {
        open_time: 1_700_000_000_000,
        open: 70_000.0, high: 70_500.0, low: 69_500.0, close: 70_250.0,
        volume: 12.34, quote_volume: 870_000.0, trades_count: 42,
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; BarPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BarPoint::decode(&buf).unwrap();
    assert_eq!(back.open, p.open);
    assert_eq!(back.high, p.high);
    assert_eq!(back.low, p.low);
    assert_eq!(back.close, p.close);
    assert_eq!(back.volume, p.volume);
}

#[test]
fn ticker_round_trip() {
    assert_eq!(TickerPoint::RECORD_SIZE, 72);
    let p = TickerPoint {
        ts_ms: 1_700_000_000_000,
        last: 70_000.0, bid: 69_999.5, ask: 70_000.5,
        high_24h: 71_000.0, low_24h: 69_000.0,
        vol_24h: 1234.56, quote_vol_24h: 8.7e7, change_pct_24h: -0.4321,
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; TickerPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = TickerPoint::decode(&buf).unwrap();
    assert_eq!(back.last, p.last);
    assert_eq!(back.bid, p.bid);
    assert_eq!(back.ask, p.ask);
    assert_eq!(back.change_pct_24h, p.change_pct_24h);
}

#[test]
fn ob_snapshot_round_trip_top_3() {
    let p = ObSnapshotPoint {
        ts_ms: 1_700_000_000_000,
        bids: vec![(70_000.0, 1.0), (69_999.0, 2.0), (69_998.0, 3.0)],
        asks: vec![(70_001.0, 1.5), (70_002.0, 2.5)],
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; ObSnapshotPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = ObSnapshotPoint::decode(&buf).unwrap();
    assert_eq!(back.bids.len(), 3);
    assert_eq!(back.asks.len(), 2);
    assert_eq!(back.bids[0], (70_000.0, 1.0));
    assert_eq!(back.asks[1], (70_002.0, 2.5));
}

#[test]
fn ob_delta_round_trip_with_removal() {
    // bid level (70_001.0, 0.0) means "remove 70_001.0 from bid side".
    // ask level (70_002.0, 1.5) means "set ask 70_002.0 to size 1.5".
    let p = ObDeltaPoint {
        ts_ms: 1_700_000_000_321,
        bid_changes: vec![(70_000.0, 2.5), (70_001.0, 0.0)],
        ask_changes: vec![(70_002.0, 1.5)],
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; ObDeltaPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = ObDeltaPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert_eq!(back.bid_changes.len(), 2);
    assert_eq!(back.bid_changes[0], (70_000.0, 2.5));
    assert_eq!(back.bid_changes[1], (70_001.0, 0.0), "removal entry must survive round-trip");
    assert_eq!(back.ask_changes.len(), 1);
    assert_eq!(back.ask_changes[0], (70_002.0, 1.5));
}

#[test]
fn ob_delta_empty_round_trip() {
    let p = ObDeltaPoint {
        ts_ms: 1_700_000_000_000,
        bid_changes: vec![],
        ask_changes: vec![],
    };
    let mut buf = vec![0u8; ObDeltaPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = ObDeltaPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!(back.bid_changes.is_empty());
    assert!(back.ask_changes.is_empty());
}

#[test]
fn small_types_round_trip() {
    rt_bytes_stable(MarkPricePoint { ts_ms: 100, mark: 70_000.0, index: 69_999.5 });
    rt_bytes_stable(FundingRatePoint { ts_ms: 200, rate: 0.0001, next_funding_time_ms: 1_700_000_000_000 });
    rt_bytes_stable(OpenInterestPoint { ts_ms: 300, open_interest: 12345.6, open_interest_value: 8.7e8 });
    rt_bytes_stable(LiquidationPoint { ts_ms: 400, price: 70_001.0, quantity: 0.5, value: 35_000.5, side: 1 });
    rt_bytes_stable(AggTradePoint { ts_ms: 500, price: 70_002.0, quantity: 0.25, side: 0, agg_id: 999 });
}

// --- BasisPoint (32 B — expanded schema) ---

#[test]
fn basis_point_round_trip_32b() {
    assert_eq!(BasisPoint::RECORD_SIZE, 32, "BasisPoint must be 32 B");
    let p = BasisPoint {
        ts_ms: 1_700_000_000_000,
        value: 10.5,
        mark:  70_000.0,
        index: 69_989.5,
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; BasisPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BasisPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!((back.value - p.value).abs() < 1e-9);
    assert_eq!(back.mark, p.mark);
    assert_eq!(back.index, p.index);
}

#[test]
fn basis_point_nan_fields_survive_round_trip() {
    // from_stream_event WS path: mark/index populated as NaN.
    let p = BasisPoint { ts_ms: 42, value: 1.0, mark: f64::NAN, index: f64::NAN };
    let mut buf = vec![0u8; BasisPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = BasisPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, 42);
    assert!(back.mark.is_nan());
    assert!(back.index.is_nan());
}

// --- LongShortRatioPoint round-trip (32 B) ---

#[test]
fn long_short_ratio_point_round_trip_32b() {
    assert_eq!(LongShortRatioPoint::RECORD_SIZE, 32, "LongShortRatioPoint must be 32 B");
    let p = LongShortRatioPoint {
        ts_ms: 1_700_000_000_000,
        ratio: 1.2774,
        long_pct: 0.5609,
        short_pct: 0.4391,
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; LongShortRatioPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = LongShortRatioPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!((back.ratio - p.ratio).abs() < 1e-10);
    assert!((back.long_pct - p.long_pct).abs() < 1e-10);
    assert!((back.short_pct - p.short_pct).abs() < 1e-10);
    assert_eq!(back.timestamp_ms(), p.ts_ms);
}

#[test]
fn long_short_ratio_from_stream_event() {
    use digdigdig3_station::DataPoint;
    use digdigdig3::core::types::StreamEvent;
    let ev = StreamEvent::LongShortRatio {
        symbol: "BTCUSDT".to_string(),
        ratio_type: "globalAccount".to_string(),
        long_ratio: 0.56,
        short_ratio: 0.44,
        timestamp: 1_700_000_001_000,
    };
    let pt = LongShortRatioPoint::from_stream_event(&ev).expect("must extract from LongShortRatio event");
    assert_eq!(pt.ts_ms, 1_700_000_001_000);
    assert!((pt.long_pct - 0.56).abs() < 1e-10);
    assert!((pt.short_pct - 0.44).abs() < 1e-10);
    // ratio = 0.56 / 0.44 ≈ 1.2727
    assert!((pt.ratio - (0.56f64 / 0.44)).abs() < 1e-8);
}

// --- FundingSettlementPoint round-trip (sanity, unchanged schema) ---

#[test]
fn funding_settlement_point_round_trip() {
    assert_eq!(FundingSettlementPoint::RECORD_SIZE, 32);
    let p = FundingSettlementPoint {
        ts_ms: 1_700_000_001_000,
        settled_rate: 0.0001,
        settlement_time: 1_700_000_000_000,
    };
    rt_bytes_stable(p.clone());
    let mut buf = vec![0u8; FundingSettlementPoint::RECORD_SIZE];
    p.encode(&mut buf);
    let back = FundingSettlementPoint::decode(&buf).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert!((back.settled_rate - p.settled_rate).abs() < 1e-12);
    assert_eq!(back.settlement_time, p.settlement_time);
}
