//! # MOEX ISS Response Parsers
//!
//! Parse JSON responses from MOEX ISS API to domain types.
//!
//! ## MOEX Response Structure
//! MOEX ISS has a unique response format with named sections (blocks):
//! - Each response contains multiple named sections
//! - Each section has: `metadata`, `columns`, `data`
//! - Data is array of arrays (not objects)
//! - Column order matters - use `columns` array to map values
//!
//! Example:
//! ```json
//! {
//!   "securities": {
//!     "metadata": {...},
//!     "columns": ["SECID", "SHORTNAME", "LAST"],
//!     "data": [
//!       ["SBER", "Сбербанк", 306.75],
//!       ["GAZP", "ГАЗПРОМ", 129.0]
//!     ]
//!   },
//!   "marketdata": {
//!     "columns": ["SECID", "BID", "ASK"],
//!     "data": [...]
//!   }
//! }
//! ```

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use crate::core::types::{Kline, Ticker, OrderBook, OrderBookLevel};

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, String>;

/// MOEX ISS response parser
pub struct MoexParser;

impl MoexParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // CORE HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Extract a named block from MOEX response
    ///
    /// MOEX responses have structure: `{ "blockname": { "columns": [...], "data": [...] } }`
    fn get_block<'a>(response: &'a Value, block_name: &str) -> ParseResult<&'a Value> {
        response
            .get(block_name)
            .ok_or_else(|| format!("Missing '{}' block", block_name))
    }

    /// Get columns array from a block
    fn get_columns(block: &Value) -> ParseResult<&Vec<Value>> {
        block
            .get("columns")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Missing 'columns' array".to_string())
    }

    /// Get data array from a block
    fn get_data(block: &Value) -> ParseResult<&Vec<Value>> {
        block
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Missing 'data' array".to_string())
    }

    /// Find column index by name
    fn find_column_index(columns: &[Value], name: &str) -> Option<usize> {
        columns.iter().position(|col| col.as_str() == Some(name))
    }

    /// Get value from row by column name
    fn get_value<'a>(row: &'a Value, columns: &[Value], column: &str) -> Option<&'a Value> {
        let row_array = row.as_array()?;
        let index = Self::find_column_index(columns, column)?;
        row_array.get(index)
    }

    /// Parse f64 from value (handles both number and string)
    fn parse_f64(value: &Value) -> Option<f64> {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
    }

    /// Parse timestamp from MOEX datetime string
    ///
    /// MOEX formats:
    /// - DateTime: "2026-01-26 19:00:01"
    /// - Date only: "2026-01-26" (interpreted as midnight UTC)
    fn parse_timestamp(datetime_str: &str) -> Option<i64> {
        // Try full datetime first: "YYYY-MM-DD HH:MM:SS"
        if let Ok(ndt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
            return Some(Utc.from_utc_datetime(&ndt).timestamp_millis());
        }
        // Fall back to date only: "YYYY-MM-DD" → midnight UTC
        if let Ok(nd) = NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
            let ndt = nd.and_hms_opt(0, 0, 0)?;
            return Some(Utc.from_utc_datetime(&ndt).timestamp_millis());
        }
        None
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PRICE PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse current price from MOEX response
    ///
    /// Expected block: "marketdata"
    /// Required column: "LAST"
    pub fn parse_price(response: &Value) -> ParseResult<f64> {
        let block = Self::get_block(response, "marketdata")?;
        let columns = Self::get_columns(block)?;
        let data = Self::get_data(block)?;

        let first_row = data.first().ok_or("Empty data array")?;
        let last_value = Self::get_value(first_row, columns, "LAST")
            .ok_or("Missing 'LAST' column")?;

        Self::parse_f64(last_value)
            .ok_or_else(|| "Invalid LAST price value".to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TICKER PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ticker from MOEX response
    ///
    /// Expected blocks:
    /// - "securities" - for symbol info
    /// - "marketdata" - for price and volume data
    pub fn parse_ticker(response: &Value, symbol: &str) -> ParseResult<Ticker> {
        let marketdata = Self::get_block(response, "marketdata")?;
        let columns = Self::get_columns(marketdata)?;
        let data = Self::get_data(marketdata)?;

        let row = data.first().ok_or("Empty marketdata")?;

        // Extract values with safe fallbacks
        let last_price = Self::get_value(row, columns, "LAST")
            .and_then(Self::parse_f64)
            .ok_or("Missing LAST price")?;

        let bid_price = Self::get_value(row, columns, "BID")
            .and_then(Self::parse_f64);

        let ask_price = Self::get_value(row, columns, "ASK")
            .and_then(Self::parse_f64);

        let high_24h = Self::get_value(row, columns, "HIGH")
            .and_then(Self::parse_f64);

        let low_24h = Self::get_value(row, columns, "LOW")
            .and_then(Self::parse_f64);

        let volume_24h = Self::get_value(row, columns, "VOLUME")
            .and_then(|v| v.as_i64())
            .map(|v| v as f64);

        let value = Self::get_value(row, columns, "VALUE")
            .and_then(Self::parse_f64);

        let change = Self::get_value(row, columns, "LASTCHANGE")
            .and_then(Self::parse_f64);

        let change_pct = Self::get_value(row, columns, "LASTCHANGEPRCNT")
            .and_then(Self::parse_f64);

        // Parse timestamp from SYSTIME or UPDATETIME
        let timestamp = Self::get_value(row, columns, "SYSTIME")
            .or_else(|| Self::get_value(row, columns, "UPDATETIME"))
            .and_then(|v| v.as_str())
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
            quote_volume_24h: value,
            price_change_24h: change,
            price_change_percent_24h: change_pct,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // KLINE/CANDLE PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse klines/candles from MOEX response
    ///
    /// Expected block: "candles"
    /// Required columns: "open", "close", "high", "low", "volume", "begin", "end"
    pub fn parse_klines(response: &Value) -> ParseResult<Vec<Kline>> {
        let block = Self::get_block(response, "candles")?;
        let columns = Self::get_columns(block)?;
        let data = Self::get_data(block)?;

        data.iter()
            .map(|row| {
                let open = Self::get_value(row, columns, "open")
                    .and_then(Self::parse_f64)
                    .ok_or("Missing open")?;

                let high = Self::get_value(row, columns, "high")
                    .and_then(Self::parse_f64)
                    .ok_or("Missing high")?;

                let low = Self::get_value(row, columns, "low")
                    .and_then(Self::parse_f64)
                    .ok_or("Missing low")?;

                let close = Self::get_value(row, columns, "close")
                    .and_then(Self::parse_f64)
                    .ok_or("Missing close")?;

                let volume = Self::get_value(row, columns, "volume")
                    .and_then(Self::parse_f64)
                    .ok_or("Missing volume")?;

                let quote_volume = Self::get_value(row, columns, "value")
                    .and_then(Self::parse_f64);

                // Parse begin timestamp
                let open_time = Self::get_value(row, columns, "begin")
                    .and_then(|v| v.as_str())
                    .and_then(Self::parse_timestamp)
                    .ok_or("Missing begin timestamp")?;

                // Parse end timestamp
                let close_time = Self::get_value(row, columns, "end")
                    .and_then(|v| v.as_str())
                    .and_then(Self::parse_timestamp);

                Ok(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    quote_volume,
                    close_time,
                    trades: None,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ORDERBOOK PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse orderbook from MOEX response
    ///
    /// Note: Orderbook requires paid subscription.
    /// Expected block: "orderbook"
    pub fn parse_orderbook(response: &Value) -> ParseResult<OrderBook> {
        let block = Self::get_block(response, "orderbook")?;
        let columns = Self::get_columns(block)?;
        let data = Self::get_data(block)?;

        // MOEX orderbook structure may vary
        // This is a placeholder implementation
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for row in data {
            let side = Self::get_value(row, columns, "BUYSELL")
                .and_then(|v| v.as_str());

            let price = Self::get_value(row, columns, "PRICE")
                .and_then(Self::parse_f64)
                .ok_or("Missing price")?;

            let quantity = Self::get_value(row, columns, "QUANTITY")
                .and_then(Self::parse_f64)
                .ok_or("Missing quantity")?;

            match side {
                Some("B") => bids.push(OrderBookLevel::new(price, quantity)),
                Some("S") => asks.push(OrderBookLevel::new(price, quantity)),
                _ => {}
            }
        }

        // Sort: bids descending, asks ascending
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).expect("f64 comparison should not return None"));
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).expect("f64 comparison should not return None"));

        let timestamp = chrono::Utc::now().timestamp_millis();

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

    // ═══════════════════════════════════════════════════════════════════════════
    // SYMBOLS PARSING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse symbols list from MOEX response
    ///
    /// Expected block: "securities"
    /// Required column: "SECID"
    pub fn parse_symbols(response: &Value) -> ParseResult<Vec<String>> {
        let block = Self::get_block(response, "securities")?;
        let columns = Self::get_columns(block)?;
        let data = Self::get_data(block)?;

        Ok(data
            .iter()
            .filter_map(|row| {
                Self::get_value(row, columns, "SECID")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "marketdata": {
                "columns": ["SECID", "LAST", "BID", "ASK"],
                "data": [
                    ["SBER", 306.75, 306.74, 306.76]
                ]
            }
        });

        let price = MoexParser::parse_price(&response).unwrap();
        assert_eq!(price, 306.75);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "marketdata": {
                "columns": ["SECID", "LAST", "BID", "ASK", "HIGH", "LOW", "VOLUME", "LASTCHANGE", "LASTCHANGEPRCNT", "SYSTIME"],
                "data": [
                    ["SBER", 306.75, 306.74, 306.76, 307.35, 305.12, 4800000, -0.13, -0.04, "2026-01-26 19:00:01"]
                ]
            }
        });

        let ticker = MoexParser::parse_ticker(&response, "SBER").unwrap();
        assert_eq!(ticker.symbol, "SBER");
        assert_eq!(ticker.last_price, 306.75);
        assert_eq!(ticker.bid_price, Some(306.74));
        assert_eq!(ticker.ask_price, Some(306.76));
    }

    #[test]
    fn test_parse_symbols() {
        let response = json!({
            "securities": {
                "columns": ["SECID", "SHORTNAME"],
                "data": [
                    ["SBER", "Сбербанк"],
                    ["GAZP", "ГАЗПРОМ"],
                    ["LKOH", "ЛУКОЙЛ"]
                ]
            }
        });

        let symbols = MoexParser::parse_symbols(&response).unwrap();
        assert_eq!(symbols, vec!["SBER", "GAZP", "LKOH"]);
    }
}
