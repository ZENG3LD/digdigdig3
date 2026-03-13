//! # Paradex Response Parser
//!
//! Парсинг JSON ответов от Paradex API (REST + WebSocket).

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, Ticker, Order, Balance, Position,
    OrderSide, OrderType, OrderStatus, PositionSide,
    FundingRate, PublicTrade, StreamEvent, TradeSide,
    OrderUpdateEvent, BalanceUpdateEvent, PositionUpdateEvent,
    BalanceChangeReason, PositionChangeReason,
};

/// Парсер ответов Paradex API
pub struct ParadexParser;

impl ParadexParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Извлечь results из response (Paradex uses "results" array for list endpoints)
    pub fn extract_results(response: &Value) -> ExchangeResult<&Value> {
        response.get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' field".to_string()))
    }

    /// Парсить f64 из string или number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Парсить f64 из поля
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Парсить обязательный f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Парсить строку из поля
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Парсить обязательную строку
    fn require_str<'a>(data: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        Self::get_str(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))
    }

    /// Парсить i64 timestamp
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить price из markets/summary
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        if arr.is_empty() {
            return Err(ExchangeError::Parse("Empty results array".to_string()));
        }

        let data = &arr[0];
        Self::require_f64(data, "last_traded_price")
    }

    /// Парсить ticker из markets/summary
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        if arr.is_empty() {
            return Err(ExchangeError::Parse("Empty results array".to_string()));
        }

        let data = &arr[0];

        Ok(Ticker {
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            last_price: Self::get_f64(data, "last_traded_price").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "best_bid"),
            ask_price: Self::get_f64(data, "best_ask"),
            high_24h: None,
            low_24h: None,
            volume_24h: Self::get_f64(data, "volume_24h"),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "price_change_rate_24h")
                .map(|r| r * 100.0), // Convert 0.0234 to 2.34%
            timestamp: Self::get_i64(data, "created_at").unwrap_or(0),
        })
    }

    /// Парсить orderbook
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        let parse_levels = |key: &str| -> Vec<(f64, f64)> {
            response.get(key)
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
            timestamp: Self::get_i64(response, "last_updated_at").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: Self::get_i64(response, "seq_no").map(|n| n.to_string()),
        })
    }

    /// Парсить klines (candlestick data)
    ///
    /// Note: Exact format inferred from SDK, may need adjustment
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Expected format: array [timestamp, open, high, low, close, volume]
            if let Some(candle) = item.as_array() {
                if candle.len() < 6 {
                    continue;
                }

                let open_time = Self::parse_f64(&candle[0])
                    .map(|t| t as i64)
                    .unwrap_or(0);

                klines.push(Kline {
                    open_time,
                    open: Self::parse_f64(&candle[1]).unwrap_or(0.0),
                    high: Self::parse_f64(&candle[2]).unwrap_or(0.0),
                    low: Self::parse_f64(&candle[3]).unwrap_or(0.0),
                    close: Self::parse_f64(&candle[4]).unwrap_or(0.0),
                    volume: Self::parse_f64(&candle[5]).unwrap_or(0.0),
                    quote_volume: None,
                    close_time: None,
                    trades: None,
                });
            }
        }

        Ok(klines)
    }

    /// Парсить symbols (markets list)
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        let symbols: Vec<String> = arr.iter()
            .filter_map(|item| Self::get_str(item, "symbol").map(String::from))
            .collect();

        Ok(symbols)
    }

    /// Парсить funding rate
    pub fn parse_funding_rate(response: &Value) -> ExchangeResult<FundingRate> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        if arr.is_empty() {
            return Err(ExchangeError::Parse("Empty results array".to_string()));
        }

        let data = &arr[0];

        Ok(FundingRate {
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            rate: Self::require_f64(data, "funding_rate")?,
            next_funding_time: None,
            timestamp: Self::get_i64(data, "created_at").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TRADING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить order response (create/get order)
    pub fn parse_order(response: &Value) -> ExchangeResult<Order> {
        Self::parse_order_data(response)
    }

    /// Парсить order из data object
    fn parse_order_data(data: &Value) -> ExchangeResult<Order> {
        let status_str = Self::require_str(data, "status")?;
        let status = Self::parse_order_status(status_str);

        let side_str = Self::require_str(data, "side")?;
        let side = Self::parse_order_side(side_str);

        let type_str = Self::require_str(data, "type")?;
        let order_type = Self::parse_order_type(type_str);

        let size = Self::require_f64(data, "size")?;
        let remaining_size = Self::get_f64(data, "remaining_size").unwrap_or(size);
        let executed_qty = size - remaining_size;

        Ok(Order {
            id: Self::require_str(data, "id")?.to_string(),
            client_order_id: Self::get_str(data, "client_id").map(String::from),
            symbol: Self::require_str(data, "market")?.to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            stop_price: None,
            quantity: size,
            filled_quantity: executed_qty,
            average_price: Self::get_f64(data, "avg_fill_price"),
            commission: None,
            commission_asset: None,
            created_at: Self::get_i64(data, "created_at").unwrap_or(0),
            updated_at: Self::get_i64(data, "last_updated_at"),
            time_in_force: crate::core::TimeInForce::Gtc,
        })
    }

    /// Парсить список orders
    pub fn parse_orders(response: &Value) -> ExchangeResult<Vec<Order>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        arr.iter()
            .map(Self::parse_order_data)
            .collect()
    }

    /// Парсить order side
    fn parse_order_side(side_str: &str) -> OrderSide {
        match side_str.to_uppercase().as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy, // default
        }
    }

    /// Парсить order type
    fn parse_order_type(type_str: &str) -> OrderType {
        match type_str.to_uppercase().as_str() {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit { price: 0.0 },
            "STOP_LIMIT" => OrderType::StopLimit { stop_price: 0.0, limit_price: 0.0 },
            "STOP_MARKET" => OrderType::StopMarket { stop_price: 0.0 },
            "TAKE_PROFIT_LIMIT" => OrderType::Limit { price: 0.0 },
            "TAKE_PROFIT_MARKET" => OrderType::Limit { price: 0.0 },
            _ => OrderType::Limit { price: 0.0 }, // default
        }
    }

    /// Парсить order status
    fn parse_order_status(status_str: &str) -> OrderStatus {
        match status_str.to_uppercase().as_str() {
            "NEW" => OrderStatus::New,
            "UNTRIGGERED" => OrderStatus::New, // Stop order waiting for trigger
            "OPEN" => OrderStatus::Open,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CLOSED" => OrderStatus::Filled, // Closed = Filled
            "CANCELLED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::Open, // default
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ACCOUNT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить balance
    pub fn parse_balances(response: &Value) -> ExchangeResult<Vec<Balance>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        arr.iter()
            .map(|item| {
                let free = Self::get_f64(item, "available").unwrap_or(0.0);
                let locked = Self::get_f64(item, "locked").unwrap_or(0.0);
                Ok(Balance {
                    asset: Self::require_str(item, "currency")?.to_string(),
                    free,
                    locked,
                    total: free + locked,
                })
            })
            .collect()
    }

    /// Парсить account info
    pub fn parse_account_info(_response: &Value) -> ExchangeResult<crate::core::types::AccountInfo> {
        Ok(crate::core::types::AccountInfo {
            account_type: crate::core::AccountType::FuturesCross,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // Paradex has maker rebates
            taker_commission: 0.05, // 0.05% default
            balances: vec![], // Нужен отдельный запрос /balances
        })
    }

    /// Парсить positions
    pub fn parse_positions(response: &Value) -> ExchangeResult<Vec<Position>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        Ok(arr.iter()
            .filter_map(|item| Self::parse_position_data(item).ok())
            .collect())
    }

    /// Парсить один position
    fn parse_position_data(data: &Value) -> ExchangeResult<Position> {
        let side_str = Self::require_str(data, "side")?;
        let side = match side_str.to_uppercase().as_str() {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => PositionSide::Long,
        };

        let quantity = Self::require_f64(data, "size")?;
        let entry_price = Self::get_f64(data, "average_entry_price").unwrap_or(0.0);
        let unrealized_pnl = Self::get_f64(data, "unrealized_pnl").unwrap_or(0.0);

        Ok(Position {
            symbol: Self::require_str(data, "market")?.to_string(),
            side,
            quantity: quantity.abs(),
            entry_price,
            mark_price: Self::get_f64(data, "mark_price"),
            unrealized_pnl,
            realized_pnl: None,
            liquidation_price: Self::get_f64(data, "liquidation_price"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32).unwrap_or(1),
            margin_type: crate::core::MarginType::Cross,
            margin: Self::get_f64(data, "cost"),
            take_profit: None,
            stop_loss: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Парсить WebSocket message (JSON-RPC 2.0 format)
    ///
    /// `target_symbol` — the Paradex market identifier we subscribed to (e.g. `"BTC-USD-PERP"`).
    /// For the global `markets_summary` channel only data matching this symbol is emitted.
    pub fn parse_ws_message(text: &str, target_symbol: Option<&str>) -> ExchangeResult<StreamEvent> {
        let msg: Value = serde_json::from_str(text)
            .map_err(|e| ExchangeError::Parse(format!("Invalid JSON: {}", e)))?;

        // Check if this is a subscription notification
        if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
            if method == "subscription" {
                return Self::parse_ws_subscription(&msg, target_symbol);
            }
        }

        // Authentication response or other control messages - ignore
        Err(ExchangeError::Parse("Not a stream event".to_string()))
    }

    /// Парсить subscription notification
    fn parse_ws_subscription(msg: &Value, target_symbol: Option<&str>) -> ExchangeResult<StreamEvent> {
        let params = msg.get("params")
            .ok_or_else(|| ExchangeError::Parse("Missing 'params' in subscription".to_string()))?;

        let channel = Self::require_str(params, "channel")?;
        let data = params.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' in subscription".to_string()))?;

        // Parse by channel
        //
        // Global markets_summary (all markets) — filter by target symbol.
        if channel == "markets_summary" {
            return Self::parse_ws_ticker(data, target_symbol);
        }

        // Per-market markets_summary.{market} — pass target_symbol for consistency.
        if channel.starts_with("markets_summary.") {
            return Self::parse_ws_ticker(data, target_symbol);
        }

        if channel.starts_with("order_book.") {
            return Self::parse_ws_orderbook(data);
        }

        if channel.starts_with("bbo.") {
            return Self::parse_ws_bbo(data);
        }

        if channel.starts_with("trades.") {
            return Self::parse_ws_trades(data);
        }

        // Funding rate data
        if channel.starts_with("funding_data.") {
            return Self::parse_ws_funding_data(data);
        }

        // Private channels
        //
        // orders (global or per-market: "orders" or "orders.{market}")
        if channel == "orders" || channel.starts_with("orders.") {
            return Self::parse_ws_order_update(data);
        }

        if channel == "positions" {
            return Self::parse_ws_position_update(data);
        }

        if channel == "account" || channel == "balance_events" {
            return Self::parse_ws_balance_update(data);
        }

        // fills.{market} — trade fill events for the authenticated account
        if channel.starts_with("fills.") {
            return Self::parse_ws_fill(data);
        }

        // Unknown channel - ignore
        Err(ExchangeError::Parse(format!("Unknown channel: {}", channel)))
    }

    /// Парсить WebSocket ticker
    ///
    /// `target_symbol` — when provided, only emit a ticker if the `market` field in `data`
    /// matches this value (case-insensitive). This prevents global `markets_summary`
    /// updates for other markets from being forwarded as a ticker for the subscribed symbol.
    fn parse_ws_ticker(data: &Value, target_symbol: Option<&str>) -> ExchangeResult<StreamEvent> {
        let market = Self::get_str(data, "market").unwrap_or("");

        // Filter: if we know which symbol was subscribed, drop events for other markets.
        if let Some(target) = target_symbol {
            if !market.eq_ignore_ascii_case(target) {
                return Err(ExchangeError::Parse(format!(
                    "Skipping markets_summary update for '{}' (subscribed to '{}')",
                    market, target
                )));
            }
        }

        let ticker = Ticker {
            symbol: market.to_string(),
            last_price: Self::get_f64(data, "last_traded_price").unwrap_or(0.0),
            bid_price: Self::get_f64(data, "best_bid"),
            ask_price: Self::get_f64(data, "best_ask"),
            high_24h: None,
            low_24h: None,
            volume_24h: Self::get_f64(data, "volume_24h"),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: Self::get_f64(data, "price_change_rate_24h")
                .map(|r| r * 100.0),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        Ok(StreamEvent::Ticker(ticker))
    }

    /// Парсить WebSocket orderbook
    fn parse_ws_orderbook(data: &Value) -> ExchangeResult<StreamEvent> {
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

        let orderbook = OrderBook {
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
            bids: parse_levels("bids"),
            asks: parse_levels("asks"),
            sequence: Self::get_i64(data, "seq_no").map(|n| n.to_string()),
        };

        Ok(StreamEvent::OrderbookSnapshot(orderbook))
    }

    /// Парсить WebSocket BBO (best bid/offer)
    fn parse_ws_bbo(data: &Value) -> ExchangeResult<StreamEvent> {
        let orderbook = OrderBook {
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
            bids: vec![(
                Self::get_f64(data, "bid").unwrap_or(0.0),
                Self::get_f64(data, "bid_size").unwrap_or(0.0),
            )],
            asks: vec![(
                Self::get_f64(data, "ask").unwrap_or(0.0),
                Self::get_f64(data, "ask_size").unwrap_or(0.0),
            )],
            sequence: Self::get_i64(data, "seq_no").map(|n| n.to_string()),
        };

        Ok(StreamEvent::OrderbookSnapshot(orderbook))
    }

    /// Парсить WebSocket trades
    fn parse_ws_trades(data: &Value) -> ExchangeResult<StreamEvent> {
        let side_str = Self::get_str(data, "side").unwrap_or("BUY");
        let side = match side_str.to_uppercase().as_str() {
            "BUY" => TradeSide::Buy,
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        let trade = PublicTrade {
            id: Self::get_str(data, "id").unwrap_or("").to_string(),
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            price: Self::get_f64(data, "price").unwrap_or(0.0),
            quantity: Self::get_f64(data, "size").unwrap_or(0.0),
            side,
            timestamp: Self::get_i64(data, "timestamp").unwrap_or(0),
        };

        Ok(StreamEvent::Trade(trade))
    }

    /// Парсить WebSocket order update
    fn parse_ws_order_update(data: &Value) -> ExchangeResult<StreamEvent> {
        let status_str = Self::require_str(data, "status")?;
        let status = Self::parse_order_status(status_str);

        let side_str = Self::require_str(data, "side")?;
        let side = Self::parse_order_side(side_str);

        let type_str = Self::get_str(data, "type").unwrap_or("LIMIT");
        let order_type = Self::parse_order_type(type_str);

        let size = Self::require_f64(data, "size")?;
        let remaining_size = Self::get_f64(data, "remaining_size").unwrap_or(size);
        let filled_qty = size - remaining_size;

        let event = OrderUpdateEvent {
            order_id: Self::require_str(data, "id")?.to_string(),
            client_order_id: Self::get_str(data, "client_id").map(String::from),
            symbol: Self::require_str(data, "market")?.to_string(),
            side,
            order_type,
            status,
            price: Self::get_f64(data, "price"),
            quantity: size,
            filled_quantity: filled_qty,
            average_price: Self::get_f64(data, "avg_fill_price"),
            last_fill_price: None,
            last_fill_quantity: None,
            last_fill_commission: None,
            commission_asset: None,
            trade_id: None,
            timestamp: Self::get_i64(data, "last_updated_at").unwrap_or(0),
        };

        Ok(StreamEvent::OrderUpdate(event))
    }

    /// Парсить WebSocket position update
    fn parse_ws_position_update(data: &Value) -> ExchangeResult<StreamEvent> {
        let side_str = Self::require_str(data, "side")?;
        let side = match side_str.to_uppercase().as_str() {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => PositionSide::Long,
        };

        let quantity = Self::require_f64(data, "size")?;
        let entry_price = Self::get_f64(data, "average_entry_price").unwrap_or(0.0);
        let unrealized_pnl = Self::get_f64(data, "unrealized_pnl").unwrap_or(0.0);

        let event = PositionUpdateEvent {
            symbol: Self::require_str(data, "market")?.to_string(),
            side,
            quantity: quantity.abs(),
            entry_price,
            mark_price: None,
            unrealized_pnl,
            realized_pnl: None,
            liquidation_price: Self::get_f64(data, "liquidation_price"),
            leverage: Self::get_f64(data, "leverage").map(|l| l as u32),
            margin_type: Some(crate::core::MarginType::Cross),
            reason: Some(PositionChangeReason::Trade),
            timestamp: Self::get_i64(data, "last_updated_at").unwrap_or(0),
        };

        Ok(StreamEvent::PositionUpdate(event))
    }

    /// Парсить WebSocket balance update
    fn parse_ws_balance_update(data: &Value) -> ExchangeResult<StreamEvent> {
        let free = Self::get_f64(data, "free_collateral").unwrap_or(0.0);
        let locked = 0.0;

        let event = BalanceUpdateEvent {
            asset: "USDC".to_string(), // Paradex uses USDC as settlement
            free,
            locked,
            total: free + locked,
            delta: None,
            reason: Some(BalanceChangeReason::Trade),
            timestamp: Self::get_i64(data, "updated_at").unwrap_or(0),
        };

        Ok(StreamEvent::BalanceUpdate(event))
    }

    /// Parse a `funding_data.{market}` WebSocket message into `StreamEvent::FundingRate`.
    ///
    /// Expected payload fields (best-effort):
    /// - `market` — market identifier (e.g. `"BTC-USD-PERP"`)
    /// - `funding_rate` — current funding rate (e.g. `"0.0000125"`)
    /// - `funding_index` — cumulative funding index
    /// - `next_funding_at` — Unix ms timestamp of the next funding settlement
    fn parse_ws_funding_data(data: &Value) -> ExchangeResult<StreamEvent> {
        let symbol = Self::get_str(data, "market")
            .unwrap_or("")
            .to_string();
        let rate = Self::get_f64(data, "funding_rate")
            .or_else(|| Self::get_f64(data, "funding_index"))
            .unwrap_or(0.0);
        let next_funding_time = Self::get_i64(data, "next_funding_at")
            .or_else(|| Self::get_i64(data, "next_funding_time"));
        let timestamp = Self::get_i64(data, "timestamp")
            .or_else(|| Self::get_i64(data, "created_at"))
            .unwrap_or(0);

        Ok(StreamEvent::FundingRate {
            symbol,
            rate,
            next_funding_time,
            timestamp,
        })
    }

    /// Parse a `fills.{market}` WebSocket message into `StreamEvent::OrderUpdate`.
    ///
    /// A fill represents a trade execution for the authenticated account.
    /// Fields:
    /// - `id` — fill id
    /// - `order_id` — the order that was filled
    /// - `market` — market identifier
    /// - `side` — `"BUY"` | `"SELL"`
    /// - `price` — execution price
    /// - `size` — executed quantity
    /// - `fee` — commission paid
    /// - `fee_currency` — asset used for fee
    /// - `created_at` — Unix ms timestamp
    fn parse_ws_fill(data: &Value) -> ExchangeResult<StreamEvent> {
        let side_str = Self::get_str(data, "side").unwrap_or("BUY");
        let side = Self::parse_order_side(side_str);

        let size = Self::get_f64(data, "size").unwrap_or(0.0);
        let fill_price = Self::get_f64(data, "price");
        let fee = Self::get_f64(data, "fee");

        let event = OrderUpdateEvent {
            order_id: Self::get_str(data, "order_id").unwrap_or("").to_string(),
            client_order_id: Self::get_str(data, "client_id").map(String::from),
            symbol: Self::get_str(data, "market").unwrap_or("").to_string(),
            side,
            order_type: crate::core::types::OrderType::Market, // fills don't carry order type
            status: crate::core::types::OrderStatus::Filled,
            price: fill_price,
            quantity: size,
            filled_quantity: size,
            average_price: fill_price,
            last_fill_price: fill_price,
            last_fill_quantity: Some(size),
            last_fill_commission: fee,
            commission_asset: Self::get_str(data, "fee_currency").map(String::from),
            trade_id: Self::get_str(data, "id").map(String::from),
            timestamp: Self::get_i64(data, "created_at").unwrap_or(0),
        };

        Ok(StreamEvent::OrderUpdate(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price() {
        let json = r#"{"results":[{"market":"BTC-USD-PERP","last_traded_price":"65432.3"}]}"#;
        let response: Value = serde_json::from_str(json).unwrap();
        let price = ParadexParser::parse_price(&response).unwrap();
        assert_eq!(price, 65432.3);
    }

    #[test]
    fn test_parse_ticker() {
        let json = r#"{
            "results": [{
                "market": "BTC-USD-PERP",
                "best_bid": "65432.1",
                "best_ask": "65432.5",
                "last_traded_price": "65432.3",
                "volume_24h": "123456789.50",
                "price_change_rate_24h": "0.0234",
                "created_at": 1681759756789
            }]
        }"#;
        let response: Value = serde_json::from_str(json).unwrap();
        let ticker = ParadexParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.symbol, "BTC-USD-PERP");
        assert_eq!(ticker.last_price, 65432.3);
    }

    #[test]
    fn test_parse_orderbook() {
        let json = r#"{
            "market": "BTC-USD-PERP",
            "asks": [["65432.5", "1.234"], ["65432.6", "2.456"]],
            "bids": [["65432.4", "1.111"], ["65432.3", "2.222"]],
            "last_updated_at": 1681759756789,
            "seq_no": 12345678
        }"#;
        let response: Value = serde_json::from_str(json).unwrap();
        let orderbook = ParadexParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids[0].0, 65432.4);
    }
}
