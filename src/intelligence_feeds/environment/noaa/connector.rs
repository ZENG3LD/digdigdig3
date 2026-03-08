//! NOAA CDO connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    NoaaParser, ClimateData, Dataset, Datatype, LocationCategory, Location, Station,
};

/// NOAA Climate Data Online (CDO) connector
///
/// Provides access to climate data from NOAA's Climate Data Online API v2.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::noaa::NoaaConnector;
///
/// let connector = NoaaConnector::from_env();
///
/// // Get temperature data for a location
/// let data = connector.get_data(
///     "GHCND",
///     Some("TMAX"),
///     Some("FIPS:37"),
///     None,
///     "2024-01-01",
///     "2024-01-31",
///     None,
///     None,
///     None,
/// ).await?;
///
/// // Get dataset list
/// let datasets = connector.list_datasets(None, None).await?;
///
/// // Get station info
/// let station = connector.get_station("GHCND:USW00094728").await?;
/// ```
pub struct NoaaConnector {
    client: Client,
    auth: NoaaAuth,
    endpoints: NoaaEndpoints,
    _testnet: bool,
}

impl NoaaConnector {
    /// Create new NOAA CDO connector with authentication
    pub fn new(auth: NoaaAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: NoaaEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `NOAA_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(NoaaAuth::from_env())
    }

    /// Internal: Make GET request to NOAA CDO API
    async fn get(
        &self,
        endpoint: NoaaEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).query(&params);

        // Add headers
        for (key, value) in headers {
            request = request.header(&key, &value);
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

        // Check for NOAA API errors
        NoaaParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request to NOAA CDO API with ID in path
    async fn get_with_id(
        &self,
        endpoint: NoaaEndpoint,
        id: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path_with_id(id));

        // Build headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).query(&params);

        // Add headers
        for (key, value) in headers {
            request = request.header(&key, &value);
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

        // Check for NOAA API errors
        NoaaParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NOAA-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get climate data observations
    ///
    /// This is the CORE endpoint for retrieving climate data.
    ///
    /// # Arguments
    /// - `dataset_id` - Dataset ID (e.g., "GHCND", "GSOM")
    /// - `datatype_id` - Optional datatype ID (e.g., "TMAX", "TMIN", "PRCP")
    /// - `location_id` - Optional location ID (e.g., "FIPS:37", "CITY:US370007")
    /// - `station_id` - Optional station ID (e.g., "GHCND:USW00094728")
    /// - `start_date` - Start date (YYYY-MM-DD)
    /// - `end_date` - End date (YYYY-MM-DD)
    /// - `units` - Optional units ("standard" or "metric")
    /// - `limit` - Optional limit (1-1000, default 25)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Returns
    /// Vector of climate data observations
    #[allow(clippy::too_many_arguments)]
    pub async fn get_data(
        &self,
        dataset_id: &str,
        datatype_id: Option<&str>,
        location_id: Option<&str>,
        station_id: Option<&str>,
        start_date: &str,
        end_date: &str,
        units: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<ClimateData>> {
        let mut params = HashMap::new();
        params.insert("datasetid".to_string(), dataset_id.to_string());
        params.insert("startdate".to_string(), start_date.to_string());
        params.insert("enddate".to_string(), end_date.to_string());

        if let Some(dt) = datatype_id {
            params.insert("datatypeid".to_string(), dt.to_string());
        }
        if let Some(loc) = location_id {
            params.insert("locationid".to_string(), loc.to_string());
        }
        if let Some(sta) = station_id {
            params.insert("stationid".to_string(), sta.to_string());
        }
        if let Some(u) = units {
            params.insert("units".to_string(), u.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(NoaaEndpoint::Data, params).await?;
        NoaaParser::parse_data(&response)
    }

    /// List all available datasets
    pub async fn list_datasets(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Dataset>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(NoaaEndpoint::Datasets, params).await?;
        NoaaParser::parse_datasets(&response)
    }

    /// Get a specific dataset by ID
    pub async fn get_dataset(&self, dataset_id: &str) -> ExchangeResult<Dataset> {
        let response = self.get_with_id(NoaaEndpoint::Dataset, dataset_id, HashMap::new()).await?;
        NoaaParser::parse_dataset(&response)
    }

    /// List available datatypes
    pub async fn list_datatypes(
        &self,
        dataset_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Datatype>> {
        let mut params = HashMap::new();
        if let Some(ds) = dataset_id {
            params.insert("datasetid".to_string(), ds.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(NoaaEndpoint::Datatypes, params).await?;
        NoaaParser::parse_datatypes(&response)
    }

    /// Get a specific datatype by ID
    pub async fn get_datatype(&self, datatype_id: &str) -> ExchangeResult<Datatype> {
        let response = self.get_with_id(NoaaEndpoint::Datatype, datatype_id, HashMap::new()).await?;
        NoaaParser::parse_datatype(&response)
    }

    /// List location categories
    pub async fn list_location_categories(
        &self,
        dataset_id: Option<&str>,
    ) -> ExchangeResult<Vec<LocationCategory>> {
        let mut params = HashMap::new();
        if let Some(ds) = dataset_id {
            params.insert("datasetid".to_string(), ds.to_string());
        }

        let response = self.get(NoaaEndpoint::LocationCategories, params).await?;
        NoaaParser::parse_location_categories(&response)
    }

    /// List locations
    pub async fn list_locations(
        &self,
        dataset_id: Option<&str>,
        location_category_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Location>> {
        let mut params = HashMap::new();
        if let Some(ds) = dataset_id {
            params.insert("datasetid".to_string(), ds.to_string());
        }
        if let Some(lc) = location_category_id {
            params.insert("locationcategoryid".to_string(), lc.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(NoaaEndpoint::Locations, params).await?;
        NoaaParser::parse_locations(&response)
    }

    /// Get a specific location by ID
    pub async fn get_location(&self, location_id: &str) -> ExchangeResult<Location> {
        let response = self.get_with_id(NoaaEndpoint::Location, location_id, HashMap::new()).await?;
        NoaaParser::parse_location(&response)
    }

    /// List weather stations
    pub async fn list_stations(
        &self,
        dataset_id: Option<&str>,
        location_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Station>> {
        let mut params = HashMap::new();
        if let Some(ds) = dataset_id {
            params.insert("datasetid".to_string(), ds.to_string());
        }
        if let Some(loc) = location_id {
            params.insert("locationid".to_string(), loc.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(NoaaEndpoint::Stations, params).await?;
        NoaaParser::parse_stations(&response)
    }

    /// Get a specific station by ID
    pub async fn get_station(&self, station_id: &str) -> ExchangeResult<Station> {
        let response = self.get_with_id(NoaaEndpoint::Station, station_id, HashMap::new()).await?;
        NoaaParser::parse_station(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get temperature data (TMAX, TMIN, TAVG) for a location
    ///
    /// Convenience method that queries GHCND dataset for temperature datatypes.
    pub async fn get_temperature(
        &self,
        location_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ClimateData>> {
        // Query for all temperature types - API will return available ones
        self.get_data(
            "GHCND",
            None, // Will get all datatypes
            Some(location_id),
            None,
            start_date,
            end_date,
            None,
            Some(1000),
            None,
        ).await.map(|data| {
            // Filter to only temperature datatypes
            data.into_iter()
                .filter(|d| d.datatype == "TMAX" || d.datatype == "TMIN" || d.datatype == "TAVG")
                .collect()
        })
    }

    /// Get precipitation data for a location
    ///
    /// Convenience method that queries GHCND dataset for precipitation.
    pub async fn get_precipitation(
        &self,
        location_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ClimateData>> {
        self.get_data(
            "GHCND",
            Some("PRCP"),
            Some(location_id),
            None,
            start_date,
            end_date,
            None,
            Some(1000),
            None,
        ).await
    }

    /// Get monthly summary data for a location
    ///
    /// Convenience method that queries GSOM (Global Summary of the Month) dataset.
    pub async fn get_monthly_summary(
        &self,
        location_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> ExchangeResult<Vec<ClimateData>> {
        self.get_data(
            "GSOM",
            None,
            Some(location_id),
            None,
            start_date,
            end_date,
            None,
            Some(1000),
            None,
        ).await
    }
}
