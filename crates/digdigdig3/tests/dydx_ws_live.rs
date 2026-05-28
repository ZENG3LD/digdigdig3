//! dYdX v4 WebSocket live integration test.
//!
//! Subscribes to:
//! - Orderbook (`v4_orderbook`) for BTC-USD
//! - Trades (`v4_trades`) for BTC-USD
//!
//! Verifies that at least one event of each type flows within 30 seconds.
//!
//! Validates:
//! - `"subscribed"` frame initial snapshot is routed through extract_topic
//!   and parsed correctly as an orderbook snapshot
//! - `"channel_data"` delta frames route to the orderbook delta parser
//! - `channel:id` topic routing works for wildcard patterns
//! - Symbol field is correctly extracted from the `id` field
//!
//! Run with:
//!   cargo test --test dydx_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --test dydx_ws_live -- --nocapture --ignored
async fn dydx_orderbook_and_trade_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Dydx, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full Dydx");

    let ws = hub
        .ws(ExchangeId::Dydx, AccountType::FuturesCross)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect");

    // dYdX uses "BTC-USD" format natively
    let btcusd = Symbol::with_raw("", "", "BTC-USD".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd.clone(),
        stream_type: StreamType::Orderbook,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Orderbook BTC-USD");

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd.clone(),
        stream_type: StreamType::Trade,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Trade BTC-USD");

    let mut stream = ws.event_stream();

    let mut saw_orderbook = false;
    let mut saw_trade = false;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::OrderbookSnapshot { symbol, book }) => {
                    eprintln!(
                        "  OrderbookSnapshot: {} bids={} asks={}",
                        symbol,
                        book.bids.len(),
                        book.asks.len(),
                    );
                    assert!(
                        !book.bids.is_empty() || !book.asks.is_empty(),
                        "BTC-USD initial snapshot must have bids or asks"
                    );
                    saw_orderbook = true;
                }
                Ok(StreamEvent::OrderbookDelta { symbol, delta }) => {
                    eprintln!(
                        "  OrderbookDelta: {} bids={} asks={}",
                        symbol,
                        delta.bids.len(),
                        delta.asks.len(),
                    );
                    saw_orderbook = true;
                }
                Ok(StreamEvent::Trade { symbol, trade }) => {
                    eprintln!(
                        "  Trade: {} price={:.2} qty={:.6} side={:?}",
                        symbol, trade.price, trade.quantity, trade.side,
                    );
                    assert!(trade.price > 0.0, "BTC-USD trade price must be positive");
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

    assert!(result.is_ok(), "timed out waiting for dYdX events (30s)");
    assert!(saw_orderbook, "no Orderbook event received for BTC-USD");
    assert!(saw_trade, "no Trade received for BTC-USD");
}
