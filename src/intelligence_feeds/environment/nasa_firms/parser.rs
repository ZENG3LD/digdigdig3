//! NASA FIRMS response parsers
//!
//! Parse JSON responses to domain types based on NASA FIRMS API response formats.
//!
//! NASA FIRMS is an environmental data provider, providing near-real-time active
//! fire data from MODIS and VIIRS satellite instruments.

use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct NasaFirmsParser;

impl NasaFirmsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // NASA FIRMS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse fire hotspots
    ///
    /// Example response (array of objects):
    /// ```json
    /// [{
    ///   "latitude": 37.7749,
    ///   "longitude": -122.4194,
    ///   "brightness": 325.5,
    ///   "scan": 1.2,
    ///   "track": 1.1,
    ///   "acq_date": "2024-01-15",
    ///   "acq_time": "1234",
    ///   "satellite": "N",
    ///   "instrument": "VIIRS",
    ///   "confidence": "nominal",
    ///   "version": "2.0NRT",
    ///   "bright_t31": 290.5,
    ///   "frp": 15.3,
    ///   "daynight": "D",
    ///   "type": 0
    /// }]
    /// ```
    pub fn parse_fire_hotspots(response: &Value) -> ExchangeResult<Vec<FireHotspot>> {
        let array = response.as_array().ok_or_else(|| {
            ExchangeError::Parse("Expected array of fire hotspots".to_string())
        })?;

        array
            .iter()
            .map(|fire| {
                let latitude = Self::get_f64(fire, "latitude");
                let longitude = Self::get_f64(fire, "longitude");
                let brightness = Self::get_f64(fire, "brightness")
                    .or_else(|| Self::get_f64(fire, "bright_ti4"))
                    .or_else(|| Self::get_f64(fire, "bright_ti5"));
                let scan = Self::get_f64(fire, "scan");
                let track = Self::get_f64(fire, "track");
                let acq_date = Self::get_str(fire, "acq_date").map(|s| s.to_string());
                let acq_time = Self::get_str(fire, "acq_time").map(|s| s.to_string());
                let satellite = Self::get_str(fire, "satellite").map(|s| s.to_string());
                let instrument = Self::get_str(fire, "instrument").map(|s| s.to_string());
                let confidence = Self::get_str(fire, "confidence").map(|s| s.to_string());
                let frp = Self::get_f64(fire, "frp");
                let daynight = Self::get_str(fire, "daynight").map(|s| s.to_string());
                let country_id = Self::get_str(fire, "country_id")
                    .or_else(|| Self::get_str(fire, "country"))
                    .map(|s| s.to_string());

                Ok(FireHotspot {
                    latitude,
                    longitude,
                    brightness,
                    scan,
                    track,
                    acq_date,
                    acq_time,
                    satellite,
                    instrument,
                    confidence,
                    frp,
                    daynight,
                    country_id,
                })
            })
            .collect()
    }

    /// Parse and summarize fire data by country
    ///
    /// Groups fire hotspots by country and counts them
    pub fn parse_fire_summary(response: &Value) -> ExchangeResult<FireSummary> {
        let hotspots = Self::parse_fire_hotspots(response)?;

        let mut country_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for hotspot in &hotspots {
            if let Some(ref country) = hotspot.country_id {
                *country_counts.entry(country.clone()).or_insert(0) += 1;
            }
        }

        let mut countries: Vec<CountryFireCount> = country_counts
            .into_iter()
            .map(|(country_code, count)| CountryFireCount {
                country_code,
                fire_count: count,
            })
            .collect();

        // Sort by count descending
        countries.sort_by(|a, b| b.fire_count.cmp(&a.fire_count));

        Ok(FireSummary {
            total_fires: hotspots.len() as i64,
            countries,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // NASA FIRMS returns error as object with "error" or "message" field
        if let Some(error) = response.get("error") {
            let message = if let Some(msg) = error.as_str() {
                msg.to_string()
            } else if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
                msg.to_string()
            } else {
                "Unknown error".to_string()
            };

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }

        // Check for error message field directly
        if let Some(message) = response.get("message") {
            if let Some(msg_str) = message.as_str() {
                if msg_str.to_lowercase().contains("error")
                    || msg_str.to_lowercase().contains("invalid")
                    || msg_str.to_lowercase().contains("failed") {
                    return Err(ExchangeError::Api {
                        code: 0,
                        message: msg_str.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NASA FIRMS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Fire hotspot detection from satellite
#[derive(Debug, Clone)]
pub struct FireHotspot {
    /// Latitude of fire detection
    pub latitude: Option<f64>,
    /// Longitude of fire detection
    pub longitude: Option<f64>,
    /// Brightness temperature (Kelvin)
    pub brightness: Option<f64>,
    /// Scan pixel size (km)
    pub scan: Option<f64>,
    /// Track pixel size (km)
    pub track: Option<f64>,
    /// Acquisition date (YYYY-MM-DD)
    pub acq_date: Option<String>,
    /// Acquisition time (HHMM)
    pub acq_time: Option<String>,
    /// Satellite (T=Terra, A=Aqua, N=NOAA-20, S=Suomi-NPP)
    pub satellite: Option<String>,
    /// Instrument (MODIS or VIIRS)
    pub instrument: Option<String>,
    /// Confidence level (nominal, low, high)
    pub confidence: Option<String>,
    /// Fire Radiative Power (MW)
    pub frp: Option<f64>,
    /// Day or Night detection (D/N)
    pub daynight: Option<String>,
    /// Country code (ISO 3166-1 alpha-3)
    pub country_id: Option<String>,
}

/// Summary of fire data by country
#[derive(Debug, Clone)]
pub struct FireSummary {
    /// Total number of fires detected
    pub total_fires: i64,
    /// Fire counts by country (sorted by count descending)
    pub countries: Vec<CountryFireCount>,
}

/// Fire count for a specific country
#[derive(Debug, Clone)]
pub struct CountryFireCount {
    /// Country code (ISO 3166-1 alpha-3)
    pub country_code: String,
    /// Number of fires detected
    pub fire_count: i64,
}
