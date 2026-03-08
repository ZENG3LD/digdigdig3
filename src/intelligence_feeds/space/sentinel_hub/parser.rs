//! Sentinel Hub response parsers
//!
//! Parse JSON responses to domain types based on Sentinel Hub API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct SentinelHubParser;

impl SentinelHubParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CATALOG SEARCH PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse catalog search response (STAC format)
    pub fn parse_catalog_search(response: &Value) -> ExchangeResult<SentinelCatalogResult> {
        let features_val = response.get("features");
        let features = features_val
            .and_then(Value::as_array)
            .ok_or_else(|| ExchangeError::Parse("Missing 'features' field".to_string()))?;

        let parsed_features: Result<Vec<SentinelFeature>, ExchangeError> = features
            .iter()
            .map(Self::parse_feature)
            .collect();

        let context = if let Some(ctx) = response.get("context") {
            Self::parse_context(ctx)?
        } else {
            SentinelContext::default()
        };

        Ok(SentinelCatalogResult {
            features: parsed_features?,
            context,
        })
    }

    fn parse_feature(feature: &Value) -> ExchangeResult<SentinelFeature> {
        let id = Self::get_str(feature, "id").unwrap_or("").to_string();

        let geometry = feature.get("geometry").cloned().unwrap_or(Value::Null);

        let properties = feature.get("properties").cloned().unwrap_or(Value::Object(Default::default()));

        let bbox_val = feature.get("bbox");
        let bbox = bbox_val
            .and_then(Value::as_array)
            .and_then(|arr| {
                if arr.len() >= 4 {
                    Some([
                        arr[0].as_f64()?,
                        arr[1].as_f64()?,
                        arr[2].as_f64()?,
                        arr[3].as_f64()?,
                    ])
                } else {
                    None
                }
            });

        let datetime = properties
            .get("datetime")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let cloud_cover = properties
            .get("eo:cloud_cover")
            .and_then(|v| v.as_f64());

        Ok(SentinelFeature {
            id,
            geometry,
            properties,
            bbox,
            datetime,
            cloud_cover,
        })
    }

    fn parse_context(context: &Value) -> ExchangeResult<SentinelContext> {
        Ok(SentinelContext {
            returned: Self::get_u32(context, "returned").unwrap_or(0),
            limit: Self::get_u32(context, "limit").unwrap_or(0),
            matched: Self::get_u32(context, "matched").unwrap_or(0),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATISTICAL ANALYSIS PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse statistical analysis response
    pub fn parse_statistical(response: &Value) -> ExchangeResult<SentinelStatistical> {
        let data_val = response.get("data");
        let data = data_val
            .and_then(Value::as_array)
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        let mut bands = Vec::new();
        for item in data {
            let outputs_val = item.get("outputs");
            if let Some(outputs) = outputs_val.and_then(Value::as_object) {
                for (band_name, band_data) in outputs.iter() {
                    let bands_val = band_data.get("bands");
                    if let Some(stats_obj) = bands_val.and_then(Value::as_object) {
                        for (band_key, stats_value) in stats_obj.iter() {
                            let stats_val = stats_value.get("stats");
                            if let Some(stats) = stats_val.and_then(Value::as_object) {
                                bands.push(StatisticalBand {
                                    name: format!("{}_{}", band_name, band_key),
                                    stats: BandStats {
                                        min: stats.get("min").and_then(|v| v.as_f64()),
                                        max: stats.get("max").and_then(|v| v.as_f64()),
                                        mean: stats.get("mean").and_then(|v| v.as_f64()),
                                        stdev: stats.get("stDev").and_then(|v| v.as_f64()),
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(SentinelStatistical { data: bands })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|n| n as u32)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SENTINEL HUB TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Catalog search result (STAC format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelCatalogResult {
    pub features: Vec<SentinelFeature>,
    pub context: SentinelContext,
}

/// STAC feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelFeature {
    pub id: String,
    pub geometry: Value,
    pub properties: Value,
    #[serde(default)]
    pub bbox: Option<[f64; 4]>,
    pub datetime: String,
    #[serde(default)]
    pub cloud_cover: Option<f64>,
}

/// STAC context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentinelContext {
    pub returned: u32,
    pub limit: u32,
    pub matched: u32,
}

/// Statistical analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelStatistical {
    pub data: Vec<StatisticalBand>,
}

/// Statistical band data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalBand {
    pub name: String,
    pub stats: BandStats,
}

/// Band statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandStats {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub mean: Option<f64>,
    #[serde(default)]
    pub stdev: Option<f64>,
}
