//! GFW response parsers
//!
//! Parse JSON responses to domain types based on GFW API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct GfwParser;

impl GfwParser {
    // ═══════════════════════════════════════════════════════════════════════
    // DATASET PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse datasets list
    pub fn parse_datasets(response: &Value) -> ExchangeResult<Vec<GfwDataset>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|dataset| {
                Ok(GfwDataset {
                    id: Self::require_str(dataset, "id")?.to_string(),
                    name: Self::require_str(dataset, "name")?.to_string(),
                    slug: Self::require_str(dataset, "slug")?.to_string(),
                    description: Self::get_str(dataset, "description").map(|s| s.to_string()),
                    application: Self::get_str(dataset, "application").map(|s| s.to_string()),
                    subtitle: Self::get_str(dataset, "subtitle").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single dataset
    pub fn parse_dataset(response: &Value) -> ExchangeResult<GfwDataset> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(GfwDataset {
            id: Self::require_str(data, "id")?.to_string(),
            name: Self::require_str(data, "name")?.to_string(),
            slug: Self::require_str(data, "slug")?.to_string(),
            description: Self::get_str(data, "description").map(|s| s.to_string()),
            application: Self::get_str(data, "application").map(|s| s.to_string()),
            subtitle: Self::get_str(data, "subtitle").map(|s| s.to_string()),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FOREST CHANGE PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse tree cover loss data
    pub fn parse_tree_cover_loss(response: &Value) -> ExchangeResult<Vec<GfwTreeCoverLoss>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|item| {
                Ok(GfwTreeCoverLoss {
                    year: Self::require_u32(item, "year")?,
                    area_ha: Self::require_f64(item, "area_ha")?,
                    emissions_tonnes: Self::get_f64(item, "emissions_tonnes"),
                })
            })
            .collect()
    }

    /// Parse tree cover gain data
    pub fn parse_tree_cover_gain(response: &Value) -> ExchangeResult<GfwTreeCoverGain> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(GfwTreeCoverGain {
            area_ha: Self::require_f64(data, "area_ha")?,
            gain_start_year: Self::require_u32(data, "gain_start_year")?,
            gain_end_year: Self::require_u32(data, "gain_end_year")?,
        })
    }

    /// Parse forest statistics
    pub fn parse_forest_stats(response: &Value) -> ExchangeResult<GfwForestStats> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(GfwForestStats {
            tree_cover_area_ha: Self::require_f64(data, "tree_cover_area_ha")?,
            tree_cover_loss_ha: Self::require_f64(data, "tree_cover_loss_ha")?,
            tree_cover_gain_ha: Self::get_f64(data, "tree_cover_gain_ha"),
            co2_emissions: Self::get_f64(data, "co2_emissions"),
        })
    }

    /// Parse alerts (fire or deforestation)
    pub fn parse_alerts(response: &Value) -> ExchangeResult<Vec<GfwAlert>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|alert| {
                Ok(GfwAlert {
                    date: Self::require_str(alert, "date")?.to_string(),
                    latitude: Self::require_f64(alert, "latitude")?,
                    longitude: Self::require_f64(alert, "longitude")?,
                    confidence: Self::get_str(alert, "confidence").map(|s| s.to_string()),
                    alert_type: Self::get_str(alert, "type").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse country statistics (for global deforestation queries)
    pub fn parse_country_stats(response: &Value) -> ExchangeResult<Vec<GfwCountryStats>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|item| {
                Ok(GfwCountryStats {
                    iso_code: Self::require_str(item, "iso")?.to_string(),
                    country_name: Self::get_str(item, "name").map(|s| s.to_string()),
                    tree_cover_loss_ha: Self::require_f64(item, "tree_cover_loss_ha")?,
                    year: Self::get_u32(item, "year"),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(errors) = response.get("errors") {
            if let Some(error_array) = errors.as_array() {
                if let Some(first_error) = error_array.first() {
                    let message = first_error
                        .get("detail")
                        .or_else(|| first_error.get("title"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    let code = first_error
                        .get("status")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i32>().ok())
                        .unwrap_or(500);

                    return Err(ExchangeError::Api { code, message });
                }
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

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64().map(|n| n as u32).or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64().map(|n| n as u32).or_else(|| v.as_str().and_then(|s| s.parse().ok())))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GFW-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// GFW Dataset metadata
#[derive(Debug, Clone)]
pub struct GfwDataset {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub application: Option<String>,
    pub subtitle: Option<String>,
}

/// Tree cover loss data
#[derive(Debug, Clone)]
pub struct GfwTreeCoverLoss {
    pub year: u32,
    pub area_ha: f64,
    pub emissions_tonnes: Option<f64>,
}

/// Tree cover gain data
#[derive(Debug, Clone)]
pub struct GfwTreeCoverGain {
    pub area_ha: f64,
    pub gain_start_year: u32,
    pub gain_end_year: u32,
}

/// Forest statistics
#[derive(Debug, Clone)]
pub struct GfwForestStats {
    pub tree_cover_area_ha: f64,
    pub tree_cover_loss_ha: f64,
    pub tree_cover_gain_ha: Option<f64>,
    pub co2_emissions: Option<f64>,
}

/// Forest alert (fire or deforestation)
#[derive(Debug, Clone)]
pub struct GfwAlert {
    pub date: String,
    pub latitude: f64,
    pub longitude: f64,
    pub confidence: Option<String>,
    pub alert_type: Option<String>,
}

/// Country statistics
#[derive(Debug, Clone)]
pub struct GfwCountryStats {
    pub iso_code: String,
    pub country_name: Option<String>,
    pub tree_cover_loss_ha: f64,
    pub year: Option<u32>,
}
