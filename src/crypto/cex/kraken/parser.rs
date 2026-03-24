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
    CancelAllResponse, OrderResult,
    DepositAddress, WithdrawResponse, FundsRecord,
    SubAccountResult, SubAccount,
    UserTrade,
    FundingPayment, LedgerEntry, LedgerEntryType,
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

            // tick_size: price increment from the "tick_size" field (string like "0.1")
            let tick_size = data.get("tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // step_size: derived from lot_decimals (e.g. 8 → 0.00000001)
            let step_size = {
                let decimals = data.get("lot_decimals")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(8);
                Some(10f64.powi(-(decimals as i32)))
            };

            symbols.push(SymbolInfo {
                symbol: pair_name.clone(),
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
                account_type: Default::default(),
            });
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CANCEL ALL
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /0/private/CancelAll.
    ///
    /// Kraken returns `{"error":[],"result":{"count":5}}` — count of cancelled orders.
    pub fn parse_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        let result = Self::extract_result(response)?;

        let count = result.get("count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as u32;

        Ok(CancelAllResponse {
            cancelled_count: count,
            failed_count: 0,
            details: vec![],
        })
    }

    /// Parse response from Kraken Futures cancel-all endpoint.
    ///
    /// Futures returns `{"result":"success","cancelAllStatus":[...]}`.
    pub fn parse_futures_cancel_all_response(response: &Value) -> ExchangeResult<CancelAllResponse> {
        if response.get("result").and_then(|r| r.as_str()) == Some("error") {
            let error_msg = response.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: -1,
                message: error_msg.to_string(),
            });
        }

        let cancelled = response.get("cancelAllStatus")
            .and_then(|arr| arr.as_array())
            .map(|arr| arr.iter().filter(|item| {
                item.get("status").and_then(|s| s.as_str()) == Some("cancelled")
            }).count() as u32)
            .unwrap_or(0);

        Ok(CancelAllResponse {
            cancelled_count: cancelled,
            failed_count: 0,
            details: vec![],
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AMEND ORDER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /0/private/EditOrder (Spot).
    ///
    /// Kraken returns `{"error":[],"result":{"descr":{...},"txid":"NEW_ORDER_ID"}}`.
    pub fn parse_amend_spot_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        let result = Self::extract_result(response)?;

        let txid = result.get("txid")
            .and_then(|t| t.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'txid' in EditOrder response".to_string()))?;

        Ok(Order {
            id: txid.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side: OrderSide::Buy, // Kraken EditOrder doesn't return full order; side unknown
            order_type: OrderType::Limit { price: 0.0 },
            status: crate::core::OrderStatus::Open,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: Some(crate::core::timestamp_millis() as i64),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse response from POST /derivatives/api/v3/editorder (Futures).
    ///
    /// Futures returns `{"result":"success","editStatus":{"orderId":"NEW_ID",...}}`.
    pub fn parse_amend_futures_order(response: &Value, symbol: &str) -> ExchangeResult<Order> {
        Self::extract_futures_data(response)?;

        let order_id = response.get("editStatus")
            .and_then(|s| s.get("orderId"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'editStatus.orderId' in editorder response".to_string()))?;

        Ok(Order {
            id: order_id.to_string(),
            client_order_id: None,
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit { price: 0.0 },
            status: crate::core::OrderStatus::Open,
            price: None,
            stop_price: None,
            quantity: 0.0,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: 0,
            updated_at: Some(crate::core::timestamp_millis() as i64),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BATCH ORDERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse response from POST /derivatives/api/v3/batchorder (Futures).
    ///
    /// Kraken Futures batch returns `{"result":"success","batchStatus":[...]}`.
    /// Each item has `order_id` (success) or `error` (failure).
    pub fn parse_batch_orders_response(response: &Value) -> ExchangeResult<Vec<OrderResult>> {
        Self::extract_futures_data(response)?;

        let items = response.get("batchStatus")
            .and_then(|arr| arr.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'batchStatus' array in batchorder response".to_string()))?;

        Ok(items.iter().map(|item| {
            if let Some(error) = item.get("error").and_then(|e| e.as_str()) {
                return OrderResult {
                    order: None,
                    client_order_id: item.get("cl_ord_id").and_then(|v| v.as_str()).map(String::from),
                    success: false,
                    error: Some(error.to_string()),
                    error_code: None,
                };
            }

            let order_id = item.get("order_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            OrderResult {
                order: Some(Order {
                    id: order_id,
                    client_order_id: item.get("cl_ord_id").and_then(|v| v.as_str()).map(String::from),
                    symbol: String::new(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::New,
                    price: None,
                    stop_price: None,
                    quantity: 0.0,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }),
                client_order_id: None,
                success: true,
                error: None,
                error_code: None,
            }
        }).collect())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTODIAL FUNDS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse deposit address from `POST /0/private/DepositAddresses`
    ///
    /// Response result is an array of address objects:
    /// ```json
    /// [{"address":"1A1zP1...","expiretm":"0","new":true}]
    /// ```
    pub fn parse_deposit_address(response: &Value, asset: &str) -> ExchangeResult<DepositAddress> {
        let result = Self::extract_result(response)?;
        let arr = result.as_array()
            .ok_or_else(|| ExchangeError::Parse("DepositAddresses result is not an array".to_string()))?;

        let first = arr.first()
            .ok_or_else(|| ExchangeError::Parse("No deposit addresses returned".to_string()))?;

        let address = first.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing deposit address".to_string()))?
            .to_string();

        let tag = first.get("memo")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network: None,
            asset: asset.to_string(),
            created_at: None,
        })
    }

    /// Parse withdraw response from `POST /0/private/Withdraw`
    ///
    /// Response result: `{"refid":"AGBSO6T-..."}` for spot
    pub fn parse_withdraw_response(response: &Value) -> ExchangeResult<WithdrawResponse> {
        let result = Self::extract_result(response)?;

        let withdraw_id = result.get("refid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing withdrawal refid".to_string()))?
            .to_string();

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Parse deposit history from `POST /0/private/DepositStatus`
    ///
    /// Response result is an array:
    /// ```json
    /// [{"method":"Bitcoin","aclass":"currency","asset":"XXBT","refid":"...","txid":"...","info":"...","amount":"0.5","fee":"0.0001","time":1234567890,"status":"Success","status-prop":"return"}]
    /// ```
    pub fn parse_deposit_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let result = Self::extract_result(response)?;
        let arr = match result.as_array() {
            Some(a) => a,
            None => return Ok(vec![]),
        };

        let records = arr.iter().map(|item| {
            let id = item.get("refid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = item.get("asset")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let tx_hash = item.get("txid")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let timestamp = item.get("time")
                .and_then(|v| v.as_i64())
                .map(|t| t * 1000) // Kraken returns Unix seconds
                .unwrap_or(0);

            FundsRecord::Deposit {
                id,
                asset,
                amount,
                tx_hash,
                network: None,
                status,
                timestamp,
            }
        }).collect();

        Ok(records)
    }

    /// Parse withdrawal history from `POST /0/private/WithdrawStatus`
    ///
    /// Response result is an array of withdrawal objects.
    pub fn parse_withdrawal_history(response: &Value) -> ExchangeResult<Vec<FundsRecord>> {
        let result = Self::extract_result(response)?;
        let arr = match result.as_array() {
            Some(a) => a,
            None => return Ok(vec![]),
        };

        let records = arr.iter().map(|item| {
            let id = item.get("refid")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = item.get("asset")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let fee = item.get("fee")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());
            let address = item.get("info")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tx_hash = item.get("txid")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let timestamp = item.get("time")
                .and_then(|v| v.as_i64())
                .map(|t| t * 1000)
                .unwrap_or(0);

            FundsRecord::Withdrawal {
                id,
                asset,
                amount,
                fee,
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

    // ═══════════════════════════════════════════════════════════════════════════
    // SUB-ACCOUNTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse list of sub-accounts from `POST /0/private/ListSubaccounts`
    ///
    /// Response result is an array of account objects.
    pub fn parse_list_subaccounts(response: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(response)?;
        let arr = match result.as_array() {
            Some(a) => a,
            None => {
                return Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                });
            }
        };

        let accounts = arr.iter().map(|item| {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("username").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            let name = item.get("username")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("name").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            SubAccount {
                id,
                name,
                status: "Normal".to_string(),
            }
        }).collect();

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts,
            transaction_id: None,
        })
    }

    /// Parse sub-account transfer response
    ///
    /// Response result: `{"transfer_id":"..."}` or similar.
    pub fn parse_subaccount_transfer(response: &Value) -> ExchangeResult<SubAccountResult> {
        let result = Self::extract_result(response)?;

        let transaction_id = result.get("transfer_id")
            .or_else(|| result.get("refid"))
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(SubAccountResult {
            id: None,
            name: None,
            accounts: vec![],
            transaction_id,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // USER TRADES (FILLS / TRADE HISTORY)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse personal trade fills from `POST /0/private/TradesHistory`.
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "result": {
    ///     "trades": {
    ///       "TXID1": {
    ///         "ordertxid": "ORDER1",
    ///         "pair": "XXBTZUSD",
    ///         "type": "buy",
    ///         "price": "50000.0",
    ///         "vol": "0.001",
    ///         "fee": "0.01",
    ///         "time": 1672531200.123,
    ///         "misc": "",
    ///         "maker": true
    ///       }
    ///     },
    ///     "count": 100
    ///   }
    /// }
    /// ```
    ///
    /// Notes:
    /// - `time` is float Unix seconds — converted to milliseconds for `timestamp`.
    /// - Trade ID is the key of the `trades` object (e.g. `TXID1`).
    /// - `fee` is in the quote currency of the pair.
    /// - `maker` field indicates maker vs taker.
    pub fn parse_trades_history(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let result = Self::extract_result(response)?;

        let trades_obj = result.get("trades")
            .and_then(|t| t.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'trades' object in TradesHistory response".to_string()))?;

        let mut trades = Vec::with_capacity(trades_obj.len());

        for (trade_id, data) in trades_obj {
            let order_id = Self::get_str(data, "ordertxid")
                .unwrap_or("")
                .to_string();

            let symbol = Self::get_str(data, "pair")
                .unwrap_or("")
                .to_string();

            let side = match Self::get_str(data, "type").unwrap_or("buy") {
                "sell" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let price = Self::require_f64(data, "price")?;
            let quantity = Self::require_f64(data, "vol")?;
            let commission = Self::get_f64(data, "fee").unwrap_or(0.0);

            // Quote currency of the pair is the commission asset.
            // Kraken fee is always in the quote currency for spot trades.
            // We extract the quote from the pair string (e.g. XXBTZUSD → USD).
            let commission_asset = Self::extract_quote_from_pair(&symbol);

            // `time` is float Unix seconds; convert to integer milliseconds.
            let timestamp = data.get("time")
                .and_then(Self::parse_f64)
                .map(|t| (t * 1000.0) as i64)
                .unwrap_or(0);

            // `maker` is a boolean field; absent means taker.
            let is_maker = data.get("maker")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            trades.push(UserTrade {
                id: trade_id.clone(),
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

    /// Extract the quote currency from a Kraken pair string.
    ///
    /// Kraken response pairs use full ISO format (XXBTZUSD, XETHZUSD).
    /// We strip the Z-prefixed fiat suffix and return the clean currency name.
    ///
    /// Falls back to the last 3 characters of the pair if no known suffix matches.
    fn extract_quote_from_pair(pair: &str) -> String {
        // Known fiat suffixes with Z prefix
        for fiat in &["ZUSD", "ZEUR", "ZGBP", "ZJPY", "ZCAD", "ZCHF"] {
            if pair.ends_with(fiat) {
                return fiat.strip_prefix('Z').unwrap_or(fiat).to_string();
            }
        }

        // Crypto quote pairs (e.g., XETHXXBT → XBT)
        if pair.len() >= 3 {
            return pair[pair.len() - 3..].to_string();
        }

        pair.to_string()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding payment history from `POST /0/private/Ledgers` (type=rollover).
    ///
    /// Kraken ledger response:
    /// ```json
    /// { "result": { "ledger": { "LXXX": { "asset": "XXBT", "type": "rollover",
    ///   "time": 1234567890.5, "amount": "-0.0001", "balance": "1.0" } } } }
    /// ```
    pub fn parse_funding_payments(response: &Value) -> ExchangeResult<Vec<FundingPayment>> {
        let result = Self::extract_result(response)?;
        let ledger = result.get("ledger")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'ledger' object".to_string()))?;

        let mut payments = Vec::new();
        for (id, entry) in ledger {
            // Only rollover entries are funding payments
            let entry_type = Self::get_str(entry, "type").unwrap_or("");
            if entry_type != "rollover" {
                continue;
            }

            let asset_raw = Self::get_str(entry, "asset").unwrap_or("");
            let asset = Self::normalize_kraken_asset(asset_raw);
            let amount = Self::get_f64(entry, "amount").unwrap_or(0.0);
            // Kraken time is float seconds
            let timestamp = entry.get("time")
                .and_then(|t| t.as_f64())
                .map(|t| (t * 1000.0) as i64)
                .unwrap_or(0);

            payments.push(FundingPayment {
                symbol: id.clone(),
                funding_rate: 0.0, // Kraken doesn't expose rate in ledger entries
                position_size: 0.0,
                payment: amount,
                asset,
                timestamp,
            });
        }

        // Sort by timestamp ascending
        payments.sort_by_key(|p| p.timestamp);
        Ok(payments)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse account ledger from `POST /0/private/Ledgers`.
    ///
    /// Kraken ledger response:
    /// ```json
    /// { "result": { "ledger": { "LXXX": { "asset": "ZUSD", "type": "trade",
    ///   "time": 1234567890.5, "amount": "100.0", "balance": "1000.0",
    ///   "refid": "TXXX", "fee": "0.25" } } } }
    /// ```
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let result = Self::extract_result(response)?;
        let ledger = result.get("ledger")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'ledger' object".to_string()))?;

        let mut entries = Vec::new();
        for (id, entry) in ledger {
            let asset_raw = Self::get_str(entry, "asset").unwrap_or("");
            let asset = Self::normalize_kraken_asset(asset_raw);
            let amount = Self::get_f64(entry, "amount").unwrap_or(0.0);
            let balance = Self::get_f64(entry, "balance");
            let type_str = Self::get_str(entry, "type").unwrap_or("");
            let entry_type = Self::map_kraken_ledger_type(type_str);
            let description = type_str.to_string();
            let ref_id = Self::get_str(entry, "refid").map(String::from);
            let timestamp = entry.get("time")
                .and_then(|t| t.as_f64())
                .map(|t| (t * 1000.0) as i64)
                .unwrap_or(0);

            entries.push(LedgerEntry {
                id: id.clone(),
                asset,
                amount,
                balance,
                entry_type,
                description,
                ref_id,
                timestamp,
            });
        }

        // Sort by timestamp ascending
        entries.sort_by_key(|e| e.timestamp);
        Ok(entries)
    }

    fn map_kraken_ledger_type(type_str: &str) -> LedgerEntryType {
        match type_str {
            "trade" => LedgerEntryType::Trade,
            "deposit" => LedgerEntryType::Deposit,
            "withdrawal" => LedgerEntryType::Withdrawal,
            "rollover" => LedgerEntryType::Funding,
            "fee" => LedgerEntryType::Fee,
            "rebate" => LedgerEntryType::Rebate,
            "transfer" => LedgerEntryType::Transfer,
            "margin" => LedgerEntryType::Trade,
            "settlement" => LedgerEntryType::Settlement,
            "adjustment" => LedgerEntryType::Other("adjustment".to_string()),
            other => LedgerEntryType::Other(other.to_string()),
        }
    }

    /// Normalize Kraken asset names by stripping X/Z prefixes.
    ///
    /// - XXBT → XBT
    /// - ZUSD → USD
    /// - XETH → ETH
    fn normalize_kraken_asset(asset: &str) -> String {
        // XXBT is a special case: strip one X → XBT
        if asset.starts_with("XX") {
            return asset[1..].to_string();
        }
        // XETH, XLTC, XXRP: strip leading X for crypto
        if asset.len() == 4 && asset.starts_with('X') {
            return asset[1..].to_string();
        }
        // ZUSD, ZEUR, ZGBP, ZCAD, ZJPY: strip leading Z for fiat
        if asset.len() == 4 && asset.starts_with('Z') {
            return asset[1..].to_string();
        }
        asset.to_string()
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
