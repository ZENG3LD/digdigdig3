//! ACLED connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{AcledParser, AcledEvent};

/// ACLED (Armed Conflict Location & Event Data Project) connector
///
/// Provides access to global conflict and event data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::acled::AcledConnector;
///
/// let connector = AcledConnector::from_env()?;
///
/// // Get recent events
/// let events = connector.get_recent_events(7).await?;
///
/// // Get events by country
/// let syria_events = connector.get_events_by_country("Syria", "2024-01-01", "2024-06-30").await?;
///
/// // Get conflict hotspots
/// let hotspots = connector.get_conflict_hotspots(1, 10).await?;
/// ```
pub struct AcledConnector {
    client: Client,
    auth: AcledAuth,
    endpoints: AcledEndpoints,
    _testnet: bool,
}

impl AcledConnector {
    /// Create new ACLED connector with authentication
    pub fn new(auth: AcledAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AcledEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `ACLED_API_KEY` and `ACLED_EMAIL` environment variables
    pub fn from_env() -> ExchangeResult<Self> {
        let auth = AcledAuth::from_env();
        if !auth.is_authenticated() {
            return Err(ExchangeError::Auth(
                "ACLED_API_KEY and ACLED_EMAIL environment variables must be set".to_string(),
            ));
        }
        Ok(Self::new(auth))
    }

    /// Internal: Make GET request to ACLED API
    async fn get(
        &self,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (includes terms=accept)
        self.auth.sign_query(&mut params);

        let url = self.endpoints.rest_base;

        let response = self
            .client
            .get(url)
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

        // Check for ACLED API errors
        AcledParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACLED-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get events with pagination
    ///
    /// # Arguments
    /// - `limit` - Number of events to return (default: 100)
    /// - `page` - Page number (default: 1)
    ///
    /// # Returns
    /// Vector of ACLED events
    pub async fn get_events(&self, limit: u32, page: u32) -> ExchangeResult<Vec<AcledEvent>> {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("page".to_string(), page.to_string());

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get events by country and date range
    ///
    /// # Arguments
    /// - `country` - Country name (e.g., "Syria", "Ukraine")
    /// - `start_date` - Start date in YYYY-MM-DD format
    /// - `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of ACLED events for the specified country and date range
    pub async fn get_events_by_country(
        &self,
        country: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        let mut params = HashMap::new();
        params.insert("country".to_string(), country.to_string());
        params.insert("event_date".to_string(), "{d}".to_string());
        params.insert("event_date_where".to_string(), "BETWEEN".to_string());
        params.insert("value".to_string(), format!("{}|{}", start_date, end_date));

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get events by region and date range
    ///
    /// # Arguments
    /// - `region` - Region code (1=Western Africa, 2=Middle Africa, etc.)
    /// - `start_date` - Start date in YYYY-MM-DD format
    /// - `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of ACLED events for the specified region
    pub async fn get_events_by_region(
        &self,
        region: u32,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        let mut params = HashMap::new();
        params.insert("region".to_string(), region.to_string());
        params.insert("event_date".to_string(), "{d}".to_string());
        params.insert("event_date_where".to_string(), "BETWEEN".to_string());
        params.insert("value".to_string(), format!("{}|{}", start_date, end_date));

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get events by event type and date range
    ///
    /// # Arguments
    /// - `event_type` - Event type: "Battles", "Explosions/Remote violence",
    ///   "Violence against civilians", "Protests", "Riots", "Strategic developments"
    /// - `start_date` - Start date in YYYY-MM-DD format
    /// - `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of ACLED events matching the specified type
    pub async fn get_events_by_type(
        &self,
        event_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        let mut params = HashMap::new();
        params.insert("event_type".to_string(), event_type.to_string());
        params.insert("event_date".to_string(), "{d}".to_string());
        params.insert("event_date_where".to_string(), "BETWEEN".to_string());
        params.insert("value".to_string(), format!("{}|{}", start_date, end_date));

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get events by actor name and date range
    ///
    /// # Arguments
    /// - `actor` - Actor name (searches both actor1 and actor2)
    /// - `start_date` - Start date in YYYY-MM-DD format
    /// - `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of ACLED events involving the specified actor
    pub async fn get_events_by_actor(
        &self,
        actor: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        let mut params = HashMap::new();
        params.insert("actor1".to_string(), actor.to_string());
        params.insert("event_date".to_string(), "{d}".to_string());
        params.insert("event_date_where".to_string(), "BETWEEN".to_string());
        params.insert("value".to_string(), format!("{}|{}", start_date, end_date));

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get events with fatalities by country and date range
    ///
    /// # Arguments
    /// - `country` - Country name
    /// - `start_date` - Start date in YYYY-MM-DD format
    /// - `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of ACLED events with fatalities > 0
    pub async fn get_fatalities_by_country(
        &self,
        country: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        let events = self.get_events_by_country(country, start_date, end_date).await?;

        // Filter for events with fatalities
        let fatal_events: Vec<AcledEvent> = events
            .into_iter()
            .filter(|event| event.fatalities > 0)
            .collect();

        Ok(fatal_events)
    }

    /// Get recent events from the last N days
    ///
    /// # Arguments
    /// - `days_back` - Number of days to look back from today
    ///
    /// # Returns
    /// Vector of recent ACLED events
    pub async fn get_recent_events(&self, days_back: u32) -> ExchangeResult<Vec<AcledEvent>> {
        use chrono::{Utc, Duration};

        let end_date = Utc::now();
        let start_date = end_date - Duration::days(days_back as i64);

        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        let mut params = HashMap::new();
        params.insert("event_date".to_string(), "{d}".to_string());
        params.insert("event_date_where".to_string(), "BETWEEN".to_string());
        params.insert("value".to_string(), format!("{}|{}", start_str, end_str));

        let response = self.get(params).await?;
        let parsed = AcledParser::parse_events(&response)?;
        Ok(parsed.data)
    }

    /// Get conflict hotspots in a region with minimum fatalities
    ///
    /// # Arguments
    /// - `region` - Region code (1=Western Africa, 2=Middle Africa, etc.)
    /// - `min_fatalities` - Minimum number of fatalities to be considered a hotspot
    ///
    /// # Returns
    /// Vector of high-fatality ACLED events in the specified region
    pub async fn get_conflict_hotspots(
        &self,
        region: u32,
        min_fatalities: u32,
    ) -> ExchangeResult<Vec<AcledEvent>> {
        use chrono::{Utc, Duration};

        // Get last 30 days
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(30);

        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        let events = self.get_events_by_region(region, &start_str, &end_str).await?;

        // Filter for high-fatality events
        let hotspots: Vec<AcledEvent> = events
            .into_iter()
            .filter(|event| event.fatalities >= min_fatalities)
            .collect();

        Ok(hotspots)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get ACLED CAST predictive conflict forecasts
    ///
    /// CAST (Conflict Alert System Tool) provides probabilistic forecasts
    /// of future conflict events based on historical patterns.
    ///
    /// # Arguments
    /// - `country` - Optional country filter
    /// - `limit` - Optional limit (default: 100)
    ///
    /// # Returns
    /// Vector of CAST forecast entries as raw JSON values
    pub async fn get_cast_forecasts(
        &self,
        country: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<serde_json::Value>> {
        let mut params = HashMap::new();

        // ACLED CAST endpoint has its own full URL
        self.auth.sign_query(&mut params);

        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }

        params.insert(
            "limit".to_string(),
            limit.unwrap_or(100).to_string(),
        );

        let url = AcledEndpoint::CastForecasts.path().to_string();

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

        json.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in CAST response".to_string()))
            .cloned()
    }

    /// Get deleted ACLED records
    ///
    /// Returns event IDs that have been removed from the ACLED dataset
    /// (corrections, duplicates, or quality-flagged entries).
    ///
    /// # Arguments
    /// - `since_date` - Optional date filter (only records deleted after this date)
    ///
    /// # Returns
    /// Vector of deleted event records as raw JSON values
    pub async fn get_deleted_records(
        &self,
        since_date: Option<&str>,
    ) -> ExchangeResult<Vec<serde_json::Value>> {
        let mut params = HashMap::new();

        self.auth.sign_query(&mut params);

        if let Some(date) = since_date {
            params.insert("timestamp".to_string(), date.to_string());
        }

        let url = AcledEndpoint::DeletedRecords.path().to_string();

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

        json.get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in deleted records response".to_string()))
            .cloned()
    }
}
