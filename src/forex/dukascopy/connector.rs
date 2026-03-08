//! Dukascopy connector implementation
//!
//! Data provider connector that downloads and parses binary tick data files.

use async_trait::async_trait;
use chrono::{Datelike, Timelike};

use crate::core::{
    ExchangeId, ExchangeType, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Kline, Ticker, OrderBook, FundingRate,
    Order, Balance, AccountInfo, Position,
    HttpClient,
};
use crate::core::traits::{ExchangeIdentity, MarketData, Trading, Account, Positions};

use super::endpoints::{DukascopyUrls, build_tick_data_url, format_symbol, get_point_value};
use super::auth::DukascopyAuth;
use super::parser::{DukascopyParser, DukascopyTick};

/// Dukascopy connector
///
/// Downloads and parses binary .bi5 tick data files from Dukascopy datafeed.
pub struct DukascopyConnector {
    /// HTTP client for downloading binary files
    http: HttpClient,
    /// Auth (no-op for Dukascopy - public datafeed)
    _auth: DukascopyAuth,
    /// URLs
    _urls: DukascopyUrls,
}

impl DukascopyConnector {
    /// Create new connector
    pub fn new() -> Self {
        Self {
            http: HttpClient::new(30_000)
                .expect("Critical: HttpClient::new should never fail with valid timeout"),
            _auth: DukascopyAuth::new(),
            _urls: DukascopyUrls::default(),
        }
    }

    /// Create connector from environment (same as new - no auth needed)
    pub fn from_env() -> Self {
        Self::new()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INTERNAL HELPERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Download and decompress .bi5 file
    ///
    /// Returns empty Vec for hours with no data (weekends, holidays, future dates).
    /// Dukascopy returns HTTP 200 with 0 bytes for these periods.
    async fn download_tick_file(
        &self,
        symbol: &str,
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
    ) -> ExchangeResult<Vec<u8>> {
        let url = build_tick_data_url(symbol, year, month, day, hour);

        // Download compressed file
        let compressed = self.http.get_bytes(&url).await
            .map_err(|e| ExchangeError::Network(format!("Failed to download {}: {}", url, e)))?;

        // Empty response = no data for this hour (weekend, holiday, future)
        if compressed.is_empty() {
            return Ok(Vec::new());
        }

        // LZMA header is at least 13 bytes (5 props + 8 uncompressed size)
        if compressed.len() < 13 {
            return Err(ExchangeError::Parse(format!(
                "Invalid .bi5 file from {}: too small ({} bytes), expected LZMA data",
                url, compressed.len()
            )));
        }

        // Decompress LZMA
        let mut decompressed = Vec::new();
        lzma_rs::lzma_decompress(&mut compressed.as_slice(), &mut decompressed)
            .map_err(|e| ExchangeError::Parse(format!("LZMA decompression failed for {}: {}", url, e)))?;
        Ok(decompressed)
    }

    /// Get ticks for a specific hour
    ///
    /// Returns empty Vec for hours with no data (weekends, holidays).
    async fn get_hour_ticks(
        &self,
        symbol: &str,
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
    ) -> ExchangeResult<Vec<DukascopyTick>> {
        let data = self.download_tick_file(symbol, year, month, day, hour).await?;

        // No data for this hour (weekend, holiday, future)
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate hour start timestamp
        let hour_start_ms = chrono::NaiveDate::from_ymd_opt(year as i32, month + 1, day)
            .and_then(|date| date.and_hms_opt(hour, 0, 0))
            .map(|dt| dt.and_utc().timestamp_millis())
            .ok_or_else(|| ExchangeError::Parse("Invalid date/time".to_string()))?;

        let point_value = get_point_value(symbol);
        DukascopyParser::parse_binary_ticks(&data, hour_start_ms, point_value)
    }

    /// Get ticks for a time range (multiple hours)
    async fn get_ticks_range(
        &self,
        symbol: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> ExchangeResult<Vec<DukascopyTick>> {
        use chrono::{DateTime, Datelike, Timelike};

        let from = DateTime::from_timestamp_millis(from_ms)
            .ok_or_else(|| ExchangeError::Parse("Invalid from timestamp".to_string()))?;
        let to = DateTime::from_timestamp_millis(to_ms)
            .ok_or_else(|| ExchangeError::Parse("Invalid to timestamp".to_string()))?;

        let mut all_ticks = Vec::new();
        let mut current = from;

        // Download hour by hour
        while current <= to {
            let year = current.year() as u32;
            let month = current.month() - 1; // 0-indexed
            let day = current.day();
            let hour = current.hour();

            match self.get_hour_ticks(symbol, year, month, day, hour).await {
                Ok(mut ticks) => {
                    // Filter ticks within range
                    ticks.retain(|tick| tick.time >= from_ms && tick.time <= to_ms);
                    all_ticks.extend(ticks);
                }
                Err(e) => {
                    // Some hours may not have data - this is OK
                    eprintln!("Warning: Failed to get ticks for {}-{:02}-{:02} {:02}:00: {}",
                        year, month + 1, day, hour, e);
                }
            }

            // Move to next hour
            current += chrono::Duration::hours(1);
        }

        if all_ticks.is_empty() {
            return Err(ExchangeError::Api {
                code: 404,
                message: "No tick data available for time range".to_string(),
            });
        }

        Ok(all_ticks)
    }
}

impl Default for DukascopyConnector {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DukascopyConnector {
    fn exchange_name(&self) -> &'static str {
        "dukascopy"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Dukascopy
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DataProvider
    }

    fn is_testnet(&self) -> bool {
        false // No testnet - public datafeed
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::Spot] // Forex spot data
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for DukascopyConnector {
    /// Get current price (from latest available tick data)
    ///
    /// Note: This returns the most recent tick from the last complete hour.
    /// For true real-time prices, use JForex SDK or FIX API.
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<f64> {
        let symbol_str = format_symbol(&symbol);

        // Try to find the most recent hour with data (backtrack up to 72 hours for weekends)
        let now = chrono::Utc::now();
        for hours_back in 1..=72 {
            let target = now - chrono::Duration::hours(hours_back);
            let year = target.year() as u32;
            let month = target.month() - 1;
            let day = target.day();
            let hour = target.hour();

            match self.get_hour_ticks(&symbol_str, year, month, day, hour).await {
                Ok(ticks) if !ticks.is_empty() => {
                    return DukascopyParser::get_latest_price(&ticks);
                }
                _ => continue, // No data for this hour, try earlier
            }
        }

        Err(ExchangeError::Api {
            code: 404,
            message: format!("No tick data found for {} in last 72 hours", symbol_str),
        })
    }

    /// Get ticker (24h stats from last 24 hours of tick data)
    ///
    /// Note: This requires downloading 24 hours of data, which may be slow.
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let symbol_str = format_symbol(&symbol);

        // Try to find the most recent hour with data (backtrack up to 72 hours for weekends)
        let now = chrono::Utc::now();
        for hours_back in 1..=72 {
            let target = now - chrono::Duration::hours(hours_back);
            let year = target.year() as u32;
            let month = target.month() - 1;
            let day = target.day();
            let hour = target.hour();

            match self.get_hour_ticks(&symbol_str, year, month, day, hour).await {
                Ok(ticks) if !ticks.is_empty() => {
                    if let Some(last_tick) = ticks.last() {
                        return Ok(DukascopyParser::tick_to_ticker(last_tick, &symbol_str));
                    }
                }
                _ => continue, // No data for this hour, try earlier
            }
        }

        Err(ExchangeError::Api {
            code: 404,
            message: format!("No tick data found for {} in last 72 hours", symbol_str),
        })
    }

    /// Get orderbook (NOT SUPPORTED - tick data only)
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy provides tick data only - no orderbook via binary downloads. Use JForex SDK for orderbook.".to_string()
        ))
    }

    /// Get klines (constructed from tick data)
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol_str = format_symbol(&symbol);
        let interval_ms = DukascopyParser::parse_interval_to_ms(interval)?;

        // Calculate time range based on limit
        let now = chrono::Utc::now();
        let limit_count = limit.unwrap_or(100) as i64;
        let from_ms = now.timestamp_millis() - (interval_ms * limit_count);
        let to_ms = now.timestamp_millis();

        // Get ticks
        let ticks = self.get_ticks_range(&symbol_str, from_ms, to_ms).await?;

        // Convert to klines
        let klines = DukascopyParser::ticks_to_klines(&ticks, interval_ms)?;

        // Apply limit
        let start = if klines.len() > limit_count as usize {
            klines.len() - limit_count as usize
        } else {
            0
        };

        Ok(klines[start..].to_vec())
    }

    /// Ping (always succeeds - no server ping for binary downloads)
    async fn ping(&self) -> ExchangeResult<()> {
        // Binary downloads don't have a ping endpoint
        // We could test connectivity by downloading a tiny file, but for now just return OK
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for DukascopyConnector {
    async fn market_order(
        &self,
        _symbol: Symbol,
        _side: crate::core::types::OrderSide,
        _quantity: crate::core::types::Quantity,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - trading not supported via binary datafeed. Use JForex SDK or FIX API for trading.".to_string()
        ))
    }

    async fn limit_order(
        &self,
        _symbol: Symbol,
        _side: crate::core::types::OrderSide,
        _quantity: crate::core::types::Quantity,
        _price: crate::core::types::Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: Symbol,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for DukascopyConnector {
    async fn get_balance(&self, _asset: Option<Asset>, _account_type: AccountType) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - account operations not supported".to_string()
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - account operations not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (NOT SUPPORTED - DATA PROVIDER ONLY)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for DukascopyConnector {
    async fn get_positions(
        &self,
        _symbol: Option<Symbol>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available - Dukascopy is forex spot, not perpetual futures".to_string()
        ))
    }

    async fn set_leverage(
        &self,
        _symbol: Symbol,
        _leverage: u32,
        _account_type: AccountType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Dukascopy is a data provider - leverage not applicable".to_string()
        ))
    }
}
