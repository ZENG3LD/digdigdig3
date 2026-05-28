//! Crypto.com WebSocket live integration test.
//!
//! Subscribes to Trade + MarkPrice on BTCUSD-PERP and asserts at least 1 event
//! of each type flows within 30 seconds of real exchange data.
//!
//! BTCUSD-PERP is a perpetual contract — it proves derivative channels work.
//!
//! Run with:
//!   cargo test --test crypto_com_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

#[tokio::test]
#[ignore] // live API — run with --ignored
async fn crypto_com_perp_trade_and_mark_price_receive_events() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::CryptoCom, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full CryptoCom");

    let ws = hub
        .ws(ExchangeId::CryptoCom, AccountType::FuturesCross)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect");

    // Crypto.com perpetual: BTCUSD-PERP
    let btcusd_perp = Symbol::with_raw("", "", "BTCUSD-PERP".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd_perp.clone(),
        stream_type: StreamType::Trade,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Trade BTCUSD-PERP");

    ws.subscribe(SubscriptionRequest {
        symbol: btcusd_perp.clone(),
        stream_type: StreamType::MarkPrice,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe MarkPrice BTCUSD-PERP");

    let mut stream = ws.event_stream();

    let mut saw_trade = false;
    let mut saw_mark_price = false;

    // Crypto.com has a 1 s mandatory connect delay plus subscription latency;
    // allow 30 s for at least one trade and one mark-price update.
    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Trade { symbol, trade }) => {
                    eprintln!("  Trade: {} @ {} qty={}", symbol, trade.price, trade.quantity);
                    saw_trade = true;
                }
                Ok(StreamEvent::MarkPrice { symbol, mark_price, .. }) => {
                    eprintln!("  MarkPrice: {} = {}", symbol, mark_price);
                    saw_mark_price = true;
                }
                Err(e) => {
                    eprintln!("stream error: {:?}", e);
                }
                _ => {}
            }
            if saw_trade && saw_mark_price {
                break;
            }
        }
    })
    .await;

    assert!(result.is_ok(), "timed out waiting for Crypto.com WS events (30s)");
    assert!(saw_trade, "no Trade received from Crypto.com BTCUSD-PERP");
    assert!(
        saw_mark_price,
        "no MarkPrice received from Crypto.com BTCUSD-PERP (mark channel may only push on price change)"
    );
}
