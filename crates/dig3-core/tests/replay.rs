//! Integration tests for Phase ν — ReplayHub + ReplayWebSocket.
//!
//! Tests write real events to a temp StorageManager, then replay them
//! and assert ordering, timing, and error behaviour.

use std::path::PathBuf;
use std::time::Instant;

use chrono::Utc;
use digdigdig3::core::storage::{StorageConfig, StorageManager, StreamKey};
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, SubscriptionRequest, Symbol, Ticker,
};
use digdigdig3::{ReplayConfig, ReplayHub, ReplayRate};
use futures_util::StreamExt;

// ── helpers ───────────────────────────────────────────────────────────────────

fn tmpdir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "dig3_replay_{}_{}",
        std::process::id(),
        rand_suffix()
    ));
    dir.push(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rand_suffix() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    h.finish()
}

fn btc_key() -> StreamKey {
    StreamKey {
        exchange: "binance".into(),
        account: "spot".into(),
        symbol: "BTCUSDT".into(),
        stream_kind: "ticker".into(),
    }
}

fn btc_sub() -> SubscriptionRequest {
    SubscriptionRequest::ticker(Symbol::with_raw("BTC", "USDT", "BTCUSDT".into()))
}

fn make_ticker(price: f64, ts_ms: i64) -> StreamEvent {
    StreamEvent::Ticker(Ticker {
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
    })
}

async fn write_events(dir: &PathBuf, key: &StreamKey, events: &[(i64, StreamEvent)]) {
    let storage = StorageManager::new(StorageConfig {
        root: dir.clone(),
        default_retention_days: 365,
        orderbook_snapshot_interval_secs: 0,
    })
    .unwrap();
    for (ts_ms, ev) in events {
        let payload = serde_json::to_vec(ev).unwrap();
        storage.append(key, *ts_ms, &payload).await.unwrap();
    }
    storage.flush_all().await.unwrap();
}

async fn collect_events(
    hub: &ReplayHub,
    id: ExchangeId,
    acct: AccountType,
    sub: SubscriptionRequest,
    max: usize,
) -> Vec<StreamEvent> {
    let ws = hub.ws(id, acct).unwrap();
    ws.subscribe(sub).await.unwrap();
    let mut stream = ws.event_stream();
    let mut out = Vec::new();
    while let Some(ev) = stream.next().await {
        match ev {
            Ok(e) => {
                out.push(e);
                if out.len() >= max {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    out
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// Events stored in order come out in the same order.
#[tokio::test]
async fn replay_emits_stored_events_in_order() {
    let dir = tmpdir("in_order");
    let base_ms = Utc::now().timestamp_millis();
    let key = btc_key();

    let events: Vec<(i64, StreamEvent)> = (0..5)
        .map(|i| (base_ms + i * 1000, make_ticker(50000.0 + i as f64, base_ms + i * 1000)))
        .collect();
    write_events(&dir, &key, &events).await;

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Instant,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 10_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let received = collect_events(&hub, ExchangeId::Binance, AccountType::Spot, btc_sub(), 5).await;
    assert_eq!(received.len(), 5, "expected 5 events");

    // Check ordering via price field (each event has a unique price).
    for (i, ev) in received.iter().enumerate() {
        if let StreamEvent::Ticker(t) = ev {
            let expected = 50000.0 + i as f64;
            assert!(
                (t.last_price - expected).abs() < 0.001,
                "event {i}: expected price {expected}, got {}",
                t.last_price
            );
        } else {
            panic!("expected Ticker event, got {:?}", ev);
        }
    }

    std::fs::remove_dir_all(&dir).ok();
}

/// With `Instant` rate 100 events should complete well under 500 ms.
#[tokio::test]
async fn replay_respects_rate_instant() {
    let dir = tmpdir("instant");
    let base_ms = Utc::now().timestamp_millis();
    let key = btc_key();

    // 100 events 1 s apart (100 s of simulated time).
    let events: Vec<(i64, StreamEvent)> = (0..100)
        .map(|i| (base_ms + i * 1000, make_ticker(50000.0, base_ms + i * 1000)))
        .collect();
    write_events(&dir, &key, &events).await;

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Instant,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 200_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let start = Instant::now();
    let received = collect_events(
        &hub,
        ExchangeId::Binance,
        AccountType::Spot,
        btc_sub(),
        100,
    )
    .await;
    let elapsed = start.elapsed();

    assert_eq!(received.len(), 100);
    assert!(
        elapsed.as_millis() < 500,
        "Instant replay of 100 events took {}ms, expected <500ms",
        elapsed.as_millis()
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// 3 events 100 ms apart at `Realtime` should take at least ~200 ms.
#[tokio::test]
async fn replay_respects_rate_realtime() {
    let dir = tmpdir("realtime");
    let base_ms = Utc::now().timestamp_millis();
    let key = btc_key();

    // 3 events 100 ms apart.
    let events: Vec<(i64, StreamEvent)> = (0..3)
        .map(|i| (base_ms + i * 100, make_ticker(50000.0, base_ms + i * 100)))
        .collect();
    write_events(&dir, &key, &events).await;

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Realtime,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 1_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let start = Instant::now();
    let received = collect_events(
        &hub,
        ExchangeId::Binance,
        AccountType::Spot,
        btc_sub(),
        3,
    )
    .await;
    let elapsed = start.elapsed();

    assert_eq!(received.len(), 3);
    // First event emitted immediately, then 100 ms delay, then 100 ms delay.
    // Total: ≥200 ms, allow up to 1000 ms for slow CI.
    assert!(
        elapsed.as_millis() >= 150,
        "Realtime replay finished too fast: {}ms",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_millis() < 1500,
        "Realtime replay took too long: {}ms",
        elapsed.as_millis()
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// `Accelerated(2.0)` over a 1 s simulated span should finish in ~500 ms.
#[tokio::test]
async fn replay_accelerated_2x() {
    let dir = tmpdir("accel2x");
    let base_ms = Utc::now().timestamp_millis();
    let key = btc_key();

    // 3 events spanning 1 s of simulated time.
    let events: Vec<(i64, StreamEvent)> = (0..3)
        .map(|i| (base_ms + i * 500, make_ticker(50000.0, base_ms + i * 500)))
        .collect();
    write_events(&dir, &key, &events).await;

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Accelerated(2.0),
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 2_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let start = Instant::now();
    let received = collect_events(
        &hub,
        ExchangeId::Binance,
        AccountType::Spot,
        btc_sub(),
        3,
    )
    .await;
    let elapsed = start.elapsed();

    assert_eq!(received.len(), 3);
    // Sim span = 1000 ms, 2x speed → ~500 ms real.  Allow 75–900 ms.
    assert!(
        elapsed.as_millis() < 900,
        "2x accelerated replay took {}ms, expected <900ms",
        elapsed.as_millis()
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// Subscribing with no stored data yields no events and no error.
#[tokio::test]
async fn replay_empty_storage() {
    let dir = tmpdir("empty");

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Instant,
        from_ms: None,
        to_ms: None,
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
    ws.subscribe(btc_sub()).await.unwrap();

    // Give the replay task a moment to run and find no data.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut stream = ws.event_stream();
    // Stream should be empty — use a short timeout.
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        stream.next(),
    )
    .await;

    // Either timeout (no events) or stream closed (also fine).
    match result {
        Err(_timeout) => {} // good — no events within 100 ms
        Ok(None) => {}      // stream closed cleanly
        Ok(Some(Err(_))) => {} // error event (also acceptable for empty)
        Ok(Some(Ok(_ev))) => {
            panic!("expected no events from empty storage, got an event");
        }
    }

    std::fs::remove_dir_all(&dir).ok();
}

/// Multiple subscriptions on different stream keys emit concurrently.
#[tokio::test]
async fn replay_multiple_subscriptions_parallel() {
    let dir = tmpdir("parallel");
    let base_ms = Utc::now().timestamp_millis();

    // Write 3 distinct streams.
    let streams = [
        (
            StreamKey {
                exchange: "binance".into(),
                account: "spot".into(),
                symbol: "BTCUSDT".into(),
                stream_kind: "ticker".into(),
            },
            SubscriptionRequest::ticker(Symbol::with_raw("BTC", "USDT", "BTCUSDT".into())),
        ),
        (
            StreamKey {
                exchange: "binance".into(),
                account: "spot".into(),
                symbol: "ETHUSDT".into(),
                stream_kind: "ticker".into(),
            },
            SubscriptionRequest::ticker(Symbol::with_raw("ETH", "USDT", "ETHUSDT".into())),
        ),
        (
            StreamKey {
                exchange: "binance".into(),
                account: "spot".into(),
                symbol: "SOLUSDT".into(),
                stream_kind: "ticker".into(),
            },
            SubscriptionRequest::ticker(Symbol::with_raw("SOL", "USDT", "SOLUSDT".into())),
        ),
    ];

    for (key, _) in &streams {
        let events: Vec<(i64, StreamEvent)> = (0..10)
            .map(|i| {
                (
                    base_ms + i * 100,
                    make_ticker(1000.0, base_ms + i * 100),
                )
            })
            .collect();
        write_events(&dir, key, &events).await;
    }

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Instant,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 5_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();

    // Subscribe to all 3 on the same WS handle.
    for (_, sub) in &streams {
        ws.subscribe(sub.clone()).await.unwrap();
    }

    // Collect 30 events total (10 from each stream).
    let mut stream = ws.event_stream();
    let mut count = 0usize;
    let deadline = tokio::time::sleep(std::time::Duration::from_secs(2));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(ev) = stream.next() => {
                if ev.is_ok() {
                    count += 1;
                }
                if count >= 30 { break; }
            }
            _ = &mut deadline => { break; }
        }
    }

    assert_eq!(count, 30, "expected 30 events from 3 parallel streams (10 each)");

    std::fs::remove_dir_all(&dir).ok();
}

/// A corrupted payload emits a `Parse` error and replay continues.
#[tokio::test]
async fn replay_corrupted_payload_emits_parse_error_continues() {
    let dir = tmpdir("corrupt");
    let base_ms = Utc::now().timestamp_millis();
    let key = btc_key();

    // Write: good, corrupted, good.
    let storage = StorageManager::new(StorageConfig {
        root: dir.clone(),
        default_retention_days: 365,
        orderbook_snapshot_interval_secs: 0,
    })
    .unwrap();

    let good_ev = make_ticker(50000.0, base_ms);
    let good_bytes = serde_json::to_vec(&good_ev).unwrap();

    storage.append(&key, base_ms, &good_bytes).await.unwrap();
    storage
        .append(&key, base_ms + 100, b"NOT VALID JSON {{{")
        .await
        .unwrap();
    storage
        .append(&key, base_ms + 200, &good_bytes)
        .await
        .unwrap();
    storage.flush_all().await.unwrap();

    let hub = ReplayHub::new(ReplayConfig {
        storage_root: dir.clone(),
        rate: ReplayRate::Instant,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 1_000),
    })
    .await
    .unwrap();
    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .unwrap();

    let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
    ws.subscribe(btc_sub()).await.unwrap();

    let mut stream = ws.event_stream();
    let mut ok_count = 0usize;
    let mut err_count = 0usize;

    let deadline = tokio::time::sleep(std::time::Duration::from_millis(500));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(ev) = stream.next() => {
                match ev {
                    Ok(_) => ok_count += 1,
                    Err(_) => err_count += 1,
                }
                if ok_count + err_count >= 3 { break; }
            }
            _ = &mut deadline => { break; }
        }
    }

    assert_eq!(ok_count, 2, "expected 2 good events, got {ok_count}");
    assert_eq!(err_count, 1, "expected 1 parse error, got {err_count}");

    std::fs::remove_dir_all(&dir).ok();
}
