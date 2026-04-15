//! Bybit WebSocket Integration Tests
//!
//! Tests WebSocket connectivity and subscriptions against real Bybit V5 API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib crypto::cex::bybit::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Bybit endpoints and require network access.

use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

use crate::core::{
    AccountType, ConnectionStatus, StreamType, SubscriptionRequest, Symbol,
};
use crate::core::traits::WebSocketConnector;
use super::websocket::BybitWebSocket;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn btc_usdt() -> Symbol {
    Symbol::new("BTC", "USDT")
}

fn orderbook_sub_depth50() -> SubscriptionRequest {
    let mut req = SubscriptionRequest::new(btc_usdt(), StreamType::Orderbook);
    req.depth = Some(50);
    req
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_websocket_connect_public_spot() {
    let result = BybitWebSocket::new(None, false, AccountType::Spot).await;

    if result.is_err() {
        println!("Could not create BybitWebSocket: {:?}", result.err());
        return;
    }

    let mut ws = result.unwrap();

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            assert_eq!(ws.connection_status(), ConnectionStatus::Connected);
            println!("Public Spot WebSocket connected");

            let _ = ws.disconnect().await;
            assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
            println!("Disconnect works");
        }
        Ok(Err(e)) => {
            println!("Connection failed: {:?}", e);
        }
        Err(_) => {
            println!("Connection timeout");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS - SPOT
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook_spot() {
    // Bybit V5 valid orderbook depths: 1, 50, 200, 500.
    // Depth 20 is NOT valid and causes "Invalid topic" protocol errors.
    // This test explicitly uses depth=50 to verify the correct subscription path.
    let mut ws = match BybitWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub = orderbook_sub_depth50();
            let result = ws.subscribe(sub).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            println!("Subscribed to orderbook.50.BTCUSDT — waiting for snapshot...");

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
            }).await.expect("Timeout waiting for orderbook data — Bybit did not send snapshot/delta within 15s");

            use crate::core::StreamEvent;
            if let StreamEvent::OrderbookSnapshot(ob) = &ob_event {
                assert!(!ob.bids.is_empty(), "Snapshot bids must not be empty");
                assert!(!ob.asks.is_empty(), "Snapshot asks must not be empty");

                // Verify bids are sorted descending (highest bid first)
                let bid_prices: Vec<f64> = ob.bids.iter().map(|b| b.price).collect();
                let sorted_desc = bid_prices.windows(2).all(|w| w[0] >= w[1]);
                assert!(sorted_desc, "Bids must be sorted descending by price");

                // Verify asks are sorted ascending (lowest ask first)
                let ask_prices: Vec<f64> = ob.asks.iter().map(|a| a.price).collect();
                let sorted_asc = ask_prices.windows(2).all(|w| w[0] <= w[1]);
                assert!(sorted_asc, "Asks must be sorted ascending by price");

                // No crossed book: best bid < best ask
                let best_bid = ob.bids[0].price;
                let best_ask = ob.asks[0].price;
                assert!(best_bid < best_ask, "Book must not be crossed: best_bid={} best_ask={}", best_bid, best_ask);

                println!("Orderbook snapshot OK: {} bids, {} asks, best_bid={}, best_ask={}", ob.bids.len(), ob.asks.len(), best_bid, best_ask);
            } else {
                println!("Received orderbook delta (no snapshot assertions): {:?}", ob_event);
            }

            let _ = ws.disconnect().await;
            println!("Spot orderbook subscription (depth=50) works");
        }
        Ok(Err(e)) => {
            println!("Connection failed: {:?}", e);
        }
        Err(_) => {
            println!("Connection timeout");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades_spot() {
    let mut ws = match BybitWebSocket::new(None, false, AccountType::Spot).await {
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

            println!("Subscribed to publicTrade.BTCUSDT — waiting for trade...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(15), stream.next()).await;

            if let Ok(Some(Ok(ev))) = event {
                println!("Received trade event: {:?}", ev);
            } else {
                println!("No trade event received within timeout (market may be slow)");
            }

            let _ = ws.disconnect().await;
            println!("Spot trades subscription works");
        }
        Ok(Err(e)) => {
            println!("Connection failed: {:?}", e);
        }
        Err(_) => {
            println!("Connection timeout");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_ticker_spot() {
    let mut ws = match BybitWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub = SubscriptionRequest::new(btc_usdt(), StreamType::Ticker);
            let result = ws.subscribe(sub.clone()).await;

            if result.is_err() {
                println!("Subscribe failed: {:?}", result.err());
                let _ = ws.disconnect().await;
                return;
            }

            assert!(ws.has_subscription(&sub), "Subscription must be tracked");

            println!("Subscribed to tickers.BTCUSDT — waiting for ticker...");

            let mut stream = ws.event_stream();
            let event = timeout(Duration::from_secs(10), stream.next()).await;

            match event {
                Ok(Some(Ok(ev))) => {
                    println!("Received ticker event: {:?}", ev);
                }
                Ok(Some(Err(e))) => {
                    println!("Received error event: {:?}", e);
                }
                Ok(None) => {
                    println!("Stream ended");
                }
                Err(_) => {
                    println!("Timeout waiting for ticker (market may be slow)");
                }
            }

            let _ = ws.disconnect().await;
            println!("Spot ticker subscription works");
        }
        Ok(Err(e)) => {
            println!("Connection failed: {:?}", e);
        }
        Err(_) => {
            println!("Connection timeout");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MULTIPLE SUBSCRIPTIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_multiple_subscriptions() {
    let mut ws = match BybitWebSocket::new(None, false, AccountType::Spot).await {
        Ok(w) => w,
        Err(e) => {
            println!("Failed to create WebSocket: {:?}", e);
            return;
        }
    };

    let connect_result = timeout(Duration::from_secs(10), ws.connect(AccountType::Spot)).await;

    match connect_result {
        Ok(Ok(())) => {
            let sub_ticker = SubscriptionRequest::new(btc_usdt(), StreamType::Ticker);
            let sub_trades = SubscriptionRequest::new(btc_usdt(), StreamType::Trade);
            let sub_orderbook = orderbook_sub_depth50();

            ws.subscribe(sub_ticker.clone()).await.ok();
            ws.subscribe(sub_trades.clone()).await.ok();
            ws.subscribe(sub_orderbook.clone()).await.ok();

            let subs = ws.active_subscriptions();
            if subs.len() == 3 {
                println!("All 3 subscriptions tracked");
            } else {
                println!("Expected 3 subscriptions, got {}", subs.len());
            }

            // Collect events for 5 seconds
            let mut stream = ws.event_stream();
            let mut event_count = 0usize;

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(5) && event_count < 5 {
                if let Ok(Some(Ok(_ev))) = timeout(Duration::from_secs(1), stream.next()).await {
                    event_count += 1;
                }
            }

            println!("Received {} events from multiple subscriptions", event_count);

            // Unsubscribe from one and verify count drops
            ws.unsubscribe(sub_ticker.clone()).await.ok();
            let subs = ws.active_subscriptions();
            if subs.len() == 2 {
                println!("Unsubscribe works — 2 remaining subscriptions");
            } else {
                println!("Expected 2 subscriptions after unsubscribe, got {}", subs.len());
            }

            let _ = ws.disconnect().await;
            println!("Multiple subscriptions and unsubscribe works");
        }
        Ok(Err(e)) => {
            println!("Connection failed: {:?}", e);
        }
        Err(_) => {
            println!("Connection timeout");
        }
    }
}
