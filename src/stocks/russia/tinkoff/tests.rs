//! Integration tests for Tinkoff Invest API connector
//!
//! ## Setup
//!
//! 1. Set environment variable: `TINKOFF_TOKEN=t.your_token_here`
//! 2. For sandbox testing: `TINKOFF_SANDBOX_TOKEN=t.your_sandbox_token`
//! 3. Run tests: `cargo test --package connectors-v5 --test tinkoff_integration`
//!
//! ## Token Requirements
//!
//! - Readonly token: Can run market data tests
//! - Full-access token: Can run trading tests
//! - Sandbox token: Can run all tests in sandbox environment
//!
//! ## Test Categories
//!
//! 1. **Market Data Tests** - Read-only, safe to run
//! 2. **Account Tests** - Read account info
//! 3. **Trading Tests** - Requires full-access token (use sandbox!)

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::types::*;
    use crate::core::traits::*;

    /// Helper: Create connector from environment
    fn create_connector() -> TinkoffConnector {
        TinkoffConnector::from_env()
    }

    /// Helper: Create sandbox connector
    fn create_sandbox_connector() -> TinkoffConnector {
        TinkoffConnector::from_env_sandbox()
    }

    /// Helper: Check if token is set
    fn has_token() -> bool {
        std::env::var("TINKOFF_TOKEN").is_ok()
    }

    /// Helper: Check if sandbox token is set
    fn has_sandbox_token() -> bool {
        std::env::var("TINKOFF_SANDBOX_TOKEN").is_ok()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA TESTS (Safe to run with readonly token)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore] // Remove #[ignore] to run this test
    async fn test_get_price() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let symbol = Symbol::new("SBER", "RUB"); // Sberbank

        let result = connector.get_price(symbol, AccountType::Spot).await;
        match result {
            Ok(price) => {
                println!("SBER price: {}", price);
                assert!(price > 0.0, "Price should be positive");
            }
            Err(e) => panic!("Failed to get price: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_ticker() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let symbol = Symbol::new("GAZP", "RUB"); // Gazprom

        let result = connector.get_ticker(symbol, AccountType::Spot).await;
        match result {
            Ok(ticker) => {
                println!("GAZP ticker: {:?}", ticker);
                assert!(ticker.last_price > 0.0);
                assert!(ticker.bid_price.is_some() || ticker.ask_price.is_some());
            }
            Err(e) => panic!("Failed to get ticker: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_orderbook() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let symbol = Symbol::new("SBER", "RUB");

        let result = connector.get_orderbook(symbol, Some(10), AccountType::Spot).await;
        match result {
            Ok(orderbook) => {
                println!("Orderbook depth: {} bids, {} asks", orderbook.bids.len(), orderbook.asks.len());
                assert!(!orderbook.bids.is_empty(), "Should have bids");
                assert!(!orderbook.asks.is_empty(), "Should have asks");

                // Check that bids are sorted descending
                for i in 1..orderbook.bids.len() {
                    assert!(orderbook.bids[i-1].0 >= orderbook.bids[i].0);
                }

                // Check that asks are sorted ascending
                for i in 1..orderbook.asks.len() {
                    assert!(orderbook.asks[i-1].0 <= orderbook.asks[i].0);
                }
            }
            Err(e) => panic!("Failed to get orderbook: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_klines() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let symbol = Symbol::new("SBER", "RUB");

        let result = connector.get_klines(symbol, "1h", Some(10), AccountType::Spot).await;
        match result {
            Ok(klines) => {
                println!("Retrieved {} candles", klines.len());
                assert!(!klines.is_empty(), "Should have candles");

                for kline in &klines {
                    assert!(kline.open > 0.0);
                    assert!(kline.high >= kline.open);
                    assert!(kline.low <= kline.open);
                    assert!(kline.close > 0.0);
                    assert!(kline.volume >= 0.0);
                }

                println!("Latest candle: O:{} H:{} L:{} C:{} V:{}",
                    klines.last().unwrap().open,
                    klines.last().unwrap().high,
                    klines.last().unwrap().low,
                    klines.last().unwrap().close,
                    klines.last().unwrap().volume,
                );
            }
            Err(e) => panic!("Failed to get klines: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_symbols() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();

        let result = connector.get_symbols().await;
        match result {
            Ok(symbols) => {
                println!("Retrieved {} symbols", symbols.len());
                assert!(!symbols.is_empty(), "Should have symbols");
                assert!(symbols.contains(&"SBER".to_string()), "Should include SBER");
                assert!(symbols.contains(&"GAZP".to_string()), "Should include GAZP");

                println!("First 10 symbols: {:?}", &symbols[..10.min(symbols.len())]);
            }
            Err(e) => panic!("Failed to get symbols: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_ping() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let result = connector.ping().await;
        assert!(result.is_ok(), "Ping should succeed");
        println!("Ping successful!");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCOUNT TESTS (Requires valid token)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore]
    async fn test_get_accounts() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let result = connector.get_accounts_list().await;
        match result {
            Ok(accounts) => {
                println!("Found {} accounts", accounts.len());
                assert!(!accounts.is_empty(), "Should have at least one account");
                println!("Account IDs: {:?}", accounts);
            }
            Err(e) => panic!("Failed to get accounts: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_initialize_account() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let mut connector = create_connector();
        let result = connector.initialize_account().await;
        match result {
            Ok(account_id) => {
                println!("Initialized with account: {}", account_id);
                assert!(!account_id.is_empty());
            }
            Err(e) => panic!("Failed to initialize account: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_balance() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let mut connector = create_connector();
        connector.initialize_account().await.expect("Failed to initialize account");

        let result = connector.get_balance(None, AccountType::Spot).await;
        match result {
            Ok(balances) => {
                println!("Found {} currency balances", balances.len());
                for balance in &balances {
                    println!("  {}: free={}, locked={}", balance.asset, balance.free, balance.locked);
                }
            }
            Err(e) => panic!("Failed to get balance: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_positions() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let mut connector = create_connector();
        connector.initialize_account().await.expect("Failed to initialize account");

        let result = connector.get_positions(None, AccountType::Spot).await;
        match result {
            Ok(positions) => {
                println!("Found {} positions", positions.len());
                for position in &positions {
                    println!("  {}: qty={}, entry={:?}, current={:?}, pnl={:?}",
                        position.symbol,
                        position.quantity,
                        position.entry_price,
                        position.current_price,
                        position.unrealized_pnl
                    );
                }
            }
            Err(e) => panic!("Failed to get positions: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_account_info() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let mut connector = create_connector();
        connector.initialize_account().await.expect("Failed to initialize account");

        let result = connector.get_account_info(AccountType::Spot).await;
        match result {
            Ok(info) => {
                println!("Account info: ID={}, type={}, balances={}",
                    info.account_id,
                    info.account_type,
                    info.balances.len()
                );
            }
            Err(e) => panic!("Failed to get account info: {:?}", e),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FIGI/INSTRUMENT LOOKUP TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore]
    async fn test_get_figi_by_ticker() {
        if !has_token() {
            println!("Skipping test: TINKOFF_TOKEN not set");
            return;
        }

        let connector = create_connector();
        let result = connector.get_figi_by_ticker("SBER").await;
        match result {
            Ok(figi) => {
                println!("SBER FIGI: {}", figi);
                assert!(!figi.is_empty());
                assert!(figi.len() == 12, "FIGI should be 12 characters");
            }
            Err(e) => panic!("Failed to get FIGI: {:?}", e),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING TESTS (Use sandbox only!)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    #[ignore]
    async fn test_sandbox_market_order() {
        if !has_sandbox_token() {
            println!("Skipping test: TINKOFF_SANDBOX_TOKEN not set");
            return;
        }

        let mut connector = create_sandbox_connector();

        // Initialize sandbox account
        connector.initialize_account().await.expect("Failed to initialize sandbox account");

        // Place market order
        let symbol = Symbol::new("SBER", "RUB");
        let result = connector.market_order(symbol, OrderSide::Buy, 1.0, AccountType::Spot).await;

        match result {
            Ok(order) => {
                println!("Market order placed: ID={}, status={:?}", order.id, order.status);
                assert!(!order.id.is_empty());
            }
            Err(e) => panic!("Failed to place market order: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_sandbox_limit_order() {
        if !has_sandbox_token() {
            println!("Skipping test: TINKOFF_SANDBOX_TOKEN not set");
            return;
        }

        let mut connector = create_sandbox_connector();
        connector.initialize_account().await.expect("Failed to initialize sandbox account");

        // Place limit order
        let symbol = Symbol::new("SBER", "RUB");
        let result = connector.limit_order(symbol, OrderSide::Buy, 1.0, 100.0, AccountType::Spot).await;

        match result {
            Ok(order) => {
                println!("Limit order placed: ID={}, price={:?}, status={:?}",
                    order.id, order.price, order.status);
                assert!(!order.id.is_empty());
                assert_eq!(order.price, Some(100.0));
            }
            Err(e) => panic!("Failed to place limit order: {:?}", e),
        }
    }
}
