//! Space-Track connector implementation

use reqwest::Client;
use std::sync::Mutex;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{SpaceTrackParser, Satellite, DecayPrediction, TleData};

/// Space-Track.org connector
///
/// Provides access to satellite tracking data, orbital elements, and space debris information.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::space_track::SpaceTrackConnector;
///
/// let connector = SpaceTrackConnector::from_env();
///
/// // Login first (session cookie is stored)
/// connector.login().await?;
///
/// // Get TLE data for a specific satellite (e.g., ISS)
/// let tle = connector.get_satellite(25544).await?;
///
/// // Get recent satellite launches
/// let launches = connector.get_recent_launches(None).await?;
///
/// // Get decay predictions
/// let decays = connector.get_decay_predictions(None).await?;
/// ```
pub struct SpaceTrackConnector {
    client: Client,
    auth: SpaceTrackAuth,
    endpoints: SpaceTrackEndpoints,
    session_cookie: Mutex<Option<String>>,
}

impl SpaceTrackConnector {
    /// Create new Space-Track connector with authentication
    pub fn new(auth: SpaceTrackAuth) -> Self {
        let client = Client::new();

        Self {
            client,
            auth,
            endpoints: SpaceTrackEndpoints::default(),
            session_cookie: Mutex::new(None),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `SPACE_TRACK_USERNAME` and `SPACE_TRACK_PASSWORD` environment variables
    pub fn from_env() -> Self {
        Self::new(SpaceTrackAuth::from_env())
    }

    /// Login to Space-Track
    ///
    /// Must be called before making any data requests.
    /// The session cookie is extracted and stored for subsequent requests.
    pub async fn login(&self) -> ExchangeResult<()> {
        let login_body = self
            .auth
            .login_body()
            .ok_or_else(|| ExchangeError::Auth("Missing credentials".to_string()))?;

        let url = format!("{}{}", self.endpoints.rest_base, SpaceTrackEndpoint::Login.path());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(login_body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Login request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("Login failed with HTTP {}", response.status()),
            });
        }

        // Extract and store session cookie
        if let Some(cookie_header) = response.headers().get("set-cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                // Store the full cookie string
                let mut session_cookie = self.session_cookie.lock()
                    .map_err(|e| ExchangeError::Auth(format!("Failed to lock cookie mutex: {}", e)))?;
                *session_cookie = Some(cookie_str.to_string());
            }
        }

        Ok(())
    }

    /// Internal: Make GET request to Space-Track API
    ///
    /// Note: Must call login() first to establish session
    async fn get(&self, endpoint: SpaceTrackEndpoint) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Get session cookie - clone value so MutexGuard is dropped before await
        let cookie = {
            let session_cookie = self.session_cookie.lock()
                .map_err(|e| ExchangeError::Auth(format!("Failed to lock cookie mutex: {}", e)))?;
            session_cookie.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Not logged in - call login() first".to_string()))?
                .clone()
        };

        let mut request = self.client.get(&url);

        // Add session cookie header
        request = request.header("Cookie", cookie.as_str());

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
    // SPACE-TRACK SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get TLE (Two-Line Element) data for a specific satellite by NORAD ID
    ///
    /// # Arguments
    /// - `norad_id` - NORAD Catalog ID (e.g., 25544 for ISS)
    ///
    /// # Returns
    /// TLE data containing orbital elements
    pub async fn get_satellite(&self, norad_id: u32) -> ExchangeResult<TleData> {
        let response = self
            .get(SpaceTrackEndpoint::GeneralPerturbations { norad_id })
            .await?;

        let tle_list = SpaceTrackParser::parse_tle_data(&response)?;

        tle_list
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse(format!("No TLE data found for NORAD ID {}", norad_id)))
    }

    /// Get recent satellite launches
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default: 25)
    ///
    /// # Returns
    /// List of recently launched satellites
    pub async fn get_recent_launches(&self, limit: Option<u32>) -> ExchangeResult<Vec<Satellite>> {
        // Note: The endpoint has a built-in limit of 25
        // To support custom limits, we would need to modify the endpoint path
        let _ = limit; // Suppress unused warning
        let response = self.get(SpaceTrackEndpoint::SatelliteCatalog).await?;
        SpaceTrackParser::parse_satellites(&response)
    }

    /// Get decay predictions for deorbiting objects
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default: 25)
    ///
    /// # Returns
    /// List of decay predictions
    pub async fn get_decay_predictions(&self, limit: Option<u32>) -> ExchangeResult<Vec<DecayPrediction>> {
        let _ = limit; // Suppress unused warning
        let response = self.get(SpaceTrackEndpoint::Decay).await?;
        SpaceTrackParser::parse_decay_predictions(&response)
    }

    /// Get space debris tracking data
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default: 50)
    ///
    /// # Returns
    /// List of tracked debris objects
    pub async fn get_debris(&self, limit: Option<u32>) -> ExchangeResult<Vec<Satellite>> {
        let _ = limit; // Suppress unused warning
        let response = self.get(SpaceTrackEndpoint::Debris).await?;
        SpaceTrackParser::parse_satellites(&response)
    }

    /// Get launch sites information
    ///
    /// # Returns
    /// List of launch sites (raw JSON values)
    pub async fn get_launch_sites(&self) -> ExchangeResult<Vec<serde_json::Value>> {
        let response = self.get(SpaceTrackEndpoint::LaunchSites).await?;

        response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))
            .cloned()
    }

    /// Get Tracking & Impact Predictions (TIP)
    ///
    /// # Returns
    /// List of TIP entries (raw JSON values)
    pub async fn get_tip(&self) -> ExchangeResult<Vec<serde_json::Value>> {
        let response = self.get(SpaceTrackEndpoint::Tip).await?;

        response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array response".to_string()))
            .cloned()
    }
}
