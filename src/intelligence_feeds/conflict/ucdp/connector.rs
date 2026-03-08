//! UCDP connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UcdpParser, UcdpEvent, UcdpResponse, UcdpBattleDeath,
    UcdpNonStateConflict, UcdpOneSidedViolence, UcdpStateConflict,
};

/// UCDP (Uppsala Conflict Data Program) connector
///
/// Provides access to global conflict data including georeferenced events,
/// battle deaths, and various conflict types.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ucdp::UcdpConnector;
///
/// let connector = UcdpConnector::new();
///
/// // Get recent georeferenced events
/// let events = connector.get_recent_events(100).await?;
///
/// // Get conflict events for a specific year and country
/// let ukraine_2022 = connector.get_events(Some(2022), Some("Ukraine"), None, None).await?;
///
/// // Get high casualty events
/// let high_casualty = connector.get_high_casualty_events(Some(2023), 100).await?;
/// ```
pub struct UcdpConnector {
    client: Client,
    auth: UcdpAuth,
    endpoints: UcdpEndpoints,
    _testnet: bool,
}

impl UcdpConnector {
    /// Create new UCDP connector (no authentication required)
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: UcdpAuth::new(),
            endpoints: UcdpEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to UCDP API
    async fn get(
        &self,
        endpoint: UcdpEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for UCDP)
        self.auth.sign_query(&mut params);

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

        // Check for UCDP API errors
        UcdpParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UCDP-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get georeferenced events
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country` - Optional country name filter
    /// - `page` - Optional page number (1-based)
    /// - `page_size` - Optional page size (default 1000)
    ///
    /// # Returns
    /// Paginated response with georeferenced conflict events
    pub async fn get_events(
        &self,
        year: Option<u32>,
        country: Option<&str>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> ExchangeResult<UcdpResponse<UcdpEvent>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }
        if let Some(c) = country {
            params.insert("Country".to_string(), c.to_string());
        }
        if let Some(p) = page {
            params.insert("pagesize".to_string(), page_size.unwrap_or(1000).to_string());
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(UcdpEndpoint::GeoEvents, params).await?;
        UcdpParser::parse_events(&response)
    }

    /// Get battle-related deaths
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `country` - Optional country name filter
    ///
    /// # Returns
    /// Battle-related death statistics
    pub async fn get_battle_deaths(
        &self,
        year: Option<u32>,
        country: Option<&str>,
    ) -> ExchangeResult<UcdpResponse<UcdpBattleDeath>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }
        if let Some(c) = country {
            params.insert("Country".to_string(), c.to_string());
        }

        let response = self.get(UcdpEndpoint::BattleDeaths, params).await?;
        UcdpParser::parse_battle_deaths(&response)
    }

    /// Get non-state conflicts
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Non-state conflict data
    pub async fn get_nonstate_conflicts(
        &self,
        year: Option<u32>,
    ) -> ExchangeResult<UcdpResponse<UcdpNonStateConflict>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }

        let response = self.get(UcdpEndpoint::NonState, params).await?;
        UcdpParser::parse_nonstate_conflicts(&response)
    }

    /// Get one-sided violence
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// One-sided violence data
    pub async fn get_onesided_violence(
        &self,
        year: Option<u32>,
    ) -> ExchangeResult<UcdpResponse<UcdpOneSidedViolence>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }

        let response = self.get(UcdpEndpoint::OneSided, params).await?;
        UcdpParser::parse_onesided_violence(&response)
    }

    /// Get state-based conflicts
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// State-based conflict data
    pub async fn get_state_conflicts(
        &self,
        year: Option<u32>,
    ) -> ExchangeResult<UcdpResponse<UcdpStateConflict>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }

        let response = self.get(UcdpEndpoint::StateConflict, params).await?;
        UcdpParser::parse_state_conflicts(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get events in a specific region
    ///
    /// # Arguments
    /// - `region` - Region name (e.g., "Europe", "Middle East")
    /// - `year` - Optional year filter
    ///
    /// # Returns
    /// Events filtered by region
    pub async fn get_events_by_region(
        &self,
        region: &str,
        year: Option<u32>,
    ) -> ExchangeResult<UcdpResponse<UcdpEvent>> {
        let mut params = HashMap::new();
        params.insert("Region".to_string(), region.to_string());

        if let Some(y) = year {
            params.insert("Year".to_string(), y.to_string());
        }

        let response = self.get(UcdpEndpoint::GeoEvents, params).await?;
        UcdpParser::parse_events(&response)
    }

    /// Get high casualty events
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `min_deaths` - Minimum death count threshold
    ///
    /// # Returns
    /// Events with casualties >= min_deaths (filtered client-side)
    pub async fn get_high_casualty_events(
        &self,
        year: Option<u32>,
        min_deaths: u32,
    ) -> ExchangeResult<Vec<UcdpEvent>> {
        let response = self.get_events(year, None, None, None).await?;

        let filtered: Vec<UcdpEvent> = response
            .result
            .into_iter()
            .filter(|event| event.best_estimate >= min_deaths)
            .collect();

        Ok(filtered)
    }

    /// Get most recent events
    ///
    /// # Arguments
    /// - `page_size` - Number of events to retrieve
    ///
    /// # Returns
    /// Most recent georeferenced events
    pub async fn get_recent_events(
        &self,
        page_size: u32,
    ) -> ExchangeResult<UcdpResponse<UcdpEvent>> {
        let mut params = HashMap::new();
        params.insert("pagesize".to_string(), page_size.to_string());
        params.insert("page".to_string(), "1".to_string());

        let response = self.get(UcdpEndpoint::GeoEvents, params).await?;
        UcdpParser::parse_events(&response)
    }

    /// Get conflict summary for a country/year
    ///
    /// # Arguments
    /// - `country` - Country name
    /// - `year` - Year
    ///
    /// # Returns
    /// All conflict data types for the specified country and year
    pub async fn get_conflict_summary(
        &self,
        country: &str,
        year: u32,
    ) -> ExchangeResult<ConflictSummary> {
        // Fetch all data types in parallel would be ideal, but for simplicity:
        let events = self.get_events(Some(year), Some(country), None, None).await?;
        let battle_deaths = self.get_battle_deaths(Some(year), Some(country)).await?;

        Ok(ConflictSummary {
            country: country.to_string(),
            year,
            events: events.result,
            battle_deaths: battle_deaths.result,
        })
    }

    /// Get active conflicts for a year
    ///
    /// # Arguments
    /// - `year` - Year
    ///
    /// # Returns
    /// All active conflicts (state-based, non-state, one-sided)
    pub async fn get_active_conflicts(
        &self,
        year: u32,
    ) -> ExchangeResult<ActiveConflicts> {
        let state_conflicts = self.get_state_conflicts(Some(year)).await?;
        let nonstate_conflicts = self.get_nonstate_conflicts(Some(year)).await?;
        let onesided_violence = self.get_onesided_violence(Some(year)).await?;

        Ok(ActiveConflicts {
            year,
            state_conflicts: state_conflicts.result,
            nonstate_conflicts: nonstate_conflicts.result,
            onesided_violence: onesided_violence.result,
        })
    }
}

impl Default for UcdpConnector {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONVENIENCE TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Comprehensive conflict summary for a country/year
#[derive(Debug, Clone)]
pub struct ConflictSummary {
    pub country: String,
    pub year: u32,
    pub events: Vec<UcdpEvent>,
    pub battle_deaths: Vec<UcdpBattleDeath>,
}

/// All active conflicts for a year
#[derive(Debug, Clone)]
pub struct ActiveConflicts {
    pub year: u32,
    pub state_conflicts: Vec<UcdpStateConflict>,
    pub nonstate_conflicts: Vec<UcdpNonStateConflict>,
    pub onesided_violence: Vec<UcdpOneSidedViolence>,
}
