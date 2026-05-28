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

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn gemini_ticker_synthetic_from_l2() {
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
        stream_type: StreamType::Ticker,
        account_type: AccountType::Spot,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Ticker must succeed (l2 synthesis)");

    let mut stream = ws.event_stream();

    let mut saw_ticker = false;
    let mut ticker_bid = 0.0_f64;
    let mut ticker_ask = 0.0_f64;
    let mut ticker_last = 0.0_f64;

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Ticker { symbol, ticker }) => {
                    let bid = ticker.bid_price.unwrap_or(0.0);
                    let ask = ticker.ask_price.unwrap_or(0.0);
                    eprintln!(
                        "  Ticker: {symbol} bid={bid:.2} ask={ask:.2} last={:.2}",
                        ticker.last_price
                    );
                    if bid > 0.0 && ask > 0.0 {
                        ticker_bid = bid;
                        ticker_ask = ask;
                        ticker_last = ticker.last_price;
                        saw_ticker = true;
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Gemini synthetic Ticker (30s)");
    assert!(saw_ticker, "no valid Ticker event received from Gemini");
    assert!(
        ticker_bid.is_finite() && ticker_bid > 0.0,
        "bid_price must be positive, got {ticker_bid}"
    );
    assert!(
        ticker_ask.is_finite() && ticker_ask > 0.0,
        "ask_price must be positive, got {ticker_ask}"
    );
    assert!(
        ticker_bid < ticker_ask,
        "bid ({ticker_bid}) must be < ask ({ticker_ask})"
    );
    assert!(ticker_last > 0.0, "last_price must be positive, got {ticker_last}");
    eprintln!("PASS: Gemini synthetic Ticker bid={ticker_bid:.2} ask={ticker_ask:.2} last={ticker_last:.2}, bid<ask={}", ticker_bid < ticker_ask);
}
