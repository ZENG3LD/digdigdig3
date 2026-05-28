//! GateIO WebSocket live integration tests.
//!
//! Tests that verify real data flows from GateIO WS channels.
//!
//! Run with:
//!   cargo test --release --test gateio_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest};

/// Subscribe to OpenInterest (futures.contract_stats) for BTC_USDT on the USDT futures WS.
/// contract_stats pushes every 10s — 30s window is sufficient.
#[tokio::test]
#[ignore] // live API — run with: cargo test --release --test gateio_ws_live -- --nocapture --ignored
async fn gateio_open_interest_btc_usdt_receives_event() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::GateIO, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full GateIO FuturesCross");

    let ws = hub
        .ws(ExchangeId::GateIO, AccountType::FuturesCross)
        .expect("no WS connector after connect_full");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect GateIO futures");

    // GateIO futures symbol format: BASE_QUOTE with underscore
    let sym = Symbol::with_raw("", "", "BTC_USDT".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: sym.clone(),
        stream_type: StreamType::OpenInterest,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe OpenInterest BTC_USDT");

    let mut stream = ws.event_stream();

    eprintln!("  [debug] waiting for events...");

    // contract_stats pushes at 1-minute boundaries — need up to 75s to guarantee at least one.
    let result = timeout(Duration::from_secs(75), async {
        while let Some(event) = stream.next().await {
            eprintln!("  [debug] raw event: {:?}", event);
            match event {
                Ok(StreamEvent::OpenInterestUpdate {
                    symbol,
                    open_interest,
                    open_interest_value,
                    timestamp,
                }) => {
                    eprintln!(
                        "  OpenInterest: {} oi={:.2} oi_value={:?} ts={}",
                        symbol, open_interest, open_interest_value, timestamp
                    );
                    assert!(!symbol.is_empty(), "symbol must be populated");
                    assert!(open_interest > 0.0, "open_interest must be > 0, got {}", open_interest);
                    assert!(timestamp > 0, "timestamp must be > 0");
                    return true;
                }
                Err(e) => {
                    eprintln!("  stream error: {:?}", e);
                }
                _ => {}
            }
        }
        false
    })
    .await;

    assert!(result.is_ok(), "timed out (75s) waiting for GateIO OpenInterest event");
    assert!(result.unwrap(), "no OpenInterestUpdate event received within 30s");
}
