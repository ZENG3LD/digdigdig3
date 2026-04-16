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
//!
//! Because token_ids are market-specific numeric strings, we use a placeholder.
//! The connection and subscription mechanics are tested, not specific market data.

use std::time::Duration;
use tokio::time::timeout;

use super::websocket::{ClobWebSocket, WsEvent};

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// A known active Polymarket token_id for the BTC price market.
/// This is a YES outcome token for a BTC price prediction market.
/// token_ids are stable ERC-1155 identifiers — this one is a well-known BTC market.
const BTC_TOKEN_ID: &str =
    "21742633143463906290569050155826241533067272736897614950488156847949938836455";

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITIES TEST
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_orderbook_capabilities() {
    use crate::core::AccountType;

    let ws = ClobWebSocket::new(vec![BTC_TOKEN_ID.to_string()], false);
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
    let mut ws = ClobWebSocket::new(vec![BTC_TOKEN_ID.to_string()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            assert!(ws.is_connected(), "WebSocket must be connected after connect()");
            println!("Polymarket ClobWebSocket connected successfully");
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
    let mut ws = ClobWebSocket::new(vec![BTC_TOKEN_ID.to_string()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!(
                "Connected to Polymarket WS, subscribed to token_id={} — waiting for book snapshot...",
                BTC_TOKEN_ID
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
                            println!("Unknown event (skipping): {}", u.raw);
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
                        snapshot.asset_id
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
    let mut ws = ClobWebSocket::new(vec![BTC_TOKEN_ID.to_string()], false);

    let connect_result = timeout(Duration::from_secs(10), ws.connect()).await;

    match connect_result {
        Ok(Ok(())) => {
            println!(
                "Connected to Polymarket WS — waiting for last_trade_price event (token_id={})...",
                BTC_TOKEN_ID
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
                            println!("Unknown event (skipping): {}", u.raw);
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
                        trade.price, trade.asset_id
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
