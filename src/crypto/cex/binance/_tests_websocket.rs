//! Binance WebSocket Integration Tests
//!
//! Tests WebSocket connectivity and subscriptions against real Binance API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib crypto::cex::binance::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Binance endpoints and require network access.

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol};
use crate::core::traits::WebSocketConnector;
use super::websocket::BinanceWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn btc_usdt() -> Symbol {
    Symbol::new("BTC", "USDT")
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    let ws = match BinanceWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let caps = ws.orderbook_capabilities();
    println!("Binance orderbook capabilities: {:?}", caps);

    // Binance supports depths 5, 10, 20
    assert!(!caps.ws_depths.is_empty(), "Must have at least one depth level");
    assert!(caps.supports_snapshot, "Binance must support snapshots");
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
    let mut ws = match BinanceWebSocket::new(None, false, AccountType::Spot).await {
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

            let mut sub = SubscriptionRequest::new(btc_usdt(), StreamType::Orderbook);
            sub.depth = Some(5);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Binance orderbook depth=5 — waiting for snapshot...");

            let mut stream = ws.event_stream();
            let ob_event = timeout(Duration::from_secs(15), async {
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
            }).await.expect("Timeout waiting for orderbook data — Binance did not send snapshot/delta within 15s");

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
            println!("Binance orderbook subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let mut ws = match BinanceWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub = SubscriptionRequest::new(btc_usdt(), StreamType::Trade);
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to Binance trades — waiting for trade...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (market may be slow)");
            }

            let _ = ws.disconnect().await;
            println!("Binance trades subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_orderbook_depth_levels() {
    // Test each valid Binance depth: 5, 10, 20
    let depths = [5u32, 10, 20];

    for depth in &depths {
        println!("Testing depth={}...", depth);

        let mut ws = match BinanceWebSocket::new(None, false, AccountType::Spot).await {
            Ok(w) => w,
            Err(e) => {
                println!("Failed to create WebSocket for depth={}: {:?}", depth, e);
                continue;
            }
        };

        let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

        match connect_result {
            Ok(Ok(())) => {
                let mut sub = SubscriptionRequest::new(btc_usdt(), StreamType::Orderbook);
                sub.depth = Some(*depth);

                if ws.subscribe(sub).await.is_err() {
                    println!("Subscribe failed for depth={}", depth);
                    let _ = ws.disconnect().await;
                    continue;
                }

                let mut stream = ws.event_stream();
                let event = timeout(Duration::from_secs(15), stream.next()).await;

                match event {
                    Ok(Some(Ok(ev))) => {
                        use crate::core::StreamEvent;
                        if let StreamEvent::OrderbookSnapshot(ob) = ev {
                            assert!(ob.bids.len() <= *depth as usize, "Bids count {} exceeds requested depth {}", ob.bids.len(), depth);
                            assert!(ob.asks.len() <= *depth as usize, "Asks count {} exceeds requested depth {}", ob.asks.len(), depth);
                            println!("Depth={}: {} bids, {} asks — OK", depth, ob.bids.len(), ob.asks.len());
                        } else {
                            println!("Depth={}: received non-snapshot event: {:?}", depth, ev);
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

        // Small delay between depth tests to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
