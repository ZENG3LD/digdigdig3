//! # Raydium Response Parser
//!
//! Parse JSON responses from Raydium API V3.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker,
};

/// Parser for Raydium API responses
pub struct RaydiumParser;

impl RaydiumParser {
    // HELPERS

    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    pub fn check_success(response: &Value) -> ExchangeResult<()> {
        let success = response.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            if let Some(error) = response.get("error") {
                let code = error.get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN");
                let message = error.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("{}: {}", code, message),
                });
            }

            return Err(ExchangeError::Parse("API returned success: false".to_string()));
        }

        Ok(())
    }

    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        Self::check_success(response)?;

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    // MARKET DATA

    pub fn parse_price(response: &Value, mint_address: &str) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;

        Self::get_f64(data, mint_address)
            .ok_or_else(|| ExchangeError::Parse(format!("Mint {} not found in price data", mint_address)))
    }

    pub fn parse_ticker(pool_data: &Value) -> ExchangeResult<Ticker> {
        let price = Self::require_f64(pool_data, "price")?;

        let day = pool_data.get("day");
        let volume_24h = day.and_then(|d| Self::get_f64(d, "volume"));
        let high_24h = day.and_then(|d| Self::get_f64(d, "priceMax"));
        let low_24h = day.and_then(|d| Self::get_f64(d, "priceMin"));

        Ok(Ticker {
            symbol: String::new(),
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: volume_24h,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn parse_orderbook(pool_data: &Value) -> ExchangeResult<OrderBook> {
        let price = Self::require_f64(pool_data, "price")?;
        let reserve_a = Self::require_f64(pool_data, "mintAmountA")?;

        // AMM synthetic orderbook
        let bids = vec![(price * 0.99, reserve_a * 0.1)];
        let asks = vec![(price * 1.01, reserve_a * 0.1)];

        Ok(OrderBook {
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
            sequence: None,
        })
    }

    pub fn parse_klines(_response: &Value) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::NotSupported("Raydium API does not provide kline data".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_success() {
        let success_response = json!({
            "success": true,
            "data": {}
        });
        assert!(RaydiumParser::check_success(&success_response).is_ok());

        let error_response = json!({
            "success": false,
            "error": {
                "code": "NOT_FOUND",
                "message": "Pool not found"
            }
        });
        assert!(RaydiumParser::check_success(&error_response).is_err());
    }

    #[test]
    fn test_parse_price() {
        let response = json!({
            "success": true,
            "data": {
                "So11111111111111111111111111111111111111112": 145.67
            }
        });

        let sol_price = RaydiumParser::parse_price(
            &response,
            "So11111111111111111111111111111111111111112"
        ).unwrap();

        assert_eq!(sol_price, 145.67);
    }
}
