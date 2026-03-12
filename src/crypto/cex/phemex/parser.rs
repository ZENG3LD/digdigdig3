//! # Phemex Response Parser
//!
//! Парсинг JSON ответов от Phemex API.
//!
//! ## CRITICAL: REST vs WebSocket
//! - REST responses use `/md/*` format with `result` field
//! - Other REST endpoints use `code`, `msg`, `data` format
//! - WebSocket uses different structure
//!
//! ## Value Scaling
//! Phemex uses integer representation:
//! - Ep (Price): scaled by priceScale
//! - Er (Ratio): scaled by ratioScale
//! - Ev (Value): scaled by valueScale

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult, AccountType,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, SymbolInfo, BracketResponse,
};

use super::endpoints::{unscale_price, unscale_value};

/// Парсер ответов Phemex API
pub struct PhemexParser;

impl PhemexParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract data from standard REST response (code/msg/data format)
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        // Check for error code
        if let Some(code) = response.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = response.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: code as i32,
                    message: msg.to_string(),
                });
            }
        }

        // Check for bizError in data
        if let Some(data) = response.get("data") {
            if let Some(biz_error) = data.get("bizError").and_then(|e| e.as_i64()) {
                if biz_error != 0 {
                    return Err(ExchangeError::Api {
                        code: biz_error as i32,
                        message: "Business error".to_string(),
                    });
                }
            }
            Ok(data)
        } else {
            Err(ExchangeError::Parse("Missing 'data' field".to_string()))
        }
    }

    /// Extract result from market data response (/md/* endpoints)
    pub fn extract_result(response: &Value) -> ExchangeResult<&Value> {
        if let Some(error) = response.get("error") {
            if !error.is_null() {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("Error: {:?}", error),
                });
            }
        }

        response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))
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
    fn _require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    /// Parse required i64
    fn _require_i64(data: &Value, key: &str) -> ExchangeResult<i64> {
        Self::get_i64(data, key)
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

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse server time (nanoseconds)
    pub fn parse_server_time(response: &Value) -> ExchangeResult<i64> {
        let result = Self::extract_result(response)?;
        result.as_i64()
            .map(|ns| ns / 1_000_000) // Convert nanoseconds to milliseconds
            .ok_or_else(|| ExchangeError::Parse("Invalid server time".to_string()))
    }

    /// Parse orderbook (market data format with result field)
    pub fn parse_orderbook(response: &Value, price_scale: u8) -> ExchangeResult<OrderBook> {
        let result = Self::extract_result(response)?;

        let book = result.get("book")
            .ok_or_else(|| ExchangeError::Parse("Missing 'book' field".to_string()))?;

        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            book.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price_ep = pair[0].as_i64()?;
                            let size = Self::parse_f64(&pair[1])?;
                            let price = unscale_price(price_ep, price_scale);
                            Some((price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(OrderBook {
            timestamp: result.get("timestamp")
                .and_then(|t| t.as_i64())
                .map(|ns| ns / 1_000_000) // nanoseconds to milliseconds
                .unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: result.get("sequence")
                .and_then(|s| s.as_i64())
                .map(|n| n.to_string()),
        })
    }

    /// Parse ticker (market data format)
    /// Note: Spot uses "lastEp", "bidEp", etc. Contract uses "close", "markPrice", etc.
    pub fn parse_ticker(response: &Value, price_scale: u8, account_type: AccountType) -> ExchangeResult<Ticker> {
        let result = Self::extract_result(response)?;

        let (last_price, bid_price, ask_price, high_24h, low_24h, open_price) = match account_type {
            AccountType::Spot => {
                // Spot format: uses "Ep" suffix (scaled integers)
                let last_ep = Self::get_i64(result, "lastEp").unwrap_or(0);
                let bid_ep = Self::get_i64(result, "bidEp");
                let ask_ep = Self::get_i64(result, "askEp");
                let high_ep = Self::get_i64(result, "highEp");
                let low_ep = Self::get_i64(result, "lowEp");
                let open_ep = Self::get_i64(result, "openEp");

                (
                    unscale_price(last_ep, price_scale),
                    bid_ep.map(|p| unscale_price(p, price_scale)),
                    ask_ep.map(|p| unscale_price(p, price_scale)),
                    high_ep.map(|p| unscale_price(p, price_scale)),
                    low_ep.map(|p| unscale_price(p, price_scale)),
                    open_ep.map(|p| unscale_price(p, price_scale)),
                )
            }
            _ => {
                // Contract format: uses direct scaled integers without "Ep" suffix
                let close_ep = Self::get_i64(result, "close").unwrap_or(0);
                let high_ep = Self::get_i64(result, "high");
                let low_ep = Self::get_i64(result, "low");
                let open_ep = Self::get_i64(result, "open");

                (
                    unscale_price(close_ep, price_scale),
                    None, // Contract ticker doesn't provide bid
                    None, // Contract ticker doesn't provide ask
                    high_ep.map(|p| unscale_price(p, price_scale)),
                    low_ep.map(|p| unscale_price(p, price_scale)),
                    open_ep.map(|p| unscale_price(p, price_scale)),
                )
            }
        };

        Ok(Ticker {
            symbol: Self::get_str(result, "symbol").unwrap_or("").to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h: Self::get_f64(result, "volume"),
            quote_volume_24h: None,
            price_change_24h: open_price.map(|o| last_price - o),
            price_change_percent_24h: open_price.map(|o| {
                if o > 0.0 {
                    ((last_price - o) / o) * 100.0
                } else {
                    0.0
                }
            }),
            timestamp: result.get("timestamp")
                .and_then(|t| t.as_i64())
                .map(|ns| ns / 1_000_000)
                .unwrap_or(0),
        })
    }

    /// Parse klines
    pub fn parse_klines(response: &Value, price_scale: u8) -> ExchangeResult<Vec<Kline>> {
        let data = Self::extract_data(response)?;
        let rows = data.get("rows")
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'rows' field".to_string()))?;

        let mut klines = Vec::with_capacity(rows.len());

        for row in rows {
            let arr = row.as_array()
                .ok_or_else(|| ExchangeError::Parse("Kline is not an array".to_string()))?;

            if arr.len() < 8 {
                continue;
            }

            // Phemex format: [timestamp, interval, openEp, closeEp, highEp, lowEp, volume, turnoverEv]
            let open_time = arr[0].as_i64().unwrap_or(0) * 1000; // seconds to ms
            let open_ep = arr[2].as_i64().unwrap_or(0);
            let close_ep = arr[3].as_i64().unwrap_or(0);
            let high_ep = arr[4].as_i64().unwrap_or(0);
            let low_ep = arr[5].as_i64().unwrap_or(0);

            klines.push(Kline {
                open_time,
                open: unscale_price(open_ep, price_scale),
                close: unscale_price(close_ep, price_scale),
                high: unscale_price(high_ep, price_scale),
                low: unscale_price(low_ep, price_scale),
                volume: Self::parse_f64(&arr[6]).unwrap_or(0.0),
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let data = Self::extract_data(response)?;

        // Funding rate is in Er format (ratioScale = 8)
        let rate_er = Self::get_i64(data, "fundingRateEr").unwrap_or(0);
        let rate = rate_er as f64 / 100_000_000.0; // ratioScale = 8

        Ok(FundingRate {
            symbol: Self::get_str(data, "symbol").unwrap_or("").to_string(),
            rate,
            next_funding_time: None,
            timestamp: data.get("timestamp")
                .and_then(|t| t.as_i64())
                .map(|ns| ns / 1_000_000)
                .unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order from response
    pub fn parse_order(response: &Value, symbol: &str, price_scale: u8) -> ExchangeResult<Order> {
        let data = Self::extract_data(response)?;
        Self::parse_order_data(data, symbol, price_scale)
    }

    /// Parse order from data object
    pub fn parse_order_data(data: &Value, symbol: &str, price_scale: u8) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("Buy") {
            "Sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "ordType")
            .or_else(|| Self::get_str(data, "orderType"))
            .unwrap_or("Limit")
        {
            "Market" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        let price_ep = Self::get_i64(data, "priceEp");
        let stop_price_ep = Self::get_i64(data, "stopPxEp");

        // For spot: baseQtyEv or quoteQtyEv
        // For contract: orderQty
        let quantity = Self::get_i64(data, "orderQty")
            .map(|q| q as f64)
            .or_else(|| Self::get_i64(data, "baseQtyEv").map(|q| unscale_value(q, 8)))
            .unwrap_or(0.0);

        let filled_quantity = Self::get_i64(data, "cumQty")
            .map(|q| q as f64)
            .unwrap_or(0.0);

        Ok(Order {
            id: Self::get_str(data, "orderID")
                .unwrap_or("")
                .to_string(),
            client_order_id: Self::get_str(data, "clOrdID").map(String::from),
            symbol: Self::get_str(data, "symbol").unwrap_or(symbol).to_string(),
            side,
            order_type,
            status,
            price: price_ep.map(|p| unscale_price(p, price_scale)),
            stop_price: stop_price_ep.map(|p| unscale_price(p, price_scale)),
            quantity,
            filled_quantity,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: data.get("createTimeNs")
                .or_else(|| data.get("actionTimeNs"))
                .and_then(|t| t.as_i64())
                .map(|ns| ns / 1_000_000)
                .unwrap_or(0),
            updated_at: data.get("transactTimeNs")
                .and_then(|t| t.as_i64())
                .map(|ns| ns / 1_000_000),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "ordStatus").unwrap_or("New") {
            "New" | "Untriggered" => OrderStatus::New,
            "PartiallyFilled" => OrderStatus::PartiallyFilled,
            "Filled" => OrderStatus::Filled,
            "Canceled" | "Cancelled" => OrderStatus::Canceled,
            "Rejected" => OrderStatus::Rejected,
            "Triggered" => OrderStatus::New, // Conditional order triggered, now active
            _ => OrderStatus::New,
        }
    }

    /// Parse list of orders
    pub fn parse_orders(response: &Value, price_scale: u8) -> ExchangeResult<Vec<Order>> {
        let data = Self::extract_data(response)?;

        let orders_array = data.get("rows")
            .or_else(|| data.get("data"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        orders_array.iter()
            .map(|item| Self::parse_order_data(item, "", price_scale))
            .collect()
    }

    /// Parse order ID from create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        let data = Self::extract_data(response)?;
        Self::require_str(data, "orderID").map(String::from)
    }

    /// Parse Bracket order response (Phemex ordType=11).
    ///
    /// Phemex returns the entry order in `data`.  The TP and SL legs (ordType 12/14)
    /// may appear in `data.takeProfitOrder` and `data.stopLossOrder` sub-objects when
    /// the exchange includes them, otherwise we construct minimal pending placeholders
    /// from the `takeProfitEp` / `stopLossEp` fields on the entry order data.
    pub fn parse_bracket_order(
        response: &Value,
        symbol: &str,
        price_scale: u8,
    ) -> ExchangeResult<BracketResponse> {
        let data = Self::extract_data(response)?;
        let entry_order = Self::parse_order_data(data, symbol, price_scale)?;

        // Try dedicated sub-objects first (present in some API versions)
        let tp_order = if let Some(tp_data) = data.get("takeProfitOrder") {
            Self::parse_order_data(tp_data, symbol, price_scale)
                .unwrap_or_else(|_| Self::synthetic_tp_from_entry(data, &entry_order, price_scale))
        } else {
            Self::synthetic_tp_from_entry(data, &entry_order, price_scale)
        };

        let sl_order = if let Some(sl_data) = data.get("stopLossOrder") {
            Self::parse_order_data(sl_data, symbol, price_scale)
                .unwrap_or_else(|_| Self::synthetic_sl_from_entry(data, &entry_order, price_scale))
        } else {
            Self::synthetic_sl_from_entry(data, &entry_order, price_scale)
        };

        Ok(BracketResponse {
            entry_order,
            tp_order,
            sl_order,
        })
    }

    /// Build a synthetic TP order from `takeProfitEp` on the entry order data.
    fn synthetic_tp_from_entry(data: &Value, entry: &Order, price_scale: u8) -> Order {
        let tp_price_ep = Self::get_i64(data, "takeProfitEp").unwrap_or(0);
        let tp_price = unscale_price(tp_price_ep, price_scale);
        Order {
            id: "tp_pending".to_string(),
            client_order_id: None,
            symbol: entry.symbol.clone(),
            side: entry.side.opposite(),
            order_type: OrderType::Limit { price: tp_price },
            status: OrderStatus::New,
            price: Some(tp_price),
            stop_price: None,
            quantity: entry.quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: entry.created_at,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        }
    }

    /// Build a synthetic SL order from `stopLossEp` on the entry order data.
    fn synthetic_sl_from_entry(data: &Value, entry: &Order, price_scale: u8) -> Order {
        let sl_price_ep = Self::get_i64(data, "stopLossEp").unwrap_or(0);
        let sl_price = unscale_price(sl_price_ep, price_scale);
        Order {
            id: "sl_pending".to_string(),
            client_order_id: None,
            symbol: entry.symbol.clone(),
            side: entry.side.opposite(),
            order_type: OrderType::StopMarket { stop_price: sl_price },
            status: OrderStatus::New,
            price: None,
            stop_price: Some(sl_price),
            quantity: entry.quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: entry.created_at,
            updated_at: None,
            time_in_force: crate::core::TimeInForce::Gtc,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse spot balances
    pub fn parse_spot_balances(response: &Value, value_scale: u8) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;
        let balances_array = data.get("balances")
            .and_then(|b| b.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected balances array".to_string()))?;

        let mut balances = Vec::new();

        for item in balances_array {
            let asset = Self::get_str(item, "currency").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let balance_ev = Self::get_i64(item, "balanceEv").unwrap_or(0);
            let locked_ev = Self::get_i64(item, "lockedTradingBalanceEv").unwrap_or(0);

            let total = unscale_value(balance_ev, value_scale);
            let locked = unscale_value(locked_ev, value_scale);

            balances.push(Balance {
                asset,
                free: total - locked,
                locked,
                total,
            });
        }

        Ok(balances)
    }

    /// Parse contract account
    pub fn parse_contract_account(response: &Value, value_scale: u8) -> ExchangeResult<Vec<Balance>> {
        let data = Self::extract_data(response)?;

        let account = data.get("account")
            .ok_or_else(|| ExchangeError::Parse("Missing account field".to_string()))?;

        let currency = Self::get_str(account, "currency").unwrap_or("BTC").to_string();
        let balance_ev = Self::get_i64(account, "accountBalanceEv").unwrap_or(0);
        let used_ev = Self::get_i64(account, "totalUsedBalanceEv").unwrap_or(0);

        let total = unscale_value(balance_ev, value_scale);
        let used = unscale_value(used_ev, value_scale);

        Ok(vec![Balance {
            asset: currency,
            free: total - used,
            locked: used,
            total,
        }])
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
    pub fn parse_positions(response: &Value, price_scale: u8, value_scale: u8) -> ExchangeResult<Vec<Position>> {
        let data = Self::extract_data(response)?;
        let positions_array = data.get("positions")
            .and_then(|p| p.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected positions array".to_string()))?;

        let mut positions = Vec::new();

        for item in positions_array {
            if let Some(pos) = Self::parse_position_data(item, price_scale, value_scale) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    fn parse_position_data(data: &Value, price_scale: u8, value_scale: u8) -> Option<Position> {
        let symbol = Self::get_str(data, "symbol")?.to_string();
        let size = Self::get_i64(data, "size").unwrap_or(0);

        if size == 0 {
            return None;
        }

        let side_str = Self::get_str(data, "side").unwrap_or("Buy");
        let side = if side_str == "Sell" {
            PositionSide::Short
        } else {
            PositionSide::Long
        };

        let entry_price_ep = Self::get_i64(data, "avgEntryPriceEp").unwrap_or(0);
        let mark_price_ep = Self::get_i64(data, "markPriceEp");
        let liq_price_ep = Self::get_i64(data, "liquidationPriceEp");
        let unrealized_pnl_ev = Self::get_i64(data, "unrealisedPnlEv").unwrap_or(0);

        // Leverage: positive = isolated, zero/negative = cross
        let leverage_er = Self::get_i64(data, "leverageEr").unwrap_or(0);
        let margin_type = if leverage_er > 0 {
            crate::core::MarginType::Isolated
        } else {
            crate::core::MarginType::Cross
        };

        Some(Position {
            symbol,
            side,
            quantity: size.abs() as f64,
            entry_price: unscale_price(entry_price_ep, price_scale),
            mark_price: mark_price_ep.map(|p| unscale_price(p, price_scale)),
            unrealized_pnl: unscale_value(unrealized_pnl_ev, value_scale),
            realized_pnl: None,
            leverage: (leverage_er.abs() as f64 / 100_000_000.0) as u32,
            liquidation_price: liq_price_ep.map(|p| unscale_price(p, price_scale)),
            margin: None,
            margin_type,
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Phemex /public/products response.
    ///
    /// Response format:
    /// ```json
    /// {"code":0,"msg":"OK","data":{"products":[{"symbol":"BTCUSD","type":"Perpetual","baseCurrency":"BTC","quoteCurrency":"USD","settlementCurrency":"BTC","maxOrderQty":1000000,"maxPriceEp":10000000000,"minOrderValue":1,"priceScale":4,"ratioScale":8,"valueScale":8,"defaultLeverage":100,"status":"Listed"},...],"perpProductsV2":[...]}}
    /// ```
    pub fn parse_exchange_info(response: &Value) -> ExchangeResult<Vec<SymbolInfo>> {
        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        let mut symbols = Vec::new();

        // Parse both products (contracts) and spot products if available
        for key in &["products", "spotProducts"] {
            if let Some(items) = data.get(key).and_then(|v| v.as_array()) {
                for item in items {
                    // Only listed products
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("Listed");
                    if status != "Listed" && status != "Trading" {
                        continue;
                    }

                    let symbol = match item.get("symbol").and_then(|v| v.as_str()) {
                        Some(s) => s.to_string(),
                        None => continue,
                    };

                    let base_asset = item.get("baseCurrency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let quote_asset = item.get("quoteCurrency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("USD")
                        .to_string();

                    if base_asset.is_empty() {
                        continue;
                    }

                    // priceScale: number of decimal places in price
                    let price_precision = item.get("priceScale")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(4) as u8;

                    // Minimum order qty
                    let min_quantity = item.get("lotSize")
                        .and_then(|v| v.as_f64())
                        .or_else(|| item.get("qtyStep").and_then(|v| v.as_f64()));

                    let step_size = item.get("qtyStep")
                        .and_then(|v| v.as_f64());

                    let min_notional = item.get("minOrderValue")
                        .and_then(|v| v.as_f64());

                    symbols.push(SymbolInfo {
                        symbol,
                        base_asset,
                        quote_asset,
                        status: "TRADING".to_string(),
                        price_precision,
                        quantity_precision: 8,
                        min_quantity,
                        max_quantity: None,
                        step_size,
                        min_notional,
                    });
                }
            }
        }

        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_server_time() {
        let response = json!({
            "error": null,
            "id": 0,
            "result": 1234567890000000i64
        });

        let time = PhemexParser::parse_server_time(&response).unwrap();
        assert_eq!(time, 1234567890);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "error": null,
            "id": 0,
            "result": {
                "book": {
                    "asks": [[87705000i64, 1000000], [87710000i64, 500000]],
                    "bids": [[87700000i64, 2000000], [87695000i64, 1000000]]
                },
                "depth": 30,
                "sequence": 123456789i64,
                "timestamp": 1234567890000000000i64,
                "symbol": "BTCUSD"
            }
        });

        let orderbook = PhemexParser::parse_orderbook(&response, 4).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].0 - 8770.0).abs() < 0.1);
        assert!((orderbook.asks[0].0 - 8770.5).abs() < 0.1);
    }

    #[test]
    fn test_unscale_price() {
        // BTCUSD: priceScale = 4
        let price = unscale_price(87700000, 4);
        assert!((price - 8770.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unscale_value() {
        // BTC: valueScale = 8
        let value = unscale_value(100000000, 8);
        assert!((value - 1.0).abs() < f64::EPSILON);
    }
}
