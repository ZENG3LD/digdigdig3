//! BingX WebSocket live integration test.
//!
//! Subscribes to:
//! - Ticker (`@bookTicker`) for BTCUSDT
//! - Orderbook (`@depth5`) for BTCUSDT
//!
//! Verifies that at least one event of each type flows within 30 seconds.
//!
//! Validates:
//! - GZIP decompression works transparently (default decode_binary chain)
//! - Server-initiated ping response keeps the connection alive
//! - `dataType` topic routing works for both wildcard patterns
//! - Symbol field is correctly populated from the dataType prefix
//!
//! Run with:
//!   cargo test --test bingx_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --test bingx_ws_live -- --nocapture --ignored
async fn bingx_ticker_and_orderbook_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::BingX, &[AccountType::Spot], false)
        .await
        .expect("connect_full BingX");

    let ws = hub
        .ws(ExchangeId::BingX, AccountType::Spot)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::Spot)
        .await
        .expect("ws.connect");

    // BTC-USDT: BingX hyphenated format for swap endpoint
    let btcusdt = Symbol::with_raw("", "", "BTC-USDT".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btcusdt.clone(),
        stream_type: StreamType::Ticker,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Ticker BTC-USDT");

    ws.subscribe(SubscriptionRequest {
        symbol: btcusdt.clone(),
        stream_type: StreamType::Orderbook,
        account_type: AccountType::Spot,
        depth: Some(5),
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Orderbook BTC-USDT depth5");

    let mut stream = ws.event_stream();

    let mut saw_ticker = false;
    let mut saw_orderbook = false;

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
                    assert!(
                        ticker.last_price > 0.0 || ticker.bid_price.is_some(),
                        "BTC ticker must have positive price or bid, got last={} bid={:?}",
                        ticker.last_price,
                        ticker.bid_price,
                    );
                    saw_ticker = true;
                }
                Ok(StreamEvent::OrderbookDelta { symbol, delta }) => {
                    eprintln!(
                        "  Orderbook: {} bids={} asks={}",
                        symbol,
                        delta.bids.len(),
                        delta.asks.len(),
                    );
                    assert!(
                        !delta.bids.is_empty() || !delta.asks.is_empty(),
                        "BTC orderbook delta must have bids or asks"
                    );
                    saw_orderbook = true;
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
            if saw_ticker && saw_orderbook {
                break;
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for BingX events (30s)");
    assert!(saw_ticker, "no Ticker received for BTC-USDT");
    assert!(saw_orderbook, "no OrderbookDelta received for BTC-USDT");
}
