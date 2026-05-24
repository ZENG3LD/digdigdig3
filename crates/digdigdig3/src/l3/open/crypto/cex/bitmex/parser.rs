//! BitMEX WebSocket frame parsers.
//!
//! All parsers receive the full frame Value.  BitMEX push format:
//! ```json
//! {"table": "<topic>", "action": "partial"|"insert"|"update"|"delete", "data": [...]}
//! ```
//! Delta frames carry only changed fields — parsers use `?` / `continue` when a
//! required field is absent in a specific row (that row is silently skipped).

use chrono::DateTime;
use serde_json::Value;

use crate::core::types::{StreamEvent, TradeSide, WebSocketError, WebSocketResult};

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the "data" array from a BitMEX frame.
fn frame_data(raw: &Value) -> WebSocketResult<&Vec<Value>> {
    raw.get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| WebSocketError::Parse("bitmex: frame missing 'data' array".into()))
}

/// Parse an ISO 8601 timestamp string to milliseconds since epoch.
fn iso_to_ms(s: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

/// Current time in ms as fallback when no timestamp field is present.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────────────────────
// instrument channel — PredictedFunding
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `instrument:{sym}` frame → `StreamEvent::PredictedFunding`.
///
/// BitMEX sends `indicativeFundingRate` (NOT `estimatedFundingRate`) as the
/// predicted next funding rate.  Delta updates include only changed fields, so
/// rows without `indicativeFundingRate` are silently skipped — that is normal
/// behaviour and is NOT an error.
pub fn parse_predicted_funding(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let predicted_rate = match item.get("indicativeFundingRate").and_then(Value::as_f64) {
            Some(r) => r,
            None => continue, // field absent in this delta row
        };

        let next_funding_time = item
            .get("fundingTimestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or(0);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        // Return on first row that carries the field.  BitMEX perpetuals like
        // XBTUSD emit one-item data arrays on most updates.
        return Ok(StreamEvent::PredictedFunding {
            symbol,
            predicted_rate,
            next_funding_time,
            timestamp,
        });
    }

    Err(WebSocketError::FieldAbsent(
        "bitmex instrument: no row contained indicativeFundingRate".into(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// instrument channel — FundingRate
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `instrument:{sym}` frame → `StreamEvent::FundingRate`.
///
/// `fundingRate` is the locked rate for the current 8h period.  It changes
/// once per period at 04:00 / 12:00 / 20:00 UTC.
pub fn parse_funding_rate(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let rate = match item.get("fundingRate").and_then(Value::as_f64) {
            Some(r) => r,
            None => continue,
        };

        let next_funding_time = item
            .get("fundingTimestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        return Ok(StreamEvent::FundingRate {
            symbol,
            rate,
            next_funding_time,
            timestamp,
        });
    }

    Err(WebSocketError::FieldAbsent(
        "bitmex instrument: no row contained fundingRate".into(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// instrument channel — MarkPrice
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `instrument:{sym}` frame → `StreamEvent::MarkPrice`.
pub fn parse_mark_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let mark_price = match item.get("markPrice").and_then(Value::as_f64) {
            Some(p) => p,
            None => continue,
        };

        let index_price = item.get("indexPrice").and_then(Value::as_f64);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        return Ok(StreamEvent::MarkPrice {
            symbol,
            mark_price,
            index_price,
            timestamp,
        });
    }

    Err(WebSocketError::FieldAbsent(
        "bitmex instrument: no row contained markPrice".into(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// instrument channel — IndexPrice
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `instrument:{sym}` frame → `StreamEvent::IndexPrice`.
pub fn parse_index_price(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        // BitMEX uses `indicativeSettlePrice` as the index / spot reference price.
        // `indexPrice` is also available on some instruments.
        let price = match item
            .get("indexPrice")
            .or_else(|| item.get("indicativeSettlePrice"))
            .and_then(Value::as_f64)
        {
            Some(p) => p,
            None => continue,
        };

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        return Ok(StreamEvent::IndexPrice { symbol, price, timestamp });
    }

    Err(WebSocketError::FieldAbsent(
        "bitmex instrument: no row contained indexPrice or indicativeSettlePrice".into(),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// trade channel
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `trade:{sym}` frame → `StreamEvent::Trade`.
///
/// BitMEX trade frames always use `"action": "insert"`.  Data is an array —
/// we return on the first valid row (caller dispatches once per frame).
pub fn parse_trade(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::PublicTrade;

    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let price = match item.get("price").and_then(Value::as_f64) {
            Some(p) => p,
            None => continue,
        };

        let quantity = item
            .get("size")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        let side = item
            .get("side")
            .and_then(Value::as_str)
            .map(|s| if s == "Buy" { TradeSide::Buy } else { TradeSide::Sell })
            .unwrap_or(TradeSide::Buy);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        let trade_id = item
            .get("trdMatchID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let trade = PublicTrade {
            id: trade_id,
            price,
            quantity,
            side,
            timestamp,
        };

        return Ok(StreamEvent::Trade { symbol, trade });
    }

    Err(WebSocketError::Parse("bitmex trade: empty or invalid data array".into()))
}

// ─────────────────────────────────────────────────────────────────────────────
// quote channel
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `quote:{sym}` frame → `StreamEvent::Ticker`.
///
/// BitMEX `quote` channel provides best bid/ask + sizes.
pub fn parse_quote(raw: &Value) -> WebSocketResult<StreamEvent> {
    use crate::core::types::Ticker;

    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let bid_price = item.get("bidPrice").and_then(Value::as_f64);
        let ask_price = item.get("askPrice").and_then(Value::as_f64);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        let ticker = Ticker {
            last_price: bid_price.or(ask_price).unwrap_or(0.0),
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        };

        return Ok(StreamEvent::Ticker { symbol, ticker });
    }

    Err(WebSocketError::Parse("bitmex quote: empty or invalid data array".into()))
}

// ─────────────────────────────────────────────────────────────────────────────
// liquidation channel
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `liquidation` (global) frame → `StreamEvent::Liquidation`.
pub fn parse_liquidation(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let price = match item.get("price").and_then(Value::as_f64) {
            Some(p) => p,
            None => continue,
        };

        let quantity = item
            .get("leavesQty")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        // BitMEX `side` on a liquidation is the direction of the liquidation order
        // (opposite to the position that was liquidated).
        let side = item
            .get("side")
            .and_then(Value::as_str)
            .map(|s| if s == "Buy" { TradeSide::Buy } else { TradeSide::Sell })
            .unwrap_or(TradeSide::Sell);

        return Ok(StreamEvent::Liquidation {
            symbol,
            side,
            price,
            quantity,
            value: None,
            timestamp: now_ms(),
        });
    }

    Err(WebSocketError::Parse("bitmex liquidation: empty or invalid data array".into()))
}

// ─────────────────────────────────────────────────────────────────────────────
// funding channel — settlement events
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `funding:{sym}` frame → `StreamEvent::FundingSettlement`.
///
/// Published at each 8-hour funding settlement (04:00, 12:00, 20:00 UTC).
pub fn parse_funding_settled(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let settled_rate = match item.get("fundingRate").and_then(Value::as_f64) {
            Some(r) => r,
            None => continue,
        };

        let settlement_time = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or(0);

        let timestamp = settlement_time;

        return Ok(StreamEvent::FundingSettlement {
            symbol,
            settled_rate,
            settlement_time,
            timestamp,
        });
    }

    Err(WebSocketError::Parse("bitmex funding: empty or invalid data array".into()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn instrument_frame(symbol: &str, indicative_rate: f64, funding_ts: &str, ts: &str) -> Value {
        serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{
                "symbol": symbol,
                "indicativeFundingRate": indicative_rate,
                "fundingTimestamp": funding_ts,
                "timestamp": ts
            }]
        })
    }

    #[test]
    fn parse_predicted_funding_extracts_indicative_rate() {
        let frame = instrument_frame(
            "XBTUSD",
            0.000085,
            "2024-01-01T08:00:00.000Z",
            "2024-01-01T07:45:00.123Z",
        );

        let event = parse_predicted_funding(&frame).expect("should parse PredictedFunding");
        match event {
            StreamEvent::PredictedFunding {
                symbol,
                predicted_rate,
                next_funding_time,
                timestamp,
            } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((predicted_rate - 0.000085).abs() < 1e-12, "rate mismatch");
                assert!(next_funding_time > 0, "next_funding_time must be set");
                assert!(timestamp > 0, "timestamp must be set");
            }
            other => panic!("expected PredictedFunding, got {:?}", other),
        }
    }

    #[test]
    fn parse_predicted_funding_skips_row_without_field() {
        // Delta frame without indicativeFundingRate — should return FieldAbsent
        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{"symbol": "XBTUSD", "markPrice": 45200.0, "timestamp": "2024-01-01T07:45:00.000Z"}]
        });
        let err = parse_predicted_funding(&frame).expect_err("should return FieldAbsent");
        assert!(
            matches!(err, WebSocketError::FieldAbsent(_)),
            "expected FieldAbsent, got {:?}", err
        );
    }

    #[test]
    fn parse_funding_rate_extracts_funding_rate_field() {
        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{
                "symbol": "XBTUSD",
                "fundingRate": 0.0001,
                "fundingTimestamp": "2024-01-01T08:00:00.000Z",
                "timestamp": "2024-01-01T04:00:00.000Z"
            }]
        });
        let event = parse_funding_rate(&frame).expect("should parse FundingRate");
        match event {
            StreamEvent::FundingRate { symbol, rate, next_funding_time, .. } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((rate - 0.0001).abs() < 1e-12);
                assert!(next_funding_time.is_some());
            }
            other => panic!("expected FundingRate, got {:?}", other),
        }
    }

    #[test]
    fn parse_trade_extracts_price_qty_side() {
        let frame = serde_json::json!({
            "table": "trade",
            "action": "insert",
            "data": [{
                "symbol": "XBTUSD",
                "side": "Buy",
                "size": 100,
                "price": 45200.0,
                "trdMatchID": "abc-123",
                "timestamp": "2024-01-01T00:00:00.123Z"
            }]
        });
        let event = parse_trade(&frame).expect("should parse Trade");
        match event {
            StreamEvent::Trade { symbol, trade } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((trade.price - 45200.0).abs() < 1e-6);
                assert_eq!(trade.side, TradeSide::Buy);
                assert_eq!(trade.id, "abc-123");
            }
            other => panic!("expected Trade, got {:?}", other),
        }
    }

    #[test]
    fn parse_funding_settled_extracts_settled_rate() {
        let frame = serde_json::json!({
            "table": "funding",
            "action": "insert",
            "data": [{
                "symbol": "XBTUSD",
                "fundingRate": 0.0001,
                "fundingInterval": "2000-01-01T08:00:00.000Z",
                "fundingRateDaily": 0.0003,
                "timestamp": "2024-01-01T08:00:00.000Z"
            }]
        });
        let event = parse_funding_settled(&frame).expect("should parse FundingSettlement");
        match event {
            StreamEvent::FundingSettlement { symbol, settled_rate, .. } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((settled_rate - 0.0001).abs() < 1e-12);
            }
            other => panic!("expected FundingSettlement, got {:?}", other),
        }
    }
}
