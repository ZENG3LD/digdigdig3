//! Series<T> + DiskStore<T> round-trip basics.

use digdigdig3_station::{
    data::TradePoint, AccountType, DataPoint, DiskStore, ExchangeId, Kind, Series, SeriesKey,
};

#[test]
fn ring_evicts_oldest_at_capacity() {
    let mut s: Series<TradePoint> = Series::new(3);
    for i in 0..5 {
        s.push(TradePoint {
            ts_ms: i,
            price: i as f64,
            quantity: 1.0,
            side: 0,
            trade_id_hash: 0,
        });
    }
    let snap = s.snapshot();
    assert_eq!(snap.len(), 3);
    assert_eq!(snap[0].ts_ms, 2);
    assert_eq!(snap[2].ts_ms, 4);
}

#[tokio::test]
async fn disk_store_tail_round_trip() {
    let tmp = std::env::temp_dir().join(format!("dig3-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);

    let key = SeriesKey::new(ExchangeId::Binance, AccountType::Spot, "BTCUSDT", Kind::Trade);

    {
        let mut store = DiskStore::<TradePoint>::new(&tmp, key.clone()).await.unwrap();
        for i in 0..10 {
            store
                .append(&TradePoint {
                    ts_ms: 1_700_000_000_000 + i,
                    price: 70_000.0 + i as f64,
                    quantity: 0.1,
                    side: (i % 2) as u8,
                    trade_id_hash: i as u64,
                })
                .unwrap();
        }
        store.flush().await.unwrap();
    }

    // Re-open and read last 4 records.
    let store = DiskStore::<TradePoint>::new(&tmp, key).await.unwrap();
    let tail = store.read_tail(4).await.unwrap();
    assert_eq!(tail.len(), 4);
    assert_eq!(tail[0].ts_ms, 1_700_000_000_006);
    assert_eq!(tail[3].ts_ms, 1_700_000_000_009);
    assert_eq!(tail[0].timestamp_ms(), 1_700_000_000_006);

    let _ = std::fs::remove_dir_all(&tmp);
}
