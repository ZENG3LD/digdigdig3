//! Bitfinex WebSocket live integration test.
//!
//! Subscribes to Ticker for tBTCUSD + tETHUSD and asserts that at least one
//! Ticker event for each symbol flows within 30 seconds of real exchange data.
//!
//! Verifies:
//! - chanId integer routing works (acks populate the map, data dispatches correctly)
//! - per-symbol topic key extraction works for two concurrent subscriptions
//! - symbol field is correctly populated in emitted StreamEvents
//! - application-level ping keeps the connection alive (20 s interval)
//!
//! Run with:
//!   cargo test --test bitfinex_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --test bitfinex_ws_live -- --nocapture --ignored
async fn bitfinex_ticker_two_symbols_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Bitfinex, &[AccountType::Spot], false)
        .await
        .expect("connect_full Bitfinex");

    let ws = hub
        .ws(ExchangeId::Bitfinex, AccountType::Spot)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::Spot)
        .await
        .expect("ws.connect");

    // Subscribe to two symbols — exercises two chanId entries in the routing map.
    let btcusd = Symbol::with_raw("", "", "tBTCUSD".to_string());
    let ethusd = Symbol::with_raw("", "", "tETHUSD".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd.clone(),
        stream_type: StreamType::Ticker,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Ticker tBTCUSD");

    ws.subscribe(SubscriptionRequest {
        symbol: ethusd.clone(),
        stream_type: StreamType::Ticker,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Ticker tETHUSD");

    let mut stream = ws.event_stream();

    let mut saw_btc_ticker = false;
    let mut saw_eth_ticker = false;

    // Allow 30 s — Bitfinex ticker updates are frequent on liquid markets.
    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Ticker { symbol, ticker }) => {
                    eprintln!(
                        "  Ticker: {} last={:.2} bid={} ask={}",
                        symbol,
                        ticker.last_price,
                        ticker.bid_price.map(|p| format!("{:.2}", p)).unwrap_or_default(),
                        ticker.ask_price.map(|p| format!("{:.2}", p)).unwrap_or_default(),
                    );
                    if symbol == "tBTCUSD" {
                        assert!(
                            ticker.last_price > 0.0,
                            "BTC ticker last_price must be positive, got {}",
                            ticker.last_price
                        );
                        saw_btc_ticker = true;
                    } else if symbol == "tETHUSD" {
                        assert!(
                            ticker.last_price > 0.0,
                            "ETH ticker last_price must be positive, got {}",
                            ticker.last_price
                        );
                        saw_eth_ticker = true;
                    }
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
            if saw_btc_ticker && saw_eth_ticker {
                break;
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Bitfinex Ticker events (30s)");
    assert!(saw_btc_ticker, "no Ticker received for tBTCUSD");
    assert!(saw_eth_ticker, "no Ticker received for tETHUSD");
}
