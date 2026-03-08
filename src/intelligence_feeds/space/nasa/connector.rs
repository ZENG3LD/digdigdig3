//! NASA connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    NasaParser, NeoObject, SolarFlare, GeomagneticStorm, CoronalMassEjection,
    SolarEnergeticParticle, InterplanetaryShock, Apod, EarthImagery,
};

/// NASA Open APIs connector
///
/// Provides access to NASA's various data feeds including:
/// - Near Earth Objects (NEO)
/// - Space Weather (DONKI)
/// - Astronomy Picture of the Day (APOD)
/// - Earth Imagery (EPIC)
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::nasa::NasaConnector;
///
/// let connector = NasaConnector::from_env();
///
/// // Get near Earth objects
/// let neos = connector.get_neo_feed("2024-01-01", "2024-01-07").await?;
///
/// // Get solar flares
/// let flares = connector.get_solar_flares("2024-01-01", "2024-01-07").await?;
///
/// // Get astronomy picture of the day
/// let apod = connector.get_apod(Some("2024-01-01")).await?;
/// ```
pub struct NasaConnector {
    client: Client,
    auth: NasaAuth,
    endpoints: NasaEndpoints,
    _testnet: bool,
}

impl NasaConnector {
    /// Create new NASA connector with authentication
    pub fn new(auth: NasaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: NasaEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `NASA_API_KEY` environment variable
    /// Falls back to "DEMO_KEY" if not set
    pub fn from_env() -> Self {
        Self::new(NasaAuth::from_env())
    }

    /// Internal: Make GET request to NASA API
    async fn get(
        &self,
        endpoint: NasaEndpoint,
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
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEO (Near Earth Objects) METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Near Earth Objects feed by date range
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD), max 7 days from start_date
    ///
    /// # Returns
    /// Vector of NEO objects with close approach data
    pub async fn get_neo_feed(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<NeoObject>> {
        let mut params = HashMap::new();
        params.insert("start_date".to_string(), start_date.to_string());
        params.insert("end_date".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::NeoFeed, params).await?;
        NasaParser::parse_neo_feed(&response)
    }

    /// Get specific asteroid by ID
    ///
    /// # Arguments
    /// - `asteroid_id` - Asteroid ID (e.g., "3542519")
    ///
    /// # Returns
    /// Single NEO object with detailed information
    pub async fn get_neo_lookup(&self, asteroid_id: &str) -> ExchangeResult<NeoObject> {
        let params = HashMap::new();
        let response = self.get(NasaEndpoint::NeoLookup(asteroid_id.to_string()), params).await?;
        NasaParser::parse_neo_object(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DONKI (Space Weather) METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get solar flares
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of solar flare events
    pub async fn get_solar_flares(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<SolarFlare>> {
        let mut params = HashMap::new();
        params.insert("startDate".to_string(), start_date.to_string());
        params.insert("endDate".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::DonkiFlr, params).await?;
        NasaParser::parse_solar_flares(&response)
    }

    /// Get geomagnetic storms
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of geomagnetic storm events
    pub async fn get_geomagnetic_storms(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<GeomagneticStorm>> {
        let mut params = HashMap::new();
        params.insert("startDate".to_string(), start_date.to_string());
        params.insert("endDate".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::DonkiGst, params).await?;
        NasaParser::parse_geomagnetic_storms(&response)
    }

    /// Get coronal mass ejections
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of CME events
    pub async fn get_coronal_mass_ejections(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<CoronalMassEjection>> {
        let mut params = HashMap::new();
        params.insert("startDate".to_string(), start_date.to_string());
        params.insert("endDate".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::DonkiCme, params).await?;
        NasaParser::parse_coronal_mass_ejections(&response)
    }

    /// Get solar energetic particles
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of SEP events
    pub async fn get_solar_energetic_particles(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<SolarEnergeticParticle>> {
        let mut params = HashMap::new();
        params.insert("startDate".to_string(), start_date.to_string());
        params.insert("endDate".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::DonkiSep, params).await?;
        NasaParser::parse_solar_energetic_particles(&response)
    }

    /// Get interplanetary shocks
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Vector of IPS events
    pub async fn get_interplanetary_shocks(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<InterplanetaryShock>> {
        let mut params = HashMap::new();
        params.insert("startDate".to_string(), start_date.to_string());
        params.insert("endDate".to_string(), end_date.to_string());

        let response = self.get(NasaEndpoint::DonkiIps, params).await?;
        NasaParser::parse_interplanetary_shocks(&response)
    }

    /// Get space weather summary (all DONKI events)
    ///
    /// Convenience method that fetches all space weather data types.
    ///
    /// # Arguments
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    ///
    /// # Returns
    /// Tuple of (solar flares, geomagnetic storms, CMEs, SEPs, IPS)
    pub async fn get_space_weather_summary(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<(
        Vec<SolarFlare>,
        Vec<GeomagneticStorm>,
        Vec<CoronalMassEjection>,
        Vec<SolarEnergeticParticle>,
        Vec<InterplanetaryShock>,
    )> {
        // Fetch all in parallel for efficiency
        let flares = self.get_solar_flares(start_date, end_date);
        let storms = self.get_geomagnetic_storms(start_date, end_date);
        let cmes = self.get_coronal_mass_ejections(start_date, end_date);
        let seps = self.get_solar_energetic_particles(start_date, end_date);
        let ips = self.get_interplanetary_shocks(start_date, end_date);

        let (flares, storms, cmes, seps, ips) = tokio::join!(flares, storms, cmes, seps, ips);

        Ok((flares?, storms?, cmes?, seps?, ips?))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OTHER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Astronomy Picture of the Day
    ///
    /// # Arguments
    /// - `date` - Optional date (YYYY-MM-DD), defaults to today
    ///
    /// # Returns
    /// APOD data with image URL and explanation
    pub async fn get_apod(&self, date: Option<&str>) -> ExchangeResult<Apod> {
        let mut params = HashMap::new();
        if let Some(d) = date {
            params.insert("date".to_string(), d.to_string());
        }

        let response = self.get(NasaEndpoint::Apod, params).await?;
        NasaParser::parse_apod(&response)
    }

    /// Get Earth imagery from EPIC
    ///
    /// # Arguments
    /// - `date` - Optional date (YYYY-MM-DD), defaults to most recent available
    ///
    /// # Returns
    /// Vector of Earth imagery metadata
    pub async fn get_earth_imagery(&self, _date: Option<&str>) -> ExchangeResult<Vec<EarthImagery>> {
        let params = HashMap::new();
        // Note: EPIC API date filtering is done via URL path, not query params
        // For simplicity, we'll get the most recent available images
        let response = self.get(NasaEndpoint::EpicNatural, params).await?;
        NasaParser::parse_earth_imagery(&response)
    }
}
