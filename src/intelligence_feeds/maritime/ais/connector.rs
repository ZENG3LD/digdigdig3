//! AIS connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{AisParser, AisVessel, AisPort, AisPosition};

/// AIS (Automatic Identification System) connector
///
/// Provides access to vessel tracking data via Datalastic API.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ais::AisConnector;
///
/// let connector = AisConnector::from_env();
///
/// // Search for vessels
/// let vessels = connector.find_vessel("MAERSK").await?;
///
/// // Get vessel by MMSI
/// let vessel = connector.find_vessel_by_mmsi(123456789).await?;
///
/// // Get port information
/// let ports = connector.find_port("Rotterdam").await?;
/// ```
pub struct AisConnector {
    client: Client,
    auth: AisAuth,
    endpoints: AisEndpoints,
    _testnet: bool,
}

impl AisConnector {
    /// Create new AIS connector with authentication
    pub fn new(auth: AisAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AisEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `AIS_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(AisAuth::from_env())
    }

    /// Internal: Make GET request to AIS API
    async fn get(
        &self,
        endpoint: AisEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors
        AisParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // VESSEL SEARCH AND INFO
    // ═══════════════════════════════════════════════════════════════════════

    /// Search vessels by name
    ///
    /// # Arguments
    /// - `name` - Vessel name (partial match supported)
    ///
    /// # Returns
    /// Vector of matching vessels
    pub async fn find_vessel(&self, name: &str) -> ExchangeResult<Vec<AisVessel>> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), name.to_string());

        let response = self.get(AisEndpoint::VesselFind, params).await?;
        AisParser::parse_vessels(&response)
    }

    /// Search vessel by MMSI number
    ///
    /// # Arguments
    /// - `mmsi` - Maritime Mobile Service Identity number
    ///
    /// # Returns
    /// Vector of matching vessels (typically 0 or 1)
    pub async fn find_vessel_by_mmsi(&self, mmsi: u64) -> ExchangeResult<Vec<AisVessel>> {
        let mut params = HashMap::new();
        params.insert("mmsi".to_string(), mmsi.to_string());

        let response = self.get(AisEndpoint::VesselFind, params).await?;
        AisParser::parse_vessels(&response)
    }

    /// Search vessel by IMO number
    ///
    /// # Arguments
    /// - `imo` - International Maritime Organization number
    ///
    /// # Returns
    /// Vector of matching vessels (typically 0 or 1)
    pub async fn find_vessel_by_imo(&self, imo: u64) -> ExchangeResult<Vec<AisVessel>> {
        let mut params = HashMap::new();
        params.insert("imo".to_string(), imo.to_string());

        let response = self.get(AisEndpoint::VesselFind, params).await?;
        AisParser::parse_vessels(&response)
    }

    /// Get vessel details by UUID
    ///
    /// # Arguments
    /// - `uuid` - Vessel UUID from search results
    ///
    /// # Returns
    /// Detailed vessel information
    pub async fn get_vessel_info(&self, uuid: &str) -> ExchangeResult<AisVessel> {
        let mut params = HashMap::new();
        params.insert("uuid".to_string(), uuid.to_string());

        let response = self.get(AisEndpoint::VesselInfo, params).await?;
        AisParser::parse_vessel_info(&response)
    }

    /// Get vessel position history
    ///
    /// # Arguments
    /// - `uuid` - Vessel UUID
    /// - `days` - Number of days of history (default: 7)
    ///
    /// # Returns
    /// Vector of historical positions
    pub async fn get_vessel_history(
        &self,
        uuid: &str,
        days: Option<u32>,
    ) -> ExchangeResult<Vec<AisPosition>> {
        let mut params = HashMap::new();
        params.insert("uuid".to_string(), uuid.to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let response = self.get(AisEndpoint::VesselHistory, params).await?;
        AisParser::parse_vessel_history(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PORT SEARCH AND INFO
    // ═══════════════════════════════════════════════════════════════════════

    /// Search ports by name
    ///
    /// # Arguments
    /// - `name` - Port name (partial match supported)
    ///
    /// # Returns
    /// Vector of matching ports
    pub async fn find_port(&self, name: &str) -> ExchangeResult<Vec<AisPort>> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), name.to_string());

        let response = self.get(AisEndpoint::PortFind, params).await?;
        AisParser::parse_ports(&response)
    }

    /// Get port details by UUID
    ///
    /// # Arguments
    /// - `uuid` - Port UUID from search results
    ///
    /// # Returns
    /// Detailed port information
    pub async fn get_port_info(&self, uuid: &str) -> ExchangeResult<AisPort> {
        let mut params = HashMap::new();
        params.insert("uuid".to_string(), uuid.to_string());

        let response = self.get(AisEndpoint::PortInfo, params).await?;
        AisParser::parse_port_info(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FLEET AND AREA QUERIES
    // ═══════════════════════════════════════════════════════════════════════

    /// Get live fleet positions in a bounding box
    ///
    /// # Arguments
    /// - `lat_min` - Minimum latitude
    /// - `lon_min` - Minimum longitude
    /// - `lat_max` - Maximum latitude
    /// - `lon_max` - Maximum longitude
    ///
    /// # Returns
    /// Vector of vessels currently in the area
    pub async fn get_fleet_in_area(
        &self,
        lat_min: f64,
        lon_min: f64,
        lat_max: f64,
        lon_max: f64,
    ) -> ExchangeResult<Vec<AisVessel>> {
        let mut params = HashMap::new();
        params.insert("lat_min".to_string(), lat_min.to_string());
        params.insert("lon_min".to_string(), lon_min.to_string());
        params.insert("lat_max".to_string(), lat_max.to_string());
        params.insert("lon_max".to_string(), lon_max.to_string());

        let response = self.get(AisEndpoint::FleetLiveMap, params).await?;
        AisParser::parse_vessels(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS - SHIP TYPE FILTERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Find tanker vessels by name
    ///
    /// # Arguments
    /// - `name_contains` - Vessel name filter
    ///
    /// # Returns
    /// Vector of tanker vessels matching the name
    pub async fn get_tankers(&self, name_contains: &str) -> ExchangeResult<Vec<AisVessel>> {
        let vessels = self.find_vessel(name_contains).await?;

        // Filter by ship type containing "tanker"
        Ok(vessels
            .into_iter()
            .filter(|v| {
                v.ship_type
                    .as_ref()
                    .map(|st| st.to_lowercase().contains("tanker"))
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Find cargo ships by name
    ///
    /// # Arguments
    /// - `name_contains` - Vessel name filter
    ///
    /// # Returns
    /// Vector of cargo vessels matching the name
    pub async fn get_cargo_ships(&self, name_contains: &str) -> ExchangeResult<Vec<AisVessel>> {
        let vessels = self.find_vessel(name_contains).await?;

        // Filter by ship type containing "cargo"
        Ok(vessels
            .into_iter()
            .filter(|v| {
                v.ship_type
                    .as_ref()
                    .map(|st| st.to_lowercase().contains("cargo"))
                    .unwrap_or(false)
            })
            .collect())
    }
}
