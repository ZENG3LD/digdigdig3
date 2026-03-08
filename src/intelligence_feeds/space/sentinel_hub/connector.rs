//! Sentinel Hub connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::json;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{SentinelHubParser, SentinelCatalogResult, SentinelStatistical};

/// Copernicus Sentinel Hub API connector
///
/// Provides access to satellite imagery and geospatial data including:
/// - STAC catalog search for satellite imagery
/// - Statistical analysis of imagery
/// - Custom processing of satellite data
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::sentinel_hub::SentinelHubConnector;
///
/// let connector = SentinelHubConnector::from_env();
///
/// // Authenticate first
/// let token = connector.authenticate().await?;
///
/// // Search for imagery
/// let bbox = [-122.5, 37.5, -122.0, 38.0];
/// let results = connector.catalog_search(
///     "sentinel-2-l2a",
///     &bbox,
///     "2024-01-01T00:00:00Z",
///     "2024-01-07T23:59:59Z",
///     Some(10)
/// ).await?;
///
/// // Get statistics
/// let stats = connector.get_statistics(
///     "sentinel-2-l2a",
///     &bbox,
///     "2024-01-01T00:00:00Z",
///     "2024-01-07T23:59:59Z"
/// ).await?;
/// ```
pub struct SentinelHubConnector {
    client: Client,
    auth: SentinelHubAuth,
    endpoints: SentinelHubEndpoints,
}

impl SentinelHubConnector {
    /// Create new Sentinel Hub connector with authentication
    pub fn new(auth: SentinelHubAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: SentinelHubEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects:
    /// - `SENTINEL_HUB_CLIENT_ID` environment variable
    /// - `SENTINEL_HUB_CLIENT_SECRET` environment variable
    pub fn from_env() -> Self {
        Self::new(SentinelHubAuth::from_env())
    }

    /// Authenticate and obtain access token
    ///
    /// This method performs OAuth2 client credentials flow.
    /// The access token is stored in the auth object and automatically
    /// used for subsequent requests.
    ///
    /// # Returns
    /// Access token string
    pub async fn authenticate(&mut self) -> ExchangeResult<String> {
        let client_id = self.auth.client_id().ok_or_else(|| {
            ExchangeError::Auth("Missing client_id. Set SENTINEL_HUB_CLIENT_ID environment variable".to_string())
        })?;

        let client_secret = self.auth.client_secret().ok_or_else(|| {
            ExchangeError::Auth("Missing client_secret. Set SENTINEL_HUB_CLIENT_SECRET environment variable".to_string())
        })?;

        let url = format!("{}{}", self.endpoints.rest_base, SentinelHubEndpoint::Token.path());

        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");
        params.insert("client_id", client_id);
        params.insert("client_secret", client_secret);

        let response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Authentication request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("Authentication failed HTTP {}: {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        let token_val = json.get("access_token");
        let access_token = token_val
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ExchangeError::Parse("Missing 'access_token' in response".to_string()))?
            .to_string();

        self.auth.set_access_token(&access_token);

        Ok(access_token)
    }

    /// Internal: Make POST request to Sentinel Hub API
    async fn post(
        &self,
        endpoint: SentinelHubEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        if !self.auth.has_token() {
            return Err(ExchangeError::Auth(
                "Not authenticated. Call authenticate() first".to_string()
            ));
        }

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.post(&url);
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        // Check for rate limiting
        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            return Err(ExchangeError::RateLimitExceeded {
                retry_after,
                message: "Rate limit exceeded".to_string(),
            });
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CATALOG SEARCH METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search STAC catalog for satellite imagery
    ///
    /// # Arguments
    /// - `collection` - Collection ID (e.g., "sentinel-2-l2a", "sentinel-1-grd", "landsat-8-l1c")
    /// - `bbox` - Bounding box [min_lon, min_lat, max_lon, max_lat]
    /// - `datetime_from` - Start datetime (ISO 8601 format, e.g., "2024-01-01T00:00:00Z")
    /// - `datetime_to` - End datetime (ISO 8601 format)
    /// - `limit` - Maximum number of results (default: 10)
    ///
    /// # Returns
    /// Catalog search result with features and context
    pub async fn catalog_search(
        &self,
        collection: &str,
        bbox: &[f64; 4],
        datetime_from: &str,
        datetime_to: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<SentinelCatalogResult> {
        let body = json!({
            "collections": [collection],
            "bbox": bbox,
            "datetime": format!("{}/{}", datetime_from, datetime_to),
            "limit": limit.unwrap_or(10),
        });

        let response = self.post(SentinelHubEndpoint::CatalogSearch, body).await?;
        SentinelHubParser::parse_catalog_search(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATISTICAL ANALYSIS METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get statistical analysis of satellite imagery
    ///
    /// # Arguments
    /// - `collection` - Collection ID (e.g., "sentinel-2-l2a")
    /// - `bbox` - Bounding box [min_lon, min_lat, max_lon, max_lat]
    /// - `datetime_from` - Start datetime (ISO 8601 format)
    /// - `datetime_to` - End datetime (ISO 8601 format)
    ///
    /// # Returns
    /// Statistical analysis with band statistics (min, max, mean, stdev)
    pub async fn get_statistics(
        &self,
        collection: &str,
        bbox: &[f64; 4],
        datetime_from: &str,
        datetime_to: &str,
    ) -> ExchangeResult<SentinelStatistical> {
        let body = json!({
            "input": {
                "bounds": {
                    "bbox": bbox,
                    "properties": {
                        "crs": "http://www.opengis.net/def/crs/EPSG/0/4326"
                    }
                },
                "data": [{
                    "type": collection,
                    "dataFilter": {
                        "timeRange": {
                            "from": datetime_from,
                            "to": datetime_to
                        }
                    }
                }]
            },
            "aggregation": {
                "timeRange": {
                    "from": datetime_from,
                    "to": datetime_to
                },
                "aggregationInterval": {
                    "of": "P1D"
                },
                "evalscript": "//VERSION=3\nfunction setup() { return { input: [{ bands: [\"B01\", \"B02\", \"B03\", \"B04\"] }], output: [{ id: \"default\", bands: 4 }] }; }\nfunction evaluatePixel(sample) { return [sample.B01, sample.B02, sample.B03, sample.B04]; }",
                "resx": 10,
                "resy": 10
            },
            "calculations": {
                "default": {
                    "statistics": {
                        "default": {
                            "percentiles": {
                                "k": [25, 50, 75]
                            }
                        }
                    }
                }
            }
        });

        let response = self.post(SentinelHubEndpoint::Statistical, body).await?;
        SentinelHubParser::parse_statistical(&response)
    }
}
