//! # Hyperliquid Response Parser
//!
//! Parse JSON responses from Hyperliquid API.
//!
//! ## Response Formats
//!
//! ### Info Responses
//! Direct data (no wrapper):
//! ```json
//! {
//!   "universe": [...],
//!   "tokens": [...]
//! }
//! ```
//!
//! ### Exchange Responses
//! Wrapped format:
//! ```json
//! {
//!   "status": "ok",
//!   "response": {
//!     "type": "order",
//!     "data": { ... }
//!   }
//! }
//! ```

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, TradeSide, SymbolInfo,
    UserTrade, FundingPayment,
};

/// Parser for Hyperliquid API responses
pub struct HyperliquidParser;

impl HyperliquidParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse f64 from string or number
    ///
    /// Hyperliquid returns all numbers as strings to preserve precision
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
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    /// Check if exchange response is error
    pub fn check_exchange_response(response: &Value) -> ExchangeResult<()> {
        if let Some(status) = Self::get_str(response, "status") {
            if status != "ok" {
                let error = Self::get_str(response, "response")
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: -1,
                    message: error.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Extract response data from exchange endpoint response
    pub fn extract_exchange_data(response: &Value) -> ExchangeResult<&Value> {
        Self::check_exchange_response(response)?;

        response.get("response")
            .and_then(|r| r.get("data"))
            .ok_or_else(|| ExchangeError::Parse("Missing response.data field".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from allMids or metaAndAssetCtxs response
    pub fn parse_price(response: &Value, symbol: &str) -> ExchangeResult<f64> {
        // allMids response: { "BTC": "50123.45", "ETH": "2500.67" }
        if let Some(_mids) = response.as_object() {
            return Self::get_f64(response, symbol)
                .ok_or_else(|| ExchangeError::Parse(format!("Symbol {} not found in mids", symbol)));
        }

        // metaAndAssetCtxs response
        if let Some(assets) = response.as_array() {
            for asset in assets.iter() {
                if let Some(ctx) = asset.get("ctx") {
                    if let Some(mid) = Self::get_f64(ctx, "midPx") {
                        // TODO: Need to match against symbol name from universe
                        // For now, use index-based matching
                        return Ok(mid);
                    }
                }
            }
        }

        Err(ExchangeError::Parse("Cannot parse price from response".to_string()))
    }

    /// Parse order book from l2Book response
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let levels = response.get("levels")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'levels' array".to_string()))?;

        if levels.len() != 2 {
            return Err(ExchangeError::Parse("Expected [bids, asks] in levels".to_string()));
        }

        let parse_levels = |level_array: &Value| -> Vec<(f64, f64)> {
            level_array.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let px = Self::get_f64(level, "px")?;
                            let sz = Self::get_f64(level, "sz")?;
                            Some((px, sz))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: Self::get_i64(response, "time").unwrap_or(0),
            bids: parse_levels(&levels[0]), // levels[0] = bids
            asks: parse_levels(&levels[1]), // levels[1] = asks
            sequence: None,
        })
    }

    /// Parse klines from candleSnapshot response
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let candles = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of candles".to_string()))?;

        let mut klines = Vec::with_capacity(candles.len());

        for candle in candles {
            // Format: { "t": 1704067200000, "T": 1704067259999, "s": "BTC", "i": "15m",
            //           "o": "50100.0", "c": "50200.0", "h": "50250.0", "l": "50050.0",
            //           "v": "123.456", "n": 1234 }
            let open_time = Self::get_i64(candle, "t").unwrap_or(0);
            let close_time = Self::get_i64(candle, "T");

            klines.push(Kline {
                open_time,
                open: Self::get_f64(candle, "o").unwrap_or(0.0),
                high: Self::get_f64(candle, "h").unwrap_or(0.0),
                low: Self::get_f64(candle, "l").unwrap_or(0.0),
                close: Self::get_f64(candle, "c").unwrap_or(0.0),
                volume: Self::get_f64(candle, "v").unwrap_or(0.0),
                close_time,
                quote_volume: None,
                trades: Self::get_i64(candle, "n").map(|n| n as u64),
            });
        }

        Ok(klines)
    }

    /// Parse ticker from metaAndAssetCtxs response
    pub fn parse_ticker(response: &Value, index: usize) -> ExchangeResult<Ticker> {
        let assets = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of assets".to_string()))?;

        if index >= assets.len() {
            return Err(ExchangeError::Parse(format!("Asset index {} out of bounds", index)));
        }

        let asset = &assets[index];
        let ctx = asset.get("ctx")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ctx' field".to_string()))?;

        // Calculate 24h change from prevDayPx and markPx
        let prev_day_px = Self::get_f64(ctx, "prevDayPx");
        let mark_px = Self::get_f64(ctx, "markPx").unwrap_or(0.0);

        let (price_change_24h, price_change_percent_24h) = if let Some(prev) = prev_day_px {
            let change = mark_px - prev;
            let change_pct = if prev > 0.0 {
                (change / prev) * 100.0
            } else {
                0.0
            };
            (Some(change), Some(change_pct))
        } else {
            (None, None)
        };

        Ok(Ticker {
            symbol: String::new(), // Will be filled by connector using metadata
            last_price: mark_px,
            bid_price: None, // Not in metaAndAssetCtxs
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: Self::get_f64(ctx, "dayNtlVlm"),
            quote_volume_24h: None,
            price_change_24h,
            price_change_percent_24h,
            timestamp: 0, // Not provided in response
        })
    }

    /// Parse funding rate from metaAndAssetCtxs or fundingHistory response
    pub fn parse_funding_rate(response: &Value, index: Option<usize>) -> ExchangeResult<FundingRate> {
        // If it's fundingHistory, response is array
        if let Some(history) = response.as_array() {
            let item = history.first()
                .ok_or_else(|| ExchangeError::Parse("Empty funding history".to_string()))?;

            return Ok(FundingRate {
                symbol: Self::get_str(item, "coin").unwrap_or("").to_string(),
                rate: Self::require_f64(item, "fundingRate")?,
                next_funding_time: None,
                timestamp: Self::get_i64(item, "time").unwrap_or(0),
            });
        }

        // metaAndAssetCtxs format
        let index = index.ok_or_else(|| ExchangeError::Parse("Asset index required".to_string()))?;
        let assets = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of assets".to_string()))?;

        if index >= assets.len() {
            return Err(ExchangeError::Parse(format!("Asset index {} out of bounds", index)));
        }

        let asset = &assets[index];
        let ctx = asset.get("ctx")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ctx' field".to_string()))?;

        Ok(FundingRate {
            symbol: String::new(), // Will be filled by connector
            rate: Self::require_f64(ctx, "funding")?,
            next_funding_time: None,
            timestamp: 0,
        })
    }

    /// Parse recent trades
    pub fn parse_recent_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        let trades = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of trades".to_string()))?;

        let mut result = Vec::with_capacity(trades.len());

        for trade in trades {
            let side = match Self::get_str(trade, "side").unwrap_or("B") {
                "A" => TradeSide::Sell,
                _ => TradeSide::Buy,
            };

            result.push(PublicTrade {
                id: Self::get_i64(trade, "tid")
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                symbol: Self::get_str(trade, "coin").unwrap_or("").to_string(),
                price: Self::require_f64(trade, "px")?,
                quantity: Self::get_f64(trade, "sz").unwrap_or(0.0),
                side,
                timestamp: Self::get_i64(trade, "time").unwrap_or(0),
            });
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse perpetuals account state (clearinghouseState)
    pub fn parse_perp_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let margin_summary = response.get("marginSummary")
            .ok_or_else(|| ExchangeError::Parse("Missing 'marginSummary'".to_string()))?;

        let account_value = Self::get_f64(margin_summary, "accountValue").unwrap_or(0.0);
        let total_raw_usd = Self::get_f64(margin_summary, "totalRawUsd").unwrap_or(0.0);

        // In Hyperliquid, balance is in USDC for perpetuals
        Ok(vec![Balance {
            asset: "USDC".to_string(),
            free: total_raw_usd,
            locked: account_value - total_raw_usd,
            total: account_value,
        }])
    }

    /// Parse spot account state (spotClearinghouseState)
    pub fn parse_spot_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let balances_array = response.get("balances")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'balances' array".to_string()))?;

        let mut balances = Vec::new();

        for item in balances_array {
            let coin = Self::get_str(item, "coin").unwrap_or("").to_string();
            if coin.is_empty() { continue; }

            let total = Self::get_f64(item, "total").unwrap_or(0.0);
            let hold = Self::get_f64(item, "hold").unwrap_or(0.0);
            let free = total - hold;

            balances.push(Balance {
                asset: coin,
                free,
                locked: hold,
                total,
            });
        }

        Ok(balances)
    }

    /// Parse positions from clearinghouseState
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let asset_positions = response.get("assetPositions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'assetPositions' array".to_string()))?;

        let mut positions = Vec::new();

        for asset_pos in asset_positions {
            let position = asset_pos.get("position")
                .ok_or_else(|| ExchangeError::Parse("Missing 'position' field".to_string()))?;

            // szi is signed size (positive = long, negative = short)
            let szi = Self::get_f64(position, "szi").unwrap_or(0.0);

            // Skip empty positions
            if szi.abs() < f64::EPSILON {
                continue;
            }

            let side = if szi > 0.0 {
                PositionSide::Long
            } else {
                PositionSide::Short
            };

            // Parse leverage
            let leverage = position.get("leverage")
                .and_then(|lev| {
                    // Leverage can be {"type": "cross", "value": 5} or {"type": "isolated", "value": 10}
                    lev.get("value").and_then(|v| v.as_u64()).map(|v| v as u32)
                })
                .unwrap_or(1);

            positions.push(Position {
                symbol: Self::get_str(position, "coin").unwrap_or("").to_string(),
                side,
                quantity: szi.abs(),
                entry_price: Self::get_f64(position, "entryPx").unwrap_or(0.0),
                mark_price: None, // Not in position object, need to get from ctx
                unrealized_pnl: Self::get_f64(position, "unrealizedPnl").unwrap_or(0.0),
                realized_pnl: None,
                leverage,
                liquidation_price: Self::get_f64(position, "liquidationPx"),
                margin: Self::get_f64(position, "marginUsed"),
                margin_type: {
                    let lev_type = position.get("leverage")
                        .and_then(|lev| lev.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("cross");
                    if lev_type.eq_ignore_ascii_case("isolated") {
                        crate::core::MarginType::Isolated
                    } else {
                        crate::core::MarginType::Cross
                    }
                },
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    /// Parse orders from openOrders response
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let orders = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        orders.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Parse single order data
    fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("B") {
            "A" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        // Parse order type from "t" field.
        // Hyperliquid "t" is an object: {"limit": {"tif": "Gtc"}} or
        // {"trigger": {"triggerPx": "65000", "isMarket": true, "tpsl": "sl"}}
        let order_type = if let Some(t_obj) = data.get("t") {
            if t_obj.get("limit").is_some() {
                let price = Self::get_f64(data, "limitPx").unwrap_or(0.0);
                OrderType::Limit { price }
            } else if let Some(trigger) = t_obj.get("trigger") {
                let trigger_px = trigger.get("triggerPx")
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64()))
                    .unwrap_or(0.0);
                let is_market = trigger.get("isMarket").and_then(|v| v.as_bool()).unwrap_or(true);
                if is_market {
                    OrderType::StopMarket { stop_price: trigger_px }
                } else {
                    let limit_price = Self::get_f64(data, "limitPx").unwrap_or(trigger_px);
                    OrderType::StopLimit { stop_price: trigger_px, limit_price }
                }
            } else {
                // Unknown sub-type — fall back to limit with actual price
                let price = Self::get_f64(data, "limitPx").unwrap_or(0.0);
                OrderType::Limit { price }
            }
        } else {
            let price = Self::get_f64(data, "limitPx").unwrap_or(0.0);
            OrderType::Limit { price }
        };

        let orig_sz = Self::get_f64(data, "origSz").unwrap_or(0.0);
        let sz = Self::get_f64(data, "sz").unwrap_or(0.0);
        let filled_quantity = orig_sz - sz;

        let status = if sz.abs() < f64::EPSILON {
            OrderStatus::Filled
        } else if filled_quantity > 0.0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::New
        };

        Ok(Order {
            id: Self::get_i64(data, "oid")
                .map(|id| id.to_string())
                .unwrap_or_default(),
            client_order_id: Self::get_str(data, "cloid").map(String::from),
            symbol: Self::get_str(data, "coin").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "limitPx"),
            stop_price: None,
            quantity: orig_sz,
            filled_quantity,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "timestamp").unwrap_or(0),
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse a single order from `orderStatus` response
    pub fn parse_order_status(response: &Value) -> ExchangeResult<Order> {
        // Response: { "order": {...}, "status": "open", "statusTimestamp": ... }
        let order_data = response.get("order")
            .ok_or_else(|| ExchangeError::Parse("Missing 'order' field in orderStatus".to_string()))?;

        let status_str = Self::get_str(response, "status").unwrap_or("open");
        let order_status = match status_str {
            "open" => OrderStatus::Open,
            "filled" => OrderStatus::Filled,
            "canceled" | "marginCanceled" => OrderStatus::Canceled,
            "rejected" => OrderStatus::Rejected,
            "triggered" => OrderStatus::Filled, // trigger fired
            "partiallyFilled" => OrderStatus::PartiallyFilled,
            "expired" => OrderStatus::Expired,
            _ => OrderStatus::Open,
        };

        let side = match Self::get_str(order_data, "side").unwrap_or("B") {
            "A" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let orig_sz = Self::get_f64(order_data, "origSz").unwrap_or(0.0);
        let sz = Self::get_f64(order_data, "sz").unwrap_or(0.0);
        let filled_quantity = (orig_sz - sz).max(0.0);

        Ok(Order {
            id: Self::get_i64(order_data, "oid")
                .map(|id| id.to_string())
                .unwrap_or_default(),
            client_order_id: Self::get_str(order_data, "cloid").map(String::from),
            symbol: Self::get_str(order_data, "coin").unwrap_or("").to_string(),
            side,
            order_type: OrderType::Limit {
                price: Self::get_f64(order_data, "limitPx").unwrap_or(0.0),
            },
            status: order_status,
            price: Self::get_f64(order_data, "limitPx"),
            stop_price: None,
            quantity: orig_sz,
            filled_quantity,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(order_data, "timestamp").unwrap_or(0),
            updated_at: Self::get_i64(response, "statusTimestamp"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse historical orders from `historicalOrders` response.
    ///
    /// Response is an array of `{order: {...}, status: "filled"|"canceled", statusTimestamp: ...}`
    pub fn parse_historical_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        // historicalOrders returns array of order-status pairs
        if let Some(arr) = response.as_array() {
            return arr.iter().map(|item| {
                // Each item is { order: {...}, status: "...", statusTimestamp: ... }
                if let Some(order_data) = item.get("order") {
                    let status_str = item.get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("filled");
                    let status = match status_str {
                        "filled" => OrderStatus::Filled,
                        "canceled" | "marginCanceled" => OrderStatus::Canceled,
                        "rejected" => OrderStatus::Rejected,
                        "triggered" => OrderStatus::Filled,
                        "partiallyFilled" => OrderStatus::PartiallyFilled,
                        "expired" => OrderStatus::Expired,
                        _ => OrderStatus::Filled,
                    };
                    let side = match Self::get_str(order_data, "side").unwrap_or("B") {
                        "A" => OrderSide::Sell,
                        _ => OrderSide::Buy,
                    };
                    let orig_sz = Self::get_f64(order_data, "origSz").unwrap_or(0.0);
                    let sz = Self::get_f64(order_data, "sz").unwrap_or(0.0);
                    let filled_quantity = (orig_sz - sz).max(0.0);

                    Ok(Order {
                        id: Self::get_i64(order_data, "oid")
                            .map(|id| id.to_string())
                            .unwrap_or_default(),
                        client_order_id: Self::get_str(order_data, "cloid").map(String::from),
                        symbol: Self::get_str(order_data, "coin").unwrap_or("").to_string(),
                        side,
                        order_type: OrderType::Limit {
                            price: Self::get_f64(order_data, "limitPx").unwrap_or(0.0),
                        },
                        status,
                        price: Self::get_f64(order_data, "limitPx"),
                        stop_price: None,
                        quantity: orig_sz,
                        filled_quantity,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: Self::get_i64(order_data, "timestamp").unwrap_or(0),
                        updated_at: item.get("statusTimestamp").and_then(|v| v.as_i64()),
                        time_in_force: crate::core::TimeInForce::Gtc,
                    })
                } else {
                    // Try treating the item itself as an order (userFills format)
                    Self::parse_fill_as_order(item)
                }
            }).collect();
        }

        Ok(Vec::new())
    }

    /// Parse a userFill as an Order (for order history queries using userFills endpoint)
    fn parse_fill_as_order(fill: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(fill, "side").unwrap_or("B") {
            "A" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        Ok(Order {
            id: Self::get_i64(fill, "oid")
                .map(|id| id.to_string())
                .unwrap_or_default(),
            client_order_id: Self::get_str(fill, "cloid").map(String::from),
            symbol: Self::get_str(fill, "coin").unwrap_or("").to_string(),
            side,
            order_type: OrderType::Limit { price: Self::get_f64(fill, "px").unwrap_or(0.0) },
            status: OrderStatus::Filled,
            price: Self::get_f64(fill, "px"),
            stop_price: None,
            quantity: Self::get_f64(fill, "sz").unwrap_or(0.0),
            filled_quantity: Self::get_f64(fill, "sz").unwrap_or(0.0),
            average_price: Self::get_f64(fill, "px"),
            commission: Self::get_f64(fill, "fee"),
            commission_asset: Self::get_str(fill, "feeToken").map(String::from),
            created_at: Self::get_i64(fill, "time").unwrap_or(0),
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse funding rate for a specific symbol from metaAndAssetCtxs response.
    ///
    /// metaAndAssetCtxs returns [meta_obj, [ctx_obj, ctx_obj, ...]]
    /// where the index of ctx corresponds to the universe index.
    pub fn parse_funding_rate_for_symbol(response: &Value, symbol: &str) -> ExchangeResult<FundingRate> {
        // Response is [{"universe": [...]}, [{"funding": "..."}, ...]]
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array from metaAndAssetCtxs".to_string()))?;

        if arr.len() < 2 {
            return Err(ExchangeError::Parse("metaAndAssetCtxs response too short".to_string()));
        }

        let meta = &arr[0];
        let ctxs = arr[1].as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected ctxs array at index 1".to_string()))?;

        let universe = meta.get("universe")
            .and_then(|u| u.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing universe in meta".to_string()))?;

        // Find symbol index
        let idx = universe.iter().position(|item| {
            item.get("name")
                .and_then(|v| v.as_str())
                .map(|n| n.eq_ignore_ascii_case(symbol))
                .unwrap_or(false)
        }).ok_or_else(|| ExchangeError::Parse(
            format!("Symbol '{}' not found in universe", symbol)
        ))?;

        if idx >= ctxs.len() {
            return Err(ExchangeError::Parse(
                format!("Asset context index {} out of bounds (len={})", idx, ctxs.len())
            ));
        }

        let ctx = &ctxs[idx];
        let funding_str = ctx.get("funding")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let rate = funding_str.parse::<f64>()
            .map_err(|_| ExchangeError::Parse(format!("Invalid funding rate: {}", funding_str)))?;

        Ok(FundingRate {
            symbol: symbol.to_uppercase(),
            rate,
            next_funding_time: None, // Funding occurs every hour
            timestamp: 0,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Hyperliquid meta response.
    ///
    /// Perp metadata response format (from /info with type=meta):
    /// ```json
    /// {"universe":[{"name":"BTC","szDecimals":5,"maxLeverage":40,"onlyIsolated":false},...],"tokens":[...]}
    /// ```
    pub fn parse_perp_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let universe = response.get("universe")
            .and_then(|u| u.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'universe' array in meta response".to_string()))?;

        let mut symbols = Vec::with_capacity(universe.len());

        for (idx, item) in universe.iter().enumerate() {
            let name = match item.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => continue,
            };

            // Hyperliquid perp symbols are like "BTC" (not "BTC-USDC"), quote is always USD
            let base_asset = name.to_string();
            let quote_asset = "USD".to_string();

            let sz_decimals = item.get("szDecimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as u8;

            // symbol format: use name + "-PERP" for clarity
            let symbol = format!("{}-PERP", name);

            // step_size = 10^(-sz_decimals)
            let step_size = Some(10f64.powi(-(sz_decimals as i32)));

            let _ = idx; // index used if needed for ordering

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision: 6,
                quantity_precision: sz_decimals,
                min_quantity: step_size,
                max_quantity: None,
                tick_size: None,
                step_size,
                min_notional: None,
                account_type: Default::default(),
            });
        }

        Ok(symbols)
    }

    /// Parse exchange info from Hyperliquid spot meta response.
    ///
    /// Spot metadata response format (from /info with type=spotMeta):
    /// ```json
    /// {"universe":[{"tokens":[0,1],"name":"@0","index":0,"isCanonical":true},...],"tokens":[{"name":"PURR","szDecimals":0,"weiDecimals":0,"index":0,"tokenId":"0x...","isCanonical":true},...],"universe":[...]}
    /// ```
    /// Parse user fills (trades) from a `userFills` or `userFillsByTime` response.
    ///
    /// Each fill element format:
    /// ```json
    /// {"tid":123,"oid":456,"coin":"BTC","side":"B","px":"50000","sz":"0.001",
    ///  "fee":"0.01","feeToken":"USDC","time":1672531200000,"crossed":true}
    /// ```
    /// `side`: "B" = buy, "A" = sell. `crossed`: true = taker, false = maker.
    pub fn parse_user_fills(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let fills = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of fills".to_string()))?;

        fills.iter().map(|fill| {
            let id = fill.get("tid")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let order_id = fill.get("oid")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let symbol = Self::get_str(fill, "coin").unwrap_or("").to_string();
            let side = match Self::get_str(fill, "side").unwrap_or("B") {
                "A" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };
            let price = Self::get_f64(fill, "px").unwrap_or(0.0);
            let quantity = Self::get_f64(fill, "sz").unwrap_or(0.0);
            let commission = Self::get_f64(fill, "fee").unwrap_or(0.0).abs();
            let commission_asset = Self::get_str(fill, "feeToken")
                .unwrap_or("USDC")
                .to_string();
            // crossed: true = taker (not maker), false = maker
            let is_maker = !fill.get("crossed")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let timestamp = fill.get("time")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

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

    pub fn parse_spot_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        // spotMeta response has a different structure
        let universe = match response.get("universe").and_then(|u| u.as_array()) {
            Some(u) => u,
            None => return Ok(vec![]),
        };

        let tokens = match response.get("tokens").and_then(|t| t.as_array()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let mut symbols = Vec::new();

        for market in universe {
            let token_indices = match market.get("tokens").and_then(|t| t.as_array()) {
                Some(t) => t,
                None => continue,
            };

            if token_indices.len() < 2 {
                continue;
            }

            let base_idx = token_indices[0].as_u64().unwrap_or(0) as usize;
            let quote_idx = token_indices[1].as_u64().unwrap_or(0) as usize;

            let base_token = tokens.get(base_idx);
            let quote_token = tokens.get(quote_idx);

            let base_asset = base_token
                .and_then(|t| t.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = quote_token
                .and_then(|t| t.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("USDC")
                .to_string();

            if base_asset.is_empty() {
                continue;
            }

            let sz_decimals = base_token
                .and_then(|t| t.get("szDecimals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as u8;

            let market_name = market.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&base_asset);

            let symbol = format!("{}/{}", base_asset, quote_asset);
            let step_size = Some(10f64.powi(-(sz_decimals as i32)));

            let _ = market_name;

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision: 6,
                quantity_precision: sz_decimals,
                min_quantity: step_size,
                max_quantity: None,
                tick_size: None,
                step_size,
                min_notional: None,
                account_type: Default::default(),
            });
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse historical funding payments from `POST /info` with `type: userFunding`.
    ///
    /// Response:
    /// ```json
    /// [
    ///   {"time":1672531200000,"coin":"BTC","fundingRate":"0.0001",
    ///    "payment":"-0.01","positionSize":"0.1"}
    /// ]
    /// ```
    /// All numbers are returned as strings.
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let list = response.as_array()
            .ok_or_else(|| ExchangeError::Parse(
                "Expected array for userFunding response".to_string(),
            ))?;

        let mut payments = Vec::with_capacity(list.len());
        for item in list {
            let symbol = item.get("coin")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let funding_rate = Self::get_f64(item, "fundingRate").unwrap_or(0.0);

            let position_size = Self::get_f64(item, "positionSize").unwrap_or(0.0);

            let payment = Self::get_f64(item, "payment").unwrap_or(0.0);

            // HyperLiquid perps settle in USDC
            let asset = "USDC".to_string();

            // time is already milliseconds
            let timestamp = item.get("time")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            payments.push(FundingPayment {
                symbol,
                funding_rate,
                position_size,
                payment,
                asset,
                timestamp,
            });
        }
        Ok(payments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "coin": "BTC",
            "time": 1704067200000i64,
            "levels": [
                [
                    {"px": "50123.5", "sz": "1.234", "n": 3},
                    {"px": "50123.0", "sz": "2.567", "n": 5}
                ],
                [
                    {"px": "50124.0", "sz": "0.567", "n": 1},
                    {"px": "50124.5", "sz": "3.456", "n": 7}
                ]
            ]
        });

        let orderbook = HyperliquidParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 50123.5).abs() < f64::EPSILON);
        assert!((orderbook.asks[0].0 - 50124.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_klines() {
        let response = json!([
            {
                "t": 1704067200000i64,
                "T": 1704067259999i64,
                "s": "BTC",
                "i": "15m",
                "o": "50100.0",
                "c": "50200.0",
                "h": "50250.0",
                "l": "50050.0",
                "v": "123.456",
                "n": 1234
            }
        ]);

        let klines = HyperliquidParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert!((klines[0].open - 50100.0).abs() < f64::EPSILON);
        assert!((klines[0].high - 50250.0).abs() < f64::EPSILON);
        assert_eq!(klines[0].open_time, 1704067200000);
    }

    #[test]
    fn test_parse_recent_trades() {
        let response = json!([
            {
                "coin": "BTC",
                "side": "B",
                "px": "50123.45",
                "sz": "0.5",
                "hash": "0x...",
                "time": 1704067200000i64,
                "tid": 123456789i64,
                "fee": "0.25"
            }
        ]);

        let trades = HyperliquidParser::parse_recent_trades(&response).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].side, TradeSide::Buy);
        assert!((trades[0].price - 50123.45).abs() < f64::EPSILON);
    }
}
