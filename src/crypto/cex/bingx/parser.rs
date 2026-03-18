//! # BingX Response Parser
//!
//! Парсинг JSON ответов от BingX API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide, SymbolInfo,
    UserTrade,
};

/// Парсер ответов BingX API
pub struct BingxParser;

impl BingxParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Извлечь data из response
    /// BingX format: {"code": 0, "msg": "", "data": {...}}
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check for error
        let code = response.get("code")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);

        if code != 0 {
            let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: msg.to_string(),
            });
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Парсить f64 из string или number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Парсить f64 из поля
    pub fn get_f64(data: &Value, key: &str) -> Option<f64> {
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

    /// Парсить price from ticker response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let data = Self::extract_data(response)?;

        // Try multiple field names: price (spot), lastPrice (swap)
        Self::get_f64(data, "price")
            .or_else(|| Self::get_f64(data, "lastPrice"))
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'price'".to_string()))
    }

    /// Парсить klines
    /// BingX format: [{"time": 1649404800000, "open": "43250.00", "high": "43350.00", ...}]
    /// or array format: [[timestamp, open, high, low, close, volume], ...]
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Check if item is an array (array format) or object (object format)
            if let Some(kline_arr) = item.as_array() {
                // Array format: [timestamp, open, high, low, close, volume]
                if kline_arr.len() >= 6 {
                    let open_time = kline_arr[0].as_i64().unwrap_or(0);
                    klines.push(Kline {
                        open_time,
                        open: Self::parse_f64(&kline_arr[1]).unwrap_or(0.0),
                        high: Self::parse_f64(&kline_arr[2]).unwrap_or(0.0),
                        low: Self::parse_f64(&kline_arr[3]).unwrap_or(0.0),
                        close: Self::parse_f64(&kline_arr[4]).unwrap_or(0.0),
                        volume: Self::parse_f64(&kline_arr[5]).unwrap_or(0.0),
                        quote_volume: None,
                        close_time: None,
                        trades: None,
                    });
                }
            } else {
                // Object format: {"time": ..., "open": ..., ...}
                let open_time = item.get("time")
                    .or_else(|| item.get("openTime"))
                    .and_then(|t| t.as_i64())
                    .unwrap_or(0);

                klines.push(Kline {
                    open_time,
                    open: Self::get_f64(item, "open").unwrap_or(0.0),
                    close: Self::get_f64(item, "close").unwrap_or(0.0),
                    high: Self::get_f64(item, "high").unwrap_or(0.0),
                    low: Self::get_f64(item, "low").unwrap_or(0.0),
                    volume: Self::get_f64(item, "volume").unwrap_or(0.0),
                    quote_volume: None,
                    close_time: None,
                    trades: None,
                });
            }
        }

        // BingX returns klines newest-first; reverse to oldest-first for chart display
        klines.reverse();

        Ok(klines)
    }

    /// Парсить orderbook
    /// BingX format: {"bids": [["43302.00", "0.521000"], ...], "asks": [...]}
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
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let mut bids = parse_levels("bids");
        let mut asks = parse_levels("asks");

        // Ensure bids are sorted descending (highest price first)
        bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Ensure asks are sorted ascending (lowest price first)
        asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(OrderBook {
            timestamp: crate::core::timestamp_millis() as i64,
            bids,
            asks,
            sequence: None,
        })
    }

    /// Парсить ticker
    /// Supports both single symbol and all symbols response
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let data = Self::extract_data(response)?;

        // Handle array response (all symbols)
        let ticker_data = if data.is_array() {
            data.as_array()
                .and_then(|arr| arr.first())
                .ok_or_else(|| ExchangeError::Parse("Empty ticker array".to_string()))?
        } else {
            data
        };

        let bid_price = Self::get_f64(ticker_data, "bidPrice");
        let ask_price = Self::get_f64(ticker_data, "askPrice");

        // last_price might be missing in BookTicker, calculate from bid/ask
        let last_price = Self::get_f64(ticker_data, "lastPrice")
            .or_else(|| Self::get_f64(ticker_data, "price"))
            .or_else(|| {
                // If we have both bid and ask, use midpoint
                match (bid_price, ask_price) {
                    (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
                    _ => None,
                }
            })
            .unwrap_or(0.0);

        Ok(Ticker {
            symbol: Self::get_str(ticker_data, "symbol").unwrap_or("").to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: Self::get_f64(ticker_data, "highPrice"),
            low_24h: Self::get_f64(ticker_data, "lowPrice"),
            volume_24h: Self::get_f64(ticker_data, "volume"),
            quote_volume_24h: Self::get_f64(ticker_data, "quoteVolume"),
            price_change_24h: Self::get_f64(ticker_data, "priceChange"),
            price_change_percent_24h: Self::get_f64(ticker_data, "priceChangePercent"),
            timestamp: ticker_data.get("closeTime")
                .or_else(|| ticker_data.get("time"))
                .and_then(|t| t.as_i64())
                .unwrap_or(crate::core::timestamp_millis() as i64),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order из response
    pub fn parse_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        let data = Self::extract_data(response)?;

        // Spot returns order directly in data, Swap wraps it in "order"
        let order_data = data.get("order").unwrap_or(data);

        Self::parse_order_data(order_data, symbol)
    }

    /// Парсить order из data object
    pub fn parse_order_data(data: &Value, symbol: &str) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("BUY").to_uppercase().as_str() {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("LIMIT").to_uppercase().as_str() {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        // Get order ID (can be long or string)
        let id = data.get("orderId")
            .and_then(|v| v.as_i64().map(|i| i.to_string()))
            .or_else(|| Self::get_str(data, "orderId").map(String::from))
            .unwrap_or_default();

        Ok(Order {
            id,
            client_order_id: Self::get_str(data, "clientOrderId").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "stopPrice"),
            quantity: Self::get_f64(data, "origQty")
                .or_else(|| Self::get_f64(data, "quantity"))
                .unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "executedQty")
                .or_else(|| Self::get_f64(data, "cumExecQty"))
                .unwrap_or(0.0),
            average_price: Self::get_f64(data, "avgPrice"),
            commission: Self::get_f64(data, "commission"),
            commission_asset: Self::get_str(data, "commissionAsset").map(String::from),
            created_at: data.get("time")
                .or_else(|| data.get("transactTime"))
                .and_then(|t| t.as_i64())
                .unwrap_or(0),
            updated_at: data.get("updateTime").and_then(|t| t.as_i64()),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить статус ордера
    /// BingX status: NEW, PARTIALLY_FILLED, FILLED, CANCELED, REJECTED, EXPIRED
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("NEW").to_uppercase().as_str() {
            "FILLED" => OrderStatus::Filled,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "CANCELED" | "CANCELLED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        }
    }

    /// Парсить список ордеров
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;

        // BingX wraps orders in "orders" array
        let items = data.get("orders")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        items.iter()
            .map(|item| Self::parse_order_data(item, ""))
            .collect()
    }

    /// Парсить order ID из create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;

        // Try to get orderId as number or string
        data.get("orderId")
            .and_then(|v| v.as_i64().map(|i| i.to_string()))
            .or_else(|| Self::get_str(data, "orderId").map(String::from))
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balances (Spot)
    /// BingX format: {"balances": [{"asset": "USDT", "free": "10000", "locked": "500"}, ...]}
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let balances_array = data.get("balances")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'balances' array".to_string()))?;

        let mut balances = Vec::new();

        for item in balances_array {
            let asset = Self::get_str(item, "asset").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "free").unwrap_or(0.0);
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

    /// Парсить swap account balance
    /// BingX format: {"balance": {"asset": "USDT", "balance": "15000", ...}}
    pub fn parse_swap_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let balance_obj = data.get("balance")
            .ok_or_else(|| ExchangeError::Parse("Missing 'balance' object".to_string()))?;

        let asset = Self::get_str(balance_obj, "asset").unwrap_or("USDT").to_string();
        let total_balance = Self::get_f64(balance_obj, "balance").unwrap_or(0.0);
        let available = Self::get_f64(balance_obj, "availableMargin").unwrap_or(0.0);
        let used = Self::get_f64(balance_obj, "usedMargin").unwrap_or(0.0);

        Ok(vec![Balance {
            asset,
            free: available,
            locked: used,
            total: total_balance,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
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
        let data = Self::extract_data(response)?;
        Self::parse_position_data(data)
            .ok_or_else(|| ExchangeError::Parse("Invalid position data".to_string()))
    }

    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
        let position_amt = Self::get_f64(data, "positionAmt")
            .or_else(|| Self::get_f64(data, "positionAmount"))
            .unwrap_or(0.0);

        // Skip empty positions
        if position_amt.abs() < f64::EPSILON {
            return None;
        }

        // Determine side from positionSide field or amount sign
        let side = if let Some(side_str) = Self::get_str(data, "positionSide") {
            match side_str.to_uppercase().as_str() {
                "LONG" => PositionSide::Long,
                "SHORT" => PositionSide::Short,
                _ => if position_amt > 0.0 { PositionSide::Long } else { PositionSide::Short }
            }
        } else if position_amt > 0.0 { PositionSide::Long } else { PositionSide::Short };

        Some(Position {
            symbol,
            side,
            quantity: position_amt.abs(),
            entry_price: Self::get_f64(data, "avgPrice")
                .or_else(|| Self::get_f64(data, "entryPrice"))
                .unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPrice"),
            unrealized_pnl: Self::get_f64(data, "unrealizedProfit")
                .or_else(|| Self::get_f64(data, "unrealisedPnl"))
                .unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "realisedProfit")
                .or_else(|| Self::get_f64(data, "realisedPnl")),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: Self::get_f64(data, "liquidationPrice"),
            margin: Self::get_f64(data, "initialMargin"),
            margin_type: if data.get("isolated").and_then(|v| v.as_bool()).unwrap_or(false) {
                crate::core::MarginType::Isolated
            } else {
                crate::core::MarginType::Cross
            },
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker
    pub fn parse_ws_ticker(data: &Value, _account_type: crate::core::AccountType) -> ExchangeResult<Ticker> {
        let symbol = Self::require_str(data, "s")?.to_string();
        let last_price = Self::require_f64(data, "c")?;
        let timestamp = data.get("C").and_then(|v| v.as_i64()).unwrap_or(0);

        let price_change_24h = Self::get_f64(data, "p");

        // BingX spot @ticker may omit the `P` (percent change) field.
        // Compute it from open_price (`o`) and last_price (`c`) as a fallback.
        let price_change_percent_24h = Self::get_f64(data, "P").or_else(|| {
            let open = Self::get_f64(data, "o")?;
            if open > 0.0 {
                Some((last_price - open) / open * 100.0)
            } else {
                None
            }
        });

        Ok(Ticker {
            symbol,
            last_price,
            bid_price: None, // Not in ticker stream
            ask_price: None, // Not in ticker stream
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: Self::get_f64(data, "q"),
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse WebSocket trade
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<crate::core::PublicTrade> {
        use crate::core::types::TradeSide;

        let symbol = Self::require_str(data, "s")?.to_string();
        let price = Self::require_f64(data, "p")?;
        let quantity = Self::require_f64(data, "q")?;
        let timestamp = Self::get_f64(data, "t").map(|t| t as i64).unwrap_or(0);
        let is_buyer_maker = data.get("m").and_then(|v| v.as_bool()).unwrap_or(false);

        // If buyer is maker, then seller is taker (Sell side)
        // If buyer is taker, then buyer initiated (Buy side)
        let side = if is_buyer_maker {
            TradeSide::Sell
        } else {
            TradeSide::Buy
        };

        Ok(crate::core::PublicTrade {
            id: data.get("t").and_then(|v| v.as_i64()).map(|id| id.to_string()).unwrap_or_default(),
            symbol,
            price,
            quantity,
            side,
            timestamp,
        })
    }

    /// Parse WebSocket orderbook
    pub fn parse_ws_orderbook(data: &Value) -> ExchangeResult<crate::core::StreamEvent> {
        let bids_arr = data.get("bids").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing bids".to_string()))?;
        let asks_arr = data.get("asks").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing asks".to_string()))?;

        let mut bids = Vec::new();
        for bid in bids_arr {
            if let Some(arr) = bid.as_array() {
                if arr.len() >= 2 {
                    let price = Self::parse_f64(&arr[0]).unwrap_or(0.0);
                    let qty = Self::parse_f64(&arr[1]).unwrap_or(0.0);
                    bids.push((price, qty));
                }
            }
        }

        let mut asks = Vec::new();
        for ask in asks_arr {
            if let Some(arr) = ask.as_array() {
                if arr.len() >= 2 {
                    let price = Self::parse_f64(&arr[0]).unwrap_or(0.0);
                    let qty = Self::parse_f64(&arr[1]).unwrap_or(0.0);
                    asks.push((price, qty));
                }
            }
        }

        let timestamp = crate::core::timestamp_millis() as i64;

        Ok(crate::core::StreamEvent::OrderbookDelta {
            bids,
            asks,
            timestamp,
        })
    }

    /// Parse WebSocket kline
    pub fn parse_ws_kline(data: &Value) -> ExchangeResult<Kline> {
        let kline_data = data.get("k")
            .ok_or_else(|| ExchangeError::Parse("Missing 'k' field in kline data".to_string()))?;

        let open_time = Self::get_f64(kline_data, "t").map(|t| t as i64)
            .ok_or_else(|| ExchangeError::Parse("Missing timestamp".to_string()))?;
        let open = Self::require_f64(kline_data, "o")?;
        let high = Self::require_f64(kline_data, "h")?;
        let low = Self::require_f64(kline_data, "l")?;
        let close = Self::require_f64(kline_data, "c")?;
        let volume = Self::require_f64(kline_data, "v")?;

        Ok(Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            close_time: Self::get_f64(kline_data, "T").map(|t| t as i64),
            quote_volume: None,
            trades: None,
        })
    }

    /// Parse WebSocket order update
    pub fn parse_ws_order_update(
        data: &Value,
        _account_type: crate::core::AccountType,
    ) -> ExchangeResult<crate::core::OrderUpdateEvent> {
        let order_id = Self::require_str(data, "i")?.to_string();
        let client_order_id = Self::get_str(data, "c").map(|s| s.to_string());
        let symbol = Self::require_str(data, "s")?.to_string();

        let side_str = Self::require_str(data, "S")?;
        let side = match side_str.to_uppercase().as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse(format!("Invalid side: {}", side_str))),
        };

        let order_type_str = Self::require_str(data, "o")?;
        let order_type = match order_type_str.to_uppercase().as_str() {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            _ => OrderType::Limit { price: 0.0 },
        };

        let status_str = Self::require_str(data, "X")?;
        let status = match status_str.to_uppercase().as_str() {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" | "CANCELLED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        };

        let price = Self::get_f64(data, "p");
        let quantity = Self::require_f64(data, "q")?;
        let filled_quantity = Self::get_f64(data, "z").unwrap_or(0.0);
        let average_price = Self::get_f64(data, "L");

        let last_fill_price = Self::get_f64(data, "L");
        let last_fill_quantity = Self::get_f64(data, "l");
        let last_fill_commission = Self::get_f64(data, "n");
        let commission_asset = Self::get_str(data, "N").map(|s| s.to_string());
        let trade_id = data.get("t").and_then(|v| v.as_i64()).map(|id| id.to_string());

        let timestamp = Self::get_f64(data, "T").map(|t| t as i64).unwrap_or(0);

        Ok(crate::core::OrderUpdateEvent {
            order_id,
            client_order_id,
            symbol,
            side,
            order_type,
            status,
            price,
            quantity,
            filled_quantity,
            average_price,
            last_fill_price,
            last_fill_quantity,
            last_fill_commission,
            commission_asset,
            trade_id,
            timestamp,
        })
    }

    /// Parse WebSocket balance update
    pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<crate::core::BalanceUpdateEvent> {
        // BingX balance update format
        let balances = data.get("B").and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'B' field in balance update".to_string()))?;

        if balances.is_empty() {
            return Err(ExchangeError::Parse("Empty balance array".to_string()));
        }

        // Take first balance (or we could emit multiple events)
        let balance = &balances[0];

        let asset = Self::require_str(balance, "a")?.to_string();
        let free = Self::require_f64(balance, "f")?;
        let locked = Self::require_f64(balance, "l")?;
        let total = free + locked;

        let timestamp = Self::get_f64(data, "E").map(|t| t as i64)
            .unwrap_or_else(|| crate::core::timestamp_millis() as i64);

        Ok(crate::core::BalanceUpdateEvent {
            asset,
            free,
            locked,
            total,
            delta: None,
            reason: None,
            timestamp,
        })
    }

    /// Parse WebSocket position update
    pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<crate::core::PositionUpdateEvent> {
        let symbol = Self::require_str(data, "s")?.to_string();
        let position_amt = Self::require_f64(data, "pa")?;

        let side = if position_amt > 0.0 {
            PositionSide::Long
        } else if position_amt < 0.0 {
            PositionSide::Short
        } else {
            PositionSide::Long // Flat position
        };

        let quantity = position_amt.abs();
        let entry_price = Self::require_f64(data, "ep")?;
        let mark_price = Self::get_f64(data, "mp");
        let unrealized_pnl = Self::require_f64(data, "up")?;
        let realized_pnl = Self::get_f64(data, "rp");

        let timestamp = crate::core::timestamp_millis() as i64;

        Ok(crate::core::PositionUpdateEvent {
            symbol,
            side,
            quantity,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl,
            liquidation_price: None,
            leverage: None,
            margin_type: None,
            reason: None,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from BingX swap contracts response.
    ///
    /// Response format:
    /// ```json
    /// {"code":0,"msg":"","data":[{"symbol":"BTC-USDT","currency":"USDT","asset":"BTC","size":0.0001,"tickSize":0.1,"tradeMinLimit":1,"maxLongLeverage":150,"maxShortLeverage":150,"status":1},...],"timestamp":...}
    /// ```
    pub fn parse_swap_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let data = Self::extract_data(response)?;

        let items = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array in data".to_string()))?;

        let mut symbols = Vec::with_capacity(items.len());

        for item in items {
            // status: 1 = trading
            let status = item.get("status").and_then(|v| v.as_i64()).unwrap_or(1);
            if status != 1 {
                continue;
            }

            let symbol = match item.get("symbol").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let base_asset = item.get("asset")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USDT")
                .to_string();

            if base_asset.is_empty() {
                continue;
            }

            let price_precision = item.get("tickSize")
                .and_then(|v| v.as_f64())
                .map(|ts| {
                    if ts <= 0.0 { 8u8 }
                    else { (-ts.log10().ceil()).max(0.0) as u8 }
                })
                .unwrap_or(2);

            let quantity_precision = item.get("size")
                .and_then(|v| v.as_f64())
                .map(|ts| {
                    if ts <= 0.0 { 8u8 }
                    else { (-ts.log10().ceil()).max(0.0) as u8 }
                })
                .unwrap_or(4);

            let min_quantity = item.get("tradeMinLimit")
                .and_then(|v| v.as_f64());

            let step_size = item.get("size").and_then(|v| v.as_f64());

            // tickSize is a number in the swap contracts response
            let tick_size = item.get("tickSize").and_then(|v| v.as_f64());

            symbols.push(SymbolInfo {
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
            });
        }

        Ok(symbols)
    }

    /// Parse exchange info from BingX spot symbols response.
    ///
    /// Response format:
    /// ```json
    /// {"code":0,"msg":"","data":{"symbols":[{"symbol":"BTC-USDT","minQty":"0.00001","maxQty":"9000","stepSize":"0.00001","tickSize":"0.01","minNotional":"1","status":1},...]}}
    /// ```
    pub fn parse_spot_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let data = Self::extract_data(response)?;

        let symbols_arr = data.get("symbols")
            .and_then(|s| s.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'symbols' array".to_string()))?;

        let mut symbols = Vec::with_capacity(symbols_arr.len());

        for item in symbols_arr {
            let status = item.get("status").and_then(|v| v.as_i64()).unwrap_or(1);
            if status != 1 {
                continue;
            }

            let raw_symbol = match item.get("symbol").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };

            // Spot symbol format: "BTC-USDT"
            let parts: Vec<&str> = raw_symbol.splitn(2, '-').collect();
            let (base_asset, quote_asset) = if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                continue;
            };

            let price_precision = item.get("tickSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|ts| {
                    if ts <= 0.0 { 8u8 }
                    else { (-ts.log10().ceil()).max(0.0) as u8 }
                })
                .unwrap_or(2);

            let quantity_precision = item.get("stepSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|ts| {
                    if ts <= 0.0 { 8u8 }
                    else { (-ts.log10().ceil()).max(0.0) as u8 }
                })
                .unwrap_or(8);

            let min_quantity = item.get("minQty")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let max_quantity = item.get("maxQty")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let step_size = item.get("stepSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // tickSize is a string in the spot symbols response (e.g. "0.01")
            let tick_size = item.get("tickSize")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = item.get("minNotional")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            symbols.push(SymbolInfo {
                symbol: raw_symbol.to_string(),
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity,
                tick_size,
                step_size,
                min_notional,
            });
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER TRADES (FILLS)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse user trade fills from BingX.
    ///
    /// Spot response (`/openApi/spot/v1/trade/myTrades`):
    /// `{"data":{"list":[{"tradeId":"123","orderId":"456","symbol":"BTC-USDT","side":"BUY","price":"50000","qty":"0.001","commission":"0.01","commissionAsset":"USDT","isMaker":false,"time":1672531200000}]}}`
    ///
    /// Swap response (`/openApi/swap/v2/trade/fillHistory`):
    /// `{"data":{"fills":[{"filledTm":"123","symbol":"BTC-USDT","side":"BUY","price":"50000","qty":"0.001","commission":"0.01","commissionAsset":"USDT","role":"MAKER","orderId":"456","tradeId":"789"}]}}`
    pub fn parse_user_trades(response: &Value, is_futures: bool) -> ExchangeResult<Vec<UserTrade>> {
        let data = Self::extract_data(response)?;

        // Spot wraps in data.list; swap wraps in data.fills
        let arr = if is_futures {
            data.get("fills")
                .and_then(|v| v.as_array())
                .ok_or_else(|| ExchangeError::Parse("Missing 'fills' array in swap trade response".to_string()))?
        } else {
            data.get("list")
                .and_then(|v| v.as_array())
                .ok_or_else(|| ExchangeError::Parse("Missing 'list' array in spot trade response".to_string()))?
        };

        arr.iter()
            .map(|item| {
                let id = item.get("tradeId")
                    .and_then(|v| v.as_str().map(|s| s.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string())))
                    .ok_or_else(|| ExchangeError::Parse("Missing 'tradeId' in trade".to_string()))?;

                let order_id = item.get("orderId")
                    .and_then(|v| v.as_str().map(|s| s.to_string())
                        .or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();

                let symbol = Self::get_str(item, "symbol")
                    .unwrap_or("")
                    .to_string();

                let side = match Self::get_str(item, "side").unwrap_or("BUY").to_uppercase().as_str() {
                    "SELL" | "SHORT" => OrderSide::Sell,
                    _ => OrderSide::Buy,
                };

                let price = Self::require_f64(item, "price")?;
                let quantity = Self::require_f64(item, "qty")?;

                let commission = Self::get_f64(item, "commission").unwrap_or(0.0).abs();
                let commission_asset = Self::get_str(item, "commissionAsset")
                    .unwrap_or("")
                    .to_string();

                // Spot: isMaker bool; Swap: role string "MAKER"/"TAKER"
                let is_maker = if is_futures {
                    Self::get_str(item, "role")
                        .map(|r| r.to_uppercase() == "MAKER")
                        .unwrap_or(false)
                } else {
                    item.get("isMaker")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                };

                // Spot: "time" ms; Swap: "filledTm" ms (string or number)
                let timestamp = if is_futures {
                    item.get("filledTm")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok())
                            .or_else(|| v.as_i64()))
                        .unwrap_or(0)
                } else {
                    item.get("time")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0)
                };

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "code": 0,
            "msg": "",
            "data": {
                "price": "43302.50"
            }
        });

        let price = BingxParser::parse_price(&response).unwrap();
        assert!((price - 43302.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": 0,
            "msg": "",
            "data": {
                "symbol": "BTC-USDT",
                "lastPrice": "43302.50",
                "bidPrice": "43302.00",
                "askPrice": "43303.00",
                "highPrice": "43500.00",
                "lowPrice": "41800.00",
                "volume": "12458.250000",
                "quoteVolume": "536428950.25",
                "priceChange": "1250.50",
                "priceChangePercent": "2.98",
                "closeTime": 1649404670162i64
            }
        });

        let ticker = BingxParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-USDT");
        assert!((ticker.last_price - 43302.50).abs() < f64::EPSILON);
        assert!(ticker.bid_price.unwrap() < ticker.ask_price.unwrap());
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "code": 0,
            "msg": "",
            "data": {
                "bids": [
                    ["43302.00", "0.521000"],
                    ["43301.50", "0.234000"]
                ],
                "asks": [
                    ["43303.00", "0.321000"],
                    ["43303.50", "0.892000"]
                ]
            }
        });

        let orderbook = BingxParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 43302.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "code": 100401,
            "msg": "AUTHENTICATION_FAIL",
            "data": null
        });

        let result = BingxParser::extract_data(&response);
        assert!(result.is_err());

        match result {
            Err(ExchangeError::Api { code, message }) => {
                assert_eq!(code, 100401);
                assert_eq!(message, "AUTHENTICATION_FAIL");
            }
            _ => panic!("Expected API error"),
        }
    }
}
