//! UCDP response parsers
//!
//! Parse JSON responses to domain types based on UCDP API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct UcdpParser;

impl UcdpParser {
    // ═══════════════════════════════════════════════════════════════════════
    // UCDP-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse georeferenced events response
    pub fn parse_events(response: &Value) -> ExchangeResult<UcdpResponse<UcdpEvent>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse events: {}", e)))
    }

    /// Parse battle deaths response
    pub fn parse_battle_deaths(response: &Value) -> ExchangeResult<UcdpResponse<UcdpBattleDeath>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse battle deaths: {}", e)))
    }

    /// Parse non-state conflicts response
    pub fn parse_nonstate_conflicts(response: &Value) -> ExchangeResult<UcdpResponse<UcdpNonStateConflict>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse non-state conflicts: {}", e)))
    }

    /// Parse one-sided violence response
    pub fn parse_onesided_violence(response: &Value) -> ExchangeResult<UcdpResponse<UcdpOneSidedViolence>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse one-sided violence: {}", e)))
    }

    /// Parse state conflicts response
    pub fn parse_state_conflicts(response: &Value) -> ExchangeResult<UcdpResponse<UcdpStateConflict>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse state conflicts: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

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
// UCDP-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Generic UCDP paginated response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpResponse<T> {
    #[serde(rename = "TotalCount")]
    pub total_count: u64,
    #[serde(rename = "PageIndex")]
    pub page: u32,
    #[serde(rename = "PageSize")]
    pub page_size: u32,
    #[serde(rename = "Result")]
    pub result: Vec<T>,
}

/// UCDP georeferenced event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpEvent {
    pub id: u64,
    #[serde(rename = "relid")]
    pub relid: Option<u64>,
    pub year: u32,
    #[serde(rename = "type_of_violence")]
    pub type_of_violence: u8,
    #[serde(rename = "conflict_name")]
    pub conflict_name: String,
    #[serde(rename = "dyad_name")]
    pub dyad_name: Option<String>,
    #[serde(rename = "side_a")]
    pub side_a: String,
    #[serde(rename = "side_b")]
    pub side_b: Option<String>,
    pub country: String,
    pub region: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "deaths_a")]
    pub deaths_a: u32,
    #[serde(rename = "deaths_b")]
    pub deaths_b: u32,
    #[serde(rename = "deaths_civilians")]
    pub deaths_civilians: u32,
    #[serde(rename = "deaths_unknown")]
    pub deaths_unknown: u32,
    #[serde(rename = "best")]
    pub best_estimate: u32,
    #[serde(rename = "date_start")]
    pub date_start: String,
    #[serde(rename = "date_end")]
    pub date_end: String,
}

/// UCDP battle-related deaths
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpBattleDeath {
    pub id: u64,
    #[serde(rename = "conflict_id")]
    pub conflict_id: u64,
    pub location: String,
    #[serde(rename = "side_a")]
    pub side_a: String,
    #[serde(rename = "side_b")]
    pub side_b: String,
    pub year: u32,
    #[serde(rename = "type_of_violence")]
    pub type_of_violence: u8,
    pub region: String,
    #[serde(rename = "bd_best")]
    pub best_estimate: u32,
    #[serde(rename = "bd_low")]
    pub low_estimate: Option<u32>,
    #[serde(rename = "bd_high")]
    pub high_estimate: Option<u32>,
}

/// UCDP non-state conflict
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpNonStateConflict {
    pub id: u64,
    #[serde(rename = "conflict_id")]
    pub conflict_id: u64,
    pub location: String,
    #[serde(rename = "side_a")]
    pub side_a: String,
    #[serde(rename = "side_b")]
    pub side_b: String,
    pub year: u32,
    pub region: String,
    #[serde(rename = "best_fatality_estimate")]
    pub best_estimate: u32,
    #[serde(rename = "low_fatality_estimate")]
    pub low_estimate: Option<u32>,
    #[serde(rename = "high_fatality_estimate")]
    pub high_estimate: Option<u32>,
}

/// UCDP one-sided violence
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpOneSidedViolence {
    pub id: u64,
    #[serde(rename = "conflict_id")]
    pub conflict_id: u64,
    pub location: String,
    #[serde(rename = "actor_name")]
    pub actor_name: String,
    pub year: u32,
    pub region: String,
    #[serde(rename = "best_fatality_estimate")]
    pub best_estimate: u32,
    #[serde(rename = "low_fatality_estimate")]
    pub low_estimate: Option<u32>,
    #[serde(rename = "high_fatality_estimate")]
    pub high_estimate: Option<u32>,
}

/// UCDP state-based conflict
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UcdpStateConflict {
    pub id: u64,
    #[serde(rename = "conflict_id")]
    pub conflict_id: u64,
    pub location: String,
    #[serde(rename = "side_a")]
    pub side_a: String,
    #[serde(rename = "side_b")]
    pub side_b: String,
    #[serde(rename = "territory_name")]
    pub territory_name: Option<String>,
    pub year: u32,
    #[serde(rename = "type_of_violence")]
    pub type_of_violence: u8,
    pub region: String,
    #[serde(rename = "intensity_level")]
    pub cumulative_intensity: u8,
    #[serde(rename = "bd_best")]
    pub best_estimate: u32,
    #[serde(rename = "bd_low")]
    pub low_estimate: Option<u32>,
    #[serde(rename = "bd_high")]
    pub high_estimate: Option<u32>,
}
