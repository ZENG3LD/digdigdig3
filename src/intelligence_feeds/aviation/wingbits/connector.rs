//! Wingbits connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::json;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    WingbitsParser, AircraftDetails, AircraftCategory,
};

/// Wingbits aircraft enrichment connector
///
/// Provides detailed aircraft information based on ICAO 24-bit addresses,
/// including ownership, registration, manufacturer details, and classification.
///
/// # Features
/// - Single aircraft lookup by ICAO24
/// - Batch aircraft lookup (up to 100 per request)
/// - Military aircraft classification
/// - Circuit breaker pattern for resilience
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::aviation::wingbits::WingbitsConnector;
///
/// let connector = WingbitsConnector::from_env();
///
/// // Get single aircraft details
/// let details = connector.get_aircraft_details("a12345").await?;
///
/// // Check if military
/// if connector.is_military(&details) {
///     println!("Military aircraft detected");
/// }
///
/// // Batch lookup
/// let icao24s = vec!["a12345", "a67890"];
/// let batch = connector.get_batch_details(&icao24s).await?;
/// ```
pub struct WingbitsConnector {
    client: Client,
    auth: WingbitsAuth,
    endpoints: WingbitsEndpoints,
}

impl WingbitsConnector {
    /// Create new Wingbits connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: WingbitsAuth::new(api_key),
            endpoints: WingbitsEndpoints::default(),
        }
    }

    /// Create new connector with custom base URL
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: WingbitsAuth::new(api_key),
            endpoints: WingbitsEndpoints::new(base_url),
        }
    }

    /// Create connector from environment variable (WINGBITS_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: WingbitsAuth::from_env(),
            endpoints: WingbitsEndpoints::default(),
        }
    }

    /// Internal: Make GET request to Wingbits API
    async fn get(
        &self,
        endpoint: WingbitsEndpoint,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Create headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Check for rate limit (429)
            if status.as_u16() == 429 {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: "Wingbits API rate limit exceeded".to_string(),
                });
            }

            // Check for not found (404)
            if status.as_u16() == 404 {
                return Err(ExchangeError::Api {
                    code: 404,
                    message: "Aircraft not found".to_string(),
                });
            }

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {} - {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for Wingbits API errors
        WingbitsParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to Wingbits API
    async fn post(
        &self,
        endpoint: WingbitsEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Create headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add body
        request = request.json(&body);

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Check for rate limit (429)
            if status.as_u16() == 429 {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: "Wingbits API rate limit exceeded".to_string(),
                });
            }

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {} - {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for Wingbits API errors
        WingbitsParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WINGBITS-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get aircraft details by ICAO 24-bit address
    ///
    /// # Arguments
    /// - `icao24` - ICAO 24-bit address in hex format (e.g., "a12345")
    ///
    /// # Returns
    /// Detailed aircraft information including registration, manufacturer,
    /// operator, owner, and category classification
    ///
    /// # Errors
    /// - `ExchangeError::Api` if aircraft not found (404)
    /// - `ExchangeError::RateLimitExceeded` if rate limit hit
    /// - `ExchangeError::Network` on connection issues
    pub async fn get_aircraft_details(&self, icao24: &str) -> ExchangeResult<AircraftDetails> {
        let response = self.get(WingbitsEndpoint::Details {
            icao24: icao24.to_lowercase()
        }).await?;

        WingbitsParser::parse_aircraft_details(&response)
    }

    /// Get batch aircraft details for multiple ICAO addresses
    ///
    /// # Arguments
    /// - `icao24s` - Slice of ICAO 24-bit addresses (max 100 recommended)
    ///
    /// # Returns
    /// Vector of aircraft details for found aircraft. Missing aircraft
    /// are silently skipped (no error).
    ///
    /// # Note
    /// Large batches (>100) may hit rate limits or timeouts. Consider
    /// splitting into smaller batches if needed.
    pub async fn get_batch_details(&self, icao24s: &[&str]) -> ExchangeResult<Vec<AircraftDetails>> {
        // Normalize to lowercase
        let normalized: Vec<String> = icao24s.iter()
            .map(|s| s.to_lowercase())
            .collect();

        let body = json!({
            "icao24": normalized
        });

        let response = self.post(WingbitsEndpoint::BatchDetails, body).await?;
        WingbitsParser::parse_batch_details(&response)
    }

    /// Check if aircraft is military based on operator/owner
    ///
    /// Uses keyword matching on operator and owner fields to identify
    /// military aircraft. Keywords include: "air force", "navy", "army",
    /// "military", "defense", etc.
    ///
    /// # Arguments
    /// - `details` - Aircraft details to check
    ///
    /// # Returns
    /// `true` if aircraft appears to be military, `false` otherwise
    pub fn is_military(&self, details: &AircraftDetails) -> bool {
        details.category == AircraftCategory::Military
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for WingbitsConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
