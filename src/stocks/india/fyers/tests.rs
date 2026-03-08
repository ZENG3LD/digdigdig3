//! # Fyers Connector Tests
//!
//! Integration tests for Fyers API v3 connector.
//!
//! ## Setup
//!
//! Set environment variables:
//! ```bash
//! export FYERS_APP_ID="your_app_id"
//! export FYERS_APP_SECRET="your_app_secret"
//! export FYERS_ACCESS_TOKEN="your_access_token"
//! ```
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all tests
//! cargo test --package connectors-v5 --lib stocks::india::fyers::tests
//!
//! # Run specific test
//! cargo test --package connectors-v5 --lib stocks::india::fyers::tests::test_get_price
//!
//! # Run with output
//! cargo test --package connectors-v5 --lib stocks::india::fyers::tests -- --nocapture
//! ```

#[cfg(test)]
mod tests {
    use crate::core::{AccountType, OrderSide, Symbol};
    use crate::core::traits::{Account, ExchangeIdentity, MarketData, Positions, Trading};
    use crate::stocks::india::fyers::{FyersAuth, FyersConnector};

    /// Create test connector from environment variables
    fn create_connector() -> FyersConnector {
        let auth = FyersAuth::from_env();

        if !auth.has_token() {
            panic!(
                "FYERS_ACCESS_TOKEN not set. Please set environment variables:\n\
                 - FYERS_APP_ID\n\
                 - FYERS_APP_SECRET\n\
                 - FYERS_ACCESS_TOKEN"
            );
        }

        FyersConnector::new(auth).expect("Failed to create connector")
    }

    /// Test symbol (SBI - State Bank of India)
    fn test_symbol() -> Symbol {
        Symbol::new("SBIN", "NSE")
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_exchange_identity() {
        let connector = create_connector();

        assert_eq!(connector.exchange_id().as_str(), "fyers");
        assert!(!connector.is_testnet());
        assert!(connector
            .supported_account_types()
            .contains(&AccountType::Spot));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_ping() {
        let connector = create_connector();
        let result = connector.ping().await;

        assert!(result.is_ok(), "Ping failed: {:?}", result.err());
        println!("✓ Ping successful");
    }

    #[tokio::test]
    async fn test_get_price() {
        let connector = create_connector();
        let symbol = test_symbol();

        let result = connector.get_price(symbol, AccountType::Spot).await;

        assert!(result.is_ok(), "get_price failed: {:?}", result.err());

        let price = result.unwrap();
        println!("✓ Price for NSE:SBIN-EQ: {}", price);

        assert!(price > 0.0, "Price should be positive");
    }

    #[tokio::test]
    async fn test_get_ticker() {
        let connector = create_connector();
        let symbol = test_symbol();

        let result = connector.get_ticker(symbol, AccountType::Spot).await;

        assert!(result.is_ok(), "get_ticker failed: {:?}", result.err());

        let ticker = result.unwrap();
        println!("✓ Ticker for NSE:SBIN-EQ:");
        println!("  Last Price: {}", ticker.last_price);
        println!("  Open: {}", ticker.open);
        println!("  High: {}", ticker.high);
        println!("  Low: {}", ticker.low);
        println!("  Volume: {}", ticker.volume);

        assert!(ticker.last_price > 0.0);
        assert!(ticker.high >= ticker.low);
    }

    #[tokio::test]
    async fn test_get_orderbook() {
        let connector = create_connector();
        let symbol = test_symbol();

        let result = connector
            .get_orderbook(symbol, Some(5), AccountType::Spot)
            .await;

        assert!(result.is_ok(), "get_orderbook failed: {:?}", result.err());

        let orderbook = result.unwrap();
        println!("✓ Orderbook for NSE:SBIN-EQ:");
        println!("  Bids: {}", orderbook.bids.len());
        println!("  Asks: {}", orderbook.asks.len());

        if !orderbook.bids.is_empty() {
            println!("  Best Bid: {} @ {}", orderbook.bids[0].1, orderbook.bids[0].0);
        }
        if !orderbook.asks.is_empty() {
            println!("  Best Ask: {} @ {}", orderbook.asks[0].1, orderbook.asks[0].0);
        }

        assert!(
            !orderbook.bids.is_empty() || !orderbook.asks.is_empty(),
            "Orderbook should have bids or asks"
        );
    }

    #[tokio::test]
    async fn test_get_klines() {
        let connector = create_connector();
        let symbol = test_symbol();

        let result = connector
            .get_klines(symbol, "5m", Some(10), AccountType::Spot)
            .await;

        assert!(result.is_ok(), "get_klines failed: {:?}", result.err());

        let klines = result.unwrap();
        println!("✓ Klines for NSE:SBIN-EQ (5m): {} candles", klines.len());

        if !klines.is_empty() {
            let last = &klines[klines.len() - 1];
            println!(
                "  Latest: O:{} H:{} L:{} C:{} V:{}",
                last.open, last.high, last.low, last.close, last.volume
            );
        }

        assert!(!klines.is_empty(), "Should return some klines");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_balance() {
        let connector = create_connector();

        let result = connector.get_balance(None, AccountType::Spot).await;

        assert!(result.is_ok(), "get_balance failed: {:?}", result.err());

        let balances = result.unwrap();
        println!("✓ Balances:");

        for balance in &balances {
            println!(
                "  {}: Total={}, Free={}, Locked={}",
                balance.asset, balance.total, balance.free, balance.locked
            );
        }

        assert!(!balances.is_empty(), "Should have at least one balance");
    }

    #[tokio::test]
    async fn test_get_account_info() {
        let connector = create_connector();

        let result = connector.get_account_info(AccountType::Spot).await;

        assert!(
            result.is_ok(),
            "get_account_info failed: {:?}",
            result.err()
        );

        let account_info = result.unwrap();
        println!("✓ Account Info:");
        println!("  User ID: {}", account_info.user_id);
        if let Some(email) = &account_info.email {
            println!("  Email: {}", email);
        }
        println!("  Can Trade: {}", account_info.can_trade);

        assert!(!account_info.user_id.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_positions() {
        let connector = create_connector();

        let result = connector.get_positions(None, AccountType::Spot).await;

        assert!(result.is_ok(), "get_positions failed: {:?}", result.err());

        let positions = result.unwrap();
        println!("✓ Positions: {} found", positions.len());

        for pos in &positions {
            println!(
                "  {} {} @ {} (P&L: {})",
                pos.side, pos.symbol, pos.entry_price, pos.unrealized_pnl
            );
        }

        // Positions may be empty, that's OK
        println!("  (Note: Positions may be empty if no active trades)");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING TESTS (CAUTION: Real orders!)
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore] // Ignore by default - requires manual run with --ignored
    async fn test_limit_order_and_cancel() {
        let connector = create_connector();
        let symbol = test_symbol();

        // Get current price first
        let price = connector
            .get_price(symbol.clone(), AccountType::Spot)
            .await
            .expect("Failed to get price");

        println!("Current price: {}", price);

        // Place limit order well below market (won't fill)
        let limit_price = price * 0.8; // 20% below market
        let quantity = 1.0; // 1 share

        println!(
            "Placing limit BUY order: {} @ {} (won't fill)",
            quantity, limit_price
        );

        let order_result = connector
            .limit_order(
                symbol.clone(),
                OrderSide::Buy,
                quantity,
                limit_price,
                AccountType::Spot,
            )
            .await;

        assert!(
            order_result.is_ok(),
            "limit_order failed: {:?}",
            order_result.err()
        );

        let order = order_result.unwrap();
        println!("✓ Order placed: ID = {}", order.id);

        // Wait a moment
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Cancel the order
        println!("Canceling order {}", order.id);

        let cancel_result = connector
            .cancel_order(symbol.clone(), &order.id, AccountType::Spot)
            .await;

        assert!(
            cancel_result.is_ok(),
            "cancel_order failed: {:?}",
            cancel_result.err()
        );

        println!("✓ Order canceled successfully");
    }

    #[tokio::test]
    async fn test_get_open_orders() {
        let connector = create_connector();

        let result = connector.get_open_orders(None, AccountType::Spot).await;

        assert!(
            result.is_ok(),
            "get_open_orders failed: {:?}",
            result.err()
        );

        let orders = result.unwrap();
        println!("✓ Open Orders: {} found", orders.len());

        for order in &orders {
            println!(
                "  {} {} {} @ {:?} - Status: {:?}",
                order.id, order.side, order.symbol, order.price, order.status
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_holdings() {
        let connector = create_connector();

        let result = connector.get_holdings().await;

        assert!(result.is_ok(), "get_holdings failed: {:?}", result.err());

        let holdings = result.unwrap();
        println!("✓ Holdings response received");

        if let Some(holdings_array) = holdings.get("holdings").and_then(|v| v.as_array()) {
            println!("  Holdings count: {}", holdings_array.len());

            for holding in holdings_array.iter().take(3) {
                if let Some(symbol) = holding.get("symbol").and_then(|v| v.as_str()) {
                    println!("  - {}", symbol);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_tradebook() {
        let connector = create_connector();

        let result = connector.get_tradebook().await;

        assert!(result.is_ok(), "get_tradebook failed: {:?}", result.err());

        let tradebook = result.unwrap();
        println!("✓ Tradebook response received");

        if let Some(trades_array) = tradebook.get("tradeBook").and_then(|v| v.as_array()) {
            println!("  Trades count: {}", trades_array.len());
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTH TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_auth_url_generation() {
        let auth = FyersAuth::new("TEST_APP_ID", "TEST_SECRET");

        let url = auth.get_authorization_url("https://example.com/callback", Some("test_state"));

        assert!(url.contains("client_id=TEST_APP_ID"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
        assert!(url.contains("state=test_state"));

        println!("✓ Authorization URL: {}", url);
    }

    #[test]
    fn test_app_id_hash() {
        let auth = FyersAuth::new("ABC123", "SECRET123");
        let hash = auth.generate_app_id_hash();

        assert_eq!(hash.len(), 64); // SHA-256 = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        println!("✓ App ID Hash (SHA-256): {}", hash);
    }
}
