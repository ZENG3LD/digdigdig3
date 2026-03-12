//! # Upbit Response Parser
//!
//! Парсинг JSON ответов от Upbit API.
//!
//! ## CRITICAL: Two Different Formats
//! - **REST API**: Returns arrays/objects directly (no wrapper)
//! - **WebSocket API**: Returns objects with different field names
//!
//! This parser handles both formats.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, PublicTrade,
    OrderSide, OrderType, OrderStatus, TradeSide, SymbolInfo,
};

/// Парсер ответов Upbit API
pub struct UpbitParser;

impl UpbitParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check for error response
    /// Upbit errors have: {"error": {"name": "...", "message": "..."}}
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let name = error.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_error");
            let message = error.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: 0,
                message: format!("{}: {}", name, message),
            });
        }
        Ok(())
    }

    /// Парсить f64 из number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_f64()
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

    /// Parse ISO 8601 timestamp to milliseconds
    fn _parse_iso_timestamp(_iso_str: &str) -> Option<i64> {
        // Upbit format: "2024-06-19T08:31:43+00:00"
        // For now, we'll use a simple parser
        // TODO: Use chrono or time crate for proper parsing
        None // Placeholder
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Парсить price from ticker
    /// Upbit returns array of tickers, we take first one
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of tickers".to_string()))?;

        let ticker = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?;

        Self::require_f64(ticker, "trade_price")
    }

    /// Парсить klines
    /// Upbit returns array of candles (newest first)
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of candles".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Upbit fields:
            // - candle_date_time_utc (string)
            // - opening_price, high_price, low_price, trade_price (close)
            // - timestamp (milliseconds - last trade in candle)
            // - candle_acc_trade_volume, candle_acc_trade_price

            let open_time = item.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: Self::get_f64(item, "opening_price").unwrap_or(0.0),
                high: Self::get_f64(item, "high_price").unwrap_or(0.0),
                low: Self::get_f64(item, "low_price").unwrap_or(0.0),
                close: Self::get_f64(item, "trade_price").unwrap_or(0.0),
                volume: Self::get_f64(item, "candle_acc_trade_volume").unwrap_or(0.0),
                quote_volume: Self::get_f64(item, "candle_acc_trade_price"),
                close_time: None,
                trades: None,
            });
        }

        // Upbit returns newest first, reverse to oldest first
        klines.reverse();
        Ok(klines)
    }

    /// Парсить orderbook
    /// Upbit returns array with one orderbook object
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orderbooks".to_string()))?;

        let data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty orderbook array".to_string()))?;

        // Parse orderbook_units array
        let units = data.get("orderbook_units")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing orderbook_units".to_string()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for unit in units {
            // Each unit has: bid_price, bid_size, ask_price, ask_size
            if let (Some(bid_price), Some(bid_size)) = (
                Self::get_f64(unit, "bid_price"),
                Self::get_f64(unit, "bid_size")
            ) {
                bids.push((bid_price, bid_size));
            }

            if let (Some(ask_price), Some(ask_size)) = (
                Self::get_f64(unit, "ask_price"),
                Self::get_f64(unit, "ask_size")
            ) {
                asks.push((ask_price, ask_size));
            }
        }

        Ok(OrderBook {
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            bids,
            asks,
            sequence: None,
        })
    }

    /// Парсить ticker
    /// Upbit returns array of tickers, we take first one
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of tickers".to_string()))?;

        let data = arr.first()
            .ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?;

        Ok(Ticker {
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "trade_price").unwrap_or(0.0),
            bid_price: None, // Upbit ticker doesn't include bid/ask
            ask_price: None,
            high_24h: Self::get_f64(data, "high_price"),
            low_24h: Self::get_f64(data, "low_price"),
            volume_24h: Self::get_f64(data, "acc_trade_volume_24h"),
            quote_volume_24h: Self::get_f64(data, "acc_trade_price_24h"),
            price_change_24h: Self::get_f64(data, "change_price"),
            price_change_percent_24h: Self::get_f64(data, "change_rate")
                .map(|r| r * 100.0), // Convert decimal to percentage
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    /// Parse recent trades
    pub fn parse_recent_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of trades".to_string()))?;

        let mut trades = Vec::with_capacity(arr.len());

        for item in arr {
            let side = match Self::get_str(item, "ask_bid") {
                Some("ASK") => TradeSide::Sell,
                Some("BID") => TradeSide::Buy,
                _ => TradeSide::Buy,
            };

            trades.push(PublicTrade {
                id: item.get("sequential_id")
                    .and_then(|v| v.as_i64())
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                symbol: Self::get_str(item, "market")
                    .or_else(|| Self::get_str(item, "code"))
                    .unwrap_or("")
                    .to_string(),
                price: Self::get_f64(item, "trade_price").unwrap_or(0.0),
                quantity: Self::get_f64(item, "trade_volume").unwrap_or(0.0),
                timestamp: item.get("timestamp")
                    .and_then(|t| t.as_i64())
                    .unwrap_or(0),
                side,
            });
        }

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRADING (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        Self::check_error(response)?;
        Self::parse_order_data(response, symbol)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        // Upbit order fields:
        // - uuid: order ID
        // - side: "bid" (buy) or "ask" (sell)
        // - ord_type: "limit", "price" (market buy), "market" (market sell)
        // - state: "wait", "watch", "done", "cancel"
        // - price, volume, remaining_volume, executed_volume
        // - created_at (ISO 8601 string)

        let side = match Self::get_str(data, "side") {
            Some("ask") => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ord_type") {
            Some("limit") => OrderType::Limit,
            _ => OrderType::Market,
        };

        let status = Self::parse_order_status(data);

        // Parse price (may be string in Upbit)
        let price = data.get("price")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| v.as_f64())
            });

        // Parse volumes (may be strings)
        let quantity = data.get("volume")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let filled_quantity = data.get("executed_volume")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        // Calculate average price
        let average_price = if filled_quantity > 0.0 {
            // TODO: Need trades_count and total value to calculate
            None
        } else {
            None
        };

        Ok(Order {
            id: Self::get_str(data, "uuid").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "identifier").map(String::from),
            symbol: Self::get_str(data, "market").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity,
            average_price,
            commission: None,
            commission_asset: None,
            created_at: 0, // TODO: Parse created_at ISO 8601
            updated_at: None,
            time_in_force: crate::core::TimeInForce::GTC,
        })
    }

    /// Парсить статус ордера
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "state") {
            Some("wait") => OrderStatus::New,
            Some("watch") => OrderStatus::New, // Conditional orders
            Some("done") => {
                // Check if partially or fully filled
                let volume = data.get("volume")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                    .unwrap_or(1.0);
                let executed = data.get("executed_volume")
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                    .unwrap_or(0.0);

                if executed >= volume {
                    OrderStatus::Filled
                } else {
                    OrderStatus::PartiallyFilled
                }
            },
            Some("cancel") => OrderStatus::Canceled,
            _ => OrderStatus::New,
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        arr.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Парсить order ID из create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        Self::check_error(response)?;
        Self::require_str(response, "uuid").map(String::from)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Парсить balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of balances".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "currency").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            // Upbit balance fields (strings):
            // - balance: total balance
            // - locked: balance locked in orders
            let total = item.get("balance")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                .unwrap_or(0.0);

            let locked = item.get("locked")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                .unwrap_or(0.0);

            let free = total - locked;

            balances.push(Balance {
                asset,
                free,
                locked,
                total,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING (DIFFERENT FORMAT!)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        // WebSocket ticker has same format as REST
        let symbol = Self::get_str(data, "code").unwrap_or("").to_string();

        Ok(Ticker {
            symbol,
            last_price: Self::get_f64(data, "trade_price").unwrap_or(0.0),
            bid_price: None,
            ask_price: None,
            high_24h: Self::get_f64(data, "high_price"),
            low_24h: Self::get_f64(data, "low_price"),
            volume_24h: Self::get_f64(data, "acc_trade_volume_24h")
                .or_else(|| Self::get_f64(data, "acc_trade_volume")),
            quote_volume_24h: Self::get_f64(data, "acc_trade_price_24h"),
            price_change_24h: Self::get_f64(data, "change_price"),
            price_change_percent_24h: Self::get_f64(data, "change_rate")
                .map(|r| r * 100.0),
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "ask_bid") {
            Some("ASK") => TradeSide::Sell,
            Some("BID") => TradeSide::Buy,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: data.get("sequential_id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string())
                .unwrap_or_default(),
            symbol: Self::get_str(data, "code")
                .or_else(|| Self::get_str(data, "market"))
                .unwrap_or("")
                .to_string(),
            price: Self::get_f64(data, "trade_price").unwrap_or(0.0),
            quantity: Self::get_f64(data, "trade_volume").unwrap_or(0.0),
            timestamp: data.get("trade_timestamp")
                .or_else(|| data.get("timestamp"))
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            side,
        })
    }

    /// Parse WebSocket orderbook message
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<OrderBook> {
        let units = data.get("orderbook_units")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing orderbook_units".to_string()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for unit in units {
            if let (Some(bid_price), Some(bid_size)) = (
                Self::get_f64(unit, "bid_price"),
                Self::get_f64(unit, "bid_size")
            ) {
                bids.push((bid_price, bid_size));
            }

            if let (Some(ask_price), Some(ask_size)) = (
                Self::get_f64(unit, "ask_price"),
                Self::get_f64(unit, "ask_size")
            ) {
                asks.push((ask_price, ask_size));
            }
        }

        Ok(OrderBook {
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            bids,
            asks,
            sequence: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Upbit /v1/market/all response.
    ///
    /// Response format:
    /// ```json
    /// [{"market":"KRW-BTC","korean_name":"비트코인","english_name":"Bitcoin","market_warning":"NONE"},...]
    /// ```
    /// Symbol format is "QUOTE-BASE" (e.g. "KRW-BTC" means base=BTC, quote=KRW)
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let items = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        let mut symbols = Vec::with_capacity(items.len());

        for item in items {
            let market = match item.get("market").and_then(|v| v.as_str()) {
                Some(m) => m,
                None => continue,
            };

            // Upbit format: "QUOTE-BASE" e.g. "KRW-BTC"
            let parts: Vec<&str> = market.splitn(2, '-').collect();
            if parts.len() != 2 {
                continue;
            }

            let quote_asset = parts[0].to_string();
            let base_asset = parts[1].to_string();

            // Filter caution symbols if they have CAUTION warning (still tradeable but flag it)
            // We include all by default
            let status = "TRADING".to_string();

            symbols.push(SymbolInfo {
                symbol: market.to_string(),
                base_asset,
                quote_asset,
                status,
                price_precision: 8,
                quantity_precision: 8,
                min_quantity: None,
                max_quantity: None,
                step_size: None,
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
    fn test_parse_price() {
        let response = json!([
            {
                "market": "SGD-BTC",
                "trade_price": 67300.0
            }
        ]);

        let price = UpbitParser::parse_price(&response).unwrap();
        assert_eq!(price, 67300.0);
    }

    #[test]
    fn test_parse_error() {
        let response = json!({
            "error": {
                "name": "invalid_signature",
                "message": "JWT signature does not match"
            }
        });

        let result = UpbitParser::check_error(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_order_status() {
        let data = json!({"state": "wait"});
        assert_eq!(UpbitParser::parse_order_status(&data), OrderStatus::New);

        let data = json!({"state": "done", "volume": "1.0", "executed_volume": "1.0"});
        assert_eq!(UpbitParser::parse_order_status(&data), OrderStatus::Filled);

        let data = json!({"state": "cancel"});
        assert_eq!(UpbitParser::parse_order_status(&data), OrderStatus::Canceled);
    }
}
