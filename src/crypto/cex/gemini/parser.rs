//! # Gemini Response Parser
//!
//! Парсинг JSON ответов от Gemini API.
//!
//! **ВАЖНО**: REST и WebSocket парсеры РАЗНЫЕ - смотри research/websocket.md

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult, AccountType,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, StreamEvent, TradeSide,
    OrderUpdateEvent, SymbolInfo, FeeInfo,
    UserTrade,
};

/// Парсер ответов Gemini API
pub struct GeminiParser;

impl GeminiParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(result) = response.get("result").and_then(|r| r.as_str()) {
            if result == "error" {
                let reason = response.get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Unknown");
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("");
                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("{}: {}", reason, message),
                });
            }
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

    /// Парсить i64 из поля
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA (REST)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить symbols list
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of symbols".to_string()))?;

        Ok(arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
    }

    /// Парсить ticker (v1 или v2)
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        // V2 format has "symbol" field, V1 doesn't
        let is_v2 = response.get("symbol").is_some();

        // V2: compute price change percent from open and close
        let (volume_24h, price_change_percent_24h) = if is_v2 {
            let open = Self::get_f64(response, "open");
            let close = Self::get_f64(response, "close");
            let pct = match (open, close) {
                (Some(o), Some(c)) if o != 0.0 => Some((c - o) / o * 100.0),
                _ => None,
            };
            // V2 API does not provide a volume field
            (None, pct)
        } else {
            // V1: extract base volume from the volume object (skip "timestamp" key)
            let vol = response.get("volume")
                .and_then(|v| v.as_object())
                .and_then(|obj| {
                    obj.iter()
                        .filter(|(k, _)| k.as_str() != "timestamp")
                        .find_map(|(_, val)| Self::parse_f64(val))
                });
            (vol, None)
        };

        Ok(Ticker {
            symbol: Self::get_str(response, "symbol")
                .unwrap_or(symbol)
                .to_string(),
            last_price: Self::get_f64(response, "last")
                .or_else(|| Self::get_f64(response, "close"))
                .unwrap_or(0.0),
            bid_price: Self::get_f64(response, "bid"),
            ask_price: Self::get_f64(response, "ask"),
            high_24h: Self::get_f64(response, "high"),
            low_24h: Self::get_f64(response, "low"),
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h,
            timestamp: response.get("volume")
                .and_then(|v| v.get("timestamp"))
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            response.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let obj = level.as_object()?;
                            let price = obj.get("price").and_then(Self::parse_f64)?;
                            let amount = obj.get("amount").and_then(Self::parse_f64)?;
                            Some((price, amount))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: 0, // Gemini doesn't provide orderbook timestamp
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
        })
    }

    /// Парсить trades
    pub fn parse_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of trades".to_string()))?;

        let mut trades = Vec::new();

        for item in arr {
            let side = match Self::get_str(item, "type").unwrap_or("buy") {
                "sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            };

            trades.push(PublicTrade {
                id: Self::get_i64(item, "tid")
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                symbol: String::new(), // Not provided in response
                price: Self::require_f64(item, "price")?,
                quantity: Self::get_f64(item, "amount").unwrap_or(0.0),
                side,
                timestamp: Self::get_i64(item, "timestampms").unwrap_or(0),
            });
        }

        Ok(trades)
    }

    /// Парсить candles
    pub fn parse_candles(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of candles".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Candle is not an array".to_string()))?;

            if candle.len() < 6 {
                continue;
            }

            // Gemini format: [timestamp, open, high, low, close, volume]
            let open_time = Self::parse_f64(&candle[0])
                .map(|t| t as i64)
                .unwrap_or(0);

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

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_error(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(response, "symbol").unwrap_or("").to_string(),
            rate: Self::require_f64(response, "funding_amount")?,
            next_funding_time: Self::get_i64(response, "next_funding_time"),
            timestamp: Self::get_i64(response, "funding_time").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING (REST)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        Self::check_error(response)?;
        Self::parse_order_data(response)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy").to_lowercase().as_str() {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("exchange limit") {
            "exchange market" | "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "order_id")
                .or_else(|| Self::get_str(data, "id"))
                .unwrap_or("")
                .to_string(),
            client_order_id: Self::get_str(data, "client_order_id").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "stop_price"),
            quantity: Self::get_f64(data, "original_amount")
                .or_else(|| Self::get_f64(data, "amount"))
                .unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "executed_amount").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avg_execution_price")
                .filter(|&p| p > 0.0),
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "timestampms").unwrap_or(0),
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    fn parse_order_status(data: &Value) -> OrderStatus {
        let is_live = data.get("is_live").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_cancelled = data.get("is_cancelled").and_then(|v| v.as_bool()).unwrap_or(false);
        let executed = Self::get_f64(data, "executed_amount").unwrap_or(0.0);
        let _remaining = Self::get_f64(data, "remaining_amount").unwrap_or(0.0);

        if is_cancelled {
            if executed > 0.0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Canceled
            }
        } else if is_live {
            if executed > 0.0 {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::New
            }
        } else {
            // Not live and not cancelled = filled
            OrderStatus::Filled
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        arr.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Парсить order ID из create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        Self::check_error(response)?;

        Self::get_str(response, "order_id")
            .or_else(|| Self::get_str(response, "id"))
            .map(String::from)
            .ok_or_else(|| ExchangeError::Parse("Missing order_id".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT (REST)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of balances".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "currency").unwrap_or("").to_string();
            if asset.is_empty() {
                continue;
            }

            let amount = Self::get_f64(item, "amount").unwrap_or(0.0);
            let available = Self::get_f64(item, "available").unwrap_or(0.0);
            let locked = amount - available;

            balances.push(Balance {
                asset,
                free: available,
                locked,
                total: amount,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS (REST)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        let mut positions = Vec::new();

        for item in arr {
            if let Some(pos) = Self::parse_position_data(item) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    /// Парсить single position
    pub fn parse_position(response: &Value) -> ExchangeResult<Position> {
        Self::check_error(response)?;

        Self::parse_position_data(response)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
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
            entry_price: Self::get_f64(data, "average_cost").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl: Self::get_f64(data, "unrealised_pnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realised_pnl"),
            leverage: 1, // Gemini doesn't expose leverage directly
            liquidation_price: Self::get_f64(data, "estimated_liquidation_price"),
            margin: Self::get_f64(data, "initial_margin"),
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING (DIFFERENT FROM REST!)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket L2 update message
    ///
    /// Gemini L2 changes format: `[["buy", "50000.00", "1.5"], ["sell", "50001.00", "0.8"]]`
    /// Each change is an array of `[side, price, quantity]` where all elements are strings.
    pub fn parse_ws_l2_update(data: &Value) -> ExchangeResult<StreamEvent> {
        let changes = data.get("changes")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing changes array".to_string()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for change in changes {
            let arr = change.as_array()
                .ok_or_else(|| ExchangeError::Parse("Change is not array".to_string()))?;

            if arr.len() < 3 {
                continue;
            }

            // arr[0] is a plain string value like "buy" or "sell", not an object
            let side = arr[0].as_str()
                .ok_or_else(|| ExchangeError::Parse("Change side is not a string".to_string()))?;
            let price = Self::parse_f64(&arr[1]).unwrap_or(0.0);
            let quantity = Self::parse_f64(&arr[2]).unwrap_or(0.0);

            if side == "buy" {
                bids.push((price, quantity));
            } else {
                asks.push((price, quantity));
            }
        }

        Ok(StreamEvent::OrderbookDelta {
            bids,
            asks,
            timestamp: 0,
        })
    }

    /// Extract the most recent trade from a WebSocket L2 update message.
    ///
    /// Gemini `l2_updates` messages carry an optional `trades` array with executed
    /// trades that happened since the last update:
    /// ```json
    /// {"type":"l2_updates","symbol":"BTCUSD","trades":[{"type":"trade","tid":12345,
    ///   "price":"50000.00","amount":"0.5","makerSide":"bid","timestampms":1234567890}]}
    /// ```
    /// Returns `StreamEvent::Trade` built from the last entry of that array.
    pub fn parse_ws_l2_trade(data: &Value) -> ExchangeResult<StreamEvent> {
        let trades = data.get("trades")
            .and_then(|t| t.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing trades array in l2_updates".to_string()))?;

        let trade = trades.last()
            .ok_or_else(|| ExchangeError::Parse("Empty trades array in l2_updates".to_string()))?;

        let price = Self::require_f64(trade, "price")?;
        let quantity = Self::get_f64(trade, "amount")
            .or_else(|| Self::get_f64(trade, "quantity"))
            .unwrap_or(0.0);
        let timestamp = Self::get_i64(trade, "timestampms")
            .or_else(|| Self::get_i64(trade, "timestamp"))
            .unwrap_or(0);
        let id = Self::get_i64(trade, "tid")
            .or_else(|| Self::get_i64(trade, "event_id"))
            .map(|i| i.to_string())
            .unwrap_or_default();
        let symbol = data.get("symbol")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        // "makerSide":"bid" means the maker was the buyer → taker sold → Sell trade.
        // "makerSide":"ask" means the maker was the seller → taker bought → Buy trade.
        let side = match trade.get("makerSide").and_then(|s| s.as_str()) {
            Some("bid") => TradeSide::Sell,
            Some("ask") => TradeSide::Buy,
            _ => match Self::get_str(trade, "side").unwrap_or("buy") {
                "sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            },
        };

        Ok(StreamEvent::Trade(PublicTrade {
            id,
            symbol,
            price,
            quantity,
            side,
            timestamp,
        }))
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_i64(data, "event_id")
                .map(|i| i.to_string())
                .unwrap_or_default(),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            price: Self::require_f64(data, "price")?,
            quantity: Self::get_f64(data, "quantity").unwrap_or(0.0),
            side,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket candle update
    pub fn parse_ws_candle(data: &Value) -> ExchangeResult<Kline> {
        let changes = data.get("changes")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing changes array".to_string()))?;

        if changes.is_empty() {
            return Err(ExchangeError::Parse("Empty changes array".to_string()));
        }

        let candle = changes[0].as_array()
            .ok_or_else(|| ExchangeError::Parse("Candle is not array".to_string()))?;

        if candle.len() < 6 {
            return Err(ExchangeError::Parse("Invalid candle format".to_string()));
        }

        Ok(Kline {
            open_time: Self::parse_f64(&candle[0]).map(|t| t as i64).unwrap_or(0),
            open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
            high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
            low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
            close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
            volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
            quote_volume: None,
            close_time: None,
            trades: None,
        })
    }

    /// Parse WebSocket order event
    pub fn parse_ws_order_event(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let event_type = Self::require_str(data, "type")?;

        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "order_type").unwrap_or("exchange limit") {
            "exchange market" | "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match event_type {
            "accepted" => OrderStatus::New,
            "booked" => OrderStatus::New,
            "fill" => {
                let remaining = Self::get_f64(data, "remaining_amount").unwrap_or(0.0);
                if remaining == 0.0 {
                    OrderStatus::Filled
                } else {
                    OrderStatus::PartiallyFilled
                }
            }
            "cancelled" => OrderStatus::Canceled,
            "rejected" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        };

        // Extract fill info if present
        let (last_fill_price, last_fill_quantity) = if let Some(fill) = data.get("fill") {
            (
                Self::get_f64(fill, "price"),
                Self::get_f64(fill, "amount"),
            )
        } else {
            (None, None)
        };

        Ok(OrderUpdateEvent {
            order_id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "client_order_id").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            quantity: Self::get_f64(data, "original_amount").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "executed_amount").unwrap_or(0.0),
            average_price: None,
            last_fill_price,
            last_fill_quantity,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp: Self::get_i64(data, "timestampms").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Gemini /v1/symbols/details/{symbol} response.
    ///
    /// Details response format:
    /// ```json
    /// {"symbol":"BTCUSD","base_currency":"BTC","quote_currency":"USD","tick_size":1e-8,"quote_increment":0.01,"min_order_size":"0.00001","status":"open","wrap_enabled":false}
    /// ```
    pub fn parse_symbol_details(response: &Value, symbol_lower: &str, account_type: AccountType) -> Option<SymbolInfo> {
        let status = response.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status != "open" && !status.is_empty() {
            return None;
        }

        let base_asset = response.get("base_currency")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let quote_asset = response.get("quote_currency")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if base_asset.is_empty() || quote_asset.is_empty() {
            return None;
        }

        let symbol = response.get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(symbol_lower)
            .to_string();

        // tick_size is like 1e-8, count decimal places
        let price_precision = response.get("quote_increment")
            .and_then(|v| v.as_f64())
            .map(|inc| {
                if inc <= 0.0 { 8u8 }
                else { (-inc.log10().ceil()) as u8 }
            })
            .unwrap_or(8);

        let quantity_precision = response.get("tick_size")
            .and_then(|v| v.as_f64())
            .map(|inc| {
                if inc <= 0.0 { 8u8 }
                else { (-inc.log10().ceil()) as u8 }
            })
            .unwrap_or(8);

        let min_quantity = response.get("min_order_size")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| response.get("min_order_size").and_then(|v| v.as_f64()));

        // quote_increment is the minimum price increment (tick_size for price)
        let tick_size = response.get("quote_increment")
            .and_then(|v| v.as_f64())
            .filter(|&v| v > 0.0);

        // tick_size field in Gemini is the minimum quantity increment (step_size)
        let step_size = response.get("tick_size")
            .and_then(|v| v.as_f64())
            .filter(|&v| v > 0.0);

        Some(SymbolInfo {
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
            account_type,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ORDER HISTORY / FEES (REST)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse past trades response from /v1/mytrades.
    ///
    /// Each entry format:
    /// ```json
    /// {"price":"50000.00","amount":"0.5","timestamp":1234567,"timestampms":1234567890,
    ///  "type":"Buy","aggressor":true,"fee_currency":"USD","fee_amount":"1.25",
    ///  "tid":12345,"order_id":"98765","exchange":"gemini","is_auction_fill":false}
    /// ```
    pub fn parse_past_trades(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of past trades".to_string()))?;

        let mut orders = Vec::with_capacity(arr.len());

        for item in arr {
            let side = match Self::get_str(item, "type").unwrap_or("Buy").to_lowercase().as_str() {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let price = Self::get_f64(item, "price");
            let quantity = Self::get_f64(item, "amount").unwrap_or(0.0);
            let commission = Self::get_f64(item, "fee_amount");
            let commission_asset = Self::get_str(item, "fee_currency").map(String::from);

            orders.push(Order {
                id: Self::get_str(item, "order_id")
                    .unwrap_or_else(|| Self::get_str(item, "tid").unwrap_or(""))
                    .to_string(),
                client_order_id: None,
                symbol: Self::get_str(item, "symbol").unwrap_or("").to_string(),
                side,
                order_type: OrderType::Limit { price: price.unwrap_or(0.0) },
                status: OrderStatus::Filled,
                price,
                stop_price: None,
                quantity,
                filled_quantity: quantity,
                average_price: price,
                commission,
                commission_asset,
                created_at: Self::get_i64(item, "timestampms").unwrap_or(0),
                updated_at: None,
                time_in_force: crate::core::TimeInForce::Gtc,
            });
        }

        Ok(orders)
    }

    /// Parse user trade fills from Gemini `POST /v1/mytrades`.
    ///
    /// Response: array of trade fill objects.
    /// `timestamp` is in **seconds**; `timestampms` is in milliseconds.
    ///
    /// ```json
    /// [{"tid":123,"order_id":"456","symbol":"btcusd","type":"Buy",
    ///   "price":"50000.00","amount":"0.001","fee_amount":"0.50",
    ///   "fee_currency":"USD","is_maker":true,"timestamp":1672531200,
    ///   "timestampms":1672531200000}]
    /// ```
    pub fn parse_user_trades(response: &Value, end_time_ms: Option<u64>) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of user trades".to_string()))?;

        let mut trades = Vec::with_capacity(arr.len());

        for item in arr {
            // Gemini timestamp is in seconds; timestampms in ms
            let timestamp_ms = Self::get_i64(item, "timestampms")
                .unwrap_or_else(|| {
                    Self::get_i64(item, "timestamp")
                        .map(|s| s * 1000)
                        .unwrap_or(0)
                });

            // Apply end_time filter (start_time is filtered server-side via the `timestamp` param)
            if let Some(et) = end_time_ms {
                if (timestamp_ms as u64) > et {
                    continue;
                }
            }

            let id = item.get("tid")
                .and_then(|v| v.as_i64().map(|n| n.to_string()))
                .or_else(|| Self::get_str(item, "tid").map(String::from))
                .unwrap_or_default();

            let order_id = Self::get_str(item, "order_id")
                .unwrap_or("")
                .to_string();

            let symbol = Self::get_str(item, "symbol")
                .unwrap_or("")
                .to_string();

            // "type": "Buy" or "Sell"
            let side = match Self::get_str(item, "type").unwrap_or("Buy").to_lowercase().as_str() {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let price = Self::get_f64(item, "price").unwrap_or(0.0);
            let quantity = Self::get_f64(item, "amount").unwrap_or(0.0);
            let commission = Self::get_f64(item, "fee_amount").unwrap_or(0.0);
            let commission_asset = Self::get_str(item, "fee_currency")
                .unwrap_or("USD")
                .to_string();

            let is_maker = item.get("is_maker")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            trades.push(UserTrade {
                id,
                order_id,
                symbol,
                side,
                price,
                quantity,
                commission,
                commission_asset,
                is_maker,
                timestamp: timestamp_ms,
            });
        }

        Ok(trades)
    }

    /// Parse notional volume response from /v1/notionalvolume for fee rates.
    ///
    /// Response format:
    /// ```json
    /// {"web_maker_fee_bps":25,"web_taker_fee_bps":35,"web_auction_fee_bps":25,
    ///  "api_maker_fee_bps":10,"api_taker_fee_bps":35,"api_auction_fee_bps":20,
    ///  "fix_maker_fee_bps":10,"fix_taker_fee_bps":35,"fix_auction_fee_bps":20,
    ///  "block_maker_fee_bps":0,"block_taker_fee_bps":0,
    ///  "notional_30d_volume":1234567.89,"last_updated_ms":1234567890000}
    /// ```
    /// API fees are in basis points; divide by 10000 to get decimal rate.
    pub fn parse_notional_volume_fees(response: &Value, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Self::check_error(response)?;

        // API maker/taker fees are in basis points (1 bps = 0.01%)
        let maker_bps = response.get("api_maker_fee_bps")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0); // default 10 bps = 0.10%
        let taker_bps = response.get("api_taker_fee_bps")
            .and_then(|v| v.as_f64())
            .unwrap_or(35.0); // default 35 bps = 0.35%

        Ok(FeeInfo {
            maker_rate: maker_bps / 10_000.0,
            taker_rate: taker_bps / 10_000.0,
            symbol: symbol.map(String::from),
            tier: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_error() {
        let error_response = json!({
            "result": "error",
            "reason": "InvalidNonce",
            "message": "Nonce must be increasing"
        });

        assert!(GeminiParser::check_error(&error_response).is_err());

        let success_response = json!({"price": "50000.00"});
        assert!(GeminiParser::check_error(&success_response).is_ok());
    }

    #[test]
    fn test_parse_symbols() {
        let response = json!(["btcusd", "ethusd", "btcgusdperp"]);
        let symbols = GeminiParser::parse_symbols(&response).unwrap();

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "btcusd");
        assert_eq!(symbols[2], "btcgusdperp");
    }

    #[test]
    fn test_parse_ticker_v1() {
        let response = json!({
            "bid": "50000.00",
            "ask": "50001.00",
            "last": "50000.50",
            "volume": {
                "BTC": "1234.56",
                "USD": "61728000.00",
                "timestamp": 1640000000000i64
            }
        });

        let ticker = GeminiParser::parse_ticker(&response, "btcusd").unwrap();
        assert!((ticker.last_price - 50000.50).abs() < f64::EPSILON);
        assert_eq!(ticker.bid_price, Some(50000.00));
        assert_eq!(ticker.ask_price, Some(50001.00));
    }

    #[test]
    fn test_parse_order() {
        let response = json!({
            "order_id": "987654321",
            "symbol": "btcusd",
            "side": "buy",
            "type": "exchange limit",
            "price": "50000.00",
            "original_amount": "0.5",
            "executed_amount": "0.2",
            "remaining_amount": "0.3",
            "is_live": true,
            "is_cancelled": false,
            "timestampms": 1640000000000i64
        });

        let order = GeminiParser::parse_order(&response).unwrap();
        assert_eq!(order.id, "987654321");
        assert_eq!(order.symbol, "btcusd");
        assert_eq!(order.side, OrderSide::Buy);
        assert!(matches!(order.order_type, OrderType::Limit { .. }));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
    }

    #[test]
    fn test_parse_balances() {
        let response = json!([
            {
                "currency": "BTC",
                "amount": "1.5",
                "available": "1.0"
            },
            {
                "currency": "USD",
                "amount": "10000.00",
                "available": "9500.00"
            }
        ]);

        let balances = GeminiParser::parse_balances(&response).unwrap();
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0].asset, "BTC");
        assert_eq!(balances[0].free, 1.0);
        assert_eq!(balances[0].locked, 0.5);
        assert_eq!(balances[0].total, 1.5);
    }
}
