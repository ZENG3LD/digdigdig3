//! # Dhan Response Parser
//!
//! Parsing JSON responses from Dhan API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, AccountInfo,
};

/// Parser for Dhan API responses
pub struct DhanParser;

impl DhanParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

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
    fn _get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse().ok()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check for error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_type) = response.get("errorType").and_then(|v| v.as_str()) {
            let error_code_str = Self::get_str(response, "errorCode").unwrap_or("");
            let error_message = Self::get_str(response, "errorMessage").unwrap_or("Unknown error");

            // Try to parse error code as i32, default to 0
            let error_code = error_code_str.parse::<i32>().unwrap_or(0);

            return Err(ExchangeError::Api {
                code: error_code,
                message: format!("{} ({}): {}", error_type, error_code_str, error_message),
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse LTP (Last Traded Price) response
    pub fn parse_ltp(response: &Value, security_id: &str) -> ExchangeResult<f64> {
        Self::check_error(response)?;

        // Response format: { "NSE_EQ": { "1333": { "LTP": 2500.0, ... } } }
        let data = response.get("data")
            .or(Some(response))
            .ok_or_else(|| ExchangeError::Parse("Missing data".to_string()))?;

        // Try different segment keys
        for segment in &["NSE_EQ", "NSE_FNO", "BSE_EQ", "MCX_COMM"] {
            if let Some(segment_data) = data.get(*segment) {
                if let Some(security_data) = segment_data.get(security_id) {
                    if let Some(ltp) = Self::get_f64(security_data, "LTP") {
                        return Ok(ltp);
                    }
                }
            }
        }

        Err(ExchangeError::Parse(format!(
            "LTP not found for security_id {}",
            security_id
        )))
    }

    /// Parse Quote response (includes orderbook)
    pub fn parse_quote(response: &Value, security_id: &str) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let data = response.get("data")
            .or(Some(response))
            .ok_or_else(|| ExchangeError::Parse("Missing data".to_string()))?;

        // Find security data in any segment
        let security_data = ["NSE_EQ", "NSE_FNO", "BSE_EQ", "MCX_COMM"]
            .iter()
            .find_map(|segment| {
                data.get(*segment).and_then(|seg| seg.get(security_id))
            })
            .ok_or_else(|| ExchangeError::Parse(format!(
                "Quote not found for security_id {}",
                security_id
            )))?;

        // Parse bid/ask levels (5 levels)
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for i in 0..5 {
            let bid_price_key = format!("bid{}_price", i);
            let bid_qty_key = format!("bid{}_quantity", i);
            let ask_price_key = format!("ask{}_price", i);
            let ask_qty_key = format!("ask{}_quantity", i);

            if let (Some(price), Some(qty)) = (
                Self::get_f64(security_data, &bid_price_key),
                Self::get_f64(security_data, &bid_qty_key),
            ) {
                if price > 0.0 && qty > 0.0 {
                    bids.push((price, qty));
                }
            }

            if let (Some(price), Some(qty)) = (
                Self::get_f64(security_data, &ask_price_key),
                Self::get_f64(security_data, &ask_qty_key),
            ) {
                if price > 0.0 && qty > 0.0 {
                    asks.push((price, qty));
                }
            }
        }

        Ok(OrderBook {
            timestamp: 0, // Dhan doesn't provide timestamp in quote
            bids,
            asks,
            sequence: None,
        })
    }

    /// Parse historical daily data
    pub fn parse_historical_daily(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Format: { "date": "2024-01-15", "open": 2500, "high": 2550, "low": 2480, "close": 2520, "volume": 1000000 }
            let date_str = Self::require_str(item, "date")?;
            let open_time = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc().timestamp_millis())
                .unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::require_f64(item, "open")?,
                high: Self::require_f64(item, "high")?,
                low: Self::require_f64(item, "low")?,
                close: Self::require_f64(item, "close")?,
                volume: Self::get_f64(item, "volume").unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse historical intraday data
    pub fn parse_historical_intraday(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Format: { "timestamp": "2024-01-15 09:30:00", "open": 2500, ... }
            let timestamp_str = Self::require_str(item, "timestamp")?;
            let open_time = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc().timestamp_millis())
                .unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::require_f64(item, "open")?,
                high: Self::require_f64(item, "high")?,
                low: Self::require_f64(item, "low")?,
                close: Self::require_f64(item, "close")?,
                volume: Self::get_f64(item, "volume").unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse ticker from OHLC response
    pub fn parse_ticker(response: &Value, security_id: &str) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        let data = response.get("data")
            .or(Some(response))
            .ok_or_else(|| ExchangeError::Parse("Missing data".to_string()))?;

        // Find security data
        let security_data = ["NSE_EQ", "NSE_FNO", "BSE_EQ", "MCX_COMM"]
            .iter()
            .find_map(|segment| {
                data.get(*segment).and_then(|seg| seg.get(security_id))
            })
            .ok_or_else(|| ExchangeError::Parse(format!(
                "Ticker not found for security_id {}",
                security_id
            )))?;

        Ok(Ticker {
            symbol: security_id.to_string(),
            last_price: Self::require_f64(security_data, "LTP")?,
            bid_price: Self::get_f64(security_data, "bid0_price"),
            ask_price: Self::get_f64(security_data, "ask0_price"),
            high_24h: Self::get_f64(security_data, "high"),
            low_24h: Self::get_f64(security_data, "low"),
            volume_24h: Self::get_f64(security_data, "volume"),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order status from string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "PENDING" | "TRANSIT" => OrderStatus::New,
            "REJECTED" => OrderStatus::Rejected,
            "CANCELLED" => OrderStatus::Canceled,
            "TRADED" => OrderStatus::Filled,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        }
    }

    /// Parse order side
    fn parse_order_side(side: &str) -> OrderSide {
        match side {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        }
    }

    /// Parse order type
    fn parse_order_type(order_type: &str) -> OrderType {
        match order_type {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            "STOP_LOSS" | "STOP_LOSS_MARKET" => OrderType::StopMarket { stop_price: 0.0 },
            _ => OrderType::Market,
        }
    }

    /// Parse single order
    pub fn parse_order(data: &Value) -> ExchangeResult<Order> {
        let order_id = Self::require_str(data, "orderId")?;
        let status = Self::require_str(data, "orderStatus")?;
        let transaction_type = Self::require_str(data, "transactionType")?;
        let order_type = Self::require_str(data, "orderType")?;
        let symbol = Self::get_str(data, "tradingSymbol")
            .or_else(|| Self::get_str(data, "securityId"))
            .unwrap_or("");

        let quantity = Self::get_f64(data, "quantity").unwrap_or(0.0);
        let filled = Self::get_f64(data, "filled").unwrap_or(0.0);
        let price = Self::get_f64(data, "price").unwrap_or(0.0);

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side: Self::parse_order_side(transaction_type),
            order_type: Self::parse_order_type(order_type),
            status: Self::parse_order_status(status),
            price: if price > 0.0 { Some(price) } else { None },
            stop_price: None,
            quantity,
            filled_quantity: filled,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: chrono::Utc::now().timestamp_millis(),
            updated_at: None,
            time_in_force: crate::core::types::TimeInForce::Gtc,
        })
    }

    /// Parse order book (list of orders)
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut orders = Vec::new();
        for item in arr {
            if let Ok(order) = Self::parse_order(item) {
                orders.push(order);
            }
        }

        Ok(orders)
    }

    /// Parse order placement response
    pub fn parse_order_placement(response: &Value) -> ExchangeResult<Order> {
        Self::check_error(response)?;

        let order_id = Self::require_str(response, "orderId")?;
        let status = Self::require_str(response, "orderStatus")?;

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: None,
            symbol: String::new(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            status: Self::parse_order_status(status),
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: chrono::Utc::now().timestamp_millis(),
            updated_at: None,
            time_in_force: crate::core::types::TimeInForce::Gtc,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse fund limit / cash balance from /v2/fundlimit
    ///
    /// Response: { "availabelBalance": 50000.0, "utilizedAmount": 10000.0, ... }
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let available = Self::get_f64(response, "availabelBalance")
            .or_else(|| Self::get_f64(response, "availableBalance"))
            .unwrap_or(0.0);
        let used = Self::get_f64(response, "utilizedAmount").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: "INR".to_string(),
            free: available,
            locked: used,
            total: available + used,
        }])
    }

    /// Parse holdings
    pub fn parse_holdings(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut balances = Vec::new();
        for item in arr {
            let symbol = Self::get_str(item, "tradingSymbol").unwrap_or("");
            let quantity = Self::get_f64(item, "totalQty").unwrap_or(0.0);
            let available = Self::get_f64(item, "availableQty").unwrap_or(0.0);

            balances.push(Balance {
                asset: symbol.to_string(),
                free: available,
                locked: quantity - available,
                total: quantity,
            });
        }

        Ok(balances)
    }

    /// Parse positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let mut positions = Vec::new();
        for item in arr {
            let symbol = Self::get_str(item, "tradingSymbol").unwrap_or("");
            let quantity = Self::get_f64(item, "netQty").unwrap_or(0.0);

            if quantity == 0.0 {
                continue; // Skip closed positions
            }

            let side = if quantity > 0.0 {
                PositionSide::Long
            } else {
                PositionSide::Short
            };

            positions.push(Position {
                symbol: symbol.to_string(),
                side,
                quantity: quantity.abs(),
                entry_price: Self::get_f64(item, "avgPrice").unwrap_or(0.0),
                mark_price: Self::get_f64(item, "LTP"),
                unrealized_pnl: Self::get_f64(item, "realizedProfit").unwrap_or(0.0),
                realized_pnl: None,
                liquidation_price: None,
                leverage: 1, // Dhan doesn't use leverage for equity
                margin_type: crate::core::types::MarginType::Cross,
                margin: None,
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    /// Parse funds/balance
    pub fn parse_funds(response: &Value) -> ExchangeResult<AccountInfo> {
        Self::check_error(response)?;

        let available = Self::get_f64(response, "availabelBalance").unwrap_or(0.0);
        let used = Self::get_f64(response, "utilizedAmount").unwrap_or(0.0);

        Ok(AccountInfo {
            account_type: crate::core::types::AccountType::Spot,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.03, // Typical Dhan commission
            taker_commission: 0.03,
            balances: vec![Balance {
                asset: "INR".to_string(),
                free: available,
                locked: used,
                total: available + used,
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ltp() {
        let response = json!({
            "NSE_EQ": {
                "1333": {
                    "LTP": 2500.0
                }
            }
        });

        let price = DhanParser::parse_ltp(&response, "1333").unwrap();
        assert_eq!(price, 2500.0);
    }

    #[test]
    fn test_parse_order_status() {
        assert_eq!(DhanParser::parse_order_status("TRADED"), OrderStatus::Filled);
        assert_eq!(DhanParser::parse_order_status("PENDING"), OrderStatus::New);
        assert_eq!(DhanParser::parse_order_status("CANCELLED"), OrderStatus::Canceled);
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "errorType": "ValidationError",
            "errorCode": "OR4001",
            "errorMessage": "Invalid order parameters"
        });

        let result = DhanParser::check_error(&response);
        assert!(result.is_err());
    }
}
