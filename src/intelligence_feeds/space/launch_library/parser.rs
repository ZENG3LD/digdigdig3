//! Launch Library 2 response parsers
//!
//! Parse JSON responses to domain types based on Launch Library 2 API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct LaunchLibraryParser;

impl LaunchLibraryParser {
    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCH LIBRARY 2 SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse paginated launch response
    pub fn parse_launches(response: &Value) -> ExchangeResult<PaginatedResponse<SpaceLaunch>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse launches: {}", e)))
    }

    /// Parse single launch
    pub fn parse_launch(response: &Value) -> ExchangeResult<SpaceLaunch> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse launch: {}", e)))
    }

    /// Parse paginated events response
    pub fn parse_events(response: &Value) -> ExchangeResult<PaginatedResponse<SpaceEvent>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse events: {}", e)))
    }

    /// Parse paginated agencies response
    pub fn parse_agencies(response: &Value) -> ExchangeResult<PaginatedResponse<SpaceAgency>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse agencies: {}", e)))
    }

    /// Parse paginated astronauts response
    pub fn parse_astronauts(response: &Value) -> ExchangeResult<PaginatedResponse<SpaceAstronaut>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse astronauts: {}", e)))
    }

    /// Parse paginated space stations response
    pub fn parse_space_stations(response: &Value) -> ExchangeResult<PaginatedResponse<SpaceStation>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse space stations: {}", e)))
    }

    /// Parse paginated rockets response
    pub fn parse_rockets(response: &Value) -> ExchangeResult<PaginatedResponse<RocketConfig>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse rockets: {}", e)))
    }

    /// Parse paginated spacecraft response
    pub fn parse_spacecraft(response: &Value) -> ExchangeResult<PaginatedResponse<SpacecraftConfig>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse spacecraft: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(detail) = response.get("detail").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 0,
                message: detail.to_string(),
            });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LAUNCH LIBRARY 2 SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Paginated API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub count: u64,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub previous: Option<String>,
    pub results: Vec<T>,
}

/// Space launch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceLaunch {
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    pub name: String,
    pub status: LaunchStatus,
    pub net: String, // datetime string
    #[serde(default)]
    pub window_start: Option<String>,
    #[serde(default)]
    pub window_end: Option<String>,
    #[serde(default)]
    pub mission: Option<SpaceMission>,
    pub pad: SpacePad,
    pub rocket: SpaceRocket,
    pub launch_service_provider: SpaceAgency,
    #[serde(default)]
    pub image: Option<String>,
}

/// Launch status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchStatus {
    pub id: u32,
    pub name: String,
    pub abbrev: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Space mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMission {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub orbit: Option<SpaceOrbit>,
    #[serde(default)]
    pub mission_type: Option<String>,
}

/// Space orbit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceOrbit {
    pub id: u32,
    pub name: String,
    pub abbrev: String,
}

/// Launch pad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacePad {
    pub id: u32,
    pub name: String,
    pub location: SpaceLocation,
}

/// Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceLocation {
    pub id: u32,
    pub name: String,
    pub country_code: String,
    #[serde(default)]
    pub total_launch_count: Option<u32>,
}

/// Space rocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceRocket {
    pub id: u32,
    pub configuration: RocketConfig,
}

/// Rocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocketConfig {
    pub id: u32,
    pub name: String,
    pub family: String,
    pub full_name: String,
    pub variant: String,
}

/// Space agency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceAgency {
    pub id: u32,
    pub name: String,
    pub country_code: String,
    #[serde(default)]
    pub abbrev: Option<String>,
    #[serde(default)]
    pub agency_type: Option<String>,
}

/// Space event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceEvent {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub date: String,
    #[serde(rename = "type")]
    pub type_name: SpaceEventType,
    #[serde(default)]
    pub location: Option<String>,
}

/// Space event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceEventType {
    pub id: u32,
    pub name: String,
}

/// Astronaut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceAstronaut {
    pub id: u32,
    pub name: String,
    pub nationality: String,
    #[serde(default)]
    pub agency: Option<SpaceAgency>,
    #[serde(default)]
    pub flights_count: Option<u32>,
    pub status: AstronautStatus,
}

/// Astronaut status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstronautStatus {
    pub id: u32,
    pub name: String,
}

/// Space station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceStation {
    pub id: u32,
    pub name: String,
    pub status: SpaceStationStatus,
    #[serde(default)]
    pub orbit: Option<String>,
    #[serde(default)]
    pub owners: Vec<SpaceAgency>,
}

/// Space station status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceStationStatus {
    pub id: u32,
    pub name: String,
}

/// Spacecraft configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacecraftConfig {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub agency: Option<SpaceAgency>,
    #[serde(default)]
    pub in_use: Option<bool>,
    #[serde(default)]
    pub capability: Option<String>,
}
