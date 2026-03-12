//! # Fyers Response Parser
//!
//! JSON response parsing for Fyers API v3.

use serde_json::Value;

use crate::core::types::{
    AccountInfo, AccountType, Balance, ExchangeError, ExchangeResult, Kline, Order, OrderBook, OrderSide,
    OrderStatus, OrderType, Position, PositionSide, Price, Ticker, TimeInForce, MarginType,
};

/// Fyers response parser
pub struct FyersParser;

impl FyersParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPER FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response is successful
    fn is_success(value: &Value) -> ExchangeResult<()> {
        let status = value["s"].as_str().unwrap_or("error");
        let code = value["code"].as_i64().unwrap_or(-1);

        if status == "ok" && code == 200 {
            Ok(())
        } else {
            let message = value["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            Err(ExchangeError::Api {
                code: code as i32,
                message: format!("Fyers API error: {}", message),
            })
        }
    }

    /// Get data field from response
    fn get_data(value: &Value) -> ExchangeResult<&Value> {
        Self::is_success(value)?;
        value
            .get("data")
            .or_else(|| value.get("d"))
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' or 'd' field".to_string()))
    }

    /// Parse order side from integer
    fn parse_order_side(side: i64) -> OrderSide {
        match side {
            1 => OrderSide::Buy,
            -1 => OrderSide::Sell,
            _ => OrderSide::Buy, // default
        }
    }

    /// Parse order type from integer
    fn parse_order_type(order_type: i64) -> OrderType {
        match order_type {
            1 => OrderType::Limit,
            2 => OrderType::Market,
            3 => OrderType::StopLoss,
            4 => OrderType::StopLossLimit,
            _ => OrderType::Market, // default
        }
    }

    /// Parse order status from integer
    fn parse_order_status(status: i64) -> OrderStatus {
        match status {
            1 => OrderStatus::Canceled,
            2 => OrderStatus::Filled,
            4 => OrderStatus::PartiallyFilled, // transit
            5 => OrderStatus::Rejected,
            6 => OrderStatus::Open, // pending
            7 => OrderStatus::Expired,
            _ => OrderStatus::Open, // default
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse last traded price from quotes response
    pub fn parse_ltp(response: &Value, symbol: &str) -> ExchangeResult<Price> {
        Self::is_success(response)?;

        let data = response
            .get("d")
            .ok_or_else(|| ExchangeError::Parse("Missing 'd' field".to_string()))?;

        if let Some(array) = data.as_array() {
            for item in array {
                if item["n"].as_str() == Some(symbol) {
                    let ltp = item["v"]["lp"]
                        .as_f64()
                        .ok_or_else(|| ExchangeError::Parse("Missing 'lp' field".to_string()))?;
                    return Ok(ltp);
                }
            }
        }

        Err(ExchangeError::Parse(format!("Symbol {} not found in response", symbol)))
    }

    /// Parse ticker from quotes response
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Self::is_success(response)?;

        let data = response
            .get("d")
            .ok_or_else(|| ExchangeError::Parse("Missing 'd' field".to_string()))?;

        if let Some(array) = data.as_array() {
            for item in array {
                if item["n"].as_str() == Some(symbol) {
                    let v = &item["v"];

                    return Ok(Ticker {
                        symbol: symbol.to_string(),
                        last_price: v["lp"].as_f64().unwrap_or(0.0),
                        bid_price: v["bid"].as_f64(),
                        ask_price: v["ask"].as_f64(),
                        high_24h: v["high_price"].as_f64(),
                        low_24h: v["low_price"].as_f64(),
                        volume_24h: v["volume"].as_f64(),
                        quote_volume_24h: None,
                        price_change_24h: v["ch"].as_f64(),
                        price_change_percent_24h: v["chp"].as_f64(),
                        timestamp: v["timestamp"].as_i64().unwrap_or(0),
                    });
                }
            }
        }

        Err(ExchangeError::Parse(format!("Symbol {} not found in response", symbol)))
    }

    /// Parse orderbook from depth response
    pub fn parse_orderbook(response: &Value, symbol: &str) -> ExchangeResult<OrderBook> {
        Self::is_success(response)?;

        let data = response
            .get("d")
            .ok_or_else(|| ExchangeError::Parse("Missing 'd' field".to_string()))?;

        let depth = data
            .get(symbol)
            .ok_or_else(|| ExchangeError::Parse(format!("Symbol {} not found", symbol)))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if let Some(bids_array) = depth["bids"].as_array() {
            for bid in bids_array {
                let price = bid["price"].as_f64().unwrap_or(0.0);
                let volume = bid["volume"].as_f64().unwrap_or(0.0);
                bids.push((price, volume));
            }
        }

        if let Some(asks_array) = depth["ask"].as_array() {
            for ask in asks_array {
                let price = ask["price"].as_f64().unwrap_or(0.0);
                let volume = ask["volume"].as_f64().unwrap_or(0.0);
                asks.push((price, volume));
            }
        }

        Ok(OrderBook {
            bids,
            asks,
            timestamp: depth["timestamp"].as_i64().unwrap_or(0),
            sequence: None,
        })
    }

    /// Parse klines from history response
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::is_success(response)?;

        let candles = response
            .get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'candles' array".to_string()))?;

        let mut klines = Vec::new();

        for candle in candles {
            if let Some(arr) = candle.as_array() {
                if arr.len() >= 6 {
                    klines.push(Kline {
                        open_time: arr[0].as_i64().unwrap_or(0),
                        open: arr[1].as_f64().unwrap_or(0.0),
                        high: arr[2].as_f64().unwrap_or(0.0),
                        low: arr[3].as_f64().unwrap_or(0.0),
                        close: arr[4].as_f64().unwrap_or(0.0),
                        volume: arr[5].as_f64().unwrap_or(0.0),
                        close_time: arr[0].as_i64(),
                        quote_volume: None,
                        trades: None,
                    });
                }
            }
        }

        Ok(klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse single order from response
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        Self::is_success(response)?;

        // For place/modify/cancel responses, extract order_id
        if let Some(order_id) = response.get("id").and_then(|v| v.as_str()) {
            return Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: String::new(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                status: OrderStatus::Open,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: None,
                time_in_force: TimeInForce::GTC,
            });
        }

        // For get_order responses, parse full order data
        let data = Self::get_data(response)?;
        Self::parse_order_data(data)
    }

    /// Parse order data object
    fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        Ok(Order {
            id: data["id"]
                .as_str()
                .or_else(|| data["orderNumber"].as_str())
                .unwrap_or("")
                .to_string(),
            client_order_id: data["clientId"].as_str().map(|s| s.to_string()),
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            side: Self::parse_order_side(data["side"].as_i64().unwrap_or(1)),
            order_type: Self::parse_order_type(data["type"].as_i64().unwrap_or(2)),
            status: Self::parse_order_status(data["orderStatus"].as_i64().unwrap_or(6)),
            price: data["limitPrice"].as_f64(),
            stop_price: data["stopPrice"].as_f64(),
            quantity: data["qty"].as_f64().unwrap_or(0.0),
            filled_quantity: data["filledQty"].as_f64().unwrap_or(0.0),
            average_price: data["tradedPrice"].as_f64(),
            commission: None,
            commission_asset: None,
            created_at: data["orderDateTime"]
                .as_str()
                .and_then(Self::parse_datetime)
                .unwrap_or(0),
            updated_at: None,
            time_in_force: if data["orderValidity"].as_str() == Some("IOC") {
                TimeInForce::IOC
            } else {
                TimeInForce::GTC
            },
        })
    }

    /// Parse orders list
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::is_success(response)?;

        let orderbook = response
            .get("orderBook")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orderBook' array".to_string()))?;

        orderbook.iter().map(Self::parse_order_data).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balance from funds response
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::is_success(response)?;

        let fund_limit = response
            .get("fund_limit")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'fund_limit' array".to_string()))?;

        let mut balances = Vec::new();

        for segment in fund_limit {
            let title = segment["title"].as_str().unwrap_or("Unknown");
            let total = segment["total_balance"].as_f64().unwrap_or(0.0);
            let available = segment["available_margin"].as_f64().unwrap_or(0.0);
            let locked = segment["used_margin"].as_f64().unwrap_or(0.0);

            balances.push(Balance {
                asset: title.to_string(),
                free: available,
                locked,
                total,
            });
        }

        Ok(balances)
    }

    /// Parse account info from profile response
    pub fn parse_account_info(response: &Value) -> ExchangeResult<AccountInfo> {
        let _data = Self::get_data(response)?;

        Ok(AccountInfo {
            account_type: AccountType::Spot, // Fyers supports multiple account types
            can_trade: true, // Assume true if we have valid credentials
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // Fyers doesn't provide this via API
            taker_commission: 0.0,
            balances: Vec::new(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions from positions response
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::is_success(response)?;

        let net_positions = response
            .get("netPositions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'netPositions' array".to_string()))?;

        let mut positions = Vec::new();

        for pos in net_positions {
            let net_qty = pos["netQty"].as_f64().unwrap_or(0.0);

            if net_qty != 0.0 {
                let side = if pos["side"].as_i64().unwrap_or(1) == 1 {
                    PositionSide::Long
                } else {
                    PositionSide::Short
                };

                positions.push(Position {
                    symbol: pos["symbol"].as_str().unwrap_or("").to_string(),
                    side,
                    quantity: net_qty.abs(),
                    entry_price: pos["netAvg"].as_f64().unwrap_or(0.0),
                    mark_price: pos["ltp"].as_f64(),
                    unrealized_pnl: pos["unrealized_profit"].as_f64().unwrap_or(0.0),
                    realized_pnl: Some(pos["realized_profit"].as_f64().unwrap_or(0.0)),
                    liquidation_price: None,
                    leverage: 1,
                    margin_type: MarginType::Cross,
                    margin: None,
                    take_profit: None,
                    stop_loss: None,
                });
            }
        }

        Ok(positions)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UTILITY FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse datetime string to timestamp (milliseconds)
    /// Format: "2026-01-26 09:15:30"
    fn parse_datetime(_datetime_str: &str) -> Option<i64> {
        // Simple parsing - in production, use chrono or time crate
        // For now, return current timestamp
        Some(crate::core::timestamp_millis() as i64)
    }

    /// Parse access token from token response
    pub fn parse_access_token(response: &Value) -> ExchangeResult<String> {
        Self::is_success(response)?;

        response
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing 'access_token' field".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ltp() {
        let response = json!({
            "s": "ok",
            "code": 200,
            "d": [
                {
                    "n": "NSE:SBIN-EQ",
                    "v": {
                        "lp": 550.50
                    }
                }
            ]
        });

        let ltp = FyersParser::parse_ltp(&response, "NSE:SBIN-EQ").unwrap();
        assert_eq!(ltp, 550.50);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "s": "ok",
            "code": 200,
            "d": [
                {
                    "n": "NSE:SBIN-EQ",
                    "v": {
                        "lp": 550.50,
                        "open_price": 548.00,
                        "high_price": 552.00,
                        "low_price": 547.50,
                        "close_price": 549.00,
                        "volume": 1234567,
                        "ch": 1.50,
                        "chp": 0.27,
                        "bid": 550.45,
                        "ask": 550.55,
                        "timestamp": 1640000000
                    }
                }
            ]
        });

        let ticker = FyersParser::parse_ticker(&response, "NSE:SBIN-EQ").unwrap();
        assert_eq!(ticker.last_price, 550.50);
        assert_eq!(ticker.open, 548.00);
        assert_eq!(ticker.high, 552.00);
        assert_eq!(ticker.low, 547.50);
    }

    #[test]
    fn test_parse_order_side() {
        assert_eq!(FyersParser::parse_order_side(1), OrderSide::Buy);
        assert_eq!(FyersParser::parse_order_side(-1), OrderSide::Sell);
    }

    #[test]
    fn test_parse_order_type() {
        assert_eq!(FyersParser::parse_order_type(1), OrderType::Limit);
        assert_eq!(FyersParser::parse_order_type(2), OrderType::Market);
        assert_eq!(FyersParser::parse_order_type(3), OrderType::StopLoss);
        assert_eq!(FyersParser::parse_order_type(4), OrderType::StopLossLimit);
    }

    #[test]
    fn test_parse_order_status() {
        assert_eq!(FyersParser::parse_order_status(1), OrderStatus::Canceled);
        assert_eq!(FyersParser::parse_order_status(2), OrderStatus::Filled);
        assert_eq!(FyersParser::parse_order_status(6), OrderStatus::Open);
    }

    #[test]
    fn test_parse_access_token() {
        let response = json!({
            "s": "ok",
            "code": 200,
            "access_token": "eyJ0eXAiOiJKV1Qi"
        });

        let token = FyersParser::parse_access_token(&response).unwrap();
        assert_eq!(token, "eyJ0eXAiOiJKV1Qi");
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "s": "error",
            "code": -1600,
            "message": "Could not authenticate the user"
        });

        let result = FyersParser::is_success(&response);
        assert!(result.is_err());
    }
}
