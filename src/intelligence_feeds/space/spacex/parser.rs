//! SpaceX API response parsers
//!
//! Parse JSON responses to domain types based on SpaceX API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct SpaceXParser;

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// SpaceX launch data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceXLaunch {
    /// Launch ID
    pub id: String,
    /// Launch name
    pub name: String,
    /// Launch date in UTC
    pub date_utc: String,
    /// Launch date in local time
    pub date_local: Option<String>,
    /// Launch success (null if not determined yet)
    pub success: Option<bool>,
    /// Launch details/description
    pub details: Option<String>,
    /// Rocket ID
    pub rocket: Option<String>,
    /// Crew IDs
    pub crew: Vec<String>,
    /// Payload IDs
    pub payloads: Vec<String>,
    /// Launchpad ID
    pub launchpad: Option<String>,
    /// Flight number
    pub flight_number: Option<i32>,
}

/// SpaceX rocket data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceXRocket {
    /// Rocket ID
    pub id: String,
    /// Rocket name
    pub name: String,
    /// Rocket type
    pub type_name: Option<String>,
    /// Active status
    pub active: bool,
    /// Number of stages
    pub stages: Option<i32>,
    /// Number of boosters
    pub boosters: Option<i32>,
    /// Cost per launch in USD
    pub cost_per_launch: Option<i64>,
    /// First flight date
    pub first_flight: Option<String>,
    /// Country of origin
    pub country: Option<String>,
    /// Company
    pub company: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// SpaceX crew member data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceXCrew {
    /// Crew member ID
    pub id: String,
    /// Name
    pub name: String,
    /// Agency
    pub agency: Option<String>,
    /// Status
    pub status: Option<String>,
    /// Launch IDs
    pub launches: Vec<String>,
}

/// SpaceX Starlink satellite data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceXStarlink {
    /// Starlink ID
    pub id: String,
    /// Version
    pub version: Option<String>,
    /// Launch ID
    pub launch: Option<String>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Height in kilometers
    pub height_km: Option<f64>,
    /// Velocity in km/s
    pub velocity_kms: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════

impl SpaceXParser {
    /// Check for API error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // SpaceX API returns HTTP error codes for errors
        // If we have JSON with an error field, handle it
        if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: -1,
                message: error.to_string(),
            });
        }
        Ok(())
    }

    /// Parse single launch
    pub fn parse_launch(response: &Value) -> ExchangeResult<SpaceXLaunch> {
        Self::check_error(response)?;

        Ok(SpaceXLaunch {
            id: Self::require_str(response, "id")?.to_string(),
            name: Self::require_str(response, "name")?.to_string(),
            date_utc: Self::require_str(response, "date_utc")?.to_string(),
            date_local: Self::get_str(response, "date_local").map(|s| s.to_string()),
            success: response.get("success").and_then(|v| v.as_bool()),
            details: Self::get_str(response, "details").map(|s| s.to_string()),
            rocket: Self::get_str(response, "rocket").map(|s| s.to_string()),
            crew: Self::parse_string_array(response.get("crew")),
            payloads: Self::parse_string_array(response.get("payloads")),
            launchpad: Self::get_str(response, "launchpad").map(|s| s.to_string()),
            flight_number: response.get("flight_number").and_then(|v| v.as_i64()).map(|i| i as i32),
        })
    }

    /// Parse array of launches
    pub fn parse_launches(response: &Value) -> ExchangeResult<Vec<SpaceXLaunch>> {
        let launches_array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of launches".to_string()))?;

        launches_array
            .iter()
            .map(Self::parse_launch)
            .collect()
    }

    /// Parse single rocket
    pub fn parse_rocket(response: &Value) -> ExchangeResult<SpaceXRocket> {
        Self::check_error(response)?;

        Ok(SpaceXRocket {
            id: Self::require_str(response, "id")?.to_string(),
            name: Self::require_str(response, "name")?.to_string(),
            type_name: Self::get_str(response, "type").map(|s| s.to_string()),
            active: response.get("active").and_then(|v| v.as_bool()).unwrap_or(false),
            stages: response.get("stages").and_then(|v| v.as_i64()).map(|i| i as i32),
            boosters: response.get("boosters").and_then(|v| v.as_i64()).map(|i| i as i32),
            cost_per_launch: response.get("cost_per_launch").and_then(|v| v.as_i64()),
            first_flight: Self::get_str(response, "first_flight").map(|s| s.to_string()),
            country: Self::get_str(response, "country").map(|s| s.to_string()),
            company: Self::get_str(response, "company").map(|s| s.to_string()),
            description: Self::get_str(response, "description").map(|s| s.to_string()),
        })
    }

    /// Parse array of rockets
    pub fn parse_rockets(response: &Value) -> ExchangeResult<Vec<SpaceXRocket>> {
        let rockets_array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of rockets".to_string()))?;

        rockets_array
            .iter()
            .map(Self::parse_rocket)
            .collect()
    }

    /// Parse single crew member
    pub fn parse_crew_member(response: &Value) -> ExchangeResult<SpaceXCrew> {
        Self::check_error(response)?;

        Ok(SpaceXCrew {
            id: Self::require_str(response, "id")?.to_string(),
            name: Self::require_str(response, "name")?.to_string(),
            agency: Self::get_str(response, "agency").map(|s| s.to_string()),
            status: Self::get_str(response, "status").map(|s| s.to_string()),
            launches: Self::parse_string_array(response.get("launches")),
        })
    }

    /// Parse array of crew members
    pub fn parse_crew(response: &Value) -> ExchangeResult<Vec<SpaceXCrew>> {
        let crew_array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of crew members".to_string()))?;

        crew_array
            .iter()
            .map(Self::parse_crew_member)
            .collect()
    }

    /// Parse single Starlink satellite
    pub fn parse_starlink_satellite(response: &Value) -> ExchangeResult<SpaceXStarlink> {
        Self::check_error(response)?;

        Ok(SpaceXStarlink {
            id: Self::require_str(response, "id")?.to_string(),
            version: Self::get_str(response, "version").map(|s| s.to_string()),
            launch: Self::get_str(response, "launch").map(|s| s.to_string()),
            longitude: response.get("longitude").and_then(|v| v.as_f64()),
            latitude: response.get("latitude").and_then(|v| v.as_f64()),
            height_km: response.get("height_km").and_then(|v| v.as_f64()),
            velocity_kms: response.get("velocity_kms").and_then(|v| v.as_f64()),
        })
    }

    /// Parse array of Starlink satellites
    pub fn parse_starlink(response: &Value) -> ExchangeResult<Vec<SpaceXStarlink>> {
        let starlink_array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of Starlink satellites".to_string()))?;

        starlink_array
            .iter()
            .map(Self::parse_starlink_satellite)
            .collect()
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

    fn parse_string_array(value: Option<&Value>) -> Vec<String> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
