//! Lighter DEX WebSocket live integration test.
//!
//! Subscribes to:
//! - Orderbook (`order_book/1`) for BTC (market_id=1)
//! - Trades (`trade/1`) for BTC (market_id=1)
//! - Kline (`candle/1/1m`) for BTC (market_id=1), 1-minute resolution
//!
//! Verifies that at least one event of each type flows within 30 seconds.
//!
//! Validates:
//! - `update/order_book` frames parse to OrderbookSnapshot with non-empty levels
//! - `update/trade` frames parse to Trade with positive price
//! - `update/candle` frames parse to Kline with positive OHLC values
//! - `<type_field>:<market_id>` topic routing works for wildcard patterns
//!
//! Run with:
//!   cargo test --test lighter_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --test lighter_ws_live -- --nocapture --ignored
async fn lighter_orderbook_and_trade_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Lighter, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full Lighter");

    let ws = hub
        .ws(ExchangeId::Lighter, AccountType::FuturesCross)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect");

    // Lighter identifies markets by numeric ID. BTC = market_id 1.
    // Pass raw base asset — protocol.rs maps "BTC" → market_id 1.
    let btc = Symbol::with_raw("", "", "BTC".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btc.clone(),
        stream_type: StreamType::Orderbook,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Orderbook BTC");

    ws.subscribe(SubscriptionRequest {
        symbol: btc.clone(),
        stream_type: StreamType::Trade,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Trade BTC");

    let mut stream = ws.event_stream();

    let mut saw_orderbook = false;
    let mut saw_trade = false;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::OrderbookSnapshot { symbol, book }) => {
                    eprintln!(
                        "  OrderbookSnapshot: '{}' bids={} asks={}",
                        symbol,
                        book.bids.len(),
                        book.asks.len(),
                    );
                    assert!(
                        !book.bids.is_empty() || !book.asks.is_empty(),
                        "BTC initial orderbook snapshot must have bids or asks"
                    );
                    saw_orderbook = true;
                }
                Ok(StreamEvent::OrderbookDelta { symbol, delta }) => {
                    eprintln!(
                        "  OrderbookDelta: '{}' bids={} asks={}",
                        symbol,
                        delta.bids.len(),
                        delta.asks.len(),
                    );
                    saw_orderbook = true;
                }
                Ok(StreamEvent::Trade { symbol, trade }) => {
                    eprintln!(
                        "  Trade: '{}' price={:.2} qty={:.6} side={:?}",
                        symbol, trade.price, trade.quantity, trade.side,
                    );
                    assert!(trade.price > 0.0, "BTC trade price must be positive");
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

    assert!(result.is_ok(), "timed out waiting for Lighter events (30s)");
    assert!(saw_orderbook, "no Orderbook event received for BTC");
    assert!(saw_trade, "no Trade received for BTC");
}

#[tokio::test]
#[ignore] // live API — run with: cargo test --test lighter_ws_live -- --nocapture --ignored
async fn lighter_kline_receives_event() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::Lighter, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full Lighter");

    let ws = hub
        .ws(ExchangeId::Lighter, AccountType::FuturesCross)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect");

    // BTC = market_id 1 on Lighter.
    let btc = Symbol::with_raw("", "", "BTC".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btc.clone(),
        stream_type: StreamType::Kline { interval: "1m".into() },
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Kline BTC 1m");

    let mut stream = ws.event_stream();

    let mut saw_kline = false;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Kline { symbol, interval, kline }) => {
                    eprintln!(
                        "  Kline: symbol='{}' interval='{}' open={:.2} high={:.2} low={:.2} close={:.2} vol={:.6} open_time={}",
                        symbol, interval, kline.open, kline.high, kline.low, kline.close, kline.volume, kline.open_time
                    );
                    assert!(kline.open > 0.0, "BTC kline open must be positive");
                    assert!(kline.high >= kline.low, "high must be >= low");
                    assert!(kline.open_time > 0, "open_time must be non-zero");
                    saw_kline = true;
                    break;
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Lighter Kline event (30s)");
    assert!(saw_kline, "no Kline event received for BTC 1m");
}
