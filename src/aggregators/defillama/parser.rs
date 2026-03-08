//! # DefiLlama Response Parser
//!
//! Парсинг JSON ответов от DefiLlama API.
//!
//! ## Response Types
//!
//! - Protocol data (TVL, chains, metadata)
//! - Token prices (current, historical)
//! - Stablecoin data
//! - Yield pool data
//! - Fee/revenue data

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ExchangeResult, ExchangeError};

// ═══════════════════════════════════════════════════════════════════════════════
// PROTOCOL DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Protocol metadata and TVL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolData {
    /// Protocol ID (slug)
    #[serde(default)]
    pub id: Option<String>,
    /// Protocol name
    #[serde(default)]
    pub name: Option<String>,
    /// Protocol URL
    #[serde(default)]
    pub url: Option<String>,
    /// Protocol description
    #[serde(default)]
    pub description: Option<String>,
    /// Protocol logo URL
    #[serde(default)]
    pub logo: Option<String>,
    /// Total Value Locked (USD) - can be f64, array of chain TVLs, or null
    #[serde(default, deserialize_with = "deserialize_tvl")]
    pub tvl: Option<f64>,
    /// Chain breakdown of TVL
    #[serde(rename = "chainTvls", default)]
    pub chain_tvls: serde_json::Map<String, Value>,
    /// Chain list
    #[serde(default)]
    pub chains: Vec<String>,
    /// Category (DEX, Lending, etc.)
    #[serde(default)]
    pub category: Option<String>,
    /// Change in TVL (1h, 1d, 7d)
    #[serde(default)]
    pub change_1h: Option<f64>,
    #[serde(default)]
    pub change_1d: Option<f64>,
    #[serde(default)]
    pub change_7d: Option<f64>,
    /// Market cap (if token exists)
    #[serde(rename = "mcap", default)]
    pub market_cap: Option<f64>,
}

/// Historical TVL data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvlDataPoint {
    /// Unix timestamp
    pub date: u64,
    /// Total Value Locked (USD)
    #[serde(rename = "totalLiquidityUSD")]
    pub tvl: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRICE DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Token price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// Price in USD
    pub price: f64,
    /// Symbol (if available)
    #[serde(default)]
    pub symbol: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// Confidence score (0-1)
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Batch price response (coin_id -> PriceData)
pub type PriceResponse = std::collections::HashMap<String, CoinPrice>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinPrice {
    /// Decimals
    #[serde(default)]
    pub decimals: u32,
    /// Price in USD
    pub price: f64,
    /// Symbol
    #[serde(default)]
    pub symbol: String,
    /// Timestamp
    pub timestamp: u64,
    /// Confidence
    #[serde(default)]
    pub confidence: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHAIN DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Chain TVL data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainData {
    /// Chain name
    #[serde(default)]
    pub gecko_id: Option<String>,
    /// Total TVL
    #[serde(default)]
    pub tvl: Option<f64>,
    /// Token symbol (if native token)
    #[serde(rename = "tokenSymbol", default)]
    pub token_symbol: Option<String>,
    /// Chain CMC ID
    #[serde(rename = "cmcId", default, deserialize_with = "deserialize_string_or_number")]
    pub cmc_id: Option<String>,
    /// Chain name
    #[serde(default)]
    pub name: Option<String>,
    /// Chain ID (can be string or number from API)
    #[serde(rename = "chainId", default, deserialize_with = "deserialize_string_or_number")]
    pub chain_id: Option<String>,
}

/// Deserialize TVL that may be f64, array of chain TVL objects, or null
fn deserialize_tvl<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::Array(arr)) => {
            // Array of chain TVL objects: [{"date":..., "totalLiquidityUSD":...}, ...]
            // Return the latest (last) entry's totalLiquidityUSD
            let last_tvl = arr.iter().rev()
                .find_map(|entry| {
                    entry.get("totalLiquidityUSD")
                        .and_then(|v| v.as_f64())
                });
            Ok(last_tvl)
        }
        _ => Ok(None),
    }
}

/// Deserialize a field that may be a string, integer, or null
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Ok(Some(other.to_string())),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STABLECOIN DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Stablecoin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StablecoinData {
    /// Stablecoin ID (can be integer from API)
    #[serde(deserialize_with = "deserialize_id_to_string")]
    pub id: String,
    /// Stablecoin name
    pub name: String,
    /// Symbol
    pub symbol: String,
    /// Gecko ID
    #[serde(rename = "gecko_id", default)]
    pub gecko_id: Option<String>,
    /// Peg type (USD, EUR, etc.)
    #[serde(rename = "pegType", default)]
    pub peg_type: String,
    /// Price (should be ~$1 for USD-pegged, can be string or number from API)
    #[serde(default, deserialize_with = "deserialize_optional_f64_or_string")]
    pub price: Option<f64>,
    /// Total circulating supply (can be object or number)
    #[serde(default, deserialize_with = "deserialize_circulating")]
    pub circulating: Option<f64>,
}

/// Deserialize id that may be string or integer
fn deserialize_id_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        other => Ok(other.to_string()),
    }
}

/// Deserialize an optional f64 that may come as a string, number, or null
fn deserialize_optional_f64_or_string<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(s)) => Ok(s.parse::<f64>().ok()),
        _ => Ok(None),
    }
}

/// Deserialize circulating supply that may be a number or object (with chain breakdown)
fn deserialize_circulating<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::Object(map)) => {
            // Sum all chain values if it's a chain breakdown object
            let total: f64 = map.values()
                .filter_map(|v| v.as_f64())
                .sum();
            if total > 0.0 { Ok(Some(total)) } else { Ok(None) }
        }
        _ => Ok(None),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// YIELD DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Yield pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldPoolData {
    /// Pool ID
    pub pool: String,
    /// Chain
    pub chain: String,
    /// Project (protocol)
    pub project: String,
    /// Symbol
    pub symbol: String,
    /// TVL in USD
    #[serde(rename = "tvlUsd", default)]
    pub tvl_usd: f64,
    /// APY (Annual Percentage Yield)
    #[serde(default)]
    pub apy: Option<f64>,
    /// APY base (without rewards)
    #[serde(rename = "apyBase", default)]
    pub apy_base: Option<f64>,
    /// APY from rewards
    #[serde(rename = "apyReward", default)]
    pub apy_reward: Option<f64>,
    /// Stablecoin pool
    #[serde(default)]
    pub stablecoin: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// DefiLlama response parser
pub struct DefiLlamaParser;

impl DefiLlamaParser {
    /// Parse protocol list
    pub fn parse_protocols(response: &Value) -> ExchangeResult<Vec<ProtocolData>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse protocols: {}", e)))
    }

    /// Parse single protocol
    pub fn parse_protocol(response: &Value) -> ExchangeResult<ProtocolData> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse protocol: {}", e)))
    }

    /// Parse historical TVL data
    ///
    /// The `/tvl/{protocol}` endpoint returns a single number (current TVL).
    /// The full historical TVL array comes from `/protocol/{protocol}` in the `tvl` field.
    /// This method handles both: a single number or an array of data points.
    pub fn parse_tvl_history(response: &Value) -> ExchangeResult<Vec<TvlDataPoint>> {
        // If response is a number, return a single data point with current time
        if let Some(tvl) = response.as_f64() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Ok(vec![TvlDataPoint { date: now, tvl }]);
        }

        // If response is an array, parse as Vec<TvlDataPoint>
        if response.is_array() {
            return serde_json::from_value(response.clone())
                .map_err(|e| ExchangeError::Parse(format!("Failed to parse TVL history: {}", e)));
        }

        Err(ExchangeError::Parse(format!(
            "Unexpected TVL response format: expected number or array, got {}",
            response
        )))
    }

    /// Parse current prices
    pub fn parse_prices(response: &Value) -> ExchangeResult<PriceResponse> {
        // Response format: { "coins": { "ethereum:0xabc": { "price": 1.0, ... } } }
        let coins = response.get("coins")
            .ok_or_else(|| ExchangeError::Parse("Missing 'coins' field in prices response".to_string()))?;

        serde_json::from_value(coins.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse prices: {}", e)))
    }

    /// Parse chain list
    pub fn parse_chains(response: &Value) -> ExchangeResult<Vec<ChainData>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse chains: {}", e)))
    }

    /// Parse stablecoins
    pub fn parse_stablecoins(response: &Value) -> ExchangeResult<Vec<StablecoinData>> {
        // Response format: { "peggedAssets": [...] }
        let assets = response.get("peggedAssets")
            .ok_or_else(|| ExchangeError::Parse("Missing 'peggedAssets' field".to_string()))?;

        serde_json::from_value(assets.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse stablecoins: {}", e)))
    }

    /// Parse yield pools
    pub fn parse_yield_pools(response: &Value) -> ExchangeResult<Vec<YieldPoolData>> {
        // Response format: { "data": [...] }
        let data = response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse yield pools: {}", e)))
    }

    /// Extract TVL from protocol data
    pub fn extract_tvl(protocol: &ProtocolData) -> Option<f64> {
        protocol.tvl
    }

    /// Extract price from price data
    pub fn extract_price(coin_id: &str, prices: &PriceResponse) -> Option<f64> {
        prices.get(coin_id).map(|p| p.price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_protocol() {
        let json: Value = serde_json::from_str(r#"{
            "id": "aave",
            "name": "Aave",
            "url": "https://aave.com",
            "description": "Decentralized lending protocol",
            "logo": "https://icons.llama.fi/aave.png",
            "tvl": 5000000000.0,
            "chains": ["Ethereum", "Polygon"],
            "category": "Lending"
        }"#).unwrap();

        let protocol = DefiLlamaParser::parse_protocol(&json).unwrap();
        assert_eq!(protocol.id.as_deref(), Some("aave"));
        assert_eq!(protocol.name.as_deref(), Some("Aave"));
        assert_eq!(protocol.tvl, Some(5000000000.0));
    }

    #[test]
    fn test_parse_prices() {
        let json: Value = serde_json::from_str(r#"{
            "coins": {
                "ethereum:0x6b175474e89094c44da98b954eedeac495271d0f": {
                    "decimals": 18,
                    "price": 1.0,
                    "symbol": "DAI",
                    "timestamp": 1234567890,
                    "confidence": 0.99
                }
            }
        }"#).unwrap();

        let prices = DefiLlamaParser::parse_prices(&json).unwrap();
        let dai_price = prices.get("ethereum:0x6b175474e89094c44da98b954eedeac495271d0f");
        assert!(dai_price.is_some());
        assert_eq!(dai_price.unwrap().price, 1.0);
    }
}
