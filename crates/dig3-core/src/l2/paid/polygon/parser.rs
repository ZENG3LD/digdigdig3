//! # Polygon.io Response Parser
//!
//! Parse JSON responses from Polygon.io API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, StreamEvent,
};

/// Parser for Polygon.io API responses
pub struct PolygonParser;

impl PolygonParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract results from response
    pub fn extract_results(response: &Value) -> ExchangeResult<&Value> {
        // Check status first
        if let Some(status) = response.get("status").and_then(|s| s.as_str()) {
            if status == "ERROR" {
                let error_msg = response.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api { code: -1, message: error_msg.to_string() });
            }
        }

        response.get("results")
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' field".to_string()))
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

    /// Parse i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse price from snapshot or last trade
    pub fn _parse_price(response: &Value) -> ExchangeResult<f64> {
        // Try snapshot format first
        if let Some(ticker) = response.get("ticker") {
            if let Some(last_trade) = ticker.get("lastTrade") {
                if let Some(price) = Self::get_f64(last_trade, "p") {
                    return Ok(price);
                }
            }
            if let Some(day) = ticker.get("day") {
                if let Some(price) = Self::get_f64(day, "c") {
                    return Ok(price);
                }
            }
        }

        // Try last trade format
        let results = Self::extract_results(response)?;
        if let Some(price) = Self::get_f64(results, "p") {
            return Ok(price);
        }

        Err(ExchangeError::Parse("Could not extract price".to_string()))
    }

    /// Parse OHLC aggregates (klines)
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let results = Self::extract_results(response)?;
        let arr = results.as_array()
            .ok_or_else(|| ExchangeError::Parse("'results' is not an array".to_string()))?;

        let mut klines = Vec::with_capacity(arr.len());

        for item in arr {
            // Polygon aggregate format:
            // { "c": close, "h": high, "l": low, "o": open, "t": timestamp_ms, "v": volume, "vw": vwap, "n": trades }
            let open_time = Self::get_i64(item, "t").unwrap_or(0);
            let open = Self::get_f64(item, "o").unwrap_or(0.0);
            let high = Self::get_f64(item, "h").unwrap_or(0.0);
            let low = Self::get_f64(item, "l").unwrap_or(0.0);
            let close = Self::get_f64(item, "c").unwrap_or(0.0);
            let volume = Self::get_f64(item, "v").unwrap_or(0.0);
            let trades = Self::get_i64(item, "n").map(|n| n as u64);

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                close_time: None,
                quote_volume: Self::get_f64(item, "vw"), // Use VWAP as quote_volume
                trades,
            });
        }

        Ok(klines)
    }

    /// Parse ticker/snapshot data
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        // Polygon snapshot format
        let ticker_obj = response.get("ticker")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticker' field".to_string()))?;

        let symbol = Self::get_str(ticker_obj, "ticker")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticker' symbol".to_string()))?
            .to_string();

        // Get last price from day data or last trade
        let last_price = if let Some(day) = ticker_obj.get("day") {
            Self::get_f64(day, "c").unwrap_or(0.0)
        } else if let Some(last_trade) = ticker_obj.get("lastTrade") {
            Self::get_f64(last_trade, "p").unwrap_or(0.0)
        } else {
            0.0
        };

        // Get volume from day data
        let volume = ticker_obj.get("day")
            .and_then(|d| Self::get_f64(d, "v"))
            .unwrap_or(0.0);

        // Get 24h change
        let price_change = Self::get_f64(ticker_obj, "todaysChange");
        let price_change_percent = Self::get_f64(ticker_obj, "todaysChangePerc");

        // Get high/low from day data
        let day = ticker_obj.get("day");
        let high = day.and_then(|d| Self::get_f64(d, "h"));
        let low = day.and_then(|d| Self::get_f64(d, "l"));
        let _open = day.and_then(|d| Self::get_f64(d, "o"));

        // Get bid/ask from last quote
        let last_quote = ticker_obj.get("lastQuote");
        let bid = last_quote.and_then(|q| Self::get_f64(q, "p"));
        let ask = last_quote.and_then(|q| Self::get_f64(q, "P"));
        let _bid_size = last_quote.and_then(|q| Self::get_f64(q, "s"));
        let _ask_size = last_quote.and_then(|q| Self::get_f64(q, "S"));

        Ok(Ticker {
            symbol,
            last_price,
            bid_price: bid,
            ask_price: ask,
            high_24h: high,
            low_24h: low,
            volume_24h: Some(volume),
            quote_volume_24h: None,
            price_change_24h: price_change,
            price_change_percent_24h: price_change_percent,
            timestamp: Self::get_i64(ticker_obj, "updated").unwrap_or(0),
        })
    }

    /// Parse orderbook (Polygon doesn't provide full orderbook, only best bid/ask)
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        // Polygon only provides NBBO (best bid/ask), not full orderbook
        // We'll return a minimal orderbook with just top level
        let ticker_obj = response.get("ticker")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticker' field".to_string()))?;

        let last_quote = ticker_obj.get("lastQuote")
            .ok_or_else(|| ExchangeError::Parse("Missing 'lastQuote' field".to_string()))?;

        let bid_price = Self::require_f64(last_quote, "p")?;
        let ask_price = Self::require_f64(last_quote, "P")?;
        let bid_size = Self::get_f64(last_quote, "s").unwrap_or(0.0);
        let ask_size = Self::get_f64(last_quote, "S").unwrap_or(0.0);

        Ok(OrderBook {
            bids: vec![OrderBookLevel::new(bid_price, bid_size)],
            asks: vec![OrderBookLevel::new(ask_price, ask_size)],
            timestamp: Self::get_i64(last_quote, "t").unwrap_or(0),
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET MESSAGES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket message to StreamEvent
    pub fn parse_ws_message(msg: &Value) -> ExchangeResult<Vec<StreamEvent>> {
        // Polygon WebSocket messages are arrays of events
        let events = msg.as_array()
            .ok_or_else(|| ExchangeError::Parse("WebSocket message is not an array".to_string()))?;

        let mut stream_events = Vec::new();

        for event in events {
            let event_type = Self::get_str(event, "ev")
                .ok_or_else(|| ExchangeError::Parse("Missing 'ev' field".to_string()))?;

            match event_type {
                "status" => {
                    // Status messages (connected, auth_success, etc.)
                    // Skip for now, handled in WebSocket layer
                    continue;
                }
                "AM" => {
                    // Minute aggregate
                    if let Ok(kline_event) = Self::parse_ws_aggregate(event) {
                        stream_events.push(kline_event);
                    }
                }
                "AS" => {
                    // Second aggregate
                    if let Ok(kline_event) = Self::parse_ws_aggregate(event) {
                        stream_events.push(kline_event);
                    }
                }
                "T" => {
                    // Trade
                    if let Ok(trade_event) = Self::parse_ws_trade(event) {
                        stream_events.push(trade_event);
                    }
                }
                "Q" => {
                    // Quote
                    if let Ok(ticker_event) = Self::parse_ws_quote(event) {
                        stream_events.push(ticker_event);
                    }
                }
                _ => {
                    // Unknown event type, skip
                    continue;
                }
            }
        }

        Ok(stream_events)
    }

    /// Parse WebSocket aggregate (minute/second bar)
    fn parse_ws_aggregate(event: &Value) -> ExchangeResult<StreamEvent> {
        let _symbol = Self::get_str(event, "sym")
            .ok_or_else(|| ExchangeError::Parse("Missing 'sym' field".to_string()))?
            .to_string();

        let open_time = Self::get_i64(event, "s").unwrap_or(0);
        let close_time = Self::get_i64(event, "e");
        let open = Self::get_f64(event, "o").unwrap_or(0.0);
        let high = Self::get_f64(event, "h").unwrap_or(0.0);
        let low = Self::get_f64(event, "l").unwrap_or(0.0);
        let close = Self::get_f64(event, "c").unwrap_or(0.0);
        let volume = Self::get_f64(event, "v").unwrap_or(0.0);
        let vwap = Self::get_f64(event, "a");

        Ok(StreamEvent::Kline(Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            close_time,
            quote_volume: vwap,
            trades: None,
        }))
    }

    /// Parse WebSocket trade
    fn parse_ws_trade(event: &Value) -> ExchangeResult<StreamEvent> {
        let symbol = Self::get_str(event, "sym")
            .ok_or_else(|| ExchangeError::Parse("Missing 'sym' field".to_string()))?
            .to_string();

        let price = Self::require_f64(event, "p")?;
        let size = Self::get_f64(event, "s").unwrap_or(0.0);
        let timestamp = Self::get_i64(event, "t").unwrap_or(0);

        // Create a ticker event from trade data
        Ok(StreamEvent::Ticker(Ticker {
            symbol,
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: Some(size),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        }))
    }

    /// Parse WebSocket quote
    fn parse_ws_quote(event: &Value) -> ExchangeResult<StreamEvent> {
        let symbol = Self::get_str(event, "sym")
            .ok_or_else(|| ExchangeError::Parse("Missing 'sym' field".to_string()))?
            .to_string();

        let bid = Self::get_f64(event, "bp");
        let ask = Self::get_f64(event, "ap");
        let _bid_size = Self::get_f64(event, "bs");
        let _ask_size = Self::get_f64(event, "as");
        let timestamp = Self::get_i64(event, "t").unwrap_or(0);

        let last_price = ask.or(bid).unwrap_or(0.0);

        Ok(StreamEvent::Ticker(Ticker {
            symbol,
            last_price,
            bid_price: bid,
            ask_price: ask,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        }))
    }
}

