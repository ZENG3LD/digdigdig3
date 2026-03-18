//! # Crypto.com Response Parser
//!
//! JSON parsing for Crypto.com Exchange API v1 responses.
//!
//! ## Important Notes
//! - All numeric values in Crypto.com responses are STRINGS (e.g., "50000.00")
//! - Response format: { "code": 0, "result": { ... } }
//! - Success: code = 0, errors: code != 0
//! - REST and WebSocket use different formats for some messages

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, TradeSide, SymbolInfo,
    UserTrade,
};

/// Parser for Crypto.com API responses
pub struct CryptoComParser;

impl CryptoComParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract result from response
    pub fn extract_result(response: &Value) -> ExchangeResult<&Value> {
        response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))
    }

    /// Check response code (0 = success)
    pub fn check_response(response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);

        if code != 0 {
            let message = response.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: message.to_string(),
            });
        }

        Ok(())
    }

    /// Parse f64 from string or number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Get f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Get required f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Get string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Get required string
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
            .or_else(|| data.get(key).and_then(|v| v.as_i64()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price (ticker response)
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No ticker data".to_string()))?;

        Self::get_f64(data, "a") // "a" = last price
            .ok_or_else(|| ExchangeError::Parse("Missing last price".to_string()))
    }

    /// Parse klines
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of candlesticks".to_string()))?;

        let mut klines = Vec::with_capacity(data.len());

        for candle in data {
            let open_time = Self::get_i64(candle, "t").unwrap_or(0);
            let open = Self::get_f64(candle, "o").unwrap_or(0.0);
            let high = Self::get_f64(candle, "h").unwrap_or(0.0);
            let low = Self::get_f64(candle, "l").unwrap_or(0.0);
            let close = Self::get_f64(candle, "c").unwrap_or(0.0);
            let volume = Self::get_f64(candle, "v").unwrap_or(0.0);

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No orderbook data".to_string()))?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let timestamp = Self::get_i64(data, "t").unwrap_or(0);

        Ok(OrderBook {
            timestamp,
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
        })
    }

    /// Parse ticker
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No ticker data".to_string()))?;

        Ok(Ticker {
            symbol: Self::get_str(data, "i").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "a").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "b"),
            ask_price: Self::get_f64(data, "k"),
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: Self::get_f64(data, "vv"),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "c").map(|r| r * 100.0),
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
        })
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No valuation data".to_string()))?;

        Ok(FundingRate {
            symbol: Self::get_str(data, "instrument_name").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "funding_rate")?,
            next_funding_time: Self::get_i64(data, "next_funding_time"),
            timestamp: 0,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order from create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        Self::require_str(result, "order_id").map(String::from)
    }

    /// Parse order details
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        Self::parse_order_data(result)
    }

    /// Parse order from data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("BUY") {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("LIMIT") {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "client_oid").map(String::from),
            symbol: Self::get_str(data, "instrument_name").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "trigger_price"),
            quantity: Self::get_f64(data, "quantity").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "cumulative_quantity").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avg_price"),
            commission: None,
            commission_asset: Self::get_str(data, "fee_currency").map(String::from),
            created_at: Self::get_i64(data, "create_time").unwrap_or(0),
            updated_at: Self::get_i64(data, "update_time"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("ACTIVE") {
            "ACTIVE" => OrderStatus::New,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            "PENDING" => OrderStatus::New,
            _ => OrderStatus::New,
        }
    }

    /// Parse list of orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let order_list = result.get("order_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected order_list array".to_string()))?;

        order_list.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Parse user trades (fills) from `private/get-trades` response.
    ///
    /// Response format:
    /// ```json
    /// {"result":{"data":[{"trade_id":"123","order_id":"456","instrument_name":"BTC_USDT",
    ///   "side":"BUY","price":"50000","quantity":"0.001","fee":"0.01",
    ///   "fee_currency":"USDT","liquidity_indicator":"MAKER","create_time":1672531200000}]}}
    /// ```
    pub fn parse_user_trades(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;

        let data = result.get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected 'data' array in get-trades response".to_string()))?;

        let mut trades = Vec::with_capacity(data.len());

        for item in data {
            let side = match Self::get_str(item, "side").unwrap_or("BUY") {
                "SELL" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let is_maker = matches!(
                Self::get_str(item, "liquidity_indicator"),
                Some("MAKER")
            );

            trades.push(UserTrade {
                id: Self::get_str(item, "trade_id").unwrap_or("").to_string(),
                order_id: Self::get_str(item, "order_id").unwrap_or("").to_string(),
                symbol: Self::get_str(item, "instrument_name").unwrap_or("").to_string(),
                side,
                price: Self::get_f64(item, "price").unwrap_or(0.0),
                quantity: Self::get_f64(item, "quantity").unwrap_or(0.0),
                commission: Self::get_f64(item, "fee").unwrap_or(0.0),
                commission_asset: Self::get_str(item, "fee_currency").unwrap_or("").to_string(),
                is_maker,
                timestamp: Self::get_i64(item, "create_time").unwrap_or(0),
            });
        }

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balances from user-balance response
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let instruments = result.get("instrument_collateral_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected instrument_collateral_list".to_string()))?;

        let mut balances = Vec::new();

        for item in instruments {
            let asset = Self::get_str(item, "instrument_name").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "quantity").unwrap_or(0.0);
            let locked = Self::get_f64(item, "reserved_qty").unwrap_or(0.0);

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected positions array".to_string()))?;

        let mut positions = Vec::new();

        for item in data {
            if let Some(pos) = Self::parse_position_data(item) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    /// Parse single position
    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "instrument_name")?.to_string();
        let quantity = Self::get_f64(data, "quantity").unwrap_or(0.0);

        // Skip empty positions
        if quantity.abs() < f64::EPSILON {
            return None;
        }

        let side = if quantity > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Position {
            symbol,
            side,
            quantity: quantity.abs(),
            entry_price: Self::get_f64(data, "entry_price").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl: Self::get_f64(data, "open_position_pnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "session_pnl"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: None,
            margin: Self::get_f64(data, "initial_margin"),
            margin_type: if Self::get_str(data, "type") == Some("ISOLATED") {
                crate::core::MarginType::Isolated
            } else {
                crate::core::MarginType::Cross
            },
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: Self::get_str(data, "i").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "a").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "b"),
            ask_price: Self::get_f64(data, "k"),
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: Self::get_f64(data, "vv"),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "c").map(|r| r * 100.0),
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
        })
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "s").unwrap_or("BUY") {
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "d").unwrap_or("").to_string(),
            symbol: Self::get_str(data, "i").unwrap_or("").to_string(),
            price: Self::require_f64(data, "p")?,
            quantity: Self::get_f64(data, "q").unwrap_or(0.0),
            side,
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Crypto.com get-instruments response.
    ///
    /// Response format:
    /// ```json
    /// {"code":0,"result":{"data":[{"symbol":"BTC_USDT","inst_type":"CCY_PAIR","display_name":"BTC/USDT","base_ccy":"BTC","quote_ccy":"USDT","quote_decimals":2,"quantity_decimals":4,"price_tick_size":"0.01","qty_tick_size":"0.0001","max_leverage":"50","tradable":true,"expiry_timestamp_ms":0,"put_call":"NONE","strike_price":"0","underlying_symbol":""},...]}}
    /// ```
    /// Parse fee rate from private/get-fee-rate or private/get-instrument-fee-rate response
    pub fn parse_fee_rate(response: &Value) -> ExchangeResult<crate::core::FeeInfo> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;

        // get-fee-rate: result.maker_rate, result.taker_rate (strings, already in decimal form e.g. "0.001")
        // get-instrument-fee-rate: result.maker_rate, result.taker_rate per instrument
        let maker = result.get("maker_rate")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.001); // 0.1% default maker

        let taker = result.get("taker_rate")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.00075); // 0.075% default taker

        let symbol = result.get("instrument_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(crate::core::FeeInfo {
            maker_rate: maker,
            taker_rate: taker,
            symbol,
            tier: None,
        })
    }

    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))?;

        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in result".to_string()))?;

        let mut symbols = Vec::with_capacity(data.len());

        for item in data {
            // Only include tradable instruments
            let tradable = item.get("tradable").and_then(|v| v.as_bool()).unwrap_or(true);
            if !tradable {
                continue;
            }

            let symbol = match item.get("symbol").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let base_asset = item.get("base_ccy")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = item.get("quote_ccy")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if base_asset.is_empty() || quote_asset.is_empty() {
                continue;
            }

            let price_precision = item.get("quote_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u8;

            let quantity_precision = item.get("quantity_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(4) as u8;

            let step_size = item.get("qty_tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_quantity = step_size; // Minimum tradeable is typically 1 step

            let tick_size = item.get("price_tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity: None,
                tick_size,
                step_size,
                min_notional: None,
            });
        }

        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_response_success() {
        let response = json!({
            "code": 0,
            "result": {}
        });
        assert!(CryptoComParser::check_response(&response).is_ok());
    }

    #[test]
    fn test_check_response_error() {
        let response = json!({
            "code": 10003,
            "message": "INVALID_SIGNATURE"
        });
        assert!(CryptoComParser::check_response(&response).is_err());
    }

    #[test]
    fn test_parse_price() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "i": "BTCUSD-PERP",
                    "a": "50000.00"
                }]
            }
        });

        let price = CryptoComParser::parse_price(&response).unwrap();
        assert!((price - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "bids": [["50000.00", "1.5"], ["49999.00", "2.0"]],
                    "asks": [["50001.00", "1.0"], ["50002.00", "0.5"]],
                    "t": 1234567890
                }]
            }
        });

        let orderbook = CryptoComParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 50000.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1234567890);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "i": "BTCUSD-PERP",
                    "b": "50000.00",
                    "k": "50001.00",
                    "a": "50000.50",
                    "h": "51000.00",
                    "l": "49000.00",
                    "v": "1000.5",
                    "vv": "50000000",
                    "c": "0.02",
                    "t": 1234567890
                }]
            }
        });

        let ticker = CryptoComParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTCUSD-PERP");
        assert!((ticker.last_price - 50000.50).abs() < f64::EPSILON);
        assert_eq!(ticker.timestamp, 1234567890);
    }

    #[test]
    fn test_parse_order_status() {
        let data = json!({"status": "FILLED"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::Filled);

        let data = json!({"status": "ACTIVE"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::New);

        let data = json!({"status": "CANCELED"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::Canceled);
    }
}
