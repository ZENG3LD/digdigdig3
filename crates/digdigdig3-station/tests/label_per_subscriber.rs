#![cfg(not(target_arch = "wasm32"))]
//! Label-per-subscriber regression test.
//!
//! Two SubscriptionHandles on the SAME Station, subscribed to the SAME
//! (exchange, symbol-after-normalization, kind) — but with DIFFERENT input
//! symbol formats:
//!
//!   handle A: SubscriptionSet::add(.., "BTC-USDT", ..)   → routes to BTCUSDT
//!   handle B: SubscriptionSet::add(.., "BTCUSDT",  ..)   → routes to BTCUSDT
//!
//! Both share ONE underlying multiplex (1 WS connection, 1 broadcast). The
//! routing key is the raw exchange-native symbol (`"BTCUSDT"`).
//!
//! Each handle must receive Events carrying ITS OWN input label
//! ("BTC-USDT" for A, "BTCUSDT" for B), not the label of whichever consumer
//! arrived first.
//!
//! Live API (Binance) — run with `--ignored`.

use std::time::Duration;

use digdigdig3_station::{AccountType, ExchangeId, Station, Stream, SubscriptionSet};
use tokio::time::timeout;

const COLLECT: Duration = Duration::from_secs(6);

#[tokio::test]
#[ignore] // live API
async fn dual_format_each_handle_keeps_its_own_label() {
    let station = Station::builder().build().await.expect("Station::build");

    // Subscribe twice to the same routing key — different input formats.
    let set_a = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT", // canonical-style input
        AccountType::Spot,
        [Stream::Trade],
    );
    let set_b = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTCUSDT", // raw exchange-native input
        AccountType::Spot,
        [Stream::Trade],
    );

    let mut h_a = station.subscribe(set_a).await.expect("subscribe a").handle;
    let mut h_b = station.subscribe(set_b).await.expect("subscribe b").handle;

    // Only one underlying multiplex should exist — they share.
    assert_eq!(
        station.active_streams(),
        1,
        "expected 1 shared multiplex for two same-routing-key subscribers"
    );

    let mut a_labels: std::collections::HashSet<String> = Default::default();
    let mut b_labels: std::collections::HashSet<String> = Default::default();
    let mut a_count = 0u32;
    let mut b_count = 0u32;

    let deadline = tokio::time::Instant::now() + COLLECT;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let tick = remaining.min(Duration::from_millis(500));
        tokio::select! {
            r = timeout(tick, h_a.recv()) => {
                if let Ok(Some(ev)) = r {
                    a_labels.insert(ev.symbol().to_string());
                    a_count += 1;
                }
            }
            r = timeout(tick, h_b.recv()) => {
                if let Ok(Some(ev)) = r {
                    b_labels.insert(ev.symbol().to_string());
                    b_count += 1;
                }
            }
        }
    }

    println!("\nhandle A ({} events) labels: {:?}", a_count, a_labels);
    println!("handle B ({} events) labels: {:?}", b_count, b_labels);

    assert!(a_count > 0, "handle A received 0 events — feed dead?");
    assert!(b_count > 0, "handle B received 0 events — feed dead?");

    assert_eq!(
        a_labels.len(),
        1,
        "handle A saw multiple labels: {:?}",
        a_labels
    );
    assert_eq!(
        b_labels.len(),
        1,
        "handle B saw multiple labels: {:?}",
        b_labels
    );
    assert!(
        a_labels.contains("BTC-USDT"),
        "handle A label = {:?} (expected only \"BTC-USDT\")",
        a_labels
    );
    assert!(
        b_labels.contains("BTCUSDT"),
        "handle B label = {:?} (expected only \"BTCUSDT\")",
        b_labels
    );
}
