//! NASA EONET connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{NasaEonetParser, NaturalEvent, EventCategory, EventSource};

/// NASA EONET (Earth Observatory Natural Event Tracker) connector
///
/// Provides access to natural disaster and environmental event data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::nasa_eonet::NasaEonetConnector;
///
/// let connector = NasaEonetConnector::new();
///
/// // Get open events from last 30 days
/// let events = connector.get_open_events(Some(30)).await?;
///
/// // Get wildfire events
/// let wildfires = connector.get_events_by_category("wildfires", Some(7)).await?;
///
/// // Get all categories
/// let categories = connector.get_categories().await?;
///
/// // Get specific event
/// let event = connector.get_event_by_id("EONET_17841").await?;
/// ```
pub struct NasaEonetConnector {
    client: Client,
    _auth: NasaEonetAuth,
    endpoints: NasaEonetEndpoints,
}

impl NasaEonetConnector {
    /// Create new NASA EONET connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: NasaEonetAuth::new(),
            endpoints: NasaEonetEndpoints::default(),
        }
    }

    /// Internal: Make GET request to EONET API
    async fn get(&self, endpoint: NasaEonetEndpoint, params: HashMap<String, String>) -> ExchangeResult<String> {
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

        let json = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(json)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get open events
    ///
    /// # Arguments
    /// - `days` - Optional number of days to look back (e.g., Some(30) for last 30 days)
    ///
    /// # Returns
    /// List of natural events
    pub async fn get_open_events(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "open".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get closed events
    ///
    /// # Arguments
    /// - `days` - Optional number of days to look back
    ///
    /// # Returns
    /// List of closed natural events
    pub async fn get_closed_events(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "closed".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get all events (open and closed)
    ///
    /// # Arguments
    /// - `days` - Optional number of days to look back
    /// - `limit` - Optional limit on number of results
    ///
    /// # Returns
    /// List of all natural events
    pub async fn get_all_events(&self, days: Option<u32>, limit: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        params.insert("status".to_string(), "all".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get events by category
    ///
    /// # Arguments
    /// - `category` - Category ID (e.g., "wildfires", "severeStorms", "volcanoes")
    /// - `days` - Optional number of days to look back
    ///
    /// # Category IDs
    /// - `wildfires` - Wildfires
    /// - `severeStorms` - Severe Storms (hurricanes, cyclones, tornadoes)
    /// - `volcanoes` - Volcanic Eruptions
    /// - `floods` - Floods
    /// - `earthquakes` - Earthquakes
    /// - `drought` - Drought
    /// - `landslides` - Landslides
    /// - `dustHaze` - Dust and Haze
    /// - `snow` - Snow
    /// - `tempExtremes` - Temperature Extremes
    /// - `seaLakeIce` - Sea and Lake Ice (icebergs)
    /// - `waterColor` - Water Color (algae, phytoplankton)
    /// - `manmade` - Manmade Events
    ///
    /// # Returns
    /// List of natural events in the category
    pub async fn get_events_by_category(&self, category: &str, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), category.to_string());
        params.insert("status".to_string(), "open".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get events by source
    ///
    /// # Arguments
    /// - `source` - Source ID (e.g., "InciWeb", "USGS_EHP", "CALFIRE")
    /// - `days` - Optional number of days to look back
    ///
    /// # Returns
    /// List of natural events from the source
    pub async fn get_events_by_source(&self, source: &str, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), source.to_string());
        params.insert("status".to_string(), "open".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get events within bounding box
    ///
    /// # Arguments
    /// - `min_lon` - Minimum longitude (west)
    /// - `min_lat` - Minimum latitude (south)
    /// - `max_lon` - Maximum longitude (east)
    /// - `max_lat` - Maximum latitude (north)
    /// - `days` - Optional number of days to look back
    ///
    /// # Returns
    /// List of natural events within the geographic area
    pub async fn get_events_in_bbox(
        &self,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
        days: Option<u32>,
    ) -> ExchangeResult<Vec<NaturalEvent>> {
        let mut params = HashMap::new();
        let bbox = format!("{},{},{},{}", min_lon, min_lat, max_lon, max_lat);
        params.insert("bbox".to_string(), bbox);
        params.insert("status".to_string(), "open".to_string());

        if let Some(d) = days {
            params.insert("days".to_string(), d.to_string());
        }

        let json = self.get(NasaEonetEndpoint::Events, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_events(&data)
    }

    /// Get all event categories
    ///
    /// # Returns
    /// List of all event categories
    pub async fn get_categories(&self) -> ExchangeResult<Vec<EventCategory>> {
        let params = HashMap::new();
        let json = self.get(NasaEonetEndpoint::Categories, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_categories(&data)
    }

    /// Get all event sources
    ///
    /// # Returns
    /// List of all event sources
    pub async fn get_sources(&self) -> ExchangeResult<Vec<EventSource>> {
        let params = HashMap::new();
        let json = self.get(NasaEonetEndpoint::Sources, params).await?;
        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_sources(&data)
    }

    /// Get specific event by ID
    ///
    /// # Arguments
    /// - `id` - Event ID (e.g., "EONET_17841")
    ///
    /// # Returns
    /// Natural event details
    pub async fn get_event_by_id(&self, id: &str) -> ExchangeResult<NaturalEvent> {
        let params: HashMap<String, String> = HashMap::new();
        let url = format!("{}/events/{}", self.endpoints.rest_base, id);

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

        let json = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        let data: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NasaEonetParser::parse_single_event(&data)
    }

    // ==========================================================================
    // CONVENIENCE METHODS FOR SPECIFIC EVENT TYPES
    // ==========================================================================

    /// Get active wildfire events
    pub async fn get_wildfires(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("wildfires", days).await
    }

    /// Get active severe storm events (hurricanes, cyclones, tornadoes)
    pub async fn get_severe_storms(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("severeStorms", days).await
    }

    /// Get volcanic eruption events
    pub async fn get_volcanoes(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("volcanoes", days).await
    }

    /// Get earthquake events
    pub async fn get_earthquakes(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("earthquakes", days).await
    }

    /// Get flood events
    pub async fn get_floods(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("floods", days).await
    }

    /// Get drought events
    pub async fn get_droughts(&self, days: Option<u32>) -> ExchangeResult<Vec<NaturalEvent>> {
        self.get_events_by_category("drought", days).await
    }
}

impl Default for NasaEonetConnector {
    fn default() -> Self {
        Self::new()
    }
}
