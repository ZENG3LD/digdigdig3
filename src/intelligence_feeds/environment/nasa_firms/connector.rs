//! NASA FIRMS connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    NasaFirmsParser, FireHotspot, FireSummary,
};

/// NASA FIRMS (Fire Information for Resource Management System) connector
///
/// Provides access to near-real-time active fire data from MODIS and VIIRS satellites.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::nasa_firms::NasaFirmsConnector;
///
/// // With API key from environment
/// let connector = NasaFirmsConnector::from_env();
///
/// // Get global fires in last 24 hours
/// let fires = connector.get_global_fires_24h().await?;
///
/// // Get fires in a bounding box
/// let bbox_fires = connector.get_fires_by_area(
///     "VIIRS_NOAA20_NRT",
///     "-125,24,-66,49", // Continental USA
///     1,
///     None
/// ).await?;
///
/// // Get fires near a location
/// let nearby = connector.get_fires_near(37.7749, -122.4194, 100.0, 2).await?;
/// ```
pub struct NasaFirmsConnector {
    client: Client,
    auth: NasaFirmsAuth,
    endpoints: NasaFirmsEndpoints,
}

impl NasaFirmsConnector {
    /// Create new NASA FIRMS connector with authentication
    pub fn new(auth: NasaFirmsAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: NasaFirmsEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `NASA_FIRMS_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(NasaFirmsAuth::from_env())
    }

    /// Internal: Make GET request to NASA FIRMS API
    async fn get(
        &self,
        endpoint: NasaFirmsEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication
        self.auth.sign_params(&mut params);

        let request = self.client.get(&url).query(&params);

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

        // Check for NASA FIRMS API errors
        NasaFirmsParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NASA FIRMS-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get fire data by geographic area (bounding box)
    ///
    /// # Arguments
    /// - `source` - Data source (VIIRS_NOAA20_NRT, VIIRS_SNPP_NRT, MODIS_NRT)
    /// - `bbox` - Bounding box as "west,south,east,north" (e.g., "-180,-90,180,90")
    ///   or "world" for global data
    /// - `days` - Number of days (1-10)
    /// - `date` - Optional specific date (YYYY-MM-DD), defaults to today
    ///
    /// # Returns
    /// List of fire hotspots in the specified area
    ///
    /// # Example
    /// ```ignore
    /// // Get fires in California area for last 2 days
    /// let fires = connector.get_fires_by_area(
    ///     "VIIRS_NOAA20_NRT",
    ///     "-124.4,32.5,-114.1,42.0",
    ///     2,
    ///     None
    /// ).await?;
    /// ```
    pub async fn get_fires_by_area(
        &self,
        source: &str,
        bbox: &str,
        days: u32,
        date: Option<&str>,
    ) -> ExchangeResult<Vec<FireHotspot>> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), source.to_string());
        params.insert("area".to_string(), bbox.to_string());
        params.insert("day_range".to_string(), days.to_string());
        params.insert("format".to_string(), "json".to_string());

        if let Some(d) = date {
            params.insert("date".to_string(), d.to_string());
        }

        let response = self.get(NasaFirmsEndpoint::Area, params).await?;
        NasaFirmsParser::parse_fire_hotspots(&response)
    }

    /// Get fire data by country code
    ///
    /// # Arguments
    /// - `source` - Data source (VIIRS_NOAA20_NRT, VIIRS_SNPP_NRT, MODIS_NRT)
    /// - `country` - ISO 3166-1 alpha-3 country code (e.g., "USA", "BRA", "AUS")
    /// - `days` - Number of days (1-10)
    ///
    /// # Returns
    /// List of fire hotspots in the specified country
    ///
    /// # Example
    /// ```ignore
    /// // Get fires in USA for last 24 hours
    /// let fires = connector.get_fires_by_country(
    ///     "VIIRS_NOAA20_NRT",
    ///     "USA",
    ///     1
    /// ).await?;
    /// ```
    pub async fn get_fires_by_country(
        &self,
        source: &str,
        country: &str,
        days: u32,
    ) -> ExchangeResult<Vec<FireHotspot>> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), source.to_string());
        params.insert("country".to_string(), country.to_string());
        params.insert("day_range".to_string(), days.to_string());
        params.insert("format".to_string(), "json".to_string());

        let response = self.get(NasaFirmsEndpoint::Country, params).await?;
        NasaFirmsParser::parse_fire_hotspots(&response)
    }

    /// Get global fires in the last 24 hours
    ///
    /// Convenience method using VIIRS NOAA-20 NRT data source (highest resolution)
    ///
    /// # Returns
    /// List of all fire hotspots detected globally in the last 24 hours
    ///
    /// # Example
    /// ```ignore
    /// let fires = connector.get_global_fires_24h().await?;
    /// println!("Total fires detected: {}", fires.len());
    /// ```
    pub async fn get_global_fires_24h(&self) -> ExchangeResult<Vec<FireHotspot>> {
        self.get_fires_by_area("VIIRS_NOAA20_NRT", "world", 1, None)
            .await
    }

    /// Get fires near a specific location
    ///
    /// # Arguments
    /// - `lat` - Latitude (decimal degrees)
    /// - `lon` - Longitude (decimal degrees)
    /// - `radius_km` - Search radius in kilometers
    /// - `days` - Number of days (1-10)
    ///
    /// # Returns
    /// List of fire hotspots within the specified radius
    ///
    /// # Example
    /// ```ignore
    /// // Get fires within 50km of San Francisco for last 3 days
    /// let fires = connector.get_fires_near(37.7749, -122.4194, 50.0, 3).await?;
    /// ```
    pub async fn get_fires_near(
        &self,
        lat: f64,
        lon: f64,
        radius_km: f64,
        days: u32,
    ) -> ExchangeResult<Vec<FireHotspot>> {
        // Calculate approximate bounding box from center point and radius
        // 1 degree latitude ≈ 111 km
        // 1 degree longitude ≈ 111 km * cos(latitude)
        let lat_offset = radius_km / 111.0;
        let lon_offset = radius_km / (111.0 * lat.to_radians().cos());

        let south = lat - lat_offset;
        let north = lat + lat_offset;
        let west = lon - lon_offset;
        let east = lon + lon_offset;

        let bbox = format!("{},{},{},{}", west, south, east, north);

        // Get all fires in the bounding box
        let all_fires = self.get_fires_by_area("VIIRS_NOAA20_NRT", &bbox, days, None).await?;

        // Filter by actual distance
        let filtered_fires: Vec<FireHotspot> = all_fires
            .into_iter()
            .filter(|fire| {
                if let (Some(fire_lat), Some(fire_lon)) = (fire.latitude, fire.longitude) {
                    let distance = haversine_distance(lat, lon, fire_lat, fire_lon);
                    distance <= radius_km
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered_fires)
    }

    /// Get fire summary by area
    ///
    /// Returns aggregated statistics about fires in the specified area
    ///
    /// # Arguments
    /// - `source` - Data source
    /// - `bbox` - Bounding box or "world"
    /// - `days` - Number of days (1-10)
    ///
    /// # Returns
    /// Fire summary with total count and breakdown by country
    ///
    /// # Example
    /// ```ignore
    /// let summary = connector.get_fire_summary("VIIRS_NOAA20_NRT", "world", 1).await?;
    /// println!("Total fires: {}", summary.total_fires);
    /// for country in &summary.countries {
    ///     println!("{}: {} fires", country.country_code, country.fire_count);
    /// }
    /// ```
    pub async fn get_fire_summary(
        &self,
        source: &str,
        bbox: &str,
        days: u32,
    ) -> ExchangeResult<FireSummary> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), source.to_string());
        params.insert("area".to_string(), bbox.to_string());
        params.insert("day_range".to_string(), days.to_string());
        params.insert("format".to_string(), "json".to_string());

        let response = self.get(NasaFirmsEndpoint::Area, params).await?;
        NasaFirmsParser::parse_fire_summary(&response)
    }

    /// Get high-confidence fires only
    ///
    /// Filters for fires with "high" or "nominal" confidence
    ///
    /// # Arguments
    /// - `source` - Data source
    /// - `bbox` - Bounding box or "world"
    /// - `days` - Number of days (1-10)
    ///
    /// # Returns
    /// List of high-confidence fire hotspots
    pub async fn get_high_confidence_fires(
        &self,
        source: &str,
        bbox: &str,
        days: u32,
    ) -> ExchangeResult<Vec<FireHotspot>> {
        let all_fires = self.get_fires_by_area(source, bbox, days, None).await?;

        let high_confidence_fires: Vec<FireHotspot> = all_fires
            .into_iter()
            .filter(|fire| {
                if let Some(ref conf) = fire.confidence {
                    conf.to_lowercase() == "high" || conf.to_lowercase() == "nominal"
                } else {
                    false
                }
            })
            .collect();

        Ok(high_confidence_fires)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate haversine distance between two points (in kilometers)
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth radius in kilometers

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}
