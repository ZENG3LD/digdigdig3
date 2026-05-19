//! Tests for AlphaVantage connector
//!
//! These tests verify parsing with REAL API responses.

#[cfg(test)]
mod tests {
    use super::super::parser::AlphaVantageParser;
    use serde_json::json;

    #[test]
    fn test_parse_demo_key_error() {
        let response = json!({
            "Information": "The **demo** API key is for demo purposes only. Please claim your free API key at (https://www.alphavantage.co/support/#api-key) to explore our full API offerings. It takes fewer than 20 seconds."
        });

        let result = AlphaVantageParser::check_error(&response);
        assert!(result.is_err());
        if let Err(e) = result {
            println!("Demo key error: {:?}", e);
        }
    }

    #[test]
    fn test_parse_fx_intraday_5min() {
        // Real response from: curl "https://www.alphavantage.co/query?function=FX_INTRADAY&from_symbol=EUR&to_symbol=USD&interval=5min&apikey=demo"
        let response = json!({
            "Meta Data": {
                "1. Information": "FX Intraday (5min) Time Series",
                "2. From Symbol": "EUR",
                "3. To Symbol": "USD",
                "4. Last Refreshed": "2026-01-25 22:40:00",
                "5. Interval": "5min",
                "6. Output Size": "Compact",
                "7. Time Zone": "UTC"
            },
            "Time Series FX (5min)": {
                "2026-01-25 22:40:00": {
                    "1. open": "1.18660",
                    "2. high": "1.18680",
                    "3. low": "1.18640",
                    "4. close": "1.18680"
                },
                "2026-01-25 22:35:00": {
                    "1. open": "1.18600",
                    "2. high": "1.18660",
                    "3. low": "1.18600",
                    "4. close": "1.18650"
                }
            }
        });

        let result = AlphaVantageParser::parse_fx_intraday(&response, "5min");
        assert!(result.is_ok(), "Failed to parse FX_INTRADAY: {:?}", result.err());

        let klines = result.unwrap();
        assert_eq!(klines.len(), 2);

        // Should be sorted oldest first
        assert_eq!(klines[0].close, 1.18650);
        assert_eq!(klines[1].close, 1.18680);

        println!("✓ FX_INTRADAY (5min) works with demo key");
    }

    #[test]
    fn test_parse_fx_daily() {
        // Real response from: curl "https://www.alphavantage.co/query?function=FX_DAILY&from_symbol=EUR&to_symbol=USD&apikey=demo"
        let response = json!({
            "Meta Data": {
                "1. Information": "Forex Daily Prices (open, high, low, close)",
                "2. From Symbol": "EUR",
                "3. To Symbol": "USD",
                "4. Output Size": "Compact",
                "5. Last Refreshed": "2026-01-23",
                "6. Time Zone": "UTC"
            },
            "Time Series FX (Daily)": {
                "2026-01-23": {
                    "1. open": "1.17520",
                    "2. high": "1.18330",
                    "3. low": "1.17270",
                    "4. close": "1.18260"
                },
                "2026-01-22": {
                    "1. open": "1.16820",
                    "2. high": "1.17560",
                    "3. low": "1.16680",
                    "4. close": "1.17540"
                }
            }
        });

        let result = AlphaVantageParser::parse_fx_daily(&response);
        assert!(result.is_ok(), "Failed to parse FX_DAILY: {:?}", result.err());

        let klines = result.unwrap();
        assert_eq!(klines.len(), 2);

        // Check values
        assert_eq!(klines[1].close, 1.18260); // Most recent
        assert_eq!(klines[0].open, 1.16820);

        println!("✓ FX_DAILY works with demo key");
    }

    #[test]
    fn test_parse_fx_weekly() {
        // Real response from: curl "https://www.alphavantage.co/query?function=FX_WEEKLY&from_symbol=EUR&to_symbol=USD&apikey=demo"
        let response = json!({
            "Meta Data": {
                "1. Information": "Forex Weekly Prices (open, high, low, close)",
                "2. From Symbol": "EUR",
                "3. To Symbol": "USD",
                "4. Last Refreshed": "2026-01-23",
                "5. Time Zone": "UTC"
            },
            "Time Series FX (Weekly)": {
                "2026-01-23": {
                    "1. open": "1.15830",
                    "2. high": "1.18330",
                    "3. low": "1.15700",
                    "4. close": "1.18260"
                },
                "2026-01-16": {
                    "1. open": "1.16290",
                    "2. high": "1.16980",
                    "3. low": "1.15830",
                    "4. close": "1.15970"
                }
            }
        });

        let result = AlphaVantageParser::parse_fx_weekly(&response);
        assert!(result.is_ok(), "Failed to parse FX_WEEKLY: {:?}", result.err());

        let klines = result.unwrap();
        assert_eq!(klines.len(), 2);

        println!("✓ FX_WEEKLY works with demo key");
    }

    #[test]
    fn test_parse_exchange_rate_demo_key_fails() {
        // CURRENCY_EXCHANGE_RATE doesn't work with demo key
        let response = json!({
            "Information": "The **demo** API key is for demo purposes only. Please claim your free API key at (https://www.alphavantage.co/support/#api-key) to explore our full API offerings. It takes fewer than 20 seconds."
        });

        let result = AlphaVantageParser::check_error(&response);
        assert!(result.is_err(), "Demo key should fail for CURRENCY_EXCHANGE_RATE");

        println!("✓ CURRENCY_EXCHANGE_RATE correctly fails with demo key");
    }
}
