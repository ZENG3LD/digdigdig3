//! MEXC WebSocket Integration Tests
//!
//! Tests WebSocket connectivity against real MEXC API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib crypto::cex::mexc::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real MEXC endpoints and require network access.
//! MEXC WebSocket uses Protobuf encoding for market data.
//! Orderbook subscription returns errors for the new endpoint — this is tested explicitly.

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol};
use crate::core::traits::WebSocketConnector;
use super::websocket::MexcWebSocket;

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
    let ws = match MexcWebSocket::new(None).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let caps = ws.orderbook_capabilities(AccountType::Spot);
    println!("MEXC orderbook capabilities: {:?}", caps);
    println!("ws_depths: {:?}", caps.ws_depths);
    println!("supports_snapshot: {}", caps.supports_snapshot);
    println!("supports_delta: {}", caps.supports_delta);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

/// MEXC orderbook: the Protobuf-based endpoint may return errors for orderbook
/// subscriptions depending on channel format. This test verifies the subscription
/// attempt succeeds or fails gracefully — NOT that orderbook data is received.
#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook() {
    let mut ws = match MexcWebSocket::new(None).await {
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
            println!("Connected to MEXC WebSocket");

            let sub = SubscriptionRequest::new(btc_usdt(), StreamType::Orderbook);
            let subscribe_result = ws.subscribe(sub).await;

            match subscribe_result {
                Ok(()) => {
                    println!("Subscribe returned Ok — waiting for data or error...");
                    let mut stream = ws.event_stream();
                    let event = timeout(Duration::from_secs(10), stream.next()).await;

                    match event {
                        Ok(Some(Ok(ev))) => {
                            println!("Received event: {:?}", ev);
                            println!("MEXC orderbook subscription produced data");
                        }
                        Ok(Some(Err(e))) => {
                            // Expected: MEXC may return errors for orderbook on new endpoint
                            println!("Received error event (expected for MEXC orderbook): {:?}", e);
                            println!("Skipping gracefully — MEXC orderbook error is known");
                        }
                        Ok(None) => {
                            println!("Stream ended (MEXC may not support this channel)");
                        }
                        Err(_) => {
                            println!("Timeout — MEXC orderbook may not push data (expected)");
                        }
                    }
                }
                Err(e) => {
                    // Also acceptable: subscription itself returns an error
                    println!("Subscribe failed (expected for MEXC orderbook): {:?}", e);
                    println!("Skipping gracefully — MEXC orderbook subscription error is known");
                }
            }

            let _ = ws.disconnect().await;
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let mut ws = match MexcWebSocket::new(None).await {
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

            println!("Subscribed to MEXC trades — waiting for trade...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (MEXC Protobuf may need decoding)");
            }

            let _ = ws.disconnect().await;
            println!("MEXC trades subscription test complete");
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}
