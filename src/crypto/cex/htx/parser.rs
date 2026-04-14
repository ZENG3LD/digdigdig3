//! # HTX Parser
//!
//! Parsing HTX API responses to internal types.
//!
//! ## Response Structure
//!
//! HTX uses two response formats:
//!
//! ### V1 Format (Most endpoints)
//! ```json
//! {
//!   "status": "ok|error",
//!   "ch": "channel",
//!   "ts": 1234567890,
//!   "data": { ... } or "tick": { ... }
//! }
//! ```
//!
//! ### V2 Format (Newer endpoints)
//! ```json
//! {
//!   "code": 200,
//!   "message": "success",
//!   "data": { ... }
//! }
//! ```
//!
//! ## Error Responses
//!
//! V1: `{status: "error", "err-code": "...", "err-msg": "..."}`
//! V2: `{code: non-200, message: "..."}`

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeResult, ExchangeError, UserTrade};

pub struct HtxParser;

impl HtxParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RESPONSE WRAPPER
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Extract result from HTX V1 response
    ///
    /// Checks status == "ok" for success
    pub fn extract_result_v1(json: &Value) -> ExchangeResult<&Value> {
        let status = json["status"].as_str().unwrap_or("error");

        if status != "ok" {
            let err_code = json["err-code"].as_str().unwrap_or("unknown");
            let err_msg = json["err-msg"].as_str().unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: 0, // HTX V1 uses string codes
                message: format!("{}: {}", err_code, err_msg),
            });
        }

        // Try "data" first, then "tick" (market data uses "tick")
        if let Some(data) = json.get("data") {
            Ok(data)
        } else if let Some(tick) = json.get("tick") {
            Ok(tick)
        } else {
            Err(ExchangeError::Parse("Missing data/tick field".into()))
        }
    }

    /// Extract result from HTX V2 response
    ///
    /// Checks code == 200 for success
    pub fn extract_result_v2(json: &Value) -> ExchangeResult<&Value> {
        let code = json["code"].as_i64().unwrap_or(-1);

        if code != 200 {
            let message = json["message"].as_str().unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: message.to_string(),
            });
        }

        json.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing data field".into()))
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse ticker from REST response
    ///
    /// Endpoint: GET /market/detail/merged
    /// Response: tick = { close, open, high, low, amount, vol, count, bid, ask }
    pub fn parse_ticker(json: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let tick = Self::extract_result_v1(json)?;

        let last_price = tick["close"].as_f64()
            .ok_or_else(|| ExchangeError::Parse("Invalid close price".into()))?;

        let bid_price = tick["bid"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64());

        let ask_price = tick["ask"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64());

        let high_24h = tick["high"].as_f64();
        let low_24h = tick["low"].as_f64();
        let volume_24h = tick["amount"].as_f64(); // Base currency volume
        let quote_volume_24h = tick["vol"].as_f64(); // Quote currency volume

        let timestamp = json["ts"].as_i64().unwrap_or(0);

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    /// Parse orderbook from REST response
    ///
    /// Endpoint: GET /market/depth
    /// Response: tick = { version, ts, bids: [[price, size]], asks: [[price, size]] }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        let tick = Self::extract_result_v1(json)?;

        let bids = tick["bids"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                let size = arr.get(1)?.as_f64()?;
                Some(OrderBookLevel::new(price, size))
            })
            .collect();

        let asks = tick["asks"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_f64()?;
                let size = arr.get(1)?.as_f64()?;
                Some(OrderBookLevel::new(price, size))
            })
            .collect();

        let timestamp = tick["ts"].as_i64().unwrap_or(0);
        let sequence = tick["version"].as_i64().map(|v| v.to_string());

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Parse klines from REST response
    ///
    /// Endpoint: GET /market/history/kline
    /// Response: data = [{ id, open, close, low, high, amount, vol, count }, ...]
    ///
    /// Note: id is timestamp in seconds (not milliseconds!)
    pub fn parse_klines(json: &Value) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_result_v1(json)?;
        let list = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?;

        let mut klines: Vec<Kline> = list.iter()
            .filter_map(|entry| {
                // HTX format: { id, open, close, low, high, amount, vol, count }
                // id is timestamp in SECONDS
                let open_time = entry["id"].as_i64()? * 1000; // Convert to milliseconds
                let open = entry["open"].as_f64()?;
                let high = entry["high"].as_f64()?;
                let low = entry["low"].as_f64()?;
                let close = entry["close"].as_f64()?;
                let volume = entry["amount"].as_f64()?; // Base currency volume
                let quote_volume = entry["vol"].as_f64(); // Quote currency volume

                Some(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume,
                    close_time: None,
                    trades: entry["count"].as_i64().map(|c| c as u64),
                })
            })
            .collect();

        // HTX returns klines in descending order (newest first); reverse to ascending.
        klines.reverse();

        Ok(klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse account list from REST response
    ///
    /// Endpoint: GET /v1/account/accounts
    /// Response: data = [{ id, type, subtype, state }, ...]
    ///
    /// Used to get account-id for subsequent API calls
    pub fn parse_account_list(json: &Value) -> ExchangeResult<Vec<(i64, String)>> {
        let data = Self::extract_result_v1(json)?;
        let list = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?;

        let accounts = list.iter()
            .filter_map(|entry| {
                let id = entry["id"].as_i64()?;
                let account_type = entry["type"].as_str()?.to_string();
                Some((id, account_type))
            })
            .collect();

        Ok(accounts)
    }

    /// Parse balance from REST response
    ///
    /// Endpoint: GET /v1/account/accounts/{account-id}/balance
    /// Response: data = { id, type, state, list: [{ currency, type, balance }, ...] }
    pub fn parse_balance(json: &Value) -> ExchangeResult<Vec<crate::core::Balance>> {
        let data = Self::extract_result_v1(json)?;
        let list = data["list"].as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing list array".into()))?;

        // Group by currency (separate entries for 'trade' and 'frozen' types)
        let mut balances_map: std::collections::HashMap<String, (f64, f64)> = std::collections::HashMap::new();

        for entry in list {
            let currency = entry["currency"].as_str()
                .ok_or_else(|| ExchangeError::Parse("Missing currency".into()))?;
            let balance_type = entry["type"].as_str().unwrap_or("trade");
            let balance = entry["balance"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let entry = balances_map.entry(currency.to_uppercase()).or_insert((0.0, 0.0));

            match balance_type {
                "trade" => entry.0 = balance,  // free
                "frozen" => entry.1 = balance, // locked
                _ => {}
            }
        }

        let balances = balances_map.into_iter()
            .map(|(asset, (free, locked))| {
                crate::core::Balance {
                    asset,
                    free,
                    locked,
                    total: free + locked,
                }
            })
            .collect();

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRADING PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse order from REST response
    ///
    /// Endpoint: POST /v1/order/orders/place OR GET /v1/order/orders/{order-id}
    /// Response: data = order_id (string) OR data = { id, symbol, state, type, ... }
    pub fn parse_order(json: &Value) -> ExchangeResult<Order> {
        let data = Self::extract_result_v1(json)?;

        // Handle order placement response (just returns order ID as string)
        if data.is_string() {
            let id = data.as_str().unwrap_or("").to_string();
            return Ok(Order {
                id,
                client_order_id: None,
                symbol: String::new(),
                side: OrderSide::Buy, // Unknown at this point
                order_type: OrderType::Limit { price: 0.0 },
                status: OrderStatus::New,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                time_in_force: TimeInForce::Gtc,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: None,
            });
        }

        // Handle order query response (full order details)
        let id = data["id"].as_i64()
            .ok_or_else(|| ExchangeError::Parse("Missing id".into()))?
            .to_string();

        let symbol = data["symbol"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".into()))?
            .to_string();

        let order_type_str = data["type"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Invalid type".into()))?;

        // HTX order type format: "buy-limit", "sell-limit", "buy-market", "sell-market"
        let (side, order_type) = Self::parse_order_type(order_type_str)?;

        let status = Self::parse_order_status(data["state"].as_str().unwrap_or(""));

        let price = data["price"].as_str()
            .and_then(|s| s.parse::<f64>().ok());

        let quantity = data["amount"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let filled_quantity = data["field-amount"].as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let average_price = if filled_quantity > 0.0 {
            data["field-cash-amount"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|cash| cash / filled_quantity)
        } else {
            None
        };

        let created_at = data["created-at"].as_i64().unwrap_or(0);
        let updated_at = data["finished-at"].as_i64();

        Ok(Order {
            id,
            client_order_id: data["client-order-id"].as_str().map(String::from),
            symbol,
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity,
            average_price,
            time_in_force: TimeInForce::Gtc,
            commission: data["field-fees"].as_str().and_then(|s| s.parse::<f64>().ok()),
            commission_asset: None,
            created_at,
            updated_at,
        })
    }

    /// Parse order type from HTX format
    ///
    /// HTX format: "buy-limit", "sell-limit", "buy-market", "sell-market", etc.
    fn parse_order_type(type_str: &str) -> ExchangeResult<(OrderSide, OrderType)> {
        let parts: Vec<&str> = type_str.split('-').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::Parse(format!("Invalid order type: {}", type_str)));
        }

        let side = match parts[0] {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse(format!("Invalid side: {}", parts[0]))),
        };

        let order_type = match parts[1] {
            "market" => OrderType::Market,
            "limit" => OrderType::Limit { price: 0.0 },
            "ioc" => OrderType::Limit { price: 0.0 }, // IOC is a limit order variant
            "limit-maker" => OrderType::Limit { price: 0.0 }, // Post-only limit
            _ => OrderType::Limit { price: 0.0 }, // Default to limit
        };

        Ok((side, order_type))
    }

    /// Parse order status from HTX string
    fn parse_order_status(status: &str) -> OrderStatus {
        match status {
            "submitted" | "pre-submitted" => OrderStatus::New,
            "partial-filled" => OrderStatus::PartiallyFilled,
            "filled" | "partial-canceled" => {
                // partial-canceled means partially filled then canceled
                // We'll map it to PartiallyFilled for now
                if status == "filled" {
                    OrderStatus::Filled
                } else {
                    OrderStatus::PartiallyFilled
                }
            },
            "canceled" => OrderStatus::Canceled,
            _ => OrderStatus::New,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse exchange info (symbol list) from HTX response
    ///
    /// Endpoint: GET /v1/common/symbols
    /// Response: { status: "ok", data: [{ symbol, "base-currency", "quote-currency", state,
    ///             "amount-precision", "price-precision", "min-order-amt", "max-order-amt",
    ///             "min-order-value" }] }
    ///
    /// Note: all field names use dashes (e.g. "base-currency"), symbols are lowercase.
    /// Filters to state == "online" symbols only.
    pub fn parse_exchange_info(json: &Value, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let data = Self::extract_result_v1(json)?;
        let list = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?;

        let symbols = list.iter()
            .filter_map(|item| {
                // HTX uses lowercase symbol names and dash-separated field names
                let symbol_raw = item["symbol"].as_str()?.to_string();
                let base_asset = item["base-currency"].as_str()
                    .unwrap_or("")
                    .to_uppercase();
                let quote_asset = item["quote-currency"].as_str()
                    .unwrap_or("")
                    .to_uppercase();
                let state = item["state"].as_str().unwrap_or("");

                // Filter to active symbols only
                if state != "online" {
                    return None;
                }

                let status = "TRADING".to_string();

                // HTX provides integer precision fields
                let price_precision = item["price-precision"].as_i64()
                    .map(|p| p as u8)
                    .unwrap_or(8);

                let quantity_precision = item["amount-precision"].as_i64()
                    .map(|p| p as u8)
                    .unwrap_or(8);

                let min_quantity = item["min-order-amt"].as_f64();
                let max_quantity = item["max-order-amt"].as_f64();

                // min-order-value = min notional (price * qty)
                let min_notional = item["min-order-value"].as_f64();

                // Derive tick_size from price-precision: 10^(-price_precision)
                // e.g. price_precision=2 → tick_size=0.01
                let tick_size = item["tick-size"].as_f64().or_else(|| {
                    let p = item["price-precision"].as_i64().unwrap_or(8);
                    Some(10f64.powi(-(p as i32)))
                });

                // Derive step_size from amount-precision: 10^(-amount_precision)
                let step_size = {
                    let p = item["amount-precision"].as_i64().unwrap_or(8);
                    Some(10f64.powi(-(p as i32)))
                };

                Some(crate::core::types::SymbolInfo {
                    symbol: symbol_raw,
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
                    account_type,
                })
            })
            .collect();

        Ok(symbols)
    }

    /// Parse user trades (fills) from `GET /v1/order/matchresults`
    ///
    /// Response: `{"status":"ok","data":[{"id":123,"order-id":456,"symbol":"btcusdt",
    /// "type":"buy-market","price":"50000","filled-amount":"0.001",
    /// "filled-fees":"0.01","fee-currency":"usdt","role":"maker","created-at":1672531200000}]}`
    ///
    /// - `type`:         side prefix — "buy-…" or "sell-…"
    /// - `role`:         "maker" or "taker"
    /// - `filled-fees`:  fee amount (string)
    /// - `fee-currency`: fee asset
    /// - `created-at`:   timestamp in milliseconds
    pub fn parse_user_trades(json: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let data = Self::extract_result_v1(json)?;
        let list = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?;

        let mut trades = Vec::with_capacity(list.len());

        for item in list {
            let id = item["id"].as_i64()
                .ok_or_else(|| ExchangeError::Parse("Missing 'id' in trade".into()))?
                .to_string();

            let order_id = item["order-id"].as_i64()
                .unwrap_or(0)
                .to_string();

            let symbol = item["symbol"].as_str()
                .unwrap_or("")
                .to_string();

            // "type" is "buy-market", "sell-limit", etc. — take prefix before '-'
            let type_str = item["type"].as_str().unwrap_or("buy-market");
            let side = if type_str.starts_with("buy") {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            };

            let price = item["price"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let quantity = item["filled-amount"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let commission = item["filled-fees"].as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|f| f.abs())
                .unwrap_or(0.0);

            let commission_asset = item["fee-currency"].as_str()
                .unwrap_or("")
                .to_uppercase();

            let is_maker = item["role"].as_str()
                .map(|r| r == "maker")
                .unwrap_or(false);

            let timestamp = item["created-at"].as_i64().unwrap_or(0);

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
                timestamp,
            });
        }

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket message (after GZIP decompression)
    ///
    /// Format: { "ch": "...", "tick": {...}, "ts": ... }
    pub fn parse_ws_message(json: &Value) -> ExchangeResult<(String, &Value)> {
        let channel = json["ch"].as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing ch field".into()))?;

        let data = json.get("tick")
            .ok_or_else(|| ExchangeError::Parse("Missing tick field".into()))?;

        Ok((channel.to_string(), data))
    }

    /// Check if WebSocket message is a ping
    pub fn is_ws_ping(json: &Value) -> Option<i64> {
        json["ping"].as_i64()
    }

    /// Check if WebSocket v2 message is a ping
    pub fn is_ws_v2_ping(json: &Value) -> Option<i64> {
        if json["action"].as_str() == Some("ping") {
            json["data"]["ts"].as_i64()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_ticker() {
        let json = json!({
            "status": "ok",
            "ch": "market.btcusdt.detail.merged",
            "ts": 1629384000000i64,
            "tick": {
                "id": 311869842476i64,
                "amount": 18344.5126,
                "count": 89472,
                "open": 48000.00,
                "close": 49500.00,
                "low": 47500.00,
                "high": 50000.00,
                "vol": 896748251.2574,
                "bid": [49499.00, 1.5],
                "ask": [49500.00, 2.3]
            }
        });

        let ticker = HtxParser::parse_ticker(&json, "BTCUSDT").unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last_price, 49500.0);
        assert_eq!(ticker.bid_price, Some(49499.0));
        assert_eq!(ticker.ask_price, Some(49500.0));
        assert_eq!(ticker.high_24h, Some(50000.0));
        assert_eq!(ticker.low_24h, Some(47500.0));
    }

    #[test]
    fn test_parse_klines() {
        let json = json!({
            "status": "ok",
            "ch": "market.btcusdt.kline.1min",
            "ts": 1629384000000i64,
            "data": [
                {
                    "id": 1629384000i64, // Timestamp in SECONDS
                    "open": 48000.00,
                    "close": 48100.00,
                    "low": 47900.00,
                    "high": 48200.00,
                    "amount": 123.456,
                    "vol": 5940000.00,
                    "count": 456
                }
            ]
        });

        let klines = HtxParser::parse_klines(&json).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open_time, 1629384000000); // Converted to milliseconds
        assert_eq!(klines[0].open, 48000.0);
        assert_eq!(klines[0].high, 48200.0);
        assert_eq!(klines[0].low, 47900.0);
        assert_eq!(klines[0].close, 48100.0);
    }

    #[test]
    fn test_parse_order_type() {
        let (side, order_type) = HtxParser::parse_order_type("buy-limit").unwrap();
        assert_eq!(side, OrderSide::Buy);
        assert_eq!(order_type, OrderType::Limit { price: 0.0 });

        let (side, order_type) = HtxParser::parse_order_type("sell-market").unwrap();
        assert_eq!(side, OrderSide::Sell);
        assert_eq!(order_type, OrderType::Market);
    }

    #[test]
    fn test_is_ws_ping() {
        let json = json!({"ping": 1629384000000i64});
        assert_eq!(HtxParser::is_ws_ping(&json), Some(1629384000000));

        let json = json!({"ch": "market.btcusdt.ticker"});
        assert_eq!(HtxParser::is_ws_ping(&json), None);
    }
}
