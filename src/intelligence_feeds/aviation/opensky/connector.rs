//! OpenSky Network connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{OpenskyParser, StateVector, Flight, OpenskyStates, OpenskyTrack};

/// OpenSky Network connector
///
/// Provides access to real-time and historical aviation data from the OpenSky Network.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::opensky::OpenskyConnector;
///
/// // Anonymous access (10 requests per 10 seconds)
/// let connector = OpenskyConnector::anonymous();
///
/// // Authenticated access (4000 credits per day)
/// let connector = OpenskyConnector::from_env();
///
/// // Get all current aircraft state vectors
/// let states = connector.get_all_states(None, None).await?;
///
/// // Get aircraft in specific area
/// let aircraft = connector.get_aircraft_in_area(37.0, 38.0, -123.0, -122.0).await?;
///
/// // Get flights by aircraft
/// let flights = connector.get_flights_by_aircraft("abc123", 1706000000, 1706086400).await?;
/// ```
pub struct OpenskyConnector {
    client: Client,
    auth: OpenskyAuth,
    endpoints: OpenskyEndpoints,
}

impl OpenskyConnector {
    /// Create new OpenSky connector with authentication
    pub fn new(auth: OpenskyAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenskyEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENSKY_USERNAME`, `OPENSKY_PASSWORD` (optional)
    /// Falls back to anonymous access if not present
    pub fn from_env() -> Self {
        Self::new(OpenskyAuth::from_env())
    }

    /// Create connector with anonymous access
    ///
    /// Rate limit: 10 requests per 10 seconds
    pub fn anonymous() -> Self {
        Self::new(OpenskyAuth::anonymous())
    }

    /// Check if connector is using authenticated access
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }

    /// Internal: Make GET request to OpenSky API
    async fn get(
        &self,
        endpoint: OpenskyEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = reqwest::header::HeaderMap::new();
        self.auth
            .apply_auth_headers(&mut headers, &self.client)
            .await
            .map_err(|e| ExchangeError::Auth(format!("OAuth2 token error: {}", e)))?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
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
        OpenskyParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OPENSKY-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all aircraft state vectors
    ///
    /// Returns real-time position, velocity, and metadata for all aircraft.
    ///
    /// # Arguments
    /// - `time` - Optional UNIX timestamp (seconds). Retrieve data for this time. Current time if omitted.
    /// - `icao24` - Optional ICAO24 address. If provided, only return state for this aircraft.
    ///
    /// # Rate Limits
    /// - Anonymous: 10 requests per 10 seconds
    /// - Authenticated: Credits based on response size
    pub async fn get_all_states(
        &self,
        time: Option<i64>,
        icao24: Option<&str>,
    ) -> ExchangeResult<OpenskyStates> {
        let mut params = HashMap::new();

        if let Some(t) = time {
            params.insert("time".to_string(), format_timestamp(t));
        }
        if let Some(aircraft) = icao24 {
            params.insert("icao24".to_string(), format_icao24(aircraft));
        }

        let response = self.get(OpenskyEndpoint::StatesAll, params).await?;
        OpenskyParser::parse_states(&response)
    }

    /// Get state vectors from own sensors (authenticated only)
    ///
    /// Returns state vectors received by your own OpenSky Network sensors.
    ///
    /// # Arguments
    /// - `time` - Optional UNIX timestamp (seconds)
    /// - `icao24` - Optional ICAO24 address filter
    /// - `serials` - Optional array of sensor serial numbers
    ///
    /// # Authentication
    /// Requires authenticated access. Returns error for anonymous users.
    pub async fn get_own_states(
        &self,
        time: Option<i64>,
        icao24: Option<&str>,
        serials: Option<Vec<i64>>,
    ) -> ExchangeResult<OpenskyStates> {
        if !self.is_authenticated() {
            return Err(ExchangeError::Auth(
                "get_own_states requires authentication".to_string(),
            ));
        }

        let mut params = HashMap::new();

        if let Some(t) = time {
            params.insert("time".to_string(), format_timestamp(t));
        }
        if let Some(aircraft) = icao24 {
            params.insert("icao24".to_string(), format_icao24(aircraft));
        }
        if let Some(s) = serials {
            params.insert(
                "serials".to_string(),
                s.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        let response = self.get(OpenskyEndpoint::StatesOwn, params).await?;
        OpenskyParser::parse_states(&response)
    }

    /// Get all flights in a time range
    ///
    /// Returns flights with departure in the specified interval.
    ///
    /// # Arguments
    /// - `begin` - Start of time interval (UNIX timestamp in seconds)
    /// - `end` - End of time interval (UNIX timestamp in seconds)
    ///
    /// # Notes
    /// - Maximum time range: 2 hours
    /// - Credits: 4 per query
    pub async fn get_flights_all(&self, begin: i64, end: i64) -> ExchangeResult<Vec<Flight>> {
        let mut params = HashMap::new();
        params.insert("begin".to_string(), format_timestamp(begin));
        params.insert("end".to_string(), format_timestamp(end));

        let response = self.get(OpenskyEndpoint::FlightsAll, params).await?;
        OpenskyParser::parse_flights(&response)
    }

    /// Get flights by specific aircraft
    ///
    /// Returns all flights for a particular aircraft in the given time range.
    ///
    /// # Arguments
    /// - `icao24` - ICAO24 address of aircraft (hex string, e.g., "abc123")
    /// - `begin` - Start of time interval (UNIX timestamp in seconds)
    /// - `end` - End of time interval (UNIX timestamp in seconds)
    ///
    /// # Notes
    /// - Maximum time range: 30 days
    /// - Credits: 1 per query
    pub async fn get_flights_by_aircraft(
        &self,
        icao24: &str,
        begin: i64,
        end: i64,
    ) -> ExchangeResult<Vec<Flight>> {
        let mut params = HashMap::new();
        params.insert("icao24".to_string(), format_icao24(icao24));
        params.insert("begin".to_string(), format_timestamp(begin));
        params.insert("end".to_string(), format_timestamp(end));

        let response = self.get(OpenskyEndpoint::FlightsAircraft, params).await?;
        OpenskyParser::parse_flights(&response)
    }

    /// Get arrivals at an airport
    ///
    /// Returns flights arriving at the specified airport in the given time range.
    ///
    /// # Arguments
    /// - `airport` - ICAO code of airport (4 chars, e.g., "KJFK", "EDDF")
    /// - `begin` - Start of time interval (UNIX timestamp in seconds)
    /// - `end` - End of time interval (UNIX timestamp in seconds)
    ///
    /// # Notes
    /// - Maximum time range: 7 days
    /// - Credits: 2 per query
    pub async fn get_arrivals(
        &self,
        airport: &str,
        begin: i64,
        end: i64,
    ) -> ExchangeResult<Vec<Flight>> {
        let mut params = HashMap::new();
        params.insert("airport".to_string(), format_airport_icao(airport));
        params.insert("begin".to_string(), format_timestamp(begin));
        params.insert("end".to_string(), format_timestamp(end));

        let response = self.get(OpenskyEndpoint::FlightsArrival, params).await?;
        OpenskyParser::parse_flights(&response)
    }

    /// Get departures from an airport
    ///
    /// Returns flights departing from the specified airport in the given time range.
    ///
    /// # Arguments
    /// - `airport` - ICAO code of airport (4 chars, e.g., "KJFK", "EDDF")
    /// - `begin` - Start of time interval (UNIX timestamp in seconds)
    /// - `end` - End of time interval (UNIX timestamp in seconds)
    ///
    /// # Notes
    /// - Maximum time range: 7 days
    /// - Credits: 2 per query
    pub async fn get_departures(
        &self,
        airport: &str,
        begin: i64,
        end: i64,
    ) -> ExchangeResult<Vec<Flight>> {
        let mut params = HashMap::new();
        params.insert("airport".to_string(), format_airport_icao(airport));
        params.insert("begin".to_string(), format_timestamp(begin));
        params.insert("end".to_string(), format_timestamp(end));

        let response = self.get(OpenskyEndpoint::FlightsDeparture, params).await?;
        OpenskyParser::parse_flights(&response)
    }

    /// Get flight track (waypoints)
    ///
    /// Returns the waypoint trajectory for a specific flight.
    ///
    /// # Arguments
    /// - `icao24` - ICAO24 address of aircraft
    /// - `time` - UNIX timestamp (seconds) - any time during the flight
    ///
    /// # Notes
    /// - Credits: 1 per query
    pub async fn get_track(&self, icao24: &str, time: i64) -> ExchangeResult<OpenskyTrack> {
        let mut params = HashMap::new();
        params.insert("icao24".to_string(), format_icao24(icao24));
        params.insert("time".to_string(), format_timestamp(time));

        let response = self.get(OpenskyEndpoint::TracksAll, params).await?;
        OpenskyParser::parse_track(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get aircraft in a geographic bounding box
    ///
    /// Filters state vectors to only include aircraft within the specified area.
    ///
    /// # Arguments
    /// - `lat_min` - Minimum latitude (decimal degrees)
    /// - `lat_max` - Maximum latitude (decimal degrees)
    /// - `lon_min` - Minimum longitude (decimal degrees)
    /// - `lon_max` - Maximum longitude (decimal degrees)
    pub async fn get_aircraft_in_area(
        &self,
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
    ) -> ExchangeResult<Vec<StateVector>> {
        let states = self.get_all_states(None, None).await?;

        let filtered = states
            .states
            .into_iter()
            .filter(|sv| {
                if let (Some(lat), Some(lon)) = (sv.latitude, sv.longitude) {
                    lat >= lat_min && lat <= lat_max && lon >= lon_min && lon <= lon_max
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    /// Get airport traffic (combined arrivals and departures)
    ///
    /// Convenience method that fetches both arrivals and departures for an airport.
    ///
    /// # Arguments
    /// - `airport_icao` - ICAO code of airport (4 chars)
    /// - `hours_back` - How many hours back to look (max 168 for 7 days)
    pub async fn get_airport_traffic(
        &self,
        airport_icao: &str,
        hours_back: i64,
    ) -> ExchangeResult<(Vec<Flight>, Vec<Flight>)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let begin = now - (hours_back * 3600);

        let arrivals = self.get_arrivals(airport_icao, begin, now).await?;
        let departures = self.get_departures(airport_icao, begin, now).await?;

        Ok((arrivals, departures))
    }

    /// Get active aircraft count
    ///
    /// Returns the total number of aircraft currently in the air.
    pub async fn get_active_aircraft_count(&self) -> ExchangeResult<usize> {
        let states = self.get_all_states(None, None).await?;
        Ok(states.states.len())
    }
}
