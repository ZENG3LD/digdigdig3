//! OpenSky Network response parsers
//!
//! Parse JSON responses to domain types based on OpenSky API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::*;

pub struct OpenskyParser;

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// State vector response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenskyStates {
    /// Time of query (UNIX timestamp in seconds)
    pub time: i64,
    /// Array of state vectors
    pub states: Vec<StateVector>,
}

/// Aircraft state vector (position, velocity, etc.)
///
/// Response format is array with fixed positions:
/// [icao24, callsign, origin_country, time_position, last_contact, longitude,
///  latitude, baro_altitude, on_ground, velocity, true_track, vertical_rate,
///  sensors, geo_altitude, squawk, spi, position_source]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVector {
    /// Unique ICAO 24-bit address of transponder (hex string)
    pub icao24: String,
    /// Callsign of the vehicle (8 chars, can be empty)
    pub callsign: Option<String>,
    /// Country name inferred from ICAO 24-bit address
    pub origin_country: String,
    /// Unix timestamp (seconds) of last position update (can be null)
    pub time_position: Option<i64>,
    /// Unix timestamp (seconds) of last update
    pub last_contact: i64,
    /// Longitude in decimal degrees (WGS-84, can be null)
    pub longitude: Option<f64>,
    /// Latitude in decimal degrees (WGS-84, can be null)
    pub latitude: Option<f64>,
    /// Barometric altitude in meters (can be null)
    pub baro_altitude: Option<f64>,
    /// Boolean indicating if aircraft is on ground
    pub on_ground: bool,
    /// Velocity over ground in m/s (can be null)
    pub velocity: Option<f64>,
    /// True track in decimal degrees clockwise from north (0-360, can be null)
    pub true_track: Option<f64>,
    /// Vertical rate in m/s (positive means climbing, can be null)
    pub vertical_rate: Option<f64>,
    /// IDs of sensors which contributed to this state vector (can be null)
    pub sensors: Option<Vec<i64>>,
    /// Geometric altitude in meters (can be null)
    pub geo_altitude: Option<f64>,
    /// Transponder code (squawk, can be null)
    pub squawk: Option<String>,
    /// Special purpose indicator (boolean)
    pub spi: bool,
    /// Origin of position (0=ADS-B, 1=ASTERIX, 2=MLAT, 3=FLARM)
    pub position_source: i32,
}

/// Flight data response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenskyFlights {
    /// Array of flight data
    pub flights: Vec<Flight>,
}

/// Flight information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flight {
    /// Unique ICAO 24-bit address of transponder (hex string)
    pub icao24: String,
    /// Estimated time of departure (UNIX timestamp in seconds)
    pub first_seen: i64,
    /// ICAO code of departure airport (can be null)
    pub estdeparture_airport: Option<String>,
    /// Estimated time of arrival (UNIX timestamp in seconds)
    pub last_seen: i64,
    /// ICAO code of arrival airport (can be null)
    pub estarrival_airport: Option<String>,
    /// Callsign of the vehicle (can be null)
    pub callsign: Option<String>,
    /// Horizontal distance of the last received airborne position (meters)
    pub estdeparture_airport_horiz_distance: Option<f64>,
    /// Vertical distance of the last received airborne position (meters)
    pub estdeparture_airport_vert_distance: Option<f64>,
    /// Horizontal distance of the last received airborne position (meters)
    pub estarrival_airport_horiz_distance: Option<f64>,
    /// Vertical distance of the last received airborne position (meters)
    pub estarrival_airport_vert_distance: Option<f64>,
    /// Number of position updates received
    pub departure_airport_candidates_count: Option<i32>,
    /// Number of position updates received
    pub arrival_airport_candidates_count: Option<i32>,
}

/// Flight track response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenskyTrack {
    /// Unique ICAO 24-bit address
    pub icao24: String,
    /// Time of query (UNIX timestamp in seconds)
    pub start_time: i64,
    /// End time of track (UNIX timestamp in seconds)
    pub end_time: i64,
    /// Callsign (can be null)
    pub callsign: Option<String>,
    /// Array of waypoints
    pub path: Vec<TrackPoint>,
}

/// Track waypoint
///
/// Response format is array: [time, latitude, longitude, baro_altitude, true_track, on_ground]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPoint {
    /// UNIX timestamp in seconds
    pub time: i64,
    /// Latitude in decimal degrees (can be null)
    pub latitude: Option<f64>,
    /// Longitude in decimal degrees (can be null)
    pub longitude: Option<f64>,
    /// Barometric altitude in meters (can be null)
    pub baro_altitude: Option<f64>,
    /// True track in decimal degrees (can be null)
    pub true_track: Option<f64>,
    /// On ground flag
    pub on_ground: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════

impl OpenskyParser {
    /// Check for API error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // OpenSky returns HTTP error codes for errors
        // If we have JSON, check for error fields (though API doesn't use this pattern)
        if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: -1,
                message: error.to_string(),
            });
        }
        Ok(())
    }

    /// Parse state vectors response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "time": 1706000000,
    ///   "states": [
    ///     ["abc123", "CALLSIGN ", "United States", 1706000000, 1706000000,
    ///      -122.5, 37.8, 10000, false, 250, 45, 0.5, null, 10500, "1234", false, 0]
    ///   ]
    /// }
    /// ```
    pub fn parse_states(response: &Value) -> ExchangeResult<OpenskyStates> {
        let time = Self::require_i64(response, "time")?;

        let states_array = response
            .get("states")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'states' array".to_string()))?;

        let states = states_array
            .iter()
            .filter_map(|state| Self::parse_state_vector(state).ok())
            .collect();

        Ok(OpenskyStates { time, states })
    }

    /// Parse single state vector from array format
    fn parse_state_vector(state: &Value) -> ExchangeResult<StateVector> {
        let arr = state
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("State vector must be array".to_string()))?;

        if arr.len() < 17 {
            return Err(ExchangeError::Parse(format!(
                "State vector array too short: {} elements",
                arr.len()
            )));
        }

        Ok(StateVector {
            icao24: Self::parse_string_from_value(&arr[0])?,
            callsign: Self::parse_optional_string(&arr[1]),
            origin_country: Self::parse_string_from_value(&arr[2])?,
            time_position: Self::parse_optional_i64(&arr[3]),
            last_contact: Self::parse_i64_from_value(&arr[4])?,
            longitude: Self::parse_optional_f64(&arr[5]),
            latitude: Self::parse_optional_f64(&arr[6]),
            baro_altitude: Self::parse_optional_f64(&arr[7]),
            on_ground: arr[8].as_bool().unwrap_or(false),
            velocity: Self::parse_optional_f64(&arr[9]),
            true_track: Self::parse_optional_f64(&arr[10]),
            vertical_rate: Self::parse_optional_f64(&arr[11]),
            sensors: arr[12].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_i64())
                    .collect()
            }),
            geo_altitude: Self::parse_optional_f64(&arr[13]),
            squawk: Self::parse_optional_string(&arr[14]),
            spi: arr[15].as_bool().unwrap_or(false),
            position_source: arr[16].as_i64().unwrap_or(0) as i32,
        })
    }

    /// Parse flights response
    pub fn parse_flights(response: &Value) -> ExchangeResult<Vec<Flight>> {
        let flights_array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of flights".to_string()))?;

        flights_array
            .iter()
            .map(Self::parse_flight)
            .collect()
    }

    /// Parse single flight object
    fn parse_flight(flight: &Value) -> ExchangeResult<Flight> {
        Ok(Flight {
            icao24: Self::require_str(flight, "icao24")?.to_string(),
            first_seen: Self::require_i64(flight, "firstSeen")?,
            estdeparture_airport: Self::get_str(flight, "estDepartureAirport")
                .map(|s| s.to_string()),
            last_seen: Self::require_i64(flight, "lastSeen")?,
            estarrival_airport: Self::get_str(flight, "estArrivalAirport")
                .map(|s| s.to_string()),
            callsign: Self::get_str(flight, "callsign").map(|s| s.to_string()),
            estdeparture_airport_horiz_distance: Self::get_f64(flight, "estDepartureAirportHorizDistance"),
            estdeparture_airport_vert_distance: Self::get_f64(flight, "estDepartureAirportVertDistance"),
            estarrival_airport_horiz_distance: Self::get_f64(flight, "estArrivalAirportHorizDistance"),
            estarrival_airport_vert_distance: Self::get_f64(flight, "estArrivalAirportVertDistance"),
            departure_airport_candidates_count: Self::get_i64(flight, "departureAirportCandidatesCount")
                .map(|i| i as i32),
            arrival_airport_candidates_count: Self::get_i64(flight, "arrivalAirportCandidatesCount")
                .map(|i| i as i32),
        })
    }

    /// Parse track response
    pub fn parse_track(response: &Value) -> ExchangeResult<OpenskyTrack> {
        let icao24 = Self::require_str(response, "icao24")?.to_string();
        let start_time = Self::require_i64(response, "startTime")?;
        let end_time = Self::require_i64(response, "endTime")?;
        let callsign = Self::get_str(response, "callsign").map(|s| s.to_string());

        let path_array = response
            .get("path")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'path' array".to_string()))?;

        let path = path_array
            .iter()
            .filter_map(|point| Self::parse_track_point(point).ok())
            .collect();

        Ok(OpenskyTrack {
            icao24,
            start_time,
            end_time,
            callsign,
            path,
        })
    }

    /// Parse single track point from array format
    fn parse_track_point(point: &Value) -> ExchangeResult<TrackPoint> {
        let arr = point
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Track point must be array".to_string()))?;

        if arr.len() < 6 {
            return Err(ExchangeError::Parse(format!(
                "Track point array too short: {} elements",
                arr.len()
            )));
        }

        Ok(TrackPoint {
            time: Self::parse_i64_from_value(&arr[0])?,
            latitude: Self::parse_optional_f64(&arr[1]),
            longitude: Self::parse_optional_f64(&arr[2]),
            baro_altitude: Self::parse_optional_f64(&arr[3]),
            true_track: Self::parse_optional_f64(&arr[4]),
            on_ground: arr[5].as_bool().unwrap_or(false),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field '{}'", key)))
    }

    fn get_str<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
        obj.get(key).and_then(|v| v.as_str())
    }

    fn require_i64(obj: &Value, key: &str) -> ExchangeResult<i64> {
        obj.get(key)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required i64 field '{}'", key)))
    }

    fn get_i64(obj: &Value, key: &str) -> Option<i64> {
        obj.get(key).and_then(|v| v.as_i64())
    }

    fn get_f64(obj: &Value, key: &str) -> Option<f64> {
        obj.get(key).and_then(|v| v.as_f64())
    }

    fn parse_string_from_value(val: &Value) -> ExchangeResult<String> {
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Expected string value".to_string()))
    }

    fn parse_optional_string(val: &Value) -> Option<String> {
        val.as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    fn parse_i64_from_value(val: &Value) -> ExchangeResult<i64> {
        val.as_i64()
            .or_else(|| val.as_f64().map(|f| f as i64))
            .ok_or_else(|| ExchangeError::Parse("Expected i64 value".to_string()))
    }

    fn parse_optional_i64(val: &Value) -> Option<i64> {
        val.as_i64().or_else(|| val.as_f64().map(|f| f as i64))
    }

    fn parse_optional_f64(val: &Value) -> Option<f64> {
        val.as_f64()
    }
}
