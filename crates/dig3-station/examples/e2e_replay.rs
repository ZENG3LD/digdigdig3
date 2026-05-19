use std::path::PathBuf;
use std::time::Duration;

use digdigdig3_core::core::types::{AccountType, ExchangeId, StreamEvent, StreamType, SubscriptionRequest, Symbol};
use digdigdig3_station::{ReplayConfig, ReplayHub, ReplayRate};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: to_ms=None causes replay to use i64::MAX which overflows ms_to_date in StorageManager.
    // Workaround: pass explicit to_ms = now.
    let now_ms = chrono::Utc::now().timestamp_millis();
    let config = ReplayConfig {
        storage_root: PathBuf::from("./e2e_test_data"),
        rate: ReplayRate::Instant,
        from_ms: None,
        to_ms: Some(now_ms),
    };
    let hub = ReplayHub::new(config).await?;
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
    let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();

    ws.subscribe(SubscriptionRequest {
        symbol: Symbol::with_raw("BTC", "USDT", "BTCUSDT".into()),
        stream_type: StreamType::Trade,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    }).await?;

    let mut stream = ws.event_stream();
    let mut count = 0u32;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), stream.next()).await {
            Ok(Some(Ok(StreamEvent::Trade(_)))) => count += 1,
            Ok(Some(Ok(_other))) => {}
            Ok(Some(Err(e))) => eprintln!("stream error: {e}"),
            Ok(None) => {
                eprintln!("stream ended");
                break;
            }
            Err(_timeout) => {}
        }
    }

    println!("Replay emitted {count} Binance Trade events from storage");
    Ok(())
}
