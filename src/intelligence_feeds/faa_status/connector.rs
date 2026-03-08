//! FAA Airport Status connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{FaaStatusParser, AirportDelay, AirportStatus, DelaySeverity};

/// FAA Airport Status connector
///
/// Provides real-time airport delay and status information for major US airports.
///
/// # Features
/// - Airport closures
/// - Ground stops and ground delay programs
/// - Arrival/departure delays
/// - Airspace flow programs
/// - No authentication required
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::faa_status::FaaStatusConnector;
///
/// let connector = FaaStatusConnector::new();
///
/// // Get all current delays
/// let status = connector.get_all_delays().await?;
///
/// // Get only severe delays
/// let severe = connector.get_delays_by_severity(DelaySeverity::Major).await?;
///
/// // Check if there are any delays
/// let has_delays = connector.has_delays().await?;
/// ```
pub struct FaaStatusConnector {
    client: Client,
    auth: FaaStatusAuth,
    endpoints: FaaStatusEndpoints,
}

impl FaaStatusConnector {
    /// Create new FAA Status connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: FaaStatusAuth::new(),
            endpoints: FaaStatusEndpoints::default(),
        }
    }

    /// Internal: Make GET request and return raw XML text
    async fn get_xml(&self) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        // No authentication needed for FAA
        self.auth.sign_query(&mut params);

        let url = format!(
            "{}{}",
            self.endpoints.rest_base,
            FaaStatusEndpoint::AirportStatusInfo.path()
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/xml")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let xml = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(xml)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get all current delays and airport status information
    ///
    /// Returns comprehensive information about all US airports with active delays,
    /// closures, or restrictions.
    ///
    /// # Returns
    /// Airport status with list of all delays
    ///
    /// # Example
    /// ```ignore
    /// let status = connector.get_all_delays().await?;
    /// println!("Current delays: {}", status.count);
    /// for delay in &status.delays {
    ///     println!("{}: {} - {:?}",
    ///         delay.airport_code,
    ///         delay.delay_type.as_str(),
    ///         delay.severity
    ///     );
    /// }
    /// ```
    pub async fn get_all_delays(&self) -> ExchangeResult<AirportStatus> {
        let xml = self.get_xml().await?;
        FaaStatusParser::parse_airport_status(&xml)
    }

    /// Get delays filtered by minimum severity level
    ///
    /// # Arguments
    /// - `min_severity` - Minimum severity level to include
    ///
    /// # Returns
    /// Vector of delays meeting or exceeding the severity threshold
    ///
    /// # Example
    /// ```ignore
    /// // Get only major and severe delays
    /// let major_delays = connector.get_delays_by_severity(DelaySeverity::Major).await?;
    /// ```
    pub async fn get_delays_by_severity(
        &self,
        min_severity: DelaySeverity,
    ) -> ExchangeResult<Vec<AirportDelay>> {
        let status = self.get_all_delays().await?;
        Ok(status
            .delays
            .into_iter()
            .filter(|d| d.severity >= min_severity)
            .collect())
    }

    /// Get delays for a specific airport
    ///
    /// # Arguments
    /// - `airport_code` - 3-letter IATA code (e.g., "ATL", "ORD", "LAX")
    ///
    /// # Returns
    /// Vector of delays for the specified airport (empty if no delays)
    ///
    /// # Example
    /// ```ignore
    /// let jfk_delays = connector.get_airport_delays("JFK").await?;
    /// ```
    pub async fn get_airport_delays(
        &self,
        airport_code: &str,
    ) -> ExchangeResult<Vec<AirportDelay>> {
        let status = self.get_all_delays().await?;
        let code_upper = airport_code.to_uppercase();
        Ok(status
            .delays
            .into_iter()
            .filter(|d| d.airport_code == code_upper)
            .collect())
    }

    /// Check if there are any active delays in the system
    ///
    /// # Returns
    /// `true` if there are any delays, `false` if all airports operating normally
    ///
    /// # Example
    /// ```ignore
    /// if connector.has_delays().await? {
    ///     println!("There are active delays in the NAS");
    /// } else {
    ///     println!("All airports operating normally");
    /// }
    /// ```
    pub async fn has_delays(&self) -> ExchangeResult<bool> {
        let status = self.get_all_delays().await?;
        Ok(!status.delays.is_empty())
    }

    /// Get count of delays by type
    ///
    /// Returns a breakdown of how many delays of each type are currently active.
    ///
    /// # Returns
    /// HashMap with delay type names as keys and counts as values
    ///
    /// # Example
    /// ```ignore
    /// let counts = connector.get_delay_counts().await?;
    /// for (delay_type, count) in counts {
    ///     println!("{}: {}", delay_type, count);
    /// }
    /// ```
    pub async fn get_delay_counts(&self) -> ExchangeResult<HashMap<String, usize>> {
        let status = self.get_all_delays().await?;
        let mut counts: HashMap<String, usize> = HashMap::new();

        for delay in status.delays {
            *counts.entry(delay.delay_type.as_str().to_string()).or_insert(0) += 1;
        }

        Ok(counts)
    }

    /// Get airports with ground stops
    ///
    /// Ground stops are the most severe type of delay - no departures allowed.
    ///
    /// # Returns
    /// Vector of airport codes currently under ground stop
    ///
    /// # Example
    /// ```ignore
    /// let stopped_airports = connector.get_ground_stops().await?;
    /// println!("Airports with ground stops: {:?}", stopped_airports);
    /// ```
    pub async fn get_ground_stops(&self) -> ExchangeResult<Vec<String>> {
        let status = self.get_all_delays().await?;
        Ok(status
            .delays
            .into_iter()
            .filter(|d| matches!(d.delay_type, super::parser::DelayType::GroundStop))
            .map(|d| d.airport_code)
            .collect())
    }

    /// Get airport closures
    ///
    /// # Returns
    /// Vector of airport codes currently closed
    ///
    /// # Example
    /// ```ignore
    /// let closed_airports = connector.get_closures().await?;
    /// println!("Closed airports: {:?}", closed_airports);
    /// ```
    pub async fn get_closures(&self) -> ExchangeResult<Vec<String>> {
        let status = self.get_all_delays().await?;
        Ok(status
            .delays
            .into_iter()
            .filter(|d| matches!(d.delay_type, super::parser::DelayType::Closure))
            .map(|d| d.airport_code)
            .collect())
    }

    /// Get last update timestamp
    ///
    /// # Returns
    /// Timestamp string of when the data was last updated
    ///
    /// # Example
    /// ```ignore
    /// let timestamp = connector.get_last_update().await?;
    /// println!("Data last updated: {}", timestamp);
    /// ```
    pub async fn get_last_update(&self) -> ExchangeResult<String> {
        let status = self.get_all_delays().await?;
        Ok(status.timestamp)
    }
}

impl Default for FaaStatusConnector {
    fn default() -> Self {
        Self::new()
    }
}
