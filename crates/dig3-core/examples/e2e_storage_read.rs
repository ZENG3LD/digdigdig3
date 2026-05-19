use std::path::PathBuf;
use digdigdig3_core::{StorageManager, StorageConfig, StreamKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = StorageConfig {
        root: PathBuf::from("./e2e_test_data"),
        default_retention_days: 365,
        orderbook_snapshot_interval_secs: 0,
    };
    let mgr = StorageManager::new(cfg)?;

    for (exchange, account, sym, stream) in [
        ("binance", "spot", "BTCUSDT",  "trade"),
        ("binance", "spot", "BTCUSDT",  "ticker"),
        ("bybit",   "spot", "BTCUSDT",  "trade"),
        ("bybit",   "spot", "BTCUSDT",  "ticker"),
        ("okx",     "spot", "BTC-USDT", "trade"),
        ("okx",     "spot", "BTC-USDT", "ticker"),
    ] {
        let key = StreamKey {
            exchange: exchange.to_string(),
            account: account.to_string(),
            symbol: sym.to_string(),
            stream_kind: stream.to_string(),
        };
        // read today only — avoid iterating thousands of days from epoch
        let now_ms = chrono::Utc::now().timestamp_millis();
        let day_start = now_ms - (now_ms % 86_400_000);
        let records = mgr.read_range(&key, day_start, now_ms).await?;
        let first_ts = records.first().map(|(t, _)| *t);
        let last_ts = records.last().map(|(t, _)| *t);
        let span_secs = match (first_ts, last_ts) {
            (Some(f), Some(l)) => (l - f) / 1000,
            _ => 0,
        };
        println!("{exchange}:{sym}:{stream:8} records={} span={span_secs}s first_ts={first_ts:?} last_ts={last_ts:?}",
            records.len());

        if let Some((_, payload)) = records.first() {
            match serde_json::from_slice::<serde_json::Value>(payload) {
                Ok(v) => {
                    let keys: Vec<_> = v.as_object()
                        .map(|o| o.keys().map(|k| k.as_str()).collect())
                        .unwrap_or_default();
                    println!("  first event keys: {:?}", keys);
                }
                Err(e) => println!("  first event PARSE FAIL: {e}"),
            }
        }
    }
    Ok(())
}
