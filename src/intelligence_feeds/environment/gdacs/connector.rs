//! GDACS connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{GdacsParser, DisasterEvent, DisasterType, AlertLevel};

/// GDACS (Global Disaster Alert and Coordination System) connector
///
/// Provides access to real-time disaster alerts and humanitarian impact data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::gdacs::GdacsConnector;
///
/// let connector = GdacsConnector::new();
///
/// // Get all events
/// let events = connector.get_all_events().await?;
///
/// // Get earthquakes only
/// let earthquakes = connector.get_events_by_type(DisasterType::Earthquake).await?;
///
/// // Get active alerts (Orange and Red only)
/// let alerts = connector.get_active_alerts().await?;
/// ```
pub struct GdacsConnector {
    client: Client,
    auth: GdacsAuth,
    endpoints: GdacsEndpoints,
    _testnet: bool,
}

impl GdacsConnector {
    /// Create new GDACS connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: GdacsAuth::new(),
            endpoints: GdacsEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to GDACS API
    async fn get(&self, mut params: HashMap<String, String>) -> ExchangeResult<String> {
        // No authentication needed for GDACS
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, GdacsEndpoint::EventList.path());

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
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

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get all disaster events
    ///
    /// Returns recent events across all disaster types.
    /// Results are paginated (max 100 per request).
    pub async fn get_all_events(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        let params = HashMap::new();
        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        GdacsParser::parse_events(&data)
    }

    /// Get events by disaster type
    ///
    /// # Arguments
    /// - `disaster_type` - Type of disaster to filter (EQ, TC, FL, VO, WF, DR, TS)
    ///
    /// # Returns
    /// List of events matching the disaster type
    pub async fn get_events_by_type(
        &self,
        disaster_type: DisasterType,
    ) -> ExchangeResult<Vec<DisasterEvent>> {
        let mut params = HashMap::new();
        params.insert("eventlist".to_string(), disaster_type.code().to_string());

        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        GdacsParser::parse_events(&data)
    }

    /// Get event by ID
    ///
    /// # Arguments
    /// - `event_type` - Event type code (e.g., "EQ", "TC")
    /// - `event_id` - Event ID as string
    ///
    /// # Returns
    /// Single event matching the ID
    pub async fn get_event_by_id(
        &self,
        event_type: &str,
        event_id: &str,
    ) -> ExchangeResult<DisasterEvent> {
        let mut params = HashMap::new();
        params.insert("eventtype".to_string(), event_type.to_string());
        params.insert("eventid".to_string(), event_id.to_string());

        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        let events = GdacsParser::parse_events(&data)?;

        events
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("Event not found".to_string()))
    }

    /// Get active alerts (Orange and Red levels only)
    ///
    /// Filters out Green (minor) alerts to focus on events requiring
    /// national or international response.
    ///
    /// # Returns
    /// List of events with Orange or Red alert levels
    pub async fn get_active_alerts(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        let mut params = HashMap::new();
        params.insert("alertlevel".to_string(), "orange;red".to_string());

        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        GdacsParser::parse_events(&data)
    }

    /// Get events by alert level
    ///
    /// # Arguments
    /// - `alert_level` - Alert level to filter (Green, Orange, Red)
    ///
    /// # Returns
    /// List of events matching the alert level
    pub async fn get_events_by_alert_level(
        &self,
        alert_level: AlertLevel,
    ) -> ExchangeResult<Vec<DisasterEvent>> {
        let mut params = HashMap::new();
        params.insert("alertlevel".to_string(), alert_level.as_str().to_string());

        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        GdacsParser::parse_events(&data)
    }

    /// Get earthquakes
    ///
    /// Returns recent earthquake events with GDACS alerts.
    pub async fn get_earthquakes(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Earthquake).await
    }

    /// Get tropical cyclones (hurricanes, typhoons)
    ///
    /// Returns active and recent tropical cyclone events.
    pub async fn get_tropical_cyclones(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::TropicalCyclone).await
    }

    /// Get floods
    ///
    /// Returns recent flood events monitored by GLOFAS.
    pub async fn get_floods(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Flood).await
    }

    /// Get volcanic eruptions
    ///
    /// Returns recent volcanic activity with humanitarian impact.
    pub async fn get_volcanoes(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Volcano).await
    }

    /// Get wildfires
    ///
    /// Returns recent large-scale wildfire events.
    pub async fn get_wildfires(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Wildfire).await
    }

    /// Get droughts
    ///
    /// Returns drought events affecting agricultural regions.
    pub async fn get_droughts(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Drought).await
    }

    /// Get tsunamis
    ///
    /// Returns tsunami events and warnings.
    pub async fn get_tsunamis(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        self.get_events_by_type(DisasterType::Tsunami).await
    }

    /// Get events with date filter
    ///
    /// # Arguments
    /// - `from_date` - Start date (ISO 8601 format: YYYY-MM-DD)
    /// - `to_date` - End date (ISO 8601 format: YYYY-MM-DD)
    ///
    /// # Returns
    /// List of events within the date range
    pub async fn get_events_by_date_range(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> ExchangeResult<Vec<DisasterEvent>> {
        let mut params = HashMap::new();
        params.insert("fromdate".to_string(), from_date.to_string());
        params.insert("todate".to_string(), to_date.to_string());

        let json_str = self.get(params).await?;

        let data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        GdacsParser::parse_events(&data)
    }

    /// Get current events only
    ///
    /// Filters result to only include active/ongoing events.
    ///
    /// # Returns
    /// List of current events (is_current = true)
    pub async fn get_current_events(&self) -> ExchangeResult<Vec<DisasterEvent>> {
        let events = self.get_all_events().await?;
        Ok(events.into_iter().filter(|e| e.is_current).collect())
    }
}

impl Default for GdacsConnector {
    fn default() -> Self {
        Self::new()
    }
}
