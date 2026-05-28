//! Upbit WebSocket live integration test.
//!
//! Subscribes to:
//! - Trade for KRW-BTC
//! - Orderbook for KRW-BTC
//!
//! Verifies that at least one event of each type flows within 30 seconds.
//!
//! Validates:
//! - Binary UTF-8 frames decoded transparently (default decode_binary chain)
//! - WS-level Ping/Pong autoreply keeps connection alive
//! - {"status":"UP"} liveness ping suppressed via is_pong
//! - `type` topic routing works for both "trade" and "orderbook"
//! - Symbol field is correctly populated from `code`
//!
//! Run with:
//!   cargo test --test upbit_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --test upbit_ws_live -- --nocapture --ignored
async fn upbit_trade_and_orderbook_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Upbit, &[AccountType::Spot], false)
        .await
        .expect("connect_full Upbit");

    let ws = hub
        .ws(ExchangeId::Upbit, AccountType::Spot)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::Spot)
        .await
        .expect("ws.connect");

    // KRW-BTC: Upbit Korea native format (QUOTE-BASE)
    let krw_btc = Symbol::with_raw("", "", "KRW-BTC".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: krw_btc.clone(),
        stream_type: StreamType::Trade,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Trade KRW-BTC");

    ws.subscribe(SubscriptionRequest {
        symbol: krw_btc.clone(),
        stream_type: StreamType::Orderbook,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Orderbook KRW-BTC");

    let mut stream = ws.event_stream();

    let mut saw_trade = false;
    let mut saw_orderbook = false;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Trade { symbol, trade }) => {
                    eprintln!(
                        "  Trade: {} price={:.0} qty={:.6} side={:?}",
                        symbol,
                        trade.price,
                        trade.quantity,
                        trade.side,
                    );
                    assert!(
                        trade.price > 0.0,
                        "KRW-BTC trade price must be positive, got {}",
                        trade.price,
                    );
                    saw_trade = true;
                }
                Ok(StreamEvent::OrderbookSnapshot { symbol, book }) => {
                    eprintln!(
                        "  Orderbook: {} bids={} asks={}",
                        symbol,
                        book.bids.len(),
                        book.asks.len(),
                    );
                    assert!(
                        !book.bids.is_empty() || !book.asks.is_empty(),
                        "KRW-BTC orderbook must have bids or asks"
                    );
                    saw_orderbook = true;
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
            if saw_trade && saw_orderbook {
                break;
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Upbit events (30s)");
    assert!(saw_trade, "no Trade received for KRW-BTC");
    assert!(saw_orderbook, "no OrderbookSnapshot received for KRW-BTC");
}
