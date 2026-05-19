//! Phase ν — replay demo.
//!
//! Writes 200 synthetic ticker events to a temp storage directory, then
//! replays them at 10× speed and counts received events.
//!
//! Run with:
//!   cargo run --example replay_demo --release

use std::path::PathBuf;

use digdigdig3::core::storage::{StorageConfig, StorageManager, StreamKey};
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, SubscriptionRequest, Symbol, Ticker,
};
use digdigdig3::{ReplayConfig, ReplayHub, ReplayRate};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── 1. Write synthetic events ─────────────────────────────────────────────

    let tmp_dir = {
        let mut d = std::env::temp_dir();
        d.push("dig3_replay_demo");
        d
    };
    std::fs::remove_dir_all(&tmp_dir).ok();
    std::fs::create_dir_all(&tmp_dir)?;

    let storage = StorageManager::new(StorageConfig {
        root: tmp_dir.clone(),
        default_retention_days: 365,
        orderbook_snapshot_interval_secs: 0,
    })?;

    let key = StreamKey {
        exchange: "binance".into(),
        account: "spot".into(),
        symbol: "BTCUSDT".into(),
        stream_kind: "ticker".into(),
    };

    let base_ms = chrono::Utc::now().timestamp_millis();
    let n_events = 200usize;

    for i in 0..n_events {
        let ts_ms = base_ms + (i as i64) * 100; // 100 ms apart → 20 s simulated
        let price = 50_000.0 + i as f64;
        let ev = StreamEvent::Ticker(Ticker {
            symbol: "BTCUSDT".into(),
            last_price: price,
            bid_price: Some(price - 1.0),
            ask_price: Some(price + 1.0),
            volume_24h: Some(1000.0),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            high_24h: None,
            low_24h: None,
            timestamp: ts_ms,
        });
        let payload = serde_json::to_vec(&ev)?;
        storage.append(&key, ts_ms, &payload).await?;
    }
    storage.flush_all().await?;
    println!("Wrote {n_events} synthetic ticker events to {}", tmp_dir.display());

    // ── 2. Replay at 10× speed ────────────────────────────────────────────────

    let config = ReplayConfig {
        storage_root: PathBuf::from(&tmp_dir),
        rate: ReplayRate::Accelerated(10.0),
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + (n_events as i64) * 100),
    };
    let hub = ReplayHub::new(config).await?;
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await?;

    let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
    ws.subscribe(SubscriptionRequest::ticker(Symbol::with_raw(
        "BTC",
        "USDT",
        "BTCUSDT".into(),
    )))
    .await?;

    let mut stream = ws.event_stream();
    let mut count = 0usize;
    let start = std::time::Instant::now();

    while let Some(ev) = stream.next().await {
        match ev {
            Ok(StreamEvent::Ticker(_t)) => {
                count += 1;
                if count % 50 == 0 {
                    println!("  {count} events received…");
                }
                if count >= n_events {
                    break;
                }
            }
            Ok(_other) => {}
            Err(e) => eprintln!("stream error: {e}"),
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Replay done: {count} events in {:.2}s  (simulated 20s at 10×)",
        elapsed.as_secs_f64()
    );

    // ── 3. Cleanup ────────────────────────────────────────────────────────────
    std::fs::remove_dir_all(&tmp_dir).ok();

    Ok(())
}
