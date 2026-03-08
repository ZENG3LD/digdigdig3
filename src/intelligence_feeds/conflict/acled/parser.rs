//! ACLED response parsers
//!
//! Parse JSON responses to domain types based on ACLED API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct AcledParser;

impl AcledParser {
    /// Parse ACLED events response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "success": true,
    ///   "count": 100,
    ///   "data": [
    ///     {
    ///       "event_id_cnty": "SYR12345",
    ///       "event_date": "2024-01-15",
    ///       "year": 2024,
    ///       "event_type": "Battles",
    ///       "sub_event_type": "Armed clash",
    ///       "actor1": "Military Forces",
    ///       "actor2": "Rebel Group",
    ///       "country": "Syria",
    ///       "admin1": "Aleppo",
    ///       "admin2": "Aleppo",
    ///       "admin3": "",
    ///       "location": "Aleppo City",
    ///       "latitude": 36.2021,
    ///       "longitude": 37.1343,
    ///       "fatalities": 5,
    ///       "notes": "Event description...",
    ///       "source": "Local Media"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_events(response: &Value) -> ExchangeResult<AcledResponse> {
        let success = response
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let count = response
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let data_array = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let data: Result<Vec<AcledEvent>, ExchangeError> = data_array
            .iter()
            .map(Self::parse_event)
            .collect();

        Ok(AcledResponse {
            success,
            count,
            data: data?,
        })
    }

    /// Parse single ACLED event
    fn parse_event(event: &Value) -> ExchangeResult<AcledEvent> {
        Ok(AcledEvent {
            event_id_cnty: Self::require_str(event, "event_id_cnty")?.to_string(),
            event_date: Self::require_str(event, "event_date")?.to_string(),
            year: Self::require_u32(event, "year")?,
            event_type: Self::require_str(event, "event_type")?.to_string(),
            sub_event_type: Self::require_str(event, "sub_event_type")?.to_string(),
            actor1: Self::require_str(event, "actor1")?.to_string(),
            actor2: Self::get_str(event, "actor2").map(|s| s.to_string()),
            country: Self::require_str(event, "country")?.to_string(),
            admin1: Self::require_str(event, "admin1")?.to_string(),
            admin2: Self::get_str(event, "admin2").map(|s| s.to_string()),
            admin3: Self::get_str(event, "admin3").map(|s| s.to_string()),
            location: Self::require_str(event, "location")?.to_string(),
            latitude: Self::require_f64(event, "latitude")?,
            longitude: Self::require_f64(event, "longitude")?,
            fatalities: Self::require_u32(event, "fatalities")?,
            notes: Self::require_str(event, "notes")?.to_string(),
            source: Self::require_str(event, "source")?.to_string(),
        })
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

        // Check for success=false
        if let Some(success) = response.get("success") {
            if success.as_bool() == Some(false) {
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Request failed")
                    .to_string();
                return Err(ExchangeError::Api { code: 0, message });
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ACLED-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// ACLED event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcledEvent {
    pub event_id_cnty: String,
    pub event_date: String,
    pub year: u32,
    pub event_type: String,
    pub sub_event_type: String,
    pub actor1: String,
    pub actor2: Option<String>,
    pub country: String,
    pub admin1: String,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,
    pub fatalities: u32,
    pub notes: String,
    pub source: String,
}

/// ACLED API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct AcledResponse {
    pub success: bool,
    pub count: u32,
    pub data: Vec<AcledEvent>,
}
