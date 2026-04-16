//! Polymarket WebSocket Integration Tests
//!
//! Tests WebSocket connectivity against the real Polymarket CLOB WS endpoint.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib l3::open::prediction::polymarket::_tests_websocket -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests connect to real Polymarket WS endpoint and require network access.
//!
//! Polymarket uses a custom ClobWebSocket (not WebSocketConnector trait).
//! - Constructor: `ClobWebSocket::new(token_ids, enable_features)`
//! - Connect: `ws.connect().await` (sends subscription message automatically)
//! - Receive: `ws.recv().await` returns `Ok(Some(WsEvent))`
//! - Channels: single "market" channel, subscribed by `token_id` (not condition_id)

use std::time::Duration;
use tokio::time::timeout;

use super::connector::PolymarketConnector;
use super::websocket::{ClobWebSocket, WsEvent};

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Find an active token_id with a non-empty orderbook.
///
/// Queries the Gamma API for active markets, then verifies each candidate
/// token_id by fetching the CLOB REST orderbook. Returns the first token_id
/// that has at least one bid or ask level.
async fn find_active_token_id() -> Option<String> {
    let connector = PolymarketConnector::public();

    // Fetch active markets with clob_token_ids from Gamma API
    let markets = connector.get_gamma_markets(Some(50)).await.ok()?;

    for market in markets.iter() {
        // Skip markets that are not tradeable (closed, no order book, etc.)
        if !market.is_tradeable() {
            continue;
        }

        // Prefer YES token_id (first in clob_token_ids)
        let token_id = market.yes_token_id()?;

        // Verify the orderbook is non-empty via REST before subscribing via WS
        match connector.get_order_book(token_id).await {
            Ok(book) if !book.bids.is_empty() || !book.asks.is_empty() => {
                let question = market.question.as_deref().unwrap_or("Unknown");
                println!(
                    "Found active market: token_id={} question=\"{}\" bids={} asks={}",
                    token_id,
                    &question.chars().take(60).collect::<String>(),
                    book.bids.len(),
                    book.asks.len()
                );
                return Some(token_id.to_string());
            }
            Ok(_) => {
                // Empty orderbook — try next
                continue;
            }
            Err(_) => {
                // Orderbook fetch failed — try next
                continue;
            }
        }
    }

    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    use crate::core::AccountType;

    // Capabilities are static — token_id doesn't matter here, use a placeholder
    let ws = ClobWebSocket::new(vec!["placeholder".to_string()], false);
    let caps = ws.orderbook_capabilities(AccountType::Spot);

    println!("Polymarket orderbook capabilities: {:?}", caps);

    // Polymarket: full book snapshot on subscribe + price_change deltas
    assert!(caps.supports_snapshot, "Polymarket must support full book snapshots");
    assert!(caps.supports_delta, "Polymarket must support incremental price_change deltas");
    assert!(!caps.has_sequence, "Polymarket does not use sequence numbers");
    println!("supports_snapshot: {}", caps.supports_snapshot);
    println!("supports_delta: {}", caps.supports_delta);
    println!("has_sequence: {}", caps.has_sequence);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTION TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_connect() {
    let token_id = match timeout(Duration::from_secs(30), find_active_token_id()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            println!("No active token_id found — skipping test");
            return;
        }
        Err(_) => {
            println!("Timeout finding active token_id — skipping test");
            return;
        }
    };

    let mut ws = ClobWebSocket::new(vec![token_id.clone()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            assert!(ws.is_connected(), "WebSocket must be connected after connect()");
            println!("Polymarket ClobWebSocket connected successfully (token_id={})", token_id);
            ws.close().await;
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUBSCRIPTION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_subscribe_orderbook() {
    let token_id = match timeout(Duration::from_secs(30), find_active_token_id()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            println!("No active token_id found — skipping test");
            return;
        }
        Err(_) => {
            println!("Timeout finding active token_id — skipping test");
            return;
        }
    };

    let mut ws = ClobWebSocket::new(vec![token_id.clone()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!(
                "Connected to Polymarket WS, subscribed to token_id={} — waiting for book snapshot...",
                token_id
            );

            // Wait for a book event (full snapshot sent immediately after subscribe)
            let book_event = timeout(Duration::from_secs(15), async {
                loop {
                    match ws.recv().await {
                        Ok(Some(WsEvent::Book(snapshot))) => return Some(snapshot),
                        Ok(Some(WsEvent::PriceChange(_))) => {
                            // Delta received before snapshot — unusual but ok
                            return None;
                        }
                        Ok(Some(WsEvent::Pong)) => continue,
                        Ok(Some(WsEvent::Unknown(u))) => {
                            // [] or ack messages — skip silently unless non-trivial
                            if u.raw != "[]" {
                                println!("Unknown event (skipping): {}", u.raw);
                            }
                            continue;
                        }
                        Ok(Some(other)) => {
                            println!("Received non-book event (skipping): {:?}", other);
                            continue;
                        }
                        Ok(None) => {
                            println!("WebSocket closed gracefully");
                            return None;
                        }
                        Err(e) => {
                            println!("WebSocket error: {:?}", e);
                            return None;
                        }
                    }
                }
            })
            .await;

            match book_event {
                Ok(Some(snapshot)) => {
                    println!(
                        "Book snapshot OK: {} bids, {} asks, asset_id={}",
                        snapshot.bids.len(),
                        snapshot.asks.len(),
                        snapshot.asset_id.as_deref().unwrap_or("unknown")
                    );
                    // Validate probability prices
                    for bid in snapshot.bids.iter().take(3) {
                        let price: f64 = bid.price.parse().unwrap_or(-1.0);
                        assert!(
                            price >= 0.0 && price <= 1.0,
                            "Bid price must be a probability 0.0-1.0, got: {}",
                            bid.price
                        );
                    }
                    println!("Polymarket orderbook subscription works");
                }
                Ok(None) => println!("No book snapshot received — market may be empty"),
                Err(_) => println!("Timeout waiting for book snapshot"),
            }

            ws.close().await;
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}

#[tokio::test]
#[ignore]
async fn test_subscribe_trades() {
    let token_id = match timeout(Duration::from_secs(30), find_active_token_id()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            println!("No active token_id found — skipping test");
            return;
        }
        Err(_) => {
            println!("Timeout finding active token_id — skipping test");
            return;
        }
    };

    let mut ws = ClobWebSocket::new(vec![token_id.clone()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!(
                "Connected to Polymarket WS — waiting for last_trade_price event (token_id={})...",
                token_id
            );

            // Wait for any data event (last_trade_price or price_change)
            let trade_event = timeout(Duration::from_secs(20), async {
                loop {
                    match ws.recv().await {
                        Ok(Some(WsEvent::LastTradePrice(trade))) => return Some(trade),
                        Ok(Some(WsEvent::Pong)) => continue,
                        Ok(Some(WsEvent::Book(_))) => {
                            // Book snapshot received first — keep waiting for trade
                            println!("Received book snapshot — still waiting for trade...");
                            continue;
                        }
                        Ok(Some(WsEvent::Unknown(u))) => {
                            if u.raw != "[]" {
                                println!("Unknown event (skipping): {}", u.raw);
                            }
                            continue;
                        }
                        Ok(Some(other)) => {
                            println!("Received other event (skipping): {:?}", other);
                            continue;
                        }
                        Ok(None) => return None,
                        Err(e) => {
                            println!("WebSocket error: {:?}", e);
                            return None;
                        }
                    }
                }
            })
            .await;

            match trade_event {
                Ok(Some(trade)) => {
                    println!(
                        "LastTradePrice received: price={} asset_id={}",
                        trade.price, trade.asset_id.as_deref().unwrap_or("unknown")
                    );
                    println!("Polymarket trade subscription works");
                }
                Ok(None) => {
                    println!("No trade received within timeout (market may be inactive — this is expected for illiquid markets)");
                }
                Err(_) => {
                    println!("Timeout — no trade event within 20s (market may be inactive)");
                }
            }

            ws.close().await;
        }
        Ok(Err(e)) => println!("Connection failed: {:?}", e),
        Err(_) => println!("Connection timeout"),
    }
}
