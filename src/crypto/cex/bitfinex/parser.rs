//! # Bitfinex Response Parser
//!
//! Parsing JSON array-based responses from Bitfinex API v2.
//!
//! ## Array-Based Format
//!
//! Unlike most exchanges, Bitfinex returns arrays instead of objects:
//! - Ticker: `[BID, BID_SIZE, ASK, ASK_SIZE, ...]`
//! - Order: `[ID, GID, CID, SYMBOL, MTS_CREATE, ...]` (32 fields)
//! - Position: `[SYMBOL, STATUS, AMOUNT, BASE_PRICE, ...]` (18 fields)

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position, PublicTrade, TradeSide,
    OrderSide, OrderType, OrderStatus, PositionSide, SymbolInfo,
};

/// Parser for Bitfinex API v2 responses
pub struct BitfinexParser;

impl BitfinexParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(arr) = response.as_array() {
            if !arr.is_empty() {
                if let Some(first) = arr[0].as_str() {
                    if first == "error" {
                        let code = arr.get(1)
                            .and_then(|v| v.as_i64())
                            .unwrap_or(-1);
                        let message = arr.get(2)
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        return Err(ExchangeError::Api {
                            code: code as i32,
                            message,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Parse f64 from Value (handles both int and float)
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_f64()
            .or_else(|| value.as_i64().map(|i| i as f64))
    }

    /// Get f64 from array at index
    fn get_f64(arr: &[Value], index: usize) -> Option<f64> {
        arr.get(index).and_then(Self::parse_f64)
    }

    /// Require f64 from array at index
    fn require_f64(arr: &[Value], index: usize) -> ExchangeResult<f64> {
        Self::get_f64(arr, index)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing f64 at index {}", index)))
    }

    /// Get string from array at index
    fn get_str(arr: &[Value], index: usize) -> Option<&str> {
        arr.get(index).and_then(|v| v.as_str())
    }

    /// Require string from array at index
    fn require_str(arr: &[Value], index: usize) -> ExchangeResult<&str> {
        Self::get_str(arr, index)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing string at index {}", index)))
    }

    /// Get i64 from array at index
    fn get_i64(arr: &[Value], index: usize) -> Option<i64> {
        arr.get(index).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ticker (trading pair)
    ///
    /// Format: `[BID, BID_SIZE, ASK, ASK_SIZE, DAILY_CHANGE, DAILY_CHANGE_RELATIVE, LAST_PRICE, VOLUME, HIGH, LOW]`
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for ticker".to_string()))?;

        if arr.len() < 10 {
            return Err(ExchangeError::Parse(format!("Ticker array too short: {} fields", arr.len())));
        }

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: Self::get_f64(arr, 6).unwrap_or(0.0),      // [6] LAST_PRICE
            bid_price: Self::get_f64(arr, 0),                       // [0] BID
            ask_price: Self::get_f64(arr, 2),                       // [2] ASK
            high_24h: Self::get_f64(arr, 8),                        // [8] HIGH
            low_24h: Self::get_f64(arr, 9),                         // [9] LOW
            volume_24h: Self::get_f64(arr, 7),                      // [7] VOLUME
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(arr, 4),                // [4] DAILY_CHANGE
            price_change_percent_24h: Self::get_f64(arr, 5).map(|r| r * 100.0), // [5] DAILY_CHANGE_RELATIVE
            timestamp: 0, // Bitfinex ticker doesn't include timestamp
        })
    }

    /// Parse orderbook
    ///
    /// Format (P0-P4): `[[PRICE, COUNT, AMOUNT], ...]`
    /// - Positive AMOUNT = bid
    /// - Negative AMOUNT = ask
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for orderbook".to_string()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for item in arr {
            if let Some(level) = item.as_array() {
                if level.len() < 3 { continue; }

                let price = Self::get_f64(level, 0).unwrap_or(0.0);    // [0] PRICE
                let amount = Self::get_f64(level, 2).unwrap_or(0.0);   // [2] AMOUNT

                if amount > 0.0 {
                    // Positive = bid
                    bids.push((price, amount));
                } else if amount < 0.0 {
                    // Negative = ask
                    asks.push((price, amount.abs()));
                }
            }
        }

        Ok(OrderBook {
            timestamp: 0, // Bitfinex orderbook doesn't include timestamp
            bids,
            asks,
            sequence: None,
        })
    }

    /// Parse klines/candles
    ///
    /// Format: `[[MTS, OPEN, CLOSE, HIGH, LOW, VOLUME], ...]`
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for candles".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            if let Some(candle) = item.as_array() {
                if candle.len() < 6 { continue; }

                // Bitfinex format: [MTS, OPEN, CLOSE, HIGH, LOW, VOLUME]
                let open_time = Self::get_i64(candle, 0).unwrap_or(0);  // [0] MTS

                klines.push(Kline {
                    open_time,
                    open: Self::get_f64(candle, 1).unwrap_or(0.0),      // [1] OPEN
                    close: Self::get_f64(candle, 2).unwrap_or(0.0),     // [2] CLOSE
                    high: Self::get_f64(candle, 3).unwrap_or(0.0),      // [3] HIGH
                    low: Self::get_f64(candle, 4).unwrap_or(0.0),       // [4] LOW
                    volume: Self::get_f64(candle, 5).unwrap_or(0.0),    // [5] VOLUME
                    quote_volume: None,
                    close_time: None,
                    trades: None,
                });
            }
        }

        // Bitfinex returns newest first by default, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order from array (32 fields)
    ///
    /// Format: `[ID, GID, CID, SYMBOL, MTS_CREATE, MTS_UPDATE, AMOUNT, AMOUNT_ORIG, TYPE, ...]`
    pub fn parse_order(data: &[Value]) -> ExchangeResult<Order> {
        if data.len() < 32 {
            return Err(ExchangeError::Parse(format!("Order array too short: {} fields", data.len())));
        }

        let id = Self::get_i64(data, 0)
            .ok_or_else(|| ExchangeError::Parse("Missing order ID".to_string()))?
            .to_string();

        let symbol = Self::require_str(data, 3)?.to_string();        // [3] SYMBOL

        // [6] AMOUNT: positive=buy, negative=sell
        let amount = Self::get_f64(data, 6).unwrap_or(0.0);
        let side = if amount >= 0.0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        // [8] TYPE: "EXCHANGE LIMIT", "EXCHANGE MARKET", etc.
        let order_type_str = Self::get_str(data, 8).unwrap_or("");
        let order_type = if order_type_str.contains("MARKET") {
            OrderType::Market
        } else {
            OrderType::Limit { price: 0.0 }
        };

        // [13] ORDER_STATUS: "ACTIVE", "EXECUTED", "CANCELED", etc.
        let status_str = Self::get_str(data, 13).unwrap_or("ACTIVE");
        let status = Self::parse_order_status(status_str);

        let amount_orig = Self::get_f64(data, 7).unwrap_or(0.0).abs();  // [7] AMOUNT_ORIG
        let filled = (amount_orig - amount.abs()).max(0.0);

        Ok(Order {
            id,
            client_order_id: Self::get_i64(data, 2).map(|i| i.to_string()), // [2] CID
            symbol,
            side,
            order_type,
            status,
            price: Self::get_f64(data, 16),                          // [16] PRICE
            stop_price: None,
            quantity: amount_orig,
            filled_quantity: filled,
            average_price: Self::get_f64(data, 17),                  // [17] PRICE_AVG
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, 4).unwrap_or(0),         // [4] MTS_CREATE
            updated_at: Self::get_i64(data, 5),                      // [5] MTS_UPDATE
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status from string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "ACTIVE" => OrderStatus::New,
            "PARTIALLY FILLED" => OrderStatus::PartiallyFilled,
            "EXECUTED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "INSUFFICIENT BALANCE" => OrderStatus::Rejected,
            "INSUFFICIENT MARGIN" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        }
    }

    /// Parse submit order response
    ///
    /// Format: `[[MTS, TYPE, MESSAGE_ID, null, [ORDER_DATA], CODE, STATUS, TEXT]]`
    pub fn parse_submit_order(response: &Value) -> ExchangeResult<Order> {
        Self::check_error(response)?;

        let outer = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        if outer.is_empty() {
            return Err(ExchangeError::Parse("Empty response array".to_string()));
        }

        let notification = outer[0].as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected notification array".to_string()))?;

        // [4] contains the order array (32 fields)
        let order_data = notification.get(4)
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing order data".to_string()))?;

        Self::parse_order(order_data)
    }

    /// Parse orders array
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        let mut orders = Vec::new();

        for item in arr {
            if let Some(order_data) = item.as_array() {
                if let Ok(order) = Self::parse_order(order_data) {
                    orders.push(order);
                }
            }
        }

        Ok(orders)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse wallets
    ///
    /// Format: `[["exchange", "BTC", BALANCE, UNSETTLED, AVAILABLE, ...], ...]`
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of wallets".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            if let Some(wallet) = item.as_array() {
                if wallet.len() < 5 { continue; }

                let asset = Self::get_str(wallet, 1).unwrap_or("").to_string();  // [1] CURRENCY
                if asset.is_empty() { continue; }

                let total = Self::get_f64(wallet, 2).unwrap_or(0.0);             // [2] BALANCE
                let available = Self::get_f64(wallet, 4).unwrap_or(0.0);         // [4] AVAILABLE_BALANCE

                balances.push(Balance {
                    asset,
                    free: available,
                    locked: (total - available).max(0.0),
                    total,
                });
            }
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
    ///
    /// Format: `[[SYMBOL, STATUS, AMOUNT, BASE_PRICE, FUNDING, ...], ...]` (18 fields)
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        let mut positions = Vec::new();

        for item in arr {
            if let Some(pos_data) = item.as_array() {
                if let Some(pos) = Self::parse_position_data(pos_data) {
                    positions.push(pos);
                }
            }
        }

        Ok(positions)
    }

    /// Parse single position from array (18 fields)
    fn parse_position_data(data: &[Value]) -> Option<Position> {
        if data.len() < 18 {
            return None;
        }

        let symbol = Self::get_str(data, 0)?.to_string();           // [0] SYMBOL
        let amount = Self::get_f64(data, 2).unwrap_or(0.0);         // [2] AMOUNT

        // Skip closed positions
        if amount.abs() < f64::EPSILON {
            return None;
        }

        let side = if amount > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Position {
            symbol,
            side,
            quantity: amount.abs(),
            entry_price: Self::get_f64(data, 3).unwrap_or(0.0),     // [3] BASE_PRICE
            mark_price: None,
            unrealized_pnl: Self::get_f64(data, 6).unwrap_or(0.0),  // [6] PL
            realized_pnl: None,
            leverage: Self::get_f64(data, 9).map(|l| l as u32).unwrap_or(1), // [9] LEVERAGE
            liquidation_price: Self::get_f64(data, 8),              // [8] PRICE_LIQ
            margin: Self::get_f64(data, 15),                        // [15] COLLATERAL
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker
    /// Format: [BID, BID_SIZE, ASK, ASK_SIZE, DAILY_CHANGE, DAILY_CHANGE_RELATIVE, LAST_PRICE, VOLUME, HIGH, LOW]
    pub fn parse_ws_ticker(data: &[Value]) -> ExchangeResult<Ticker> {
        if data.len() < 10 {
            return Err(ExchangeError::Parse(format!("WS Ticker array too short: {} fields", data.len())));
        }

        Ok(Ticker {
            symbol: "".to_string(), // Symbol needs to be filled by caller
            last_price: Self::require_f64(data, 6)?,
            bid_price: Some(Self::require_f64(data, 0)?),
            ask_price: Some(Self::require_f64(data, 2)?),
            high_24h: Self::get_f64(data, 8),
            low_24h: Self::get_f64(data, 9),
            volume_24h: Self::get_f64(data, 7),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(data, 4),
            price_change_percent_24h: Self::get_f64(data, 5).map(|r| r * 100.0),
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    /// Parse WebSocket trade
    /// Format: [ID, MTS, AMOUNT, PRICE]
    pub fn parse_ws_trade(data: &[Value]) -> ExchangeResult<PublicTrade> {
        if data.len() < 4 {
            return Err(ExchangeError::Parse(format!("WS Trade array too short: {} fields", data.len())));
        }

        let amount = Self::require_f64(data, 2)?;
        let side = if amount > 0.0 {
            TradeSide::Buy
        } else {
            TradeSide::Sell
        };

        Ok(PublicTrade {
            id: Self::get_i64(data, 0).map(|id| id.to_string()).unwrap_or_default(),
            symbol: "".to_string(), // Symbol needs to be filled by caller
            price: Self::require_f64(data, 3)?,
            quantity: amount.abs(),
            side,
            timestamp: Self::get_i64(data, 1).unwrap_or(0),
        })
    }

    /// Parse WebSocket orderbook delta
    /// Format: [[PRICE, COUNT, AMOUNT], ...]
    pub fn parse_ws_orderbook_delta(data: &[Value]) -> ExchangeResult<crate::core::StreamEvent> {
        // For now, return a simple orderbook update
        // In a full implementation, this would handle incremental updates
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for entry in data {
            if let Some(level) = entry.as_array() {
                if level.len() >= 3 {
                    let price = Self::require_f64(level, 0)?;
                    let count = Self::get_i64(level, 1).unwrap_or(0);
                    let amount = Self::require_f64(level, 2)?;

                    // Count = 0 means remove this price level
                    if count > 0 {
                        if amount > 0.0 {
                            bids.push((price, amount));
                        } else {
                            asks.push((price, amount.abs()));
                        }
                    }
                }
            }
        }

        Ok(crate::core::StreamEvent::OrderbookDelta {
            bids,
            asks,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    /// Parse WebSocket kline (candle)
    /// Format: [MTS, OPEN, CLOSE, HIGH, LOW, VOLUME]
    pub fn parse_ws_kline(data: &[Value]) -> ExchangeResult<Kline> {
        if data.len() < 6 {
            return Err(ExchangeError::Parse(format!("WS Kline array too short: {} fields", data.len())));
        }

        Ok(Kline {
            open_time: Self::get_i64(data, 0).unwrap_or(0),
            open: Self::require_f64(data, 1)?,
            high: Self::require_f64(data, 3)?,
            low: Self::require_f64(data, 4)?,
            close: Self::require_f64(data, 2)?,
            volume: Self::require_f64(data, 5)?,
            close_time: Some(Self::get_i64(data, 0).unwrap_or(0)), // Same as open_time for Bitfinex
            quote_volume: None,
            trades: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Bitfinex v1/symbols_details response.
    ///
    /// Response format (array of objects):
    /// ```json
    /// [{"pair":"btcusd","price_precision":5,"initial_margin":"30.0","minimum_margin":"15.0","maximum_order_size":"2000.0","minimum_order_size":"0.00006","expiration":"NA","margin":true},...]
    /// ```
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let items = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        let mut symbols = Vec::with_capacity(items.len());

        for item in items {
            let pair = match item.get("pair").and_then(|v| v.as_str()) {
                Some(p) => p,
                None => continue,
            };

            // Bitfinex pairs are lowercase: "btcusd" -> base="BTC", quote="USD"
            // Most pairs are 6 chars with 3+3 split, but some longer ones exist
            let (base_asset, quote_asset) = Self::split_pair(pair);
            if base_asset.is_empty() || quote_asset.is_empty() {
                continue;
            }

            let price_precision = item.get("price_precision")
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as u8;

            let min_quantity = item.get("minimum_order_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let max_quantity = item.get("maximum_order_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // Use symbol format that Bitfinex API uses (e.g. "tBTCUSD")
            let symbol = format!("t{}", pair.to_uppercase());

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision: 8,
                min_quantity,
                max_quantity,
                tick_size: None,
                step_size: None,
                min_notional: None,
            });
        }

        Ok(symbols)
    }

    /// Split a Bitfinex pair like "btcusd" into ("BTC", "USD").
    /// Bitfinex uses 3-char base + quote for most symbols, but longer ones exist.
    fn split_pair(pair: &str) -> (String, String) {
        let pair_upper = pair.to_uppercase();
        // Known quote currencies sorted by length descending to avoid ambiguity
        const KNOWN_QUOTES: &[&str] = &["USDT", "USDC", "TUSD", "BUSD", "CNHT", "XAUT", "USD", "EUR", "GBP", "JPY", "BTC", "ETH"];
        for quote in KNOWN_QUOTES {
            if pair_upper.ends_with(quote) {
                let base = &pair_upper[..pair_upper.len() - quote.len()];
                if !base.is_empty() {
                    return (base.to_string(), quote.to_string());
                }
            }
        }
        // Fallback: 3+rest split
        if pair_upper.len() >= 6 {
            let base = pair_upper[..3].to_string();
            let quote = pair_upper[3..].to_string();
            return (base, quote);
        }
        (String::new(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let response = json!([
            10645.0,      // BID
            73.93854271,  // BID_SIZE
            10647.0,      // ASK
            75.22266119,  // ASK_SIZE
            731.60645389, // DAILY_CHANGE
            0.0738,       // DAILY_CHANGE_RELATIVE
            10644.00645389, // LAST_PRICE
            14480.89849423, // VOLUME
            10766.0,      // HIGH
            9889.1449809  // LOW
        ]);

        let ticker = BitfinexParser::parse_ticker(&response, "tBTCUSD").unwrap();

        assert_eq!(ticker.symbol, "tBTCUSD");
        assert!((ticker.last_price - 10644.00645389).abs() < f64::EPSILON);
        assert!((ticker.bid_price.unwrap() - 10645.0).abs() < f64::EPSILON);
        assert!((ticker.ask_price.unwrap() - 10647.0).abs() < f64::EPSILON);
        assert!(ticker.bid_price.unwrap() < ticker.ask_price.unwrap());
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!([
            [8744.9, 2, 0.45603413],    // bid
            [8744.8, 1, -0.25],         // ask (negative)
            [8744.7, 3, 1.5],           // bid
            [8745.0, 1, -0.75]          // ask
        ]);

        let orderbook = BitfinexParser::parse_orderbook(&response).unwrap();

        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 8744.9).abs() < f64::EPSILON);
        assert!((orderbook.asks[0].1 - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_klines() {
        let response = json!([
            [1678465320000i64, 20097.0, 20114.0, 20125.0, 20094.0, 1.43504645],
            [1678465260000i64, 20100.0, 20097.0, 20105.0, 20090.0, 0.95234123]
        ]);

        let klines = BitfinexParser::parse_klines(&response).unwrap();

        assert_eq!(klines.len(), 2);
        // Should be reversed (oldest first)
        assert_eq!(klines[0].open_time, 1678465260000i64);
        assert!((klines[0].open - 20100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_error() {
        let error_response = json!(["error", 10020, "symbol: invalid"]);
        let result = BitfinexParser::check_error(&error_response);

        assert!(result.is_err());
        if let Err(ExchangeError::Api { code, message }) = result {
            assert_eq!(code, 10020);
            assert_eq!(message, "symbol: invalid");
        } else {
            panic!("Expected Api error");
        }
    }
}
