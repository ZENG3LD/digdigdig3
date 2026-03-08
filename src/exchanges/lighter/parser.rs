//! # Lighter Response Parser
//!
//! Parse JSON responses from Lighter API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, PublicTrade, FundingRate,
};

/// Parser for Lighter API responses
pub struct LighterParser;

impl LighterParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response indicates success (code 200)
    pub fn check_success(response: &Value) -> ExchangeResult<()> {
        if let Some(code) = response.get("code").and_then(|c| c.as_i64()) {
            if code != 200 {
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: message.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Parse f64 from string or number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Parse f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Parse required f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Parse required string
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Parse integer from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    /// Parse required integer
    fn require_i64(data: &Value, key: &str) -> ExchangeResult<i64> {
        Self::get_i64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from orderBookDetails response
    ///
    /// Returns last_trade_price from the first market in the response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_success(response)?;

        // Try order_book_details array first (perpetuals)
        if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            if let Some(first) = details.first() {
                if let Some(price) = Self::get_f64(first, "last_trade_price") {
                    return Ok(price);
                }
            }
        }

        // Try spot_order_book_details array (spot markets)
        if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            if let Some(first) = details.first() {
                if let Some(price) = Self::get_f64(first, "last_trade_price") {
                    return Ok(price);
                }
            }
        }

        Err(ExchangeError::Parse("No price data found".to_string()))
    }

    /// Parse ticker from orderBookDetails response
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_success(response)?;

        // Try order_book_details array first (perpetuals)
        let data = if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            details.first()
                .ok_or_else(|| ExchangeError::Parse("Empty order_book_details".to_string()))?
        } else if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            details.first()
                .ok_or_else(|| ExchangeError::Parse("Empty spot_order_book_details".to_string()))?
        } else {
            return Err(ExchangeError::Parse("No ticker data found".to_string()));
        };

        let symbol = Self::require_str(data, "symbol")?;
        let last_price = Self::require_f64(data, "last_trade_price")?;

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: Self::get_f64(data, "daily_price_high"),
            low_24h: Self::get_f64(data, "daily_price_low"),
            volume_24h: Self::get_f64(data, "daily_base_token_volume"),
            quote_volume_24h: Self::get_f64(data, "daily_quote_token_volume"),
            price_change_24h: Self::get_f64(data, "daily_price_change"),
            price_change_percent_24h: data.get("daily_price_change")
                .and_then(Self::parse_f64)
                .and_then(|change| {
                    // Calculate percentage: (change / (last_price - change)) * 100
                    let prev_price = last_price - change;
                    if prev_price != 0.0 {
                        Some((change / prev_price) * 100.0)
                    } else {
                        None
                    }
                }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Parse orderbook from orderBookOrders response
    ///
    /// Note: Lighter doesn't have a dedicated orderbook snapshot endpoint.
    /// This is a placeholder that would need the actual orderBookOrders data.
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_success(response)?;

        // Lighter's orderBookOrders returns full order list, not aggregated levels
        // For now, return empty orderbook - this would need proper aggregation
        Ok(OrderBook {
            timestamp: chrono::Utc::now().timestamp_millis(),
            bids: Vec::new(),
            asks: Vec::new(),
            sequence: None,
        })
    }

    /// Parse klines/candlesticks
    ///
    /// Handles two response formats:
    /// - New `/api/v1/candles` format: `"c"` array with abbreviated field names and ms timestamps
    /// - Legacy format: `"candlesticks"` array with full field names and second timestamps
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_success(response)?;

        // Try new /api/v1/candles format first ("c" array, short field names, ms timestamps)
        if let Some(candles) = response.get("c").and_then(|v| v.as_array()) {
            let mut klines = Vec::with_capacity(candles.len());
            for candle in candles {
                klines.push(Kline {
                    open_time: candle.get("t").and_then(|v| v.as_i64()).unwrap_or(0),
                    open: candle.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    high: candle.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    low: candle.get("l").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    close: candle.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    volume: candle.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    quote_volume: candle.get("V").and_then(|v| v.as_f64()),
                    close_time: None,
                    trades: None,
                });
            }
            return Ok(klines);
        }

        // Fall back to legacy "candlesticks" format (full field names, second timestamps)
        let candlesticks = response.get("candlesticks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candlesticks array".to_string()))?;

        let mut klines = Vec::with_capacity(candlesticks.len());

        for candle in candlesticks {
            let timestamp = Self::require_i64(candle, "timestamp")?;
            let open = Self::require_f64(candle, "open")?;
            let high = Self::require_f64(candle, "high")?;
            let low = Self::require_f64(candle, "low")?;
            let close = Self::require_f64(candle, "close")?;
            let volume = Self::require_f64(candle, "volume")?;
            let quote_volume = Self::get_f64(candle, "quote_volume");

            klines.push(Kline {
                open_time: timestamp * 1000, // seconds to milliseconds
                open,
                high,
                low,
                close,
                volume,
                quote_volume,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse recent trades
    pub fn parse_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        Self::check_success(response)?;

        let trades = response.get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing trades array".to_string()))?;

        let mut result = Vec::with_capacity(trades.len());

        for trade in trades {
            let id = Self::require_i64(trade, "trade_id")?;
            let price = Self::require_f64(trade, "price")?;
            let qty = Self::require_f64(trade, "size")?;
            let time = Self::require_i64(trade, "timestamp")?;
            let is_maker_ask = trade.get("is_maker_ask")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            result.push(PublicTrade {
                id: id.to_string(),
                symbol: String::new(), // Will be set by caller
                price,
                quantity: qty,
                side: if is_maker_ask {
                    crate::core::types::TradeSide::Sell
                } else {
                    crate::core::types::TradeSide::Buy
                },
                timestamp: time * 1000, // seconds to milliseconds
            });
        }

        Ok(result)
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_success(response)?;

        let fundings = response.get("fundings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing fundings array".to_string()))?;

        let first = fundings.first()
            .ok_or_else(|| ExchangeError::Parse("Empty fundings array".to_string()))?;

        let funding_rate = Self::require_f64(first, "funding_rate")?;
        let timestamp = Self::require_i64(first, "timestamp")?;

        Ok(FundingRate {
            symbol: String::new(), // Symbol not in response, caller must set
            rate: funding_rate,
            next_funding_time: None,
            timestamp: timestamp * 1000, // seconds to milliseconds
        })
    }

    /// Parse trading pairs from orderBooks or orderBookDetails
    pub fn parse_trading_pairs(response: &Value) -> ExchangeResult<Vec<String>> {
        Self::check_success(response)?;

        let mut symbols = Vec::new();

        // Parse from order_books array
        if let Some(order_books) = response.get("order_books").and_then(|v| v.as_array()) {
            for book in order_books {
                if let Some(symbol) = Self::get_str(book, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        // Parse from order_book_details array (perpetuals)
        if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            for detail in details {
                if let Some(symbol) = Self::get_str(detail, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        // Parse from spot_order_book_details array (spot)
        if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            for detail in details {
                if let Some(symbol) = Self::get_str(detail, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        if symbols.is_empty() {
            return Err(ExchangeError::Parse("No trading pairs found".to_string()));
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT DATA (Placeholders for Phase 2)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse account balance (placeholder)
    pub fn parse_balance(_response: &Value) -> ExchangeResult<Vec<crate::core::types::Balance>> {
        Err(ExchangeError::Parse("Balance parsing not yet implemented".to_string()))
    }

    /// Parse positions (placeholder)
    pub fn parse_positions(_response: &Value) -> ExchangeResult<Vec<crate::core::types::Position>> {
        Err(ExchangeError::Parse("Position parsing not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_success() {
        let success = json!({"code": 200, "message": "success"});
        assert!(LighterParser::check_success(&success).is_ok());

        let error = json!({"code": 400, "message": "Bad Request"});
        assert!(LighterParser::check_success(&error).is_err());
    }

    #[test]
    fn test_parse_klines_new_format() {
        // Actual /api/v1/candles response format
        let response = json!({
            "code": 200,
            "r": "1h",
            "c": [
                {
                    "t": 1740801600000i64,
                    "o": 85333.1,
                    "h": 86558.4,
                    "l": 85327.1,
                    "c": 86221.8,
                    "v": 17.97121,
                    "V": 1542622.63271,
                    "i": 3483696
                }
            ]
        });

        let klines = LighterParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1740801600000i64);
        assert_eq!(klines[0].open, 85333.1);
        assert_eq!(klines[0].high, 86558.4);
        assert_eq!(klines[0].low, 85327.1);
        assert_eq!(klines[0].close, 86221.8);
        assert_eq!(klines[0].volume, 17.97121);
        assert_eq!(klines[0].quote_volume, Some(1542622.63271));
    }

    #[test]
    fn test_parse_klines_legacy_format() {
        // Legacy "candlesticks" format with string values and second timestamps
        let response = json!({
            "code": 200,
            "message": "success",
            "candlesticks": [
                {
                    "timestamp": 1640995200,
                    "open": "3020.00",
                    "high": "3030.00",
                    "low": "3015.00",
                    "close": "3024.66",
                    "volume": "235.25",
                    "quote_volume": "93566.25"
                }
            ]
        });

        let klines = LighterParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1640995200 * 1000);
        assert_eq!(klines[0].open, 3020.0);
        assert_eq!(klines[0].close, 3024.66);
        assert_eq!(klines[0].quote_volume, Some(93566.25));
    }

    #[test]
    fn test_parse_trades() {
        let response = json!({
            "code": 200,
            "message": "success",
            "trades": [
                {
                    "trade_id": 12345,
                    "price": "3024.66",
                    "size": "1.5",
                    "timestamp": 1640995200,
                    "is_maker_ask": true
                }
            ]
        });

        let trades = LighterParser::parse_trades(&response).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 3024.66);
        assert_eq!(trades[0].quantity, 1.5);
    }
}
