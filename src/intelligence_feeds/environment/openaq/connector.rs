//! OpenAQ connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    OpenAqParser, OpenAqLocation, OpenAqMeasurement, OpenAqCountry,
    OpenAqCity, OpenAqParameter, OpenAqLatest,
};

/// OpenAQ (Open Air Quality) connector
///
/// Provides access to global air quality data from 10,000+ monitoring locations worldwide.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::openaq::OpenAqConnector;
///
/// // With API key
/// let connector = OpenAqConnector::from_env();
///
/// // Or public access (no API key)
/// let connector = OpenAqConnector::public();
///
/// // Get locations in a city
/// let locations = connector.get_locations(Some("US"), Some("New York"), None, None).await?;
///
/// // Get latest measurements
/// let latest = connector.get_latest(Some("US"), None, Some(10)).await?;
///
/// // Get PM2.5 measurements
/// let pm25 = connector.get_pm25_readings("US", None, None).await?;
/// ```
pub struct OpenAqConnector {
    client: Client,
    auth: OpenAqAuth,
    endpoints: OpenAqEndpoints,
    _testnet: bool,
}

impl OpenAqConnector {
    /// Create new OpenAQ connector with authentication
    pub fn new(auth: OpenAqAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenAqEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENAQ_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(OpenAqAuth::from_env())
    }

    /// Create connector for public access (no API key)
    pub fn public() -> Self {
        Self::new(OpenAqAuth::public())
    }

    /// Internal: Make GET request to OpenAQ API
    async fn get(
        &self,
        endpoint: OpenAqEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).query(&params);

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

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for OpenAQ API errors
        OpenAqParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with path parameter (for location by ID)
    async fn get_with_path(
        &self,
        path: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, path);

        // Build headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).query(&params);

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

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for OpenAQ API errors
        OpenAqParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OPENAQ-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get monitoring locations
    ///
    /// # Arguments
    /// - `country` - Optional country code (ISO 3166-1 alpha-2)
    /// - `city` - Optional city name
    /// - `limit` - Optional limit (max 100, default 100)
    /// - `page` - Optional page number
    ///
    /// # Returns
    /// List of monitoring locations matching criteria
    pub async fn get_locations(
        &self,
        country: Option<&str>,
        city: Option<&str>,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<OpenAqLocation>> {
        let mut params = HashMap::new();

        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }
        if let Some(ct) = city {
            params.insert("city".to_string(), ct.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(OpenAqEndpoint::Locations, params).await?;
        OpenAqParser::parse_locations(&response)
    }

    /// Get specific location by ID
    ///
    /// # Arguments
    /// - `id` - Location ID
    ///
    /// # Returns
    /// Location details
    pub async fn get_location(&self, id: i64) -> ExchangeResult<OpenAqLocation> {
        let path = format!("/locations/{}", id);
        let params = HashMap::new();

        let response = self.get_with_path(&path, params).await?;
        let locations = OpenAqParser::parse_locations(&response)?;

        locations
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::NotFound("Location not found".to_string()))
    }

    /// Get air quality measurements
    ///
    /// # Arguments
    /// - `location_id` - Optional location ID
    /// - `parameter` - Optional parameter (pm25, pm10, o3, no2, so2, co)
    /// - `date_from` - Optional start date (YYYY-MM-DD or ISO 8601)
    /// - `date_to` - Optional end date (YYYY-MM-DD or ISO 8601)
    /// - `limit` - Optional limit (max 10000, default 100)
    ///
    /// # Returns
    /// List of measurements
    pub async fn get_measurements(
        &self,
        location_id: Option<i64>,
        parameter: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenAqMeasurement>> {
        let mut params = HashMap::new();

        if let Some(loc_id) = location_id {
            params.insert("location_id".to_string(), loc_id.to_string());
        }
        if let Some(p) = parameter {
            params.insert("parameter".to_string(), p.to_string());
        }
        if let Some(from) = date_from {
            params.insert("date_from".to_string(), from.to_string());
        }
        if let Some(to) = date_to {
            params.insert("date_to".to_string(), to.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(OpenAqEndpoint::Measurements, params).await?;
        OpenAqParser::parse_measurements(&response)
    }

    /// Get latest measurements from all locations
    ///
    /// # Arguments
    /// - `country` - Optional country code
    /// - `city` - Optional city name
    /// - `limit` - Optional limit (max 10000, default 100)
    ///
    /// # Returns
    /// Latest measurements grouped by location
    pub async fn get_latest(
        &self,
        country: Option<&str>,
        city: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenAqLatest>> {
        let mut params = HashMap::new();

        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }
        if let Some(ct) = city {
            params.insert("city".to_string(), ct.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(OpenAqEndpoint::Latest, params).await?;
        OpenAqParser::parse_latest(&response)
    }

    /// Get list of countries with air quality data
    ///
    /// # Returns
    /// List of countries with location and measurement counts
    pub async fn get_countries(&self) -> ExchangeResult<Vec<OpenAqCountry>> {
        let params = HashMap::new();

        let response = self.get(OpenAqEndpoint::Countries, params).await?;
        OpenAqParser::parse_countries(&response)
    }

    /// Get list of cities with air quality data
    ///
    /// # Arguments
    /// - `country` - Optional country code to filter
    /// - `limit` - Optional limit (max 10000, default 100)
    ///
    /// # Returns
    /// List of cities with location and measurement counts
    pub async fn get_cities(
        &self,
        country: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenAqCity>> {
        let mut params = HashMap::new();

        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(OpenAqEndpoint::Cities, params).await?;
        OpenAqParser::parse_cities(&response)
    }

    /// Get list of measured parameters (pollutants)
    ///
    /// # Returns
    /// List of parameters (PM2.5, PM10, O3, NO2, SO2, CO)
    pub async fn get_parameters(&self) -> ExchangeResult<Vec<OpenAqParameter>> {
        let params = HashMap::new();

        let response = self.get(OpenAqEndpoint::Parameters, params).await?;
        OpenAqParser::parse_parameters(&response)
    }

    /// Get air quality measurements for a country
    ///
    /// # Arguments
    /// - `country` - Country code (ISO 3166-1 alpha-2)
    /// - `parameter` - Optional parameter filter (pm25, pm10, o3, no2, so2, co)
    /// - `date_from` - Optional start date
    /// - `date_to` - Optional end date
    ///
    /// # Returns
    /// Measurements for the country
    pub async fn get_country_air_quality(
        &self,
        country: &str,
        parameter: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<OpenAqMeasurement>> {
        let mut params = HashMap::new();
        params.insert("country".to_string(), country.to_string());

        if let Some(p) = parameter {
            params.insert("parameter".to_string(), p.to_string());
        }
        if let Some(from) = date_from {
            params.insert("date_from".to_string(), from.to_string());
        }
        if let Some(to) = date_to {
            params.insert("date_to".to_string(), to.to_string());
        }

        let response = self.get(OpenAqEndpoint::Measurements, params).await?;
        OpenAqParser::parse_measurements(&response)
    }

    /// Get air quality measurements for a city
    ///
    /// # Arguments
    /// - `country` - Country code
    /// - `city` - City name
    /// - `parameter` - Optional parameter filter
    /// - `date_from` - Optional start date
    /// - `date_to` - Optional end date
    ///
    /// # Returns
    /// Measurements for the city
    pub async fn get_city_air_quality(
        &self,
        country: &str,
        city: &str,
        parameter: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<OpenAqMeasurement>> {
        let mut params = HashMap::new();
        params.insert("country".to_string(), country.to_string());
        params.insert("city".to_string(), city.to_string());

        if let Some(p) = parameter {
            params.insert("parameter".to_string(), p.to_string());
        }
        if let Some(from) = date_from {
            params.insert("date_from".to_string(), from.to_string());
        }
        if let Some(to) = date_to {
            params.insert("date_to".to_string(), to.to_string());
        }

        let response = self.get(OpenAqEndpoint::Measurements, params).await?;
        OpenAqParser::parse_measurements(&response)
    }

    /// Convenience: Get PM2.5 measurements for a country
    ///
    /// PM2.5 (particulate matter less than 2.5 micrometers) is the most
    /// commonly tracked air quality metric.
    ///
    /// # Arguments
    /// - `country` - Country code
    /// - `date_from` - Optional start date
    /// - `date_to` - Optional end date
    ///
    /// # Returns
    /// PM2.5 measurements
    pub async fn get_pm25_readings(
        &self,
        country: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<OpenAqMeasurement>> {
        self.get_country_air_quality(country, Some("pm25"), date_from, date_to)
            .await
    }

    /// Get pollution hotspots (locations with highest recent pollution)
    ///
    /// Returns locations sorted by highest recent PM2.5 levels.
    ///
    /// # Arguments
    /// - `limit` - Number of locations to return (default 10)
    ///
    /// # Returns
    /// Latest measurements sorted by PM2.5 value (descending)
    pub async fn get_pollution_hotspots(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<OpenAqLatest>> {
        let mut params = HashMap::new();
        params.insert("parameter".to_string(), "pm25".to_string());
        params.insert("order_by".to_string(), "value".to_string());
        params.insert("sort".to_string(), "desc".to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        } else {
            params.insert("limit".to_string(), "10".to_string());
        }

        let response = self.get(OpenAqEndpoint::Latest, params).await?;
        OpenAqParser::parse_latest(&response)
    }
}
