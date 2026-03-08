//! SpaceX connector implementation

use reqwest::Client;
use std::collections::HashMap;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{SpaceXParser, SpaceXLaunch, SpaceXRocket, SpaceXCrew, SpaceXStarlink};

/// SpaceX Data connector
///
/// Provides access to SpaceX launch data, rockets, crew, and Starlink satellites.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::spacex::SpaceXConnector;
///
/// // No authentication required
/// let connector = SpaceXConnector::new();
///
/// // Get latest launch
/// let latest = connector.get_latest_launch().await?;
///
/// // Get next upcoming launch
/// let next = connector.get_next_launch().await?;
///
/// // Get all rockets
/// let rockets = connector.get_rockets().await?;
/// ```
pub struct SpaceXConnector {
    client: Client,
    auth: SpaceXAuth,
    endpoints: SpaceXEndpoints,
}

impl SpaceXConnector {
    /// Create new SpaceX connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: SpaceXAuth::new(),
            endpoints: SpaceXEndpoints::default(),
        }
    }

    /// Create connector from environment variables (no-op for SpaceX)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to SpaceX API
    async fn get(
        &self,
        endpoint: SpaceXEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = reqwest::header::HeaderMap::new();
        self.auth.sign_headers(&mut headers);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, body),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCH ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get latest launch
    ///
    /// Returns the most recent SpaceX launch.
    pub async fn get_latest_launch(&self) -> ExchangeResult<SpaceXLaunch> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::LaunchesLatest, params).await?;
        SpaceXParser::parse_launch(&response)
    }

    /// Get next upcoming launch
    ///
    /// Returns the next scheduled SpaceX launch.
    pub async fn get_next_launch(&self) -> ExchangeResult<SpaceXLaunch> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::LaunchesNext, params).await?;
        SpaceXParser::parse_launch(&response)
    }

    /// Get all upcoming launches
    ///
    /// Returns all future scheduled SpaceX launches.
    pub async fn get_upcoming_launches(&self) -> ExchangeResult<Vec<SpaceXLaunch>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::LaunchesUpcoming, params).await?;
        SpaceXParser::parse_launches(&response)
    }

    /// Get all past launches
    ///
    /// Returns all historical SpaceX launches.
    pub async fn get_past_launches(&self) -> ExchangeResult<Vec<SpaceXLaunch>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::LaunchesPast, params).await?;
        SpaceXParser::parse_launches(&response)
    }

    /// Get all launches (past and upcoming)
    ///
    /// Returns complete SpaceX launch history and future schedule.
    pub async fn get_all_launches(&self) -> ExchangeResult<Vec<SpaceXLaunch>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::LaunchesAll, params).await?;
        SpaceXParser::parse_launches(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ROCKET ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all rockets
    ///
    /// Returns information about all SpaceX rockets.
    pub async fn get_rockets(&self) -> ExchangeResult<Vec<SpaceXRocket>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::Rockets, params).await?;
        SpaceXParser::parse_rockets(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CREW ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all crew members
    ///
    /// Returns information about all SpaceX crew members.
    pub async fn get_crew(&self) -> ExchangeResult<Vec<SpaceXCrew>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::Crew, params).await?;
        SpaceXParser::parse_crew(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STARLINK ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all Starlink satellites
    ///
    /// Returns information about all Starlink satellites.
    pub async fn get_starlink(&self) -> ExchangeResult<Vec<SpaceXStarlink>> {
        let params = HashMap::new();
        let response = self.get(SpaceXEndpoint::Starlink, params).await?;
        SpaceXParser::parse_starlink(&response)
    }
}

impl Default for SpaceXConnector {
    fn default() -> Self {
        Self::new()
    }
}
