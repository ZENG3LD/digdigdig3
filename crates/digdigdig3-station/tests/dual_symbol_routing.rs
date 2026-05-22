//! Dual-symbol routing test — confirms two consumers subscribed to different
//! symbols on the SAME Station receive ONLY their own events.
//!
//! Bug class motivating the StreamEvent typed-keys refactor: before, OB variants
//! had no symbol field, so `station::event_matches_key` returned `None`
//! (accept-all) for OB events. A BTC-USDT OB subscriber would receive ETH-USDT
//! OB events too.
//!
//! Live API (Binance) — run with `--ignored`.

use std::collections::HashMap;
use std::time::Duration;

use digdigdig3_station::{AccountType, ExchangeId, Station, Stream, SubscriptionSet};
use tokio::time::timeout;

const COLLECT: Duration = Duration::from_secs(8);

fn assert_no_pollination(label: &str, expected_sym: &str, counts: &HashMap<String, u32>) {
    let self_count = counts
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case(expected_sym))
        .map(|(_, v)| *v)
        .sum::<u32>();
    let pollution: u32 = counts
        .iter()
        .filter(|(k, _)| !k.eq_ignore_ascii_case(expected_sym))
        .map(|(_, v)| *v)
        .sum();
    assert!(
        self_count > 0,
        "{label}: received 0 {expected_sym} events — feed dead? counts={counts:?}"
    );
    assert_eq!(
        pollution, 0,
        "{label}: received {pollution} cross-symbol events (expected only {expected_sym}): {counts:?}"
    );
}

#[tokio::test]
#[ignore] // live API
async fn trades_dual_symbol_no_cross_pollination() {
    let station = Station::builder().build().await.expect("Station::build");

    let btc_set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );
    let eth_set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "ETH-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let mut btc_h = station.subscribe(btc_set).await.expect("btc subscribe").handle;
    let mut eth_h = station.subscribe(eth_set).await.expect("eth subscribe").handle;

    let mut btc_counts: HashMap<String, u32> = HashMap::new();
    let mut eth_counts: HashMap<String, u32> = HashMap::new();
    let deadline = tokio::time::Instant::now() + COLLECT;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let tick = remaining.min(Duration::from_millis(500));
        tokio::select! {
            r = timeout(tick, btc_h.recv()) => {
                if let Ok(Some(ev)) = r {
                    *btc_counts.entry(ev.symbol().to_string()).or_default() += 1;
                }
            }
            r = timeout(tick, eth_h.recv()) => {
                if let Ok(Some(ev)) = r {
                    *eth_counts.entry(ev.symbol().to_string()).or_default() += 1;
                }
            }
        }
    }

    println!("\nBTC consumer received: {:?}", btc_counts);
    println!("ETH consumer received: {:?}", eth_counts);

    // Station Event.symbol() returns user-input format ("BTC-USDT"), not raw exchange-native.
    assert_no_pollination("BTC trade consumer", "BTC-USDT", &btc_counts);
    assert_no_pollination("ETH trade consumer", "ETH-USDT", &eth_counts);
}

#[tokio::test]
#[ignore] // live API
async fn orderbook_dual_symbol_no_cross_pollination() {
    let station = Station::builder().build().await.expect("Station::build");

    let btc_set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Orderbook],
    );
    let eth_set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "ETH-USDT",
        AccountType::Spot,
        [Stream::Orderbook],
    );

    let mut btc_h = station.subscribe(btc_set).await.expect("btc subscribe").handle;
    let mut eth_h = station.subscribe(eth_set).await.expect("eth subscribe").handle;

    let mut btc_counts: HashMap<String, u32> = HashMap::new();
    let mut eth_counts: HashMap<String, u32> = HashMap::new();
    let deadline = tokio::time::Instant::now() + COLLECT;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let tick = remaining.min(Duration::from_millis(500));
        tokio::select! {
            r = timeout(tick, btc_h.recv()) => {
                if let Ok(Some(ev)) = r {
                    *btc_counts.entry(ev.symbol().to_string()).or_default() += 1;
                }
            }
            r = timeout(tick, eth_h.recv()) => {
                if let Ok(Some(ev)) = r {
                    *eth_counts.entry(ev.symbol().to_string()).or_default() += 1;
                }
            }
        }
    }

    println!("\nBTC OB consumer received: {:?}", btc_counts);
    println!("ETH OB consumer received: {:?}", eth_counts);

    assert_no_pollination("BTC OB consumer", "BTC-USDT", &btc_counts);
    assert_no_pollination("ETH OB consumer", "ETH-USDT", &eth_counts);
}

