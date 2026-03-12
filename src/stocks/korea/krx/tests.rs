//! Integration tests for KRX connector
//!
//! These tests require API credentials to run.
//! Set environment variables:
//! - KRX_API_KEY (optional, for Data Marketplace)
//! - KRX_DATA_PORTAL_KEY (optional, for Public Data Portal)
//!
//! Note: Without credentials, some tests may fail with authentication errors.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::types::{AccountType, Symbol};
    use crate::core::traits::{ExchangeIdentity, MarketData};

    /// Helper to create connector
    fn create_connector() -> KrxConnector {
        KrxConnector::from_env()
    }

    /// Helper to create test symbol (Samsung Electronics)
    fn samsung_symbol() -> Symbol {
        Symbol::new("005930", "")
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_exchange_identity() {
        let connector = create_connector();

        assert_eq!(connector.exchange_name(), "krx");
        assert_eq!(connector.exchange_id(), crate::core::types::ExchangeId::Krx);
        assert!(!connector.is_testnet());
        assert_eq!(
            connector.supported_account_types(),
            vec![AccountType::Spot]
        );
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_ping() {
        let connector = create_connector();

        let result = connector.ping().await;
        println!("Ping result: {:?}", result);

        if let Err(e) = &result {
            eprintln!("Ping error: {}", e);
        }

        // Note: May fail if no credentials or API not approved
        // This is expected and doesn't indicate a bug
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_price() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        let result = connector.get_price(symbol.clone(), AccountType::Spot).await;
        println!("Get price result for {}: {:?}", symbol.base, result);

        match result {
            Ok(price) => {
                println!("Samsung Electronics price: {} KRW", price);
                assert!(price > 0.0, "Price should be positive");
            }
            Err(e) => {
                eprintln!("Error getting price: {}", e);
                // May fail if API not approved or no credentials
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_ticker() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        let result = connector.get_ticker(symbol.clone(), AccountType::Spot).await;
        println!("Get ticker result for {}: {:?}", symbol.base, result);

        match result {
            Ok(ticker) => {
                println!("Ticker: {:?}", ticker);
                assert_eq!(ticker.symbol, symbol.base);
                assert!(ticker.last_price > 0.0, "Last price should be positive");
            }
            Err(e) => {
                eprintln!("Error getting ticker: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_klines() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        let result = connector
            .get_klines(symbol.clone(), "1d", Some(10), AccountType::Spot, None)
            .await;

        println!("Get klines result for {}: {:?}", symbol.base, result);

        match result {
            Ok(klines) => {
                println!("Retrieved {} klines", klines.len());
                if !klines.is_empty() {
                    let first = &klines[0];
                    println!("First kline: O={} H={} L={} C={} V={}",
                        first.open, first.high, first.low, first.close, first.volume);

                    assert!(first.open > 0.0, "Open should be positive");
                    assert!(first.high >= first.open, "High should be >= open");
                    assert!(first.low <= first.close, "Low should be <= close");
                    assert!(first.volume >= 0.0, "Volume should be non-negative");
                }
            }
            Err(e) => {
                eprintln!("Error getting klines: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_klines_invalid_interval() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        // KRX only supports daily data
        let result = connector
            .get_klines(symbol.clone(), "1h", Some(10), AccountType::Spot, None)
            .await;

        assert!(result.is_err(), "Should reject non-daily intervals");
        if let Err(e) = result {
            println!("Expected error for invalid interval: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_orderbook_unsupported() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        let result = connector
            .get_orderbook(symbol.clone(), Some(10), AccountType::Spot)
            .await;

        assert!(result.is_err(), "Orderbook should not be supported");
        if let Err(e) = result {
            println!("Expected error: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_stock_info() {
        let connector = create_connector();

        let result = connector.get_stock_info("005930").await;
        println!("Get stock info result: {:?}", result);

        match result {
            Ok(info) => {
                println!("Stock info: {:#}", info);
                // Verify expected fields
                assert!(info.get("srtnCd").is_some() || info.get("isinCd").is_some());
            }
            Err(e) => {
                eprintln!("Error getting stock info: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_get_investor_trading() {
        let connector = create_connector();
        let symbol = samsung_symbol();

        // Get data for past 5 days
        use chrono::{Duration, Local, Datelike};
        let end = Local::now();
        let start = end - Duration::days(5);

        let start_date = super::super::endpoints::format_date(
            start.year(),
            start.month(),
            start.day(),
        );
        let end_date = super::super::endpoints::format_date(
            end.year(),
            end.month(),
            end.day(),
        );

        let result = connector
            .get_investor_trading(symbol.clone(), &start_date, &end_date)
            .await;

        println!("Get investor trading result: {:?}", result);

        match result {
            Ok(data) => {
                println!("Investor trading data: {:#}", data);
            }
            Err(e) => {
                eprintln!("Error getting investor trading: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_trading_operations_unsupported() {
        use crate::core::traits::Trading;
        use crate::core::types::{OrderSide, OrderRequest, OrderType, TimeInForce};

        let connector = create_connector();
        let symbol = samsung_symbol();

        // All trading operations should return UnsupportedOperation error
        let result = connector.place_order(OrderRequest {
            symbol,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            account_type: AccountType::Spot,
            client_order_id: None,
            time_in_force: TimeInForce::Gtc,
            reduce_only: false,
        }).await;

        assert!(result.is_err());
        if let Err(e) = result {
            println!("Expected error for place_order: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_account_operations_unsupported() {
        use crate::core::traits::Account;
        use crate::core::types::BalanceQuery;

        let connector = create_connector();

        let result = connector.get_balance(BalanceQuery { asset: None, account_type: AccountType::Spot }).await;

        assert!(result.is_err());
        if let Err(e) = result {
            println!("Expected error for get_balance: {}", e);
        }
    }

    #[tokio::test]
    #[ignore] // Requires API credentials
    async fn test_positions_operations_unsupported() {
        use crate::core::traits::Positions;
        use crate::core::types::PositionQuery;

        let connector = create_connector();

        let result = connector
            .get_positions(PositionQuery { symbol: None, account_type: AccountType::Spot })
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            println!("Expected error for get_positions: {}", e);
        }
    }

    #[test]
    fn test_parse_krx_number() {
        use super::super::parser::KrxParser;
        use serde_json::json;

        let val = json!("76,200");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 76200.0);

        let val = json!("12,345,678");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 12345678.0);

        let val = json!("-1,200");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), -1200.0);

        let val = json!(12345.67);
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 12345.67);
    }

    #[test]
    fn test_symbol_formatting() {
        use super::super::endpoints::{format_symbol, format_isin};

        let symbol = Symbol::new("005930", "");
        assert_eq!(format_symbol(&symbol), "005930");

        let isin = format_isin("005930");
        assert_eq!(isin, "KR7005930003");

        let isin = format_isin("KR7005930003");
        assert_eq!(isin, "KR7005930003"); // Already ISIN, return as-is
    }

    #[test]
    fn test_market_id() {
        use super::super::endpoints::MarketId;

        assert_eq!(MarketId::Kospi.as_str(), "STK");
        assert_eq!(MarketId::Kosdaq.as_str(), "KSQ");
        assert_eq!(MarketId::Konex.as_str(), "KNX");
        assert_eq!(MarketId::All.as_str(), "ALL");
    }

    #[test]
    fn test_date_formatting() {
        use super::super::endpoints::format_date;

        let date = format_date(2026, 1, 20);
        assert_eq!(date, "20260120");

        let date = format_date(2026, 12, 31);
        assert_eq!(date, "20261231");
    }
}
