//! AviationStack connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{AviationStackParser, AvFlight, AvRoute};

/// AviationStack connector
///
/// Provides access to real-time flight data and aviation databases.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::aviationstack::AviationStackConnector;
///
/// // Create from environment variable
/// let connector = AviationStackConnector::from_env();
///
/// // Get real-time flight data
/// let flights = connector.get_flights(Some("AA100"), None, None, None, None).await?;
///
/// // Get flights by route
/// let flights = connector.get_flights(None, Some("JFK"), Some("LAX"), None, None).await?;
///
/// // Get airport database
/// let airports = connector.get_airports(Some("Kennedy"), None).await?;
///
/// // Get routes
/// let routes = connector.get_routes("JFK", Some("LAX")).await?;
/// ```
pub struct AviationStackConnector {
    client: Client,
    auth: AviationStackAuth,
    endpoints: AviationStackEndpoints,
}

impl AviationStackConnector {
    /// Create new AviationStack connector with authentication
    pub fn new(auth: AviationStackAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: AviationStackEndpoints::default(),
        }
    }

    /// Create connector from environment variable
    ///
    /// Expects: `AVIATIONSTACK_API_KEY`
    pub fn from_env() -> Self {
        Self::new(AviationStackAuth::from_env())
    }

    /// Internal: Make GET request to AviationStack API
    async fn get(
        &self,
        endpoint: AviationStackEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add API key to params
        self.auth.sign_params(&mut params);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            // Check for rate limit (429)
            if status.as_u16() == 429 {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: format!("Rate limit exceeded: {}", body),
                });
            }

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, body),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors
        AviationStackParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AVIATIONSTACK-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get real-time flight data
    ///
    /// # Arguments
    /// - `flight_iata` - IATA flight code (e.g., "AA100")
    /// - `dep_iata` - Departure airport IATA code (e.g., "JFK")
    /// - `arr_iata` - Arrival airport IATA code (e.g., "LAX")
    /// - `status` - Flight status (e.g., "active", "scheduled", "landed")
    /// - `limit` - Maximum number of results (default 100)
    ///
    /// # Rate Limits
    /// - Free tier: 100 requests per month
    pub async fn get_flights(
        &self,
        flight_iata: Option<&str>,
        dep_iata: Option<&str>,
        arr_iata: Option<&str>,
        status: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<AvFlight>> {
        let mut params = HashMap::new();

        if let Some(flight) = flight_iata {
            params.insert("flight_iata".to_string(), format_iata(flight));
        }
        if let Some(dep) = dep_iata {
            params.insert("dep_iata".to_string(), format_iata(dep));
        }
        if let Some(arr) = arr_iata {
            params.insert("arr_iata".to_string(), format_iata(arr));
        }
        if let Some(s) = status {
            params.insert("flight_status".to_string(), s.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let response = self.get(AviationStackEndpoint::Flights, params).await?;
        AviationStackParser::parse_flights(&response)
    }

    /// Get airport database
    ///
    /// # Arguments
    /// - `search` - Search query (airport name or code)
    /// - `country` - Country ISO2 code (e.g., "US", "GB")
    ///
    /// # Returns
    /// Raw JSON array of airport objects
    pub async fn get_airports(
        &self,
        search: Option<&str>,
        country: Option<&str>,
    ) -> ExchangeResult<Vec<serde_json::Value>> {
        let mut params = HashMap::new();

        if let Some(s) = search {
            params.insert("search".to_string(), s.to_string());
        }
        if let Some(c) = country {
            params.insert("country_iso2".to_string(), format_country_iso2(c));
        }

        let response = self.get(AviationStackEndpoint::Airports, params).await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.clone())
    }

    /// Get airline database
    ///
    /// # Arguments
    /// - `search` - Search query (airline name or code)
    ///
    /// # Returns
    /// Raw JSON array of airline objects
    pub async fn get_airlines(
        &self,
        search: Option<&str>,
    ) -> ExchangeResult<Vec<serde_json::Value>> {
        let mut params = HashMap::new();

        if let Some(s) = search {
            params.insert("search".to_string(), s.to_string());
        }

        let response = self.get(AviationStackEndpoint::Airlines, params).await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.clone())
    }

    /// Get flight routes
    ///
    /// # Arguments
    /// - `dep_iata` - Departure airport IATA code (required, e.g., "JFK")
    /// - `arr_iata` - Arrival airport IATA code (optional, e.g., "LAX")
    ///
    /// # Returns
    /// Array of routes from departure airport
    pub async fn get_routes(
        &self,
        dep_iata: &str,
        arr_iata: Option<&str>,
    ) -> ExchangeResult<Vec<AvRoute>> {
        let mut params = HashMap::new();
        params.insert("dep_iata".to_string(), format_iata(dep_iata));

        if let Some(arr) = arr_iata {
            params.insert("arr_iata".to_string(), format_iata(arr));
        }

        let response = self.get(AviationStackEndpoint::Routes, params).await?;
        AviationStackParser::parse_routes(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get active flights (flights currently in the air)
    pub async fn get_active_flights(&self) -> ExchangeResult<Vec<AvFlight>> {
        self.get_flights(None, None, None, Some("active"), None).await
    }

    /// Get scheduled flights
    pub async fn get_scheduled_flights(&self) -> ExchangeResult<Vec<AvFlight>> {
        self.get_flights(None, None, None, Some("scheduled"), None).await
    }

    /// Get flights by airline
    ///
    /// # Arguments
    /// - `airline_iata` - Airline IATA code (e.g., "AA")
    pub async fn get_flights_by_airline(&self, airline_iata: &str) -> ExchangeResult<Vec<AvFlight>> {
        let mut params = HashMap::new();
        params.insert("airline_iata".to_string(), format_iata(airline_iata));

        let response = self.get(AviationStackEndpoint::Flights, params).await?;
        AviationStackParser::parse_flights(&response)
    }

    /// Search airports by name
    pub async fn search_airports(&self, query: &str) -> ExchangeResult<Vec<serde_json::Value>> {
        self.get_airports(Some(query), None).await
    }

    /// Search airlines by name
    pub async fn search_airlines(&self, query: &str) -> ExchangeResult<Vec<serde_json::Value>> {
        self.get_airlines(Some(query)).await
    }
}
