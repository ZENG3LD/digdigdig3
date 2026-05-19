//! Dukascopy response parsers
//!
//! Parse binary .bi5 tick data files to domain types.
//!
//! ## Binary Format (.bi5)
//!
//! - Compression: LZMA
//! - Record size: 20 bytes per tick
//! - Byte order: Big-endian
//!
//! ### Record Structure (20 bytes)
//! 1. Timestamp offset (4 bytes, uint32): milliseconds from hour start
//! 2. Ask price raw (4 bytes, uint32): multiply by point value
//! 3. Bid price raw (4 bytes, uint32): multiply by point value
//! 4. Ask volume (4 bytes, float32): base currency volume
//! 5. Bid volume (4 bytes, float32): base currency volume

use crate::core::types::*;
use crate::core::{ExchangeError, ExchangeResult};

/// Raw tick data from binary file
#[derive(Debug, Clone)]
pub struct RawTick {
    /// Milliseconds from hour start (0-3599999)
    pub time_offset_ms: u32,
    /// Raw ask price (multiply by point value)
    pub ask_raw: u32,
    /// Raw bid price (multiply by point value)
    pub bid_raw: u32,
    /// Ask volume (base currency)
    pub ask_volume: f32,
    /// Bid volume (base currency)
    pub bid_volume: f32,
}

/// Parsed tick with actual prices and timestamp
#[derive(Debug, Clone)]
pub struct DukascopyTick {
    /// Unix timestamp in milliseconds
    pub time: i64,
    /// Bid price
    pub bid: f64,
    /// Ask price
    pub ask: f64,
    /// Bid volume
    pub bid_volume: f64,
    /// Ask volume
    pub ask_volume: f64,
}

pub struct DukascopyParser;

impl DukascopyParser {
    // ═══════════════════════════════════════════════════════════════════════
    // BINARY PARSING (.bi5 FILES)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse decompressed binary tick data
    ///
    /// # Arguments
    /// * `data` - Decompressed binary data (LZMA already decoded)
    /// * `hour_start_ms` - Unix timestamp of hour start in milliseconds
    /// * `point_value` - Point value for price conversion (e.g., 0.00001 for EURUSD)
    ///
    /// # Returns
    /// Vector of parsed ticks
    pub fn parse_binary_ticks(
        data: &[u8],
        hour_start_ms: i64,
        point_value: f64,
    ) -> ExchangeResult<Vec<DukascopyTick>> {
        const RECORD_SIZE: usize = 20;

        if !data.len().is_multiple_of(RECORD_SIZE) {
            return Err(ExchangeError::Parse(format!(
                "Invalid data length: {} (must be multiple of {})",
                data.len(),
                RECORD_SIZE
            )));
        }

        let mut ticks = Vec::with_capacity(data.len() / RECORD_SIZE);

        for chunk in data.chunks_exact(RECORD_SIZE) {
            let raw = Self::parse_raw_tick(chunk)?;
            let tick = Self::raw_tick_to_dukascopy_tick(raw, hour_start_ms, point_value);
            ticks.push(tick);
        }

        Ok(ticks)
    }

    /// Parse single 20-byte raw tick record
    fn parse_raw_tick(bytes: &[u8]) -> ExchangeResult<RawTick> {
        if bytes.len() != 20 {
            return Err(ExchangeError::Parse(format!(
                "Invalid tick record size: {} (expected 20)",
                bytes.len()
            )));
        }

        // Parse big-endian values
        let time_offset_ms = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let ask_raw = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let bid_raw = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

        // Parse floats (big-endian)
        let ask_volume = f32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let bid_volume = f32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);

        Ok(RawTick {
            time_offset_ms,
            ask_raw,
            bid_raw,
            ask_volume,
            bid_volume,
        })
    }

    /// Convert raw tick to DukascopyTick with actual prices and timestamp
    fn raw_tick_to_dukascopy_tick(
        raw: RawTick,
        hour_start_ms: i64,
        point_value: f64,
    ) -> DukascopyTick {
        DukascopyTick {
            time: hour_start_ms + raw.time_offset_ms as i64,
            bid: raw.bid_raw as f64 * point_value,
            ask: raw.ask_raw as f64 * point_value,
            bid_volume: raw.bid_volume as f64,
            ask_volume: raw.ask_volume as f64,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVERSION TO DOMAIN TYPES
    // ═══════════════════════════════════════════════════════════════════════

    /// Build Kline (OHLCV candle) from ticks
    ///
    /// Aggregates ticks into a candle with OHLC and volume.
    /// Uses mid price (bid+ask)/2 for OHLC values.
    pub fn ticks_to_kline(
        ticks: &[DukascopyTick],
        open_time: i64,
    ) -> ExchangeResult<Kline> {
        if ticks.is_empty() {
            return Err(ExchangeError::Parse("No ticks to build kline".to_string()));
        }

        let first_mid = (ticks[0].bid + ticks[0].ask) / 2.0;
        let last_mid = (ticks[ticks.len() - 1].bid + ticks[ticks.len() - 1].ask) / 2.0;

        let mut high = f64::MIN;
        let mut low = f64::MAX;
        let mut total_volume = 0.0;

        for tick in ticks {
            let mid = (tick.bid + tick.ask) / 2.0;
            high = high.max(mid);
            low = low.min(mid);
            total_volume += tick.bid_volume + tick.ask_volume;
        }

        Ok(Kline {
            open_time,
            open: first_mid,
            high,
            low,
            close: last_mid,
            volume: total_volume,
            quote_volume: None,
            close_time: Some(ticks[ticks.len() - 1].time),
            trades: Some(ticks.len() as u64),
        })
    }

    /// Build multiple klines from ticks
    ///
    /// Splits ticks into time buckets and creates one kline per bucket.
    ///
    /// # Arguments
    /// * `ticks` - All ticks to process
    /// * `interval_ms` - Interval duration in milliseconds (e.g., 3600000 for 1h)
    pub fn ticks_to_klines(
        ticks: &[DukascopyTick],
        interval_ms: i64,
    ) -> ExchangeResult<Vec<Kline>> {
        if ticks.is_empty() {
            return Ok(Vec::new());
        }

        let mut klines = Vec::new();
        let mut bucket_ticks = Vec::new();
        let mut current_bucket_start = (ticks[0].time / interval_ms) * interval_ms;

        for tick in ticks {
            let tick_bucket = (tick.time / interval_ms) * interval_ms;

            if tick_bucket != current_bucket_start {
                // New bucket - finalize previous
                if !bucket_ticks.is_empty() {
                    klines.push(Self::ticks_to_kline(&bucket_ticks, current_bucket_start)?);
                    bucket_ticks.clear();
                }
                current_bucket_start = tick_bucket;
            }

            bucket_ticks.push(tick.clone());
        }

        // Finalize last bucket
        if !bucket_ticks.is_empty() {
            klines.push(Self::ticks_to_kline(&bucket_ticks, current_bucket_start)?);
        }

        Ok(klines)
    }

    /// Convert tick to Ticker (24h stats would need full day of data)
    ///
    /// Note: This creates a minimal ticker from a single tick.
    /// For full 24h stats, you'd need to aggregate a full day of ticks.
    pub fn tick_to_ticker(tick: &DukascopyTick, symbol: &str) -> Ticker {
        Ticker {
            symbol: symbol.to_string(),
            last_price: (tick.bid + tick.ask) / 2.0,
            bid_price: Some(tick.bid),
            ask_price: Some(tick.ask),
            high_24h: None, // Would need full day of data
            low_24h: None,  // Would need full day of data
            volume_24h: Some(tick.bid_volume + tick.ask_volume),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: tick.time,
        }
    }

    /// Get latest price from ticks
    pub fn get_latest_price(ticks: &[DukascopyTick]) -> ExchangeResult<f64> {
        ticks
            .last()
            .map(|tick| (tick.bid + tick.ask) / 2.0)
            .ok_or_else(|| ExchangeError::Parse("No ticks available".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse interval string to milliseconds
    ///
    /// Supported: "1m", "5m", "15m", "30m", "1h", "4h", "1d"
    pub fn parse_interval_to_ms(interval: &str) -> ExchangeResult<i64> {
        let interval_lower = interval.to_lowercase();

        let ms = match interval_lower.as_str() {
            "1m" | "1min" => 60_000,
            "5m" | "5min" => 300_000,
            "10m" | "10min" => 600_000,
            "15m" | "15min" => 900_000,
            "30m" | "30min" => 1_800_000,
            "1h" | "1hour" => 3_600_000,
            "4h" | "4hour" => 14_400_000,
            "1d" | "1day" => 86_400_000,
            _ => {
                return Err(ExchangeError::Parse(format!(
                    "Unsupported interval: {}",
                    interval
                )))
            }
        };

        Ok(ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_raw_tick() {
        // Simulated 20-byte tick record (big-endian)
        let bytes: [u8; 20] = [
            0x00, 0x00, 0x00, 0x64, // time_offset_ms = 100
            0x00, 0x01, 0xB5, 0x07, // ask_raw = 111879
            0x00, 0x01, 0xB4, 0xF3, // bid_raw = 111859
            0x49, 0x74, 0x24, 0x00, // ask_volume = 1000000.0 (approx)
            0x49, 0x74, 0x24, 0x00, // bid_volume = 1000000.0 (approx)
        ];

        let raw = DukascopyParser::parse_raw_tick(&bytes).unwrap();

        assert_eq!(raw.time_offset_ms, 100);
        assert_eq!(raw.ask_raw, 111879);
        assert_eq!(raw.bid_raw, 111859);
    }

    #[test]
    fn test_raw_tick_to_dukascopy_tick() {
        let raw = RawTick {
            time_offset_ms: 1000,
            ask_raw: 112347,
            bid_raw: 112345,
            ask_volume: 1500000.0,
            bid_volume: 1200000.0,
        };

        let hour_start_ms = 1234567800000; // Some hour start
        let point_value = 0.00001; // EURUSD

        let tick = DukascopyParser::raw_tick_to_dukascopy_tick(raw, hour_start_ms, point_value);

        assert_eq!(tick.time, 1234567801000); // hour_start + 1000ms
        assert!((tick.ask - 1.12347).abs() < 0.00001);
        assert!((tick.bid - 1.12345).abs() < 0.00001);
        assert_eq!(tick.ask_volume, 1500000.0);
        assert_eq!(tick.bid_volume, 1200000.0);
    }

    #[test]
    fn test_ticks_to_kline() {
        let ticks = vec![
            DukascopyTick {
                time: 1000,
                bid: 1.1000,
                ask: 1.1002,
                bid_volume: 100.0,
                ask_volume: 100.0,
            },
            DukascopyTick {
                time: 2000,
                bid: 1.1010,
                ask: 1.1012,
                bid_volume: 150.0,
                ask_volume: 150.0,
            },
            DukascopyTick {
                time: 3000,
                bid: 1.0990,
                ask: 1.0992,
                bid_volume: 120.0,
                ask_volume: 120.0,
            },
            DukascopyTick {
                time: 4000,
                bid: 1.1005,
                ask: 1.1007,
                bid_volume: 130.0,
                ask_volume: 130.0,
            },
        ];

        let kline = DukascopyParser::ticks_to_kline(&ticks, 1000).unwrap();

        assert_eq!(kline.open_time, 1000);
        assert!((kline.open - 1.1001).abs() < 0.0001); // (1.1000 + 1.1002) / 2
        assert!((kline.close - 1.1006).abs() < 0.0001); // (1.1005 + 1.1007) / 2
        assert!(kline.high >= 1.1011); // Should include highest mid
        assert!(kline.low <= 1.0991); // Should include lowest mid
        assert_eq!(kline.volume, 1000.0); // Sum of all volumes
        assert_eq!(kline.trades, Some(4)); // 4 ticks
    }

    #[test]
    fn test_parse_interval_to_ms() {
        assert_eq!(DukascopyParser::parse_interval_to_ms("1m").unwrap(), 60_000);
        assert_eq!(DukascopyParser::parse_interval_to_ms("5m").unwrap(), 300_000);
        assert_eq!(DukascopyParser::parse_interval_to_ms("1h").unwrap(), 3_600_000);
        assert_eq!(
            DukascopyParser::parse_interval_to_ms("1d").unwrap(),
            86_400_000
        );

        // Case insensitive
        assert_eq!(DukascopyParser::parse_interval_to_ms("1H").unwrap(), 3_600_000);

        // Error case
        assert!(DukascopyParser::parse_interval_to_ms("99x").is_err());
    }

    #[test]
    fn test_tick_to_ticker() {
        let tick = DukascopyTick {
            time: 1234567890000,
            bid: 1.12345,
            ask: 1.12347,
            bid_volume: 1500000.0,
            ask_volume: 1200000.0,
        };

        let ticker = DukascopyParser::tick_to_ticker(&tick, "EURUSD");

        assert_eq!(ticker.symbol, "EURUSD");
        assert!((ticker.last_price - 1.12346).abs() < 0.00001);
        assert_eq!(ticker.bid_price, Some(1.12345));
        assert_eq!(ticker.ask_price, Some(1.12347));
        assert_eq!(ticker.timestamp, 1234567890000);
    }
}
