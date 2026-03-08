//! UN OCHA HAPI response parsers
//!
//! Parse JSON responses to domain types.

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};

pub struct UnOchaParser;

impl UnOchaParser {
    /// Parse population data response
    pub fn parse_population(json: &str) -> ExchangeResult<Vec<PopulationData>> {
        let response: HapiResponse<PopulationData> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse population data: {}", e)))?;

        Ok(response.data)
    }

    /// Parse food security data response
    pub fn parse_food_security(json: &str) -> ExchangeResult<Vec<FoodSecurityData>> {
        let response: HapiResponse<FoodSecurityData> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse food security data: {}", e)))?;

        Ok(response.data)
    }

    /// Parse humanitarian needs data response
    pub fn parse_humanitarian_needs(json: &str) -> ExchangeResult<Vec<HumanitarianNeeds>> {
        let response: HapiResponse<HumanitarianNeeds> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse humanitarian needs: {}", e)))?;

        Ok(response.data)
    }

    /// Parse operational presence data response
    pub fn parse_operational_presence(json: &str) -> ExchangeResult<Vec<OperationalPresence>> {
        let response: HapiResponse<OperationalPresence> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse operational presence: {}", e)))?;

        Ok(response.data)
    }

    /// Parse funding data response
    pub fn parse_funding(json: &str) -> ExchangeResult<Vec<FundingData>> {
        let response: HapiResponse<FundingData> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse funding data: {}", e)))?;

        Ok(response.data)
    }

    /// Parse displacement data response (refugees, IDPs, returnees)
    pub fn parse_displacement(json: &str) -> ExchangeResult<Vec<DisplacementData>> {
        let response: HapiResponse<DisplacementData> = serde_json::from_str(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse displacement data: {}", e)))?;

        Ok(response.data)
    }
}

// =============================================================================
// UN OCHA HAPI-SPECIFIC TYPES
// =============================================================================

/// Generic HAPI response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
struct HapiResponse<T> {
    /// Response data array
    data: Vec<T>,
    /// Metadata about the response
    #[serde(default)]
    metadata: Option<HapiMetadata>,
}

/// Response metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct HapiMetadata {
    /// Total number of records
    #[serde(default)]
    pub total: Option<u64>,
    /// Current page
    #[serde(default)]
    pub page: Option<u32>,
    /// Records per page
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Population data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopulationData {
    /// Location code (ISO code or P-code)
    #[serde(rename = "location_code")]
    pub location_code: String,
    /// Location name
    #[serde(rename = "location_name")]
    pub location_name: String,
    /// Admin level (0 = country, 1 = province/state, etc.)
    #[serde(default)]
    pub admin_level: Option<u32>,
    /// Population count
    pub population: u64,
    /// Reference year
    pub year: u32,
    /// Data source
    #[serde(default)]
    pub source: Option<String>,
    /// Gender breakdown (if available)
    #[serde(default)]
    pub gender: Option<String>,
    /// Age range (if available)
    #[serde(default)]
    pub age_range: Option<String>,
}

/// Food security data (IPC - Integrated Food Security Phase Classification)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FoodSecurityData {
    /// Location code
    #[serde(rename = "location_code")]
    pub location_code: String,
    /// Location name
    #[serde(rename = "location_name")]
    pub location_name: String,
    /// IPC phase (1 = Minimal, 2 = Stressed, 3 = Crisis, 4 = Emergency, 5 = Catastrophe/Famine)
    pub ipc_phase: u32,
    /// IPC type (current or projected)
    #[serde(rename = "ipc_type")]
    pub ipc_type: String,
    /// Population in this phase
    #[serde(rename = "population_in_phase")]
    pub population_in_phase: u64,
    /// Analysis period
    #[serde(rename = "analysis_period")]
    pub analysis_period: String,
    /// Reference date
    #[serde(default)]
    pub reference_date: Option<String>,
    /// Data source
    #[serde(default)]
    pub source: Option<String>,
}

/// Humanitarian needs assessment data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HumanitarianNeeds {
    /// Location code
    #[serde(rename = "location_code")]
    pub location_code: String,
    /// Location name
    #[serde(rename = "location_name")]
    pub location_name: String,
    /// Sector (e.g., "Health", "Shelter", "Food Security", "WASH")
    pub sector: String,
    /// Number of people in need
    #[serde(rename = "people_in_need")]
    pub people_in_need: u64,
    /// Number of people targeted for assistance
    #[serde(default)]
    #[serde(rename = "people_targeted")]
    pub people_targeted: Option<u64>,
    /// Number of people reached with assistance
    #[serde(default)]
    #[serde(rename = "people_reached")]
    pub people_reached: Option<u64>,
    /// Reference period/year
    #[serde(default)]
    pub reference_period: Option<String>,
    /// Data source
    #[serde(default)]
    pub source: Option<String>,
}

/// Operational presence of humanitarian organizations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OperationalPresence {
    /// Organization name
    #[serde(rename = "org_name")]
    pub org_name: String,
    /// Organization type (UN, INGO, NNGO, etc.)
    #[serde(rename = "org_type")]
    pub org_type: String,
    /// Location code
    #[serde(rename = "location_code")]
    pub location_code: String,
    /// Location name
    #[serde(rename = "location_name")]
    pub location_name: String,
    /// Sector of operation
    pub sector: String,
    /// Reference period
    #[serde(default)]
    pub reference_period: Option<String>,
}

/// Humanitarian funding data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FundingData {
    /// Appeal name/plan
    pub appeal_name: String,
    /// Appeal code
    #[serde(default)]
    pub appeal_code: Option<String>,
    /// Location code
    #[serde(rename = "location_code")]
    pub location_code: String,
    /// Location name
    #[serde(rename = "location_name")]
    pub location_name: String,
    /// Requirements (USD)
    #[serde(default)]
    pub requirements_usd: Option<f64>,
    /// Funding received (USD)
    #[serde(default)]
    pub funding_usd: Option<f64>,
    /// Unmet requirements (USD)
    #[serde(default)]
    pub unmet_requirements_usd: Option<f64>,
    /// Percent funded
    #[serde(default)]
    pub percent_funded: Option<f64>,
    /// Reference year
    #[serde(default)]
    pub year: Option<u32>,
}

/// Displacement data (refugees, IDPs, returnees)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplacementData {
    /// Origin country/location code
    #[serde(rename = "origin_location_code")]
    pub origin_location_code: String,
    /// Origin country/location name
    #[serde(rename = "origin_location_name")]
    pub origin_location_name: String,
    /// Asylum country/location code (for refugees)
    #[serde(default)]
    #[serde(rename = "asylum_location_code")]
    pub asylum_location_code: Option<String>,
    /// Asylum country/location name (for refugees)
    #[serde(default)]
    #[serde(rename = "asylum_location_name")]
    pub asylum_location_name: Option<String>,
    /// Number of refugees
    #[serde(default)]
    pub refugees: Option<u64>,
    /// Number of asylum seekers
    #[serde(default)]
    pub asylum_seekers: Option<u64>,
    /// Number of internally displaced persons
    #[serde(default)]
    pub idps: Option<u64>,
    /// Number of returnees
    #[serde(default)]
    pub returnees: Option<u64>,
    /// Number of stateless persons
    #[serde(default)]
    pub stateless: Option<u64>,
    /// Reference year
    pub year: u32,
    /// Data source
    #[serde(default)]
    pub source: Option<String>,
}
