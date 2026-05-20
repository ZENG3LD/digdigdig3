//! Round-trip encode/decode for every DataPoint.

use digdigdig3_station::data::{
    AggTradePoint, BarPoint, FundingRatePoint, LiquidationPoint, MarkPricePoint, ObSnapshotPoint,
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
fn small_types_round_trip() {
    rt_bytes_stable(MarkPricePoint { ts_ms: 100, mark: 70_000.0, index: 69_999.5 });
    rt_bytes_stable(FundingRatePoint { ts_ms: 200, rate: 0.0001, next_funding_time_ms: 1_700_000_000_000 });
    rt_bytes_stable(OpenInterestPoint { ts_ms: 300, open_interest: 12345.6, open_interest_value: 8.7e8 });
    rt_bytes_stable(LiquidationPoint { ts_ms: 400, price: 70_001.0, quantity: 0.5, value: 35_000.5, side: 1 });
    rt_bytes_stable(AggTradePoint { ts_ms: 500, price: 70_002.0, quantity: 0.25, side: 0, agg_id: 999 });
}
