//! OFAC API response parsers
//!
//! Parse JSON responses to domain types based on OFAC API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OfacParser;

/// Sanctioned entity from OFAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfacEntity {
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    pub source: Option<String>,
    pub programs: Option<Vec<String>>,
    pub addresses: Option<Vec<String>>,
    pub aliases: Option<Vec<String>>,
    pub ids: Option<Vec<String>>,
    pub score: Option<f64>,
}

/// Search result from OFAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfacSearchResult {
    pub total: i32,
    pub matches: Vec<OfacEntity>,
}

/// Screen result from OFAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfacScreenResult {
    pub is_match: bool,
    pub matches: Vec<OfacEntity>,
    pub score: Option<f64>,
}

/// OFAC sanction source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfacSource {
    pub name: String,
    pub description: Option<String>,
    pub last_updated: Option<String>,
}

impl OfacParser {
    /// Check for API errors in response
    ///
    /// OFAC API error format:
    /// ```json
    /// {
    ///   "error": true,
    ///   "message": "Error message",
    ///   "code": 400
    /// }
    /// ```
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error").and_then(|v| v.as_bool()) {
            if error {
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                let code = response
                    .get("code")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                return Err(ExchangeError::Api {
                    code,
                    message: message.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Parse search results from response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 5,
    ///   "matches": [
    ///     {
    ///       "name": "PUTIN, Vladimir Vladimirovich",
    ///       "type": "individual",
    ///       "source": "SDN",
    ///       "programs": ["UKRAINE-EO13661"],
    ///       "addresses": ["The Kremlin, Moscow, Russia"],
    ///       "aliases": ["PUTIN, Vladimir"],
    ///       "ids": ["1234567"],
    ///       "score": 0.95
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_search_result(response: &Value) -> ExchangeResult<OfacSearchResult> {
        Self::check_error(response)?;

        let total = response
            .get("total")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let matches = response
            .get("matches")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| Self::parse_entity(item).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(OfacSearchResult { total, matches })
    }

    /// Parse screen result from response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "isMatch": true,
    ///   "matches": [...],
    ///   "score": 0.85
    /// }
    /// ```
    pub fn parse_screen_result(response: &Value) -> ExchangeResult<OfacScreenResult> {
        Self::check_error(response)?;

        let is_match = response
            .get("isMatch")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let score = response.get("score").and_then(|v| v.as_f64());

        let matches = response
            .get("matches")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| Self::parse_entity(item).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(OfacScreenResult {
            is_match,
            matches,
            score,
        })
    }

    /// Parse entity from JSON
    fn parse_entity(entity: &Value) -> ExchangeResult<OfacEntity> {
        Ok(OfacEntity {
            name: Self::require_str(entity, "name")?.to_string(),
            entity_type: Self::get_str(entity, "type").map(|s| s.to_string()),
            source: Self::get_str(entity, "source").map(|s| s.to_string()),
            programs: entity.get("programs").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            addresses: entity.get("addresses").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            aliases: entity.get("aliases").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            ids: entity.get("ids").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            score: entity.get("score").and_then(|v| v.as_f64()),
        })
    }

    /// Parse sources array from /sources endpoint
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "sources": [
    ///     {
    ///       "name": "SDN",
    ///       "description": "Specially Designated Nationals",
    ///       "lastUpdated": "2024-01-15"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_sources(response: &Value) -> ExchangeResult<Vec<OfacSource>> {
        Self::check_error(response)?;

        let sources = response
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'sources' array".to_string()))?;

        sources
            .iter()
            .map(|source| {
                Ok(OfacSource {
                    name: Self::require_str(source, "name")?.to_string(),
                    description: Self::get_str(source, "description").map(|s| s.to_string()),
                    last_updated: Self::get_str(source, "lastUpdated").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn _require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }
}
