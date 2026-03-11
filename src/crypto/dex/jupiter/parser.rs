//! # Jupiter Response Parser
//!
//! JSON parsing for Jupiter API responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::types::{ExchangeError, ExchangeResult, Kline, OrderBook, Price, Ticker};

use super::endpoints::{from_raw_amount, MintRegistry};

// ═══════════════════════════════════════════════════════════════════════════════
// RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Quote response from /quote endpoint
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: String,
    #[serde(default)]
    pub route_plan: Vec<RoutePlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee: Option<PlatformFee>,
    pub context_slot: Option<u64>,
    pub time_taken: Option<f64>,
}

/// Route plan item
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u16,
}

/// Swap info in route plan
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: String,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

/// Platform fee
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: String,
    pub fee_bps: u16,
}

/// Price response from /price/v3 endpoint
#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[serde(flatten)]
    pub prices: std::collections::HashMap<String, Option<PriceData>>,
}

/// Price data for a single token
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub usd_price: f64,
    pub block_id: u64,
    pub decimals: u8,
    pub price_change_24h: f64,
}

/// Token metadata from Tokens API
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organic_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd_price: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// Jupiter response parser
pub struct JupiterParser;

impl JupiterParser {
    /// Parse price from Quote response
    ///
    /// Uses the quote endpoint to derive price from in_amount / out_amount ratio
    pub fn parse_price_from_quote(response: &Value) -> ExchangeResult<Price> {
        let in_amount = response
            .get("inAmount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid inAmount".to_string()))?;

        let out_amount = response
            .get("outAmount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid outAmount".to_string()))?;

        let input_mint = response
            .get("inputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing inputMint".to_string()))?;

        let output_mint = response
            .get("outputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing outputMint".to_string()))?;

        // Get decimals for input and output tokens
        let input_decimals = MintRegistry::decimals(input_mint).unwrap_or(9);
        let output_decimals = MintRegistry::decimals(output_mint).unwrap_or(6);

        // Convert to human-readable amounts
        let human_in = from_raw_amount(in_amount, input_decimals);
        let human_out = from_raw_amount(out_amount, output_decimals);

        // Price = output / input
        let price = if human_in > 0.0 {
            human_out / human_in
        } else {
            0.0
        };

        Ok(price)
    }

    /// Parse price from Price API response
    pub fn parse_price_from_api(response: &Value, mint: &str) -> ExchangeResult<Price> {
        let price_data = response
            .get(mint)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse(format!("Price data not found for mint {}", mint)))?;

        let usd_price = price_data
            .get("usdPrice")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Missing usdPrice".to_string()))?;

        Ok(usd_price)
    }

    /// Parse ticker from Price API response
    pub fn parse_ticker_from_price(response: &Value, mint: &str) -> ExchangeResult<Ticker> {
        let price_data = response
            .get(mint)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse(format!("Price data not found for mint {}", mint)))?;

        let last_price = price_data
            .get("usdPrice")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let price_change_24h = price_data
            .get("priceChange24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        Ok(Ticker {
            symbol: mint.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: Some(price_change_24h),
            price_change_percent_24h: Some(price_change_24h),
            high_24h: None,
            low_24h: None,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    /// Parse orderbook from quote data (simulated)
    ///
    /// Jupiter is a DEX aggregator without a traditional orderbook.
    /// This creates a minimal orderbook representation using quote data.
    pub fn parse_orderbook_from_quote(response: &Value) -> ExchangeResult<OrderBook> {
        let out_amount = response
            .get("outAmount")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing outAmount".to_string()))?;

        let output_mint = response
            .get("outputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing outputMint".to_string()))?;

        let output_decimals = MintRegistry::decimals(output_mint).unwrap_or(6);
        let quantity = from_raw_amount(out_amount, output_decimals);

        // Derive price from in/out ratio
        let price = Self::parse_price_from_quote(response)?;

        // Create minimal orderbook with single ask
        Ok(OrderBook {
            bids: vec![],
            asks: vec![(price, quantity)],
            timestamp: crate::core::timestamp_millis() as i64,
            sequence: None,
        })
    }

    /// Parse klines (not supported - Jupiter has no historical kline data)
    pub fn parse_klines(_response: &Value) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Klines not supported by Jupiter API".to_string(),
        ))
    }

    /// Parse trading pairs from Tokens API
    pub fn parse_trading_pairs(response: &Value) -> ExchangeResult<Vec<String>> {
        let tokens = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of tokens".to_string()))?;

        let pairs: Vec<String> = tokens
            .iter()
            .filter_map(|token| {
                token
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(pairs)
    }

    /// Check for API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: -1,
                message,
            });
        }

        Ok(())
    }
}
