//! USGS Earthquake connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{UsgsEarthquakeParser, EarthquakeResponse};

/// USGS Earthquake Hazards API connector
///
/// Provides access to real-time and historical earthquake data worldwide.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::usgs_earthquake::UsgsEarthquakeConnector;
///
/// let connector = UsgsEarthquakeConnector::new();
///
/// // Get earthquakes from the past week
/// let earthquakes = connector.get_past_week(Some(4.0)).await?;
///
/// // Get earthquakes near a location
/// let near_tokyo = connector.get_by_location(35.6762, 139.6503, 100.0, None, None, Some(5.0)).await?;
///
/// // Get major earthquakes (magnitude >= 6)
/// let major = connector.get_major_earthquakes(Some("2024-01-01"), Some("2024-12-31")).await?;
/// ```
pub struct UsgsEarthquakeConnector {
    client: Client,
    auth: UsgsEarthquakeAuth,
    endpoints: UsgsEarthquakeEndpoints,
    _testnet: bool,
}

impl UsgsEarthquakeConnector {
    /// Create new USGS Earthquake connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: UsgsEarthquakeAuth::new(),
            endpoints: UsgsEarthquakeEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to USGS API
    async fn get(
        &self,
        endpoint: UsgsEarthquakeEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for USGS)
        self.auth.sign_query(&mut params);

        // Always request GeoJSON format
        params.insert("format".to_string(), "geojson".to_string());

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
        UsgsEarthquakeParser::check_error(&json)?;

        Ok(json)
    }

    // Public getter methods

    /// Query earthquakes with custom filters
    ///
    /// # Arguments
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    /// - `min_magnitude` - Minimum magnitude
    /// - `max_magnitude` - Maximum magnitude
    /// - `limit` - Maximum number of results (1-20000, default 20000)
    pub async fn query(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
        min_magnitude: Option<f64>,
        max_magnitude: Option<f64>,
        limit: Option<u32>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }
        if let Some(min_mag) = min_magnitude {
            params.insert("minmagnitude".to_string(), min_mag.to_string());
        }
        if let Some(max_mag) = max_magnitude {
            params.insert("maxmagnitude".to_string(), max_mag.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Get recent earthquakes
    ///
    /// # Arguments
    /// - `hours` - Number of hours to look back (e.g., 24 for past day)
    /// - `min_magnitude` - Minimum magnitude filter
    pub async fn get_recent(
        &self,
        hours: u32,
        min_magnitude: Option<f64>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        // Calculate start time (current time - hours)
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::hours(hours as i64);
        params.insert("starttime".to_string(), start.to_rfc3339());

        if let Some(min_mag) = min_magnitude {
            params.insert("minmagnitude".to_string(), min_mag.to_string());
        }

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Get significant earthquakes only
    ///
    /// # Arguments
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    pub async fn get_significant(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }

        // Filter for significant events
        params.insert("orderby".to_string(), "magnitude".to_string());
        params.insert("minmagnitude".to_string(), "4.5".to_string());

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Get earthquakes near a location
    ///
    /// # Arguments
    /// - `lat` - Latitude
    /// - `lon` - Longitude
    /// - `radius_km` - Radius in kilometers
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    /// - `min_magnitude` - Minimum magnitude
    pub async fn get_by_location(
        &self,
        lat: f64,
        lon: f64,
        radius_km: f64,
        start_time: Option<&str>,
        end_time: Option<&str>,
        min_magnitude: Option<f64>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        params.insert("latitude".to_string(), lat.to_string());
        params.insert("longitude".to_string(), lon.to_string());
        params.insert("maxradiuskm".to_string(), radius_km.to_string());

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }
        if let Some(min_mag) = min_magnitude {
            params.insert("minmagnitude".to_string(), min_mag.to_string());
        }

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Get earthquakes in a bounding box region
    ///
    /// # Arguments
    /// - `min_lat` - Minimum latitude
    /// - `max_lat` - Maximum latitude
    /// - `min_lon` - Minimum longitude
    /// - `max_lon` - Maximum longitude
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    pub async fn get_by_region(
        &self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        params.insert("minlatitude".to_string(), min_lat.to_string());
        params.insert("maxlatitude".to_string(), max_lat.to_string());
        params.insert("minlongitude".to_string(), min_lon.to_string());
        params.insert("maxlongitude".to_string(), max_lon.to_string());

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Count earthquakes matching criteria
    ///
    /// # Arguments
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    /// - `min_magnitude` - Minimum magnitude
    pub async fn count(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
        min_magnitude: Option<f64>,
    ) -> ExchangeResult<u64> {
        let mut params = HashMap::new();

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }
        if let Some(min_mag) = min_magnitude {
            params.insert("minmagnitude".to_string(), min_mag.to_string());
        }

        let response = self.get(UsgsEarthquakeEndpoint::Count, params).await?;
        UsgsEarthquakeParser::parse_count_response(&response)
    }

    /// Get major earthquakes (magnitude >= 6)
    ///
    /// # Arguments
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    pub async fn get_major_earthquakes(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> ExchangeResult<EarthquakeResponse> {
        self.query(start_time, end_time, Some(6.0), None, None).await
    }

    /// Get tsunami-generating earthquakes
    ///
    /// # Arguments
    /// - `start_time` - Start time (YYYY-MM-DD or ISO8601)
    /// - `end_time` - End time (YYYY-MM-DD or ISO8601)
    pub async fn get_tsunami_events(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let mut params = HashMap::new();

        if let Some(start) = start_time {
            params.insert("starttime".to_string(), start.to_string());
        }
        if let Some(end) = end_time {
            params.insert("endtime".to_string(), end.to_string());
        }

        // Filter for tsunami events
        params.insert("alertlevel".to_string(), "red".to_string());

        let response = self.get(UsgsEarthquakeEndpoint::Query, params).await?;
        UsgsEarthquakeParser::parse_earthquake_response(&response)
    }

    /// Get today's earthquakes
    pub async fn get_today(&self) -> ExchangeResult<EarthquakeResponse> {
        let now = chrono::Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start_str = format!("{}", start.format("%Y-%m-%d"));

        self.query(Some(&start_str), None, None, None, None).await
    }

    /// Get past week's earthquakes
    ///
    /// # Arguments
    /// - `min_magnitude` - Minimum magnitude filter
    pub async fn get_past_week(
        &self,
        min_magnitude: Option<f64>,
    ) -> ExchangeResult<EarthquakeResponse> {
        let now = chrono::Utc::now();
        let week_ago = now - chrono::Duration::days(7);
        let start_str = format!("{}", week_ago.format("%Y-%m-%d"));

        self.query(Some(&start_str), None, min_magnitude, None, None).await
    }
}

impl Default for UsgsEarthquakeConnector {
    fn default() -> Self {
        Self::new()
    }
}
