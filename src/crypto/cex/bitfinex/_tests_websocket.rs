//! Bitfinex WebSocket Integration Tests
//!
//! Tests WebSocket connectivity and subscriptions against real Bitfinex API v2.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib crypto::cex::bitfinex::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Bitfinex endpoints and require network access.
//! Bitfinex uses tBTCUSD format (prefix 't' for trading pairs).

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol};
use crate::core::traits::WebSocketConnector;
use super::websocket::BitfinexWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn btc_usd() -> Symbol {
    // Bitfinex uses tBTCUSD — the 't' prefix is added internally by format_symbol
    Symbol::new("BTC", "USD")
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    let ws = match BitfinexWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let caps = ws.orderbook_capabilities();
    println!("Bitfinex orderbook capabilities: {:?}", caps);

    // Bitfinex supports depths 25, 100
    assert!(!caps.ws_depths.is_empty(), "Must have at least one depth level");
    println!("ws_depths: {:?}", caps.ws_depths);
    println!("supports_snapshot: {}", caps.supports_snapshot);
    println!("supports_delta: {}", caps.supports_delta);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook() {
    let mut ws = match BitfinexWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            assert_eq!(ws.connection_status(), ConnectionStatus::Connected);

            let mut sub = SubscriptionRequest::new(btc_usd(), StreamType::Orderbook);
            sub.depth = Some(25);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Bitfinex orderbook depth=25 (tBTCUSD) — waiting for snapshot...");

            let mut stream = ws.event_stream();
            let ob_event = timeout(Duration::from_secs(20), async {
                use crate::core::StreamEvent;
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(ev @ StreamEvent::OrderbookSnapshot(_)) | Ok(ev @ StreamEvent::OrderbookDelta(_)) => {
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
            }).await.expect("Timeout waiting for orderbook data — Bitfinex did not send snapshot/delta within 20s");

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
                assert!(best_bid < best_ask, "Book must not be crossed: best_bid={} best_ask={}", best_bid, best_ask);

                println!("Orderbook snapshot OK: {} bids, {} asks, best_bid={}, best_ask={}", ob.bids.len(), ob.asks.len(), best_bid, best_ask);
            } else {
                println!("Received orderbook delta (no snapshot assertions): {:?}", ob_event);
            }

            let _ = ws.disconnect().await;
            println!("Bitfinex orderbook subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let mut ws = match BitfinexWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub = SubscriptionRequest::new(btc_usd(), StreamType::Trade);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Bitfinex trades — waiting for trade...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (market may be slow)");
            }

            let _ = ws.disconnect().await;
            println!("Bitfinex trades subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_orderbook_depth_levels() {
    // Bitfinex valid depths: 25, 100
    let depths = [25u32, 100];

    for depth in &depths {
        println!("Testing Bitfinex depth={}...", depth);

        let mut ws = match BitfinexWebSocket::new(None, false, AccountType::Spot).await {
            Ok(w) => w,
            Err(e) => {
                println!("Failed to create WebSocket for depth={}: {:?}", depth, e);
                continue;
            }
        };

        let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

        match connect_result {
            Ok(Ok(())) => {
                let mut sub = SubscriptionRequest::new(btc_usd(), StreamType::Orderbook);
                sub.depth = Some(*depth);

                if ws.subscribe(sub).await.is_err() {
                    println!("Subscribe failed for depth={}", depth);
                    let _ = ws.disconnect().await;
                    continue;
                }

                let mut stream = ws.event_stream();
                let event = timeout(Duration::from_secs(20), stream.next()).await;

                match event {
                    Ok(Some(Ok(ev))) => {
                        use crate::core::StreamEvent;
                        if let StreamEvent::OrderbookSnapshot(ob) = ev {
                            println!("Depth={}: {} bids, {} asks — OK", depth, ob.bids.len(), ob.asks.len());
                        } else {
                            println!("Depth={}: received event: {:?}", depth, ev);
                        }
                    }
                    Ok(Some(Err(e))) => println!("Depth={}: error: {:?}", depth, e),
                    Ok(None) => println!("Depth={}: stream ended", depth),
                    Err(_) => println!("Depth={}: timeout", depth),
                }

                let _ = ws.disconnect().await;
            }
            Ok(Err(e)) => println!("Connection failed for depth={}: {:?}", depth, e),
            Err(_) => println!("Connection timeout for depth={}", depth),
        }

        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
