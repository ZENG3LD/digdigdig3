//! # MEXC Parser
//!
//! Parsing MEXC Spot API responses to internal types.
//!
//! ## Response Structure
//!
//! Spot API - Direct response (no wrapper):
//! ```json
//! {
//!   "symbol": "BTCUSDT",
//!   "price": "93200.50"
//! }
//! ```
//!
//! Or array responses:
//! ```json
//! [{ "symbol": "BTCUSDT", "price": "93200.50" }]
//! ```
//!
//! Error response:
//! ```json
//! {
//!   "code": 10001,
//!   "msg": "Missing required parameter"
//! }
//! ```
//!
//! ## Key Differences from Bybit
//!
//! - No wrapper: Spot responses are direct data (no `retCode`/`result` wrapper)
//! - Error detection: Check for `code` field to detect errors
//! - Kline order: [time, open, high, low, close, volume, close_time, quote_volume]
//! - All timestamps in milliseconds
//! - All numeric values as strings

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeResult, ExchangeError};

pub struct MexcParser;

impl MexcParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RESPONSE WRAPPER
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    ///
    /// MEXC errors have "code" and "msg" fields
    pub fn check_error(json: &Value) -> ExchangeResult<()> {
        if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
            if code != 0 && code != 200 {
                let msg = json.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                });
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse ticker from REST response
    ///
    /// Endpoint: GET /api/v3/ticker/24hr
    /// Response: { symbol, lastPrice, bidPrice, askPrice, highPrice, lowPrice, volume, ... }
    pub fn parse_ticker(json: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(json)?;

        let symbol = json["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?;

        let last_price = json["lastPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Invalid lastPrice".into()))?;

        let bid_price = json["bidPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let ask_price = json["askPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let high_24h = json["highPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let low_24h = json["lowPrice"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let volume_24h = json["volume"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let quote_volume_24h = json["quoteVolume"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let price_change_24h = json["priceChange"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let price_change_percent_24h = json["priceChangePercent"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let timestamp = json["closeTime"].as_i64()
            .or_else(|| json["openTime"].as_i64())
            .unwrap_or(0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h,
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse orderbook from REST response (Spot)
    ///
    /// Endpoint: GET /api/v3/depth
    /// Response: { lastUpdateId, bids: [[price, qty]], asks: [[price, qty]] }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(json)?;

        let bids = json["bids"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let asks = json["asks"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let timestamp = crate::core::timestamp_millis() as i64;
        let sequence = json["lastUpdateId"].as_i64().map(|u| u.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
        })
    }

    /// Parse orderbook from futures REST response
    ///
    /// Endpoint: GET /api/v1/contract/depth/{symbol}
    /// Response: { "success": true, "code": 0, "data": { "asks": [[price, count, qty]], "bids": [[price, count, qty]], "version": 123, "timestamp": 123 } }
    ///
    /// Futures format: [price, order_count, quantity]
    pub fn parse_orderbook_futures(json: &Value) -> ExchangeResult<OrderBook> {
        let bids = json["bids"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids in futures orderbook".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                // arr[1] is order count, arr[2] is quantity
                let size = arr.get(2)?.as_f64()?;
                Some((price, size))
            })
            .collect();

        let asks = json["asks"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks in futures orderbook".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                // arr[1] is order count, arr[2] is quantity
                let size = arr.get(2)?.as_f64()?;
                Some((price, size))
            })
            .collect();

        let timestamp = json.get("timestamp")
            .and_then(|t| t.as_i64())
            .unwrap_or_else(|| crate::core::timestamp_millis() as i64);

        let sequence = json.get("version")
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
        })
    }

    /// Parse klines from REST response (Spot)
    ///
    /// Endpoint: GET /api/v3/klines
    /// Response: [[time, open, high, low, close, volume, close_time, quote_volume], ...]
    ///
    /// Array order: [open_time, open, high, low, close, volume, close_time, quote_volume]
    pub fn parse_klines(json: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(json)?;

        let list = json.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected klines array".into()))?;

        let klines = list.iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;

                // MEXC order: [open_time, open, high, low, close, volume, close_time, quote_volume]
                let open_time = arr.first()?.as_i64()?;
                let open = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                let high = arr.get(2)?.as_str()?.parse::<f64>().ok()?;
                let low = arr.get(3)?.as_str()?.parse::<f64>().ok()?;
                let close = arr.get(4)?.as_str()?.parse::<f64>().ok()?;
                let volume = arr.get(5)?.as_str()?.parse::<f64>().ok()?;
                let close_time = arr.get(6).and_then(|v| v.as_i64());
                let quote_volume = arr.get(7)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());

                Some(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume,
                    close_time,
                    trades: None,
                })
            })
            .collect();

        Ok(klines)
    }

    /// Parse klines from futures REST response
    ///
    /// Endpoint: GET /api/v1/contract/kline/{symbol}
    /// Response: { "success": true, "code": 0, "data": { "time": [times], "open": [opens], ... } }
    ///
    /// Futures format: separate arrays for each field
    pub fn parse_klines_futures(json: &Value) -> ExchangeResult<Vec<Kline>> {
        // Futures klines have separate arrays for each field
        let time_arr = json.get("time")
            .and_then(|t| t.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing time array".into()))?;

        let open_arr = json.get("open")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing open array".into()))?;

        let high_arr = json.get("high")
            .and_then(|h| h.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing high array".into()))?;

        let low_arr = json.get("low")
            .and_then(|l| l.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing low array".into()))?;

        let close_arr = json.get("close")
            .and_then(|c| c.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing close array".into()))?;

        let vol_arr = json.get("vol")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing vol array".into()))?;

        // Ensure all arrays have the same length
        let len = time_arr.len();
        if open_arr.len() != len || high_arr.len() != len ||
           low_arr.len() != len || close_arr.len() != len || vol_arr.len() != len {
            return Err(ExchangeError::Parse("Inconsistent kline array lengths".into()));
        }

        let mut klines = Vec::with_capacity(len);
        for i in 0..len {
            let open_time = time_arr[i].as_i64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid time at index {}", i)))?
                * 1000; // Convert to milliseconds

            let open = open_arr[i].as_f64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid open at index {}", i)))?;

            let high = high_arr[i].as_f64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid high at index {}", i)))?;

            let low = low_arr[i].as_f64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid low at index {}", i)))?;

            let close = close_arr[i].as_f64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid close at index {}", i)))?;

            let volume = vol_arr[i].as_f64()
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid volume at index {}", i)))?;

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse futures ticker from REST response
    ///
    /// Endpoint: GET /api/v1/contract/ticker
    /// Response: { "success": true, "code": 0, "data": [{ symbol, lastPrice, bid1, ask1, ... }] }
    pub fn parse_ticker_futures(json: &Value) -> ExchangeResult<Ticker> {
        let symbol = json["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?;

        let last_price = json["lastPrice"].as_f64()
            .ok_or_else(|| ExchangeError::Parse("Invalid lastPrice".into()))?;

        let bid_price = json["bid1"].as_f64();
        let ask_price = json["ask1"].as_f64();
        let high_24h = json["high24Price"].as_f64();
        let low_24h = json["low24Price"].as_f64();
        let volume_24h = json["volume24"].as_f64();
        let price_change_24h = json["riseFallRate"].as_f64();

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: price_change_24h,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from MEXC response
    ///
    /// Endpoint: GET /api/v3/exchangeInfo
    /// Response: { symbols: [{ symbol, baseAsset, quoteAsset, status, baseAssetPrecision,
    ///             quoteAssetPrecision, baseSizePrecision, quoteAmountPrecision, ... }] }
    ///
    /// MEXC uses "1" for active status (not "TRADING" like Binance).
    /// Precision values come directly on the symbol object, not in a filters array.
    pub fn parse_exchange_info(json: &Value) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        Self::check_error(json)?;

        let symbols_arr = json["symbols"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing symbols array".into()))?;

        let symbols = symbols_arr.iter()
            .filter_map(|item| {
                let symbol = item["symbol"].as_str()?.to_string();
                let base_asset = item["baseAsset"].as_str().unwrap_or("").to_string();
                let quote_asset = item["quoteAsset"].as_str().unwrap_or("").to_string();

                // MEXC uses "1" (string) for active status, not "TRADING".
                // Accept both formats for forward-compatibility.
                let status_raw = item["status"].as_str()
                    .or_else(|| item["status"].as_i64().map(|_| ""))
                    .unwrap_or("");

                // status "1" = active trading on MEXC
                let is_active = status_raw == "1"
                    || status_raw == "TRADING"
                    || status_raw == "ENABLED"
                    || item["status"].as_i64() == Some(1);

                if !is_active {
                    return None;
                }

                // Normalize status to a human-readable string
                let status = if status_raw == "1" || item["status"].as_i64() == Some(1) {
                    "TRADING".to_string()
                } else {
                    status_raw.to_string()
                };

                // MEXC provides precision directly on the symbol object.
                // baseAssetPrecision / quoteAssetPrecision are integer decimal places.
                let price_precision = item["quoteAssetPrecision"].as_u64()
                    .or_else(|| item["quotePrecision"].as_u64())
                    .unwrap_or(8) as u8;

                let quantity_precision = item["baseAssetPrecision"].as_u64()
                    .unwrap_or(8) as u8;

                // baseSizePrecision is a string like "0.01" giving the minimum quantity step.
                let step_size = item["baseSizePrecision"].as_str()
                    .and_then(|s| s.parse::<f64>().ok());

                // quoteAmountPrecision is the minimum notional in quote currency (string).
                let min_notional = item["quoteAmountPrecision"].as_str()
                    .and_then(|s| s.parse::<f64>().ok());

                // min_quantity equals the step size for MEXC (no separate minQty field).
                let min_quantity = step_size;
                // MEXC has maxQuoteAmount (in quote currency) but no direct maxQty field.
                let max_quantity: Option<f64> = None;

                // MEXC Spot exchangeInfo has no explicit tickSize field.
                // Derive tick_size from quoteAssetPrecision: 10^(-precision).
                // Also check pricePrecision (integer) which some MEXC futures endpoints use.
                let tick_size = item["quoteAssetPrecision"].as_u64()
                    .or_else(|| item["quotePrecision"].as_u64())
                    .or_else(|| item["pricePrecision"].as_u64())
                    .map(|p| 10f64.powi(-(p as i32)));

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
                    step_size,
                    min_notional,
                })
            })
            .collect();

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse balance from REST response
    ///
    /// Endpoint: GET /api/v3/account
    /// Response: { balances: [{ asset, free, locked }] }
    pub fn parse_balance(json: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_error(json)?;

        let balances_arr = json["balances"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing balances array".into()))?;

        let balances = balances_arr.iter()
            .filter_map(|balance_data| {
                let asset = balance_data["asset"].as_str()?.to_string();
                let free = balance_data["free"].as_str()?.parse::<f64>().ok()?;
                let locked = balance_data["locked"].as_str()?.parse::<f64>().ok().unwrap_or(0.0);
                let total = free + locked;

                // Skip zero balances
                if total == 0.0 {
                    return None;
                }

                Some(Balance {
                    asset,
                    free,
                    locked,
                    total,
                })
            })
            .collect();

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRADING PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse order from REST response
    ///
    /// Endpoint: POST /api/v3/order OR GET /api/v3/order
    /// Response: { orderId, symbol, side, type, price, origQty, executedQty, status, ... }
    pub fn parse_order(json: &Value) -> ExchangeResult<Order> {
        Self::check_error(json)?;

        let id = json["orderId"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
            .to_string();

        let symbol = json["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?
            .to_string();

        let side = match json["side"].as_str() {
            Some("BUY") => OrderSide::Buy,
            Some("SELL") => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse("Invalid side".into())),
        };

        let order_type = match json["type"].as_str() {
            Some("MARKET") => OrderType::Market,
            Some("LIMIT") | Some("LIMIT_MAKER") => OrderType::Limit { price: 0.0 },
            _ => OrderType::Limit { price: 0.0 }, // default
        };

        let status = Self::parse_order_status(json["status"].as_str().unwrap_or(""));

        let price = json["price"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let quantity = json["origQty"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let filled_quantity = json["executedQty"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let average_price = if filled_quantity > 0.0 {
            json["cummulativeQuoteQty"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|quote_qty| quote_qty / filled_quantity)
        } else {
            None
        };

        let created_at = json["time"].as_i64()
            .or_else(|| json["transactTime"].as_i64())
            .unwrap_or(0);

        let updated_at = json["updateTime"].as_i64();

        Ok(Order {
            id,
            client_order_id: json["clientOrderId"].as_str().map(String::from),
            symbol,
            side,
            order_type,
            status,
            price,
            stop_price: json["stopPrice"].as_str().and_then(|s| s.parse::<f64>().ok()),
            quantity,
            filled_quantity,
            average_price,
            time_in_force: Self::parse_time_in_force(json["timeInForce"].as_str().unwrap_or("GTC")),
            commission: None,
            commission_asset: None,
            created_at,
            updated_at,
        })
    }

    /// Parse order status from string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::New,
        }
    }

    /// Parse time in force from string
    fn parse_time_in_force(tif: &str) -> TimeInForce {
        match tif {
            "GTC" => TimeInForce::Gtc,
            "IOC" => TimeInForce::Ioc,
            "FOK" => TimeInForce::Fok,
            _ => TimeInForce::Gtc,
        }
    }

    /// Parse multiple orders from array response
    ///
    /// Used for GET /api/v3/openOrders, GET /api/v3/allOrders
    pub fn parse_orders(json: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(json)?;

        let list = json.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected orders array".into()))?;

        let orders: Vec<Order> = list.iter()
            .filter_map(|order_json| Self::parse_order(order_json).ok())
            .collect();

        Ok(orders)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket message
    ///
    /// MEXC WebSocket format:
    /// { "channel": "spot@public...", "symbol": "BTCUSDT", "sendtime": 123, "publicdeals": {...} }
    pub fn parse_ws_message(json: &Value) -> ExchangeResult<(String, &Value)> {
        // Check for ping/pong
        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
            return Ok((method.to_string(), json));
        }

        // Check for error/subscription response (has 'msg' but no 'channel')
        if json.get("channel").is_none() {
            if let Some(msg) = json.get("msg").and_then(|m| m.as_str()) {
                return Err(ExchangeError::Parse(format!("Subscription error: {}", msg)));
            }
            return Err(ExchangeError::Parse("Missing channel field".into()));
        }

        // Regular data message
        let channel = json.get("channel")
            .and_then(|c| c.as_str())
            .expect("Safe: already checked above");

        // Data is in stream-specific fields: publicdeals, publicspotkline, etc.
        // For now, just pass the whole message
        Ok((channel.to_string(), json))
    }

    /// Parse WebSocket ticker update (from book ticker stream)
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        let symbol = data.get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?;

        let timestamp = data.get("sendtime")
            .and_then(|t| t.as_i64())
            .unwrap_or(0);

        // Check for book ticker data
        if let Some(book_ticker) = data.get("publicbookticker") {
            let bid_price = book_ticker.get("bidprice")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let ask_price = book_ticker.get("askprice")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // Use mid-price as last price
            let last_price = match (bid_price, ask_price) {
                (Some(bid), Some(ask)) => (bid + ask) / 2.0,
                (Some(bid), None) => bid,
                (None, Some(ask)) => ask,
                _ => return Err(ExchangeError::Parse("Missing bid/ask prices".into())),
            };

            return Ok(Ticker {
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
            });
        }

        // Check for trade data
        if let Some(deals) = data.get("publicdeals") {
            if let Some(deals_list) = deals.get("dealsList").and_then(|d| d.as_array()) {
                if let Some(last_deal) = deals_list.first() {
                    let last_price = last_deal.get("price")
                        .and_then(|p| p.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .ok_or_else(|| ExchangeError::Parse("Invalid price".into()))?;

                    return Ok(Ticker {
                        symbol: symbol.to_string(),
                        last_price,
                        bid_price: None,
                        ask_price: None,
                        high_24h: None,
                        low_24h: None,
                        volume_24h: None,
                        quote_volume_24h: None,
                        price_change_24h: None,
                        price_change_percent_24h: None,
                        timestamp,
                    });
                }
            }
        }

        Err(ExchangeError::Parse("Unknown ticker format".into()))
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PROTOBUF DECODERS (WebSocket binary messages)
    // ═══════════════════════════════════════════════════════════════════════════════

    // MEXC WebSocket uses protobuf encoding since Aug 2025.
    // All messages are wrapped in PushDataV3ApiWrapper:
    //   field 1 (string): channel
    //   field 3 (string): symbol
    //   field 6 (varint): sendTime (millis)
    //   field 309 (bytes): PublicMiniTickerV3Api
    //   field 314 (bytes): PublicAggreDealsV3Api
    //   field 306 (bytes): PublicBookTickerV3Api
    //
    // Instead of pulling in a protobuf crate, we decode manually since
    // the schema is small and stable.

    /// Decode a varint from protobuf wire format.
    /// Returns (value, new_position).
    fn decode_varint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            if pos >= data.len() { return None; }
            let b = data[pos];
            pos += 1;
            result |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 { break; }
            shift += 7;
            if shift >= 64 { return None; }
        }
        Some((result, pos))
    }

    /// Extract a string field from protobuf bytes by field number.
    fn pb_string(data: &[u8], target_field: u32) -> Option<String> {
        let mut pos = 0;
        while pos < data.len() {
            let (tag, new_pos) = Self::decode_varint(data, pos)?;
            pos = new_pos;
            let field_num = (tag >> 3) as u32;
            let wire_type = (tag & 0x07) as u8;

            match wire_type {
                0 => { // varint - skip
                    let (_, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                }
                2 => { // length-delimited
                    let (len, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                    let end = pos + len as usize;
                    if end > data.len() { return None; }
                    if field_num == target_field {
                        return String::from_utf8(data[pos..end].to_vec()).ok();
                    }
                    pos = end;
                }
                1 => { pos += 8; } // 64-bit fixed
                5 => { pos += 4; } // 32-bit fixed
                _ => return None,
            }
        }
        None
    }

    /// Extract a varint field from protobuf bytes by field number.
    fn pb_varint(data: &[u8], target_field: u32) -> Option<u64> {
        let mut pos = 0;
        while pos < data.len() {
            let (tag, new_pos) = Self::decode_varint(data, pos)?;
            pos = new_pos;
            let field_num = (tag >> 3) as u32;
            let wire_type = (tag & 0x07) as u8;

            match wire_type {
                0 => {
                    let (val, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                    if field_num == target_field {
                        return Some(val);
                    }
                }
                2 => {
                    let (len, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                    pos += len as usize;
                    if pos > data.len() { return None; }
                }
                1 => { pos += 8; }
                5 => { pos += 4; }
                _ => return None,
            }
        }
        None
    }

    /// Extract a bytes (sub-message) field from protobuf by field number.
    fn pb_bytes(data: &[u8], target_field: u32) -> Option<&[u8]> {
        let mut pos = 0;
        while pos < data.len() {
            let (tag, new_pos) = Self::decode_varint(data, pos)?;
            pos = new_pos;
            let field_num = (tag >> 3) as u32;
            let wire_type = (tag & 0x07) as u8;

            match wire_type {
                0 => {
                    let (_, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                }
                2 => {
                    let (len, new_pos) = Self::decode_varint(data, pos)?;
                    pos = new_pos;
                    let end = pos + len as usize;
                    if end > data.len() { return None; }
                    if field_num == target_field {
                        return Some(&data[pos..end]);
                    }
                    pos = end;
                }
                1 => { pos += 8; }
                5 => { pos += 4; }
                _ => return None,
            }
        }
        None
    }

    /// Extract ALL repeated bytes fields with a given field number.
    fn pb_repeated_bytes(data: &[u8], target_field: u32) -> Vec<&[u8]> {
        let mut results = Vec::new();
        let mut pos = 0;
        while pos < data.len() {
            let (tag, new_pos) = match Self::decode_varint(data, pos) {
                Some(v) => v,
                None => break,
            };
            pos = new_pos;
            let field_num = (tag >> 3) as u32;
            let wire_type = (tag & 0x07) as u8;

            match wire_type {
                0 => {
                    if let Some((_, new_pos)) = Self::decode_varint(data, pos) {
                        pos = new_pos;
                    } else { break; }
                }
                2 => {
                    let (len, new_pos) = match Self::decode_varint(data, pos) {
                        Some(v) => v,
                        None => break,
                    };
                    pos = new_pos;
                    let end = pos + len as usize;
                    if end > data.len() { break; }
                    if field_num == target_field {
                        results.push(&data[pos..end]);
                    }
                    pos = end;
                }
                1 => { pos += 8; }
                5 => { pos += 4; }
                _ => break,
            }
        }
        results
    }

    /// Parse a protobuf binary WebSocket message into a StreamEvent.
    ///
    /// The outer wrapper is PushDataV3ApiWrapper with:
    ///   field 1: channel, field 3: symbol, field 6: sendTime
    ///   field 309: MiniTicker, field 314: AggreDeals, field 306: BookTicker
    pub fn parse_protobuf_message(data: &[u8]) -> ExchangeResult<(String, StreamEvent)> {
        // Extract wrapper fields
        let channel = Self::pb_string(data, 1)
            .ok_or_else(|| ExchangeError::Parse("Missing channel in protobuf wrapper".into()))?;
        let symbol = Self::pb_string(data, 3)
            .unwrap_or_default();
        let timestamp = Self::pb_varint(data, 6)
            .unwrap_or(0) as i64;

        // Dispatch based on channel content
        if channel.contains("miniTicker") {
            // field 309: PublicMiniTickerV3Api
            let body = Self::pb_bytes(data, 309)
                .ok_or_else(|| ExchangeError::Parse("Missing miniTicker body (field 309)".into()))?;
            let ticker = Self::parse_pb_mini_ticker(body, &symbol, timestamp)?;
            Ok((channel, StreamEvent::Ticker(ticker)))
        } else if channel.contains("aggre.deals") || channel.contains("public.deals") {
            // field 314: PublicAggreDealsV3Api (aggre deals)
            // field 305: PublicDealsV3Api (non-aggre deals)
            let body = Self::pb_bytes(data, 314)
                .or_else(|| Self::pb_bytes(data, 305))
                .ok_or_else(|| ExchangeError::Parse("Missing deals body".into()))?;
            let ticker = Self::parse_pb_aggre_deals(body, &symbol, timestamp)?;
            Ok((channel, StreamEvent::Ticker(ticker)))
        } else if channel.contains("bookTicker") {
            // field 306: PublicBookTickerV3Api
            let body = Self::pb_bytes(data, 306)
                .ok_or_else(|| ExchangeError::Parse("Missing bookTicker body (field 306)".into()))?;
            let ticker = Self::parse_pb_book_ticker(body, &symbol, timestamp)?;
            Ok((channel, StreamEvent::Ticker(ticker)))
        } else {
            Err(ExchangeError::Parse(format!("Unsupported protobuf channel: {}", channel)))
        }
    }

    /// Parse PublicMiniTickerV3Api protobuf body.
    ///
    /// Fields: 1=symbol, 2=lastPrice, 3=priceChange%, 4=?, 5=high, 6=low,
    ///         7=volume, 8=quoteVolume, 9-12=alternate timezone variants
    fn parse_pb_mini_ticker(body: &[u8], symbol: &str, timestamp: i64) -> ExchangeResult<Ticker> {
        let last_price_str = Self::pb_string(body, 2)
            .ok_or_else(|| ExchangeError::Parse("Missing lastPrice in miniTicker".into()))?;
        let last_price: f64 = last_price_str.parse()
            .map_err(|_| ExchangeError::Parse(format!("Invalid lastPrice: {}", last_price_str)))?;

        let price_change_pct = Self::pb_string(body, 3)
            .and_then(|s| s.parse::<f64>().ok());

        let high_24h = Self::pb_string(body, 5)
            .and_then(|s| s.parse::<f64>().ok());

        let low_24h = Self::pb_string(body, 6)
            .and_then(|s| s.parse::<f64>().ok());

        let volume_24h = Self::pb_string(body, 7)
            .and_then(|s| s.parse::<f64>().ok());

        let quote_volume_24h = Self::pb_string(body, 8)
            .and_then(|s| s.parse::<f64>().ok());

        // Use symbol from body if available, otherwise use wrapper symbol
        let sym = Self::pb_string(body, 1)
            .unwrap_or_else(|| symbol.to_string());

        Ok(Ticker {
            symbol: sym,
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h,
            price_change_24h: None,
            price_change_percent_24h: price_change_pct,
            timestamp,
        })
    }

    /// Parse PublicAggreDealsV3Api protobuf body.
    ///
    /// Fields: 1 (repeated)=deal items, 2=eventType
    /// Each deal item: 1=price, 2=quantity, 3=tradeType(1=buy,2=sell), 4=time(ms)
    fn parse_pb_aggre_deals(body: &[u8], symbol: &str, timestamp: i64) -> ExchangeResult<Ticker> {
        let deals = Self::pb_repeated_bytes(body, 1);

        if deals.is_empty() {
            return Err(ExchangeError::Parse("No deals in aggre deals message".into()));
        }

        // Use the last (most recent) deal as the price
        let last_deal = deals.last().unwrap();
        let price_str = Self::pb_string(last_deal, 1)
            .ok_or_else(|| ExchangeError::Parse("Missing price in deal".into()))?;
        let last_price: f64 = price_str.parse()
            .map_err(|_| ExchangeError::Parse(format!("Invalid deal price: {}", price_str)))?;

        let deal_time = Self::pb_varint(last_deal, 4)
            .unwrap_or(timestamp as u64) as i64;

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: deal_time,
        })
    }

    /// Parse PublicBookTickerV3Api protobuf body.
    ///
    /// Fields: 1=bidPrice, 2=bidQuantity, 3=askPrice, 4=askQuantity
    fn parse_pb_book_ticker(body: &[u8], symbol: &str, timestamp: i64) -> ExchangeResult<Ticker> {
        let bid_price = Self::pb_string(body, 1)
            .and_then(|s| s.parse::<f64>().ok());
        let ask_price = Self::pb_string(body, 3)
            .and_then(|s| s.parse::<f64>().ok());

        let last_price = match (bid_price, ask_price) {
            (Some(b), Some(a)) => (b + a) / 2.0,
            (Some(b), None) => b,
            (None, Some(a)) => a,
            _ => return Err(ExchangeError::Parse("Missing bid/ask in bookTicker".into())),
        };

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let json = json!({
            "symbol": "BTCUSDT",
            "lastPrice": "93200.50",
            "bidPrice": "93200.00",
            "askPrice": "93210.00",
            "highPrice": "93500.00",
            "lowPrice": "91800.00",
            "volume": "12345.67",
            "quoteVolume": "1147893256.23",
            "priceChange": "1200.50",
            "priceChangePercent": "1.3",
            "openTime": 1640080800000_i64,
            "closeTime": 1640167200000_i64
        });

        let ticker = MexcParser::parse_ticker(&json).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last_price, 93200.50);
        assert_eq!(ticker.bid_price, Some(93200.00));
        assert_eq!(ticker.ask_price, Some(93210.00));
    }

    #[test]
    fn test_parse_orderbook() {
        let json = json!({
            "lastUpdateId": 123456789_i64,
            "bids": [
                ["93220.00", "0.5"],
                ["93210.00", "1.2"]
            ],
            "asks": [
                ["93230.00", "0.8"],
                ["93240.00", "2.1"]
            ]
        });

        let orderbook = MexcParser::parse_orderbook(&json).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids[0].0, 93220.00);
        assert_eq!(orderbook.bids[0].1, 0.5);
    }

    #[test]
    fn test_parse_error() {
        let json = json!({
            "code": 10001,
            "msg": "Missing required parameter"
        });

        let result = MexcParser::check_error(&json);
        assert!(result.is_err());

        match result {
            Err(ExchangeError::Api { code, message }) => {
                assert_eq!(code, 10001);
                assert_eq!(message, "Missing required parameter");
            },
            _ => panic!("Expected API error"),
        }
    }
}
