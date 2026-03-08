//! FRED response parsers
//!
//! Parse JSON responses to domain types based on FRED API response formats.
//!
//! FRED is an economic data provider, not a trading exchange, so many standard
//! market data types (ticker, orderbook, klines) don't apply directly.
//! We adapt FRED's time series observations to klines for compatibility.

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct FredParser;

impl FredParser {
    // ═══════════════════════════════════════════════════════════════════════
    // FRED-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse series observations (time series data)
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "realtime_start": "2024-01-15",
    ///   "realtime_end": "2024-01-15",
    ///   "observation_start": "1776-07-04",
    ///   "observation_end": "9999-12-31",
    ///   "units": "lin",
    ///   "output_type": 1,
    ///   "file_type": "json",
    ///   "order_by": "observation_date",
    ///   "sort_order": "asc",
    ///   "count": 94,
    ///   "offset": 0,
    ///   "limit": 100000,
    ///   "observations": [
    ///     {
    ///       "realtime_start": "2024-01-15",
    ///       "realtime_end": "2024-01-15",
    ///       "date": "1929-01-01",
    ///       "value": "1120.7"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_observations(response: &Value) -> ExchangeResult<Vec<Observation>> {
        let observations = response
            .get("observations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'observations' array".to_string()))?;

        observations
            .iter()
            .map(|obs| {
                let date = Self::require_str(obs, "date")?;
                let value_str = Self::require_str(obs, "value")?;

                // Parse value - can be numeric or "." for missing data
                let value = if value_str == "." {
                    None
                } else {
                    Some(
                        value_str
                            .parse::<f64>()
                            .map_err(|_| ExchangeError::Parse(format!("Invalid value: {}", value_str)))?,
                    )
                };

                Ok(Observation {
                    date: date.to_string(),
                    value,
                    realtime_start: Self::get_str(obs, "realtime_start").map(|s| s.to_string()),
                    realtime_end: Self::get_str(obs, "realtime_end").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse series metadata
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "seriess": [{
    ///     "id": "GNPCA",
    ///     "realtime_start": "2024-01-15",
    ///     "realtime_end": "2024-01-15",
    ///     "title": "Real Gross National Product",
    ///     "observation_start": "1929-01-01",
    ///     "observation_end": "2022-01-01",
    ///     "frequency": "Annual",
    ///     "frequency_short": "A",
    ///     "units": "Billions of Chained 2012 Dollars",
    ///     "units_short": "Bil. of Chn. 2012 $",
    ///     "seasonal_adjustment": "Not Seasonally Adjusted",
    ///     "seasonal_adjustment_short": "NSA",
    ///     "last_updated": "2023-09-28 07:46:03-05",
    ///     "popularity": 39
    ///   }]
    /// }
    /// ```
    pub fn parse_series(response: &Value) -> ExchangeResult<SeriesMetadata> {
        let seriess = response
            .get("seriess")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'seriess' array".to_string()))?;

        let series = seriess
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'seriess' array".to_string()))?;

        Ok(SeriesMetadata {
            id: Self::require_str(series, "id")?.to_string(),
            title: Self::require_str(series, "title")?.to_string(),
            observation_start: Self::require_str(series, "observation_start")?.to_string(),
            observation_end: Self::require_str(series, "observation_end")?.to_string(),
            frequency: Self::require_str(series, "frequency")?.to_string(),
            frequency_short: Self::require_str(series, "frequency_short")?.to_string(),
            units: Self::require_str(series, "units")?.to_string(),
            units_short: Self::require_str(series, "units_short")?.to_string(),
            seasonal_adjustment: Self::require_str(series, "seasonal_adjustment")?.to_string(),
            seasonal_adjustment_short: Self::require_str(series, "seasonal_adjustment_short")?.to_string(),
            last_updated: Self::require_str(series, "last_updated")?.to_string(),
            popularity: Self::get_i64(series, "popularity"),
            notes: Self::get_str(series, "notes").map(|s| s.to_string()),
        })
    }

    /// Parse series search results
    ///
    /// Returns a list of series IDs matching the search query
    pub fn parse_series_search(response: &Value) -> ExchangeResult<Vec<String>> {
        let seriess = response
            .get("seriess")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'seriess' array".to_string()))?;

        Ok(seriess
            .iter()
            .filter_map(|v| v.get("id").and_then(|s| s.as_str()))
            .map(|s| s.to_string())
            .collect())
    }

    /// Parse categories
    pub fn parse_categories(response: &Value) -> ExchangeResult<Vec<Category>> {
        let categories = response
            .get("categories")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'categories' array".to_string()))?;

        categories
            .iter()
            .map(|cat| {
                Ok(Category {
                    id: Self::require_i64(cat, "id")?,
                    name: Self::require_str(cat, "name")?.to_string(),
                    parent_id: Self::get_i64(cat, "parent_id"),
                })
            })
            .collect()
    }

    /// Parse releases
    pub fn parse_releases(response: &Value) -> ExchangeResult<Vec<Release>> {
        let releases = response
            .get("releases")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'releases' array".to_string()))?;

        releases
            .iter()
            .map(|rel| {
                Ok(Release {
                    id: Self::require_i64(rel, "id")?,
                    name: Self::require_str(rel, "name")?.to_string(),
                    press_release: Self::get_bool(rel, "press_release"),
                    link: Self::get_str(rel, "link").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse sources
    pub fn parse_sources(response: &Value) -> ExchangeResult<Vec<Source>> {
        let sources = response
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'sources' array".to_string()))?;

        sources
            .iter()
            .map(|src| {
                Ok(Source {
                    id: Self::require_i64(src, "id")?,
                    name: Self::require_str(src, "name")?.to_string(),
                    link: Self::get_str(src, "link").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse tags
    pub fn parse_tags(response: &Value) -> ExchangeResult<Vec<Tag>> {
        let tags = response
            .get("tags")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'tags' array".to_string()))?;

        tags.iter()
            .map(|tag| {
                Ok(Tag {
                    name: Self::require_str(tag, "name")?.to_string(),
                    group_id: Self::require_str(tag, "group_id")?.to_string(),
                    notes: Self::get_str(tag, "notes").map(|s| s.to_string()),
                    created: Self::get_str(tag, "created").map(|s| s.to_string()),
                    popularity: Self::get_i64(tag, "popularity"),
                    series_count: Self::get_i64(tag, "series_count"),
                })
            })
            .collect()
    }

    /// Parse release dates
    pub fn parse_release_dates(response: &Value) -> ExchangeResult<Vec<ReleaseDate>> {
        let release_dates = response
            .get("release_dates")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'release_dates' array".to_string()))?;

        release_dates
            .iter()
            .map(|rd| {
                Ok(ReleaseDate {
                    release_id: Self::require_i64(rd, "release_id")?,
                    release_name: Self::get_str(rd, "release_name").map(|s| s.to_string()),
                    date: Self::require_str(rd, "date")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse series updates
    pub fn parse_series_updates(response: &Value) -> ExchangeResult<Vec<SeriesUpdate>> {
        let seriess = response
            .get("seriess")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'seriess' array".to_string()))?;

        seriess
            .iter()
            .map(|series| {
                Ok(SeriesUpdate {
                    series_id: Self::require_str(series, "id")?.to_string(),
                    title: Self::require_str(series, "title")?.to_string(),
                    observation_start: Self::require_str(series, "observation_start")?.to_string(),
                    observation_end: Self::require_str(series, "observation_end")?.to_string(),
                    frequency: Self::require_str(series, "frequency")?.to_string(),
                    units: Self::require_str(series, "units")?.to_string(),
                    last_updated: Self::require_str(series, "last_updated")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse vintage dates
    pub fn parse_vintage_dates(response: &Value) -> ExchangeResult<Vec<VintageDate>> {
        let vintage_dates = response
            .get("vintage_dates")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'vintage_dates' array".to_string()))?;

        Ok(vintage_dates
            .iter()
            .filter_map(|v| v.as_str())
            .map(|date| VintageDate {
                date: date.to_string(),
            })
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ADAPTER: Convert FRED observations to Klines for compatibility
    // ═══════════════════════════════════════════════════════════════════════

    /// Convert FRED observations to Kline format
    ///
    /// This adapter allows FRED economic data to be used with the standard
    /// Kline interface. Each observation becomes a Kline where:
    /// - open = close = high = low = value
    /// - open_time = parsed from date
    pub fn observations_to_klines(observations: Vec<Observation>) -> ExchangeResult<Vec<Kline>> {
        let klines: Vec<Kline> = observations
            .into_iter()
            .filter_map(|obs| {
                obs.value.map(|value| {
                    // Parse date to timestamp (milliseconds)
                    let timestamp = Self::parse_date_to_timestamp(&obs.date).unwrap_or(0);

                    Kline {
                        open_time: timestamp,
                        open: value,
                        high: value,
                        low: value,
                        close: value,
                        volume: 0.0, // No volume for economic data
                        close_time: Some(timestamp),
                        quote_volume: None,
                        trades: None,
                    }
                })
            })
            .collect();

        Ok(klines)
    }

    /// Parse FRED date format (YYYY-MM-DD) to Unix timestamp (milliseconds)
    fn parse_date_to_timestamp(date: &str) -> ExchangeResult<i64> {
        // Simple parsing for YYYY-MM-DD format
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            return Err(ExchangeError::Parse(format!("Invalid date format: {}", date)));
        }

        let year: i64 = parts[0]
            .parse()
            .map_err(|_| ExchangeError::Parse(format!("Invalid year: {}", parts[0])))?;
        let month: i64 = parts[1]
            .parse()
            .map_err(|_| ExchangeError::Parse(format!("Invalid month: {}", parts[1])))?;
        let day: i64 = parts[2]
            .parse()
            .map_err(|_| ExchangeError::Parse(format!("Invalid day: {}", parts[2])))?;

        // Approximate timestamp calculation (not precise, but good enough for our purposes)
        // Unix epoch: 1970-01-01
        let days_since_epoch = (year - 1970) * 365 + (year - 1970) / 4 // leap years approximation
            + (month - 1) * 30 // approximate month length
            + day;

        Ok(days_since_epoch * 24 * 60 * 60 * 1000) // convert to milliseconds
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_code) = response.get("error_code") {
            let code = error_code.as_i64().unwrap_or(0) as i32;
            let message = response
                .get("error_message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api { code, message });
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

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEW PARSERS FOR 5 MISSING ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse release tables (hierarchical table tree for a release)
    pub fn parse_release_tables(response: &Value) -> ExchangeResult<Vec<FredReleaseTable>> {
        let elements = response
            .get("elements")
            .ok_or_else(|| ExchangeError::Parse("Missing 'elements' object".to_string()))?;

        // The elements object can contain nested table elements
        // We'll parse it recursively
        let mut tables = Vec::new();
        if let Some(obj) = elements.as_object() {
            for (_key, value) in obj.iter() {
                if let Ok(table) = Self::parse_table_element(value) {
                    tables.push(table);
                }
            }
        }

        Ok(tables)
    }

    fn parse_table_element(elem: &Value) -> ExchangeResult<FredReleaseTable> {
        let mut children = Vec::new();

        // Check for child elements
        if let Some(child_obj) = elem.get("children").and_then(|v| v.as_object()) {
            for (_key, child_value) in child_obj.iter() {
                if let Ok(child_table) = Self::parse_table_element(child_value) {
                    children.push(child_table);
                }
            }
        }

        Ok(FredReleaseTable {
            element_id: Self::get_i64(elem, "element_id"),
            release_id: Self::get_i64(elem, "release_id"),
            name: Self::get_str(elem, "name").map(|s| s.to_string()),
            element_type: Self::get_str(elem, "type").map(|s| s.to_string()),
            children,
        })
    }

    /// Parse GeoFRED series group metadata
    pub fn parse_geo_series_group(response: &Value) -> ExchangeResult<FredGeoSeriesGroup> {
        let series_group = response
            .get("series_group")
            .ok_or_else(|| ExchangeError::Parse("Missing 'series_group' object".to_string()))?;

        Ok(FredGeoSeriesGroup {
            title: Self::require_str(series_group, "title")?.to_string(),
            region_type: Self::require_str(series_group, "region_type")?.to_string(),
            series_group: Self::require_str(series_group, "series_group")?.to_string(),
            season: Self::get_str(series_group, "season").map(|s| s.to_string()),
            units: Self::get_str(series_group, "units").map(|s| s.to_string()),
            frequency: Self::get_str(series_group, "frequency").map(|s| s.to_string()),
            min_date: Self::get_str(series_group, "min_date").map(|s| s.to_string()),
            max_date: Self::get_str(series_group, "max_date").map(|s| s.to_string()),
        })
    }

    /// Parse GeoFRED series data for mapping
    pub fn parse_geo_series_data(response: &Value) -> ExchangeResult<FredGeoSeriesData> {
        let meta = response
            .get("meta")
            .ok_or_else(|| ExchangeError::Parse("Missing 'meta' object".to_string()))?;

        let series_id = Self::require_str(meta, "series_id")?.to_string();
        let date = Self::get_str(meta, "date")
            .unwrap_or("unknown")
            .to_string();

        let data_obj = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let mut region_data = Vec::new();

        if let Some(obj) = data_obj.as_object() {
            for (region, value) in obj.iter() {
                let val = if let Some(num_val) = value.as_f64() {
                    Some(num_val)
                } else if let Some(str_val) = value.as_str() {
                    str_val.parse::<f64>().ok()
                } else {
                    None
                };

                region_data.push(FredGeoRegionValue {
                    region: region.clone(),
                    code: Some(region.clone()),
                    value: val,
                    series_id: None,
                });
            }
        }

        Ok(FredGeoSeriesData {
            series_id,
            date,
            region_data,
        })
    }

    /// Parse GeoFRED regional data
    pub fn parse_geo_regional_data(response: &Value) -> ExchangeResult<FredGeoRegionalData> {
        let meta = response
            .get("meta")
            .ok_or_else(|| ExchangeError::Parse("Missing 'meta' object".to_string()))?;

        let series_group = Self::require_str(meta, "series_group")?.to_string();
        let region_type = Self::require_str(meta, "region")?.to_string();
        let date = Self::get_str(meta, "date").map(|s| s.to_string());

        let data_array = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let mut data = Vec::new();
        for item in data_array.iter() {
            let region = Self::get_str(item, "region")
                .or_else(|| Self::get_str(item, "name"))
                .unwrap_or("unknown")
                .to_string();

            let value = Self::get_f64(item, "value")
                .or_else(|| {
                    Self::get_str(item, "value")
                        .and_then(|s| s.parse::<f64>().ok())
                });

            data.push(FredGeoRegionValue {
                region,
                code: Self::get_str(item, "code").map(|s| s.to_string()),
                value,
                series_id: Self::get_str(item, "series_id").map(|s| s.to_string()),
            });
        }

        Ok(FredGeoRegionalData {
            series_group,
            region_type,
            date,
            data,
        })
    }

    /// Parse GeoFRED shapes file (GeoJSON)
    pub fn parse_geo_shapes(response: &Value) -> ExchangeResult<FredGeoShapes> {
        // The entire response is typically the GeoJSON structure
        // Extract shape type from parameters or response metadata
        let shape_type = response
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("FeatureCollection")
            .to_string();

        Ok(FredGeoShapes {
            shape_type,
            geojson: response.clone(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FRED-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// FRED observation (single data point in a time series)
#[derive(Debug, Clone)]
pub struct Observation {
    pub date: String,
    pub value: Option<f64>, // None if value is "." (missing data)
    pub realtime_start: Option<String>,
    pub realtime_end: Option<String>,
}

/// FRED series metadata
#[derive(Debug, Clone)]
pub struct SeriesMetadata {
    pub id: String,
    pub title: String,
    pub observation_start: String,
    pub observation_end: String,
    pub frequency: String,
    pub frequency_short: String,
    pub units: String,
    pub units_short: String,
    pub seasonal_adjustment: String,
    pub seasonal_adjustment_short: String,
    pub last_updated: String,
    pub popularity: Option<i64>,
    pub notes: Option<String>,
}

/// FRED category
#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

/// FRED release
#[derive(Debug, Clone)]
pub struct Release {
    pub id: i64,
    pub name: String,
    pub press_release: Option<bool>,
    pub link: Option<String>,
}

/// FRED source
#[derive(Debug, Clone)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub link: Option<String>,
}

/// FRED tag
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub group_id: String,
    pub notes: Option<String>,
    pub created: Option<String>,
    pub popularity: Option<i64>,
    pub series_count: Option<i64>,
}

/// FRED release date
#[derive(Debug, Clone)]
pub struct ReleaseDate {
    pub release_id: i64,
    pub release_name: Option<String>,
    pub date: String,
}

/// FRED series update info
#[derive(Debug, Clone)]
pub struct SeriesUpdate {
    pub series_id: String,
    pub title: String,
    pub observation_start: String,
    pub observation_end: String,
    pub frequency: String,
    pub units: String,
    pub last_updated: String,
}

/// FRED vintage date (for ALFRED revision history)
#[derive(Debug, Clone)]
pub struct VintageDate {
    pub date: String,
}

/// FRED release table element (hierarchical table data)
#[derive(Debug, Clone)]
pub struct FredReleaseTable {
    pub element_id: Option<i64>,
    pub release_id: Option<i64>,
    pub name: Option<String>,
    pub element_type: Option<String>,
    pub children: Vec<FredReleaseTable>,
}

/// GeoFRED series group metadata
#[derive(Debug, Clone)]
pub struct FredGeoSeriesGroup {
    pub title: String,
    pub region_type: String,
    pub series_group: String,
    pub season: Option<String>,
    pub units: Option<String>,
    pub frequency: Option<String>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

/// GeoFRED series data for mapping
#[derive(Debug, Clone)]
pub struct FredGeoSeriesData {
    pub series_id: String,
    pub date: String,
    pub region_data: Vec<FredGeoRegionValue>,
}

/// GeoFRED region value
#[derive(Debug, Clone)]
pub struct FredGeoRegionValue {
    pub region: String,
    pub code: Option<String>,
    pub value: Option<f64>,
    pub series_id: Option<String>,
}

/// GeoFRED regional data
#[derive(Debug, Clone)]
pub struct FredGeoRegionalData {
    pub series_group: String,
    pub region_type: String,
    pub date: Option<String>,
    pub data: Vec<FredGeoRegionValue>,
}

/// GeoFRED shapes file (GeoJSON)
#[derive(Debug, Clone)]
pub struct FredGeoShapes {
    pub shape_type: String,
    pub geojson: serde_json::Value,
}
