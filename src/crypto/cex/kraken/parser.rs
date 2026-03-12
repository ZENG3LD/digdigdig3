//! # Kraken Response Parser
//!
//! JSON response parsing for Kraken API.
//!
//! ## Response Format
//!
//! Spot API responses:
//! ```json
//! {
//!   "error": [],
//!   "result": { ... }
//! }
//! ```
//!
//! Futures API responses:
//! ```json
//! {
//!   "result": "success",
//!   "data": { ... }
//! }
//! ```

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, SymbolInfo,
};

/// Parser for Kraken API responses
pub struct KrakenParser;

impl KrakenParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract result from Spot API response
    pub fn extract_result(response: &Value) -> ExchangeResult<&Value> {
        // Check for errors first
        if let Some(errors) = response.get("error").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let error_msg = errors
                    .iter()
                    .filter_map(|e| e.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(ExchangeError::Api {
                    code: -1,
                    message: error_msg,
                });
            }
        }

        response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))
    }

    /// Extract data from Futures API response
    pub fn extract_futures_data(response: &Value) -> ExchangeResult<&Value> {
        if response.get("result").and_then(|r| r.as_str()) == Some("error") {
            let error_msg = response.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: -1,
                message: error_msg.to_string(),
            });
        }

        response.as_object()
            .map(|_| response)
            .ok_or_else(|| ExchangeError::Parse("Invalid response format".to_string()))
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
    fn _require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get first key from result object (for symbol lookups)
    fn get_first_key(data: &Value) -> Option<&str> {
        data.as_object()?.keys().next().map(|s| s.as_str())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from Ticker response
    pub fn parse_price(response: &Value, symbol: &str) -> ExchangeResult<f64> {
        let result = Self::extract_result(response)?;

        // Try to get data for the requested symbol
        let ticker = result.get(symbol)
            .or_else(|| {
                // If not found, try to find by any key (response might use full format)
                Self::get_first_key(result).and_then(|k| result.get(k))
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Symbol '{}' not found in response", symbol)))?;

        // Last trade price is in 'c' field: [price, volume]
        ticker.get("c")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(Self::parse_f64)
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid price".to_string()))
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value, symbol: &str) -> ExchangeResult<OrderBook> {
        let result = Self::extract_result(response)?;

        let data = result.get(symbol)
            .or_else(|| Self::get_first_key(result).and_then(|k| result.get(k)))
            .ok_or_else(|| ExchangeError::Parse(format!("Symbol '{}' not found", symbol)))?;

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

        Ok(OrderBook {
            timestamp: chrono::Utc::now().timestamp_millis(),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
        })
    }

    /// Parse klines (OHLC)
    pub fn parse_klines(response: &Value, symbol: &str) -> ExchangeResult<Vec<Kline>> {
        let result = Self::extract_result(response)?;

        let arr = result.get(symbol)
            .or_else(|| Self::get_first_key(result).and_then(|k| result.get(k)))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of klines".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            let candle = item.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if candle.len() < 8 {
                continue;
            }

            // Kraken OHLC format: [time, open, high, low, close, vwap, volume, count]
            let time = candle[0].as_i64().unwrap_or(0);

            klines.push(Kline {
                open_time: time * 1000, // seconds to ms
                open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                volume: Self::parse_f64(&candle[6]).unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: candle[7].as_i64().map(|t| t as u64),
            });
        }

        Ok(klines)
    }

    /// Parse ticker
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let result = Self::extract_result(response)?;

        let data = result.get(symbol)
            .or_else(|| Self::get_first_key(result).and_then(|k| result.get(k)))
            .ok_or_else(|| ExchangeError::Parse(format!("Symbol '{}' not found", symbol)))?;

        // Kraken ticker format:
        // a = ask [price, whole lot volume, lot volume]
        // b = bid [price, whole lot volume, lot volume]
        // c = last trade [price, lot volume]
        // v = volume [today, last 24 hours]
        // p = vwap [today, last 24 hours]
        // t = trades [today, last 24 hours]
        // l = low [today, last 24 hours]
        // h = high [today, last 24 hours]
        // o = today's opening price

        let ask_price = data.get("a")
            .and_then(|a| a.as_array())
            .and_then(|arr| arr.first())
            .and_then(Self::parse_f64);

        let bid_price = data.get("b")
            .and_then(|b| b.as_array())
            .and_then(|arr| arr.first())
            .and_then(Self::parse_f64);

        let last_price = data.get("c")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(Self::parse_f64)
            .unwrap_or(0.0);

        let high_24h = data.get("h")
            .and_then(|h| h.as_array())
            .and_then(|arr| arr.get(1))
            .and_then(Self::parse_f64);

        let low_24h = data.get("l")
            .and_then(|l| l.as_array())
            .and_then(|arr| arr.get(1))
            .and_then(Self::parse_f64);

        let volume_24h = data.get("v")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(1))
            .and_then(Self::parse_f64);

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
            price_change_percent_24h: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order ID from AddOrder response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let result = Self::extract_result(response)?;

        // Response contains "txid" array
        result.get("txid")
            .and_then(|txid| txid.as_array())
            .and_then(|arr| arr.first())
            .and_then(|id| id.as_str())
            .map(String::from)
            .ok_or_else(|| ExchangeError::Parse("Missing order ID".to_string()))
    }

    /// Parse order from QueryOrders response
    pub fn parse_order(response: &Value, order_id: &str) -> ExchangeResult<Order> {
        let result = Self::extract_result(response)?;

        let data = result.get(order_id)
            .ok_or_else(|| ExchangeError::Parse(format!("Order '{}' not found", order_id)))?;

        Self::parse_order_data(data, order_id)
    }

    /// Parse order from order data object
    fn parse_order_data(data: &Value, order_id: &str) -> ExchangeResult<Order> {
        let descr = data.get("descr")
            .ok_or_else(|| ExchangeError::Parse("Missing order description".to_string()))?;

        let side = match Self::get_str(descr, "type").unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(descr, "ordertype").unwrap_or("limit") {
            "market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = match Self::get_str(data, "status").unwrap_or("pending") {
            "canceled" | "cancelled" => OrderStatus::Canceled,
            "closed" => OrderStatus::Filled,
            "open" => {
                let vol_exec = Self::get_f64(data, "vol_exec").unwrap_or(0.0);
                if vol_exec > 0.0 {
                    OrderStatus::PartiallyFilled
                } else {
                    OrderStatus::New
                }
            }
            _ => OrderStatus::New,
        };

        let price = Self::get_str(descr, "price")
            .and_then(|s| s.parse().ok());

        let quantity = Self::get_f64(data, "vol").unwrap_or(0.0);
        let filled_quantity = Self::get_f64(data, "vol_exec").unwrap_or(0.0);

        let average_price = Self::get_f64(data, "price");

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: Self::get_str(data, "userref").map(String::from),
            symbol: Self::get_str(descr, "pair").unwrap_or("").to_string(),
            side,
            order_type,
            status,
            price,
            stop_price: None,
            quantity,
            filled_quantity,
            average_price,
            commission: Self::get_f64(data, "fee"),
            commission_asset: None,
            created_at: (Self::get_f64(data, "opentm").unwrap_or(0.0) * 1000.0) as i64,
            updated_at: Self::get_f64(data, "closetm").map(|t| (t * 1000.0) as i64),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse open orders
    pub fn parse_open_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let result = Self::extract_result(response)?;

        let open = result.get("open")
            .and_then(|o| o.as_object())
            .ok_or_else(|| ExchangeError::Parse("Expected 'open' object".to_string()))?;

        open.iter()
            .map(|(order_id, data)| Self::parse_order_data(data, order_id))
            .collect()
    }

    /// Parse closed orders (order history) from ClosedOrders response
    pub fn parse_closed_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let result = Self::extract_result(response)?;

        let closed = result.get("closed")
            .and_then(|o| o.as_object())
            .ok_or_else(|| ExchangeError::Parse("Expected 'closed' object".to_string()))?;

        closed.iter()
            .map(|(order_id, data)| Self::parse_order_data(data, order_id))
            .collect()
    }

    /// Parse futures fills (trade history) from Futures fills response
    pub fn parse_futures_fills(response: &Value) -> ExchangeResult<Vec<Order>> {
        // Kraken Futures fills: {"result": "success", "fills": [...]}
        let fills = response.get("fills")
            .and_then(|f| f.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected 'fills' array".to_string()))?;

        fills.iter().map(|fill| {
            let order_id = fill.get("order_id")
                .or_else(|| fill.get("fill_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let side = match fill.get("side").and_then(|s| s.as_str()).unwrap_or("buy") {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let price = fill.get("price")
                .and_then(|p| p.as_f64())
                .or_else(|| fill.get("price").and_then(|p| p.as_str()).and_then(|s| s.parse().ok()));

            let quantity = fill.get("size")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);

            let symbol = fill.get("symbol")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let ts_ms = fill.get("fillTime")
                .or_else(|| fill.get("time"))
                .and_then(|t| t.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .map(|t| (t * 1000.0) as i64)
                .unwrap_or(0);

            Ok(Order {
                id: order_id,
                client_order_id: None,
                symbol,
                side,
                order_type: OrderType::Market,
                status: crate::core::OrderStatus::Filled,
                price,
                stop_price: None,
                quantity,
                filled_quantity: quantity,
                average_price: price,
                commission: fill.get("fee").and_then(|f| f.as_f64()),
                commission_asset: None,
                created_at: ts_ms,
                updated_at: Some(ts_ms),
                time_in_force: crate::core::TimeInForce::Gtc,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balances
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let result = Self::extract_result(response)?;

        let balances_obj = result.as_object()
            .ok_or_else(|| ExchangeError::Parse("Expected object of balances".to_string()))?;

        let mut balances = Vec::new();

        for (asset, amount_val) in balances_obj {
            // Skip balance extensions (.F, .B, .T, .S, .M)
            if asset.contains('.') {
                continue;
            }

            let amount = Self::parse_f64(amount_val).unwrap_or(0.0);

            if amount > 0.0 {
                // Strip X/Z prefixes
                let clean_asset = asset
                    .strip_prefix("X")
                    .or_else(|| asset.strip_prefix("Z"))
                    .unwrap_or(asset);

                balances.push(Balance {
                    asset: clean_asset.to_string(),
                    free: amount,
                    locked: 0.0,
                    total: amount,
                });
            }
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUTURES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse futures positions
    pub fn parse_futures_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_futures_data(response)?;

        let positions = data.get("openPositions")
            .or_else(|| data.get("positions"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected positions array".to_string()))?;

        positions.iter()
            .filter_map(Self::parse_position_data)
            .collect::<Result<Vec<_>, _>>()
    }

    fn parse_position_data(data: &Value) -> Option<ExchangeResult<Position>> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
        let size = Self::get_f64(data, "size")?;

        if size.abs() < f64::EPSILON {
            return None; // Skip empty positions
        }

        let side = if size > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Ok(Position {
            symbol,
            side,
            quantity: size.abs(),
            entry_price: Self::get_f64(data, "fillPrice").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "markPrice"),
            unrealized_pnl: Self::get_f64(data, "pnl").unwrap_or(0.0),
            realized_pnl: None,
            leverage: 1,
            liquidation_price: None,
            margin: None,
            margin_type: crate::core::MarginType::Cross,
            take_profit: None,
            stop_loss: None,
        }))
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value, symbol: &str) -> ExchangeResult<FundingRate> {
        let data = Self::extract_futures_data(response)?;

        let rates = data.get("rates")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing rates array".to_string()))?;

        let latest = rates.last()
            .ok_or_else(|| ExchangeError::Parse("No funding rate data".to_string()))?;

        Ok(FundingRate {
            symbol: symbol.to_string(),
            rate: Self::require_f64(latest, "fundingRate")?,
            next_funding_time: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Kraken AssetPairs response.
    ///
    /// Response format:
    /// ```json
    /// {"error":[],"result":{"XXBTZUSD":{"wsname":"XBT/USD","base":"XXBT","quote":"ZUSD","pair_decimals":1,"lot_decimals":8,...},...}}
    /// ```
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let result = Self::extract_result(response)?;

        let pairs = result.as_object()
            .ok_or_else(|| ExchangeError::Parse("Expected object in result".to_string()))?;

        let mut symbols = Vec::with_capacity(pairs.len());

        for (pair_name, pair_data) in pairs {
            // Skip pairs that are "alt" variants (Kraken sometimes returns .d suffix alternates)
            if pair_name.ends_with(".d") {
                continue;
            }

            // Skip if pair_data is not an object
            let data = match pair_data.as_object() {
                Some(d) => d,
                None => continue,
            };

            // Extract base and quote from wsname (e.g. "XBT/USD") if available,
            // otherwise fall back to base/quote fields
            let (base_asset, quote_asset) = if let Some(wsname) = data.get("wsname").and_then(|v| v.as_str()) {
                let parts: Vec<&str> = wsname.splitn(2, '/').collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    let base = data.get("base").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let quote = data.get("quote").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    (base, quote)
                }
            } else {
                let base = data.get("base").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let quote = data.get("quote").and_then(|v| v.as_str()).unwrap_or("").to_string();
                (base, quote)
            };

            // Filter out pairs with empty base or quote
            if base_asset.is_empty() || quote_asset.is_empty() {
                continue;
            }

            // Only include pairs with "online" status (if present)
            let status = data.get("status").and_then(|v| v.as_str()).unwrap_or("online");
            if status != "online" && !status.is_empty() {
                continue;
            }

            let price_precision = data.get("pair_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as u8;

            let quantity_precision = data.get("lot_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as u8;

            let min_quantity = data.get("ordermin")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| data.get("ordermin").and_then(|v| v.as_f64()));

            symbols.push(SymbolInfo {
                symbol: pair_name.clone(),
                base_asset,
                quote_asset,
                status: "TRADING".to_string(),
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity: None,
                step_size: None,
                min_notional: None,
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
    fn test_parse_price() {
        let response = json!({
            "error": [],
            "result": {
                "XXBTZUSD": {
                    "c": ["42000.50", "1.5"]
                }
            }
        });

        let price = KrakenParser::parse_price(&response, "XXBTZUSD").unwrap();
        assert!((price - 42000.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "error": [],
            "result": {
                "XXBTZUSD": {
                    "asks": [["42000.0", "1.5", 1234567890]],
                    "bids": [["41999.0", "2.0", 1234567890]]
                }
            }
        });

        let orderbook = KrakenParser::parse_orderbook(&response, "XXBTZUSD").unwrap();
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks.len(), 1);
        assert!((orderbook.bids[0].0 - 41999.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "error": [],
            "result": {
                "XXBTZUSD": {
                    "a": ["42001.0", "1", "1"],
                    "b": ["42000.0", "2", "2"],
                    "c": ["42000.5", "0.5"],
                    "h": ["42500.0", "42600.0"],
                    "l": ["41500.0", "41400.0"],
                    "v": ["100.0", "200.0"]
                }
            }
        });

        let ticker = KrakenParser::parse_ticker(&response, "XXBTZUSD").unwrap();
        assert!((ticker.last_price - 42000.5).abs() < f64::EPSILON);
        assert!((ticker.bid_price.unwrap() - 42000.0).abs() < f64::EPSILON);
        assert!((ticker.ask_price.unwrap() - 42001.0).abs() < f64::EPSILON);
    }
}
