//! # Bithumb Response Parser
//!
//! Парсинг JSON ответов от Bithumb Pro API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, Order, Balance,
    OrderSide, OrderType, OrderStatus,
    PublicTrade, TradeSide, StreamEvent, OrderUpdateEvent, BalanceUpdateEvent,
    OrderbookDelta as OrderbookDeltaData,
};

/// Парсер ответов Bithumb Pro API
pub struct BithumbParser;

impl BithumbParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Извлечь data из response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Проверить успешность ответа
    /// Bithumb Pro: code="0" или code < 10000 = success
    pub fn check_response(response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("10000");

        let code_num = code.parse::<i32>().unwrap_or(10000);

        if code != "0" && code_num >= 10000 {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code_num,
                message: msg.to_string(),
            });
        }

        Ok(())
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
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Парсить обязательную строку
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;
        Self::require_f64(data, "c")  // "c" = current/close price
    }

    /// Парсить klines
    /// Bithumb Pro format: [[timestamp, open, high, low, close, volume], ...]
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 6 {
                continue;
            }

            // Bithumb Pro format: [timestamp, open, high, low, close, volume]
            let open_time = candle[0].as_i64().unwrap_or(0); // milliseconds

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Парсить orderbook
    /// Bithumb Pro format: { "b": [[price, qty], ...], "s": [[price, qty], ...], "ver": "..." }
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: 0, // Bithumb Pro doesn't provide timestamp in orderbook
            bids: parse_levels("b"),  // "b" = bids
            asks: parse_levels("s"),  // "s" = asks (sell orders)
            sequence: data.get("ver").and_then(|v| v.as_str()).map(String::from),
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Парсить ticker
    /// Bithumb Pro format: { "c": "price", "h": "high", "l": "low", "p": "change%", "v": "volume", "s": "symbol" }
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;

        let last_price = Self::get_f64(data, "c").unwrap_or(0.0);  // "c" = current/close

        Ok(Ticker {
            symbol: Self::get_str(data, "s").unwrap_or("").to_string(),  // "s" = symbol
            last_price,
            bid_price: None,  // Not provided in ticker response
            ask_price: None,  // Not provided in ticker response
            high_24h: Self::get_f64(data, "h"),  // "h" = 24h high
            low_24h: Self::get_f64(data, "l"),   // "l" = 24h low
            volume_24h: Self::get_f64(data, "v"),  // "v" = 24h volume
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "p"),  // "p" = price change percent
            timestamp: 0,  // Not provided in ticker response
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;
        Self::parse_order_data(data, symbol)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("limit").to_lowercase().as_str() {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "orderId").unwrap_or("").to_string(),
            client_order_id: None,
            symbol: Self::get_str(data, "symbol").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: None,
            quantity: Self::get_f64(data, "quantity").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "dealQuantity").unwrap_or(0.0),
            average_price: Self::get_f64(data, "dealPrice"),
            commission: Self::get_f64(data, "fee"),
            commission_asset: None,
            created_at: data.get("createTime").and_then(|t| t.as_i64()).unwrap_or(0),
            updated_at: data.get("updateTime").and_then(|t| t.as_i64()),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    /// Bithumb Pro status: "trading", "traded", "cancelled", "pending"
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("pending") {
            "trading" => OrderStatus::PartiallyFilled,
            "traded" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Canceled,
            "pending" => OrderStatus::New,
            _ => OrderStatus::New,
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;

        // Bithumb Pro wraps orders in "list"
        let items = data.get("list")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        items.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Парсить order ID из create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;
        Self::require_str(data, "orderId").map(String::from)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances
    /// Bithumb Pro format: [{"coinType": "BTC", "count": "1.23", "frozen": "0.1", "available": "1.13"}, ...]
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_response(response)?;
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of balances".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "coinType").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let total = Self::get_f64(item, "count").unwrap_or(0.0);
            let locked = Self::get_f64(item, "frozen").unwrap_or(0.0);
            let free = Self::get_f64(item, "available").unwrap_or(0.0);

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
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker update
    /// Format: { "c": "51000.00", "h": "52000.00", "l": "49500.00", "p": "2.00", "v": "12345.678", "s": "BTC-USDT" }
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        let last_price = Self::get_f64(data, "c").unwrap_or(0.0);

        Ok(Ticker {
            symbol: Self::get_str(data, "s").unwrap_or("").to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "p"),
            timestamp: 0,
        })
    }

    /// Parse WebSocket orderbook snapshot
    /// Format: { "b": [[price, qty], ...], "s": [[price, qty], ...], "ver": "123456789", "symbol": "BTC-USDT" }
    pub fn parse_ws_orderbook_snapshot(data: &Value) -> ExchangeResult<OrderBook> {
        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: 0,
            bids: parse_levels("b"),
            asks: parse_levels("s"),
            sequence: data.get("ver").and_then(|v| v.as_str()).map(String::from),
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Parse WebSocket orderbook delta
    /// Format: { "b": [[price, qty], ...], "s": [[price, qty], ...], "ver": "123456790" }
    /// Note: qty = 0 means remove that price level
    pub fn parse_ws_orderbook_delta(data: &Value) -> ExchangeResult<StreamEvent> {
        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(StreamEvent::OrderbookDelta(OrderbookDeltaData {
            bids: parse_levels("b"),
            asks: parse_levels("s"),
            timestamp: 0,
            first_update_id: None,
            last_update_id: None,
            prev_update_id: None,
            event_time: None,
            checksum: None,
        }))
    }

    /// Parse WebSocket trades
    /// Format: [{ "p": "50000.00", "s": "buy", "v": "0.123", "t": 1712230310689 }, ...]
    pub fn parse_ws_trades(data: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of trades".to_string()))?;

        let mut trades = Vec::new();

        for item in arr {
            let price = Self::get_f64(item, "p").unwrap_or(0.0);
            let quantity = Self::get_f64(item, "v").unwrap_or(0.0);
            let timestamp = item.get("t").and_then(|t| t.as_i64()).unwrap_or(0);

            let side = match Self::get_str(item, "s").unwrap_or("buy") {
                "sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            };

            trades.push(PublicTrade {
                id: timestamp.to_string(),
                symbol: String::new(), // Not provided in WebSocket trade data
                price,
                quantity,
                side,
                timestamp,
            });
        }

        Ok(trades)
    }

    /// Parse WebSocket order update
    /// Format: { "oId": "...", "price": "50000.00", "quantity": "1.00", "dealQuantity": "0.50",
    ///           "side": "buy", "symbol": "BTC-USDT", "type": "limit", "status": "trading", ... }
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let order_id = Self::get_str(data, "oId")
            .or_else(|| Self::get_str(data, "orderId"))
            .unwrap_or("")
            .to_string();

        let symbol = Self::get_str(data, "symbol").unwrap_or("").to_string();

        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("limit") {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match Self::get_str(data, "status").unwrap_or("pending") {
            "trading" => OrderStatus::PartiallyFilled,
            "traded" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Canceled,
            "pending" => OrderStatus::New,
            _ => OrderStatus::New,
        };

        let price = Self::get_f64(data, "price");
        let quantity = Self::get_f64(data, "quantity").unwrap_or(0.0);
        let filled_quantity = Self::get_f64(data, "dealQuantity")
            .or_else(|| Self::get_f64(data, "amountFill"))
            .unwrap_or(0.0);
        let average_price = Self::get_f64(data, "dealPrice");
        let timestamp = data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);

        Ok(OrderUpdateEvent {
            order_id,
            client_order_id: None,
            symbol,
            side,
            order_type,
            status,
            price,
            quantity,
            filled_quantity,
            average_price,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: Self::get_f64(data, "fee"),
            commission_asset: None,
            trade_id: None,
            timestamp,
        })
    }

    /// Parse WebSocket balance update
    /// Format: { "availableAmount": "10000.00", "totalAmount": "12000.00", "coin": "USDT", ... }
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
        let asset = Self::get_str(data, "coin").unwrap_or("").to_string();
        let free = Self::get_f64(data, "availableAmount").unwrap_or(0.0);
        let total = Self::get_f64(data, "totalAmount").unwrap_or(0.0);
        let locked = total - free;
        let timestamp = data.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);

        Ok(BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta: None,
            reason: None,
            timestamp,
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
            "code": "0",
            "msg": "success",
            "success": true,
            "data": {
                "c": "51000.00",
                "s": "BTC-USDT"
            }
        });

        let price = BithumbParser::parse_price(&response).unwrap();
        assert!((price - 51000.00).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "data": {
                "b": [["50000.00", "0.123"], ["49990.00", "0.234"]],
                "s": [["50010.00", "0.345"], ["50020.00", "0.456"]],
                "ver": "123456789"
            }
        });

        let orderbook = BithumbParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].price - 50000.0).abs() < f64::EPSILON);
        assert!((orderbook.asks[0].price - 50010.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.sequence, Some("123456789".to_string()));
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "data": {
                "c": "51000.00",
                "h": "52000.00",
                "l": "49500.00",
                "p": "2.00",
                "v": "12345.678",
                "s": "BTC-USDT"
            }
        });

        let ticker = BithumbParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert!((ticker.last_price - 51000.0).abs() < f64::EPSILON);
        assert!((ticker.high_24h.unwrap() - 52000.0).abs() < f64::EPSILON);
        assert!((ticker.low_24h.unwrap() - 49500.0).abs() < f64::EPSILON);
        assert!((ticker.price_change_percent_24h.unwrap() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_response_success() {
        let response = json!({"code": "0", "msg": "success"});
        assert!(BithumbParser::check_response(&response).is_ok());
    }

    #[test]
    fn test_check_response_error() {
        let response = json!({"code": "10005", "msg": "Invalid apiKey"});
        let result = BithumbParser::check_response(&response);
        assert!(result.is_err());
        if let Err(ExchangeError::Api { code, message }) = result {
            assert_eq!(code, 10005);
            assert_eq!(message, "Invalid apiKey");
        }
    }
}
