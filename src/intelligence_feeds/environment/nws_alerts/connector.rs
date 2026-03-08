//! NWS Weather Alerts connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::Value;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{NwsAlertsParser, WeatherAlert, Severity};

/// NWS Weather Alerts connector
///
/// Provides access to official US weather alerts from the National Weather Service.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::environment::nws_alerts::NwsAlertsConnector;
///
/// let connector = NwsAlertsConnector::new();
///
/// // Get all active alerts
/// let alerts = connector.get_active_alerts().await?;
///
/// // Get alerts for a specific state
/// let tx_alerts = connector.get_alerts_by_area("TX").await?;
///
/// // Get alerts for a specific zone
/// let zone_alerts = connector.get_alerts_by_zone("TXZ253").await?;
///
/// // Get only severe alerts
/// let severe = connector.get_severe_alerts().await?;
/// ```
pub struct NwsAlertsConnector {
    client: Client,
    auth: NwsAlertsAuth,
    endpoints: NwsAlertsEndpoints,
}

impl NwsAlertsConnector {
    /// Create new NWS Alerts connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: NwsAlertsAuth::default(),
            endpoints: NwsAlertsEndpoints::default(),
        }
    }

    /// Create new NWS Alerts connector with custom User-Agent
    pub fn with_user_agent(user_agent: String) -> Self {
        Self {
            client: Client::new(),
            auth: NwsAlertsAuth::new(user_agent),
            endpoints: NwsAlertsEndpoints::default(),
        }
    }

    /// Internal: Make GET request to NWS API
    async fn get(&self, endpoint: NwsAlertsEndpoint) -> ExchangeResult<String> {
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add headers
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

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get all currently active weather alerts across the US
    ///
    /// # Returns
    /// Vector of all active weather alerts
    ///
    /// # Example
    /// ```ignore
    /// let alerts = connector.get_active_alerts().await?;
    /// println!("Found {} active alerts", alerts.len());
    /// ```
    pub async fn get_active_alerts(&self) -> ExchangeResult<Vec<WeatherAlert>> {
        let json_str = self.get(NwsAlertsEndpoint::ActiveAlerts).await?;
        let data: Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        NwsAlertsParser::parse_alerts(&data)
    }

    /// Get specific alert by its ID
    ///
    /// # Arguments
    /// - `id` - Alert URN identifier (e.g., "urn:oid:2.49.0.1.840.0...")
    ///
    /// # Returns
    /// Single weather alert
    ///
    /// # Example
    /// ```ignore
    /// let alert = connector.get_alert_by_id("urn:oid:2.49.0.1.840.0.1f2a3b4c").await?;
    /// ```
    pub async fn get_alert_by_id(&self, id: &str) -> ExchangeResult<WeatherAlert> {
        let json_str = self.get(NwsAlertsEndpoint::AlertById(id.to_string())).await?;
        let data: Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        // Single alert response is a Feature, not FeatureCollection
        NwsAlertsParser::parse_alert(&data)
    }

    /// Get active alerts for a specific NWS forecast zone
    ///
    /// # Arguments
    /// - `zone` - NWS zone identifier (e.g., "TXZ253", "KSZ027", "FLC015")
    ///
    /// # Returns
    /// Vector of weather alerts affecting the specified zone
    ///
    /// # Zone Format
    /// - Format: State (2 letters) + Zone type (1 letter) + Number (3 digits)
    /// - Zone types: `Z` (public forecast zones), `C` (county zones)
    /// - Examples: `TXZ253`, `KSZ027`, `FLC015`
    ///
    /// # Example
    /// ```ignore
    /// let alerts = connector.get_alerts_by_zone("TXZ253").await?;
    /// println!("Bexar County has {} active alerts", alerts.len());
    /// ```
    pub async fn get_alerts_by_zone(&self, zone: &str) -> ExchangeResult<Vec<WeatherAlert>> {
        let json_str = self.get(NwsAlertsEndpoint::AlertsByZone(zone.to_string())).await?;
        let data: Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        NwsAlertsParser::parse_alerts(&data)
    }

    /// Get active alerts for a state, territory, or marine area
    ///
    /// # Arguments
    /// - `area` - Two-letter state/territory code
    ///
    /// # Area Codes
    /// - US States: `AL`, `AK`, `AZ`, ..., `WY`
    /// - Territories: `PR` (Puerto Rico), `GU` (Guam), `VI` (US Virgin Islands)
    /// - Marine: `AM` (Atlantic Marine), `GM` (Gulf of Mexico)
    ///
    /// # Returns
    /// Vector of weather alerts affecting the specified area
    ///
    /// # Example
    /// ```ignore
    /// let alerts = connector.get_alerts_by_area("TX").await?;
    /// println!("Texas has {} active alerts", alerts.len());
    /// ```
    pub async fn get_alerts_by_area(&self, area: &str) -> ExchangeResult<Vec<WeatherAlert>> {
        let json_str = self.get(NwsAlertsEndpoint::AlertsByArea(area.to_string())).await?;
        let data: Value = serde_json::from_str(&json_str)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        NwsAlertsParser::parse_alerts(&data)
    }

    /// Get only severe and extreme alerts (nationwide)
    ///
    /// Filters active alerts to include only those with severity level
    /// of "Extreme" or "Severe".
    ///
    /// # Returns
    /// Vector of severe weather alerts
    ///
    /// # Example
    /// ```ignore
    /// let severe = connector.get_severe_alerts().await?;
    /// for alert in severe {
    ///     println!("SEVERE: {} - {}", alert.event, alert.area_desc.unwrap_or_default());
    /// }
    /// ```
    pub async fn get_severe_alerts(&self) -> ExchangeResult<Vec<WeatherAlert>> {
        let all_alerts = self.get_active_alerts().await?;

        Ok(all_alerts
            .into_iter()
            .filter(|alert| {
                matches!(alert.severity, Severity::Extreme | Severity::Severe)
            })
            .collect())
    }

    /// Get extreme alerts only (highest severity)
    ///
    /// Returns alerts with "Extreme" severity - representing extraordinary
    /// threats to life or property.
    ///
    /// # Returns
    /// Vector of extreme weather alerts
    ///
    /// # Example
    /// ```ignore
    /// let extreme = connector.get_extreme_alerts().await?;
    /// println!("CRITICAL: {} extreme alerts active", extreme.len());
    /// ```
    pub async fn get_extreme_alerts(&self) -> ExchangeResult<Vec<WeatherAlert>> {
        let all_alerts = self.get_active_alerts().await?;

        Ok(all_alerts
            .into_iter()
            .filter(|alert| alert.severity == Severity::Extreme)
            .collect())
    }

    /// Get alerts with immediate urgency
    ///
    /// Returns alerts where responsive action should be taken immediately.
    ///
    /// # Returns
    /// Vector of immediate urgency weather alerts
    ///
    /// # Example
    /// ```ignore
    /// let urgent = connector.get_immediate_alerts().await?;
    /// ```
    pub async fn get_immediate_alerts(&self) -> ExchangeResult<Vec<WeatherAlert>> {
        let all_alerts = self.get_active_alerts().await?;

        use super::parser::Urgency;
        Ok(all_alerts
            .into_iter()
            .filter(|alert| alert.urgency == Urgency::Immediate)
            .collect())
    }
}

impl Default for NwsAlertsConnector {
    fn default() -> Self {
        Self::new()
    }
}
