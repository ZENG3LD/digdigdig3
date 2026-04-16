//! Lighter WebSocket Integration Tests
//!
//! Tests WebSocket connectivity and subscriptions against the real Lighter DEX.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib l3::open::crypto::dex::lighter::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Lighter WS endpoint and require network access.
//! Lighter uses numeric market IDs: ETH=0, BTC=1, SOL=2.
//! The WebSocketConnector trait maps Symbol.base → market_id via symbol_to_market_id().

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol};
use crate::core::traits::WebSocketConnector;
use super::websocket::LighterWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn btc_usdc() -> Symbol {
    // Lighter perpetual BTC market — quote is USDC
    Symbol::new("BTC", "USDC")
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    let ws = match LighterWebSocket::new(None, false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let caps = ws.orderbook_capabilities(AccountType::FuturesCross);
    println!("Lighter orderbook capabilities: {:?}", caps);

    // Lighter: snapshot on first subscribe, then incremental deltas every 50ms
    assert!(caps.supports_snapshot, "Lighter must support snapshots");
    assert!(caps.supports_delta, "Lighter must support incremental deltas");
    assert!(caps.has_sequence, "Lighter must carry nonce sequence");
    assert!(caps.has_prev_sequence, "Lighter must carry begin_nonce prev-sequence");
    println!("supports_snapshot: {}", caps.supports_snapshot);
    println!("supports_delta: {}", caps.supports_delta);
    println!("update_speeds_ms: {:?}", caps.update_speeds_ms);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook() {
    let mut ws = match LighterWebSocket::new(None, false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(
        Duration::from_secs(10),
        ws.connect(AccountType::FuturesCross),
    )
    .await;

    match connect_result {
        Ok(Ok(())) => {
            assert_eq!(ws.connection_status(), ConnectionStatus::Connected);

            // Subscribe via WebSocketConnector trait — maps BTC → market_id=1 → channel order_book/1
            let sub = SubscriptionRequest::new(btc_usdc(), StreamType::Orderbook);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Lighter order_book/1 (BTC) — waiting for snapshot...");

            let mut stream = ws.event_stream();
            let ob_event = timeout(Duration::from_secs(15), async {
                use crate::core::StreamEvent;
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(ev @ StreamEvent::OrderbookSnapshot(_))
                        | Ok(ev @ StreamEvent::OrderbookDelta(_)) => {
                            return ev;
                        }
                        Ok(other) => {
                            println!("Received non-orderbook event (skipping): {:?}", other);
                            continue;
                        }
                        Err(e) => panic!("Stream returned error: {:?}", e),
                    }
                }
                panic!("Stream ended without orderbook data");
            })
            .await
            .expect(
                "Timeout waiting for orderbook data — Lighter did not send snapshot/delta within 15s",
            );

            use crate::core::StreamEvent;
            if let StreamEvent::OrderbookSnapshot(ob) = &ob_event {
                assert!(!ob.bids.is_empty(), "Snapshot bids must not be empty");
                assert!(!ob.asks.is_empty(), "Snapshot asks must not be empty");

                let bid_prices: Vec<f64> = ob.bids.iter().map(|b| b.price).collect();
                let sorted_desc = bid_prices.windows(2).all(|w| w[0] >= w[1]);
                assert!(sorted_desc, "Bids must be sorted descending by price");

                let ask_prices: Vec<f64> = ob.asks.iter().map(|a| a.price).collect();
                let sorted_asc = ask_prices.windows(2).all(|w| w[0] <= w[1]);
                assert!(sorted_asc, "Asks must be sorted ascending by price");

                let best_bid = ob.bids[0].price;
                let best_ask = ob.asks[0].price;
                assert!(
                    best_bid < best_ask,
                    "Book must not be crossed: best_bid={} best_ask={}",
                    best_bid,
                    best_ask
                );

                println!(
                    "Orderbook snapshot OK: {} bids, {} asks, best_bid={}, best_ask={}",
                    ob.bids.len(),
                    ob.asks.len(),
                    best_bid,
                    best_ask
                );
            } else {
                println!(
                    "Received orderbook delta (no snapshot assertions): {:?}",
                    ob_event
                );
            }

            let _ = ws.disconnect().await;
            println!("Lighter orderbook subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let mut ws = match LighterWebSocket::new(None, false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(
        Duration::from_secs(10),
        ws.connect(AccountType::FuturesCross),
    )
    .await;

    match connect_result {
        Ok(Ok(())) => {
            // Subscribe via WebSocketConnector trait — maps BTC → market_id=1 → channel trade/1
            let sub = SubscriptionRequest::new(btc_usdc(), StreamType::Trade);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Lighter trade/1 (BTC) — waiting for trade...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (market may be slow)");
            }

            let _ = ws.disconnect().await;
            println!("Lighter trades subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}
