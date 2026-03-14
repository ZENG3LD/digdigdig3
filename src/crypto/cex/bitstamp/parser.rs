//! # Bitstamp Parser
//!
//! Parsing Bitstamp V2 API responses to internal types.
//!
//! ## Response Structure
//!
//! Success response: Direct JSON data (no wrapper)
//! ```json
//! {
//!   "last": "2211.00",
//!   "volume": "213.26801100",
//!   ...
//! }
//! ```
//!
//! Error response:
//! ```json
//! {
//!   "status": "error",
//!   "reason": "Error description",
//!   "code": "API0007"
//! }
//! ```
//!
//! ## Key Differences from Other Exchanges
//!
//! - No wrapper object (data comes directly)
//! - All numbers are strings for precision
//! - Error responses have specific format with "status": "error"
//! - Timestamps can be strings or integers
//! - Order types: "0" = buy, "1" = sell

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeResult, ExchangeError, PublicTrade, TradeSide};

pub struct BitstampParser;

impl BitstampParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    pub fn check_error(json: &Value) -> ExchangeResult<()> {
        if let Some(status) = json.get("status").and_then(|s| s.as_str()) {
            if status == "error" {
                let reason = json.get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Unknown error");
                let code = json.get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("UNKNOWN");

                return Err(ExchangeError::Api {
                    code: 0, // Bitstamp uses string codes
                    message: format!("{}: {}", code, reason),
                });
            }
        }
        Ok(())
    }

    /// Parse string to f64 safely
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| value.as_f64())
    }

    /// Parse string to i64 safely
    fn parse_i64(value: &Value) -> Option<i64> {
        value.as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| value.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse ticker from REST response
    ///
    /// Endpoint: GET /api/v2/ticker/{pair}/
    /// Response: { last, high, low, vwap, volume, bid, ask, timestamp, open, ... }
    pub fn parse_ticker(json: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(json)?;

        let symbol = json.get("pair")
            .or_else(|| json.get("market"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let last_price = json.get("last")
            .and_then(Self::parse_f64)
            .ok_or_else(|| ExchangeError::Parse("Missing last price".into()))?;

        let bid_price = json.get("bid").and_then(Self::parse_f64);
        let ask_price = json.get("ask").and_then(Self::parse_f64);
        let high_24h = json.get("high").and_then(Self::parse_f64);
        let low_24h = json.get("low").and_then(Self::parse_f64);
        let volume_24h = json.get("volume").and_then(Self::parse_f64);
        let vwap = json.get("vwap").and_then(Self::parse_f64);

        let timestamp = json.get("timestamp")
            .and_then(Self::parse_i64)
            .map(|ts| ts * 1000) // Convert seconds to milliseconds
            .unwrap_or(0);

        // Calculate percent change if we have open_24 and last
        let price_change_percent_24h = json.get("percent_change_24")
            .and_then(Self::parse_f64);

        Ok(Ticker {
            symbol,
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: vwap, // Use vwap as quote volume approximation
            price_change_24h: None,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse orderbook from REST response
    ///
    /// Endpoint: GET /api/v2/order_book/{pair}/
    /// Response: { timestamp, microtimestamp, bids: [[price, amount]], asks: [[price, amount]] }
    pub fn parse_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(json)?;

        let bids = json.get("bids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing bids".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let asks = json.get("asks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing asks".into()))?
            .iter()
            .filter_map(|entry| {
                let arr = entry.as_array()?;
                let price = arr.first()?.as_str()?.parse::<f64>().ok()?;
                let size = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                Some((price, size))
            })
            .collect();

        let timestamp = json.get("timestamp")
            .or_else(|| json.get("microtimestamp"))
            .and_then(Self::parse_i64)
            .map(|ts| if ts > 10000000000 { ts / 1000 } else { ts * 1000 }) // Handle both micro and normal
            .unwrap_or(0);

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence: None,
        })
    }

    /// Parse klines from REST response
    ///
    /// Endpoint: GET /api/v2/ohlc/{pair}/
    /// Response: { data: { ohlc: [{ timestamp, open, high, low, close, volume }], pair } }
    pub fn parse_klines(json: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(json)?;

        let ohlc_data = json.get("data")
            .and_then(|d| d.get("ohlc"))
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing data.ohlc".into()))?;

        let klines = ohlc_data.iter()
            .filter_map(|entry| {
                let open_time = entry.get("timestamp")
                    .and_then(Self::parse_i64)?
                    * 1000; // Convert to milliseconds

                let open = entry.get("open").and_then(Self::parse_f64)?;
                let high = entry.get("high").and_then(Self::parse_f64)?;
                let low = entry.get("low").and_then(Self::parse_f64)?;
                let close = entry.get("close").and_then(Self::parse_f64)?;
                let volume = entry.get("volume").and_then(Self::parse_f64)?;

                Some(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume: None,
                    close_time: None,
                    trades: None,
                })
            })
            .collect();

        Ok(klines)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCOUNT PARSERS (REST)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse balance from REST response
    ///
    /// Endpoint: POST /api/v2/account_balances/
    /// Response: [{ currency, total, available, reserved }, ...]
    pub fn parse_balance(json: &Value) -> ExchangeResult<Vec<crate::core::Balance>> {
        Self::check_error(json)?;

        let balances_array = json.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of balances".into()))?;

        let balances = balances_array.iter()
            .filter_map(|balance_data| {
                let asset = balance_data.get("currency")?.as_str()?.to_uppercase();

                let total = balance_data.get("total")
                    .and_then(Self::parse_f64)?;
                let available = balance_data.get("available")
                    .and_then(Self::parse_f64)?;
                let locked = balance_data.get("reserved")
                    .and_then(Self::parse_f64)
                    .unwrap_or(0.0);

                Some(crate::core::Balance {
                    asset,
                    free: available,
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
    /// Endpoint: POST /api/v2/buy/{pair}/ OR /api/v2/order_status/
    /// Response: { id, datetime, type, price, amount, ... }
    pub fn parse_order(json: &Value) -> ExchangeResult<Order> {
        Self::check_error(json)?;

        let id = json.get("id")
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|i| Box::leak(i.to_string().into_boxed_str()) as &str)))
            .ok_or_else(|| ExchangeError::Parse("Missing order id".into()))?
            .to_string();

        // Bitstamp doesn't always return symbol in order response
        let symbol = json.get("currency_pair")
            .or_else(|| json.get("market"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        // Parse side: "0" = buy, "1" = sell
        let side = match json.get("type").and_then(|t| t.as_str()) {
            Some("0") => OrderSide::Buy,
            Some("1") => OrderSide::Sell,
            _ => OrderSide::Buy, // default
        };

        // Bitstamp doesn't distinguish limit/market in status response
        let order_type = OrderType::Limit { price: 0.0 }; // Default to limit

        // Parse status
        let status = match json.get("status").and_then(|s| s.as_str()) {
            Some("Open") | Some("open") => OrderStatus::Open,
            Some("Finished") | Some("finished") => OrderStatus::Filled,
            Some("Canceled") | Some("canceled") => OrderStatus::Canceled,
            _ => OrderStatus::Open,
        };

        let price = json.get("price").and_then(Self::parse_f64);
        let quantity = json.get("amount")
            .and_then(Self::parse_f64)
            .unwrap_or(0.0);

        // For filled orders, check transactions
        let filled_quantity = if let Some(transactions) = json.get("transactions").and_then(|t| t.as_array()) {
            if transactions.is_empty() {
                0.0
            } else {
                // Calculate from amount_remaining if available
                json.get("amount_remaining")
                    .and_then(Self::parse_f64)
                    .map(|remaining| quantity - remaining)
                    .unwrap_or(quantity)
            }
        } else {
            0.0
        };

        // Parse datetime to timestamp (milliseconds)
        // For simplicity, we'll use 0 here - proper parsing would use chrono
        let created_at = json.get("datetime")
            .and_then(|v| v.as_str())
            .map(|_s| 0)
            .unwrap_or(0);

        Ok(Order {
            id,
            client_order_id: json.get("client_order_id").and_then(|v| v.as_str()).map(String::from),
            symbol,
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        })
    }

    /// Parse array of orders from open_orders response
    ///
    /// Endpoint: POST /api/v2/open_orders/all/
    /// Response: [{ id, datetime, type, price, amount, currency_pair, market }, ...]
    pub fn parse_orders(json: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_error(json)?;

        let orders_array = json.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".into()))?;

        let orders = orders_array.iter()
            .filter_map(|order_data| {
                Self::parse_order(order_data).ok()
            })
            .collect();

        Ok(orders)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket trade event
    ///
    /// Channel: live_trades_{pair}
    /// Message: { data: { amount, price, timestamp, type, id, ... }, channel, event }
    pub fn parse_ws_trade(json: &Value) -> ExchangeResult<PublicTrade> {
        let data = json.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing data field".into()))?;

        let price = data.get("price_str")
            .or_else(|| data.get("price"))
            .and_then(Self::parse_f64)
            .ok_or_else(|| ExchangeError::Parse("Missing price".into()))?;

        let quantity = data.get("amount_str")
            .or_else(|| data.get("amount"))
            .and_then(Self::parse_f64)
            .ok_or_else(|| ExchangeError::Parse("Missing amount".into()))?;

        let timestamp = data.get("timestamp")
            .and_then(Self::parse_i64)
            .map(|ts| ts * 1000) // Convert to milliseconds
            .unwrap_or(0);

        let id = data.get("id")
            .and_then(|v| v.as_i64())
            .map(|i| i.to_string())
            .unwrap_or_default();

        // Type: 0 = buy, 1 = sell (taker side)
        let side = data.get("type")
            .and_then(|v| v.as_i64())
            .map(|t| if t == 0 { TradeSide::Buy } else { TradeSide::Sell })
            .unwrap_or(TradeSide::Buy);

        let symbol = json.get("channel")
            .and_then(|v| v.as_str())
            .and_then(|s| s.strip_prefix("live_trades_"))
            .unwrap_or("")
            .to_string();

        Ok(PublicTrade {
            id,
            symbol,
            price,
            quantity,
            side,
            timestamp,
        })
    }

    /// Parse WebSocket order book snapshot
    ///
    /// Channel: order_book_{pair}
    /// Message: { data: { timestamp, microtimestamp, bids: [[p, a]], asks: [[p, a]] }, channel, event }
    pub fn parse_ws_orderbook(json: &Value) -> ExchangeResult<OrderBook> {
        let data = json.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing data field".into()))?;

        Self::parse_orderbook(data)
    }

    /// Parse WebSocket orderbook diff update
    ///
    /// Channel: diff_order_book_{pair}
    /// Message: { data: { timestamp, bids: [[p, a]], asks: [[p, a]] }, channel, event }
    ///
    /// Note: Amount "0.00000000" means price level was removed
    pub fn parse_ws_orderbook_diff(json: &Value) -> ExchangeResult<OrderBook> {
        Self::parse_ws_orderbook(json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse Bitstamp trading fees from /api/v2/fees/trading/ response.
    ///
    /// Response: array of { "currency_pair": "btcusd", "fees": { "maker": "0.30", "taker": "0.50" } }
    pub fn parse_fee_rate(json: &Value, symbol: Option<&str>) -> ExchangeResult<crate::core::FeeInfo> {
        Self::check_error(json)?;

        // Response is an array of per-pair fee entries
        let items = match json.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                // Single object response
                let maker = json.get("fees")
                    .and_then(|f| f.get("maker"))
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.3) / 100.0;

                let taker = json.get("fees")
                    .and_then(|f| f.get("taker"))
                    .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| v.as_f64()))
                    .unwrap_or(0.5) / 100.0;

                return Ok(crate::core::FeeInfo {
                    maker_rate: maker,
                    taker_rate: taker,
                    symbol: symbol.map(String::from),
                    tier: None,
                });
            }
        };

        // Find the matching symbol entry or use first entry
        let entry = symbol
            .and_then(|sym| items.iter().find(|item| {
                item.get("currency_pair")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_lowercase().replace("/", "") == sym.to_lowercase().replace("/", ""))
                    .unwrap_or(false)
            }))
            .or_else(|| items.first());

        let (maker, taker) = if let Some(e) = entry {
            let fees = e.get("fees");
            let m = fees.and_then(|f| f.get("maker"))
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64()))
                .unwrap_or(0.3) / 100.0;
            let t = fees.and_then(|f| f.get("taker"))
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| v.as_f64()))
                .unwrap_or(0.5) / 100.0;
            (m, t)
        } else {
            (0.003, 0.005) // Bitstamp default: 0.3% maker, 0.5% taker
        };

        Ok(crate::core::FeeInfo {
            maker_rate: maker,
            taker_rate: taker,
            symbol: symbol.map(String::from),
            tier: None,
        })
    }

    /// Parse user_transactions response as order history.
    ///
    /// Bitstamp /api/v2/user_transactions/ returns trade executions (type=2 = trade),
    /// not open orders. We convert these to Order structs.
    pub fn parse_user_transactions(json: &Value) -> ExchangeResult<Vec<crate::core::Order>> {
        Self::check_error(json)?;

        let items = json.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for user_transactions".to_string()))?;

        let mut orders = Vec::new();

        for item in items {
            // type: 0=deposit, 1=withdrawal, 2=trade — we only care about trades
            let tx_type = item.get("type")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()).or_else(|| v.as_i64()))
                .unwrap_or(-1);

            if tx_type != 2 {
                continue;
            }

            let order_id = item.get("order_id")
                .and_then(|v| v.as_i64().map(|n| n.to_string())
                    .or_else(|| v.as_str().map(String::from)))
                .unwrap_or_default();

            if order_id.is_empty() {
                continue;
            }

            let timestamp = item.get("datetime")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    // Try parsing ISO format
                    chrono::DateTime::parse_from_rfc3339(s).ok()
                        .map(|dt| dt.timestamp_millis())
                })
                .unwrap_or(0);

            // Bitstamp trade transactions don't have a direct side field —
            // we'd need to look at positive/negative amounts. Use a neutral approach.
            orders.push(crate::core::Order {
                id: order_id,
                client_order_id: None,
                symbol: item.get("market")
                    .and_then(|v| v.as_str())
                    .unwrap_or("").to_string(),
                side: crate::core::OrderSide::Buy, // Unknown without deeper parsing
                order_type: crate::core::OrderType::Limit { price: 0.0 },
                status: crate::core::OrderStatus::Filled,
                price: item.get("price").and_then(|v| Self::parse_f64(v)),
                stop_price: None,
                quantity: item.get("amount")
                    .and_then(|v| Self::parse_f64(v))
                    .unwrap_or(0.0),
                filled_quantity: item.get("amount")
                    .and_then(|v| Self::parse_f64(v))
                    .unwrap_or(0.0),
                average_price: item.get("price").and_then(|v| Self::parse_f64(v)),
                commission: item.get("fee").and_then(|v| Self::parse_f64(v)),
                commission_asset: None,
                created_at: timestamp,
                updated_at: Some(timestamp),
                time_in_force: crate::core::TimeInForce::Gtc,
            });
        }

        Ok(orders)
    }

    /// Parse exchange info from Bitstamp trading-pairs-info response.
    ///
    /// Response format:
    /// ```json
    /// [{"name":"BTC/USD","url_symbol":"btcusd","base_decimals":8,"counter_decimals":2,"instant_and_market_orders":"Enabled","minimum_order":"25 USD","trading":"Enabled","description":"Bitcoin / US Dollar"},...]
    /// ```
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let items = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        let mut symbols = Vec::with_capacity(items.len());

        for item in items {
            // Only include enabled pairs
            let trading = item.get("trading").and_then(|v| v.as_str()).unwrap_or("");
            if trading != "Enabled" {
                continue;
            }

            // "name" is "BTC/USD" format
            let name = match item.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => continue,
            };

            let parts: Vec<&str> = name.splitn(2, '/').collect();
            if parts.len() != 2 {
                continue;
            }

            let base_asset = parts[0].trim().to_string();
            let quote_asset = parts[1].trim().to_string();

            // "url_symbol" is "btcusd" - use as canonical symbol
            let symbol = match item.get("url_symbol").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => format!("{}{}", base_asset.to_lowercase(), quote_asset.to_lowercase()),
            };

            let price_precision = item.get("counter_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u8;

            let quantity_precision = item.get("base_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as u8;

            // Derive tick_size from counter_decimals: e.g. 2 decimals -> 0.01
            let tick_size = Some(10f64.powi(-(price_precision as i32)));

            // Derive step_size from base_decimals: e.g. 8 decimals -> 0.00000001
            let step_size = Some(10f64.powi(-(quantity_precision as i32)));

            // "minimum_order" is like "25 USD" - parse the number
            let min_notional = item.get("minimum_order")
                .and_then(|v| v.as_str())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<f64>().ok());

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision,
                min_quantity: None,
                max_quantity: None,
                tick_size,
                step_size,
                min_notional,
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
    fn test_parse_ticker() {
        let data = json!({
            "last": "2211.00",
            "high": "2811.00",
            "low": "2188.97",
            "vwap": "2189.80",
            "volume": "213.26801100",
            "bid": "2188.97",
            "ask": "2211.00",
            "timestamp": "1643640186",
            "pair": "BTC/USD"
        });

        let ticker = BitstampParser::parse_ticker(&data).unwrap();
        assert_eq!(ticker.last_price, 2211.00);
        assert_eq!(ticker.bid_price, Some(2188.97));
        assert_eq!(ticker.ask_price, Some(2211.00));
        assert_eq!(ticker.high_24h, Some(2811.00));
    }

    #[test]
    fn test_parse_orderbook() {
        let data = json!({
            "timestamp": "1643643584",
            "microtimestamp": "1643643584684047",
            "bids": [
                ["9484.34", "1.00000000"],
                ["9483.00", "0.50000000"]
            ],
            "asks": [
                ["9485.00", "1.00000000"],
                ["9486.50", "0.75000000"]
            ]
        });

        let orderbook = BitstampParser::parse_orderbook(&data).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids[0].0, 9484.34);
        assert_eq!(orderbook.asks[0].0, 9485.00);
    }

    #[test]
    fn test_parse_balance() {
        let data = json!([
            {
                "currency": "usd",
                "total": "100.00",
                "available": "90.00",
                "reserved": "10.00"
            },
            {
                "currency": "btc",
                "total": "0.50000000",
                "available": "0.45000000",
                "reserved": "0.05000000"
            }
        ]);

        let balances = BitstampParser::parse_balance(&data).unwrap();
        assert_eq!(balances.len(), 2);
        assert_eq!(balances[0].asset, "USD");
        assert_eq!(balances[0].total, 100.00);
        assert_eq!(balances[0].free, 90.00);
        assert_eq!(balances[0].locked, 10.00);
    }

    #[test]
    fn test_parse_error() {
        let data = json!({
            "status": "error",
            "reason": "Invalid signature",
            "code": "API0007"
        });

        let result = BitstampParser::parse_ticker(&data);
        assert!(result.is_err());
    }
}
