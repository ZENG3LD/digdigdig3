//! Gemini WebSocket live integration test.
//!
//! Subscribes to Trade + Orderbook on BTCUSD and asserts at least 1 event of
//! each type flows within 20 seconds of real exchange data.
//!
//! Run with:
//!   cargo test --test gemini_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn gemini_orderbook_and_trade_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Gemini, &[AccountType::Spot], false)
        .await
        .expect("connect_full Gemini");

    let ws = hub
        .ws(ExchangeId::Gemini, AccountType::Spot)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::Spot)
        .await
        .expect("ws.connect");

    let btcusd = Symbol::with_raw("", "", "BTCUSD".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd.clone(),
        stream_type: StreamType::Orderbook,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Orderbook");

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd.clone(),
        stream_type: StreamType::Trade,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Trade");

    let mut stream = ws.event_stream();

    let mut saw_orderbook = false;
    let mut saw_trade = false;

    let result = timeout(Duration::from_secs(20), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::OrderbookDelta { symbol, .. }) => {
                    eprintln!("  OB delta: {symbol}");
                    saw_orderbook = true;
                }
                Ok(StreamEvent::Trade { symbol, .. }) => {
                    eprintln!("  Trade:    {symbol}");
                    saw_trade = true;
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
            if saw_orderbook && saw_trade {
                break;
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Gemini WS events (20s)");
    assert!(saw_orderbook, "no OrderbookDelta received from Gemini");
    assert!(saw_trade, "no Trade received from Gemini (l2_updates frames may not carry trades every frame — wait longer or check during active trading hours)");
}
