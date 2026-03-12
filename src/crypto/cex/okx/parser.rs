//! # OKX Response Parser
//!
//! Парсинг JSON ответов от OKX API v5.
//!
//! ## OKX Response Format
//!
//! Все ответы имеют структуру:
//! ```json
//! {
//!   "code": "0",
//!   "msg": "",
//!   "data": [...]
//! }
//! ```
//!
//! - `code`: "0" = success, other = error
//! - `msg`: Error message (empty on success)
//! - `data`: Always array, even for single object

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, TimeInForce, MarginType,
    FundingRate, PublicTrade, TradeSide,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    SymbolInfo,
};

/// Order book level pairs (price, quantity)
type OrderBookLevels = Vec<(f64, f64)>;

/// Parsed order book bids and asks
type OrderBookSides = (OrderBookLevels, OrderBookLevels);

/// Парсер ответов OKX API
pub struct OkxParser;

impl OkxParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Проверить код ответа и извлечь data
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check code field
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("0");

        if code != "0" {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code.parse().unwrap_or(-1),
                message: format!("OKX error {}: {}", code, msg),
            });
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Извлечь первый элемент из data array
    pub fn extract_first_data(response: &Value) -> ExchangeResult<&Value> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        arr.first()
            .ok_or_else(|| ExchangeError::Parse("'data' array is empty".to_string()))
    }

    /// Парсить f64 из string или number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Парсить f64 из поля
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Парсить обязательный f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Парсить строку из поля
    pub fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Парсить обязательную строку
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Парсить i64 из string или number
    pub fn parse_i64(value: &Value) -> Option<i64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_i64())
    }

    /// Парсить i64 из поля
    pub fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(Self::parse_i64)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить ticker
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let data = Self::extract_first_data(response)?;

        Ok(Ticker {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "bidPx"),
            ask_price: Self::get_f64(data, "askPx"),
            high_24h: Self::get_f64(data, "high24h"),
            low_24h: Self::get_f64(data, "low24h"),
            volume_24h: Self::get_f64(data, "vol24h"),
            quote_volume_24h: Self::get_f64(data, "volCcy24h"),
            price_change_24h: None, // OKX doesn't provide this directly
            price_change_percent_24h: None, // Would need to calculate from open24h
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let data = Self::extract_first_data(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            // OKX format: [price, size, deprecated, amount]
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None, // OKX doesn't provide sequence in this endpoint
        })
    }

    /// Парсить klines
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 9 {
                continue;
            }

            // OKX format: [timestamp, open, high, low, close, vol, volCcy, volCcyQuote, confirm]
            let open_time = Self::parse_i64(&candle[0]).unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                quote_volume: Self::parse_f64(&candle[6]),
                close_time: None,
                trades: None,
            });
        }

        // OKX returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Парсить symbols/instruments
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut symbols = Vec::with_capacity(arr.len());

        for item in arr {
            let symbol = Self::get_str(item, "instId").unwrap_or("").to_string();
            let base_asset = Self::get_str(item, "baseCcy").unwrap_or("").to_string();
            let quote_asset = Self::get_str(item, "quoteCcy").unwrap_or("").to_string();

            let min_quantity = Self::get_f64(item, "minSz");
            let max_quantity = Self::get_f64(item, "maxLmtSz");
            let step_size = Self::get_f64(item, "lotSz");
            let min_notional = None; // OKX doesn't provide this directly

            let status = Self::get_str(item, "state").unwrap_or("").to_string();
            let price_precision = 8; // Default
            let quantity_precision = 8; // Default

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status,
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity,
                step_size,
                min_notional,
            });
        }

        Ok(symbols)
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let data = Self::extract_first_data(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "fundingRate")?,
            next_funding_time: Self::get_i64(data, "nextFundingTime"),
            timestamp: Self::get_i64(data, "fundingTime").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order response (place/cancel)
    pub fn parse_order_response(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_first_data(response)?;

        // Check sCode for individual order status
        let s_code = Self::get_str(data, "sCode").unwrap_or("0");
        if s_code != "0" {
            let s_msg = Self::get_str(data, "sMsg").unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: s_code.parse().unwrap_or(-1),
                message: format!("Order error {}: {}", s_code, s_msg),
            });
        }

        let order_id = Self::get_str(data, "ordId")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ordId'".to_string()))?
            .to_string();

        Ok(order_id)
    }

    /// Парсить order details
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        let data = Self::extract_first_data(response)?;
        Self::parse_order_data(data)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ordType").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "ordId").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "clOrdId").map(String::from),
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "px"),
            stop_price: Self::get_f64(data, "slTriggerPx"),
            quantity: Self::get_f64(data, "sz").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "accFillSz").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avgPx"),
            commission: None, // Would need to get from fills
            commission_asset: None,
            created_at: Self::get_i64(data, "cTime").unwrap_or(0),
            updated_at: Self::get_i64(data, "uTime"),
            time_in_force: TimeInForce::Gtc, // Default
        })
    }

    /// Парсить список orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let orders = arr.iter()
            .filter_map(|item| Self::parse_order_data(item).ok())
            .collect::<Vec<_>>();

        Ok(orders)
    }

    /// Парсить order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "state").unwrap_or("live") {
            "live" => OrderStatus::Open,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" => OrderStatus::Canceled,
            _ => OrderStatus::Open,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balance
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_first_data(response)?;

        let details = data.get("details")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'details' array".to_string()))?;

        let mut balances = Vec::with_capacity(details.len());

        for item in details {
            let asset = Self::get_str(item, "ccy").unwrap_or("").to_string();
            let free = Self::get_f64(item, "availBal").unwrap_or(0.0);
            let locked = Self::get_f64(item, "frozenBal").unwrap_or(0.0);
            let total = Self::get_f64(item, "eq").unwrap_or(free + locked);

            balances.push(Balance {
                asset,
                free,
                locked,
                total,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut positions = Vec::with_capacity(arr.len());

        for item in arr {
            let pos_side_str = Self::get_str(item, "posSide").unwrap_or("net");
            let pos_qty = Self::get_f64(item, "pos").unwrap_or(0.0);

            // Determine position side
            let side = match pos_side_str {
                "long" => PositionSide::Long,
                "short" => PositionSide::Short,
                "net" => {
                    if pos_qty > 0.0 {
                        PositionSide::Long
                    } else if pos_qty < 0.0 {
                        PositionSide::Short
                    } else {
                        continue; // Skip zero positions
                    }
                }
                _ => continue,
            };

            let quantity = pos_qty.abs();
            if quantity == 0.0 {
                continue; // Skip zero positions
            }

            positions.push(Position {
                symbol: Self::get_str(item, "instId").unwrap_or("").to_string(),
                side,
                quantity,
                entry_price: Self::get_f64(item, "avgPx").unwrap_or(0.0),
                mark_price: Self::get_f64(item, "markPx"),
                liquidation_price: Self::get_f64(item, "liqPx"),
                unrealized_pnl: Self::get_f64(item, "upl").unwrap_or(0.0),
                realized_pnl: None, // OKX doesn't provide realized PnL in position endpoint
                leverage: Self::get_f64(item, "lever").map(|l| l as u32).unwrap_or(1),
                margin: Self::get_f64(item, "margin"),
                margin_type: match Self::get_str(item, "mgnMode") {
                    Some("isolated") => MarginType::Isolated,
                    _ => MarginType::Cross,
                },
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить WebSocket ticker update
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "bidPx"),
            ask_price: Self::get_f64(data, "askPx"),
            high_24h: Self::get_f64(data, "high24h"),
            low_24h: Self::get_f64(data, "low24h"),
            volume_24h: Self::get_f64(data, "vol24h"),
            quote_volume_24h: Self::get_f64(data, "volCcy24h"),
            price_change_24h: {
                let last = Self::get_f64(data, "last");
                let open24h = Self::get_f64(data, "open24h");
                match (last, open24h) {
                    (Some(l), Some(o)) => Some(l - o),
                    _ => None,
                }
            },
            price_change_percent_24h: {
                let last = Self::get_f64(data, "last");
                let open24h = Self::get_f64(data, "open24h");
                match (last, open24h) {
                    (Some(l), Some(o)) if o != 0.0 => Some(((l - o) / o) * 100.0),
                    _ => None,
                }
            },
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить WebSocket orderbook update
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<OrderBookSides> {
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

        Ok((parse_levels("asks"), parse_levels("bids")))
    }

    /// Парсить WebSocket trade
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            symbol: Self::get_str(data, "instId").unwrap_or("").to_string(),
            id: Self::get_str(data, "tradeId").unwrap_or("").to_string(),
            price: Self::require_f64(data, "px")?,
            quantity: Self::require_f64(data, "sz")?,
            side,
            timestamp: Self::get_i64(data, "ts").unwrap_or(0),
        })
    }

    /// Парсить WebSocket kline
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> {
        let candle = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Kline data is not an array".to_string()))?;

        if candle.len() < 9 {
            return Err(ExchangeError::Parse("Incomplete kline data".to_string()));
        }

        Ok(Kline {
            open_time: Self::parse_i64(&candle[0]).unwrap_or(0),
            open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
            high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
            low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
            close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
            volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
            quote_volume: Self::parse_f64(&candle[6]),
            close_time: None,
            trades: None,
        })
    }

    /// Парсить WebSocket order update
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        // TODO: Implement proper OrderUpdateEvent parsing
        // For now, extract minimal required fields
        let _ = data; // Suppress unused variable warning
        Err(ExchangeError::Parse("WebSocket order updates not yet implemented".to_string()))
    }

    /// Парсить WebSocket balance update
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
        // TODO: Implement proper BalanceUpdateEvent parsing
        let _ = data;
        Err(ExchangeError::Parse("WebSocket balance updates not yet implemented".to_string()))
    }

    /// Парсить WebSocket position update
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
        // TODO: Implement proper PositionUpdateEvent parsing
        let _ = data;
        Err(ExchangeError::Parse("WebSocket position updates not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": "0",
            "msg": "",
            "data": [{
                "instId": "BTC-USDT",
                "last": "43250.5",
                "bidPx": "43250.0",
                "askPx": "43251.0",
                "high24h": "43500.0",
                "low24h": "42500.0",
                "vol24h": "1850.25",
                "volCcy24h": "79852341.25",
                "ts": "1672841403093"
            }]
        });

        let ticker = OkxParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert_eq!(ticker.last_price, 43250.5);
    }

    #[test]
    fn test_parse_error_response() {
        let response = json!({
            "code": "50111",
            "msg": "Invalid sign",
            "data": []
        });

        let result = OkxParser::extract_data(&response);
        assert!(result.is_err());
        if let Err(ExchangeError::Api { code: _, message }) = result {
            assert!(message.contains("50111"));
            assert!(message.contains("Invalid sign"));
        }
    }
}
