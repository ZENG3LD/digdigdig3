use std::path::PathBuf;

use digdigdig3::core::normalization::{CanonicalEvent, Canonicalize};
use digdigdig3::core::types::StreamEvent;
use digdigdig3_station::{StorageConfig, StorageManager, StreamKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = StorageConfig {
        root: PathBuf::from("./e2e_test_data"),
        default_retention_days: 365,
        orderbook_snapshot_interval_secs: 0,
    };
    let mgr = StorageManager::new(cfg)?;

    let key = StreamKey {
        exchange: "binance".to_string(),
        account: "spot".to_string(),
        symbol: "BTCUSDT".to_string(),
        stream_kind: "trade".to_string(),
    };

    let now_ms = chrono::Utc::now().timestamp_millis();
    let day_start = now_ms - (now_ms % 86_400_000);
    let records = mgr.read_range(&key, day_start, now_ms).await?;

    println!("Total Binance trade records: {}", records.len());

    let sample: Vec<_> = records.iter().take(10).collect();
    let mut ok = 0u32;
    let mut failed = 0u32;

    for (i, (ts_ms, payload)) in sample.iter().enumerate() {
        match serde_json::from_slice::<StreamEvent>(payload) {
            Ok(event) => {
                match event.canonicalize() {
                    Some(CanonicalEvent::Trade(t)) => {
                        let price_ok = t.price > rust_decimal::Decimal::ZERO;
                        let qty_ok = t.quantity > rust_decimal::Decimal::ZERO;
                        let ts_ok = t.timestamp_ms > 1_700_000_000_000;
                        if price_ok && qty_ok && ts_ok {
                            ok += 1;
                            println!("[{i}] OK  price={} qty={} ts_ms={} side={:?}",
                                t.price, t.quantity, t.timestamp_ms, t.side);
                        } else {
                            failed += 1;
                            println!("[{i}] BAD price={} qty={} ts_ms={} (record ts_ms={})",
                                t.price, t.quantity, t.timestamp_ms, ts_ms);
                        }
                    }
                    Some(other) => {
                        failed += 1;
                        println!("[{i}] WRONG VARIANT: {:?}", other);
                    }
                    None => {
                        failed += 1;
                        println!("[{i}] canonicalize() returned None (record ts_ms={})", ts_ms);
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("[{i}] DESERIALIZE FAIL: {e}");
            }
        }
    }

    println!("\nResult: {ok}/10 canonical OK, {failed} failed");
    Ok(())
}
