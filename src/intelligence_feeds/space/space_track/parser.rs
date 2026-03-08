//! Space-Track response parsers
//!
//! Parse JSON responses to domain types based on Space-Track API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct SpaceTrackParser;

impl SpaceTrackParser {
    // ═══════════════════════════════════════════════════════════════════════
    // SPACE-TRACK SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse satellite catalog data
    ///
    /// Example response:
    /// ```json
    /// [{
    ///   "NORAD_CAT_ID": "25544",
    ///   "OBJECT_NAME": "ISS (ZARYA)",
    ///   "OBJECT_TYPE": "PAYLOAD",
    ///   "COUNTRY": "ISS",
    ///   "LAUNCH": "1998-11-20",
    ///   "DECAY": null,
    ///   "PERIOD": "92.68",
    ///   "INCLINATION": "51.64",
    ///   "APOGEE": "421",
    ///   "PERIGEE": "418",
    ///   "RCS_SIZE": "LARGE"
    /// }]
    /// ```
    pub fn parse_satellites(response: &Value) -> ExchangeResult<Vec<Satellite>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        array
            .iter()
            .map(|obj| {
                Ok(Satellite {
                    norad_cat_id: Self::get_u32(obj, "NORAD_CAT_ID")
                        .ok_or_else(|| ExchangeError::Parse("Missing NORAD_CAT_ID".to_string()))?,
                    object_name: Self::get_str(obj, "OBJECT_NAME").unwrap_or("UNKNOWN").to_string(),
                    object_type: Self::get_str(obj, "OBJECT_TYPE").map(|s| s.to_string()),
                    country_code: Self::get_str(obj, "COUNTRY").map(|s| s.to_string()),
                    launch_date: Self::get_str(obj, "LAUNCH").map(|s| s.to_string()),
                    decay_date: Self::get_str(obj, "DECAY").map(|s| s.to_string()),
                    period: Self::get_f64(obj, "PERIOD"),
                    inclination: Self::get_f64(obj, "INCLINATION"),
                    apogee: Self::get_f64(obj, "APOGEE"),
                    perigee: Self::get_f64(obj, "PERIGEE"),
                    rcs_size: Self::get_str(obj, "RCS_SIZE").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse decay prediction data
    ///
    /// Example response:
    /// ```json
    /// [{
    ///   "NORAD_CAT_ID": "12345",
    ///   "OBJECT_NAME": "COSMOS 1234",
    ///   "DECAY_EPOCH": "2024-01-15 12:34:56",
    ///   "MSG_EPOCH": "2024-01-14 00:00:00",
    ///   "SOURCE": "USSTRATCOM"
    /// }]
    /// ```
    pub fn parse_decay_predictions(response: &Value) -> ExchangeResult<Vec<DecayPrediction>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        array
            .iter()
            .map(|obj| {
                Ok(DecayPrediction {
                    norad_cat_id: Self::get_u32(obj, "NORAD_CAT_ID")
                        .ok_or_else(|| ExchangeError::Parse("Missing NORAD_CAT_ID".to_string()))?,
                    object_name: Self::get_str(obj, "OBJECT_NAME").unwrap_or("UNKNOWN").to_string(),
                    decay_epoch: Self::get_str(obj, "DECAY_EPOCH").unwrap_or("").to_string(),
                    msg_epoch: Self::get_str(obj, "MSG_EPOCH").unwrap_or("").to_string(),
                    source: Self::get_str(obj, "SOURCE").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse TLE (Two-Line Element) data
    ///
    /// Example response:
    /// ```json
    /// [{
    ///   "NORAD_CAT_ID": "25544",
    ///   "TLE_LINE1": "1 25544U 98067A   24015.12345678  .00012345  00000-0  12345-3 0  9999",
    ///   "TLE_LINE2": "2 25544  51.6400 123.4567 0001234  12.3456 347.6543 15.54012345123456",
    ///   "EPOCH": "2024-01-15 12:34:56",
    ///   "MEAN_MOTION": "15.54012345",
    ///   "ECCENTRICITY": "0.0001234",
    ///   "INCLINATION": "51.6400"
    /// }]
    /// ```
    pub fn parse_tle_data(response: &Value) -> ExchangeResult<Vec<TleData>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        array
            .iter()
            .map(|obj| {
                Ok(TleData {
                    norad_cat_id: Self::get_u32(obj, "NORAD_CAT_ID")
                        .ok_or_else(|| ExchangeError::Parse("Missing NORAD_CAT_ID".to_string()))?,
                    tle_line1: Self::get_str(obj, "TLE_LINE1").unwrap_or("").to_string(),
                    tle_line2: Self::get_str(obj, "TLE_LINE2").unwrap_or("").to_string(),
                    epoch: Self::get_str(obj, "EPOCH").map(|s| s.to_string()),
                    mean_motion: Self::get_f64(obj, "MEAN_MOTION"),
                    eccentricity: Self::get_f64(obj, "ECCENTRICITY"),
                    inclination: Self::get_f64(obj, "INCLINATION"),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| {
            if let Some(num) = v.as_u64() {
                Some(num as u32)
            } else if let Some(s) = v.as_str() {
                s.parse::<u32>().ok()
            } else {
                None
            }
        })
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            if let Some(num) = v.as_f64() {
                Some(num)
            } else if let Some(s) = v.as_str() {
                s.parse::<f64>().ok()
            } else {
                None
            }
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPACE-TRACK SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Satellite catalog entry
#[derive(Debug, Clone)]
pub struct Satellite {
    pub norad_cat_id: u32,
    pub object_name: String,
    pub object_type: Option<String>,
    pub country_code: Option<String>,
    pub launch_date: Option<String>,
    pub decay_date: Option<String>,
    pub period: Option<f64>,
    pub inclination: Option<f64>,
    pub apogee: Option<f64>,
    pub perigee: Option<f64>,
    pub rcs_size: Option<String>,
}

/// Decay prediction entry
#[derive(Debug, Clone)]
pub struct DecayPrediction {
    pub norad_cat_id: u32,
    pub object_name: String,
    pub decay_epoch: String,
    pub msg_epoch: String,
    pub source: Option<String>,
}

/// TLE (Two-Line Element) data
#[derive(Debug, Clone)]
pub struct TleData {
    pub norad_cat_id: u32,
    pub tle_line1: String,
    pub tle_line2: String,
    pub epoch: Option<String>,
    pub mean_motion: Option<f64>,
    pub eccentricity: Option<f64>,
    pub inclination: Option<f64>,
}
