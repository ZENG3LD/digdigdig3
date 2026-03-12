//! # Vertex Protocol Response Parser
//!
//! JSON parsing utilities for Vertex Protocol REST and WebSocket responses.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, TradeSide,
    OrderUpdateEvent, PositionUpdateEvent,
    PositionChangeReason,
};

use super::auth::{from_x18};

/// Parser for Vertex Protocol responses
pub struct VertexParser;

impl VertexParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract data from response (handles {"status": "success", "data": {...}})
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check status
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status != "success" {
                // Parse error
                if let Some(error) = response.get("error") {
                    let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("UNKNOWN");
                    let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                    return Err(ExchangeError::Api {
                        code: -1,
                        message: format!("{}: {}", code, message),
                    });
                }
                return Err(ExchangeError::Parse(format!("Request failed with status: {}", status)));
            }
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Parse f64 from string or number (X18 format)
    fn parse_x18(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| from_x18(s).ok())
            .or_else(|| value.as_f64())
    }

    /// Parse f64 from field
    fn get_x18(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_x18)
    }

    /// Parse required f64
    fn require_x18(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_x18(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse string
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Parse required string
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Parse u32
    fn get_u32(data: &Value, key: &str) -> Option<u32> {
        data.get(key).and_then(|v| v.as_u64()).map(|n| n as u32)
    }

    /// Parse i64
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from market_price query
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;

        // Try bid_x18 or ask_x18
        Self::get_x18(data, "bid_x18")
            .or_else(|| Self::get_x18(data, "ask_x18"))
            .ok_or_else(|| ExchangeError::Parse("Missing price data".to_string()))
    }

    /// Parse orderbook from market_liquidity query
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_data(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_x18(&pair[0])?;
                            let size = Self::parse_x18(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
        })
    }

    /// Parse ticker from market_price query
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;

        let bid_price = Self::get_x18(data, "bid_x18");
        let ask_price = Self::get_x18(data, "ask_x18");
        let last_price = bid_price.or(ask_price).unwrap_or(0.0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: Self::get_i64(data, "last_updated").unwrap_or(0),
        })
    }

    /// Parse candlesticks from archive indexer
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        // Archive response format: {"candlesticks": [...]}
        let candles = response.get("candlesticks")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candlesticks array".to_string()))?;

        let mut klines = Vec::with_capacity(candles.len());

        for candle in candles {
            let open_time = Self::get_i64(candle, "timestamp").unwrap_or(0) * 1000; // sec to ms

            klines.push(Kline {
                open_time,
                open: Self::get_x18(candle, "open_x18").unwrap_or(0.0),
                high: Self::get_x18(candle, "high_x18").unwrap_or(0.0),
                low: Self::get_x18(candle, "low_x18").unwrap_or(0.0),
                close: Self::get_x18(candle, "close_x18").unwrap_or(0.0),
                volume: Self::get_x18(candle, "volume").unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse funding rate from archive indexer
    pub fn parse_funding_rate(response: &Value, symbol: &str) -> ExchangeResult<FundingRate> {
        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing data field".to_string()))?;

        Ok(FundingRate {
            symbol: symbol.to_string(),
            rate: Self::require_x18(data, "funding_rate_x18")?,
            next_funding_time: Self::get_i64(data, "next_funding_time"),
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order ID from place_order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;
        Self::require_str(data, "digest").map(String::from)
    }

    /// Parse order from subaccount_orders query
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;

        let orders = data.get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing orders array".to_string()))?;

        orders.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Parse single order from data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let amount = Self::get_x18(data, "amount").unwrap_or(0.0);
        let side = if amount >= 0.0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let unfilled_amount = Self::get_x18(data, "unfilled_amount").unwrap_or(0.0);
        let filled_quantity = amount.abs() - unfilled_amount.abs();

        // Determine status based on filled amount
        let status = if unfilled_amount.abs() < f64::EPSILON {
            OrderStatus::Filled
        } else if filled_quantity > f64::EPSILON {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::New
        };

        Ok(Order {
            id: Self::get_str(data, "digest").unwrap_or("").to_string(),
            client_order_id: None,
            symbol,
            side,
            order_type: OrderType::Limit { price: 0.0 },
            status,
            price: Self::get_x18(data, "priceX18"),
            stop_price: None,
            quantity: amount.abs(),
            filled_quantity,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "placed_at").unwrap_or(0) * 1000, // sec to ms
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balances from subaccount_info query
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let mut balances = Vec::new();

        // Parse spot balances
        if let Some(spot_balances) = data.get("spot_balances").and_then(|v| v.as_array()) {
            for item in spot_balances {
                let product_id = Self::get_u32(item, "product_id").unwrap_or(0);
                let asset = format!("PRODUCT-{}", product_id);

                if let Some(balance) = item.get("balance") {
                    let amount = Self::get_x18(balance, "amount").unwrap_or(0.0);

                    balances.push(Balance {
                        asset,
                        free: amount.max(0.0),
                        locked: (-amount).max(0.0),
                        total: amount.abs(),
                    });
                }
            }
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions from subaccount_info query
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;

        let mut positions = Vec::new();

        // Parse perp balances
        if let Some(perp_balances) = data.get("perp_balances").and_then(|v| v.as_array()) {
            for item in perp_balances {
                if let Some(pos) = Self::parse_position_data(item) {
                    positions.push(pos);
                }
            }
        }

        Ok(positions)
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let product_id = Self::get_u32(data, "product_id")?;
        let symbol = format!("PRODUCT-{}", product_id);

        let balance = data.get("balance")?;
        let amount = Self::get_x18(balance, "amount").unwrap_or(0.0);

        // Skip empty positions
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
            entry_price: 0.0, // Not provided directly
            mark_price: None,
            unrealized_pnl: 0.0,
            realized_pnl: None,
            leverage: 1,
            liquidation_price: None,
            margin: None,
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let bid_price = Self::get_x18(data, "bid_x18");
        let ask_price = Self::get_x18(data, "ask_x18");
        let last_price = bid_price.or(ask_price).unwrap_or(0.0);

        Ok(Ticker {
            symbol,
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "digest").unwrap_or("").to_string(),
            symbol,
            price: Self::require_x18(data, "price_x18")?,
            quantity: Self::get_x18(data, "size").unwrap_or(0.0).abs(),
            side,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket orderbook message
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<OrderBook> {
        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_x18(&pair[0])?;
                            let size = Self::parse_x18(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
        })
    }

    /// Parse WebSocket order update message
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let order = data.get("order")
            .ok_or_else(|| ExchangeError::Parse("Missing order field".to_string()))?;

        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let amount = Self::get_x18(order, "amount").unwrap_or(0.0);
        let side = if amount >= 0.0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let status = match Self::get_str(order, "status").unwrap_or("open") {
            "open" => OrderStatus::New,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Canceled,
            "rejected" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        };

        let unfilled_amount = Self::get_x18(order, "unfilled_amount").unwrap_or(0.0);
        let filled_quantity = amount.abs() - unfilled_amount.abs();

        Ok(OrderUpdateEvent {
            order_id: Self::get_str(order, "digest").unwrap_or("").to_string(),
            client_order_id: None,
            symbol,
            side,
            order_type: OrderType::Limit { price: 0.0 },
            status,
            price: Self::get_x18(order, "priceX18"),
            quantity: amount.abs(),
            filled_quantity,
            average_price: None,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket fill message
    pub fn parse_ws_fill(data: &Value) -> ExchangeResult<PublicTrade> {
        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "digest").unwrap_or("").to_string(),
            symbol,
            price: Self::require_x18(data, "price_x18")?,
            quantity: Self::get_x18(data, "size").unwrap_or(0.0).abs(),
            side,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket position change message
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
        let product_id = Self::get_u32(data, "product_id").unwrap_or(0);
        let symbol = format!("PRODUCT-{}", product_id);

        let balance = data.get("balance")
            .ok_or_else(|| ExchangeError::Parse("Missing balance field".to_string()))?;

        let amount = Self::get_x18(balance, "amount").unwrap_or(0.0);

        let side = if amount > 0.0 {
            PositionSide::Long
        } else if amount < 0.0 {
            PositionSide::Short
        } else {
            PositionSide::Both
        };

        Ok(PositionUpdateEvent {
            symbol,
            side,
            quantity: amount.abs(),
            entry_price: 0.0,
            mark_price: None,
            unrealized_pnl: 0.0,
            realized_pnl: None,
            liquidation_price: None,
            leverage: None,
            margin_type: Some(crate::core::MarginType::Cross),
            reason: Some(PositionChangeReason::Trade),
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "status": "success",
            "data": {
                "product_id": 2,
                "bid_x18": "30000000000000000000000",
                "ask_x18": "30050000000000000000000"
            }
        });

        let price = VertexParser::parse_price(&response).unwrap();
        assert!((price - 30000.0).abs() < 1.0);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "status": "success",
            "data": {
                "product_id": 2,
                "timestamp": 1234567890,
                "bids": [
                    ["29950000000000000000000", "5000000000000000000"],
                    ["29900000000000000000000", "10000000000000000000"]
                ],
                "asks": [
                    ["30050000000000000000000", "3000000000000000000"],
                    ["30100000000000000000000", "8000000000000000000"]
                ]
            }
        });

        let orderbook = VertexParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 29950.0).abs() < 1.0);
        assert!((orderbook.asks[0].0 - 30050.0).abs() < 1.0);
    }

    #[test]
    fn test_parse_error_response() {
        let response = json!({
            "status": "error",
            "error": {
                "code": "INVALID_SIGNATURE",
                "message": "Signature verification failed"
            }
        });

        let result = VertexParser::extract_data(&response);
        assert!(result.is_err());
    }
}
