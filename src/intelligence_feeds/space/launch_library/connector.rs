//! Launch Library 2 connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    LaunchLibraryParser, SpaceLaunch, SpaceEvent, SpaceAgency, SpaceAstronaut,
    SpaceStation, RocketConfig, SpacecraftConfig, PaginatedResponse,
};

/// Launch Library 2 connector
///
/// Provides access to space launch data, events, agencies, astronauts, and more.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::launch_library::LaunchLibraryConnector;
///
/// let connector = LaunchLibraryConnector::new();
///
/// // Get upcoming launches
/// let launches = connector.get_upcoming_launches(Some(10)).await?;
///
/// // Get next launch
/// let next = connector.get_next_launch().await?;
///
/// // Search launches
/// let results = connector.search_launches("SpaceX", Some(5)).await?;
/// ```
pub struct LaunchLibraryConnector {
    client: Client,
    auth: LaunchLibraryAuth,
    endpoints: LaunchLibraryEndpoints,
    _testnet: bool,
}

impl LaunchLibraryConnector {
    /// Create new Launch Library 2 connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: LaunchLibraryAuth::new(),
            endpoints: LaunchLibraryEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: LaunchLibraryAuth::from_env(),
            endpoints: LaunchLibraryEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to Launch Library 2 API
    async fn get(
        &self,
        endpoint: LaunchLibraryEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for public API)
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
        LaunchLibraryParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with ID parameter
    async fn get_with_id(
        &self,
        endpoint: LaunchLibraryEndpoint,
        id: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path_with_id(id));

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

        LaunchLibraryParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LAUNCH LIBRARY 2 SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get upcoming launches
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default: API default, max varies)
    ///
    /// # Returns
    /// Paginated response with upcoming space launches
    pub async fn get_upcoming_launches(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceLaunch>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::LaunchUpcoming, params).await?;
        LaunchLibraryParser::parse_launches(&response)
    }

    /// Get previous launches
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with previous space launches
    pub async fn get_previous_launches(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceLaunch>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::LaunchPrevious, params).await?;
        LaunchLibraryParser::parse_launches(&response)
    }

    /// Get launch details by ID
    ///
    /// # Arguments
    /// - `id` - Launch ID
    ///
    /// # Returns
    /// Single launch details
    pub async fn get_launch(&self, id: &str) -> ExchangeResult<SpaceLaunch> {
        let params = HashMap::new();
        let response = self.get_with_id(LaunchLibraryEndpoint::LaunchDetail, id, params).await?;
        LaunchLibraryParser::parse_launch(&response)
    }

    /// Get upcoming events (landings, dockings, etc.)
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with upcoming space events
    pub async fn get_upcoming_events(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceEvent>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::EventUpcoming, params).await?;
        LaunchLibraryParser::parse_events(&response)
    }

    /// Get space agencies
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with space agencies
    pub async fn get_agencies(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceAgency>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::Agency, params).await?;
        LaunchLibraryParser::parse_agencies(&response)
    }

    /// Get astronauts
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with astronaut data
    pub async fn get_astronauts(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceAstronaut>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::Astronaut, params).await?;
        LaunchLibraryParser::parse_astronauts(&response)
    }

    /// Get active space stations
    ///
    /// # Returns
    /// Paginated response with space station data
    pub async fn get_space_stations(&self) -> ExchangeResult<PaginatedResponse<SpaceStation>> {
        let params = HashMap::new();
        let response = self.get(LaunchLibraryEndpoint::SpaceStation, params).await?;
        LaunchLibraryParser::parse_space_stations(&response)
    }

    /// Get rocket configurations
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with rocket/launch vehicle configurations
    pub async fn get_rockets(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<RocketConfig>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::Rocket, params).await?;
        LaunchLibraryParser::parse_rockets(&response)
    }

    /// Get spacecraft configurations
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with spacecraft configurations
    pub async fn get_spacecraft(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpacecraftConfig>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::Spacecraft, params).await?;
        LaunchLibraryParser::parse_spacecraft(&response)
    }

    /// Search launches by query string
    ///
    /// # Arguments
    /// - `query` - Search query (e.g., "SpaceX", "Falcon", "ISS")
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with matching launches
    pub async fn search_launches(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceLaunch>> {
        let mut params = HashMap::new();
        params.insert("search".to_string(), query.to_string());
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::LaunchUpcoming, params).await?;
        LaunchLibraryParser::parse_launches(&response)
    }

    /// Get next upcoming launch (convenience method)
    ///
    /// # Returns
    /// The next upcoming launch
    pub async fn get_next_launch(&self) -> ExchangeResult<SpaceLaunch> {
        let launches = self.get_upcoming_launches(Some(1)).await?;
        launches
            .results
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::NotFound("No upcoming launches found".to_string()))
    }

    /// Get launches by launch service provider (convenience method)
    ///
    /// # Arguments
    /// - `provider` - Provider name (e.g., "SpaceX", "NASA", "Roscosmos")
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated response with launches from the specified provider
    pub async fn get_launches_by_provider(
        &self,
        provider: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<PaginatedResponse<SpaceLaunch>> {
        let mut params = HashMap::new();
        params.insert("lsp__name".to_string(), provider.to_string());
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(LaunchLibraryEndpoint::LaunchUpcoming, params).await?;
        LaunchLibraryParser::parse_launches(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get launcher vehicle instances (specific serial numbers / flight hardware)
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated launcher instance data as raw JSON
    pub async fn get_launchers(&self, limit: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Launcher, params).await
    }

    /// Get launch pad details
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated pad data as raw JSON
    pub async fn get_pads(&self, limit: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Pad, params).await
    }

    /// Get launch location details
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated location data as raw JSON
    pub async fn get_locations(&self, limit: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Location, params).await
    }

    /// Get ISS / space station expedition data
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated expedition data as raw JSON
    pub async fn get_expeditions(&self, limit: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Expedition, params).await
    }

    /// Get docking events (ISS, Mir, etc.)
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated docking event data as raw JSON
    pub async fn get_docking_events(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Docking, params).await
    }

    /// Get payload (spacecraft flight vehicle) information
    ///
    /// # Arguments
    /// - `limit` - Optional limit
    ///
    /// # Returns
    /// Paginated payload data as raw JSON
    pub async fn get_payloads(&self, limit: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        self.get(LaunchLibraryEndpoint::Payload, params).await
    }
}

impl Default for LaunchLibraryConnector {
    fn default() -> Self {
        Self::new()
    }
}
