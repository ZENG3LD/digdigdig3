//! # Deribit Response Parser
//!
//! Parsing JSON-RPC responses from Deribit API.
//!
//! ## CRITICAL: REST vs WebSocket Parsing
//!
//! - **REST**: JSON-RPC format with `result` or `error` fields
//! - **WebSocket**: Notifications use `params` field, no `id` field
//!
//! Always check which format you're parsing!

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult, AccountType, OrderBook, OrderBookLevel, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, StreamEvent, TradeSide,
    OrderUpdateEvent, SymbolInfo, Kline, BracketResponse,
    UserTrade, OrderbookDelta as OrderbookDeltaData,
};

/// Parser for Deribit JSON-RPC responses
pub struct DeribitParser;

impl DeribitParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // JSON-RPC HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract `result` field from JSON-RPC response
    pub fn extract_result(response: &Value) -> ExchangeResult<&Value> {
        // Check for error first
        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
            let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
            let reason = error.get("data")
                .and_then(|d| d.get("reason"))
                .and_then(|r| r.as_str())
                .unwrap_or("");

            let error_msg = if reason.is_empty() {
                message.to_string()
            } else {
                format!("{} - {}", message, reason)
            };

            return Err(ExchangeError::Api {
                code: code as i32,
                message: error_msg,
            });
        }

        // Extract result
        response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field in JSON-RPC response".to_string()))
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

    /// Require f64 field
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Get string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Require string field
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTHENTICATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse authentication response
    ///
    /// Returns (access_token, refresh_token, expires_in)
    pub fn parse_auth(response: &Value) -> ExchangeResult<(String, String, u64)> {
        let result = Self::extract_result(response)?;

        let access_token = Self::require_str(result, "access_token")?.to_string();
        let refresh_token = Self::require_str(result, "refresh_token")?.to_string();
        let expires_in = result.get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(900); // Default 15 min

        Ok((access_token, refresh_token, expires_in))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from ticker response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let result = Self::extract_result(response)?;
        Self::get_f64(result, "last_price")
            .or_else(|| Self::get_f64(result, "mark_price"))
            .or_else(|| Self::get_f64(result, "index_price"))
            .ok_or_else(|| ExchangeError::Parse("No price field found".to_string()))
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let result = Self::extract_result(response)?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            result.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            // Deribit format: ["action", price, amount] or [price, amount]
                            let (price, size) = if pair.len() >= 3 {
                                // WebSocket format with action
                                (Self::parse_f64(&pair[1])?, Self::parse_f64(&pair[2])?)
                            } else {
                                // REST format without action
                                (Self::parse_f64(&pair[0])?, Self::parse_f64(&pair[1])?)
                            };
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(result, "timestamp").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: Self::get_i64(result, "change_id").map(|id| id.to_string()),
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Parse ticker
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let result = Self::extract_result(response)?;

        Ok(Ticker {
            symbol: Self::get_str(result, "instrument_name").unwrap_or("").to_string(),
            last_price: Self::get_f64(result, "last_price").unwrap_or(0.0),
            bid_price: Self::get_f64(result, "best_bid_price"),
            ask_price: Self::get_f64(result, "best_ask_price"),
            high_24h: result.get("stats")
                .and_then(|s| s.get("high"))
                .and_then(Self::parse_f64),
            low_24h: result.get("stats")
                .and_then(|s| s.get("low"))
                .and_then(Self::parse_f64),
            volume_24h: result.get("stats")
                .and_then(|s| s.get("volume"))
                .and_then(Self::parse_f64),
            quote_volume_24h: result.get("stats")
                .and_then(|s| s.get("volume_usd"))
                .and_then(Self::parse_f64),
            price_change_24h: result.get("stats")
                .and_then(|s| s.get("price_change"))
                .and_then(Self::parse_f64),
            price_change_percent_24h: result.get("stats")
                .and_then(|s| s.get("price_change"))
                .and_then(Self::parse_f64),
            timestamp: Self::get_i64(result, "timestamp").unwrap_or(0),
        })
    }

    /// Parse klines from `public/get_tradingview_chart_data` response.
    ///
    /// The response `result` contains parallel arrays:
    /// `ticks`, `open`, `high`, `low`, `close`, `volume`, `cost`, `status`.
    pub fn parse_klines(response: &Value, interval_ms: u64) -> ExchangeResult<Vec<Kline>> {
        let result = Self::extract_result(response)?;

        let ticks = result.get("ticks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticks' array in chart data".to_string()))?;

        let opens = result.get("open")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'open' array in chart data".to_string()))?;

        let highs = result.get("high")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'high' array in chart data".to_string()))?;

        let lows = result.get("low")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'low' array in chart data".to_string()))?;

        let closes = result.get("close")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'close' array in chart data".to_string()))?;

        let volumes = result.get("volume")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'volume' array in chart data".to_string()))?;

        let costs = result.get("cost")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'cost' array in chart data".to_string()))?;

        let len = ticks.len();
        let mut klines = Vec::with_capacity(len);

        for i in 0..len {
            let open_time = ticks[i].as_u64().unwrap_or(0);
            let close_time = open_time + interval_ms.saturating_sub(1);

            klines.push(Kline {
                open_time: open_time as i64,
                open: Self::parse_f64(&opens[i]).unwrap_or(0.0),
                high: Self::parse_f64(&highs[i]).unwrap_or(0.0),
                low: Self::parse_f64(&lows[i]).unwrap_or(0.0),
                close: Self::parse_f64(&closes[i]).unwrap_or(0.0),
                volume: Self::parse_f64(&volumes[i]).unwrap_or(0.0),
                quote_volume: Self::parse_f64(&costs[i]),
                close_time: Some(close_time as i64),
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let result = Self::extract_result(response)?;

        Ok(FundingRate {
            symbol: Self::get_str(result, "instrument_name").unwrap_or("").to_string(),
            rate: Self::get_f64(result, "current_funding")
                .or_else(|| Self::get_f64(result, "funding_8h"))
                .unwrap_or(0.0),
            next_funding_time: None,
            timestamp: Self::get_i64(result, "timestamp").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order from response
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        let result = Self::extract_result(response)?;

        // Result might be the order object directly, or wrapped in "order"
        let order_data = result.get("order").unwrap_or(result);
        Self::parse_order_data(order_data, symbol)
    }

    /// Parse order from data object
    fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        let direction = Self::get_str(data, "direction").unwrap_or("buy");
        let side = match direction {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type_str = Self::get_str(data, "order_type").unwrap_or("limit");
        let order_type = match order_type_str {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "label").map(String::from),
            symbol: Self::get_str(data, "instrument_name").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "trigger_price"),
            quantity: Self::get_f64(data, "amount").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "filled_amount").unwrap_or(0.0),
            average_price: Self::get_f64(data, "average_price"),
            commission: Self::get_f64(data, "commission"),
            commission_asset: None, // Deribit uses instrument's settlement currency
            created_at: Self::get_i64(data, "creation_timestamp").unwrap_or(0),
            updated_at: Self::get_i64(data, "last_update_timestamp"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "order_state").unwrap_or("open") {
            "open" => {
                let filled = Self::get_f64(data, "filled_amount").unwrap_or(0.0);
                if filled > 0.0 {
                    OrderStatus::PartiallyFilled
                } else {
                    OrderStatus::New
                }
            }
            "filled" => OrderStatus::Filled,
            "rejected" => OrderStatus::Rejected,
            "cancelled" => OrderStatus::Canceled,
            "untriggered" => OrderStatus::New, // Stop order not triggered yet
            _ => OrderStatus::New,
        }
    }

    /// Parse list of orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let result = Self::extract_result(response)?;

        let arr = result.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        arr.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Parse order ID from create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let result = Self::extract_result(response)?;

        // Order data might be in "order" sub-object
        let order_data = result.get("order").unwrap_or(result);

        Self::require_str(order_data, "order_id").map(String::from)
    }

    /// Parse user trade fills from `private/get_user_trades_by_instrument` or
    /// `private/get_user_trades_by_currency`.
    ///
    /// Response format:
    /// ```json
    /// {"result":{"trades":[{"trade_id":"123","order_id":"456","instrument_name":"BTC-PERPETUAL",
    ///   "direction":"buy","price":50000.0,"amount":0.001,"fee":0.00001,
    ///   "fee_currency":"BTC","liquidity":"M","timestamp":1672531200000}]}}
    /// ```
    /// `liquidity`: "M" = maker, "T" = taker.
    pub fn parse_user_trades(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let result = Self::extract_result(response)?;

        // Result contains a `trades` array
        let trades_arr = result.get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'trades' array in user trades response".to_string()))?;

        trades_arr.iter().map(|item| {
            let id = Self::get_str(item, "trade_id")
                .map(|s| s.to_string())
                .ok_or_else(|| ExchangeError::Parse("Missing 'trade_id'".to_string()))?;

            let order_id = Self::get_str(item, "order_id")
                .unwrap_or("")
                .to_string();

            let symbol = Self::get_str(item, "instrument_name")
                .unwrap_or("")
                .to_string();

            let direction = Self::get_str(item, "direction").unwrap_or("buy");
            let side = match direction {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let price = Self::get_f64(item, "price").unwrap_or(0.0);
            let quantity = Self::get_f64(item, "amount").unwrap_or(0.0);

            let commission = Self::get_f64(item, "fee").unwrap_or(0.0);
            let commission_asset = Self::get_str(item, "fee_currency")
                .unwrap_or("")
                .to_string();

            // "M" = maker, "T" = taker
            let is_maker = Self::get_str(item, "liquidity")
                .map(|l| l == "M")
                .unwrap_or(false);

            let timestamp = Self::get_i64(item, "timestamp").unwrap_or(0);

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
        }).collect()
    }

    /// Parse OTOCO (Bracket) response from Deribit `private/buy` / `private/sell`
    /// with `linked_order_type: "one_triggers_one_cancels_other"`.
    ///
    /// Deribit returns the entry order in `result.order`.  The TP and SL legs are
    /// created atomically when the entry fills; the exchange does not return their
    /// order IDs in the placement response.  We reconstruct synthetic pending legs
    /// from the `otoco_config` embedded in `result.order.otoco_config` when present,
    /// or build minimal placeholder orders if that field is absent.
    pub fn parse_bracket_order(response: &Value, symbol: &str) -> ExchangeResult<BracketResponse> {
        let result = Self::extract_result(response)?;

        // Entry order lives at result.order (or result directly for some API versions)
        let entry_data = result.get("order").unwrap_or(result);
        let entry_order = Self::parse_order_data(entry_data, symbol)?;

        // Try to find leg configurations from the response
        let legs_config = entry_data
            .get("otoco_config")
            .and_then(|v| v.as_array());

        // Build TP order — first leg in otoco_config
        let tp_order = if let Some(legs) = legs_config {
            let leg = legs.first().unwrap_or(&serde_json::Value::Null);
            let tp_price = Self::get_f64(leg, "limit_price")
                .or_else(|| Self::get_f64(leg, "price"))
                .unwrap_or(0.0);
            Order {
                id: Self::get_str(leg, "order_id").unwrap_or("tp_pending").to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: entry_order.side.opposite(),
                order_type: OrderType::Limit { price: tp_price },
                status: OrderStatus::New,
                price: Some(tp_price),
                stop_price: None,
                quantity: entry_order.quantity,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: entry_order.created_at,
                updated_at: None,
                time_in_force: crate::core::TimeInForce::Gtc,
            }
        } else {
            // Minimal placeholder when config not returned
            Order {
                id: "tp_pending".to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: entry_order.side.opposite(),
                order_type: OrderType::Market,
                status: OrderStatus::New,
                price: None,
                stop_price: None,
                quantity: entry_order.quantity,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: entry_order.created_at,
                updated_at: None,
                time_in_force: crate::core::TimeInForce::Gtc,
            }
        };

        // Build SL order — second leg in otoco_config
        let sl_order = if let Some(legs) = legs_config {
            let leg = legs.get(1).unwrap_or(&serde_json::Value::Null);
            let sl_price = Self::get_f64(leg, "trigger_price")
                .or_else(|| Self::get_f64(leg, "price"))
                .unwrap_or(0.0);
            Order {
                id: Self::get_str(leg, "order_id").unwrap_or("sl_pending").to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: entry_order.side.opposite(),
                order_type: OrderType::StopMarket { stop_price: sl_price },
                status: OrderStatus::New,
                price: None,
                stop_price: Some(sl_price),
                quantity: entry_order.quantity,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: entry_order.created_at,
                updated_at: None,
                time_in_force: crate::core::TimeInForce::Gtc,
            }
        } else {
            Order {
                id: "sl_pending".to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: entry_order.side.opposite(),
                order_type: OrderType::Market,
                status: OrderStatus::New,
                price: None,
                stop_price: None,
                quantity: entry_order.quantity,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: entry_order.created_at,
                updated_at: None,
                time_in_force: crate::core::TimeInForce::Gtc,
            }
        };

        Ok(BracketResponse {
            entry_order,
            tp_order,
            sl_order,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse account summary (balances)
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let result = Self::extract_result(response)?;

        let currency = Self::get_str(result, "currency").unwrap_or("BTC").to_string();
        let equity = Self::get_f64(result, "equity").unwrap_or(0.0);
        let available = Self::get_f64(result, "available_funds")
            .or_else(|| Self::get_f64(result, "balance"))
            .unwrap_or(0.0);
        let margin = Self::get_f64(result, "initial_margin").unwrap_or(0.0);

        Ok(vec![Balance {
            asset: currency,
            free: available,
            locked: margin,
            total: equity,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let result = Self::extract_result(response)?;

        let arr = result.as_array()
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
        let result = Self::extract_result(response)?;
        Self::parse_position_data(result)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "instrument_name")?.to_string();
        let size = Self::get_f64(data, "size").unwrap_or(0.0);

        // Skip empty positions
        if size.abs() < f64::EPSILON {
            return None;
        }

        let direction = Self::get_str(data, "direction").unwrap_or("buy");
        let side = match direction {
            "sell" => PositionSide::Short,
            "buy" => PositionSide::Long,
            _ => PositionSide::Both,
        };

        Some(Position {
            symbol,
            side,
            quantity: size.abs(),
            entry_price: Self::get_f64(data, "average_price").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl: Self::get_f64(data, "floating_profit_loss").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realized_profit_loss"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: Self::get_f64(data, "estimated_liquidation_price"),
            margin: Self::get_f64(data, "initial_margin"),
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING (Separate from REST!)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker notification
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        let stats = data.get("stats");
        Ok(Ticker {
            symbol: Self::get_str(data, "instrument_name").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last_price").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "best_bid_price"),
            ask_price: Self::get_f64(data, "best_ask_price"),
            high_24h: stats.and_then(|s| Self::get_f64(s, "high")),
            low_24h: stats.and_then(|s| Self::get_f64(s, "low")),
            volume_24h: stats.and_then(|s| Self::get_f64(s, "volume")),
            quote_volume_24h: stats.and_then(|s| Self::get_f64(s, "volume_usd")),
            price_change_24h: stats.and_then(|s| Self::get_f64(s, "price_change")),
            price_change_percent_24h: stats.and_then(|s| Self::get_f64(s, "price_change")),
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket trade notification
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        // Deribit sends an array of trades; take the last element if so
        let item = if data.is_array() {
            data.as_array().and_then(|a| a.last()).unwrap_or(data)
        } else {
            data
        };

        let direction = Self::get_str(item, "direction").unwrap_or("buy");
        let side = match direction {
            "sell" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(item, "trade_id").unwrap_or("").to_string(),
            symbol: Self::get_str(item, "instrument_name").unwrap_or("").to_string(),
            price: Self::require_f64(item, "price")?,
            quantity: Self::get_f64(item, "amount").unwrap_or(0.0),
            side,
            timestamp: Self::get_i64(item, "timestamp").unwrap_or(0),
        })
    }

    /// Parse WebSocket orderbook notification
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<StreamEvent> {
        let msg_type = Self::get_str(data, "type").unwrap_or("change");

        if msg_type == "snapshot" {
            // Full orderbook snapshot
            let orderbook = Self::parse_orderbook(&serde_json::json!({
                "result": data
            }))?;

            Ok(StreamEvent::OrderbookSnapshot(orderbook))
        } else {
            // Delta update
            let parse_changes = |key: &str| -> Vec<OrderBookLevel> {
                data.get(key)
                    .and_then(|arr| arr.as_array())
                    .map(|changes| {
                        changes.iter()
                            .filter_map(|change| {
                                let arr = change.as_array()?;
                                if arr.len() < 3 { return None; }
                                let price = Self::parse_f64(&arr[1])?;
                                let size = Self::parse_f64(&arr[2])?;
                                Some(OrderBookLevel::new(price, size))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };

            Ok(StreamEvent::OrderbookDelta(OrderbookDeltaData {
                bids: parse_changes("bids"),
                asks: parse_changes("asks"),
                timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
                first_update_id: None,
                last_update_id: None,
                prev_update_id: None,
                event_time: None,
                checksum: None,
            }))
        }
    }

    /// Parse WebSocket order update notification
    pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
        let direction = Self::get_str(data, "direction").unwrap_or("buy");
        let side = match direction {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type_str = Self::get_str(data, "order_type").unwrap_or("limit");
        let order_type = match order_type_str {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(OrderUpdateEvent {
            order_id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "label").map(String::from),
            symbol: Self::get_str(data, "instrument_name").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            quantity: Self::get_f64(data, "amount").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "filled_amount").unwrap_or(0.0),
            average_price: Self::get_f64(data, "average_price"),
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp: Self::get_i64(data, "last_update_timestamp").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Deribit public/get_instruments response.
    ///
    /// Response format (JSON-RPC):
    /// ```json
    /// {"jsonrpc":"2.0","result":[{"instrument_name":"BTC-PERPETUAL","base_currency":"BTC","quote_currency":"USD","tick_size":0.5,"min_trade_amount":10,"is_active":true,...},...],"id":1}
    /// ```
    pub fn parse_exchange_info(response: &Value, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let result = Self::extract_result(response)?;

        let items = result.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in result".to_string()))?;

        let mut symbols = Vec::with_capacity(items.len());

        for item in items {
            // Only include active instruments
            let is_active = item.get("is_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if !is_active {
                continue;
            }

            let instrument_name = match item.get("instrument_name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let base_asset = item.get("base_currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = item.get("quote_currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string();

            if base_asset.is_empty() {
                continue;
            }

            let raw_tick_size = item.get("tick_size")
                .and_then(|v| v.as_f64());

            let price_precision = raw_tick_size
                .map(|ts| {
                    if ts <= 0.0 { 8u8 }
                    else { (-ts.log10().ceil()).max(0.0) as u8 }
                })
                .unwrap_or(2);

            let min_quantity = item.get("min_trade_amount")
                .and_then(|v| v.as_f64());

            let step_size = item.get("contract_size")
                .and_then(|v| v.as_f64());

            symbols.push(SymbolInfo {
                symbol: instrument_name,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision: 8,
                min_quantity,
                max_quantity: None,
                tick_size: raw_tick_size,
                step_size,
                min_notional: None,
                account_type,
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
    fn test_parse_auth() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "access_token": "test_access_token",
                "refresh_token": "test_refresh_token",
                "expires_in": 900,
                "token_type": "bearer"
            }
        });

        let (access, refresh, expires) = DeribitParser::parse_auth(&response).unwrap();
        assert_eq!(access, "test_access_token");
        assert_eq!(refresh, "test_refresh_token");
        assert_eq!(expires, 900);
    }

    #[test]
    fn test_parse_error() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 10004,
                "message": "order_not_found"
            }
        });

        let result = DeribitParser::extract_result(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("order_not_found"));
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "instrument_name": "BTC-PERPETUAL",
                "last_price": 50000.5,
                "best_bid_price": 50000.0,
                "best_ask_price": 50001.0,
                "timestamp": 1234567890000i64,
                "stats": {
                    "volume": 15894.89,
                    "high": 51000.0,
                    "low": 49000.0
                }
            }
        });

        let ticker = DeribitParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-PERPETUAL");
        assert!((ticker.last_price - 50000.5).abs() < f64::EPSILON);
        assert_eq!(ticker.bid_price, Some(50000.0));
        assert_eq!(ticker.ask_price, Some(50001.0));
    }
}
