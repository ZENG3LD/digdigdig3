//! NASA response parsers
//!
//! Parse JSON responses to domain types based on NASA API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct NasaParser;

impl NasaParser {
    // ═══════════════════════════════════════════════════════════════════════
    // NEO PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse NEO feed response
    pub fn parse_neo_feed(response: &Value) -> ExchangeResult<Vec<NeoObject>> {
        let near_earth_objects = response
            .get("near_earth_objects")
            .ok_or_else(|| ExchangeError::Parse("Missing 'near_earth_objects' field".to_string()))?;

        let mut all_neos = Vec::new();

        if let Some(obj) = near_earth_objects.as_object() {
            for (_date, neos_array) in obj.iter() {
                if let Some(neos) = neos_array.as_array() {
                    for neo in neos {
                        all_neos.push(Self::parse_neo_object(neo)?);
                    }
                }
            }
        }

        Ok(all_neos)
    }

    /// Parse single NEO object
    pub fn parse_neo_object(neo: &Value) -> ExchangeResult<NeoObject> {
        let id = Self::require_str(neo, "id")?.to_string();
        let name = Self::require_str(neo, "name")?.to_string();
        let nasa_jpl_url = Self::get_str(neo, "nasa_jpl_url").unwrap_or("").to_string();
        let is_potentially_hazardous = Self::get_bool(neo, "is_potentially_hazardous_asteroid").unwrap_or(false);

        // Parse diameter
        let estimated_diameter = neo.get("estimated_diameter").and_then(|d| d.as_object());
        let (diameter_km_min, diameter_km_max) = if let Some(diam) = estimated_diameter {
            let km = diam.get("kilometers").and_then(|k| k.as_object());
            if let Some(km_obj) = km {
                (
                    km_obj.get("estimated_diameter_min").and_then(|v| v.as_f64()),
                    km_obj.get("estimated_diameter_max").and_then(|v| v.as_f64()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Parse close approach data
        let mut close_approach_data = Vec::new();
        if let Some(approaches) = neo.get("close_approach_data").and_then(|v| v.as_array()) {
            for approach in approaches {
                if let Ok(ca) = Self::parse_close_approach(approach) {
                    close_approach_data.push(ca);
                }
            }
        }

        Ok(NeoObject {
            id,
            name,
            nasa_jpl_url,
            estimated_diameter_km_min: diameter_km_min,
            estimated_diameter_km_max: diameter_km_max,
            is_potentially_hazardous,
            close_approach_data,
        })
    }

    fn parse_close_approach(approach: &Value) -> ExchangeResult<CloseApproach> {
        let date = Self::get_str(approach, "close_approach_date").unwrap_or("").to_string();
        let orbiting_body = Self::get_str(approach, "orbiting_body").unwrap_or("").to_string();

        let relative_velocity = approach.get("relative_velocity").and_then(|v| v.as_object());
        let relative_velocity_kph = if let Some(vel) = relative_velocity {
            vel.get("kilometers_per_hour")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
        } else {
            None
        };

        let miss_distance = approach.get("miss_distance").and_then(|v| v.as_object());
        let miss_distance_km = if let Some(dist) = miss_distance {
            dist.get("kilometers")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
        } else {
            None
        };

        Ok(CloseApproach {
            date,
            relative_velocity_kph,
            miss_distance_km,
            orbiting_body,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DONKI PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse solar flares
    pub fn parse_solar_flares(response: &Value) -> ExchangeResult<Vec<SolarFlare>> {
        let flares = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        flares.iter().map(Self::parse_solar_flare).collect()
    }

    fn parse_solar_flare(flare: &Value) -> ExchangeResult<SolarFlare> {
        Ok(SolarFlare {
            flr_id: Self::get_str(flare, "flrID").unwrap_or("").to_string(),
            begin_time: Self::get_str(flare, "beginTime").unwrap_or("").to_string(),
            peak_time: Self::get_str(flare, "peakTime").map(|s| s.to_string()),
            end_time: Self::get_str(flare, "endTime").map(|s| s.to_string()),
            class_type: Self::get_str(flare, "classType").unwrap_or("").to_string(),
            source_location: Self::get_str(flare, "sourceLocation").unwrap_or("").to_string(),
        })
    }

    /// Parse geomagnetic storms
    pub fn parse_geomagnetic_storms(response: &Value) -> ExchangeResult<Vec<GeomagneticStorm>> {
        let storms = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        storms.iter().map(Self::parse_geomagnetic_storm).collect()
    }

    fn parse_geomagnetic_storm(storm: &Value) -> ExchangeResult<GeomagneticStorm> {
        let mut kp_index = Vec::new();
        if let Some(all_kp) = storm.get("allKpIndex").and_then(|v| v.as_array()) {
            for kp in all_kp {
                if let Ok(kp_obj) = Self::parse_kp_index(kp) {
                    kp_index.push(kp_obj);
                }
            }
        }

        Ok(GeomagneticStorm {
            gst_id: Self::get_str(storm, "gstID").unwrap_or("").to_string(),
            start_time: Self::get_str(storm, "startTime").unwrap_or("").to_string(),
            kp_index,
        })
    }

    fn parse_kp_index(kp: &Value) -> ExchangeResult<KpIndex> {
        Ok(KpIndex {
            observed_time: Self::get_str(kp, "observedTime").unwrap_or("").to_string(),
            kp_index: Self::get_f64(kp, "kpIndex").unwrap_or(0.0),
            source: Self::get_str(kp, "source").unwrap_or("").to_string(),
        })
    }

    /// Parse coronal mass ejections
    pub fn parse_coronal_mass_ejections(response: &Value) -> ExchangeResult<Vec<CoronalMassEjection>> {
        let cmes = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        cmes.iter().map(Self::parse_coronal_mass_ejection).collect()
    }

    fn parse_coronal_mass_ejection(cme: &Value) -> ExchangeResult<CoronalMassEjection> {
        // Extract speed from analysis array if available
        let speed = cme.get("cmeAnalyses")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|analysis| Self::get_f64(analysis, "speed"));

        Ok(CoronalMassEjection {
            activity_id: Self::get_str(cme, "activityID").unwrap_or("").to_string(),
            start_time: Self::get_str(cme, "startTime").unwrap_or("").to_string(),
            source_location: Self::get_str(cme, "sourceLocation").unwrap_or("").to_string(),
            note: Self::get_str(cme, "note").unwrap_or("").to_string(),
            speed,
        })
    }

    /// Parse solar energetic particles
    pub fn parse_solar_energetic_particles(response: &Value) -> ExchangeResult<Vec<SolarEnergeticParticle>> {
        let seps = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        seps.iter().map(Self::parse_solar_energetic_particle).collect()
    }

    fn parse_solar_energetic_particle(sep: &Value) -> ExchangeResult<SolarEnergeticParticle> {
        Ok(SolarEnergeticParticle {
            sep_id: Self::get_str(sep, "sepID").unwrap_or("").to_string(),
            event_time: Self::get_str(sep, "eventTime").unwrap_or("").to_string(),
            instruments: Self::get_array_of_strings(sep, "instruments"),
        })
    }

    /// Parse interplanetary shocks
    pub fn parse_interplanetary_shocks(response: &Value) -> ExchangeResult<Vec<InterplanetaryShock>> {
        let shocks = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        shocks.iter().map(Self::parse_interplanetary_shock).collect()
    }

    fn parse_interplanetary_shock(shock: &Value) -> ExchangeResult<InterplanetaryShock> {
        Ok(InterplanetaryShock {
            activity_id: Self::get_str(shock, "activityID").unwrap_or("").to_string(),
            event_time: Self::get_str(shock, "eventTime").unwrap_or("").to_string(),
            location: Self::get_str(shock, "location").unwrap_or("").to_string(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OTHER PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse Astronomy Picture of the Day
    pub fn parse_apod(response: &Value) -> ExchangeResult<Apod> {
        Ok(Apod {
            title: Self::require_str(response, "title")?.to_string(),
            date: Self::require_str(response, "date")?.to_string(),
            explanation: Self::get_str(response, "explanation").unwrap_or("").to_string(),
            url: Self::require_str(response, "url")?.to_string(),
            hdurl: Self::get_str(response, "hdurl").map(|s| s.to_string()),
            media_type: Self::get_str(response, "media_type").unwrap_or("image").to_string(),
            copyright: Self::get_str(response, "copyright").map(|s| s.to_string()),
        })
    }

    /// Parse EPIC Earth imagery
    pub fn parse_earth_imagery(response: &Value) -> ExchangeResult<Vec<EarthImagery>> {
        let images = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))?;

        images.iter().map(Self::parse_earth_image).collect()
    }

    fn parse_earth_image(image: &Value) -> ExchangeResult<EarthImagery> {
        Ok(EarthImagery {
            identifier: Self::get_str(image, "identifier").unwrap_or("").to_string(),
            caption: Self::get_str(image, "caption").unwrap_or("").to_string(),
            image: Self::get_str(image, "image").unwrap_or("").to_string(),
            version: Self::get_str(image, "version").unwrap_or("").to_string(),
            date: Self::get_str(image, "date").unwrap_or("").to_string(),
        })
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

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    fn get_array_of_strings(obj: &Value, field: &str) -> Vec<String> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NASA-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Near Earth Object (asteroid)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoObject {
    pub id: String,
    pub name: String,
    pub nasa_jpl_url: String,
    #[serde(default)]
    pub estimated_diameter_km_min: Option<f64>,
    #[serde(default)]
    pub estimated_diameter_km_max: Option<f64>,
    pub is_potentially_hazardous: bool,
    pub close_approach_data: Vec<CloseApproach>,
}

/// Close approach data for an asteroid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseApproach {
    pub date: String,
    #[serde(default)]
    pub relative_velocity_kph: Option<f64>,
    #[serde(default)]
    pub miss_distance_km: Option<f64>,
    pub orbiting_body: String,
}

/// Solar flare event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarFlare {
    pub flr_id: String,
    pub begin_time: String,
    #[serde(default)]
    pub peak_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    pub class_type: String,
    pub source_location: String,
}

/// Geomagnetic storm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeomagneticStorm {
    pub gst_id: String,
    pub start_time: String,
    pub kp_index: Vec<KpIndex>,
}

/// Kp index measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpIndex {
    pub observed_time: String,
    pub kp_index: f64,
    pub source: String,
}

/// Coronal Mass Ejection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoronalMassEjection {
    pub activity_id: String,
    pub start_time: String,
    pub source_location: String,
    pub note: String,
    #[serde(default)]
    pub speed: Option<f64>,
}

/// Solar Energetic Particle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarEnergeticParticle {
    pub sep_id: String,
    pub event_time: String,
    pub instruments: Vec<String>,
}

/// Interplanetary Shock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterplanetaryShock {
    pub activity_id: String,
    pub event_time: String,
    pub location: String,
}

/// Astronomy Picture of the Day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apod {
    pub title: String,
    pub date: String,
    pub explanation: String,
    pub url: String,
    #[serde(default)]
    pub hdurl: Option<String>,
    pub media_type: String,
    #[serde(default)]
    pub copyright: Option<String>,
}

/// Earth imagery from EPIC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarthImagery {
    pub identifier: String,
    pub caption: String,
    pub image: String,
    pub version: String,
    pub date: String,
}
