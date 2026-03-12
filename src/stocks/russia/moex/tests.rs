//! # MOEX Connector Tests
//!
//! Tests for MOEX ISS API connector.
//!
//! ## Running Tests
//! ```bash
//! # Public data tests (no auth required)
//! cargo test --package connectors-v5 moex::tests::test_get_price -- --nocapture
//!
//! # With authentication (set MOEX_USERNAME and MOEX_PASSWORD)
//! MOEX_USERNAME=your_username MOEX_PASSWORD=your_password cargo test --package connectors-v5 moex::tests
//! ```

use crate::core::traits::{ExchangeIdentity, MarketData};
use crate::core::types::{AccountType, Symbol};
use super::{MoexConnector, MoexAuth};

#[tokio::test]
async fn test_exchange_identity() {
    let connector = MoexConnector::new_public();

    assert_eq!(connector.exchange_name(), "MOEX");
    assert!(!connector.is_testnet());

    println!("Exchange: {}", connector.exchange_name());
    println!("Testnet: {}", connector.is_testnet());
}

#[tokio::test]
async fn test_ping() {
    let connector = MoexConnector::new_public();

    let result = connector.ping().await;
    assert!(result.is_ok(), "Ping failed: {:?}", result.err());

    println!("Ping successful");
}

#[tokio::test]
async fn test_get_symbols() {
    let connector = MoexConnector::new_public();

    let result = connector.get_symbols().await;
    assert!(result.is_ok(), "Failed to get symbols: {:?}", result.err());

    let symbols = result.unwrap();
    assert!(!symbols.is_empty(), "Symbols list is empty");

    println!("Found {} symbols", symbols.len());
    println!("First 10 symbols: {:?}", &symbols[..10.min(symbols.len())]);
}

#[tokio::test]
async fn test_get_price_sber() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("SBER", "RUB");

    let result = connector.get_price(symbol.clone(), AccountType::Spot).await;
    assert!(result.is_ok(), "Failed to get price for SBER: {:?}", result.err());

    let price = result.unwrap();
    println!("SBER price: {}", price);
    assert!(price > 0.0, "Price should be positive");
}

#[tokio::test]
async fn test_get_ticker_sber() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("SBER", "RUB");

    let result = connector.get_ticker(symbol.clone(), AccountType::Spot).await;
    assert!(result.is_ok(), "Failed to get ticker for SBER: {:?}", result.err());

    let ticker = result.unwrap();
    println!("SBER ticker:");
    println!("  Last price: {}", ticker.last_price);
    println!("  Bid: {:?}", ticker.bid_price);
    println!("  Ask: {:?}", ticker.ask_price);
    println!("  High 24h: {:?}", ticker.high_24h);
    println!("  Low 24h: {:?}", ticker.low_24h);
    println!("  Volume 24h: {:?}", ticker.volume_24h);
    println!("  Change: {:?}", ticker.price_change_24h);
    println!("  Change %: {:?}", ticker.price_change_percent_24h);

    assert!(ticker.last_price > 0.0);
}

#[tokio::test]
async fn test_get_klines_sber() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("SBER", "RUB");

    // Get 1-hour candles
    let result = connector.get_klines(
        symbol.clone(),
        "1h",
        Some(10),
        AccountType::Spot
    ).await;

    assert!(result.is_ok(), "Failed to get klines for SBER: {:?}", result.err());

    let klines = result.unwrap();
    println!("Got {} candles", klines.len());

    if !klines.is_empty() {
        let latest = &klines[0];
        println!("Latest candle:");
        println!("  Open: {}", latest.open);
        println!("  High: {}", latest.high);
        println!("  Low: {}", latest.low);
        println!("  Close: {}", latest.close);
        println!("  Volume: {}", latest.volume);

        assert!(latest.open > 0.0);
        assert!(latest.high >= latest.low);
    }
}

#[tokio::test]
async fn test_get_price_gazp() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("GAZP", "RUB");

    let result = connector.get_price(symbol.clone(), AccountType::Spot).await;
    assert!(result.is_ok(), "Failed to get price for GAZP: {:?}", result.err());

    let price = result.unwrap();
    println!("GAZP price: {}", price);
    assert!(price > 0.0);
}

#[tokio::test]
async fn test_get_engines() {
    let connector = MoexConnector::new_public();

    let result = connector.get_engines().await;
    assert!(result.is_ok(), "Failed to get engines: {:?}", result.err());

    let engines = result.unwrap();
    println!("Engines response: {}", serde_json::to_string_pretty(&engines).unwrap());
}

#[tokio::test]
async fn test_get_security_info_sber() {
    let connector = MoexConnector::new_public();

    let result = connector.get_security_info("SBER").await;
    assert!(result.is_ok(), "Failed to get security info: {:?}", result.err());

    let info = result.unwrap();
    println!("SBER security info:");
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}

#[tokio::test]
async fn test_authenticated_connector() {
    // This test requires MOEX_USERNAME and MOEX_PASSWORD environment variables
    let auth = MoexAuth::from_env();

    if !auth.is_authenticated() {
        println!("Skipping authenticated test - no credentials provided");
        println!("Set MOEX_USERNAME and MOEX_PASSWORD to run this test");
        return;
    }

    let connector = MoexConnector::new(auth);
    assert!(connector.has_realtime_access());

    // Try to get price with authentication
    let symbol = Symbol::new("SBER", "RUB");
    let result = connector.get_price(symbol, AccountType::Spot).await;
    assert!(result.is_ok(), "Failed to get price with auth: {:?}", result.err());

    println!("Authenticated request successful");
}

#[tokio::test]
async fn test_orderbook_requires_subscription() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("SBER", "RUB");

    // Orderbook requires paid subscription, so this may fail
    let result = connector.get_orderbook(symbol, Some(10), AccountType::Spot).await;

    // This might fail due to lack of subscription - that's expected
    if let Err(e) = result {
        println!("Orderbook access failed (expected for free tier): {:?}", e);
    } else {
        println!("Orderbook access succeeded!");
        let orderbook = result.unwrap();
        println!("Bids: {}, Asks: {}", orderbook.bids.len(), orderbook.asks.len());
    }
}

#[tokio::test]
async fn test_trading_not_supported() {
    let connector = MoexConnector::new_public();
    let symbol = Symbol::new("SBER", "RUB");

    use crate::core::traits::Trading;
    use crate::core::types::{OrderSide, Quantity};

    // Trading should return UnsupportedOperation error
    let result = connector.market_order(
        symbol,
        OrderSide::Buy,
        Quantity::Base(1.0),
        AccountType::Spot
    ).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        crate::core::types::ExchangeError::UnsupportedOperation(msg) => {
            println!("Expected error: {}", msg);
            assert!(msg.contains("data provider"));
        }
        e => panic!("Unexpected error type: {:?}", e),
    }
}
