//! KuCoin WebSocket live integration tests.
//!
//! Tests that verify real data flows from KuCoin WS channels.
//!
//! Run with:
//!   cargo test --release --test kucoin_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

/// Subscribe to Liquidation (/contractMarket/liquidationOrders) for XBTUSDTM and
/// ETHUSDTM on KuCoin Futures.
///
/// Liquidations are sparse (~25/hr per symbol). The test subscribes two symbols
/// to increase hit probability over a 60s window.
///
/// PASS condition: subscribe succeeds + no stream error within 60s.
/// A liquidation event appearing is best-effort (sparse feed).
#[tokio::test]
#[ignore] // live API — run with: cargo test --release --test kucoin_ws_live -- --nocapture --ignored
async fn kucoin_liquidation_public_channel_no_error() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::KuCoin, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full KuCoin FuturesCross");

    let ws = hub
        .ws(ExchangeId::KuCoin, AccountType::FuturesCross)
        .expect("no WS connector after connect_full KuCoin");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect KuCoin futures");

    // Subscribe to two high-volume symbols to increase liquidation hit probability
    let xbt = Symbol::with_raw("", "", "XBTUSDTM".to_string());
    let eth = Symbol::with_raw("", "", "ETHUSDTM".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: xbt.clone(),
        stream_type: StreamType::Liquidation,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Liquidation XBTUSDTM must succeed (public channel)");

    ws.subscribe(SubscriptionRequest {
        symbol: eth.clone(),
        stream_type: StreamType::Liquidation,
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe Liquidation ETHUSDTM must succeed (public channel)");

    eprintln!("  [info] subscribed to XBTUSDTM + ETHUSDTM liquidation (60s window)");
    eprintln!("  [info] zero liquidations in window is acceptable (sparse feed)");

    let mut stream = ws.event_stream();

    let mut liq_count = 0u32;
    let mut stream_error = false;

    let result = timeout(Duration::from_secs(60), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::Liquidation { symbol, liquidation }) => {
                    liq_count += 1;
                    eprintln!(
                        "  Liquidation #{}: {} {:?} price={:.2} qty={:.4}",
                        liq_count, symbol, liquidation.side, liquidation.price, liquidation.quantity
                    );
                    assert!(!symbol.is_empty(), "liquidation symbol must be populated");
                    assert!(liquidation.price > 0.0, "liquidation price must be > 0");
                }
                Err(e) => {
                    eprintln!("  stream error: {:?}", e);
                    stream_error = true;
                    break;
                }
                Ok(other) => {
                    // Other events (e.g. trade, ticker) may arrive — ignore them
                    let _ = other;
                }
            }
        }
    })
    .await;

    // Timeout is expected (sparse feed). What must NOT happen: a subscribe error or
    // a stream error within the window.
    assert!(!stream_error, "stream emitted an error within 60s window");

    if result.is_err() {
        // Timed out (expected for sparse feed)
        eprintln!(
            "  result: timeout after 60s (expected), {} liquidations caught",
            liq_count
        );
    } else {
        eprintln!("  result: stream closed before timeout, {} liquidations", liq_count);
    }

    if liq_count > 0 {
        eprintln!("  BONUS: caught {} live liquidation(s)", liq_count);
    }

    // PASS: subscribe succeeded + no error. Liquidation presence is best-effort.
}
