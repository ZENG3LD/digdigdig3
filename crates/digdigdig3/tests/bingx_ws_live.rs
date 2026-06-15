//! BingX WebSocket live integration test.
//!
//! Subscribes to:
//! - Ticker (`@bookTicker`) for BTC-USDT
//! - Orderbook (`@depth5`) for BTC-USDT
//!
//! Channels NOT available on BingX swap-market WS (verified 2026-05-29):
//! - @fundingRate, @openInterest, @aggTrade, @forceOrder all return
//!   code 80015 "dataType not support" from the server.
//!
//! Run with:
//!   cargo test --test bingx_ws_live -- --nocapture --ignored

use std::sync::Arc;
use std::time::Duration;

use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMsg;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::traits::WebSocketConnector;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

async fn make_ws() -> Arc<dyn WebSocketConnector> {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::BingX, &[AccountType::Spot], false)
        .await
        .expect("connect_full BingX");
    let ws = hub
        .ws(ExchangeId::BingX, AccountType::Spot)
        .expect("no WS connector after connect_full");
    ws.connect(AccountType::Spot).await.expect("ws.connect");
    ws
}

fn btc_usdt() -> Symbol {
    Symbol::with_raw("", "", "BTC-USDT".to_string())
}

#[tokio::test]
#[ignore] // live API — run with: cargo test --test bingx_ws_live -- --nocapture --ignored
async fn bingx_ticker_and_orderbook_receive_events() {
    let ws = make_ws().await;
    let btcusdt = btc_usdt();

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

/// Probe test: directly verify that BingX swap-market WS rejects
/// @fundingRate/@openInterest/@aggTrade/@forceOrder with code 80015.
///
/// This test documents why these 4 StreamKinds return NotImplemented.
#[tokio::test]
#[ignore]
async fn bingx_probe_unsupported_channels_return_80015() {
    use flate2::read::GzDecoder;
    use std::io::Read;

    // Use `ring` provider — that's the feature enabled in workspace Cargo.toml
    // (rustls = { features = ["ring"] }). aws_lc_rs would need a feature flip.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let url = "wss://open-api-swap.bingx.com/swap-market";
    let (mut ws_stream, _) = connect_async(url).await.expect("raw connect");

    for suffix in ["fundingRate", "openInterest", "aggTrade", "forceOrder"] {
        let sub = format!(
            r#"{{"id":"probe_{0}","reqType":"sub","dataType":"BTC-USDT@{0}"}}"#,
            suffix
        );
        ws_stream.send(WsMsg::Text(sub.into())).await.unwrap();
    }

    let mut rejected = 0u32;
    let _ = timeout(Duration::from_secs(10), async {
        while let Some(msg) = ws_stream.next().await {
            if let Ok(WsMsg::Binary(bytes)) = msg {
                let mut decoder = GzDecoder::new(bytes.as_slice());
                let mut text = String::new();
                if decoder.read_to_string(&mut text).is_ok() {
                    eprintln!("  frame: {}", text);
                    if text.contains("80015") {
                        rejected += 1;
                    }
                    if rejected >= 4 {
                        break;
                    }
                }
            }
        }
    })
    .await;

    assert_eq!(
        rejected, 4,
        "expected 4 code-80015 rejections from BingX for unsupported channels, got {}",
        rejected
    );
    eprintln!("  confirmed: BingX swap WS rejects all 4 channels with code 80015");
}
