//! # Gate.io Response Parser
//!
//! JSON response parsing for Gate.io API V4.
//!
//! ## Key Differences from Other Exchanges
//!
//! 1. **No response wrapper**: Data returned directly (no `{" data": ...}`)
//! 2. **Klines order**: `[time, volume, close, high, low, open, quote_volume]` (DIFFERENT!)
//! 3. **All numeric values are strings**
//! 4. **Timestamps**: Seconds for most, milliseconds for orderbook

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, TradeSide,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    BalanceChangeReason, PositionChangeReason,
    CancelAllResponse, OrderResult,
    UserTrade,
};

/// Parser for Gate.io API responses
pub struct GateioParser;

impl GateioParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse f64 from string or number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Parse f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Parse required f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Parse required string
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Check if response is an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(label) = response.get("label").and_then(|v| v.as_str()) {
            let message = response.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: -1,
                message: format!("{}: {}", label, message),
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ticker (single)
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        // Gate.io returns array even for single ticker
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of tickers".to_string()))?;

        if arr.is_empty() {
            return Err(ExchangeError::Parse("Empty ticker array".to_string()));
        }

        let data = &arr[0];
        Self::parse_ticker_data(data)
    }

    /// Parse ticker data object
    fn parse_ticker_data(data: &Value) -> ExchangeResult<Ticker> {
        // Spot fields: currency_pair, last, lowest_ask, highest_bid
        // Futures fields: contract, last, lowest_ask, highest_bid, mark_price, index_price

        let symbol = Self::get_str(data, "currency_pair")
            .or_else(|| Self::get_str(data, "contract"))
            .unwrap_or("")
            .to_string();

        Ok(Ticker {
            symbol,
            last_price: Self::get_f64(data, "last").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "highest_bid"),
            ask_price: Self::get_f64(data, "lowest_ask"),
            high_24h: Self::get_f64(data, "high_24h"),
            low_24h: Self::get_f64(data, "low_24h"),
            volume_24h: Self::get_f64(data, "base_volume")
                .or_else(|| Self::get_f64(data, "volume_24h_base")),
            quote_volume_24h: Self::get_f64(data, "quote_volume")
                .or_else(|| Self::get_f64(data, "volume_24h_quote")),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "change_percentage"),
            timestamp: 0, // Gate.io doesn't include timestamp in ticker
        })
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            response.get(key)
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

        Ok(OrderBook {
            timestamp: response.get("current")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: response.get("id")
                .and_then(|s| s.as_i64())
                .map(|n| n.to_string()),
        })
    }

    /// Parse klines
    ///
    /// # CRITICAL: Gate.io Kline Order
    /// Gate.io uses: `[time, volume, close, high, low, open, quote_volume]`
    /// Most exchanges: `[time, open, high, low, close, volume]`
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of klines".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 6 {
                continue;
            }

            // Gate.io format: [time, volume, close, high, low, open, quote_volume]
            //                   0      1       2      3    4     5         6
            let open_time = candle[0].as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0) * 1000; // seconds to ms

            klines.push(Kline {
                open_time,
                open: Self::parse_f64(&candle[5]).unwrap_or(0.0),     // index 5
                close: Self::parse_f64(&candle[2]).unwrap_or(0.0),    // index 2
                high: Self::parse_f64(&candle[3]).unwrap_or(0.0),     // index 3
                low: Self::parse_f64(&candle[4]).unwrap_or(0.0),      // index 4
                volume: Self::parse_f64(&candle[1]).unwrap_or(0.0),   // index 1
                quote_volume: candle.get(6).and_then(Self::parse_f64), // index 6
                close_time: None,
                trades: None,
            });
        }

        // Gate.io returns oldest first (ascending timestamps) — no reverse needed
        Ok(klines)
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of funding rates".to_string()))?;

        if arr.is_empty() {
            return Err(ExchangeError::Parse("Empty funding rate array".to_string()));
        }

        let data = &arr[0]; // Latest funding rate

        Ok(FundingRate {
            symbol: String::new(), // Not included in response
            rate: Self::require_f64(data, "r")?,
            next_funding_time: None,
            timestamp: data.get("t")
                .and_then(|t| t.as_i64())
                .map(|t| t * 1000) // seconds to ms
                .unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from Gate.io response
    ///
    /// Spot: direct array [{ id, base, quote, trade_status, min_base_amount, min_quote_amount,
    ///                       amount_precision, precision }]
    /// Futures: direct array [{ name, underlying, settle, type, order_size_min, order_size_max,
    ///                          order_price_round, quanto_multiplier }]
    ///
    /// Filters to active/tradable symbols only.
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of symbols".to_string()))?;

        let symbols = arr.iter()
            .filter_map(|item| {
                // Spot uses "id"; futures uses "name"
                let symbol = Self::get_str(item, "id")
                    .or_else(|| Self::get_str(item, "name"))?
                    .to_string();

                // Spot fields
                let base_asset = Self::get_str(item, "base")
                    .or_else(|| Self::get_str(item, "underlying"))
                    .unwrap_or("")
                    .to_string();
                let quote_asset = Self::get_str(item, "quote")
                    .or_else(|| Self::get_str(item, "settle"))
                    .unwrap_or("")
                    .to_string();

                // Status: spot uses trade_status ("tradable"/"untradable"), futures uses "in_delisting"
                let trade_status = Self::get_str(item, "trade_status");
                let in_delisting = item.get("in_delisting")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if let Some(ts) = trade_status {
                    if ts != "tradable" {
                        return None;
                    }
                }
                if in_delisting {
                    return None;
                }

                let status = "TRADING".to_string();

                // Spot: amount_precision (integer) for quantity, precision for price
                let qty_precision_int = item.get("amount_precision")
                    .and_then(|v| v.as_i64())
                    .map(|p| p as u8);
                let price_precision_int = item.get("precision")
                    .and_then(|v| v.as_i64())
                    .map(|p| p as u8);

                // Futures: order_price_round (tick size string)
                let price_tick = Self::get_f64(item, "order_price_round");
                let price_precision = price_precision_int.unwrap_or_else(|| {
                    price_tick.map(|t| {
                        let s = format!("{:.10}", t);
                        let trimmed = s.trim_end_matches('0');
                        if let Some(dot_pos) = trimmed.find('.') {
                            (trimmed.len() - dot_pos - 1) as u8
                        } else {
                            0u8
                        }
                    }).unwrap_or(8)
                });

                let quantity_precision = qty_precision_int.unwrap_or(8);

                // Min/max quantity
                let min_quantity = Self::get_f64(item, "min_base_amount")
                    .or_else(|| item.get("order_size_min").and_then(|v| v.as_i64()).map(|v| v as f64));
                let max_quantity = item.get("order_size_max")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as f64);

                // tick_size: futures provides order_price_round as a decimal string
                // (e.g. "0.1"), which is a real tick size. Spot only provides an
                // integer precision digit count, so PrecisionCache handles that
                // fallback — leave tick_size as None for spot.
                let tick_size = price_tick;

                Some(crate::core::types::SymbolInfo {
                    symbol,
                    base_asset,
                    quote_asset,
                    status,
                    price_precision,
                    quantity_precision,
                    min_quantity,
                    max_quantity,
                    tick_size,
                    step_size: None,
                    min_notional: Self::get_f64(item, "min_quote_amount"),
                })
            })
            .collect();

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        Self::check_error(response)?;
        Self::parse_order_data(response, symbol)
    }

    /// Parse order data object
    fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("limit") {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        let amount = Self::get_f64(data, "amount")
            .or_else(|| Self::get_f64(data, "size").map(|s| s.abs()))
            .unwrap_or(0.0);

        let left = Self::get_f64(data, "left").unwrap_or(0.0);
        let filled_quantity = amount - left;

        Ok(Order {
            id: Self::get_str(data, "id")
                .unwrap_or("")
                .to_string(),
            client_order_id: Self::get_str(data, "text").map(String::from),
            symbol: Self::get_str(data, "currency_pair")
                .or_else(|| Self::get_str(data, "contract"))
                .unwrap_or(symbol)
                .to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: None,
            quantity: amount,
            filled_quantity,
            average_price: Self::get_f64(data, "fill_price")
                .or_else(|| {
                    let filled_total = Self::get_f64(data, "filled_total")?;
                    if filled_quantity > 0.0 {
                        Some(filled_total / filled_quantity)
                    } else {
                        None
                    }
                }),
            commission: Self::get_f64(data, "fee"),
            commission_asset: Self::get_str(data, "fee_currency").map(String::from),
            created_at: Self::get_str(data, "create_time")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|t| t * 1000) // seconds to ms
                .or_else(|| data.get("create_time").and_then(|v| v.as_f64()).map(|t| (t * 1000.0) as i64))
                .unwrap_or(0),
            updated_at: Self::get_str(data, "update_time")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|t| t * 1000), // seconds to ms
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("open") {
            "open" => OrderStatus::New,
            "closed" => OrderStatus::Filled,
            "cancelled" => OrderStatus::Canceled,
            _ => OrderStatus::New,
        }
    }

    /// Parse list of orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        arr.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Parse user trade fills from `/spot/my_trades` (spot) or
    /// `/futures/{settle}/my_trades` (futures).
    ///
    /// Spot response fields: id (string), order_id (string), currency_pair,
    ///   side ("buy"/"sell"), role ("maker"/"taker"), amount, price,
    ///   fee, fee_currency, create_time (seconds string), create_time_ms.
    ///
    /// Futures response fields: id (integer), order_id (string), contract,
    ///   size (integer, negative = sell), price, role ("maker"/"taker"),
    ///   fee (string, negative = rebate), create_time (float seconds).
    pub fn parse_user_trades(response: &Value, is_futures: bool) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of user trades".to_string()))?;

        arr.iter()
            .map(|item| {
                // Trade ID: spot = string, futures = integer
                let id = item.get("id")
                    .map(|v| {
                        v.as_str()
                            .map(String::from)
                            .or_else(|| v.as_i64().map(|n| n.to_string()))
                            .unwrap_or_default()
                    })
                    .ok_or_else(|| ExchangeError::Parse("Missing 'id' in trade".to_string()))?;

                // Order ID: string on both
                let order_id = Self::get_str(item, "order_id")
                    .unwrap_or("")
                    .to_string();

                // Symbol: spot = currency_pair, futures = contract
                let symbol = Self::get_str(item, "currency_pair")
                    .or_else(|| Self::get_str(item, "contract"))
                    .unwrap_or("")
                    .to_string();

                // Side
                let side = if is_futures {
                    // Futures: derive from size sign (positive = buy, negative = sell)
                    let size = item.get("size")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1);
                    if size < 0 { OrderSide::Sell } else { OrderSide::Buy }
                } else {
                    // Spot: "side" field ("buy"/"sell")
                    match Self::get_str(item, "side").unwrap_or("buy") {
                        "sell" => OrderSide::Sell,
                        _ => OrderSide::Buy,
                    }
                };

                // Price
                let price = Self::require_f64(item, "price")?;

                // Quantity: spot = "amount" (string), futures = abs("size") (integer)
                let quantity = if is_futures {
                    item.get("size")
                        .and_then(|v| v.as_i64())
                        .map(|s| s.unsigned_abs() as f64)
                        .unwrap_or(0.0)
                } else {
                    Self::get_f64(item, "amount").unwrap_or(0.0)
                };

                // Commission: may be negative (rebate) on futures — take absolute value
                let commission = Self::get_f64(item, "fee")
                    .map(|f| f.abs())
                    .unwrap_or(0.0);

                // Commission asset: spot = fee_currency, futures = settle currency
                let commission_asset = Self::get_str(item, "fee_currency")
                    .or_else(|| Self::get_str(item, "currency"))
                    .unwrap_or("USDT")
                    .to_string();

                // Maker flag: "role" = "maker" | "taker"
                let is_maker = Self::get_str(item, "role")
                    .map(|r| r == "maker")
                    .unwrap_or(false);

                // Timestamp: prefer milliseconds field, fall back to seconds * 1000
                let timestamp = item.get("create_time_ms")
                    .and_then(|v| {
                        v.as_str()
                            .and_then(|s| s.parse::<i64>().ok())
                            .or_else(|| v.as_i64())
                    })
                    .unwrap_or_else(|| {
                        // create_time may be string (spot) or float (futures)
                        let secs = item.get("create_time")
                            .and_then(|v| {
                                v.as_str()
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .or_else(|| v.as_f64())
                            })
                            .unwrap_or(0.0);
                        (secs * 1000.0) as i64
                    });

                Ok(UserTrade {
                    id,
                    order_id,
                    symbol,
                    side,
                    price,
                    quantity,
                    commission,
                    commission_asset,
                    is_maker,
                    timestamp,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse spot balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of accounts".to_string()))?;

        let mut balances = Vec::new();

        for item in arr {
            let asset = Self::get_str(item, "currency").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "available").unwrap_or(0.0);
            let locked = Self::get_f64(item, "locked").unwrap_or(0.0);

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    /// Parse futures account
    pub fn parse_futures_account(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(response)?;

        let currency = Self::get_str(response, "currency").unwrap_or("USDT").to_string();
        let available = Self::get_f64(response, "available").unwrap_or(0.0);
        let total = Self::get_f64(response, "total").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: currency,
            free: available,
            locked: total - available,
            total,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
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

    /// Parse single position
    pub fn parse_position(response: &Value) -> ExchangeResult<Position> {
        Self::check_error(response)?;
        Self::parse_position_data(response)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "contract")?.to_string();

        // Size can be integer or float
        let size = data.get("size")
            .and_then(|v| v.as_i64())
            .map(|i| i as f64)
            .or_else(|| Self::get_f64(data, "size"))
            .unwrap_or(0.0);

        // Skip empty positions
        if size.abs() < f64::EPSILON {
            return None;
        }

        let side = if size > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Position {
            symbol,
            side,
            quantity: size.abs(),
            entry_price: Self::get_f64(data, "entry_price").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl: Self::get_f64(data, "unrealised_pnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realised_pnl"),
            leverage: Self::get_str(data, "leverage")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1),
            liquidation_price: Self::get_f64(data, "liq_price"),
            margin: Self::get_f64(data, "margin"),
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
        Self::parse_ticker_data(data)
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "side").unwrap_or("buy") {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "id").unwrap_or("").to_string(),
            symbol: Self::get_str(data, "currency_pair")
                .or_else(|| Self::get_str(data, "contract"))
                .unwrap_or("")
                .to_string(),
            price: Self::require_f64(data, "price")?,
            quantity: Self::get_f64(data, "amount").unwrap_or(0.0),
            side,
            timestamp: data.get("create_time")
                .and_then(|t| t.as_i64())
                .unwrap_or(0) * 1000, // seconds to ms
        })
    }

    /// Parse WebSocket order update message
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let order = Self::parse_order_data(data, "")?;

        Ok(OrderUpdateEvent {
            order_id: order.id,
            client_order_id: order.client_order_id,
            symbol: order.symbol,
            side: order.side,
            order_type: order.order_type,
            status: order.status,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            average_price: order.average_price,
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp: order.created_at,
        })
    }

    /// Parse WebSocket balance update message
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
        let asset = Self::get_str(data, "currency").unwrap_or("").to_string();
        let free = Self::get_f64(data, "available").unwrap_or(0.0);
        let locked = Self::get_f64(data, "locked").unwrap_or(0.0);
        let total = free + locked;

        Ok(BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta: None,
            reason: Some(BalanceChangeReason::Other),
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    /// Parse WebSocket position update message
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
        let pos = Self::parse_position_data(data)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))?;

        Ok(PositionUpdateEvent {
            symbol: pos.symbol,
            side: pos.side,
            quantity: pos.quantity,
            entry_price: pos.entry_price,
            mark_price: pos.mark_price,
            unrealized_pnl: pos.unrealized_pnl,
            realized_pnl: pos.realized_pnl,
            liquidation_price: pos.liquidation_price,
            leverage: Some(pos.leverage),
            margin_type: Some(pos.margin_type),
            reason: Some(PositionChangeReason::Other),
            timestamp: data.get("update_time")
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CANCEL ALL
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from DELETE /spot/orders or DELETE /futures/{settle}/orders.
    ///
    /// Gate.io returns an array of cancelled order objects.
    pub fn parse_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        Self::check_error(response)?;

        let arr = match response.as_array() {
            Some(a) => a,
            None => {
                // Some Gate.io endpoints return an empty object on success
                return Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                });
            }
        };

        let details: Vec<OrderResult> = arr.iter().map(|item| {
            match Self::parse_order_data(item, "") {
                Ok(order) => OrderResult {
                    order: Some(order),
                    client_order_id: None,
                    success: true,
                    error: None,
                    error_code: None,
                },
                Err(e) => OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(e.to_string()),
                    error_code: None,
                },
            }
        }).collect();

        let cancelled_count = details.iter().filter(|d| d.success).count() as u32;
        let failed_count = details.iter().filter(|d| !d.success).count() as u32;

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count,
            details,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AMEND ORDER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from PATCH /futures/{settle}/orders/{order_id}.
    ///
    /// Gate.io returns a single amended order object.
    pub fn parse_amend_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        Self::check_error(response)?;
        Self::parse_order_data(response, symbol)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BATCH ORDERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /spot/batch_orders or POST /futures/{settle}/batch_orders.
    ///
    /// Gate.io returns an array of order creation results; each element may
    /// contain a `succeeded` field or an error label.
    pub fn parse_batch_orders_response(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        Self::check_error(response)?;

        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in batch orders response".to_string()))?;

        Ok(arr.iter().map(|item| {
            // Gate.io marks individual failures with a non-empty "label" field
            if item.get("label").and_then(|v| v.as_str()).map(|s| !s.is_empty()).unwrap_or(false) {
                let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                let message = item.get("message").and_then(|v| v.as_str()).unwrap_or("batch order failed");
                return OrderResult {
                    order: None,
                    client_order_id: item.get("text").and_then(|v| v.as_str()).map(String::from),
                    success: false,
                    error: Some(format!("{}: {}", label, message)),
                    error_code: None,
                };
            }

            match Self::parse_order_data(item, "") {
                Ok(order) => OrderResult {
                    client_order_id: order.client_order_id.clone(),
                    order: Some(order),
                    success: true,
                    error: None,
                    error_code: None,
                },
                Err(e) => OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some(e.to_string()),
                    error_code: None,
                },
            }
        }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let response = json!([
            {
                "currency_pair": "BTC_USDT",
                "last": "48600.5",
                "lowest_ask": "48601.0",
                "highest_bid": "48600.0",
                "change_percentage": "2.5",
                "base_volume": "1234.567",
                "quote_volume": "60000000.00",
                "high_24h": "49000.0",
                "low_24h": "47500.0"
            }
        ]);

        let ticker = GateioParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC_USDT");
        assert!((ticker.last_price - 48600.5).abs() < f64::EPSILON);
        assert!((ticker.bid_price.unwrap() - 48600.0).abs() < f64::EPSILON);
        assert!((ticker.ask_price.unwrap() - 48601.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_klines() {
        // Gate.io format: [time, volume, close, high, low, open, quote_volume]
        let response = json!([
            ["1566703320", "123.456", "8553.74", "8550.24", "8527.17", "8533.02", "1000000.00"]
        ]);

        let klines = GateioParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);

        let kline = &klines[0];
        assert!((kline.open - 8533.02).abs() < f64::EPSILON);
        assert!((kline.close - 8553.74).abs() < f64::EPSILON);
        assert!((kline.high - 8550.24).abs() < f64::EPSILON);
        assert!((kline.low - 8527.17).abs() < f64::EPSILON);
        assert!((kline.volume - 123.456).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "id": 123456789,
            "current": 1623898993123i64,
            "update": 1623898993121i64,
            "asks": [["48610.0", "0.5"], ["48615.0", "1.2"]],
            "bids": [["48600.0", "0.8"], ["48595.0", "2.1"]]
        });

        let orderbook = GateioParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 48600.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1623898993123i64);
    }
}
