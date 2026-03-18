//! # Lighter Response Parser
//!
//! Parse JSON responses from Lighter API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, PublicTrade, FundingRate,
    OrderSide, UserTrade,
};

/// Parser for Lighter API responses
pub struct LighterParser;

impl LighterParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response indicates success (code 200)
    pub fn check_success(response: &Value) -> ExchangeResult<()> {
        if let Some(code) = response.get("code").and_then(|c| c.as_i64()) {
            if code != 200 {
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: message.to_string(),
                });
            }
        }
        Ok(())
    }

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
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Parse integer from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    /// Parse required integer
    fn require_i64(data: &Value, key: &str) -> ExchangeResult<i64> {
        Self::get_i64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from orderBookDetails response
    ///
    /// Returns last_trade_price from the first market in the response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_success(response)?;

        // Try order_book_details array first (perpetuals)
        if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            if let Some(first) = details.first() {
                if let Some(price) = Self::get_f64(first, "last_trade_price") {
                    return Ok(price);
                }
            }
        }

        // Try spot_order_book_details array (spot markets)
        if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            if let Some(first) = details.first() {
                if let Some(price) = Self::get_f64(first, "last_trade_price") {
                    return Ok(price);
                }
            }
        }

        Err(ExchangeError::Parse("No price data found".to_string()))
    }

    /// Parse ticker from orderBookDetails response
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_success(response)?;

        // Try order_book_details array first (perpetuals)
        let data = if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            details.first()
                .ok_or_else(|| ExchangeError::Parse("Empty order_book_details".to_string()))?
        } else if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            details.first()
                .ok_or_else(|| ExchangeError::Parse("Empty spot_order_book_details".to_string()))?
        } else {
            return Err(ExchangeError::Parse("No ticker data found".to_string()));
        };

        let symbol = Self::require_str(data, "symbol")?;
        let last_price = Self::require_f64(data, "last_trade_price")?;

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: Self::get_f64(data, "daily_price_high"),
            low_24h: Self::get_f64(data, "daily_price_low"),
            volume_24h: Self::get_f64(data, "daily_base_token_volume"),
            quote_volume_24h: Self::get_f64(data, "daily_quote_token_volume"),
            price_change_24h: Self::get_f64(data, "daily_price_change"),
            price_change_percent_24h: data.get("daily_price_change")
                .and_then(Self::parse_f64)
                .and_then(|change| {
                    // Calculate percentage: (change / (last_price - change)) * 100
                    let prev_price = last_price - change;
                    if prev_price != 0.0 {
                        Some((change / prev_price) * 100.0)
                    } else {
                        None
                    }
                }),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Parse orderbook from orderBookOrders response
    ///
    /// Note: Lighter doesn't have a dedicated orderbook snapshot endpoint.
    /// This is a placeholder that would need the actual orderBookOrders data.
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_success(response)?;

        // Lighter's orderBookOrders returns full order list, not aggregated levels
        // For now, return empty orderbook - this would need proper aggregation
        Ok(OrderBook {
            timestamp: chrono::Utc::now().timestamp_millis(),
            bids: Vec::new(),
            asks: Vec::new(),
            sequence: None,
        })
    }

    /// Parse klines/candlesticks
    ///
    /// Handles two response formats:
    /// - New `/api/v1/candles` format: `"c"` array with abbreviated field names and ms timestamps
    /// - Legacy format: `"candlesticks"` array with full field names and second timestamps
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_success(response)?;

        // Try new /api/v1/candles format first ("c" array, short field names, ms timestamps)
        if let Some(candles) = response.get("c").and_then(|v| v.as_array()) {
            let mut klines = Vec::with_capacity(candles.len());
            for candle in candles {
                klines.push(Kline {
                    open_time: candle.get("t").and_then(|v| v.as_i64()).unwrap_or(0),
                    open: candle.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    high: candle.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    low: candle.get("l").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    close: candle.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    volume: candle.get("v").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    quote_volume: candle.get("V").and_then(|v| v.as_f64()),
                    close_time: None,
                    trades: None,
                });
            }
            return Ok(klines);
        }

        // Fall back to legacy "candlesticks" format (full field names, second timestamps)
        let candlesticks = response.get("candlesticks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candlesticks array".to_string()))?;

        let mut klines = Vec::with_capacity(candlesticks.len());

        for candle in candlesticks {
            let timestamp = Self::require_i64(candle, "timestamp")?;
            let open = Self::require_f64(candle, "open")?;
            let high = Self::require_f64(candle, "high")?;
            let low = Self::require_f64(candle, "low")?;
            let close = Self::require_f64(candle, "close")?;
            let volume = Self::require_f64(candle, "volume")?;
            let quote_volume = Self::get_f64(candle, "quote_volume");

            klines.push(Kline {
                open_time: timestamp * 1000, // seconds to milliseconds
                open,
                high,
                low,
                close,
                volume,
                quote_volume,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse recent trades
    pub fn parse_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        Self::check_success(response)?;

        let trades = response.get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing trades array".to_string()))?;

        let mut result = Vec::with_capacity(trades.len());

        for trade in trades {
            let id = Self::require_i64(trade, "trade_id")?;
            let price = Self::require_f64(trade, "price")?;
            let qty = Self::require_f64(trade, "size")?;
            let time = Self::require_i64(trade, "timestamp")?;
            let is_maker_ask = trade.get("is_maker_ask")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            result.push(PublicTrade {
                id: id.to_string(),
                symbol: String::new(), // Will be set by caller
                price,
                quantity: qty,
                side: if is_maker_ask {
                    crate::core::types::TradeSide::Sell
                } else {
                    crate::core::types::TradeSide::Buy
                },
                timestamp: time * 1000, // seconds to milliseconds
            });
        }

        Ok(result)
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_success(response)?;

        let fundings = response.get("fundings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing fundings array".to_string()))?;

        let first = fundings.first()
            .ok_or_else(|| ExchangeError::Parse("Empty fundings array".to_string()))?;

        let funding_rate = Self::require_f64(first, "funding_rate")?;
        let timestamp = Self::require_i64(first, "timestamp")?;

        Ok(FundingRate {
            symbol: String::new(), // Symbol not in response, caller must set
            rate: funding_rate,
            next_funding_time: None,
            timestamp: timestamp * 1000, // seconds to milliseconds
        })
    }

    /// Parse trading pairs from orderBooks or orderBookDetails
    pub fn parse_trading_pairs(response: &Value) -> ExchangeResult<Vec<String>> {
        Self::check_success(response)?;

        let mut symbols = Vec::new();

        // Parse from order_books array
        if let Some(order_books) = response.get("order_books").and_then(|v| v.as_array()) {
            for book in order_books {
                if let Some(symbol) = Self::get_str(book, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        // Parse from order_book_details array (perpetuals)
        if let Some(details) = response.get("order_book_details").and_then(|v| v.as_array()) {
            for detail in details {
                if let Some(symbol) = Self::get_str(detail, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        // Parse from spot_order_book_details array (spot)
        if let Some(details) = response.get("spot_order_book_details").and_then(|v| v.as_array()) {
            for detail in details {
                if let Some(symbol) = Self::get_str(detail, "symbol") {
                    symbols.push(symbol.to_string());
                }
            }
        }

        if symbols.is_empty() {
            return Err(ExchangeError::Parse("No trading pairs found".to_string()));
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse account balances from `/api/v1/account` response.
    ///
    /// Extracts the `assets` array from the first account entry.
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<crate::core::types::Balance>> {
        Self::check_success(response)?;

        let accounts = response.get("accounts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'accounts' array".to_string()))?;

        let account = accounts.first()
            .ok_or_else(|| ExchangeError::Parse("Empty accounts array".to_string()))?;

        let assets = account.get("assets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'assets' in account".to_string()))?;

        let balances = assets.iter().map(|asset| {
            let symbol = Self::get_str(asset, "symbol").unwrap_or("USDC").to_string();
            // Lighter returns balances as integer strings (scaled by precision)
            // For USDC they appear to be already in human-readable form based on research docs
            let total: f64 = asset.get("balance")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            let locked: f64 = asset.get("locked_balance")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            let free = total - locked;

            crate::core::types::Balance {
                asset: symbol,
                free: free.max(0.0),
                locked,
                total,
            }
        }).collect();

        Ok(balances)
    }

    /// Parse perpetual positions from `/api/v1/account` response.
    ///
    /// Extracts the `positions` array from the first account entry.
    /// Skips positions with zero size.
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<crate::core::types::Position>> {
        Self::check_success(response)?;

        let accounts = response.get("accounts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'accounts' array".to_string()))?;

        let account = accounts.first()
            .ok_or_else(|| ExchangeError::Parse("Empty accounts array".to_string()))?;

        let positions_raw = account.get("positions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut positions = Vec::new();

        for pos in &positions_raw {
            let size: f64 = pos.get("position")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);

            // Skip closed positions
            if size.abs() < f64::EPSILON {
                continue;
            }

            // sign: 1 = Long, -1 = Short
            let sign: i64 = pos.get("sign")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);

            let side = if sign >= 0 {
                crate::core::types::PositionSide::Long
            } else {
                crate::core::types::PositionSide::Short
            };

            let symbol_raw = Self::get_str(pos, "symbol").unwrap_or("").to_string();
            let entry_price = pos.get("avg_entry_price")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            let unrealized_pnl = pos.get("unrealized_pnl")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            let realized_pnl = pos.get("realized_pnl")
                .and_then(Self::parse_f64);

            positions.push(crate::core::types::Position {
                symbol: symbol_raw,
                side,
                quantity: size,
                entry_price,
                mark_price: None,
                unrealized_pnl,
                realized_pnl,
                leverage: 1,
                liquidation_price: None,
                margin: None,
                margin_type: crate::core::types::MarginType::Cross,
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    }

    /// Parse open/active orders from `/api/v1/accountActiveOrders` response.
    ///
    /// The active orders response uses `initial_base_amount`, `market_index`, and `type`
    /// as field names (different from the inactive orders response).
    pub fn parse_open_orders(response: &Value) -> ExchangeResult<Vec<crate::core::types::Order>> {
        Self::check_success(response)?;

        let orders_raw = response.get("orders")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orders' array".to_string()))?;

        let orders = orders_raw.iter().map(|order| {
            let order_index = order.get("order_index")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let client_order_index = order.get("client_order_index")
                .and_then(|v| v.as_i64());
            // Active orders response uses "market_index", not "market_id"
            let market_index = order.get("market_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let side_str = Self::get_str(order, "side").unwrap_or("buy");
            let side = if side_str.eq_ignore_ascii_case("sell") {
                crate::core::types::OrderSide::Sell
            } else {
                crate::core::types::OrderSide::Buy
            };
            let price = order.get("price").and_then(Self::parse_f64);
            // Active orders uses "initial_base_amount" or "remaining_base_amount" for qty
            let quantity = order.get("initial_base_amount")
                .and_then(Self::parse_f64)
                .or_else(|| order.get("remaining_base_amount").and_then(Self::parse_f64))
                .unwrap_or(0.0);
            let filled_quantity = order.get("filled_base_amount")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            // Active orders status: "open", "in-progress", "pending"
            let status_str = Self::get_str(order, "status").unwrap_or("open");
            let status = match status_str {
                "open" | "in-progress" | "pending" => crate::core::types::OrderStatus::Open,
                "filled" => crate::core::types::OrderStatus::Filled,
                "cancelled" | "canceled" => crate::core::types::OrderStatus::Canceled,
                "expired" => crate::core::types::OrderStatus::Expired,
                _ => crate::core::types::OrderStatus::Open,
            };
            let created_at = Self::get_i64(order, "created_at")
                .map(|t| t * 1000) // seconds to ms
                .unwrap_or(0);
            let updated_at = Self::get_i64(order, "updated_at")
                .map(|t| t * 1000);
            // Active orders response uses "type", not "order_type"
            let order_type_str = Self::get_str(order, "type").unwrap_or("limit");
            let order_type = if order_type_str.eq_ignore_ascii_case("market") {
                crate::core::types::OrderType::Market
            } else {
                crate::core::types::OrderType::Limit { price: price.unwrap_or(0.0) }
            };

            crate::core::types::Order {
                id: order_index.to_string(),
                client_order_id: client_order_index.map(|i| i.to_string()),
                symbol: market_index.to_string(), // market_index; caller can resolve
                side,
                order_type,
                status,
                price,
                stop_price: order.get("trigger_price").and_then(Self::parse_f64),
                quantity,
                filled_quantity,
                average_price: price,
                commission: None,
                commission_asset: None,
                created_at,
                updated_at,
                time_in_force: crate::core::types::TimeInForce::Gtc,
            }
        }).collect();

        Ok(orders)
    }

    /// Parse user trades (fills) from a `GET /api/v1/trades` response.
    ///
    /// Expected shape:
    /// ```json
    /// {"code":200,"trades":[{"id":"123","order_id":"456","market":"BTCUSDC_Market",
    ///   "side":"buy","price":"50000","amount":"0.001","fee":"0.01",
    ///   "is_maker":true,"timestamp":1672531200000}]}
    /// ```
    pub fn parse_user_trades(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_success(response)?;

        let trades_raw = response.get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'trades' array".to_string()))?;

        trades_raw.iter().map(|trade| {
            let id = Self::get_str(trade, "id").unwrap_or("").to_string();
            let order_id = Self::get_str(trade, "order_id").unwrap_or("").to_string();
            // market field e.g. "BTCUSDC_Market" — strip suffix for readability
            let market_raw = Self::get_str(trade, "market").unwrap_or("");
            let symbol = market_raw
                .trim_end_matches("_Market")
                .to_string();
            let side_str = Self::get_str(trade, "side").unwrap_or("buy");
            let side = if side_str.eq_ignore_ascii_case("sell") {
                OrderSide::Sell
            } else {
                OrderSide::Buy
            };
            let price = Self::get_f64(trade, "price").unwrap_or(0.0);
            // Lighter uses "amount" for base quantity
            let quantity = Self::get_f64(trade, "amount").unwrap_or(0.0);
            let commission = Self::get_f64(trade, "fee").unwrap_or(0.0).abs();
            // Lighter fees are paid in USDC
            let commission_asset = "USDC".to_string();
            let is_maker = trade.get("is_maker")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let timestamp = trade.get("timestamp")
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

    /// Parse orders from `/api/v1/accountInactiveOrders` response.
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<crate::core::types::Order>> {
        Self::check_success(response)?;

        let orders_raw = response.get("orders")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'orders' array".to_string()))?;

        let orders = orders_raw.iter().map(|order| {
            let order_index = order.get("order_index")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let client_order_index = order.get("client_order_index")
                .and_then(|v| v.as_i64());
            let market_id = order.get("market_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let side_str = Self::get_str(order, "side").unwrap_or("buy");
            let side = if side_str.eq_ignore_ascii_case("sell") {
                crate::core::types::OrderSide::Sell
            } else {
                crate::core::types::OrderSide::Buy
            };
            let price = order.get("price").and_then(Self::parse_f64);
            let quantity = order.get("base_amount")
                .and_then(Self::parse_f64)
                .unwrap_or(0.0);
            let status_str = Self::get_str(order, "status").unwrap_or("filled");
            let status = match status_str {
                "filled" => crate::core::types::OrderStatus::Filled,
                "cancelled" | "canceled" => crate::core::types::OrderStatus::Canceled,
                "expired" => crate::core::types::OrderStatus::Expired,
                _ => crate::core::types::OrderStatus::Filled,
            };
            let created_at = Self::get_i64(order, "created_at")
                .map(|t| t * 1000) // seconds to ms
                .unwrap_or(0);
            let updated_at = Self::get_i64(order, "updated_at")
                .map(|t| t * 1000);

            let order_type_str = Self::get_str(order, "order_type").unwrap_or("limit");
            let order_type = if order_type_str.eq_ignore_ascii_case("market") {
                crate::core::types::OrderType::Market
            } else {
                crate::core::types::OrderType::Limit { price: price.unwrap_or(0.0) }
            };

            crate::core::types::Order {
                id: order_index.to_string(),
                client_order_id: client_order_index.map(|i| i.to_string()),
                symbol: market_id.to_string(), // market_id; caller can resolve
                side,
                order_type,
                status,
                price,
                stop_price: None,
                quantity,
                filled_quantity: if matches!(status, crate::core::types::OrderStatus::Filled) { quantity } else { 0.0 },
                average_price: price,
                commission: None,
                commission_asset: None,
                created_at,
                updated_at,
                time_in_force: crate::core::types::TimeInForce::Gtc,
            }
        }).collect();

        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_success() {
        let success = json!({"code": 200, "message": "success"});
        assert!(LighterParser::check_success(&success).is_ok());

        let error = json!({"code": 400, "message": "Bad Request"});
        assert!(LighterParser::check_success(&error).is_err());
    }

    #[test]
    fn test_parse_klines_new_format() {
        // Actual /api/v1/candles response format
        let response = json!({
            "code": 200,
            "r": "1h",
            "c": [
                {
                    "t": 1740801600000i64,
                    "o": 85333.1,
                    "h": 86558.4,
                    "l": 85327.1,
                    "c": 86221.8,
                    "v": 17.97121,
                    "V": 1542622.63271,
                    "i": 3483696
                }
            ]
        });

        let klines = LighterParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1740801600000i64);
        assert_eq!(klines[0].open, 85333.1);
        assert_eq!(klines[0].high, 86558.4);
        assert_eq!(klines[0].low, 85327.1);
        assert_eq!(klines[0].close, 86221.8);
        assert_eq!(klines[0].volume, 17.97121);
        assert_eq!(klines[0].quote_volume, Some(1542622.63271));
    }

    #[test]
    fn test_parse_klines_legacy_format() {
        // Legacy "candlesticks" format with string values and second timestamps
        let response = json!({
            "code": 200,
            "message": "success",
            "candlesticks": [
                {
                    "timestamp": 1640995200,
                    "open": "3020.00",
                    "high": "3030.00",
                    "low": "3015.00",
                    "close": "3024.66",
                    "volume": "235.25",
                    "quote_volume": "93566.25"
                }
            ]
        });

        let klines = LighterParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1640995200 * 1000);
        assert_eq!(klines[0].open, 3020.0);
        assert_eq!(klines[0].close, 3024.66);
        assert_eq!(klines[0].quote_volume, Some(93566.25));
    }

    #[test]
    fn test_parse_trades() {
        let response = json!({
            "code": 200,
            "message": "success",
            "trades": [
                {
                    "trade_id": 12345,
                    "price": "3024.66",
                    "size": "1.5",
                    "timestamp": 1640995200,
                    "is_maker_ask": true
                }
            ]
        });

        let trades = LighterParser::parse_trades(&response).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 3024.66);
        assert_eq!(trades[0].quantity, 1.5);
    }
}
