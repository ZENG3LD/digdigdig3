//! Upbit WebSocket live integration tests.
//!
//! NOTE: Upbit replaces ALL subscriptions when a new subscribe frame is sent
//! on the same connection. Tests that cover different channel combinations
//! therefore use separate connections (separate test functions).
//!
//! Run with:
//!   cargo test --release --test upbit_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with: cargo test --release --test upbit_ws_live -- --nocapture --ignored
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

/// Ticker-only test on a dedicated connection.
///
/// Upbit replaces all subscriptions per frame, so mixing Ticker with Trade/Orderbook
/// on the same connection causes only the last subscription to receive data.
/// This test uses a fresh connection subscribed to Ticker only.
#[tokio::test]
#[ignore] // live API — run with: cargo test --release --test upbit_ws_live -- --nocapture --ignored
async fn upbit_ticker_receives_events() {
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
        stream_type: StreamType::Ticker,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Ticker KRW-BTC");

    let mut stream = ws.event_stream();
    let mut saw_ticker = false;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Ticker { symbol, ticker }) => {
                    eprintln!(
                        "  Ticker: {} last={:.0} high={:?} low={:?} vol24h={:?} bid={:?} ask={:?}",
                        symbol,
                        ticker.last_price,
                        ticker.high_24h,
                        ticker.low_24h,
                        ticker.volume_24h,
                        ticker.bid_price,
                        ticker.ask_price,
                    );
                    assert!(
                        ticker.last_price > 0.0,
                        "KRW-BTC ticker last_price must be positive, got {}",
                        ticker.last_price,
                    );
                    // Upbit native ticker does NOT carry bid/ask
                    assert_eq!(
                        ticker.bid_price, None,
                        "bid_price must be None for Upbit native ticker"
                    );
                    assert_eq!(
                        ticker.ask_price, None,
                        "ask_price must be None for Upbit native ticker"
                    );
                    saw_ticker = true;
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

    assert!(result.is_ok(), "timed out waiting for Upbit Ticker (30s)");
    assert!(saw_ticker, "no Ticker received for KRW-BTC");
}
