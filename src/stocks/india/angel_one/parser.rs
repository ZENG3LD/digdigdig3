//! # Angel One SmartAPI Response Parsers
//!
//! Parse JSON responses to domain types based on Angel One's response format.
//!
//! ## Response Envelope
//! All Angel One responses use a standard envelope:
//! ```json
//! {
//!   "status": true,
//!   "message": "SUCCESS",
//!   "errorcode": "",
//!   "data": { /* actual response data */ }
//! }
//! ```

use serde_json::Value;
use crate::core::{
    ExchangeError, ExchangeResult,
    Ticker, Kline, OrderBook,
    OrderSide, OrderStatus, OrderType,
    Position, PositionSide, MarginType,
    Balance, AccountInfo, AccountType,
};

/// Structured data from order details response
pub struct OrderDetailsData {
    pub order_id: String,
    pub symbol: String,
    pub status: OrderStatus,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: Option<f64>,
    pub price: Option<f64>,
    pub average_price: Option<f64>,
}

pub struct AngelOneParser;

impl AngelOneParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // RESPONSE ENVELOPE HANDLING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check response status and extract data field
    ///
    /// Angel One uses envelope format: {status, message, errorcode, data}
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check status field
        let status = response.get("status")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !status {
            let message = Self::get_str(response, "message")
                .unwrap_or("Unknown error");
            let error_code = Self::get_str(response, "errorcode")
                .unwrap_or("");

            return Err(ExchangeError::Api {
                code: -1,
                message: format!("Angel One API error: {} (code: {})", message, error_code),
            });
        }

        // Extract data field
        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field in response".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTHENTICATION RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse login response and extract tokens
    pub fn parse_login(response: &Value) -> ExchangeResult<(String, String)> {
        let data = Self::extract_data(response)?;

        let jwt_token = Self::require_str(data, "jwtToken")?.to_string();
        let refresh_token = Self::require_str(data, "refreshToken")?.to_string();

        Ok((jwt_token, refresh_token))
    }

    /// Parse token refresh response
    pub fn parse_token_refresh(response: &Value) -> ExchangeResult<(String, String)> {
        Self::parse_login(response) // Same format
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse LTP quote response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;

        let fetched = data.get("fetched")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'fetched' array".to_string()))?;

        if fetched.is_empty() {
            return Err(ExchangeError::Parse("Empty fetched array".to_string()));
        }

        let quote = &fetched[0];
        Self::require_f64(quote, "ltp")
    }

    /// Parse FULL quote response to Ticker
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;

        let fetched = data.get("fetched")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'fetched' array".to_string()))?;

        if fetched.is_empty() {
            return Err(ExchangeError::Parse("Empty fetched array".to_string()));
        }

        let quote = &fetched[0];

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: Self::require_f64(quote, "ltp")?,
            bid_price: Self::get_f64(quote, "bidprice"),
            ask_price: Self::get_f64(quote, "askprice"),
            high_24h: Self::get_f64(quote, "high"),
            low_24h: Self::get_f64(quote, "low"),
            volume_24h: Self::get_f64(quote, "volume"),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Parse order book from FULL quote
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;

        let fetched = data.get("fetched")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'fetched' array".to_string()))?;

        if fetched.is_empty() {
            return Err(ExchangeError::Parse("Empty fetched array".to_string()));
        }

        let quote = &fetched[0];

        // Angel One REST API provides best bid/ask only (not full depth)
        // For full 5-level or 20-level depth, use WebSocket
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if let (Some(bid_price), Some(bid_qty)) = (
            Self::get_f64(quote, "bidprice"),
            Self::get_f64(quote, "bidqty"),
        ) {
            bids.push((bid_price, bid_qty));
        }

        if let (Some(ask_price), Some(ask_qty)) = (
            Self::get_f64(quote, "askprice"),
            Self::get_f64(quote, "askqty"),
        ) {
            asks.push((ask_price, ask_qty));
        }

        Ok(OrderBook {
            bids,
            asks,
            timestamp: chrono::Utc::now().timestamp_millis(),
            sequence: None,
        })
    }

    /// Parse historical candles response
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;

        let candles = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of candles".to_string()))?;

        candles.iter().map(|candle| {
            let arr = candle.as_array()
                .ok_or_else(|| ExchangeError::Parse("Candle should be array".to_string()))?;

            if arr.len() < 6 {
                return Err(ExchangeError::Parse("Candle array too short".to_string()));
            }

            // Format: [timestamp, open, high, low, close, volume]
            let timestamp_str = arr[0].as_str()
                .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))?;

            // Parse ISO timestamp
            let open_time = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                .map_err(|e| ExchangeError::Parse(format!("Failed to parse timestamp: {}", e)))?
                .timestamp_millis();

            Ok(Kline {
                open_time,
                open: Self::parse_value_as_f64(&arr[1])?,
                high: Self::parse_value_as_f64(&arr[2])?,
                low: Self::parse_value_as_f64(&arr[3])?,
                close: Self::parse_value_as_f64(&arr[4])?,
                volume: Self::parse_value_as_f64(&arr[5])?,
                close_time: None,
                quote_volume: None,
                trades: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order placement response and extract order ID
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;

        // PlaceOrder endpoint returns just order ID
        let order_id = if let Some(id) = Self::get_str(data, "orderid") {
            id.to_string()
        } else {
            // PlaceOrderFullResponse returns full order details
            Self::require_str(data, "uniqueorderid")?.to_string()
        };

        Ok(order_id)
    }

    /// Parse order details response into structured data
    pub fn parse_order_details(response: &Value) -> ExchangeResult<OrderDetailsData> {
        let data = Self::extract_data(response)?;

        let order_id = Self::require_str(data, "orderid")?.to_string();
        let symbol = Self::require_str(data, "tradingsymbol")?.to_string();
        let status = Self::parse_order_status(Self::get_str(data, "orderstatus").unwrap_or(""));

        let side = match Self::get_str(data, "transactiontype") {
            Some("BUY") => OrderSide::Buy,
            Some("SELL") => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ordertype") {
            Some("MARKET") => OrderType::Market,
            Some("LIMIT") => OrderType::Limit { price: 0.0 },
            Some("STOPLOSS_LIMIT") => OrderType::StopLimit { stop_price: 0.0, limit_price: 0.0 },
            Some("STOPLOSS_MARKET") => OrderType::StopMarket { stop_price: 0.0 },
            _ => OrderType::Market,
        };

        Ok(OrderDetailsData {
            order_id,
            symbol,
            status,
            side,
            order_type,
            quantity: Self::get_f64(data, "quantity").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "filledshares"),
            price: Self::get_f64(data, "price"),
            average_price: Self::get_f64(data, "averageprice"),
        })
    }

    /// Parse order status string to OrderStatus enum
    fn parse_order_status(status: &str) -> OrderStatus {
        match status.to_uppercase().as_str() {
            "OPEN" | "PENDING" => OrderStatus::New,
            "COMPLETE" | "EXECUTED" => OrderStatus::Filled,
            "CANCELLED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PORTFOLIO RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions response
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;

        let positions_array = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        positions_array.iter().map(|pos| {
            let symbol = Self::require_str(pos, "tradingsymbol")?.to_string();
            let quantity = Self::get_f64(pos, "netqty").unwrap_or(0.0);

            let side = if quantity > 0.0 {
                PositionSide::Long
            } else if quantity < 0.0 {
                PositionSide::Short
            } else {
                PositionSide::Long // Default for zero quantity
            };

            Ok(Position {
                symbol,
                side,
                quantity: quantity.abs(),
                entry_price: Self::get_f64(pos, "averageprice").unwrap_or(0.0),
                mark_price: Self::get_f64(pos, "ltp"),
                liquidation_price: None,
                leverage: 1, // Angel One doesn't have adjustable leverage
                unrealized_pnl: Self::get_f64(pos, "unrealisedprofitandloss").unwrap_or(0.0),
                realized_pnl: Self::get_f64(pos, "realisedprofitandloss"),
                margin: None,
                margin_type: MarginType::Cross, // Default for Indian markets
                take_profit: None,
                stop_loss: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT RESPONSES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse RMS (balance) response
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let available = Self::get_f64(data, "availablecash").unwrap_or(0.0);
        let used = Self::get_f64(data, "utiliseddebits").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: "INR".to_string(),
            free: available,
            locked: used,
            total: available + used,
        }])
    }

    /// Parse account info from profile response
    pub fn parse_account_info(response: &Value) -> ExchangeResult<AccountInfo> {
        let data = Self::extract_data(response)?;

        // Verify client code exists (required field)
        let _client_code = Self::require_str(data, "clientcode")?;

        Ok(AccountInfo {
            account_type: AccountType::Spot,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0,
            taker_commission: 0.0,
            balances: vec![],
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn parse_value_as_f64(val: &Value) -> ExchangeResult<f64> {
        val.as_f64()
            .or_else(|| val.as_str().and_then(|s| s.parse().ok()))
            .ok_or_else(|| ExchangeError::Parse("Failed to parse number".to_string()))
    }
}
