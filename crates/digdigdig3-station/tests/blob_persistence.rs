#![cfg(not(target_arch = "wasm32"))]
//! Header + companion `.blob` round-trip for the 4 string-bearing DataPoint
//! types. Also regression-checks that fixed-size types never create a
//! `.blob` file.

use digdigdig3::core::types::{OrderSide, TradeSide};
use digdigdig3_station::data::{
    BlockTradePoint, MarketWarningPoint, OrderbookL3Point, TradePoint,
};
use digdigdig3_station::{AccountType, DataPoint, DiskStore, ExchangeId, Kind, SeriesKey};

fn tmpdir(tag: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "dig3-blob-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn block_trade_blob_round_trip_ascii() {
    let p = BlockTradePoint {
        ts_ms: 1_700_000_000_000,
        block_id: "BLK-ABCD-1234".to_string(),
        price: 70_123.45,
        quantity: 12.5,
        side: TradeSide::Sell,
        is_iv: true,
    };
    let mut hdr = vec![0u8; BlockTradePoint::RECORD_SIZE];
    p.encode(&mut hdr);
    let blob = p.encode_blob().unwrap();
    let back = BlockTradePoint::decode_blob(&hdr, &blob).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert_eq!(back.block_id, p.block_id);
    assert_eq!(back.price, p.price);
    assert_eq!(back.quantity, p.quantity);
    assert_eq!(back.side as u8, TradeSide::Sell as u8);
    assert!(back.is_iv);
    assert_eq!(BlockTradePoint::blob_pointer_offset(), Some(32));
}

#[test]
fn block_trade_blob_round_trip_utf8_and_empty() {
    for id in [
        "".to_string(),
        "блок-кириллица-✓".to_string(),
        "A".repeat(1024),
    ] {
        let p = BlockTradePoint {
            ts_ms: 1_700_000_000_001,
            block_id: id.clone(),
            price: 1.0,
            quantity: 2.0,
            side: TradeSide::Buy,
            is_iv: false,
        };
        let mut hdr = vec![0u8; BlockTradePoint::RECORD_SIZE];
        p.encode(&mut hdr);
        let blob = p.encode_blob().unwrap();
        let back = BlockTradePoint::decode_blob(&hdr, &blob).unwrap();
        assert_eq!(back.block_id, id);
    }
}

#[test]
fn market_warning_blob_round_trip() {
    let p = MarketWarningPoint {
        ts_ms: 1_700_000_001_000,
        warning_kind: "DELISTING".to_string(),
        message: "Symbol XYZ will be removed on 2026-12-31".to_string(),
    };
    let mut hdr = vec![0u8; MarketWarningPoint::RECORD_SIZE];
    p.encode(&mut hdr);
    let blob = p.encode_blob().unwrap();
    let back = MarketWarningPoint::decode_blob(&hdr, &blob).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert_eq!(back.warning_kind, p.warning_kind);
    assert_eq!(back.message, p.message);
    assert_eq!(MarketWarningPoint::blob_pointer_offset(), Some(8));
}

#[test]
fn orderbook_l3_blob_round_trip() {
    let p = OrderbookL3Point {
        ts_ms: 1_700_000_002_000,
        side: OrderSide::Sell,
        order_id: "ORD-AB-12345".to_string(),
        price: 70_001.0,
        quantity: 0.5,
        action: "INSERT".to_string(),
    };
    let mut hdr = vec![0u8; OrderbookL3Point::RECORD_SIZE];
    p.encode(&mut hdr);
    let blob = p.encode_blob().unwrap();
    let back = OrderbookL3Point::decode_blob(&hdr, &blob).unwrap();
    assert_eq!(back.ts_ms, p.ts_ms);
    assert_eq!(back.order_id, p.order_id);
    assert_eq!(back.price, p.price);
    assert_eq!(back.quantity, p.quantity);
    assert_eq!(back.action, p.action);
    assert_eq!(back.side as u8, OrderSide::Sell as u8);
    assert_eq!(OrderbookL3Point::blob_pointer_offset(), Some(32));
}

#[tokio::test]
async fn disk_store_block_trade_end_to_end() {
    let tmp = tmpdir("block-trade");
    let key = SeriesKey::new(
        ExchangeId::Binance,
        AccountType::FuturesCross,
        "BTCUSDT",
        Kind::BlockTrade,
    );

    {
        let mut store = DiskStore::<BlockTradePoint>::new(&tmp, key.clone()).await.unwrap();
        for i in 0..50 {
            store
                .append(&BlockTradePoint {
                    ts_ms: 1_700_000_000_000 + i,
                    block_id: format!("BLK-{i:04}-кир-✓"),
                    price: 70_000.0 + i as f64,
                    quantity: 0.1 * (i as f64),
                    side: if i % 2 == 0 { TradeSide::Buy } else { TradeSide::Sell },
                    is_iv: i % 3 == 0,
                })
                .unwrap();
        }
        store.flush().await.unwrap();
    }

    // .blob file must exist for this kind.
    let dir = tmp
        .join(Kind::BlockTrade.slug())
        .join("binance")
        .join("futures_cross")
        .join("btcusdt");
    let blobs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "blob")
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(blobs.len(), 1, "blob file must be created for string-bearing kind");

    let store = DiskStore::<BlockTradePoint>::new(&tmp, key).await.unwrap();
    let tail = store.read_tail(10).await.unwrap();
    assert_eq!(tail.len(), 10);
    // tail returns oldest → newest of last N; for 50 records last 10 starts at i=40.
    for (k, p) in tail.iter().enumerate() {
        let i = 40 + k as i64;
        assert_eq!(p.ts_ms, 1_700_000_000_000 + i);
        assert_eq!(p.block_id, format!("BLK-{i:04}-кир-✓"));
        assert_eq!(p.price, 70_000.0 + i as f64);
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn disk_store_market_warning_handles_empty_strings() {
    let tmp = tmpdir("market-warning");
    let key = SeriesKey::new(
        ExchangeId::Binance,
        AccountType::Spot,
        "BTCUSDT",
        Kind::MarketWarning,
    );

    {
        let mut store = DiskStore::<MarketWarningPoint>::new(&tmp, key.clone()).await.unwrap();
        for i in 0..5 {
            store
                .append(&MarketWarningPoint {
                    ts_ms: 1_700_000_000_000 + i,
                    warning_kind: if i % 2 == 0 { String::new() } else { format!("KIND{i}") },
                    message: format!("msg-{i}"),
                })
                .unwrap();
        }
        store.flush().await.unwrap();
    }

    let store = DiskStore::<MarketWarningPoint>::new(&tmp, key).await.unwrap();
    let tail = store.read_tail(5).await.unwrap();
    assert_eq!(tail.len(), 5);
    for (i, p) in tail.iter().enumerate() {
        assert_eq!(p.ts_ms, 1_700_000_000_000 + i as i64);
        assert_eq!(p.message, format!("msg-{i}"));
        if i % 2 == 0 {
            assert_eq!(p.warning_kind, "");
        } else {
            assert_eq!(p.warning_kind, format!("KIND{i}"));
        }
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn fixed_size_type_does_not_create_blob_file() {
    let tmp = tmpdir("trade-no-blob");
    let key = SeriesKey::new(ExchangeId::Binance, AccountType::Spot, "BTCUSDT", Kind::Trade);

    {
        let mut store = DiskStore::<TradePoint>::new(&tmp, key.clone()).await.unwrap();
        for i in 0..20 {
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

    let dir = tmp
        .join(Kind::Trade.slug())
        .join("binance")
        .join("spot")
        .join("btcusdt");
    let blobs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "blob")
                .unwrap_or(false)
        })
        .collect();
    assert!(
        blobs.is_empty(),
        "fixed-size type Trade must NOT create .blob file (found {} files)",
        blobs.len()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn blob_pointer_offset_is_set_only_on_string_bearing_types() {
    assert!(TradePoint::blob_pointer_offset().is_none());
    assert!(BlockTradePoint::blob_pointer_offset().is_some());
    assert!(MarketWarningPoint::blob_pointer_offset().is_some());
    assert!(OrderbookL3Point::blob_pointer_offset().is_some());
}
