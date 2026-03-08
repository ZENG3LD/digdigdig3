//! FBI Crime Data API connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{FbiCrimeParser, CrimeEstimate, CrimeAgency, NibrsData};

/// FBI Crime Data API (Crime Data Explorer) connector
///
/// Provides access to FBI Uniform Crime Reporting (UCR) data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::fbi_crime::FbiCrimeConnector;
///
/// let connector = FbiCrimeConnector::from_env()?;
///
/// // Get national crime estimates
/// let national = connector.get_national_estimates().await?;
///
/// // Get state-level estimates
/// let california = connector.get_state_estimates("CA").await?;
///
/// // Get agencies
/// let agencies = connector.get_agencies().await?;
/// ```
pub struct FbiCrimeConnector {
    client: Client,
    auth: FbiCrimeAuth,
    endpoints: FbiCrimeEndpoints,
    _testnet: bool,
}

impl FbiCrimeConnector {
    /// Create new FBI Crime Data connector with authentication
    pub fn new(auth: FbiCrimeAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: FbiCrimeEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `FBI_API_KEY` environment variable
    pub fn from_env() -> ExchangeResult<Self> {
        let auth = FbiCrimeAuth::from_env();
        if !auth.is_authenticated() {
            return Err(ExchangeError::Auth(
                "FBI_API_KEY environment variable must be set".to_string(),
            ));
        }
        Ok(Self::new(auth))
    }

    /// Internal: Make GET request to FBI Crime Data API
    async fn get(
        &self,
        endpoint: FbiCrimeEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication
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

        // Check for FBI Crime Data API errors
        FbiCrimeParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FBI CRIME DATA-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get national crime estimates
    ///
    /// # Returns
    /// Vector of crime estimates by year
    pub async fn get_national_estimates(&self) -> ExchangeResult<Vec<CrimeEstimate>> {
        let params = HashMap::new();
        let response = self.get(FbiCrimeEndpoint::NationalEstimates, params).await?;
        FbiCrimeParser::parse_estimates(&response)
    }

    /// Get state-level crime estimates
    ///
    /// # Arguments
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    ///
    /// # Returns
    /// Vector of crime estimates for the specified state by year
    pub async fn get_state_estimates(&self, state: &str) -> ExchangeResult<Vec<CrimeEstimate>> {
        let params = HashMap::new();
        let endpoint = FbiCrimeEndpoint::StateEstimates {
            state: state.to_uppercase(),
        };
        let response = self.get(endpoint, params).await?;
        FbiCrimeParser::parse_estimates(&response)
    }

    /// Get summarized offense data for a state
    ///
    /// # Arguments
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    /// - `offense` - Offense type (e.g., "violent-crime", "homicide", "rape",
    ///   "robbery", "aggravated-assault", "property-crime", "burglary",
    ///   "larceny", "motor-vehicle-theft", "arson")
    ///
    /// # Returns
    /// Vector of crime estimates for the specified offense in the state
    pub async fn get_summarized_offense(
        &self,
        state: &str,
        offense: &str,
    ) -> ExchangeResult<Vec<CrimeEstimate>> {
        let params = HashMap::new();
        let endpoint = FbiCrimeEndpoint::SummarizedOffense {
            state: state.to_uppercase(),
            offense: offense.to_string(),
        };
        let response = self.get(endpoint, params).await?;
        FbiCrimeParser::parse_estimates(&response)
    }

    /// Get national agency participation rates
    ///
    /// Returns data on what percentage of agencies reported crime data
    ///
    /// # Returns
    /// Vector of participation data by year
    pub async fn get_national_participation(&self) -> ExchangeResult<Vec<CrimeEstimate>> {
        let params = HashMap::new();
        let response = self.get(FbiCrimeEndpoint::NationalParticipation, params).await?;
        FbiCrimeParser::parse_estimates(&response)
    }

    /// Get list of reporting agencies
    ///
    /// # Returns
    /// Vector of crime reporting agencies
    pub async fn get_agencies(&self) -> ExchangeResult<Vec<CrimeAgency>> {
        let params = HashMap::new();
        let response = self.get(FbiCrimeEndpoint::Agencies, params).await?;
        FbiCrimeParser::parse_agencies(&response)
    }

    /// Get NIBRS offender data by offense and state
    ///
    /// # Arguments
    /// - `offense` - NIBRS offense code (e.g., "13A" for aggravated assault)
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    ///
    /// # Returns
    /// Vector of NIBRS offender count data
    pub async fn get_nibrs_offender(
        &self,
        offense: &str,
        state: &str,
    ) -> ExchangeResult<Vec<NibrsData>> {
        let params = HashMap::new();
        let endpoint = FbiCrimeEndpoint::NibrsOffender {
            offense: offense.to_string(),
            state: state.to_uppercase(),
        };
        let response = self.get(endpoint, params).await?;
        FbiCrimeParser::parse_nibrs(&response)
    }

    /// Get NIBRS victim data by offense and state
    ///
    /// # Arguments
    /// - `offense` - NIBRS offense code (e.g., "13A" for aggravated assault)
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    ///
    /// # Returns
    /// Vector of NIBRS victim count data
    pub async fn get_nibrs_victim(
        &self,
        offense: &str,
        state: &str,
    ) -> ExchangeResult<Vec<NibrsData>> {
        let params = HashMap::new();
        let endpoint = FbiCrimeEndpoint::NibrsVictim {
            offense: offense.to_string(),
            state: state.to_uppercase(),
        };
        let response = self.get(endpoint, params).await?;
        FbiCrimeParser::parse_nibrs(&response)
    }

    /// Get violent crime trends for a state
    ///
    /// Helper method to get violent crime data specifically
    ///
    /// # Arguments
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    ///
    /// # Returns
    /// Vector of violent crime estimates
    pub async fn get_violent_crime_trends(
        &self,
        state: &str,
    ) -> ExchangeResult<Vec<CrimeEstimate>> {
        self.get_summarized_offense(state, "violent-crime").await
    }

    /// Get property crime trends for a state
    ///
    /// Helper method to get property crime data specifically
    ///
    /// # Arguments
    /// - `state` - Two-letter state abbreviation (e.g., "CA", "NY", "TX")
    ///
    /// # Returns
    /// Vector of property crime estimates
    pub async fn get_property_crime_trends(
        &self,
        state: &str,
    ) -> ExchangeResult<Vec<CrimeEstimate>> {
        self.get_summarized_offense(state, "property-crime").await
    }
}
