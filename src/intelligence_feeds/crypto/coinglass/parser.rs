//! # Coinglass Response Parser
//!
//! Парсинг JSON ответов от Coinglass API V4.
//!
//! Coinglass специализируется на derivatives analytics,
//! поэтому data structures отличаются от обычных MarketData типов.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTOM DATA STRUCTURES FOR DERIVATIVES ANALYTICS
// ═══════════════════════════════════════════════════════════════════════════════

/// Standard Coinglass API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinglassResponse<T> {
    pub code: String,
    pub msg: String,
    pub success: bool,
    #[serde(default)]
    pub data: Option<T>,
}

/// Liquidation event data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidationData {
    pub t: i64,                      // timestamp (seconds)
    pub symbol: String,              // "BTC", "ETH", etc.
    pub side: String,                // "long" or "short"
    pub price: String,               // liquidation price
    pub quantity: String,            // liquidation quantity
    pub value_usd: String,           // liquidation value in USD
    #[serde(default)]
    pub exchange: Option<String>,    // exchange name
}

/// Open Interest OHLC data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenInterestOhlc {
    pub t: i64,       // timestamp (seconds)
    pub o: String,    // open
    pub h: String,    // high
    pub l: String,    // low
    pub c: String,    // close
}

/// Funding Rate data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FundingRateData {
    pub t: i64,                      // timestamp (seconds)
    pub symbol: String,              // "BTC", "ETH", etc.
    pub exchange: String,            // exchange name
    pub funding_rate: String,        // current funding rate
    #[serde(default)]
    pub next_funding_time: Option<i64>, // next funding timestamp
}

/// Long/Short Ratio data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LongShortRatio {
    pub t: i64,                      // timestamp (seconds)
    pub long_rate: String,           // long ratio (0-1)
    pub short_rate: String,          // short ratio (0-1)
    #[serde(default)]
    pub long_account: Option<String>, // long account count
    #[serde(default)]
    pub short_account: Option<String>, // short account count
}

/// Supported coins response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupportedCoins {
    pub coins: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// MARKET DISCOVERY
// ─────────────────────────────────────────────────────────────────────────────

/// Exchange pairs market info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangePairInfo {
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub instrument_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Coins market info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinMarketInfo {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Pairs market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PairsMarketData {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// LIQUIDATIONS
// ─────────────────────────────────────────────────────────────────────────────

/// Liquidation heatmap data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidationHeatmapPoint {
    pub t: i64,
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Liquidation map entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidationMapEntry {
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(default)]
    pub liq_value: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Liquidation max pain data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidationMaxPainData {
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(default)]
    pub max_pain_price: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// OPEN INTEREST
// ─────────────────────────────────────────────────────────────────────────────

/// Open Interest history data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenInterestHistory {
    pub t: i64,
    #[serde(default)]
    pub open_interest: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Open Interest volume ratio
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenInterestVolRatio {
    pub t: i64,
    #[serde(default)]
    pub oi: Option<Value>,
    #[serde(default)]
    pub vol: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// FUNDING RATES
// ─────────────────────────────────────────────────────────────────────────────

/// Current funding rate per exchange/symbol
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentFundingRate {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(default)]
    pub funding_rate: Option<Value>,
    #[serde(default)]
    pub next_funding_time: Option<i64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Aggregated (OHLC) funding rate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FundingRateAggregated {
    pub t: i64,
    #[serde(default)]
    pub o: Option<Value>,
    #[serde(default)]
    pub h: Option<Value>,
    #[serde(default)]
    pub l: Option<Value>,
    #[serde(default)]
    pub c: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// LONG/SHORT
// ─────────────────────────────────────────────────────────────────────────────

/// Top long/short position ratio data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopLongShortRatio {
    pub t: i64,
    #[serde(default)]
    pub long_rate: Option<Value>,
    #[serde(default)]
    pub short_rate: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Taker buy/sell volume
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TakerBuySellVolume {
    pub t: i64,
    #[serde(default)]
    pub buy_vol: Option<Value>,
    #[serde(default)]
    pub sell_vol: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// ORDER BOOK ANALYTICS
// ─────────────────────────────────────────────────────────────────────────────

/// Orderbook heatmap data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderbookHeatmapPoint {
    pub t: i64,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Large order entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LargeOrder {
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(default)]
    pub quantity: Option<Value>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Bid/ask range data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BidAskRange {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub exchange: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// VOLUME & FLOWS
// ─────────────────────────────────────────────────────────────────────────────

/// Cumulative Volume Delta data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CvdPoint {
    pub t: i64,
    #[serde(default)]
    pub cvd: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Net flow indicator data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetFlowPoint {
    pub t: i64,
    #[serde(default)]
    pub net_flow: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Footprint chart data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FootprintPoint {
    pub t: i64,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// OPTIONS
// ─────────────────────────────────────────────────────────────────────────────

/// Options max pain data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionsMaxPain {
    #[serde(default)]
    pub expiry: Option<String>,
    #[serde(default)]
    pub max_pain_price: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Options OI history data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionsOiHistory {
    pub t: i64,
    #[serde(default)]
    pub oi: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Options volume history data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionsVolumeHistory {
    pub t: i64,
    #[serde(default)]
    pub volume: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// ON-CHAIN
// ─────────────────────────────────────────────────────────────────────────────

/// Exchange reserve data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeReserve {
    pub t: i64,
    #[serde(default)]
    pub reserve: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Exchange balance history data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeBalanceHistory {
    pub t: i64,
    #[serde(default)]
    pub balance: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// ERC-20 transfer event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Erc20Transfer {
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub amount: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Whale transfer event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WhaleTransfer {
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub amount: Option<Value>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Token unlock event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenUnlock {
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub amount: Option<Value>,
    #[serde(default)]
    pub value_usd: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Token vesting entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenVesting {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub total_supply: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// ETF
// ─────────────────────────────────────────────────────────────────────────────

/// ETF daily flow data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EtfFlowData {
    pub t: i64,
    #[serde(default)]
    pub net_flow: Option<Value>,
    #[serde(default)]
    pub total_assets: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Grayscale premium data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrayscalePremiumData {
    pub t: i64,
    #[serde(default)]
    pub premium: Option<Value>,
    #[serde(default)]
    pub nav: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// HYPERLIQUID
// ─────────────────────────────────────────────────────────────────────────────

/// HyperLiquid whale alert
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HlWhaleAlert {
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub size: Option<Value>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// HyperLiquid whale position
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HlWhalePosition {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub size: Option<Value>,
    #[serde(default)]
    pub entry_price: Option<Value>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// HyperLiquid wallet positions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HlWalletPosition {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub size: Option<Value>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub unrealized_pnl: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// HyperLiquid position distribution bucket
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HlPositionDistribution {
    #[serde(default)]
    pub price_range_start: Option<Value>,
    #[serde(default)]
    pub price_range_end: Option<Value>,
    #[serde(default)]
    pub count: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ─────────────────────────────────────────────────────────────────────────────
// TECHNICAL INDICATORS
// ─────────────────────────────────────────────────────────────────────────────

/// RSI data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RsiData {
    pub t: i64,
    #[serde(default)]
    pub rsi: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Moving Average data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovingAverageData {
    pub t: i64,
    #[serde(default)]
    pub ma: Option<Value>,
    #[serde(default)]
    pub price: Option<Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// Парсер ответов Coinglass API
pub struct CoinglassParser;

impl CoinglassParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response is successful
    pub fn is_success(response: &Value) -> bool {
        response.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Extract error message from response
    pub fn extract_error(response: &Value) -> String {
        response.get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string()
    }

    /// Extract data field from response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        if !Self::is_success(response) {
            let error_msg = Self::extract_error(response);
            let error_code = response.get("code")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            return Err(ExchangeError::Api {
                code: error_code,
                message: error_msg,
            });
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Parse f64 from string or number
    fn _parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Get f64 from field
    fn _get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::_parse_f64)
    }

    /// Get string from field
    fn _get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Get i64 from field
    fn _get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_i64())
            .or_else(|| data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUPPORTED COINS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse supported coins list
    pub fn parse_supported_coins(response: &Value) -> ExchangeResult<Vec<String>> {
        let data = Self::extract_data(response)?;

        // Data can be an array directly or wrapped in an object
        let coins_array = if let Some(arr) = data.as_array() {
            arr
        } else if let Some(obj) = data.as_object() {
            // Try to find array in object fields
            obj.values()
                .find_map(|v| v.as_array())
                .ok_or_else(|| ExchangeError::Parse("No array found in data".to_string()))?
        } else {
            return Err(ExchangeError::Parse("Data is not an array or object".to_string()));
        };

        let coins = coins_array
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(coins)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LIQUIDATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse liquidation history
    pub fn parse_liquidations(response: &Value) -> ExchangeResult<Vec<LiquidationData>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let liquidations: Vec<LiquidationData> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse liquidations: {}", e)))?;

        Ok(liquidations)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OPEN INTEREST
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse Open Interest OHLC data
    pub fn parse_oi_ohlc(response: &Value) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let oi_data: Vec<OpenInterestOhlc> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse OI OHLC: {}", e)))?;

        Ok(oi_data)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING RATES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding rate history
    pub fn parse_funding_rates(response: &Value) -> ExchangeResult<Vec<FundingRateData>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let funding_rates: Vec<FundingRateData> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse funding rates: {}", e)))?;

        Ok(funding_rates)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LONG/SHORT RATIOS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse long/short ratio history
    pub fn parse_long_short_ratio(response: &Value) -> ExchangeResult<Vec<LongShortRatio>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let ratios: Vec<LongShortRatio> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse long/short ratios: {}", e)))?;

        Ok(ratios)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse supported exchange pairs list
    pub fn parse_exchange_pairs(response: &Value) -> ExchangeResult<Vec<ExchangePairInfo>> {
        Self::parse_array(response)
    }

    /// Parse pairs markets data
    pub fn parse_pairs_markets(response: &Value) -> ExchangeResult<Vec<PairsMarketData>> {
        Self::parse_array(response)
    }

    /// Parse coins markets data
    pub fn parse_coins_markets(response: &Value) -> ExchangeResult<Vec<CoinMarketInfo>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LIQUIDATIONS (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse liquidation heatmap
    pub fn parse_liquidation_heatmap(response: &Value) -> ExchangeResult<Vec<LiquidationHeatmapPoint>> {
        Self::parse_array(response)
    }

    /// Parse liquidation map
    pub fn parse_liquidation_map(response: &Value) -> ExchangeResult<Vec<LiquidationMapEntry>> {
        Self::parse_array(response)
    }

    /// Parse liquidation max pain
    pub fn parse_liquidation_max_pain(response: &Value) -> ExchangeResult<LiquidationMaxPainData> {
        Self::parse_object(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OPEN INTEREST (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse open interest aggregated (snapshot)
    pub fn parse_oi_aggregated(response: &Value) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        Self::parse_array(response)
    }

    /// Parse open interest history
    pub fn parse_oi_history(response: &Value) -> ExchangeResult<Vec<OpenInterestHistory>> {
        Self::parse_array(response)
    }

    /// Parse open interest volume ratio
    pub fn parse_oi_vol_ratio(response: &Value) -> ExchangeResult<Vec<OpenInterestVolRatio>> {
        Self::parse_array(response)
    }

    /// Parse open interest by coin (chart)
    pub fn parse_oi_by_coin(response: &Value) -> ExchangeResult<Vec<OpenInterestHistory>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING RATES (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse current funding rates
    pub fn parse_current_funding_rates(response: &Value) -> ExchangeResult<Vec<CurrentFundingRate>> {
        Self::parse_array(response)
    }

    /// Parse aggregated funding rate OHLC
    pub fn parse_funding_rate_aggregated(response: &Value) -> ExchangeResult<Vec<FundingRateAggregated>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LONG/SHORT (additional)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse long/short account ratio
    pub fn parse_long_short_account(response: &Value) -> ExchangeResult<Vec<LongShortRatio>> {
        Self::parse_array(response)
    }

    /// Parse global long/short account ratio chart
    pub fn parse_global_long_short(response: &Value) -> ExchangeResult<Vec<LongShortRatio>> {
        Self::parse_array(response)
    }

    /// Parse top long/short position ratio
    pub fn parse_top_long_short_position(response: &Value) -> ExchangeResult<Vec<TopLongShortRatio>> {
        Self::parse_array(response)
    }

    /// Parse top long/short account ratio
    pub fn parse_top_long_short_account(response: &Value) -> ExchangeResult<Vec<TopLongShortRatio>> {
        Self::parse_array(response)
    }

    /// Parse taker buy/sell volume chart
    pub fn parse_taker_buy_sell_volume(response: &Value) -> ExchangeResult<Vec<TakerBuySellVolume>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ORDER BOOK ANALYTICS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse bid/ask range data
    pub fn parse_bid_ask_range(response: &Value) -> ExchangeResult<Vec<BidAskRange>> {
        Self::parse_array(response)
    }

    /// Parse orderbook heatmap
    pub fn parse_orderbook_heatmap(response: &Value) -> ExchangeResult<Vec<OrderbookHeatmapPoint>> {
        Self::parse_array(response)
    }

    /// Parse large orders
    pub fn parse_large_orders(response: &Value) -> ExchangeResult<Vec<LargeOrder>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VOLUME & FLOWS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse CVD chart
    pub fn parse_cvd(response: &Value) -> ExchangeResult<Vec<CvdPoint>> {
        Self::parse_array(response)
    }

    /// Parse net flow indicator
    pub fn parse_net_flow(response: &Value) -> ExchangeResult<Vec<NetFlowPoint>> {
        Self::parse_array(response)
    }

    /// Parse footprint chart
    pub fn parse_footprint(response: &Value) -> ExchangeResult<Vec<FootprintPoint>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OPTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse options max pain
    pub fn parse_options_max_pain(response: &Value) -> ExchangeResult<Vec<OptionsMaxPain>> {
        Self::parse_array(response)
    }

    /// Parse options OI history
    pub fn parse_options_oi_history(response: &Value) -> ExchangeResult<Vec<OptionsOiHistory>> {
        Self::parse_array(response)
    }

    /// Parse options volume history
    pub fn parse_options_volume_history(response: &Value) -> ExchangeResult<Vec<OptionsVolumeHistory>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ON-CHAIN
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse exchange reserve history
    pub fn parse_exchange_reserve(response: &Value) -> ExchangeResult<Vec<ExchangeReserve>> {
        Self::parse_array(response)
    }

    /// Parse exchange balance history
    pub fn parse_exchange_balance_history(response: &Value) -> ExchangeResult<Vec<ExchangeBalanceHistory>> {
        Self::parse_array(response)
    }

    /// Parse ERC-20 transfers
    pub fn parse_erc20_transfers(response: &Value) -> ExchangeResult<Vec<Erc20Transfer>> {
        Self::parse_array(response)
    }

    /// Parse whale transfers
    pub fn parse_whale_transfers(response: &Value) -> ExchangeResult<Vec<WhaleTransfer>> {
        Self::parse_array(response)
    }

    /// Parse token unlocks
    pub fn parse_token_unlocks(response: &Value) -> ExchangeResult<Vec<TokenUnlock>> {
        Self::parse_array(response)
    }

    /// Parse token vesting schedule
    pub fn parse_token_vesting(response: &Value) -> ExchangeResult<Vec<TokenVesting>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ETF
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse ETF flow data (BTC/ETH/SOL/XRP/HK)
    pub fn parse_etf_flow(response: &Value) -> ExchangeResult<Vec<EtfFlowData>> {
        Self::parse_array(response)
    }

    /// Parse Grayscale premium data
    pub fn parse_grayscale_premium(response: &Value) -> ExchangeResult<Vec<GrayscalePremiumData>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HYPERLIQUID
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse HyperLiquid whale alerts
    pub fn parse_hl_whale_alerts(response: &Value) -> ExchangeResult<Vec<HlWhaleAlert>> {
        Self::parse_array(response)
    }

    /// Parse HyperLiquid whale positions
    pub fn parse_hl_whale_positions(response: &Value) -> ExchangeResult<Vec<HlWhalePosition>> {
        Self::parse_array(response)
    }

    /// Parse HyperLiquid wallet positions
    pub fn parse_hl_wallet_positions(response: &Value) -> ExchangeResult<Vec<HlWalletPosition>> {
        Self::parse_array(response)
    }

    /// Parse HyperLiquid position distribution
    pub fn parse_hl_position_distribution(response: &Value) -> ExchangeResult<Vec<HlPositionDistribution>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TECHNICAL INDICATORS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse RSI data
    pub fn parse_rsi(response: &Value) -> ExchangeResult<Vec<RsiData>> {
        Self::parse_array(response)
    }

    /// Parse Moving Average data
    pub fn parse_moving_average(response: &Value) -> ExchangeResult<Vec<MovingAverageData>> {
        Self::parse_array(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // GENERIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse generic array response
    pub fn parse_array<T>(response: &Value) -> ExchangeResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse array: {}", e)))
    }

    /// Parse generic object response
    pub fn parse_object<T>(response: &Value) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = Self::extract_data(response)?;
        serde_json::from_value(data.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse object: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": ["BTC", "ETH", "SOL"]
        });

        assert!(CoinglassParser::is_success(&response));
        let data = CoinglassParser::extract_data(&response).unwrap();
        assert!(data.is_array());
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "code": "30001",
            "msg": "API key missing",
            "success": false
        });

        assert!(!CoinglassParser::is_success(&response));
        let error = CoinglassParser::extract_data(&response);
        assert!(error.is_err());
    }

    #[test]
    fn test_parse_supported_coins() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": ["BTC", "ETH", "SOL", "XRP"]
        });

        let coins = CoinglassParser::parse_supported_coins(&response).unwrap();
        assert_eq!(coins.len(), 4);
        assert_eq!(coins[0], "BTC");
        assert_eq!(coins[1], "ETH");
    }

    #[test]
    fn test_parse_oi_ohlc() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": [
                {
                    "t": 1641522717,
                    "o": "1234567.89",
                    "h": "1245678.90",
                    "l": "1223456.78",
                    "c": "1239876.54"
                }
            ]
        });

        let oi_data = CoinglassParser::parse_oi_ohlc(&response).unwrap();
        assert_eq!(oi_data.len(), 1);
        assert_eq!(oi_data[0].t, 1641522717);
        assert_eq!(oi_data[0].o, "1234567.89");
    }
}
