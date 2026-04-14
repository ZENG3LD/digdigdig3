//! Alpaca response parsers
//!
//! Parse JSON responses to domain types based on Alpaca API response formats.

use serde_json::Value;
use crate::core::types::*;

pub struct AlpacaParser;

impl AlpacaParser {
    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price from snapshot response
    ///
    /// Alpaca doesn't have a direct "get price" endpoint, so we extract it from
    /// the snapshot's latestTrade or latestQuote.
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        // Try latestTrade first
        if let Some(latest_trade) = response.get("latestTrade") {
            if let Some(price) = latest_trade.get("p").and_then(|v| v.as_f64()) {
                return Ok(price);
            }
        }

        // Fall back to latestQuote (use mid-price)
        if let Some(latest_quote) = response.get("latestQuote") {
            if let Some(ask) = latest_quote.get("ap").and_then(|v| v.as_f64()) {
                if let Some(bid) = latest_quote.get("bp").and_then(|v| v.as_f64()) {
                    return Ok((ask + bid) / 2.0);
                }
                return Ok(ask);
            }
        }

        Err(ExchangeError::Parse("No price data found in snapshot".to_string()))
    }

    /// Parse ticker from snapshot response
    ///
    /// Alpaca snapshot includes:
    /// - latestTrade: {t, x, p, s, c, i, z}
    /// - latestQuote: {t, ax, ap, as, bx, bp, bs, c, z}
    /// - minuteBar: {t, o, h, l, c, v, n, vw}
    /// - dailyBar: {t, o, h, l, c, v, n, vw}
    /// - prevDailyBar: {t, o, h, l, c, v, n, vw}
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let latest_trade = response.get("latestTrade");
        let latest_quote = response.get("latestQuote");
        let daily_bar = response.get("dailyBar");
        let prev_daily_bar = response.get("prevDailyBar");

        // Get last price from latestTrade
        let last_price = latest_trade
            .and_then(|t| t.get("p"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Missing last_price in snapshot".to_string()))?;

        // Get bid/ask from latestQuote
        let bid_price = latest_quote.and_then(|q| q.get("bp")).and_then(|v| v.as_f64());
        let ask_price = latest_quote.and_then(|q| q.get("ap")).and_then(|v| v.as_f64());

        // Get 24h stats from dailyBar and prevDailyBar
        let high_24h = daily_bar.and_then(|b| b.get("h")).and_then(|v| v.as_f64());
        let low_24h = daily_bar.and_then(|b| b.get("l")).and_then(|v| v.as_f64());
        let volume_24h = daily_bar.and_then(|b| b.get("v")).and_then(|v| v.as_f64());

        // Calculate price change
        let prev_close = prev_daily_bar.and_then(|b| b.get("c")).and_then(|v| v.as_f64());
        let (price_change_24h, price_change_percent_24h) = if let Some(prev) = prev_close {
            let change = last_price - prev;
            let change_pct = (change / prev) * 100.0;
            (Some(change), Some(change_pct))
        } else {
            (None, None)
        };

        // Get timestamp (use latest trade timestamp or current time)
        let timestamp = latest_trade
            .and_then(|t| t.get("t"))
            .and_then(Self::parse_timestamp)
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None, // Alpaca doesn't provide quote volume directly
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse klines/bars from bars response
    ///
    /// Alpaca bars format:
    /// ```json
    /// {
    ///   "bars": {
    ///     "AAPL": [
    ///       {"t": "2024-01-18T14:30:00Z", "o": 150.00, "h": 150.50, "l": 149.80, "c": 150.25, "v": 125000, "n": 1500, "vw": 150.12},
    ///       ...
    ///     ]
    ///   },
    ///   "next_page_token": "..."
    /// }
    /// ```
    pub fn parse_klines(response: &Value, symbol: &str) -> ExchangeResult<Vec<Kline>> {
        let bars_obj = response
            .get("bars")
            .ok_or_else(|| ExchangeError::Parse("Missing 'bars' field".to_string()))?;

        let bars_array = bars_obj
            .get(symbol)
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse(format!("No bars found for symbol {}", symbol)))?;

        bars_array
            .iter()
            .map(|bar| {
                let open_time = bar
                    .get("t")
                    .and_then(Self::parse_timestamp)
                    .ok_or_else(|| ExchangeError::Parse("Missing timestamp".to_string()))?;

                let open = Self::require_f64(bar, "o")?;
                let high = Self::require_f64(bar, "h")?;
                let low = Self::require_f64(bar, "l")?;
                let close = Self::require_f64(bar, "c")?;
                let volume = Self::require_f64(bar, "v")?;

                let trades = bar.get("n").and_then(|v| v.as_u64());

                Ok(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume: None, // Alpaca has vw (VWAP) but not quote volume
                    close_time: None,
                    trades,
                })
            })
            .collect()
    }

    /// Parse orderbook response (crypto only)
    ///
    /// Alpaca only provides orderbook for crypto via `/v1beta3/crypto/us/latest/orderbooks`
    pub fn parse_orderbook(response: &Value, symbol: &str) -> ExchangeResult<OrderBook> {
        let orderbooks = response
            .get("orderbooks")
            .ok_or_else(|| ExchangeError::Parse("Missing 'orderbooks' field".to_string()))?;

        let book = orderbooks
            .get(symbol)
            .ok_or_else(|| ExchangeError::Parse(format!("No orderbook for {}", symbol)))?;

        let timestamp = book
            .get("t")
            .and_then(Self::parse_timestamp)
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        // Parse bids array
        let bids = Self::parse_order_levels(book.get("b"))?;

        // Parse asks array
        let asks = Self::parse_order_levels(book.get("a"))?;

        Ok(OrderBook {
            bids,
            asks,
            timestamp,
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    /// Parse symbols list from assets response
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of assets".to_string()))?;

        Ok(array
            .iter()
            .filter_map(|v| {
                // Only include tradable assets
                let tradable = v.get("tradable").and_then(|t| t.as_bool()).unwrap_or(false);
                if tradable {
                    v.get("symbol").and_then(|s| s.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse account info
    pub fn parse_account_info(response: &Value, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Ok(AccountInfo {
            account_type,
            maker_commission: 0.0, // Alpaca is commission-free
            taker_commission: 0.0,
            can_trade: !Self::get_bool(response, "trading_blocked").unwrap_or(false),
            can_withdraw: !Self::get_bool(response, "transfers_blocked").unwrap_or(false),
            can_deposit: !Self::get_bool(response, "transfers_blocked").unwrap_or(false),
            balances: vec![], // We'll populate this separately via get_balance
        })
    }

    /// Parse balance from account response
    pub fn parse_balance(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let mut balances = Vec::new();

        // Main USD balance
        if let Ok(cash) = Self::get_str_as_f64(response, "cash") {
            balances.push(Balance {
                asset: "USD".to_string(),
                free: cash,
                locked: 0.0, // Alpaca doesn't separate locked funds clearly
                total: cash,
            });
        }

        // Note: For actual positions, need to query positions endpoint separately

        Ok(balances)
    }

    /// Parse order result
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        let id = Self::get_str(response, "id")
            .ok_or_else(|| ExchangeError::Parse("Missing order id".to_string()))?
            .to_string();

        let client_order_id = Self::get_str(response, "client_order_id").map(|s| s.to_string());

        let symbol = Self::get_str(response, "symbol")
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?
            .to_string();

        let side = match Self::get_str(response, "side") {
            Some("buy") => OrderSide::Buy,
            Some("sell") => OrderSide::Sell,
            _ => return Err(ExchangeError::Parse("Invalid order side".to_string())),
        };

        let order_type = match Self::get_str(response, "type").or_else(|| Self::get_str(response, "order_type")) {
            Some("market") => OrderType::Market,
            Some("limit") => OrderType::Limit { price: 0.0 },
            Some("stop") => OrderType::StopMarket { stop_price: 0.0 },
            Some("stop_limit") => OrderType::StopLimit { stop_price: 0.0, limit_price: 0.0 },
            Some("trailing_stop") => OrderType::StopMarket { stop_price: 0.0 }, // Map trailing stop to regular stop
            _ => OrderType::Market, // Default fallback
        };

        let status = Self::parse_order_status(response)?;

        let price = Self::get_str(response, "limit_price").and_then(|s| s.parse().ok());
        let stop_price = Self::get_str(response, "stop_price").and_then(|s| s.parse().ok());

        let quantity = Self::get_str_as_f64(response, "qty")?;
        let filled_quantity = Self::get_str_as_f64(response, "filled_qty").unwrap_or(0.0);

        let average_price = Self::get_str(response, "filled_avg_price").and_then(|s| s.parse().ok());

        let created_at = Self::get_str(response, "created_at")
            .and_then(|s| Self::parse_timestamp(&Value::String(s.to_string())))
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        let updated_at = Self::get_str(response, "updated_at")
            .and_then(|s| Self::parse_timestamp(&Value::String(s.to_string())));

        let time_in_force = match Self::get_str(response, "time_in_force") {
            Some("gtc") => TimeInForce::Gtc,
            Some("ioc") => TimeInForce::Ioc,
            Some("fok") => TimeInForce::Fok,
            Some("day") => TimeInForce::Gtc, // Map day to GTC
            _ => TimeInForce::Gtc,
        };

        Ok(Order {
            id,
            client_order_id,
            symbol,
            side,
            order_type,
            status,
            price,
            stop_price,
            quantity,
            filled_quantity,
            average_price,
            commission: None, // Alpaca doesn't charge commission
            commission_asset: None,
            created_at,
            updated_at,
            time_in_force,
        })
    }

    /// Parse order status from Alpaca status string
    fn parse_order_status(response: &Value) -> ExchangeResult<OrderStatus> {
        match Self::get_str(response, "status") {
            Some("new") | Some("accepted") | Some("pending_new") => Ok(OrderStatus::New),
            Some("partially_filled") => Ok(OrderStatus::PartiallyFilled),
            Some("filled") => Ok(OrderStatus::Filled),
            Some("canceled") | Some("done_for_day") => Ok(OrderStatus::Canceled),
            Some("rejected") | Some("suspended") => Ok(OrderStatus::Rejected),
            Some("expired") => Ok(OrderStatus::Expired),
            Some(other) => {
                // Unknown status, log and default to Open
                eprintln!("Unknown Alpaca order status: {}", other);
                Ok(OrderStatus::Open)
            }
            None => Err(ExchangeError::Parse("Missing order status".to_string())),
        }
    }

    /// Parse orders list
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of orders".to_string()))?;

        array.iter().map(Self::parse_order).collect()
    }

    /// Parse position
    pub fn parse_position(response: &Value) -> ExchangeResult<Position> {
        let symbol = Self::get_str(response, "symbol")
            .ok_or_else(|| ExchangeError::Parse("Missing symbol".to_string()))?
            .to_string();

        let qty = Self::get_str_as_f64(response, "qty")?;

        let side = if qty >= 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        let quantity = qty.abs();

        let entry_price = Self::get_str_as_f64(response, "avg_entry_price")?;
        let mark_price = Self::get_str_as_f64(response, "current_price").ok();

        let unrealized_pnl = Self::get_str_as_f64(response, "unrealized_pl").unwrap_or(0.0);
        let realized_pnl = None; // Alpaca doesn't provide realized PnL in position

        Ok(Position {
            symbol,
            side,
            quantity,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl,
            liquidation_price: None,
            leverage: 1, // Alpaca stocks don't use leverage
            margin_type: MarginType::Cross, // Default margin type
            margin: None,
            take_profit: None,
            stop_loss: None,
        })
    }

    /// Parse positions list
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of positions".to_string()))?;

        array.iter().map(Self::parse_position).collect()
    }

    /// Parse cancel-all response (HTTP 207 Multi-Status from DELETE /v2/orders)
    ///
    /// Alpaca returns an array of per-order objects, each with:
    /// `{ "id": "...", "status": 200 }` on success or
    /// `{ "id": "...", "status": 422, "body": { "code": ..., "message": "..." } }` on failure.
    pub fn parse_cancel_all(response: &Value) -> ExchangeResult<CancelAllResponse> {
        // Empty response = no open orders, all "cancelled" (count 0)
        if response.is_null() {
            return Ok(CancelAllResponse {
                cancelled_count: 0,
                failed_count: 0,
                details: vec![],
            });
        }

        let items = match response.as_array() {
            Some(arr) => arr,
            None => {
                // Some Alpaca responses return empty body or a non-array on full success
                return Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                });
            }
        };

        let mut cancelled_count = 0u32;
        let mut failed_count = 0u32;
        let mut details = Vec::new();

        for item in items {
            let status_code = item.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
            let order_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let success = (200..300).contains(&status_code);

            if success {
                cancelled_count += 1;
            } else {
                failed_count += 1;
            }

            let error_msg = if !success {
                item.get("body")
                    .and_then(|b| b.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| Some(format!("HTTP {}", status_code)))
            } else {
                None
            };

            details.push(OrderResult {
                order: None, // Individual order details not returned in 207 response
                client_order_id: Some(order_id),
                success,
                error: error_msg,
                error_code: if !success { Some(status_code as i32) } else { None },
            });
        }

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count,
            details,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn _require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    /// Parse string field as f64 (Alpaca often returns numbers as strings)
    fn get_str_as_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing field '{}'", field)))?
            .as_str()
            .ok_or_else(|| ExchangeError::Parse(format!("Field '{}' is not a string", field)))?
            .parse()
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse '{}': {}", field, e)))
    }

    /// Parse order book levels
    ///
    /// Alpaca orderbook format: `[{"p": 45000.00, "s": 1.5}, ...]`
    fn parse_order_levels(value: Option<&Value>) -> ExchangeResult<Vec<OrderBookLevel>> {
        let array = value
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid order levels".to_string()))?;

        array
            .iter()
            .map(|level| {
                let price = level
                    .get("p")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ExchangeError::Parse("Invalid price in level".to_string()))?;

                let size = level
                    .get("s")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ExchangeError::Parse("Invalid size in level".to_string()))?;

                Ok(OrderBookLevel::new(price, size))
            })
            .collect()
    }

    /// Parse timestamp (RFC-3339 format or Unix timestamp)
    fn parse_timestamp(value: &Value) -> Option<i64> {
        // Try as string (RFC-3339 format)
        if let Some(s) = value.as_str() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(dt.timestamp_millis());
            }
        }

        // Try as integer (Unix timestamp in milliseconds)
        if let Some(i) = value.as_i64() {
            return Some(i);
        }

        None
    }

    // ═══════════════════════════════════════════════════════════════════════
    // USER TRADES (Account Activities — FILL type)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse account activities of type FILL into `UserTrade` records.
    ///
    /// Alpaca endpoint: `GET /v2/account/activities/FILL`
    ///
    /// Response is a JSON array:
    /// ```json
    /// [
    ///   {"id":"123","order_id":"456","symbol":"AAPL","side":"buy",
    ///    "price":"150.00","qty":"10","commission":"0",
    ///    "transaction_time":"2024-01-01T00:00:00Z","liquidity":"M"}
    /// ]
    /// ```
    ///
    /// `liquidity`: "M" = maker (limit order), "T" = taker (market order).
    pub fn parse_activities(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        let arr = response.as_array()
            .ok_or_else(|| ExchangeError::Parse(
                "Account activities: expected a JSON array".to_string()
            ))?;

        arr.iter()
            .map(Self::parse_activity_item)
            .collect()
    }

    /// Parse a single FILL activity item into a `UserTrade`.
    fn parse_activity_item(data: &Value) -> ExchangeResult<UserTrade> {
        let get_str = |key: &str| -> Option<&str> {
            data.get(key).and_then(|v| v.as_str())
        };
        let get_f64 = |key: &str| -> Option<f64> {
            data.get(key).and_then(|v| {
                v.as_str().and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
        };

        let id = get_str("id").unwrap_or("").to_string();
        let order_id = get_str("order_id").unwrap_or("").to_string();
        let symbol = get_str("symbol").unwrap_or("").to_string();

        let side_str = get_str("side").unwrap_or("buy");
        let side = match side_str.to_lowercase().as_str() {
            "sell" | "sell_short" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let price = get_f64("price").unwrap_or(0.0);

        // "qty" is base-asset quantity; "leaves_qty" is remainder (ignore)
        let quantity = get_f64("qty").unwrap_or(0.0);

        // Commission — Alpaca is commission-free for US stocks; may be 0
        let commission = get_f64("commission").unwrap_or(0.0);
        let commission_asset = "USD".to_string();

        // "M" = maker (limit), "T" = taker (market)
        let liquidity = get_str("liquidity").unwrap_or("T");
        let is_maker = liquidity.eq_ignore_ascii_case("M");

        let timestamp = data.get("transaction_time")
            .and_then(Self::parse_timestamp)
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
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse `GET /v2/account/activities` response into `LedgerEntry` items.
    ///
    /// Alpaca returns a flat JSON array. Each element's `activity_type` field
    /// drives the `LedgerEntryType` mapping.
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        let arr = response.as_array().ok_or_else(|| {
            ExchangeError::Parse(
                "Expected a JSON array for /v2/account/activities".to_string(),
            )
        })?;

        let mut entries = Vec::with_capacity(arr.len());

        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let activity_type = item
                .get("activity_type")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");

            let (entry_type, asset, amount, description) =
                Self::map_activity(item, activity_type);

            let timestamp = item
                .get("transaction_time")
                .and_then(Self::parse_timestamp)
                .or_else(|| item.get("date").and_then(Self::parse_timestamp))
                .unwrap_or(0);

            let ref_id = item
                .get("order_id")
                .and_then(|v| v.as_str())
                .map(String::from);

            entries.push(LedgerEntry {
                id,
                asset,
                amount,
                balance: None,
                entry_type,
                description,
                ref_id,
                timestamp,
            });
        }

        Ok(entries)
    }

    /// Map a single Alpaca activity JSON object to ledger fields.
    ///
    /// Returns `(entry_type, asset, amount, description)`.
    fn map_activity(
        item: &Value,
        activity_type: &str,
    ) -> (LedgerEntryType, String, f64, String) {
        let get_str = |key: &str| -> Option<&str> { item.get(key).and_then(|v| v.as_str()) };
        let get_f64 = |key: &str| -> Option<f64> {
            item.get(key).and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
        };

        match activity_type {
            "FILL" => {
                let symbol = get_str("symbol").unwrap_or("").to_string();
                let qty = get_f64("qty").unwrap_or(0.0);
                let price = get_f64("price").unwrap_or(0.0);
                let side = get_str("side").unwrap_or("buy");
                // Positive = received asset (buy), negative = gave asset (sell)
                let signed_qty = if side.eq_ignore_ascii_case("sell")
                    || side.eq_ignore_ascii_case("sell_short")
                {
                    -qty
                } else {
                    qty
                };
                let desc = format!("{} {} @ {}", side.to_uppercase(), symbol, price);
                (LedgerEntryType::Trade, symbol, signed_qty, desc)
            }
            "CSD" | "CSW_COMPLETE" => {
                let net = get_f64("net_amount")
                    .or_else(|| get_f64("amount"))
                    .unwrap_or(0.0);
                (
                    LedgerEntryType::Deposit,
                    "USD".to_string(),
                    net.abs(),
                    "Cash deposit".to_string(),
                )
            }
            "CSW" => {
                let net = get_f64("net_amount")
                    .or_else(|| get_f64("amount"))
                    .unwrap_or(0.0);
                (
                    LedgerEntryType::Withdrawal,
                    "USD".to_string(),
                    -net.abs(),
                    "Cash withdrawal".to_string(),
                )
            }
            "FEE" => {
                let net = get_f64("net_amount")
                    .or_else(|| get_f64("amount"))
                    .unwrap_or(0.0);
                (
                    LedgerEntryType::Fee,
                    "USD".to_string(),
                    -net.abs(),
                    "Fee".to_string(),
                )
            }
            "ACATC" | "ACATS" | "JNLC" | "JNLS" => {
                let net = get_f64("net_amount")
                    .or_else(|| get_f64("amount"))
                    .unwrap_or(0.0);
                let symbol = get_str("symbol").unwrap_or("USD").to_string();
                let desc = format!("{} transfer", activity_type);
                (LedgerEntryType::Transfer, symbol, net, desc)
            }
            other => {
                let net = get_f64("net_amount")
                    .or_else(|| get_f64("amount"))
                    .unwrap_or(0.0);
                let symbol = get_str("symbol").unwrap_or("USD").to_string();
                let desc = format!("{} activity", other);
                (
                    LedgerEntryType::Other(other.to_string()),
                    symbol,
                    net,
                    desc,
                )
            }
        }
    }
}
