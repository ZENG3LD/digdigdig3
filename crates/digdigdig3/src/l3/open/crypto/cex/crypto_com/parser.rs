//! # Crypto.com Response Parser
//!
//! JSON parsing for Crypto.com Exchange API v1 responses.
//!
//! ## Important Notes
//! - All numeric values in Crypto.com responses are STRINGS (e.g., "50000.00")
//! - Response format: { "code": 0, "result": { ... } }
//! - Success: code = 0, errors: code != 0
//! - REST and WebSocket use different formats for some messages

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult, AccountType,
    Kline, OrderBook, OrderBookLevel, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, TradeSide, SymbolInfo,
    UserTrade,
    LedgerEntry, LedgerEntryType,
};

/// Parser for Crypto.com API responses
pub struct CryptoComParser;

impl CryptoComParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract result from response
    pub fn extract_result(response: &Value) -> ExchangeResult<&Value> {
        response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))
    }

    /// Check response code (0 = success)
    pub fn check_response(response: &Value) -> ExchangeResult<()> {
        let code = response.get("code")
            .and_then(|c| c.as_i64())
            .unwrap_or(0);

        if code != 0 {
            let message = response.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: code as i32,
                message: message.to_string(),
            });
        }

        Ok(())
    }

    /// Parse f64 from string or number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Get f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Get required f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Get string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Get required string
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Get i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
            .or_else(|| data.get(key).and_then(|v| v.as_i64()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price (ticker response)
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No ticker data".to_string()))?;

        Self::get_f64(data, "a") // "a" = last price
            .ok_or_else(|| ExchangeError::Parse("Missing last price".to_string()))
    }

    /// Parse klines
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of candlesticks".to_string()))?;

        let mut klines = Vec::with_capacity(data.len());

        for candle in data {
            let open_time = Self::get_i64(candle, "t").unwrap_or(0);
            let open = Self::get_f64(candle, "o").unwrap_or(0.0);
            let high = Self::get_f64(candle, "h").unwrap_or(0.0);
            let low = Self::get_f64(candle, "l").unwrap_or(0.0);
            let close = Self::get_f64(candle, "c").unwrap_or(0.0);
            let volume = Self::get_f64(candle, "v").unwrap_or(0.0);

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
                ..Default::default()
            });
        }

        Ok(klines)
    }

    /// Parse orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No orderbook data".to_string()))?;

        let parse_levels = |key: &str| -> Vec<OrderBookLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|level| {
                            let pair = level.as_array()?;
                            if pair.len() < 2 { return None; }
                            let price = Self::parse_f64(&pair[0])?;
                            let size = Self::parse_f64(&pair[1])?;
                            Some(OrderBookLevel::new(price, size))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let timestamp = Self::get_i64(data, "t").unwrap_or(0);

        Ok(OrderBook {
            timestamp,
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
            ..Default::default()
        })
    }

    /// Parse ticker
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No ticker data".to_string()))?;

        Ok(Ticker {
            last_price: Self::get_f64(data, "a").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "b"),
            ask_price: Self::get_f64(data, "k"),
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: Self::get_f64(data, "vv"),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "c").map(|r| r * 100.0),
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
            open_interest: Self::get_f64(data, "oi"),
            ..Default::default()
        })
    }

    /// Parse funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No valuation data".to_string()))?;

        Ok(FundingRate {
            rate: Self::require_f64(data, "funding_rate")?,
            next_funding_time: Self::get_i64(data, "next_funding_time"),
            timestamp: 0, ..Default::default() 
        })
    }

    /// Parse insurance fund balance from `public/get-insurance` response.
    ///
    /// Returns `(instrument_type, balance_usd)` from the first data entry.
    pub fn parse_insurance(response: &Value) -> ExchangeResult<(String, f64)> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No insurance data".to_string()))?;

        let instrument_type = Self::get_str(data, "instrument_type")
            .unwrap_or("")
            .to_string();
        let balance = Self::require_f64(data, "balance")?;
        Ok((instrument_type, balance))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse order from create order response
    pub fn parse_order_id(response: &Value) -> ExchangeResult<String> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        Self::require_str(result, "order_id").map(String::from)
    }

    /// Parse order details
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        Self::parse_order_data(result)
    }

    /// Parse order from data object
    pub fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let side = match Self::get_str(data, "side").unwrap_or("BUY") {
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match Self::get_str(data, "type").unwrap_or("LIMIT") {
            "MARKET" => OrderType::Market,
            _ => OrderType::Limit { price: 0.0 },
        };

        let status = Self::parse_order_status(data);

        Ok(Order {
            id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "client_oid").map(String::from),
            symbol: Self::get_str(data, "instrument_name").map(String::from),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: Self::get_f64(data, "trigger_price"),
            quantity: Self::get_f64(data, "quantity").unwrap_or(0.0),
            filled_quantity: Self::get_f64(data, "cumulative_quantity").unwrap_or(0.0),
            average_price: Self::get_f64(data, "avg_price"),
            commission: None,
            commission_asset: Self::get_str(data, "fee_currency").map(String::from),
            created_at: Self::get_i64(data, "create_time").unwrap_or(0),
            updated_at: Self::get_i64(data, "update_time"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Parse order status
    fn parse_order_status(data: &Value) -> OrderStatus {
        match Self::get_str(data, "status").unwrap_or("ACTIVE") {
            "ACTIVE" => OrderStatus::New,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Expired,
            "PENDING" => OrderStatus::New,
            _ => OrderStatus::New,
        }
    }

    /// Parse list of orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let order_list = result.get("order_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected order_list array".to_string()))?;

        order_list.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Parse user trades (fills) from `private/get-trades` response.
    ///
    /// Response format:
    /// ```json
    /// {"result":{"data":[{"trade_id":"123","order_id":"456","instrument_name":"BTC_USDT",
    ///   "side":"BUY","price":"50000","quantity":"0.001","fee":"0.01",
    ///   "fee_currency":"USDT","liquidity_indicator":"MAKER","create_time":1672531200000}]}}
    /// ```
    pub fn parse_user_trades(response: &Value) -> ExchangeResult<Vec<UserTrade>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;

        let data = result.get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected 'data' array in get-trades response".to_string()))?;

        let mut trades = Vec::with_capacity(data.len());

        for item in data {
            let side = match Self::get_str(item, "side").unwrap_or("BUY") {
                "SELL" => OrderSide::Sell,
                _ => OrderSide::Buy,
            };

            let is_maker = matches!(
                Self::get_str(item, "liquidity_indicator"),
                Some("MAKER")
            );

            trades.push(UserTrade {
                id: Self::get_str(item, "trade_id").unwrap_or("").to_string(),
                order_id: Self::get_str(item, "order_id").unwrap_or("").to_string(),
                symbol: Self::get_str(item, "instrument_name").unwrap_or("").to_string(),
                side,
                price: Self::get_f64(item, "price").unwrap_or(0.0),
                quantity: Self::get_f64(item, "quantity").unwrap_or(0.0),
                commission: Self::get_f64(item, "fee").unwrap_or(0.0),
                commission_asset: Self::get_str(item, "fee_currency").unwrap_or("").to_string(),
                is_maker,
                timestamp: Self::get_i64(item, "create_time").unwrap_or(0),
            });
        }

        Ok(trades)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse balances from user-balance response
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let instruments = result.get("instrument_collateral_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected instrument_collateral_list".to_string()))?;

        let mut balances = Vec::new();

        for item in instruments {
            let asset = Self::get_str(item, "instrument_name").unwrap_or("").to_string();
            if asset.is_empty() { continue; }

            let free = Self::get_f64(item, "quantity").unwrap_or(0.0);
            let locked = Self::get_f64(item, "reserved_qty").unwrap_or(0.0);

            balances.push(Balance {
                asset,
                free,
                locked,
                total: free + locked,
            });
        }

        Ok(balances)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POSITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected positions array".to_string()))?;

        let mut positions = Vec::new();

        for item in data {
            if let Some(pos) = Self::parse_position_data(item) {
                positions.push(pos);
            }
        }

        Ok(positions)
    }

    /// Parse single position
    fn parse_position_data(data: &Value) -> Option<Position> {
        let symbol = Self::get_str(data, "instrument_name")?.to_string();
        let quantity = Self::get_f64(data, "quantity").unwrap_or(0.0);

        // Skip empty positions
        if quantity.abs() < f64::EPSILON {
            return None;
        }

        let side = if quantity > 0.0 {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        Some(Position {
            symbol,
            side,
            quantity: quantity.abs(),
            entry_price: Self::get_f64(data, "entry_price").unwrap_or(0.0),
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl: Self::get_f64(data, "open_position_pnl").unwrap_or(0.0),
            realized_pnl: Self::get_f64(data, "session_pnl"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            liquidation_price: None,
            margin: Self::get_f64(data, "initial_margin"),
            margin_type: if Self::get_str(data, "type") == Some("ISOLATED") {
                crate::core::MarginType::Isolated
            } else {
                crate::core::MarginType::Cross
            },
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket ticker message
    pub fn parse_ws_ticker(data: &Value) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            last_price: Self::get_f64(data, "a").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "b"),
            ask_price: Self::get_f64(data, "k"),
            high_24h: Self::get_f64(data, "h"),
            low_24h: Self::get_f64(data, "l"),
            volume_24h: Self::get_f64(data, "v"),
            quote_volume_24h: Self::get_f64(data, "vv"),
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "c").map(|r| r * 100.0),
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
            open_interest: Self::get_f64(data, "oi"),
            ..Default::default()
        })
    }

    /// Parse `public/get-trades` REST response into a list of public trades.
    ///
    /// Response shape:
    /// ```json
    /// {"id":-1,"method":"public/get-trades","code":0,
    ///  "result":{"data":[{"d":"1781450491821274859","t":1749000000000,
    ///    "q":"0.04522","p":"64072.10","s":"sell","i":"BTC_USDT","m":"..."}]}}
    /// ```
    /// Fields: `d`=tradeId, `t`=timestamp(ms), `p`=price(string),
    /// `q`=quantity(string), `s`=side("buy"/"sell" taker side).
    pub fn parse_recent_trades(response: &Value) -> ExchangeResult<Vec<PublicTrade>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'result.data' in get-trades response".to_string()))?;

        let mut trades = Vec::with_capacity(data.len());
        for item in data {
            let side = match Self::get_str(item, "s").unwrap_or("buy") {
                "sell" => TradeSide::Sell,
                _ => TradeSide::Buy,
            };
            trades.push(PublicTrade {
                id: Self::get_str(item, "d").unwrap_or("").to_string(),
                price: Self::require_f64(item, "p")?,
                quantity: Self::get_f64(item, "q").unwrap_or(0.0),
                side,
                timestamp: Self::get_i64(item, "t").unwrap_or(0),
                seq: Self::get_i64(item, "d"),
                match_id: Self::get_str(item, "m").map(String::from),
                ..Default::default()
            });
        }
        Ok(trades)
    }

    /// Parse WebSocket trade message
    pub fn parse_ws_trade(data: &Value) -> ExchangeResult<PublicTrade> {
        let side = match Self::get_str(data, "s").unwrap_or("BUY") {
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        Ok(PublicTrade {
            id: Self::get_str(data, "d").unwrap_or("").to_string(),
            price: Self::require_f64(data, "p")?,
            quantity: Self::get_f64(data, "q").unwrap_or(0.0),
            side,
            timestamp: Self::get_i64(data, "t").unwrap_or(0),
            seq: Self::get_i64(data, "d"),
            match_id: Self::get_str(data, "m").map(String::from),
            ..Default::default()
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXCHANGE INFO
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange info from Crypto.com get-instruments response.
    ///
    /// Response format:
    /// ```json
    /// {"code":0,"result":{"data":[{"symbol":"BTC_USDT","inst_type":"CCY_PAIR","display_name":"BTC/USDT","base_ccy":"BTC","quote_ccy":"USDT","quote_decimals":2,"quantity_decimals":4,"price_tick_size":"0.01","qty_tick_size":"0.0001","max_leverage":"50","tradable":true,"expiry_timestamp_ms":0,"put_call":"NONE","strike_price":"0","underlying_symbol":""},...]}}
    /// ```
    /// Parse fee rate from private/get-fee-rate or private/get-instrument-fee-rate response
    pub fn parse_fee_rate(response: &Value) -> ExchangeResult<crate::core::FeeInfo> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;

        // get-fee-rate: result.maker_rate, result.taker_rate (strings, already in decimal form e.g. "0.001")
        // get-instrument-fee-rate: result.maker_rate, result.taker_rate per instrument
        let maker = result.get("maker_rate")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.001); // 0.1% default maker

        let taker = result.get("taker_rate")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.00075); // 0.075% default taker

        let symbol = result.get("instrument_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(crate::core::FeeInfo {
            maker_rate: maker,
            taker_rate: taker,
            symbol,
            tier: None,
        })
    }

    pub fn parse_exchange_info(response: &Value, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' field".to_string()))?;

        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in result".to_string()))?;

        let mut symbols = Vec::with_capacity(data.len());

        for item in data {
            // RAW: no tradable filter — return every instrument the exchange lists.
            // Station owns normalization / filtering.

            let symbol = match item.get("symbol").and_then(|v| v.as_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let base_asset = item.get("base_ccy")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = item.get("quote_ccy")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if base_asset.is_empty() || quote_asset.is_empty() {
                continue;
            }

            // RAW: Crypto.com get-instruments has no dedicated status field.
            // The `tradable` bool is the closest proxy; carry it in `extra`.
            // Use empty string rather than faking "TRADING".
            let status = String::new();

            let price_precision = item.get("quote_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u8;

            let quantity_precision = item.get("quantity_decimals")
                .and_then(|v| v.as_u64())
                .unwrap_or(4) as u8;

            let step_size = item.get("qty_tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let min_quantity = step_size; // Minimum tradeable is typically 1 step

            let tick_size = item.get("price_tick_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            // RAW native instrument type: "CCY_PAIR", "PERPETUAL_SWAP", "FUTURE", etc.
            let instrument_type = item.get("inst_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status,
                price_precision,
                quantity_precision,
                min_quantity,
                max_quantity: None,
                tick_size,
                step_size,
                min_notional: None,
                account_type,
                instrument_type,
                // RAW passthrough — full native instrument record.
                extra: item.clone(),
            });
        }

        Ok(symbols)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT LEDGER
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse `private/get-transactions` response into ledger entries.
    ///
    /// Response shape:
    /// `{ "code": 0, "result": { "data": [ { "journal_id": "...",
    ///   "journal_type": "TRADING", "instrument_name": "BTC_USDT",
    ///   "event_type": "trade", "amount": "0.001", "fee": "0.01",
    ///   "currency": "BTC", "create_time": 1672531200000 } ] } }`
    pub fn parse_ledger(response: &Value) -> ExchangeResult<Vec<LedgerEntry>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;

        let data = result
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        let mut entries = Vec::with_capacity(data.len());

        for item in &data {
            let id = item
                .get("journal_id")
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                })
                .unwrap_or_default();

            let asset = item
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let amount = Self::get_f64(item, "amount").unwrap_or(0.0);
            let fee = Self::get_f64(item, "fee").unwrap_or(0.0);

            let journal_type = item
                .get("journal_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let event_type = item
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let (entry_type, net_amount, description) =
                Self::classify_ledger_entry(journal_type, event_type, amount, fee);

            let timestamp = item
                .get("create_time")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let instrument = item
                .get("instrument_name")
                .and_then(|v| v.as_str())
                .map(String::from);

            let desc = if description.is_empty() {
                instrument
                    .clone()
                    .unwrap_or_else(|| journal_type.to_string())
            } else {
                description
            };

            entries.push(LedgerEntry {
                id,
                asset,
                amount: net_amount,
                balance: None,
                entry_type,
                description: desc,
                ref_id: instrument,
                timestamp,
            });
        }

        Ok(entries)
    }

    /// Map Crypto.com `journal_type` + `event_type` to `LedgerEntryType` and compute net amount.
    ///
    /// Returns `(entry_type, signed_amount, description)`.
    fn classify_ledger_entry(
        journal_type: &str,
        event_type: &str,
        amount: f64,
        fee: f64,
    ) -> (LedgerEntryType, f64, String) {
        match journal_type {
            "TRADING" => {
                let desc = format!("Trade ({})", event_type);
                (LedgerEntryType::Trade, amount, desc)
            }
            "FUNDING" => (
                LedgerEntryType::Funding,
                amount,
                "Funding payment".to_string(),
            ),
            "FEE_AND_REBATE" => {
                if amount >= 0.0 {
                    (LedgerEntryType::Rebate, amount, "Fee rebate".to_string())
                } else {
                    (LedgerEntryType::Fee, amount, "Trading fee".to_string())
                }
            }
            "WITHDRAW" => {
                let net = if amount > 0.0 { -amount } else { amount };
                let net = net - fee.abs();
                (LedgerEntryType::Withdrawal, net, "Withdrawal".to_string())
            }
            "DEPOSIT" => {
                let net = if amount < 0.0 { -amount } else { amount };
                (LedgerEntryType::Deposit, net, "Deposit".to_string())
            }
            "TRANSFER" => (
                LedgerEntryType::Transfer,
                amount,
                "Internal transfer".to_string(),
            ),
            "LIQUIDATION" => (
                LedgerEntryType::Liquidation,
                amount,
                "Liquidation".to_string(),
            ),
            "SETTLEMENT" => (
                LedgerEntryType::Settlement,
                amount,
                "Settlement".to_string(),
            ),
            other => {
                let desc = format!("{} ({})", other, event_type);
                (LedgerEntryType::Other(other.to_string()), amount, desc)
            }
        }
    }
    // ═══════════════════════════════════════════════════════════════════════════
    // VALUATIONS PARSERS (mark price klines, index price klines, funding history)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Map a canonical interval string to milliseconds.
    ///
    /// Supported: `"1m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"2h"`, `"4h"`,
    /// `"6h"`, `"12h"`, `"1d"`.
    ///
    /// Returns `None` for unrecognised strings (caller should fall back to
    /// raw tick output or return an error).
    pub fn interval_to_ms(interval: &str) -> Option<i64> {
        match interval {
            "1m"  => Some(60_000),
            "5m"  => Some(300_000),
            "15m" => Some(900_000),
            "30m" => Some(1_800_000),
            "1h"  => Some(3_600_000),
            "2h"  => Some(7_200_000),
            "4h"  => Some(14_400_000),
            "6h"  => Some(21_600_000),
            "12h" => Some(43_200_000),
            "1d"  => Some(86_400_000),
            _ => None,
        }
    }

    /// Parse `public/get-valuations` response as a `Vec<Kline>`.
    ///
    /// Used for `valuation_type=mark_price` and `valuation_type=index_price`.
    /// Each data point is `{"v": "<price>", "t": <unix_ms>}` — no OHLC spread,
    /// so open/high/low/close are all set to the single value `v`.
    ///
    /// Response structure:
    /// ```json
    /// {
    ///   "code": 0,
    ///   "result": {
    ///     "data": [{"v": "45000.00", "t": 1700000000000}, ...]
    ///   }
    /// }
    /// ```
    pub fn parse_valuations_as_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Crypto.com valuations: missing 'result.data'".into()))?;

        let mut klines = Vec::with_capacity(data.len());
        for item in data {
            let ts = Self::get_i64(item, "t").unwrap_or(0);
            let price = Self::get_f64(item, "v").unwrap_or(0.0);
            klines.push(Kline {
                open_time: ts,
                open:   price,
                high:   price,
                low:    price,
                close:  price,
                volume: 0.0,
                quote_volume: None,
                close_time:   None,
                trades:       None,
                ..Default::default()
            });
        }
        Ok(klines)
    }

    /// Parse `public/get-valuations` response as OHLC klines bucketed to `interval_ms`.
    ///
    /// Crypto.com returns per-minute tick points `{"v": "<price>", "t": <unix_ms>}`.
    /// This function buckets those ticks into proper OHLC candles of the requested
    /// `interval_ms` width so that `open_time % interval_ms == 0` for every output bar.
    ///
    /// Bucketing rule:
    /// - `bar_open = floor(t / interval_ms) * interval_ms`
    /// - `open`  = `v` of the earliest tick in the bucket
    /// - `high`  = max `v` across all ticks in the bucket
    /// - `low`   = min `v` across all ticks in the bucket
    /// - `close` = `v` of the latest tick in the bucket
    /// - `volume` = 0 (valuations carry no volume)
    ///
    /// Output is sorted oldest-first. Empty buckets are omitted.
    pub fn parse_valuations_as_klines_bucketed(response: &Value, interval_ms: i64) -> ExchangeResult<Vec<Kline>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Crypto.com valuations: missing 'result.data'".into()))?;

        // Collect (timestamp_ms, price) pairs, skipping malformed entries.
        let ticks: Vec<(i64, f64)> = data.iter()
            .filter_map(|item| {
                let ts = Self::get_i64(item, "t")?;
                let v  = Self::get_f64(item, "v")?;
                Some((ts, v))
            })
            .collect();

        if ticks.is_empty() {
            return Ok(Vec::new());
        }

        // Use a BTreeMap keyed by bar_open so buckets are auto-sorted oldest-first.
        use std::collections::BTreeMap;

        struct BucketState {
            high:  f64,
            low:   f64,
            // Track earliest/latest tick timestamp to resolve open/close correctly
            // when the API does not guarantee ordering within a bucket.
            first_ts: i64,
            last_ts:  i64,
            first_v:  f64,
            last_v:   f64,
        }

        let mut buckets: BTreeMap<i64, BucketState> = BTreeMap::new();

        for (ts, v) in &ticks {
            let bar_open = (ts / interval_ms) * interval_ms;
            let entry = buckets.entry(bar_open).or_insert(BucketState {
                high:     *v,
                low:      *v,
                first_ts: *ts,
                last_ts:  *ts,
                first_v:  *v,
                last_v:   *v,
            });

            if *v > entry.high { entry.high = *v; }
            if *v < entry.low  { entry.low  = *v; }

            if *ts < entry.first_ts {
                entry.first_ts = *ts;
                entry.first_v  = *v;
            }
            if *ts > entry.last_ts {
                entry.last_ts = *ts;
                entry.last_v  = *v;
            }
        }

        let klines = buckets
            .into_iter()
            .map(|(bar_open, b)| Kline {
                open_time:    bar_open,
                open:         b.first_v,
                high:         b.high,
                low:          b.low,
                close:        b.last_v,
                volume:       0.0,
                quote_volume: None,
                close_time:   None,
                trades:       None,
                ..Default::default()
            })
            .collect();

        Ok(klines)
    }

    /// Parse `public/get-valuations` response as a `Vec<FundingRate>`.
    ///
    /// Used for `valuation_type=funding_hist`.
    /// Each data point is `{"v": "<rate>", "t": <unix_ms>}` where `v` is the
    /// settled funding rate (hourly).
    ///
    /// Response structure mirrors `parse_valuations_as_klines`.
    pub fn parse_valuations_as_funding_rates(response: &Value) -> ExchangeResult<Vec<FundingRate>> {
        Self::check_response(response)?;
        let result = Self::extract_result(response)?;
        let data = result.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Crypto.com funding hist: missing 'result.data'".into()))?;

        let mut rates = Vec::with_capacity(data.len());
        for item in data {
            let ts = Self::get_i64(item, "t").unwrap_or(0);
            let rate = Self::get_f64(item, "v").unwrap_or(0.0);
            rates.push(FundingRate {
                rate,
                next_funding_time: None,
                timestamp: ts, ..Default::default() 
            });
        }
        Ok(rates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_response_success() {
        let response = json!({
            "code": 0,
            "result": {}
        });
        assert!(CryptoComParser::check_response(&response).is_ok());
    }

    #[test]
    fn test_check_response_error() {
        let response = json!({
            "code": 10003,
            "message": "INVALID_SIGNATURE"
        });
        assert!(CryptoComParser::check_response(&response).is_err());
    }

    #[test]
    fn test_parse_price() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "i": "BTCUSD-PERP",
                    "a": "50000.00"
                }]
            }
        });

        let price = CryptoComParser::parse_price(&response).unwrap();
        assert!((price - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "bids": [["50000.00", "1.5"], ["49999.00", "2.0"]],
                    "asks": [["50001.00", "1.0"], ["50002.00", "0.5"]],
                    "t": 1234567890
                }]
            }
        });

        let orderbook = CryptoComParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!((orderbook.bids[0].price - 50000.0).abs() < f64::EPSILON);
        assert_eq!(orderbook.timestamp, 1234567890);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "code": 0,
            "result": {
                "data": [{
                    "i": "BTCUSD-PERP",
                    "b": "50000.00",
                    "k": "50001.00",
                    "a": "50000.50",
                    "h": "51000.00",
                    "l": "49000.00",
                    "v": "1000.5",
                    "vv": "50000000",
                    "c": "0.02",
                    "t": 1234567890
                }]
            }
        });

        let ticker = CryptoComParser::parse_ticker(&response).unwrap();
        assert!((ticker.last_price - 50000.50).abs() < f64::EPSILON);
        assert_eq!(ticker.timestamp, 1234567890);
    }

    #[test]
    fn test_parse_recent_trades() {
        let response = json!({
            "id": -1,
            "method": "public/get-trades",
            "code": 0,
            "result": {
                "data": [{
                    "d": "1781450491821274859",
                    "t": 1781450491821_i64,
                    "tn": 1781450491821274859_i64,
                    "q": "0.04522",
                    "p": "64072.10",
                    "s": "sell",
                    "i": "BTC_USDT",
                    "m": "4611686018682685149"
                }]
            }
        });

        let trades = CryptoComParser::parse_recent_trades(&response).unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        assert_eq!(t.id, "1781450491821274859");
        assert!((t.price - 64072.10).abs() < 0.001);
        assert!((t.quantity - 0.04522).abs() < 1e-6);
        assert_eq!(t.side, TradeSide::Sell);
        assert_eq!(t.timestamp, 1781450491821);
    }

    #[test]
    fn test_parse_order_status() {
        let data = json!({"status": "FILLED"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::Filled);

        let data = json!({"status": "ACTIVE"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::New);

        let data = json!({"status": "CANCELED"});
        assert_eq!(CryptoComParser::parse_order_status(&data), OrderStatus::Canceled);
    }
}
