#![cfg(not(target_arch = "wasm32"))]
//! Live integration tests for the 4 scenarios mlc-bridge will hit when wrapping Station.
//!
//! Target venue: Binance Spot BTCUSDT (raw exchange-native symbol via `add_raw`).
//!
//! Gated with `--ignored`. Run with:
//!   cargo test --test mlc_scenarios_live --release -- --ignored --nocapture

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::websocket::KlineInterval;
use digdigdig3_station::data::{BarPoint, TradePoint};
use digdigdig3_station::series::{Kind, Series};
use digdigdig3_station::{
    AccountType, Event, ExchangeId, PersistenceConfig, SeriesKey, Station, Stream,
    SubscriptionSet,
};
use tokio::sync::RwLock;

const EXCHANGE: ExchangeId = ExchangeId::Binance;
const SYMBOL: &str = "BTCUSDT";
const ACCOUNT: AccountType = AccountType::Spot;

/// Generate a unique temp directory path for each test. Uses process ID +
/// test name suffix so parallel test runs don't collide.
fn unique_tempdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("dig3_mlc_test_{tag}_{}", std::process::id()));
    std::fs::create_dir_all(&p).expect("create tempdir");
    p
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario 1: bars_warmstart_and_live
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to 1h Klines on Binance BTCUSDT with a fresh tempdir.
/// `warm_start(1000)` + `PersistenceConfig::on()` — disk is empty, so the
/// REST backfill path should populate ~1000 1h bars immediately.
/// Drains for 12s.
///
/// Asserts:
///   - at least 100 `Event::Bar` events received
///   - `station.series::<BarPoint>(&key)` is Some
///   - series.read().len() >= 100
#[tokio::test]
#[ignore = "live API, requires network"]
async fn bars_warmstart_and_live() {
    let storage = unique_tempdir("bars_warmstart");
    let station = Station::builder()
        .storage_root(&storage)
        .persistence(PersistenceConfig::on())
        .warm_start(1000)
        .build()
        .await
        .expect("Station::build");

    let interval = KlineInterval::new("1h");
    let key = SeriesKey::new(EXCHANGE, ACCOUNT, SYMBOL, Kind::Kline(interval.clone()));

    let set = SubscriptionSet::new().add_raw(
        EXCHANGE,
        SYMBOL,
        ACCOUNT,
        [Stream::Kline(interval.clone())],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe failed for Binance BTCUSDT Kline 1h: {:?}",
        report.failed
    );

    let mut bar_count = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(12);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining.min(Duration::from_secs(2)), report.handle.recv())
            .await
        {
            Ok(Some(Event::Bar { .. })) => bar_count += 1,
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    println!("[bars_warmstart_and_live] bar_count={bar_count}");

    assert!(
        bar_count >= 100,
        "expected >= 100 Bar events from REST warm-start (got {bar_count})"
    );

    let sh: Option<Arc<RwLock<Series<BarPoint>>>> = station.series::<BarPoint>(&key);
    assert!(
        sh.is_some(),
        "station.series::<BarPoint>(&key) returned None after subscribe"
    );
    let len = sh.unwrap().read().await.len();
    println!("[bars_warmstart_and_live] series.len()={len}");
    assert!(
        len >= 100,
        "in-memory series should have >= 100 bars after warm-start (got {len})"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario 2: scroll_left_fetch_history
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to 1h Klines, wait 5s for warm-start to populate the series,
/// then call `fetch_history` with `end_time = oldest_warm - 1` to page
/// further into the past.
///
/// Asserts:
///   - >= 400 bars returned (request limit=500, allow slack)
///   - all returned bars have open_time < oldest_warm
///   - bars sorted oldest-first
#[tokio::test]
#[ignore = "live API, requires network"]
async fn scroll_left_fetch_history() {
    let storage = unique_tempdir("scroll_left");
    let station = Station::builder()
        .storage_root(&storage)
        .persistence(PersistenceConfig::on())
        .warm_start(1000)
        .build()
        .await
        .expect("Station::build");

    let interval = KlineInterval::new("1h");
    let key = SeriesKey::new(EXCHANGE, ACCOUNT, SYMBOL, Kind::Kline(interval.clone()));

    let set = SubscriptionSet::new().add_raw(
        EXCHANGE,
        SYMBOL,
        ACCOUNT,
        [Stream::Kline(interval.clone())],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe failed: {:?}",
        report.failed
    );

    // Drain 5s to let the REST warm-start seed complete.
    let warm_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        if tokio::time::Instant::now() >= warm_deadline {
            break;
        }
        let remaining = warm_deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining.min(Duration::from_secs(1)), report.handle.recv())
            .await
        {
            Ok(None) => break,
            _ => {}
        }
    }

    // Capture oldest bar in the series.
    let sh: Option<Arc<RwLock<Series<BarPoint>>>> = station.series::<BarPoint>(&key);
    let sh = sh.expect("series handle present after warm-start");
    let oldest_warm = {
        let guard = sh.read().await;
        let snap = guard.snapshot();
        assert!(
            !snap.is_empty(),
            "warm-start produced 0 bars — REST seed did not populate series"
        );
        snap[0].open_time
    };
    println!("[scroll_left_fetch_history] oldest_warm={oldest_warm}");

    // Build a standalone hub for fetch_history — station.inner.hub is pub(crate).
    let hub = Arc::new(ExchangeHub::new());
    hub.connect_public(EXCHANGE, false)
        .await
        .expect("connect_public for history hub");

    let bars = digdigdig3_station::fetch_history(
        &hub,
        EXCHANGE,
        SYMBOL,
        ACCOUNT,
        &interval,
        oldest_warm - 1,
        500,
    )
    .await
    .expect("fetch_history");

    let count = bars.len();
    let first_ts = bars.first().map(|b| b.open_time).unwrap_or(0);
    let last_ts = bars.last().map(|b| b.open_time).unwrap_or(0);
    println!(
        "[scroll_left_fetch_history] count={count} first_ts={first_ts} last_ts={last_ts}"
    );

    assert!(count >= 400, "expected >= 400 historical bars (got {count})");

    let all_before = bars.iter().all(|b| b.open_time < oldest_warm);
    assert!(
        all_before,
        "some returned bars have open_time >= oldest_warm ({oldest_warm}) \
         — fetch_history should return only bars strictly before end_time"
    );

    let sorted = bars.windows(2).all(|w| w[0].open_time <= w[1].open_time);
    assert!(sorted, "bars not sorted oldest-first");
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario 3: trades_subscribe_and_series_reader
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to `Stream::Trade` for BTCUSDT, drain for 10s.
///
/// Asserts:
///   - at least 20 `Event::Trade` events
///   - `station.series::<TradePoint>(&key)` is Some and len >= 20
#[tokio::test]
#[ignore = "live API, requires network"]
async fn trades_subscribe_and_series_reader() {
    let storage = unique_tempdir("trades");
    let station = Station::builder()
        .storage_root(&storage)
        .persistence(PersistenceConfig::on())
        .warm_start(1000)
        .build()
        .await
        .expect("Station::build");

    let key = SeriesKey::new(EXCHANGE, ACCOUNT, SYMBOL, Kind::Trade);

    let set = SubscriptionSet::new().add_raw(EXCHANGE, SYMBOL, ACCOUNT, [Stream::Trade]);

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe failed for Binance BTCUSDT Trade: {:?}",
        report.failed
    );

    let mut trade_count = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining.min(Duration::from_secs(2)), report.handle.recv())
            .await
        {
            Ok(Some(Event::Trade { .. })) => trade_count += 1,
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    println!("[trades_subscribe_and_series_reader] trade_count={trade_count}");

    assert!(
        trade_count >= 20,
        "expected >= 20 Trade events from Binance BTCUSDT in 10s (got {trade_count})"
    );

    let sh: Option<Arc<RwLock<Series<TradePoint>>>> = station.series::<TradePoint>(&key);
    assert!(
        sh.is_some(),
        "station.series::<TradePoint>(&key) returned None after subscribe"
    );
    let len = sh.unwrap().read().await.len();
    println!("[trades_subscribe_and_series_reader] series.len()={len}");
    assert!(
        len >= 20,
        "in-memory trade series should have >= 20 points (got {len})"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Scenario 4: orderbook_rest_seed_then_deltas
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to `Stream::OrderbookDelta` with `orderbook_rest_seed(true)`.
///
/// Asserts:
///   - first event received is `Event::OrderbookSnapshot` with non-empty bids and asks
///   - at least 1 `Event::OrderbookDelta` follows within the 10s window
#[tokio::test]
#[ignore = "live API, requires network"]
async fn orderbook_rest_seed_then_deltas() {
    let storage = unique_tempdir("ob_delta");
    let station = Station::builder()
        .storage_root(&storage)
        .persistence(PersistenceConfig::on())
        .warm_start(1000)
        .orderbook_rest_seed(true)
        .orderbook_seed_depth(1000)
        .unsubscribe_grace(Duration::from_secs(5))
        .build()
        .await
        .expect("Station::build");

    let set = SubscriptionSet::new().add_raw(
        EXCHANGE,
        SYMBOL,
        ACCOUNT,
        [Stream::OrderbookDelta],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe failed for Binance BTCUSDT OrderbookDelta: {:?}",
        report.failed
    );

    let mut snapshot_count = 0usize;
    let mut delta_count = 0usize;
    // Track first received event kind (excluding non-data lifecycle events).
    let mut first_data_event: Option<&'static str> = None;
    let mut first_snapshot_bids_len = 0usize;
    let mut first_snapshot_asks_len = 0usize;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        match tokio::time::timeout(remaining.min(Duration::from_secs(2)), report.handle.recv())
            .await
        {
            Ok(Some(Event::OrderbookSnapshot { point, .. })) => {
                if first_data_event.is_none() {
                    first_data_event = Some("OrderbookSnapshot");
                    first_snapshot_bids_len = point.bids.iter().filter(|(p, _)| *p > 0.0).count();
                    first_snapshot_asks_len = point.asks.iter().filter(|(p, _)| *p > 0.0).count();
                    println!(
                        "[orderbook_rest_seed_then_deltas] first snapshot: \
                         bids={first_snapshot_bids_len} asks={first_snapshot_asks_len}"
                    );
                }
                snapshot_count += 1;
            }
            Ok(Some(Event::OrderbookDelta { .. })) => {
                if first_data_event.is_none() {
                    first_data_event = Some("OrderbookDelta");
                }
                delta_count += 1;
            }
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    println!(
        "[orderbook_rest_seed_then_deltas] snapshot_count={snapshot_count} \
         delta_count={delta_count} first={first_data_event:?}"
    );

    assert_eq!(
        first_data_event,
        Some("OrderbookSnapshot"),
        "expected first data event to be OrderbookSnapshot (REST seed), \
         got {first_data_event:?} (snapshot_count={snapshot_count}, delta_count={delta_count})"
    );
    assert!(
        first_snapshot_bids_len > 0,
        "first OrderbookSnapshot had 0 non-zero bid levels — REST seed returned empty bids"
    );
    assert!(
        first_snapshot_asks_len > 0,
        "first OrderbookSnapshot had 0 non-zero ask levels — REST seed returned empty asks"
    );
    assert!(
        delta_count >= 1,
        "expected >= 1 OrderbookDelta after seed snapshot in 10s (got {delta_count})"
    );
}
