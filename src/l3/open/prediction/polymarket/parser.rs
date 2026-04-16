//! Polymarket response parser
//!
//! Contains all Polymarket-specific domain types and conversion functions
//! to V5 core types.
//!
//! ## Type hierarchy
//!
//! Polymarket types (parse raw JSON) → V5 core types (used by chart/UI)
//! - `PolyMarket` → `SymbolInfo`, `Ticker`
//! - `PriceHistoryPoint` → `Kline`
//! - `PolyOrderBook` → `OrderBook`

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::core::types::{
    AccountType, ExchangeError, ExchangeResult, Kline, OrderBook, OrderBookLevel, SymbolInfo, Ticker,
};

// ═══════════════════════════════════════════════════════════════════════════
// CUSTOM DESERIALIZERS
// ═══════════════════════════════════════════════════════════════════════════

/// Deserialize arrays that may be native JSON arrays or stringified JSON.
///
/// The Gamma API sometimes returns arrays as JSON strings (e.g., `"[\"Yes\", \"No\"]"`).
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(arr)) => {
            let vec = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            Ok(Some(vec))
        }
        Some(Value::String(s)) => match serde_json::from_str(&s) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Ok(Some(vec![s])),
        },
        _ => Ok(None),
    }
}

/// Deserialize a string field to f64 (handles both string and number)
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v: Value = Value::deserialize(deserializer)?;
    match v {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| Error::custom("number out of range")),
        Value::String(s) => {
            // Normalize leading dot: ".48" -> "0.48"
            let s = if s.starts_with('.') {
                format!("0{}", s)
            } else {
                s
            };
            s.parse::<f64>()
                .map_err(|_| Error::custom(format!("invalid float: {}", s)))
        }
        _ => Err(Error::custom("expected string or number")),
    }
}

/// Deserialize optional string to optional f64
fn _deserialize_opt_string_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v: Option<Value> = Option::deserialize(deserializer)?;
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(s)) => {
            let s = if s.starts_with('.') {
                format!("0{}", s)
            } else {
                s
            };
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<f64>()
                    .map(Some)
                    .map_err(|_| Error::custom(format!("invalid float: {}", s)))
            }
        }
        _ => Ok(None),
    }
}

/// Deserialize a field that may be a JSON string, number, or null into an optional String.
///
/// The Polymarket CLOB API returns `minimum_order_size` and `minimum_tick_size` as numbers
/// in some responses (e.g. `15` or `0.01`) even though the documented type is string.
fn deserialize_number_or_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Option<Value> = Option::deserialize(deserializer)?;
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Ok(Some(other.to_string())),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GAMMA API TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A prediction market from the Gamma API.
///
/// Represents a single binary YES/NO prediction market.
/// Source: GET `https://gamma-api.polymarket.com/markets`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolyMarket {
    // Core identification
    /// Unique numeric market identifier
    pub id: String,
    /// Blockchain condition ID (0x + 64 hex chars) — primary CLOB identifier
    #[serde(default)]
    pub condition_id: Option<String>,
    /// Alternative identifier
    #[serde(default)]
    pub question_id: Option<String>,
    /// URL-friendly identifier
    #[serde(default)]
    pub slug: Option<String>,
    /// The prediction question text
    #[serde(default)]
    pub question: Option<String>,

    // Outcomes and tokens
    /// Outcome labels, e.g. `["Yes", "No"]`
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub outcomes: Option<Vec<String>>,
    /// Current prices (0.0-1.0) as strings, matches outcomes order
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub outcome_prices: Option<Vec<String>>,
    /// CLOB token IDs for trading, matches outcomes order
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub clob_token_ids: Option<Vec<String>>,

    // Pricing
    #[serde(default)]
    pub last_trade_price: Option<f64>,
    #[serde(default)]
    pub best_bid: Option<f64>,
    #[serde(default)]
    pub best_ask: Option<f64>,
    #[serde(default)]
    pub spread: Option<f64>,
    #[serde(default)]
    pub one_day_price_change: Option<f64>,
    #[serde(default)]
    pub one_hour_price_change: Option<f64>,
    #[serde(default)]
    pub one_week_price_change: Option<f64>,

    // Volume
    #[serde(default)]
    pub volume: Option<String>,
    #[serde(default)]
    pub volume_num: Option<f64>,
    #[serde(default, rename = "volume24hr")]
    pub volume_24hr: Option<f64>,
    #[serde(default, rename = "volume1wk")]
    pub volume_1wk: Option<f64>,
    #[serde(default, rename = "volume1mo")]
    pub volume_1mo: Option<f64>,

    // Liquidity
    #[serde(default)]
    pub liquidity: Option<String>,
    #[serde(default)]
    pub liquidity_num: Option<f64>,

    // Status flags
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub closed: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub accepting_orders: Option<bool>,
    #[serde(default)]
    pub enable_order_book: Option<bool>,
    #[serde(default)]
    pub restricted: Option<bool>,

    // Timestamps
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,

    // Metadata
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub resolution_source: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub market_type: Option<String>,

    // Trading configuration
    #[serde(default)]
    pub order_price_min_tick_size: Option<f64>,
    #[serde(default)]
    pub order_min_size: Option<f64>,
    #[serde(default)]
    pub maker_base_fee: Option<i32>,
    #[serde(default)]
    pub taker_base_fee: Option<i32>,

    // Tags
    #[serde(default)]
    pub tags: Option<Vec<PolyTag>>,
}

impl PolyMarket {
    /// Get YES price as f64 (first outcome price)
    pub fn yes_price(&self) -> Option<f64> {
        self.outcome_prices
            .as_ref()
            .and_then(|p| p.first())
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Get NO price as f64 (second outcome price)
    pub fn no_price(&self) -> Option<f64> {
        self.outcome_prices
            .as_ref()
            .and_then(|p| p.get(1))
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Get YES token ID (first CLOB token)
    pub fn yes_token_id(&self) -> Option<&str> {
        self.clob_token_ids
            .as_ref()
            .and_then(|ids| ids.first())
            .map(|s| s.as_str())
    }

    /// Get NO token ID (second CLOB token)
    pub fn no_token_id(&self) -> Option<&str> {
        self.clob_token_ids
            .as_ref()
            .and_then(|ids| ids.get(1))
            .map(|s| s.as_str())
    }

    /// Check if market is tradeable (active, not closed, order book enabled)
    pub fn is_tradeable(&self) -> bool {
        self.active.unwrap_or(false)
            && !self.closed.unwrap_or(true)
            && self.enable_order_book.unwrap_or(false)
    }
}

/// Tag for market categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyTag {
    pub id: Option<String>,
    pub label: Option<String>,
    pub slug: Option<String>,
}

/// A prediction event container from the Gamma API.
///
/// Events group related markets. Source: GET `https://gamma-api.polymarket.com/events`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolyEvent {
    pub id: String,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub closed: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub liquidity: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
    #[serde(default, rename = "volume24hr")]
    pub volume_24hr: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub markets: Option<Vec<PolyMarket>>,
}

// ═══════════════════════════════════════════════════════════════════════════
// CLOB API TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// CLOB market from the paginated /markets endpoint
///
/// Source: GET `https://clob.polymarket.com/markets`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobMarket {
    /// Blockchain condition ID — primary market identifier
    pub condition_id: String,
    /// Human-readable question
    #[serde(default)]
    pub question: Option<String>,
    /// URL slug
    #[serde(default)]
    pub market_slug: Option<String>,
    /// Active for trading
    #[serde(default)]
    pub active: Option<bool>,
    /// Market closed
    #[serde(default)]
    pub closed: Option<bool>,
    /// ISO 8601 end date
    #[serde(rename = "end_date_iso", default)]
    pub end_date: Option<String>,
    /// Tokens (outcomes)
    #[serde(default)]
    pub tokens: Vec<PolyToken>,
    /// Minimum order size (API returns number or string)
    #[serde(default, deserialize_with = "deserialize_number_or_string")]
    pub minimum_order_size: Option<String>,
    /// Minimum tick size (API returns number or string)
    #[serde(default, deserialize_with = "deserialize_number_or_string")]
    pub minimum_tick_size: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Maker fee in bps
    #[serde(default)]
    pub maker_base_fee: Option<i32>,
    /// Taker fee in bps
    #[serde(default)]
    pub taker_base_fee: Option<i32>,
    /// Negative risk market
    #[serde(default)]
    pub neg_risk: Option<bool>,
}

/// A single outcome token within a CLOB market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyToken {
    /// Token ID used for CLOB price/book/history API calls
    pub token_id: String,
    /// Outcome label ("Yes" or "No")
    pub outcome: String,
    /// Current price (0.0 - 1.0)
    #[serde(default)]
    pub price: Option<f64>,
    /// Whether this outcome won
    #[serde(default)]
    pub winner: Option<bool>,
}

/// Order book from CLOB API.
///
/// Source: GET `https://clob.polymarket.com/book?token_id=...`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyOrderBook {
    /// Market condition ID
    pub market: String,
    /// Token ID (YES or NO)
    pub asset_id: String,
    /// ISO 8601 timestamp
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Bid levels (price desc)
    #[serde(default)]
    pub bids: Vec<PolyPriceLevel>,
    /// Ask levels (price asc)
    #[serde(default)]
    pub asks: Vec<PolyPriceLevel>,
    /// Min order size
    #[serde(default)]
    pub min_order_size: Option<String>,
    /// Tick size
    #[serde(default)]
    pub tick_size: Option<String>,
}

/// Single price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyPriceLevel {
    /// Price (0.0 - 1.0) as string
    pub price: String,
    /// Total size at this level
    pub size: String,
}

impl PolyPriceLevel {
    /// Parse price as f64 (normalizes ".48" to "0.48")
    pub fn price_f64(&self) -> Option<f64> {
        let s = if self.price.starts_with('.') {
            format!("0{}", self.price)
        } else {
            self.price.clone()
        };
        s.parse::<f64>().ok()
    }

    /// Parse size as f64
    pub fn size_f64(&self) -> Option<f64> {
        self.size.parse::<f64>().ok()
    }
}

/// Price history data point from CLOB API.
///
/// Source: GET `https://clob.polymarket.com/prices-history?market=...`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    /// Unix timestamp in seconds
    #[serde(rename = "t")]
    pub timestamp: i64,
    /// Price at this point (0.0 - 1.0)
    #[serde(rename = "p")]
    pub price: f64,
}

/// Midpoint price response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyMidpoint {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub mid: f64,
}

/// Order from authenticated CLOB API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyOrder {
    pub id: String,
    pub status: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub outcome: String,
    pub owner: String,
    #[serde(default)]
    pub maker_address: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(default)]
    pub order_type: Option<String>,
}

/// Trade from CLOB API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyTrade {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: String,
    pub price: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub match_time: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// WEBSOCKET TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// WebSocket subscription message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscription {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

/// Full order book snapshot from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBookSnapshot {
    pub event_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    pub market: String,
    pub bids: Vec<PolyPriceLevel>,
    pub asks: Vec<PolyPriceLevel>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub hash: Option<String>,
}

/// Incremental price update from WebSocket
///
/// Polymarket sends price_change as a single level update with `price`/`size`
/// fields at the top level, not in a `changes` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPriceChange {
    pub event_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    /// Batch of changes (may be absent — single-level updates use price/size fields)
    #[serde(default)]
    pub changes: Vec<PolyPriceLevel>,
    /// Price of the changed level (single-level format)
    #[serde(default)]
    pub price: Option<String>,
    /// Size at this price (single-level format)
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Last trade price event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsLastTradePrice {
    pub event_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    pub market: String,
    pub price: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Tick size change event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsTickSizeChange {
    pub event_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    pub market: String,
    pub old_tick_size: String,
    pub new_tick_size: String,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Best bid/ask event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsBestBidAsk {
    pub event_type: String,
    #[serde(default)]
    pub asset_id: Option<String>,
    pub market: String,
    pub best_bid: String,
    pub best_ask: String,
    #[serde(default)]
    pub spread: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════

/// Response parser for Polymarket API responses
pub struct PolymarketParser;

impl PolymarketParser {
    // -----------------------------------------------------------------------
    // Market parsing
    // -----------------------------------------------------------------------

    /// Parse CLOB markets list from paginated response
    ///
    /// Handles both `{"data": [...]}` and bare `[...]` formats.
    pub fn parse_clob_markets(response: &Value) -> ExchangeResult<Vec<ClobMarket>> {
        let arr = response
            .get("data")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected array of markets".to_string()))?;

        arr.iter()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse ClobMarket: {}", e)))
            })
            .collect()
    }

    /// Parse single CLOB market
    pub fn parse_clob_market(response: &Value) -> ExchangeResult<ClobMarket> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse ClobMarket: {}", e)))
    }

    /// Parse Gamma markets list
    pub fn parse_gamma_markets(response: &Value) -> ExchangeResult<Vec<PolyMarket>> {
        let arr = response
            .as_array()
            .or_else(|| response.get("data").and_then(|v| v.as_array()))
            .ok_or_else(|| ExchangeError::Parse("Expected array of markets".to_string()))?;

        arr.iter()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyMarket: {}", e)))
            })
            .collect()
    }

    /// Parse single Gamma market
    pub fn parse_gamma_market(response: &Value) -> ExchangeResult<PolyMarket> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyMarket: {}", e)))
    }

    /// Parse events list from Gamma API
    pub fn parse_events(response: &Value) -> ExchangeResult<Vec<PolyEvent>> {
        let arr = response
            .as_array()
            .or_else(|| response.get("data").and_then(|v| v.as_array()))
            .ok_or_else(|| ExchangeError::Parse("Expected array of events".to_string()))?;

        arr.iter()
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyEvent: {}", e)))
            })
            .collect()
    }

    /// Parse single event from Gamma API
    pub fn parse_event(response: &Value) -> ExchangeResult<PolyEvent> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyEvent: {}", e)))
    }

    // -----------------------------------------------------------------------
    // Price / book parsing
    // -----------------------------------------------------------------------

    /// Parse order book response
    pub fn parse_order_book(response: &Value) -> ExchangeResult<PolyOrderBook> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyOrderBook: {}", e)))
    }

    /// Parse midpoint price response
    pub fn parse_midpoint(response: &Value) -> ExchangeResult<PolyMidpoint> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse PolyMidpoint: {}", e)))
    }

    /// Parse last trade price from `{"price": "0.52"}` response
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let price_str = response
            .get("price")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'price' field".to_string()))?;

        let normalized = if price_str.starts_with('.') {
            format!("0{}", price_str)
        } else {
            price_str.to_string()
        };

        normalized
            .parse::<f64>()
            .map_err(|e| ExchangeError::Parse(format!("Invalid price '{}': {}", price_str, e)))
    }

    /// Parse price history response to raw points
    ///
    /// Response format: `{"history": [{"t": 1234567890, "p": 0.65}, ...]}`
    /// or bare array: `[{"t": ..., "p": ...}, ...]`
    pub fn parse_price_history(response: &Value) -> ExchangeResult<Vec<PriceHistoryPoint>> {
        let arr = response
            .get("history")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .ok_or_else(|| ExchangeError::Parse("Expected price history array".to_string()))?;

        arr.iter()
            .map(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ExchangeError::Parse(format!("Failed to parse PriceHistoryPoint: {}", e))
                })
            })
            .collect()
    }

    /// Get pagination cursor from response
    pub fn get_next_cursor(response: &Value) -> Option<String> {
        response
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "LTE=")
            .map(String::from)
    }

    /// Check response for API errors
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let msg = error
                .as_str()
                .unwrap_or("Unknown API error")
                .to_string();
            return Err(ExchangeError::Api { code: 0, message: msg });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONVERSIONS TO V5 CORE TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Convert a ClobMarket to V5 SymbolInfo
///
/// Uses `condition_id` as the symbol identifier.
/// The market question becomes the base_asset for display purposes.
pub fn clob_market_to_symbol_info(market: &ClobMarket, account_type: AccountType) -> SymbolInfo {
    let question_short = market
        .question
        .as_deref()
        .unwrap_or("Unknown")
        .chars()
        .take(50)
        .collect::<String>();

    SymbolInfo {
        symbol: market.condition_id.clone(),
        base_asset: question_short,
        quote_asset: "USDC".to_string(),
        status: if market.active.unwrap_or(false) && !market.closed.unwrap_or(true) {
            "TRADING"
        } else {
            "BREAK"
        }
        .to_string(),
        price_precision: 4,
        quantity_precision: 2,
        min_quantity: market
            .minimum_order_size
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok()),
        max_quantity: None,
        // CLOB markets provide minimum_tick_size — use it for both tick_size and step_size
        tick_size: market
            .minimum_tick_size
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok()),
        step_size: market
            .minimum_tick_size
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok()),
        min_notional: None,
        account_type,
    }
}

/// Convert a PolyMarket (Gamma) to V5 SymbolInfo
pub fn poly_market_to_symbol_info(market: &PolyMarket, account_type: AccountType) -> SymbolInfo {
    let condition_id = market
        .condition_id
        .as_deref()
        .unwrap_or(&market.id)
        .to_string();

    let question = market
        .question
        .as_deref()
        .unwrap_or("Unknown")
        .chars()
        .take(50)
        .collect::<String>();

    SymbolInfo {
        symbol: condition_id,
        base_asset: question,
        quote_asset: "USDC".to_string(),
        status: if market.active.unwrap_or(false) && !market.closed.unwrap_or(true) {
            "TRADING"
        } else {
            "BREAK"
        }
        .to_string(),
        price_precision: 4,
        quantity_precision: 2,
        min_quantity: market.order_min_size,
        max_quantity: None,
        // Gamma markets provide order_price_min_tick_size — use it for both tick_size and step_size
        tick_size: market.order_price_min_tick_size,
        step_size: market.order_price_min_tick_size,
        min_notional: None,
        account_type,
    }
}

/// Convert price history points to V5 Klines
///
/// Prediction probability (0.0-1.0) IS the price.
/// Each PriceHistoryPoint becomes a flat kline: open=high=low=close=price, volume=0.
///
/// `interval_ms` — duration of each interval in milliseconds (for close_time calculation)
pub fn price_history_to_klines(
    history: Vec<PriceHistoryPoint>,
    interval_ms: u64,
) -> Vec<Kline> {
    history
        .into_iter()
        .map(|point| {
            let open_time = point.timestamp * 1000; // seconds → milliseconds
            let price = point.price;

            Kline {
                open_time,
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 0.0,
                quote_volume: None,
                close_time: Some(open_time + interval_ms as i64 - 1),
                trades: None,
            }
        })
        .collect()
}

/// Convert PolyOrderBook to V5 OrderBook
///
/// Bids are sorted descending (highest price first).
/// Asks are sorted ascending (lowest price first).
/// The CLOB API does not guarantee order, so we sort explicitly.
pub fn poly_orderbook_to_v5(book: &PolyOrderBook) -> OrderBook {
    let mut bids: Vec<OrderBookLevel> = book
        .bids
        .iter()
        .filter_map(|level| {
            let p = level.price_f64()?;
            let s = level.size_f64()?;
            Some(OrderBookLevel::new(p, s))
        })
        .collect();
    // Sort bids descending by price (best bid first)
    bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));

    let mut asks: Vec<OrderBookLevel> = book
        .asks
        .iter()
        .filter_map(|level| {
            let p = level.price_f64()?;
            let s = level.size_f64()?;
            Some(OrderBookLevel::new(p, s))
        })
        .collect();
    // Sort asks ascending by price (best ask first)
    asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

    OrderBook {
        bids,
        asks,
        timestamp: chrono::Utc::now().timestamp_millis(),
        sequence: book.timestamp.clone(),
        last_update_id: None,
        first_update_id: None,
        prev_update_id: None,
        event_time: None,
        transaction_time: None,
        checksum: None,
    }
}

/// Convert a ClobMarket to V5 Ticker using the primary token price.
///
/// Prefers the "Yes" outcome token; falls back to the first token for non-binary markets.
pub fn clob_market_to_ticker(market: &ClobMarket) -> Option<Ticker> {
    let yes_token = market
        .tokens
        .iter()
        .find(|t| t.outcome == "Yes")
        .or_else(|| market.tokens.first())?;
    let price = yes_token.price?;

    Some(Ticker {
        symbol: market.condition_id.clone(),
        last_price: price,
        bid_price: None,
        ask_price: None,
        high_24h: None,
        low_24h: None,
        volume_24h: None,
        quote_volume_24h: None,
        price_change_24h: None,
        price_change_percent_24h: None,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

/// Convert a PolyMarket (Gamma) to V5 Ticker
pub fn poly_market_to_ticker(market: &PolyMarket) -> Ticker {
    let condition_id = market
        .condition_id
        .as_deref()
        .unwrap_or(&market.id)
        .to_string();

    let last_price = market
        .last_trade_price
        .or_else(|| market.yes_price())
        .unwrap_or(0.0);

    Ticker {
        symbol: condition_id,
        last_price,
        bid_price: market.best_bid,
        ask_price: market.best_ask,
        high_24h: None,
        low_24h: None,
        volume_24h: market.volume_24hr,
        quote_volume_24h: market.volume_24hr,
        price_change_24h: market.one_day_price_change,
        price_change_percent_24h: market
            .one_day_price_change
            .zip(Some(last_price))
            .map(|(change, _)| change * 100.0),
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

/// Get interval duration in milliseconds for a Polymarket interval string
pub fn interval_to_ms(interval: &str) -> u64 {
    match interval {
        "1m" => 60_000,
        "1h" => 3_600_000,
        "6h" => 21_600_000,
        "1d" => 86_400_000,
        "1w" => 604_800_000,
        _ => 86_400_000,
    }
}
