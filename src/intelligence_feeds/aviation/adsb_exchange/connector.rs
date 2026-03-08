//! ADS-B Exchange connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{AdsbExchangeParser, AdsbAircraft};

/// ADS-B Exchange (Unfiltered Flight Tracking) connector
///
/// Provides access to real-time aircraft position data, including military aircraft.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::adsb_exchange::AdsbExchangeConnector;
///
/// let connector = AdsbExchangeConnector::from_env();
///
/// // Get aircraft near San Francisco (37.7749, -122.4194, 50nm radius)
/// let aircraft = connector.get_aircraft_near(37.7749, -122.4194, 50).await?;
///
/// // Get all military aircraft currently airborne
/// let military = connector.get_military_aircraft().await?;
///
/// // Get emergency aircraft (squawk 7700)
/// let emergencies = connector.get_emergencies().await?;
/// ```
pub struct AdsbExchangeConnector {
    client: Client,
    auth: AdsbExchangeAuth,
    endpoints: AdsbExchangeEndpoints,
    _testnet: bool,
}

impl AdsbExchangeConnector {
    /// Create new ADS-B Exchange connector with authentication
    pub fn new(auth: AdsbExchangeAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AdsbExchangeEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `ADSBX_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(AdsbExchangeAuth::from_env())
    }

    /// Internal: Make GET request to ADS-B Exchange API
    async fn get(&self, endpoint: AdsbExchangeEndpoint) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication headers
        let mut headers = HashMap::new();
        self.auth.add_headers(&mut headers);

        let mut request = self.client.get(&url);

        // Add headers to request
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
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

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LOCATION-BASED METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get aircraft near a location
    ///
    /// # Arguments
    /// - `lat` - Latitude (decimal degrees)
    /// - `lon` - Longitude (decimal degrees)
    /// - `dist_nm` - Distance radius (nautical miles)
    ///
    /// # Returns
    /// Vector of aircraft within the specified radius
    pub async fn get_aircraft_near(
        &self,
        lat: f64,
        lon: f64,
        dist_nm: u32,
    ) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftNearLocation { lat, lon, dist_nm };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AIRCRAFT LOOKUP METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get aircraft by ICAO hex code
    ///
    /// # Arguments
    /// - `icao_hex` - ICAO hex code (e.g., "a1b2c3")
    ///
    /// # Returns
    /// Vector of matching aircraft (typically 0 or 1)
    pub async fn get_aircraft_by_hex(&self, icao_hex: &str) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftByHex {
            icao_hex: icao_hex.to_string(),
        };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    /// Get aircraft by callsign
    ///
    /// # Arguments
    /// - `callsign` - Callsign (e.g., "UAL123")
    ///
    /// # Returns
    /// Vector of matching aircraft
    pub async fn get_aircraft_by_callsign(
        &self,
        callsign: &str,
    ) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftByCallsign {
            callsign: callsign.to_string(),
        };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    /// Get aircraft by registration
    ///
    /// # Arguments
    /// - `registration` - Registration (e.g., "N12345")
    ///
    /// # Returns
    /// Vector of matching aircraft (typically 0 or 1)
    pub async fn get_aircraft_by_registration(
        &self,
        registration: &str,
    ) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftByRegistration {
            registration: registration.to_string(),
        };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    /// Get aircraft by type
    ///
    /// # Arguments
    /// - `aircraft_type` - Aircraft type (e.g., "B738", "F16")
    ///
    /// # Returns
    /// Vector of matching aircraft
    pub async fn get_aircraft_by_type(
        &self,
        aircraft_type: &str,
    ) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftByType {
            aircraft_type: aircraft_type.to_string(),
        };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SPECIAL CATEGORY METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get ALL military aircraft currently airborne
    ///
    /// This is a KEY FEATURE of ADS-B Exchange - UNFILTERED military aircraft data.
    ///
    /// # Returns
    /// Vector of all military aircraft
    pub async fn get_military_aircraft(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::MilitaryAircraft;
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    /// Get aircraft by squawk code
    ///
    /// # Arguments
    /// - `squawk` - Squawk code (e.g., "7700" for emergency)
    ///
    /// # Returns
    /// Vector of aircraft with the specified squawk code
    pub async fn get_by_squawk(&self, squawk: &str) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::AircraftBySquawk {
            squawk: squawk.to_string(),
        };
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    /// Get LADD (Limited Aircraft Data Display) aircraft
    ///
    /// LADD aircraft are military/sensitive aircraft with limited data display.
    ///
    /// # Returns
    /// Vector of LADD aircraft
    pub async fn get_ladd_aircraft(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        let endpoint = AdsbExchangeEndpoint::LaddAircraft;
        let response = self.get(endpoint).await?;
        let parsed = AdsbExchangeParser::parse_response(&response)?;
        Ok(parsed.ac)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get aircraft in emergency (squawk 7700)
    ///
    /// # Returns
    /// Vector of aircraft with emergency squawk code
    pub async fn get_emergencies(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        self.get_by_squawk("7700").await
    }

    /// Get hijack alerts (squawk 7500)
    ///
    /// # Returns
    /// Vector of aircraft with hijack squawk code
    pub async fn get_hijack_alerts(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        self.get_by_squawk("7500").await
    }

    /// Get aircraft with radio failure (squawk 7600)
    ///
    /// # Returns
    /// Vector of aircraft with radio failure squawk code
    pub async fn get_radio_failures(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        self.get_by_squawk("7600").await
    }

    /// Get military aircraft near a location
    ///
    /// # Arguments
    /// - `lat` - Latitude (decimal degrees)
    /// - `lon` - Longitude (decimal degrees)
    /// - `dist_nm` - Distance radius (nautical miles)
    ///
    /// # Returns
    /// Vector of military aircraft within the specified radius
    pub async fn get_military_near(
        &self,
        lat: f64,
        lon: f64,
        dist_nm: u32,
    ) -> ExchangeResult<Vec<AdsbAircraft>> {
        let aircraft = self.get_aircraft_near(lat, lon, dist_nm).await?;
        Ok(aircraft
            .into_iter()
            .filter(|ac| ac.is_military())
            .collect())
    }

    /// Get fighter aircraft currently airborne
    ///
    /// Searches for common fighter types: F-16, F-15, F-22, F-35, Su-27, Su-35, MiG-29
    ///
    /// # Returns
    /// Vector of fighter aircraft
    pub async fn get_fighters_airborne(&self) -> ExchangeResult<Vec<AdsbAircraft>> {
        let fighter_types = vec![
            "F16", "F15", "F22", "F35", // US fighters
            "SU27", "SU35", // Russian fighters
            "MIG29", // Russian fighter
        ];

        let mut fighters = Vec::new();

        for fighter_type in fighter_types {
            match self.get_aircraft_by_type(fighter_type).await {
                Ok(aircraft) => fighters.extend(aircraft),
                Err(_) => continue, // Ignore errors for individual types
            }
        }

        Ok(fighters)
    }

    /// Ping (check connection)
    ///
    /// # Returns
    /// Ok if connection is successful
    pub async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to get aircraft near 0,0 with 1nm radius (minimal data)
        let _ = self.get_aircraft_near(0.0, 0.0, 1).await?;
        Ok(())
    }
}
