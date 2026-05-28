//! HTX WebSocket live integration tests.
//!
//! Tests that verify real data flows from HTX WS channels.
//!
//! Run with:
//!   cargo test --release --test htx_ws_live -- --nocapture --ignored

use std::time::Duration;

use futures_util::StreamExt;
use tokio::time::timeout;

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, StreamType, Symbol, SubscriptionRequest,
};

/// Subscribe to IndexPriceKline (market.BTC-USDT.index.1min) for BTC-USDT on HTX.
/// 1-minute klines push every ~1s within the minute window on HTX.
#[tokio::test]
#[ignore] // live API — run with: cargo test --release --test htx_ws_live -- --nocapture --ignored
async fn htx_index_price_kline_btcusdt_receives_event() {
    let hub = ExchangeHub::new();
    hub.connect_full(ExchangeId::HTX, &[AccountType::FuturesCross], false)
        .await
        .expect("connect_full HTX FuturesCross");

    let ws = hub
        .ws(ExchangeId::HTX, AccountType::FuturesCross)
        .expect("no WS connector after connect_full HTX");

    ws.connect(AccountType::FuturesCross)
        .await
        .expect("ws.connect HTX futures");

    // HTX futures uses BASE-QUOTE with dash (not baseUSDT)
    let sym = Symbol::with_raw("", "", "BTC-USDT".to_string());

    ws.subscribe(SubscriptionRequest {
        symbol: sym.clone(),
        stream_type: StreamType::IndexPriceKline { interval: "1m".to_string() },
        account_type: AccountType::FuturesCross,
        depth: None,
        update_speed_ms: None,
    })
    .await
    .expect("subscribe IndexPriceKline BTC-USDT 1m");

    let mut stream = ws.event_stream();

    eprintln!("  [debug] waiting for IndexPriceKline events...");

    let result = timeout(Duration::from_secs(30), async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::IndexPriceKline {
                    symbol,
                    interval,
                    kline,
                }) => {
                    eprintln!(
                        "  IndexPriceKline: {} {} open={:.2} close={:.2} ts={}",
                        symbol, interval, kline.open, kline.close, kline.open_time
                    );
                    assert!(!symbol.is_empty(), "symbol must be populated");
                    assert!(kline.open > 0.0, "open must be > 0, got {}", kline.open);
                    assert!(kline.close > 0.0, "close must be > 0, got {}", kline.close);
                    assert!(kline.open_time > 0, "open_time must be > 0");
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

    assert!(result.is_ok(), "timed out (30s) waiting for HTX IndexPriceKline event");
    assert!(result.unwrap(), "no IndexPriceKline event received within 30s");
}
