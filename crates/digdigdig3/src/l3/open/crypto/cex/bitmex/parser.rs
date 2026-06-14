//! BitMEX WebSocket frame parsers and REST response parsers.
//!
//! All parsers receive the full frame Value.  BitMEX push format:
//! ```json
//! {"table": "<topic>", "action": "partial"|"insert"|"update"|"delete", "data": [...]}
//! ```
//! Delta frames carry only changed fields — parsers use `?` / `continue` when a
//! required field is absent in a specific row (that row is silently skipped).

use chrono::DateTime;
use serde_json::Value;

use crate::core::types::{
    FundingRate, Kline, Liquidation, PublicTrade,
    StreamEvent, TradeSide, WebSocketError, WebSocketResult,
};
use crate::core::{ExchangeError, ExchangeResult};

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

// Shared wasm-safe wall-clock helper.
use crate::core::utils::now_ms;

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
// instrument channel — OpenInterest
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `instrument:{sym}` frame → `StreamEvent::OpenInterestUpdate`.
///
/// BitMEX `instrument` channel carries `openInterest` (number of open contracts)
/// and `openValue` (value in satoshis).  Like all instrument fields, these appear
/// only in delta rows that actually changed — rows without `openInterest` are
/// silently skipped so we never emit a bogus 0.
pub fn parse_open_interest(raw: &Value) -> WebSocketResult<StreamEvent> {
    let data = frame_data(raw)?;

    for item in data {
        let symbol = match item.get("symbol").and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let open_interest = match item.get("openInterest").and_then(Value::as_f64) {
            Some(oi) => oi,
            None => continue, // field absent in this delta row — normal for partial updates
        };

        let open_interest_value = item.get("openValue").and_then(Value::as_f64);

        let timestamp = item
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(iso_to_ms)
            .unwrap_or_else(now_ms);

        return Ok(StreamEvent::OpenInterestUpdate {
            symbol,
            open_interest,
            open_interest_value,
            timestamp,
        });
    }

    Err(WebSocketError::FieldAbsent(
        "bitmex instrument: no row contained openInterest".into(),
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
            ..Default::default()
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
            timestamp, ..Default::default() 
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
// REST parsers
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `GET /api/v1/trade` response → `Vec<PublicTrade>`.
///
/// BitMEX REST trade response is a JSON array (not wrapped).
/// Each element: `{timestamp, symbol, side, size, price, trdMatchID, …}`.
/// The `timestamp` field is an ISO-8601 string — converted to epoch ms.
pub fn parse_rest_recent_trades(v: &Value) -> ExchangeResult<Vec<PublicTrade>> {
    let arr = v
        .as_array()
        .ok_or_else(|| ExchangeError::Parse("bitmex recent_trades: expected array".into()))?;

    let trades = arr
        .iter()
        .filter_map(|item| {
            let price = item.get("price")?.as_f64()?;
            let quantity = item.get("size")?.as_f64().unwrap_or(0.0);
            let side = item
                .get("side")
                .and_then(Value::as_str)
                .map(|s| if s == "Buy" { TradeSide::Buy } else { TradeSide::Sell })
                .unwrap_or(TradeSide::Buy);
            let timestamp = item
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(iso_to_ms)
                .unwrap_or(0);
            let id = item
                .get("trdMatchID")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Some(PublicTrade { id, price, quantity, side, timestamp, ..Default::default() })
        })
        .collect();

    Ok(trades)
}

/// Parse `GET /api/v1/trade/bucketed` response → `Vec<Kline>`.
///
/// BitMEX bucket `timestamp` = **end** of the period.
/// `open_time` = `timestamp − bin_size_ms` so callers get the period open time.
///
/// `bin_size_ms` must be the millisecond duration of the requested `binSize`
/// (use `endpoints::bin_size_duration_ms`). Results arrive newest-first
/// when `reverse=true`; this function reverses to oldest-first.
pub fn parse_rest_klines(v: &Value, bin_size_ms: i64) -> ExchangeResult<Vec<Kline>> {
    let arr = v
        .as_array()
        .ok_or_else(|| ExchangeError::Parse("bitmex klines: expected array".into()))?;

    let mut klines: Vec<Kline> = arr
        .iter()
        .filter_map(|item| {
            // timestamp = end of bucket (ISO-8601 string)
            let close_ts = item
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(iso_to_ms)?;
            let open_time = close_ts - bin_size_ms;

            let open  = item.get("open")?.as_f64()?;
            let high  = item.get("high")?.as_f64()?;
            let low   = item.get("low")?.as_f64()?;
            let close = item.get("close")?.as_f64()?;
            // BitMEX "volume" = foreignNotional (USD notional for XBTUSD).
            // homeNotional (BTC) is the volume in base asset.
            let volume = item
                .get("homeNotional")
                .and_then(Value::as_f64)
                .unwrap_or_else(|| item.get("volume").and_then(Value::as_f64).unwrap_or(0.0));
            let quote_volume = item.get("foreignNotional").and_then(Value::as_f64);
            let trades = item.get("trades").and_then(Value::as_u64);

            Some(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                quote_volume,
                close_time: Some(close_ts),
                trades,
                ..Default::default()
            })
        })
        .collect();

    // `reverse=true` → newest first from API; flip to oldest-first for callers.
    klines.reverse();

    Ok(klines)
}

/// Parse `GET /api/v1/funding` response → `Vec<FundingRate>`.
///
/// Each element: `{timestamp, symbol, fundingRate, fundingRateDaily, …}`.
/// The `timestamp` field is an ISO-8601 string — converted to epoch ms.
pub fn parse_rest_funding_rate_history(v: &Value) -> ExchangeResult<Vec<FundingRate>> {
    let arr = v
        .as_array()
        .ok_or_else(|| ExchangeError::Parse("bitmex funding_history: expected array".into()))?;

    let rates = arr
        .iter()
        .filter_map(|item| {
            let rate = item.get("fundingRate")?.as_f64()?;
            let timestamp = item
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(iso_to_ms)
                .unwrap_or(0);
            Some(FundingRate {
                rate,
                next_funding_time: None,
                timestamp, ..Default::default() 
            })
        })
        .collect();

    Ok(rates)
}

/// Parse `GET /api/v1/liquidation` response → `Vec<Liquidation>`.
///
/// BitMEX liquidation endpoint returns `[]` when there are no recent forced
/// closes — that is normal; an empty result is NOT an error.
/// Each element: `{orderID, symbol, side, price, leavesQty}`.
/// NOTE: `side` on a BitMEX liquidation order is the direction of the
/// forced-close (Buy = short being liquidated; Sell = long being liquidated).
/// We invert to `Liquidation.side` convention where `Buy` means a long
/// position was liquidated.
pub fn parse_rest_liquidation_history(v: &Value, symbol: &str) -> ExchangeResult<Vec<Liquidation>> {
    let arr = v
        .as_array()
        .ok_or_else(|| ExchangeError::Parse("bitmex liquidation: expected array".into()))?;

    let liq: Vec<Liquidation> = arr
        .iter()
        .filter_map(|item| {
            let price    = item.get("price")?.as_f64()?;
            let quantity = item.get("leavesQty")?.as_f64().unwrap_or(0.0);
            // BitMEX: side = direction of the liquidation ORDER (opposite to position).
            // "Buy"  liq order → short position was liquidated → our side = Sell.
            // "Sell" liq order → long  position was liquidated → our side = Buy.
            let side = item
                .get("side")
                .and_then(Value::as_str)
                .map(|s| if s == "Buy" { TradeSide::Sell } else { TradeSide::Buy })
                .unwrap_or(TradeSide::Sell);
            // BitMEX liquidation REST rows do NOT carry a timestamp field —
            // the endpoint is intended as a snapshot of the current liquidation
            // queue, not a historical log. We use 0 as sentinel.
            Some(Liquidation {
                symbol: symbol.to_string(),
                side,
                price,
                quantity,
                timestamp: 0,
                value: None, ..Default::default() 
            })
        })
        .collect();

    Ok(liq)
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
    fn parse_open_interest_extracts_oi_and_value() {
        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{
                "symbol": "XBTUSD",
                "openInterest": 123456789_u64,
                "openValue": 8765432100_u64,
                "timestamp": "2024-01-01T12:00:00.000Z"
            }]
        });
        let event = parse_open_interest(&frame).expect("should parse OpenInterestUpdate");
        match event {
            StreamEvent::OpenInterestUpdate { symbol, open_interest, open_interest_value, timestamp } => {
                assert_eq!(symbol, "XBTUSD");
                assert!((open_interest - 123_456_789.0).abs() < 1.0);
                assert!(open_interest_value.is_some());
                assert!((open_interest_value.unwrap() - 8_765_432_100.0).abs() < 1.0);
                assert!(timestamp > 0);
            }
            other => panic!("expected OpenInterestUpdate, got {:?}", other),
        }
    }

    #[test]
    fn parse_open_interest_missing_field_returns_field_absent() {
        // Partial-update frame without openInterest — must not emit OI=0.
        let frame = serde_json::json!({
            "table": "instrument",
            "action": "update",
            "data": [{"symbol": "XBTUSD", "markPrice": 45200.0, "timestamp": "2024-01-01T07:45:00.000Z"}]
        });
        let err = parse_open_interest(&frame).expect_err("should return FieldAbsent");
        assert!(
            matches!(err, WebSocketError::FieldAbsent(_)),
            "expected FieldAbsent, got {:?}", err
        );
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

    // ─── REST parser tests ────────────────────────────────────────────────────

    #[test]
    fn iso_to_ms_parses_bitmex_timestamp_format() {
        // Live payload timestamp format: "2026-06-14T15:20:10.445Z"
        let ms = iso_to_ms("2026-06-14T15:20:10.445Z").expect("must parse");
        // chrono-computed value: 1781450410445 ms
        assert_eq!(ms, 1_781_450_410_445);
        // Sanity: must be > 1.7 trillion (well into 2026)
        assert!(ms > 1_700_000_000_000);
    }

    #[test]
    fn parse_rest_recent_trades_basic() {
        let raw = serde_json::json!([{
            "timestamp": "2026-06-14T15:20:10.445Z",
            "symbol": "XBTUSD",
            "side": "Buy",
            "size": 1300,
            "price": 63933.8,
            "trdMatchID": "00000000-006d-1000-0000-0032e5114348",
            "grossValue": 2033356,
            "homeNotional": 0.02033356,
            "foreignNotional": 1300,
            "trdType": "Regular"
        }]);
        let trades = parse_rest_recent_trades(&raw).expect("should parse trades");
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        assert_eq!(t.id, "00000000-006d-1000-0000-0032e5114348");
        assert!((t.price - 63933.8).abs() < 1e-6);
        assert!((t.quantity - 1300.0).abs() < 1e-6);
        assert_eq!(t.side, TradeSide::Buy);
        assert_eq!(t.timestamp, 1_781_450_410_445);
    }

    #[test]
    fn parse_rest_recent_trades_sell_side() {
        let raw = serde_json::json!([{
            "timestamp": "2026-06-14T15:20:10.000Z",
            "symbol": "XBTUSD",
            "side": "Sell",
            "size": 500,
            "price": 63900.0,
            "trdMatchID": "aabbcc",
        }]);
        let trades = parse_rest_recent_trades(&raw).expect("should parse");
        assert_eq!(trades[0].side, TradeSide::Sell);
    }

    #[test]
    fn parse_rest_klines_open_time_is_bucket_open() {
        // Live bucketed payload: timestamp = END of period.
        // binSize=1m → 60_000 ms; open_time = close_ts − 60_000.
        let raw = serde_json::json!([{
            "timestamp": "2026-06-14T15:20:00.000Z",
            "symbol": "XBTUSD",
            "open": 63996.3,
            "high": 63996.2,
            "low": 63933.4,
            "close": 63940.1,
            "trades": 176,
            "volume": 1664700,
            "vwap": 63944.3428,
            "lastSize": 100,
            "turnover": 2603365749_u64,
            "homeNotional": 26.03,
            "foreignNotional": 1664700
        }]);
        // reverse=true: single item, nothing to flip
        let klines = parse_rest_klines(&raw, 60_000).expect("should parse");
        assert_eq!(klines.len(), 1);
        let k = &klines[0];
        // close timestamp ms for "2026-06-14T15:20:00.000Z"
        let close_ts = iso_to_ms("2026-06-14T15:20:00.000Z").unwrap();
        assert_eq!(k.close_time, Some(close_ts));
        assert_eq!(k.open_time, close_ts - 60_000);
        assert!((k.open  - 63996.3).abs() < 1e-6);
        assert!((k.high  - 63996.2).abs() < 1e-6);
        assert!((k.low   - 63933.4).abs() < 1e-6);
        assert!((k.close - 63940.1).abs() < 1e-6);
        // volume = homeNotional (BTC)
        assert!((k.volume - 26.03).abs() < 1e-6);
        assert_eq!(k.trades, Some(176));
    }

    #[test]
    fn parse_rest_klines_reverses_to_oldest_first() {
        // API returns newest-first (reverse=true). After parsing: oldest-first.
        let raw = serde_json::json!([
            // newer bucket (comes first in API response)
            {"timestamp": "2026-06-14T15:20:00.000Z", "open": 2.0, "high": 2.0, "low": 2.0, "close": 2.0, "homeNotional": 1.0},
            // older bucket
            {"timestamp": "2026-06-14T15:19:00.000Z", "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "homeNotional": 1.0},
        ]);
        let klines = parse_rest_klines(&raw, 60_000).expect("should parse");
        // After reverse: older bucket first
        assert!((klines[0].open - 1.0).abs() < 1e-9);
        assert!((klines[1].open - 2.0).abs() < 1e-9);
    }

    #[test]
    fn parse_rest_funding_rate_history_basic() {
        let raw = serde_json::json!([{
            "timestamp": "2026-06-14T12:00:00.000Z",
            "symbol": "XBTUSD",
            "fundingInterval": "2000-01-01T08:00:00.000Z",
            "fundingRate": -0.000133,
            "fundingRateDaily": -0.000399
        }]);
        let rates = parse_rest_funding_rate_history(&raw).expect("should parse");
        assert_eq!(rates.len(), 1);
        assert!((rates[0].rate - (-0.000133)).abs() < 1e-10);
        assert!(rates[0].timestamp > 0);
    }

    #[test]
    fn parse_rest_liquidation_history_empty_array_is_ok() {
        // BitMEX returns [] when no recent liquidations — must not error.
        let raw = serde_json::json!([]);
        let liq = parse_rest_liquidation_history(&raw, "XBTUSD").expect("empty ok");
        assert!(liq.is_empty());
    }

    #[test]
    fn parse_rest_liquidation_history_side_inversion() {
        // BitMEX "Buy" liq order = short was liquidated → our side = Sell.
        let raw = serde_json::json!([{
            "orderID": "abc",
            "symbol": "XBTUSD",
            "side": "Buy",
            "price": 63000.0,
            "leavesQty": 100
        }]);
        let liq = parse_rest_liquidation_history(&raw, "XBTUSD").expect("should parse");
        assert_eq!(liq.len(), 1);
        assert_eq!(liq[0].side, TradeSide::Sell);
        assert!((liq[0].price - 63000.0).abs() < 1e-6);
    }
}
