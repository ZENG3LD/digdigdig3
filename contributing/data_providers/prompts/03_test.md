# Phase 3: Test Agent Prompt - Data Providers

## Agent Type
`rust-implementer`

## Variables
- `{PROVIDER}` - Provider name in lowercase
- `{CATEGORY}` - Category (aggregators, forex, stocks, data_feeds)

---

## Mission

Create integration and WebSocket tests for {PROVIDER} connector.

**Key Differences from Exchange Tests:**
- ❌ NO trading tests
- ❌ NO account tests (unless broker)
- ✅ Focus on data retrieval
- ✅ Verify data quality (realistic prices, valid responses)
- ✅ Graceful handling of rate limits and errors

---

## Output

Create 1-2 test files in `tests/`:

```
tests/
├── {provider}_integration.rs    # REST API tests (ALWAYS)
└── {provider}_websocket.rs       # WebSocket tests (if WS available)
```

---

## File 1: {provider}_integration.rs

```rust
//! {PROVIDER} integration tests
//!
//! Tests REST API data retrieval.
//!
//! NOTE: These tests make REAL API calls.
//! - Rate limits apply (see tiers_and_limits.md)
//! - API key may be required (check PROVIDER_API_KEY env var)
//! - Free tier limits may cause occasional failures

#[cfg(test)]
mod tests {
    use digdigdig3::core::types::*;
    use digdigdig3::core::traits::*;
    use digdigdig3::{CATEGORY}::{PROVIDER}::*;

    /// Helper: Create test connector
    fn create_connector() -> ProviderNameConnector {
        // Try to load from env, fallback to no auth
        ProviderNameConnector::from_env()
    }

    /// Helper: Test symbol (adapt to provider type)
    fn test_symbol() -> Symbol {
        match "{CATEGORY}" {
            "stocks" => Symbol {
                base: "AAPL".to_string(),  // Apple stock
                quote: "USD".to_string(),
            },
            "forex" => Symbol {
                base: "EUR".to_string(),
                quote: "USD".to_string(),
            },
            "aggregators" | "data_feeds" => Symbol {
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
            },
            _ => Symbol {
                base: "BTC".to_string(),
                quote: "USD".to_string(),
            },
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // IDENTITY TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_exchange_identity() {
        let connector = create_connector();

        assert_eq!(connector.exchange_name(), "{PROVIDER}");
        println!("✓ Exchange name: {}", connector.exchange_name());

        assert!(!connector.is_testnet());
        println!("✓ Testnet: {}", connector.is_testnet());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_price() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_price(symbol.clone(), AccountType::Spot).await {
            Ok(price) => {
                println!("✓ Price for {}/{}: ${}", symbol.base, symbol.quote, price);

                // Validate price is realistic
                match "{CATEGORY}" {
                    "stocks" => {
                        assert!(price > 1.0 && price < 10000.0, "Stock price unrealistic: ${}", price);
                    }
                    "forex" => {
                        assert!(price > 0.01 && price < 1000.0, "FX rate unrealistic: {}", price);
                    }
                    _ => {
                        assert!(price > 0.0, "Price must be positive");
                    }
                }
            }
            Err(e) => {
                println!("⚠ Price test failed: {:?}", e);
                println!("  This may be due to:");
                println!("  - Missing API key (set PROVIDER_API_KEY env var)");
                println!("  - Rate limit (free tier exhausted)");
                println!("  - Network issue");
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_ticker(symbol.clone(), AccountType::Spot).await {
            Ok(ticker) => {
                println!("✓ Ticker for {}/{}:", symbol.base, symbol.quote);
                println!("  Last: ${}", ticker.last_price);
                println!("  Bid: {:?}", ticker.bid_price);
                println!("  Ask: {:?}", ticker.ask_price);
                println!("  Volume 24h: {:?}", ticker.volume_24h);
                println!("  Change 24h: {:?}%", ticker.price_change_percent_24h);

                assert!(ticker.last_price > 0.0);
                if let (Some(bid), Some(ask)) = (ticker.bid_price, ticker.ask_price) {
                    assert!(bid < ask, "Bid must be < Ask");
                }
            }
            Err(e) => {
                println!("⚠ Ticker test failed: {:?}", e);
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_klines() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_klines(
            symbol.clone(),
            "1h",  // 1-hour interval
            Some(10),  // Last 10 candles
            AccountType::Spot
        ).await {
            Ok(klines) => {
                println!("✓ Retrieved {} klines for {}/{}", klines.len(), symbol.base, symbol.quote);

                assert!(!klines.is_empty(), "Should have at least 1 kline");

                if let Some(first) = klines.first() {
                    println!("  First candle:");
                    println!("    Open: ${}", first.open);
                    println!("    High: ${}", first.high);
                    println!("    Low: ${}", first.low);
                    println!("    Close: ${}", first.close);
                    println!("    Volume: {}", first.volume);

                    // Validate OHLC relationships
                    assert!(first.high >= first.low, "High must be >= Low");
                    assert!(first.high >= first.open, "High must be >= Open");
                    assert!(first.high >= first.close, "High must be >= Close");
                    assert!(first.low <= first.open, "Low must be <= Open");
                    assert!(first.low <= first.close, "Low must be <= Close");
                }
            }
            Err(e) => {
                println!("⚠ Klines test failed: {:?}", e);
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let connector = create_connector();
        let symbol = test_symbol();

        match connector.get_orderbook(symbol.clone(), Some(10), AccountType::Spot).await {
            Ok(orderbook) => {
                println!("✓ Orderbook for {}/{}:", symbol.base, symbol.quote);
                println!("  Bids: {}", orderbook.bids.len());
                println!("  Asks: {}", orderbook.asks.len());

                if let (Some(best_bid), Some(best_ask)) = (
                    orderbook.bids.first(),
                    orderbook.asks.first()
                ) {
                    println!("  Best bid: ${} (size: {})", best_bid.0, best_bid.1);
                    println!("  Best ask: ${} (size: {})", best_ask.0, best_ask.1);
                    println!("  Spread: ${}", best_ask.0 - best_bid.0);

                    assert!(best_bid.0 < best_ask.0, "Bid must be < Ask");
                }
            }
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("⚠ Orderbook not supported: {}", msg);
                println!("✓ Test passed - operation correctly marked as unsupported");
            }
            Err(e) => {
                println!("⚠ Orderbook test failed: {:?}", e);
                println!("✓ Test completed (with error)");
            }
        }
    }

    #[tokio::test]
    async fn test_get_symbols() {
        let connector = create_connector();

        match connector.get_symbols(AccountType::Spot).await {
            Ok(symbols) => {
                println!("✓ Retrieved {} symbols", symbols.len());

                assert!(!symbols.is_empty(), "Should have at least 1 symbol");

                // Print first few
                for symbol in symbols.iter().take(5) {
                    println!("  - {}", symbol);
                }
            }
            Err(e) => {
                println!("⚠ Symbols test failed: {:?}", e);
                println!("✓ Test completed (with expected error)");
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_symbol() {
        let connector = create_connector();
        let invalid_symbol = Symbol {
            base: "INVALID".to_string(),
            quote: "SYMBOL".to_string(),
        };

        match connector.get_price(invalid_symbol, AccountType::Spot).await {
            Ok(_) => {
                println!("⚠ Unexpected success for invalid symbol");
            }
            Err(e) => {
                println!("✓ Correctly rejected invalid symbol");
                println!("  Error: {:?}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING TESTS (should all return UnsupportedOperation)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_trading_not_supported() {
        let connector = create_connector();

        let order = Order {
            symbol: test_symbol(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(100.0),
            ..Default::default()
        };

        match connector.place_order(order).await {
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("✓ Trading correctly marked as unsupported");
                println!("  Message: {}", msg);
            }
            Ok(_) => {
                panic!("Trading should not be supported for data providers!");
            }
            Err(e) => {
                println!("⚠ Unexpected error: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_account_not_supported() {
        let connector = create_connector();

        match connector.get_balance(AccountType::Spot).await {
            Err(ExchangeError::UnsupportedOperation(msg)) => {
                println!("✓ Account operations correctly marked as unsupported");
                println!("  Message: {}", msg);
            }
            Ok(_) => {
                // If this is a BROKER (Alpaca, OANDA), account may be supported
                println!("⚠ Account supported - this may be a broker API");
            }
            Err(e) => {
                println!("⚠ Unexpected error: {:?}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXTENDED DATA TESTS (provider-specific)
    // ═══════════════════════════════════════════════════════════════════════

    // Add tests for provider-specific endpoints from data_types.md:
    // - Liquidations (derivatives feeds)
    // - Fundamentals (stock providers)
    // - Macro data (economic feeds)
    // - On-chain data (crypto feeds)

    // Example:
    // #[tokio::test]
    // async fn test_get_liquidations() {
    //     let connector = create_connector();
    //     match connector.get_liquidations(test_symbol()).await {
    //         Ok(liquidations) => {
    //             println!("✓ Retrieved {} liquidations", liquidations.len());
    //         }
    //         Err(e) => {
    //             println!("⚠ Liquidations test failed: {:?}", e);
    //         }
    //     }
    // }
}
```

---

## File 2: {provider}_websocket.rs (if WS available)

**Skip if WebSocket not available** (check api_overview.md research).

```rust
//! {PROVIDER} WebSocket tests
//!
//! Tests WebSocket real-time data streaming.
//!
//! NOTE: These tests establish REAL WebSocket connections.
//! - Connection may timeout on slow networks
//! - Rate limits apply
//! - API key may be required

#[cfg(test)]
mod tests {
    use digdigdig3::core::types::*;
    use digdigdig3::core::traits::*;
    use digdigdig3::{CATEGORY}::{PROVIDER}::*;
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    /// Helper: Create test WebSocket
    fn create_websocket() -> ProviderNameWebSocket {
        let auth = ProviderNameAuth::from_env();
        ProviderNameWebSocket::new(auth)
    }

    /// Helper: Test symbol
    fn test_symbol() -> Symbol {
        // Same as integration tests
        Symbol {
            base: "AAPL".to_string(),
            quote: "USD".to_string(),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTION TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_websocket_connect() {
        let mut ws = create_websocket();

        match ws.connect().await {
            Ok(_) => {
                println!("✓ WebSocket connected successfully");
                assert!(ws.is_connected());

                // Clean disconnect
                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("  This may be due to:");
                println!("  - Network timeout");
                println!("  - Missing API key");
                println!("  - Provider WebSocket down");
                println!("✓ Test completed (connection issue expected in test env)");
            }
        }
    }

    #[tokio::test]
    async fn test_disconnect_without_connect() {
        let mut ws = create_websocket();

        match ws.disconnect().await {
            Ok(_) => {
                println!("✓ Disconnect without connect handled gracefully");
            }
            Err(e) => {
                println!("✓ Disconnect failed as expected: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_without_connect() {
        let mut ws = create_websocket();
        let symbol = test_symbol();

        let request = SubscriptionRequest {
            symbol,
            stream_type: StreamType::Ticker,
        };

        match ws.subscribe(request).await {
            Err(WebSocketError::NotConnected) => {
                println!("✓ Subscribe without connect correctly fails");
            }
            Ok(_) => {
                panic!("Subscribe should fail when not connected!");
            }
            Err(e) => {
                println!("✓ Subscribe failed: {:?}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SUBSCRIPTION TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_subscribe_ticker() {
        let mut ws = create_websocket();
        let symbol = test_symbol();

        match ws.connect().await {
            Ok(_) => {
                let request = SubscriptionRequest {
                    symbol: symbol.clone(),
                    stream_type: StreamType::Ticker,
                };

                match ws.subscribe(request).await {
                    Ok(_) => {
                        println!("✓ Subscribed to ticker for {}/{}", symbol.base, symbol.quote);
                    }
                    Err(e) => {
                        println!("⚠ Subscribe failed: {:?}", e);
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_orderbook() {
        let mut ws = create_websocket();
        let symbol = test_symbol();

        match ws.connect().await {
            Ok(_) => {
                let request = SubscriptionRequest {
                    symbol: symbol.clone(),
                    stream_type: StreamType::OrderBook,
                };

                match ws.subscribe(request).await {
                    Ok(_) => {
                        println!("✓ Subscribed to orderbook for {}/{}", symbol.base, symbol.quote);
                    }
                    Err(WebSocketError::UnsupportedOperation(msg)) => {
                        println!("✓ Orderbook stream not supported: {}", msg);
                    }
                    Err(e) => {
                        println!("⚠ Subscribe failed: {:?}", e);
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_trades() {
        let mut ws = create_websocket();
        let symbol = test_symbol();

        match ws.connect().await {
            Ok(_) => {
                let request = SubscriptionRequest {
                    symbol: symbol.clone(),
                    stream_type: StreamType::Trades,
                };

                match ws.subscribe(request).await {
                    Ok(_) => {
                        println!("✓ Subscribed to trades for {}/{}", symbol.base, symbol.quote);
                    }
                    Err(e) => {
                        println!("⚠ Subscribe failed: {:?}", e);
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EVENT STREAMING TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_receive_ticker_events() {
        let mut ws = create_websocket();
        let symbol = test_symbol();

        match ws.connect().await {
            Ok(_) => {
                let request = SubscriptionRequest {
                    symbol: symbol.clone(),
                    stream_type: StreamType::Ticker,
                };

                if ws.subscribe(request).await.is_ok() {
                    let mut stream = ws.event_stream();
                    let mut count = 0;

                    // Wait up to 10 seconds for events
                    let result = timeout(Duration::from_secs(10), async {
                        while let Some(event_result) = stream.next().await {
                            count += 1;
                            match event_result {
                                Ok(event) => {
                                    println!("=== Received event {} ===", count);
                                    println!("{:?}", event);
                                }
                                Err(e) => {
                                    println!("⚠ Event error: {:?}", e);
                                }
                            }

                            if count >= 3 {
                                break;
                            }
                        }
                    }).await;

                    match result {
                        Ok(_) => {
                            println!("✓ Received {} ticker events", count);
                        }
                        Err(_) => {
                            println!("⚠ Timeout waiting for events (may be normal)");
                        }
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }

    #[tokio::test]
    async fn test_connection_persistence() {
        println!("Testing WebSocket connection persistence over 30 seconds...");
        println!("This verifies ping/pong heartbeat works correctly.");

        let mut ws = create_websocket();

        match ws.connect().await {
            Ok(_) => {
                let symbol = test_symbol();
                let request = SubscriptionRequest {
                    symbol: symbol.clone(),
                    stream_type: StreamType::Ticker,
                };

                if ws.subscribe(request).await.is_ok() {
                    let mut stream = ws.event_stream();
                    let mut event_count = 0;

                    // Monitor for 30 seconds
                    let result = timeout(Duration::from_secs(30), async {
                        while let Some(event_result) = stream.next().await {
                            if event_result.is_ok() {
                                event_count += 1;
                            }
                        }
                    }).await;

                    match result {
                        Err(_) => {
                            // Timeout is expected - connection persisted
                            println!("✓ Connection persisted for 30 seconds");
                            println!("✓ Received {} events during test", event_count);
                            assert!(ws.is_connected(), "Connection should still be alive");
                        }
                        Ok(_) => {
                            println!("⚠ Connection dropped early ({} events)", event_count);
                        }
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_subscriptions() {
        let mut ws = create_websocket();

        match ws.connect().await {
            Ok(_) => {
                let symbol1 = Symbol { base: "AAPL".to_string(), quote: "USD".to_string() };
                let symbol2 = Symbol { base: "GOOGL".to_string(), quote: "USD".to_string() };

                let req1 = SubscriptionRequest {
                    symbol: symbol1,
                    stream_type: StreamType::Ticker,
                };
                let req2 = SubscriptionRequest {
                    symbol: symbol2,
                    stream_type: StreamType::Ticker,
                };

                match (ws.subscribe(req1).await, ws.subscribe(req2).await) {
                    (Ok(_), Ok(_)) => {
                        println!("✓ Successfully subscribed to multiple symbols");
                    }
                    _ => {
                        println!("⚠ Multiple subscriptions failed");
                    }
                }

                let _ = ws.disconnect().await;
            }
            Err(e) => {
                println!("⚠ Connection failed: {:?}", e);
                println!("✓ Test skipped (connection issue)");
            }
        }
    }
}
```

---

## Test Execution

```bash
# Run integration tests
cargo test --package digdigdig3 --test {provider}_integration -- --nocapture

# Run WebSocket tests
cargo test --package digdigdig3 --test {provider}_websocket -- --nocapture

# Run all provider tests
cargo test --package digdigdig3 {provider} -- --nocapture
```

---

## Success Criteria

- [ ] Integration test file created
- [ ] WebSocket test file created (if WS available)
- [ ] All tests compile
- [ ] Tests use graceful error handling (no panics on network errors)
- [ ] Tests verify data quality (realistic prices, valid OHLC)
- [ ] UnsupportedOperation tests pass for trading/account methods
- [ ] Clear console output explaining what's being tested

---

## Important Notes

1. **Network Dependency** - Tests may fail due to network/rate limits - this is OK
2. **Graceful Handling** - Use `match` instead of `assert!` for API calls
3. **Clear Output** - Print what's happening, including errors
4. **Data Validation** - Check prices are in realistic ranges
5. **Extended Tests** - Add tests for provider-specific endpoints

---

## Next Phase

After tests compile and pass basic checks:
→ Phase 4: Debug until all tests return real data
