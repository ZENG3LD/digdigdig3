//! # Coinbase Parser
//!
//! Parsing Coinbase Advanced Trade API responses to internal types.
//!
//! ## Response Structure
//!
//! Success response (direct object, no wrapper):
//! ```json
//! {
//!   "field1": "value1",
//!   "field2": "value2"
//! }
//! ```
//!
//! Error response:
//! ```json
//! {
//!   "error": "error_type",
//!   "message": "Error description"
//! }
//! ```
//!
//! ## Key Differences from Bybit/KuCoin
//!
//! - No response wrapper (direct objects vs `{code, data}` or `{retCode, result}`)
//! - Timestamps in RFC3339 format (vs milliseconds)
//! - Klines as objects with named fields (vs arrays)
//! - Klines sorted descending (newest first) vs ascending
//! - All numeric values as strings

use serde_json::Value;
use chrono::DateTime;

use crate::core::types::*;
use crate::core::types::{ExchangeResult, ExchangeError};

pub struct CoinbaseParser;

impl CoinbaseParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    fn check_error(json: &Value) -> ExchangeResult<()> {
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            let message = json.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            return Err(ExchangeError::Api {
                code: 0, // Coinbase doesn't use numeric codes
                message: format!("{}: {}", error, message),
            });
        }
        Ok(())
    }

    /// Parse RFC3339 timestamp to milliseconds
    fn parse_rfc3339_to_millis(timestamp: &str) -> Option<i64> {
        DateTime::parse_from_rfc3339(timestamp)
            .ok()
            .map(|dt| dt.timestamp_millis())
    }

    /// Parse Unix seconds string to milliseconds
    fn parse_unix_seconds_to_millis(seconds_str: &str) -> Option<i64> {
        seconds_str.parse::<i64>().ok().map(|s| s * 1000)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse best bid/ask from REST response
    ///
    /// Endpoint: GET /best_bid_ask
    /// Response: { pricebooks: [{ product_id, bids: [{price, size}], asks: [{price, size}], time }] }
    pub fn parse_ticker(json: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(json)?;

        // Get first pricebook entry
        let pricebook = json.get("pricebooks")
            .and_then(|pb| pb.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("Missing pricebooks array".into()))?;

        let symbol = pricebook.get("product_id")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing product_id".into()))?;

        // Get best bid
        let bid_price = pricebook.get("bids")
            .and_then(|b| b.as_array())
            .and_then(|arr| arr.first())
            .and_then(|bid| bid.get("price"))
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        // Get best ask
        let ask_price = pricebook.get("asks")
            .and_then(|a| a.as_array())
            .and_then(|arr| arr.first())
            .and_then(|ask| ask.get("price"))
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        // Last price is mid-price of bid and ask
        let last_price = match (bid_price, ask_price) {
            (Some(bid), Some(ask)) => (bid + ask) / 2.0,
            (Some(bid), None) => bid,
            (None, Some(ask)) => ask,
            (None, None) => return Err(ExchangeError::Parse("No bid or ask prices".into())),
        };

        let timestamp = pricebook.get("time")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

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
            timestamp,
        })
    }

    /// Parse orderbook from REST response
    ///
    /// Endpoint: GET /product_book
    /// Response: { pricebook: { product_id, bids: [{price, size}], asks: [{price, size}], time } }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(json)?;

        let pricebook = json.get("pricebook")
            .ok_or_else(|| ExchangeError::Parse("Missing pricebook object".into()))?;

        // Parse bids
        let bids = pricebook.get("bids")
            .and_then(|b| b.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing bids array".into()))?
            .iter()
            .filter_map(|entry| {
                let price = entry.get("price")?.as_str()?.parse::<f64>().ok()?;
                let size = entry.get("size")?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        // Parse asks
        let asks = pricebook.get("asks")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing asks array".into()))?
            .iter()
            .filter_map(|entry| {
                let price = entry.get("price")?.as_str()?.parse::<f64>().ok()?;
                let size = entry.get("size")?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let timestamp = pricebook.get("time")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence: None, // Coinbase REST API doesn't provide sequence numbers
        })
    }

    /// Parse klines from REST response
    ///
    /// Endpoint: GET /products/{product_id}/candles
    /// Response: { candles: [{ start, low, high, open, close, volume }] }
    ///
    /// Note: Candles are sorted descending (newest first) - we reverse for ascending order
    pub fn parse_klines(json: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(json)?;

        let candles = json.get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candles array".into()))?;

        let mut klines: Vec<Kline> = candles.iter()
            .filter_map(|candle| {
                let start_str = candle.get("start")?.as_str()?;
                let timestamp = Self::parse_unix_seconds_to_millis(start_str)?;

                let open = candle.get("open")?.as_str()?.parse::<f64>().ok()?;
                let high = candle.get("high")?.as_str()?.parse::<f64>().ok()?;
                let low = candle.get("low")?.as_str()?.parse::<f64>().ok()?;
                let close = candle.get("close")?.as_str()?.parse::<f64>().ok()?;
                let volume = candle.get("volume")?.as_str()?.parse::<f64>().ok()?;

                Some(Kline {
                    open_time: timestamp,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume: None,
                    close_time: Some(timestamp),
                    trades: None,
                })
            })
            .collect();

        // Reverse to ascending order (oldest first)
        klines.reverse();

        Ok(klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRADING PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse order from REST response
    ///
    /// Endpoint: POST /orders (create) or GET /orders/historical/{order_id}
    /// Response: { order: { order_id, product_id, side, status, ... } }
    pub fn parse_order(json: &Value) -> ExchangeResult<Order> {
        Self::check_error(json)?;

        // For create order response, check success field
        if let Some(success) = json.get("success").and_then(|s| s.as_bool()) {
            if !success {
                // Parse failure response
                if let Some(failure) = json.get("failure_response") {
                    let error = failure.get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("UNKNOWN_ERROR");
                    let message = failure.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Order failed");
                    return Err(ExchangeError::Api {
                        code: 0,
                        message: format!("{}: {}", error, message),
                    });
                }
            }

            // Parse success response
            let order_data = json.get("success_response")
                .ok_or_else(|| ExchangeError::Parse("Missing success_response".into()))?;

            let order_id = order_data.get("order_id")
                .and_then(|o| o.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing order_id".into()))?;

            let product_id = order_data.get("product_id")
                .and_then(|p| p.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing product_id".into()))?;

            let side_str = order_data.get("side")
                .and_then(|s| s.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing side".into()))?;

            let side = match side_str {
                "BUY" => OrderSide::Buy,
                "SELL" => OrderSide::Sell,
                _ => return Err(ExchangeError::Parse(format!("Unknown side: {}", side_str))),
            };

            return Ok(Order {
                id: order_id.to_string(),
                client_order_id: order_data.get("client_order_id")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string()),
                symbol: product_id.to_string(),
                side,
                order_type: OrderType::Market,
                status: OrderStatus::New,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                time_in_force: TimeInForce::Gtc,
                created_at: 0,
                updated_at: None,
            });
        }

        // Parse full order details (GET endpoint)
        let order_data = json.get("order")
            .ok_or_else(|| ExchangeError::Parse("Missing order object".into()))?;

        let order_id = order_data.get("order_id")
            .and_then(|o| o.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing order_id".into()))?;

        let product_id = order_data.get("product_id")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing product_id".into()))?;

        let side_str = order_data.get("side")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing side".into()))?;

        let side = match side_str {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse(format!("Unknown side: {}", side_str))),
        };

        let order_type_str = order_data.get("order_type")
            .and_then(|t| t.as_str())
            .unwrap_or("UNKNOWN");

        let order_type = match order_type_str {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            "STOP" => OrderType::StopMarket { stop_price: 0.0 },
            "STOP_LIMIT" => OrderType::StopLimit { stop_price: 0.0, limit_price: 0.0 },
            _ => OrderType::Market,
        };

        let price = order_data.get("order_configuration")
            .and_then(|cfg| {
                cfg.get("limit_limit_gtc")
                    .or_else(|| cfg.get("limit_limit_gtd"))
                    .or_else(|| cfg.get("limit_limit_fok"))
            })
            .and_then(|limit| limit.get("limit_price"))
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let filled_size = order_data.get("filled_size")
            .and_then(|f| f.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let average_price = order_data.get("average_filled_price")
            .and_then(|a| a.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let status_str = order_data.get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("UNKNOWN");

        // Map Coinbase status to OrderStatus enum
        let status = match status_str {
            "OPEN" => OrderStatus::Open,
            "FILLED" => OrderStatus::Filled,
            "CANCELLED" | "CANCELED" => OrderStatus::Canceled,
            "EXPIRED" => OrderStatus::Expired,
            "FAILED" | "REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::New,
        };

        let created_at = order_data.get("created_time")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: order_data.get("client_order_id")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string()),
            symbol: product_id.to_string(),
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity: filled_size,
            filled_quantity: filled_size,
            average_price,
            commission: None,
            commission_asset: None,
            time_in_force: TimeInForce::Gtc,
            created_at,
            updated_at: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse balance from REST response
    ///
    /// Endpoint: GET /accounts
    /// Response: { accounts: [{ uuid, currency, available_balance: {value, currency}, hold: {value} }] }
    pub fn parse_balance(json: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(json)?;

        let accounts = json.get("accounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing accounts array".into()))?;

        let balances = accounts.iter()
            .filter_map(|account| {
                let currency = account.get("currency")?.as_str()?;

                let available = account.get("available_balance")
                    .and_then(|ab| ab.get("value"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let frozen = account.get("hold")
                    .and_then(|h| h.get("value"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                Some(Balance {
                    asset: currency.to_string(),
                    free: available,
                    locked: frozen,
                    total: available + frozen,
                })
            })
            .collect();

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker update
    ///
    /// Channel: ticker
    /// Format: { channel, timestamp, sequence_num, events: [{ type, tickers: [{ product_id, price, volume_24_h, ... }] }] }
    ///
    /// Note: Ticker data is nested inside events[].tickers[], not directly on the event object.
    pub fn parse_ws_ticker(json: &Value) -> ExchangeResult<Ticker> {
        let events = json.get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing events array".into()))?;

        let event = events.first()
            .ok_or_else(|| ExchangeError::Parse("Empty events array".into()))?;

        // Coinbase nests ticker data inside events[].tickers[]
        let ticker_data = event.get("tickers")
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("Missing tickers array in event".into()))?;

        let symbol = ticker_data.get("product_id")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing product_id".into()))?;

        let last_price = ticker_data.get("price")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid price".into()))?;

        let volume_24h = ticker_data.get("volume_24_h")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let high_24h = ticker_data.get("high_24_h")
            .and_then(|h| h.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let low_24h = ticker_data.get("low_24_h")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let price_change_percent = ticker_data.get("price_percent_chg_24_h")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok());

        let timestamp = json.get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: price_change_percent,
            timestamp,
        })
    }

    /// Parse WebSocket orderbook update
    ///
    /// Channel: level2
    /// Format: { channel, events: [{ type: "snapshot"|"update", product_id, updates: [{ side, price_level, new_quantity }] }] }
    pub fn parse_ws_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        let events = json.get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing events array".into()))?;

        let event = events.first()
            .ok_or_else(|| ExchangeError::Parse("Empty events array".into()))?;

        let updates = event.get("updates")
            .and_then(|u| u.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing updates array".into()))?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for update in updates {
            let side = update.get("side")
                .and_then(|s| s.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing side".into()))?;

            let price = update.get("price_level")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| ExchangeError::Parse("Invalid price_level".into()))?;

            let size = update.get("new_quantity")
                .and_then(|q| q.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| ExchangeError::Parse("Invalid new_quantity".into()))?;

            // Only add non-zero quantities
            if size > 0.0 {
                match side {
                    "bid" => bids.push((price, size)),
                    "ask" => asks.push((price, size)),
                    _ => {},
                }
            }
        }

        let timestamp = json.get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

        let sequence = json.get("sequence_num")
            .and_then(|s| s.as_i64())
            .map(|n| n.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
        })
    }

    /// Parse WebSocket trades update
    ///
    /// Channel: market_trades
    /// Format: { channel, timestamp, sequence_num, events: [{ type: "snapshot"|"update", trades: [{ product_id, price, size, side, time, trade_id }] }] }
    ///
    /// Note: Trade data is nested inside events[].trades[], not directly on the event object.
    pub fn parse_ws_trades(json: &Value) -> ExchangeResult<PublicTrade> {
        let events = json.get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing events array".into()))?;

        let event = events.first()
            .ok_or_else(|| ExchangeError::Parse("Empty events array".into()))?;

        // Coinbase nests trade data inside events[].trades[]
        let trade_data = event.get("trades")
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("Missing trades array in event".into()))?;

        let symbol = trade_data.get("product_id")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing product_id".into()))?;

        let price = trade_data.get("price")
            .and_then(|p| p.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid price".into()))?;

        let quantity = trade_data.get("size")
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid size".into()))?;

        let side_str = trade_data.get("side")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing side".into()))?;

        let side = match side_str.to_uppercase().as_str() {
            "BUY" => TradeSide::Buy,
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy, // Default to buy if unknown
        };

        let timestamp = trade_data.get("time")
            .and_then(|t| t.as_str())
            .and_then(Self::parse_rfc3339_to_millis)
            .unwrap_or(0);

        let trade_id = trade_data.get("trade_id")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "0".to_string());

        Ok(PublicTrade {
            id: trade_id,
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
            timestamp,
        })
    }

    /// Parse WebSocket candles update
    ///
    /// Channel: candles
    /// Format: { channel, timestamp, sequence_num, events: [{ type: "candle", product_id, candles: [{ start, high, low, open, close, volume }] }] }
    pub fn parse_ws_candles(json: &Value) -> ExchangeResult<Kline> {
        let events = json.get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing events array".into()))?;

        let event = events.first()
            .ok_or_else(|| ExchangeError::Parse("Empty events array".into()))?;

        let candles = event.get("candles")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing candles array".into()))?;

        let candle = candles.first()
            .ok_or_else(|| ExchangeError::Parse("Empty candles array".into()))?;

        let start_str = candle.get("start")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing start".into()))?;

        let timestamp = Self::parse_unix_seconds_to_millis(start_str)
            .ok_or_else(|| ExchangeError::Parse("Invalid start timestamp".into()))?;

        let open = candle.get("open")
            .and_then(|o| o.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid open".into()))?;

        let high = candle.get("high")
            .and_then(|h| h.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid high".into()))?;

        let low = candle.get("low")
            .and_then(|l| l.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid low".into()))?;

        let close = candle.get("close")
            .and_then(|c| c.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid close".into()))?;

        let volume = candle.get("volume")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid volume".into()))?;

        Ok(Kline {
            open_time: timestamp,
            open,
            high,
            low,
            close,
            volume,
            quote_volume: None,
            close_time: Some(timestamp),
            trades: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Coinbase products response.
    ///
    /// Response format:
    /// ```json
    /// {"products":[{"product_id":"BTC-USD","base_currency_id":"BTC","quote_currency_id":"USD","status":"online","base_increment":"0.00000001","quote_increment":"0.01","base_min_size":"0.00000001","base_max_size":"1000","quote_min_size":"1",...},...]}
    /// ```
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let products = response.get("products")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'products' array".to_string()))?;

        let mut symbols = Vec::with_capacity(products.len());

        for product in products {
            let status = product.get("status").and_then(|s| s.as_str()).unwrap_or("");
            // Only include online/trading products
            if status != "online" && !status.is_empty() {
                continue;
            }

            let symbol = match product.get("product_id").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let base_asset = product.get("base_currency_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = product.get("quote_currency_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if base_asset.is_empty() || quote_asset.is_empty() {
                continue;
            }

            // Derive precision from increment strings (count decimal places)
            let price_precision = product.get("quote_increment")
                .and_then(|v| v.as_str())
                .map(Self::count_decimal_places)
                .unwrap_or(8) as u8;

            let quantity_precision = product.get("base_increment")
                .and_then(|v| v.as_str())
                .map(Self::count_decimal_places)
                .unwrap_or(8) as u8;

            let min_quantity = product.get("base_min_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let max_quantity = product.get("base_max_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_notional = product.get("quote_min_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // tick_size: price increment from quote_increment (e.g. "0.01")
            let tick_size = product.get("quote_increment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // step_size: quantity increment from base_increment (e.g. "0.00000001")
            let step_size = product.get("base_increment")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            symbols.push(SymbolInfo {
                symbol,
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
    // CUSTODIAL FUNDS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Find account UUID for a given asset from `GET /api/v3/brokerage/accounts`
    ///
    /// Returns the first account matching the asset currency.
    /// Coinbase uses UUIDs per asset to identify accounts for v2 API calls.
    pub fn find_account_id_for_asset(response: &Value, asset: &str) -> ExchangeResult<String> {
        let accounts = response.get("accounts")
            .and_then(|a| a.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing accounts array".to_string()))?;

        let asset_upper = asset.to_uppercase();
        for account in accounts {
            let currency = account.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if currency == asset_upper {
                let uuid = account.get("uuid")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ExchangeError::Parse("Missing account uuid".to_string()))?;
                return Ok(uuid.to_string());
            }
        }

        Err(ExchangeError::Parse(format!(
            "No Coinbase account found for asset '{}'", asset
        )))
    }

    /// Parse deposit address from `POST /v2/accounts/{id}/addresses`
    ///
    /// Response:
    /// ```json
    /// {"data":{"id":"...","address":"1A1zP1...","address_info":{"address":"1A1zP1..."},...}}
    /// ```
    pub fn parse_deposit_address(response: &Value, asset: &str) -> ExchangeResult<DepositAddress> {
        Self::check_error(response)?;

        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field in address response".to_string()))?;

        let address = data.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'address' field".to_string()))?
            .to_string();

        let tag = data.get("destination_tag")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let network = data.get("network")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let created_at = data.get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| Self::parse_rfc3339_to_millis(s));

        Ok(DepositAddress {
            address,
            tag,
            network,
            asset: asset.to_string(),
            created_at,
        })
    }

    /// Parse withdraw response from `POST /v2/accounts/{id}/transactions` (type=send)
    ///
    /// Response:
    /// ```json
    /// {"data":{"id":"...","status":"pending","type":"send",...}}
    /// ```
    pub fn parse_withdraw_response(response: &Value) -> ExchangeResult<WithdrawResponse> {
        Self::check_error(response)?;

        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field in transaction response".to_string()))?;

        let withdraw_id = data.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing transaction 'id'".to_string()))?
            .to_string();

        let status = data.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .to_string();

        let tx_hash = data.get("network")
            .and_then(|n| n.get("hash"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(WithdrawResponse {
            withdraw_id,
            status,
            tx_hash,
        })
    }

    /// Parse deposit history from `GET /v2/accounts/{id}/deposits`
    ///
    /// Response:
    /// ```json
    /// {"data":[{"id":"...","amount":{"amount":"0.5","currency":"BTC"},"status":"completed","created_at":"..."},...]}
    /// ```
    pub fn parse_deposit_history(response: &Value, asset: &str) -> ExchangeResult<Vec<FundsRecord>> {
        Self::check_error(response)?;

        let data = response.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in deposits response".to_string()))?;

        let records = data.iter().map(|item| {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let amount = item.get("amount")
                .and_then(|a| a.get("amount"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let timestamp = item.get("created_at")
                .and_then(|v| v.as_str())
                .and_then(|s| Self::parse_rfc3339_to_millis(s))
                .unwrap_or(0);

            let tx_hash = item.get("transaction")
                .and_then(|t| t.get("hash"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);

            FundsRecord::Deposit {
                id,
                asset: asset.to_string(),
                amount,
                tx_hash,
                network: None,
                status,
                timestamp,
            }
        }).collect();

        Ok(records)
    }

    /// Parse withdrawal history from `GET /v2/accounts/{id}/transactions` (type=send)
    ///
    /// Response:
    /// ```json
    /// {"data":[{"id":"...","type":"send","amount":{"amount":"-0.1","currency":"BTC"},"status":"completed",...},...]}
    /// ```
    pub fn parse_withdrawal_history(response: &Value, asset: &str) -> ExchangeResult<Vec<FundsRecord>> {
        Self::check_error(response)?;

        let data = response.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in transactions response".to_string()))?;

        let records = data.iter()
            .filter(|item| {
                // Only include "send" type (withdrawals)
                item.get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "send")
                    .unwrap_or(false)
            })
            .map(|item| {
                let id = item.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Amount is negative for sends; take absolute value
                let amount = item.get("amount")
                    .and_then(|a| a.get("amount"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(f64::abs)
                    .unwrap_or(0.0);

                let address = item.get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let status = item.get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let timestamp = item.get("created_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Self::parse_rfc3339_to_millis(s))
                    .unwrap_or(0);

                let tx_hash = item.get("network")
                    .and_then(|n| n.get("hash"))
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from);

                FundsRecord::Withdrawal {
                    id,
                    asset: asset.to_string(),
                    amount,
                    fee: None,
                    address,
                    tag: None,
                    tx_hash,
                    network: None,
                    status,
                    timestamp,
                }
            }).collect();

        Ok(records)
    }

    /// Count decimal places in an increment string like "0.00000001"
    fn count_decimal_places(s: &str) -> usize {
        if let Some(dot_pos) = s.find('.') {
            let decimals = &s[dot_pos + 1..];
            // Trim trailing zeros then count
            decimals.trim_end_matches('0').len().max(
                // but also count if it ends in 1 (like 0.01 -> 2)
                decimals.len()
            )
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_rfc3339() {
        let timestamp = "2023-10-26T10:05:30.123456Z";
        let millis = CoinbaseParser::parse_rfc3339_to_millis(timestamp);
        assert!(millis.is_some());
        assert!(millis.unwrap() > 1698000000000);
    }

    #[test]
    fn test_parse_unix_seconds() {
        let seconds = "1698315930";
        let millis = CoinbaseParser::parse_unix_seconds_to_millis(seconds);
        assert_eq!(millis, Some(1698315930000));
    }

    #[test]
    fn test_check_error() {
        let error_json = json!({
            "error": "invalid_signature",
            "message": "Invalid JWT signature"
        });
        let result = CoinbaseParser::check_error(&error_json);
        assert!(result.is_err());

        let success_json = json!({"field": "value"});
        let result = CoinbaseParser::check_error(&success_json);
        assert!(result.is_ok());
    }
}
