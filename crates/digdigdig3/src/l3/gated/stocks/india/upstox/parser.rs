//! # Upstox Response Parser
//!
//! Парсинг JSON ответов от Upstox API.
//!
//! ## Response Format
//! All Upstox responses follow this structure:
//! ```json
//! {
//!   "status": "success" | "error",
//!   "data": {...},
//!   "errors": [...]
//! }
//! ```

use serde_json::Value;
use chrono::DateTime;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, TimeInForce,
    OrderResult,
};

/// Парсер ответов Upstox API
pub struct UpstoxParser;

impl UpstoxParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract data from Upstox response
    ///
    /// Checks status field and returns data on success, error on failure
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check status field
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "error" {
                // Parse error message from errors array
                if let Some(errors) = response.get("errors").and_then(|e| e.as_array()) {
                    if let Some(first_error) = errors.first() {
                        let code = first_error.get("errorCode")
                            .or_else(|| first_error.get("error_code"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("UNKNOWN");
                        let message = first_error.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        return Err(ExchangeError::Api {
                            code: -1,
                            message: format!("{}: {}", code, message),
                        });
                    }
                }
                return Err(ExchangeError::Api {
                    code: -1,
                    message: "API returned error status".to_string(),
                });
            }
        }

        // Extract data field
        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
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

    /// Parse timestamp from ISO 8601 or Unix millis
    fn parse_timestamp(value: &Value) -> Option<i64> {
        // Try as string (ISO 8601)
        if let Some(s) = value.as_str() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Some(dt.timestamp_millis());
            }
        }
        // Try as number (Unix millis)
        value.as_i64()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from LTP endpoint
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "status": "success",
    ///   "data": {
    ///     "NSE_EQ|INE669E01016": {
    ///       "instrument_token": "NSE_EQ|INE669E01016",
    ///       "last_price": 2750.50
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_price(response: &Value, instrument_key: &str) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;
        let instrument_data = data.get(instrument_key)
            .ok_or_else(|| ExchangeError::Parse(format!("Instrument {} not found", instrument_key)))?;
        Self::require_f64(instrument_data, "last_price")
    }

    /// Parse klines from historical candle endpoint
    ///
    /// Candle format: [timestamp, open, high, low, close, volume, oi]
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let candles = data.get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' array".to_string()))?;

        let mut klines = Vec::with_capacity(candles.len());

        for candle in candles {
            let arr = candle.as_array()
                .ok_or_else(|| ExchangeError::Parse("Candle is not an array".to_string()))?;

            if arr.len() < 6 {
                continue;
            }

            // Parse timestamp (ISO 8601 format)
            let open_time = Self::parse_timestamp(&arr[0]).unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&arr[1]).unwrap_or(0.0),
                high: Self::parse_f64(&arr[2]).unwrap_or(0.0),
                low: Self::parse_f64(&arr[3]).unwrap_or(0.0),
                close: Self::parse_f64(&arr[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&arr[5]).unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        // Upstox returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Parse orderbook from quotes endpoint
    pub fn parse_orderbook(response: &Value, instrument_key: &str) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;
        let quote = data.get(instrument_key)
            .ok_or_else(|| ExchangeError::Parse(format!("Instrument {} not found", instrument_key)))?;

        let depth = quote.get("depth")
            .ok_or_else(|| ExchangeError::Parse("Missing 'depth' field".to_string()))?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            depth.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let price = Self::get_f64(level, "price")?;
                            let quantity = Self::get_f64(level, "quantity")?;
                            Some(OrderBookLevel::new(price, quantity))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let timestamp = quote.get("timestamp")
            .and_then(Self::parse_timestamp)
            .or_else(|| quote.get("last_trade_time").and_then(|t| t.as_i64()))
            .unwrap_or(0);

        Ok(OrderBook {
            timestamp,
            bids: parse_levels("buy"),
            asks: parse_levels("sell"),
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Parse ticker from quotes endpoint
    pub fn parse_ticker(response: &Value, instrument_key: &str) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;
        let quote = data.get(instrument_key)
            .ok_or_else(|| ExchangeError::Parse(format!("Instrument {} not found", instrument_key)))?;

        let ohlc = quote.get("ohlc");
        let last_price = Self::get_f64(quote, "last_price").unwrap_or(0.0);

        let timestamp = quote.get("timestamp")
            .and_then(Self::parse_timestamp)
            .unwrap_or(0);

        Ok(Ticker {
            symbol: Self::get_str(quote, "instrument_token").unwrap_or("").to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: ohlc.and_then(|o| Self::get_f64(o, "high")),
            low_24h: ohlc.and_then(|o| Self::get_f64(o, "low")),
            volume_24h: Self::get_f64(quote, "volume"),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(quote, "net_change"),
            price_change_percent_24h: None,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order ID from order placement response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;
        Self::require_str(data, "order_id").map(String::from)
    }

    /// Parse order from order details response
    pub fn parse_order(order_data: &Value) -> ExchangeResult<Order> {
        let order_id = Self::require_str(order_data, "order_id")?.to_string();
        let symbol = Self::require_str(order_data, "instrument_token")?.to_string();

        let side = match Self::require_str(order_data, "transaction_type")? {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse("Invalid transaction_type".to_string())),
        };

        let order_type = match Self::require_str(order_data, "order_type")? {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            "SL" | "SL-M" => OrderType::StopMarket { stop_price: 0.0 },
            _ => OrderType::Market,
        };

        let status = match Self::require_str(order_data, "status")? {
            "complete" => OrderStatus::Filled,
            "rejected" => OrderStatus::Rejected,
            "cancelled" => OrderStatus::Canceled,
            "open" | "open pending" | "validation pending" => OrderStatus::Open,
            "trigger pending" => OrderStatus::New,
            _ => OrderStatus::New,
        };

        let quantity = Self::require_f64(order_data, "quantity")?;
        let filled = Self::get_f64(order_data, "filled_quantity").unwrap_or(0.0);
        let price = Self::get_f64(order_data, "price");
        let average_price = Self::get_f64(order_data, "average_price");

        let created_at = order_data.get("order_timestamp")
            .and_then(Self::parse_timestamp)
            .unwrap_or(0);

        Ok(Order {
            id: order_id,
            client_order_id: None,
            symbol,
            side,
            order_type,
            status,
            quantity,
            filled_quantity: filled,
            price,
            stop_price: None,
            average_price,
            commission: None,
            commission_asset: None,
            created_at,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }

    /// Parse orders from order book response
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;
        let orders = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        orders.iter()
            .map(Self::parse_order)
            .collect()
    }

    /// Parse batch order placement/cancellation response
    ///
    /// Upstox returns an array of individual results:
    /// `[{ "order_id": "...", "status": "success"|"error", "message": "..." }, ...]`
    pub fn parse_batch_order_results(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        // Batch response can be top-level array or wrapped in { "data": [...] }
        let items = if let Some(data) = response.get("data").and_then(|d| d.as_array()) {
            data
        } else if let Some(arr) = response.as_array() {
            arr
        } else {
            return Ok(vec![]);
        };

        let results = items.iter().map(|item| {
            let success = Self::get_str(item, "status")
                .map(|s| s == "success")
                .unwrap_or(false);

            let _order_id = Self::get_str(item, "order_id").map(String::from);
            let error_msg = if !success {
                Self::get_str(item, "message")
                    .or_else(|| Self::get_str(item, "error"))
                    .map(String::from)
            } else {
                None
            };

            OrderResult {
                order: None, // Batch responses don't include full order objects
                client_order_id: None,
                success,
                error: error_msg,
                error_code: None,
            }
        }).collect();

        Ok(results)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balance from funds endpoint
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;
        let mut balances = Vec::new();

        // Equity segment
        if let Some(equity) = data.get("equity") {
            if let Some(enabled) = equity.get("enabled").and_then(|e| e.as_bool()) {
                if enabled {
                    let available = Self::get_f64(equity, "available_margin").unwrap_or(0.0);
                    let used = Self::get_f64(equity, "used_margin").unwrap_or(0.0);

                    balances.push(Balance {
                        asset: "INR".to_string(),
                        free: available,
                        locked: used,
                        total: available + used,
                    });
                }
            }
        }

        // Commodity segment (separate before July 2025)
        if let Some(commodity) = data.get("commodity") {
            if let Some(enabled) = commodity.get("enabled").and_then(|e| e.as_bool()) {
                if enabled {
                    let available = Self::get_f64(commodity, "available_margin").unwrap_or(0.0);
                    let used = Self::get_f64(commodity, "used_margin").unwrap_or(0.0);

                    balances.push(Balance {
                        asset: "INR_COMMODITY".to_string(),
                        free: available,
                        locked: used,
                        total: available + used,
                    });
                }
            }
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions from positions endpoint
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let positions = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        Ok(positions.iter()
            .filter_map(|pos_data| Self::parse_position(pos_data).ok())
            .collect())
    }

    /// Parse single position
    fn parse_position(pos_data: &Value) -> ExchangeResult<Position> {
        let symbol = Self::require_str(pos_data, "instrument_token")?.to_string();
        let quantity = Self::require_f64(pos_data, "quantity")?;

        let side = if quantity > 0.0 {
            PositionSide::Long
        } else if quantity < 0.0 {
            PositionSide::Short
        } else {
            return Err(ExchangeError::Parse("Position quantity is zero".to_string()));
        };

        let entry_price = Self::get_f64(pos_data, "average_price").unwrap_or(0.0);
        let unrealized_pnl = Self::get_f64(pos_data, "unrealised").unwrap_or(0.0);

        Ok(Position {
            symbol,
            side,
            quantity: quantity.abs(),
            entry_price,
            mark_price: None,
            unrealized_pnl,
            realized_pnl: Self::get_f64(pos_data, "realised"),
            leverage: 1,
            liquidation_price: None,
            margin_type: crate::core::types::MarginType::Cross,
            margin: None,
            take_profit: None,
            stop_loss: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_data_success() {
        let response = json!({
            "status": "success",
            "data": {"key": "value"}
        });
        let data = UpstoxParser::extract_data(&response).unwrap();
        assert_eq!(data.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_extract_data_error() {
        let response = json!({
            "status": "error",
            "errors": [{
                "errorCode": "UDAPI1026",
                "message": "Instrument key required"
            }]
        });
        let result = UpstoxParser::extract_data(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_price() {
        let response = json!({
            "status": "success",
            "data": {
                "NSE_EQ|INE669E01016": {
                    "instrument_token": "NSE_EQ|INE669E01016",
                    "last_price": 2750.50
                }
            }
        });
        let price = UpstoxParser::parse_price(&response, "NSE_EQ|INE669E01016").unwrap();
        assert_eq!(price, 2750.50);
    }

    #[test]
    fn test_parse_order_id() {
        let response = json!({
            "status": "success",
            "data": {
                "order_id": "240126000123456"
            }
        });
        let order_id = UpstoxParser::parse_order_id(&response).unwrap();
        assert_eq!(order_id, "240126000123456");
    }
}
