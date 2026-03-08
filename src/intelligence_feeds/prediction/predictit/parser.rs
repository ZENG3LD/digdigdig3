//! PredictIt response parsers
//!
//! Parse JSON responses to domain types based on PredictIt API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct PredictItParser;

impl PredictItParser {
    /// Parse all markets response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "markets": [{
    ///     "id": 7940,
    ///     "name": "Who will win the 2024 presidential election?",
    ///     "shortName": "2024 President",
    ///     "image": "...",
    ///     "url": "...",
    ///     "status": "Open",
    ///     "contracts": [...],
    ///     "timeStamp": "2024-01-15T12:00:00"
    ///   }]
    /// }
    /// ```
    pub fn parse_all_markets(response: &Value) -> ExchangeResult<PredictItResponse> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse markets: {}", e)))
    }

    /// Parse single market response
    pub fn parse_market(response: &Value) -> ExchangeResult<PredictItMarket> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse market: {}", e)))
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PREDICTIT-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// PredictIt API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictItResponse {
    #[serde(rename = "markets")]
    pub markets: Vec<PredictItMarket>,
}

/// PredictIt market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictItMarket {
    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "shortName")]
    pub short_name: String,

    #[serde(rename = "image")]
    pub image: String,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "contracts")]
    pub contracts: Vec<PredictItContract>,

    #[serde(rename = "timeStamp")]
    pub timestamp: String,
}

/// PredictIt contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictItContract {
    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "shortName")]
    pub short_name: String,

    #[serde(rename = "image")]
    pub image: String,

    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "lastTradePrice")]
    pub last_trade_price: Option<f64>,

    #[serde(rename = "bestBuyYesCost")]
    pub best_buy_yes_cost: Option<f64>,

    #[serde(rename = "bestBuyNoCost")]
    pub best_buy_no_cost: Option<f64>,

    #[serde(rename = "bestSellYesCost")]
    pub best_sell_yes_cost: Option<f64>,

    #[serde(rename = "bestSellNoCost")]
    pub best_sell_no_cost: Option<f64>,

    #[serde(rename = "lastClosePrice")]
    pub last_close_price: Option<f64>,

    #[serde(rename = "displayOrder")]
    pub display_order: u32,
}
