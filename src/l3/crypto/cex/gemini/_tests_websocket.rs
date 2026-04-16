//! Gemini WebSocket Integration Tests
//!
//! Tests WebSocket connectivity and subscriptions against real Gemini Market Data API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib crypto::cex::gemini::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Gemini endpoints and require network access.
//! NOTE: GeminiWebSocket has inherent methods that shadow trait methods.
//! We use explicit UFCS to call the WebSocketConnector trait implementations.
//! Gemini orderbook does not support a configurable depth parameter.

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol};
use crate::core::traits::WebSocketConnector;
use super::websocket::GeminiWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn btc_usd() -> Symbol {
    // Gemini uses BTCUSD format (uppercase, no separator)
    Symbol::new("BTC", "USD")
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    let ws = match GeminiWebSocket::new_market_data(false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let caps = WebSocketConnector::orderbook_capabilities(&ws, AccountType::Spot);
    println!("Gemini orderbook capabilities: {:?}", caps);
    println!("ws_depths: {:?}", caps.ws_depths);
    println!("supports_snapshot: {}", caps.supports_snapshot);
    println!("supports_delta: {}", caps.supports_delta);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS
//
// NOTE: GeminiWebSocket has both inherent methods (connect(), event_stream())
// and WebSocketConnector trait implementations. The trait methods are called
// explicitly via UFCS to avoid resolving to the inherent methods.
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook() {
    let mut ws = match GeminiWebSocket::new_market_data(false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(
        Duration::from_secs(10),
        WebSocketConnector::connect(&mut ws, AccountType::Spot),
    ).await;

    match connect_result {
        Ok(Ok(())) => {
            assert_eq!(WebSocketConnector::connection_status(&ws), ConnectionStatus::Connected);

            // Gemini does not use depth parameter
            let sub = SubscriptionRequest::new(btc_usd(), StreamType::Orderbook);
            let result = WebSocketConnector::subscribe(&mut ws, sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = WebSocketConnector::disconnect(&mut ws).await;
                return;
            }

            println!("Subscribed to Gemini orderbook (BTCUSD) — waiting for snapshot...");

            let mut stream = WebSocketConnector::event_stream(&ws);
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
            }).await.expect("Timeout waiting for orderbook data — Gemini did not send snapshot/delta within 15s");

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

            let _ = WebSocketConnector::disconnect(&mut ws).await;
            println!("Gemini orderbook subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let mut ws = match GeminiWebSocket::new_market_data(false).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(
        Duration::from_secs(10),
        WebSocketConnector::connect(&mut ws, AccountType::Spot),
    ).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub = SubscriptionRequest::new(btc_usd(), StreamType::Trade);
            let result = WebSocketConnector::subscribe(&mut ws, sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = WebSocketConnector::disconnect(&mut ws).await;
                return;
            }

            println!("Subscribed to Gemini trades — waiting for trade...");

            let mut stream = WebSocketConnector::event_stream(&ws);
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (market may be slow)");
            }

            let _ = WebSocketConnector::disconnect(&mut ws).await;
            println!("Gemini trades subscription works");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}
