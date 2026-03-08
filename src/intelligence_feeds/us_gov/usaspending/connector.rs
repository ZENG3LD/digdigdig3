//! USASpending.gov connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UsaSpendingParser, UsaSpendingAward, UsaSpendingAgency, UsaSpendingState,
};

/// USASpending.gov connector
///
/// Provides access to federal spending and award data from USASpending.gov.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::usaspending::UsaSpendingConnector;
///
/// let connector = UsaSpendingConnector::new();
///
/// // Get federal agencies
/// let agencies = connector.get_agencies().await?;
///
/// // Search for awards
/// let awards = connector.search_awards("education", Some(10)).await?;
///
/// // Get state spending data
/// let states = connector.get_state_spending(None).await?;
/// ```
pub struct UsaSpendingConnector {
    client: Client,
    _auth: UsaSpendingAuth,
    endpoints: UsaSpendingEndpoints,
}

impl UsaSpendingConnector {
    /// Create new USASpending.gov connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: UsaSpendingAuth::new(),
            endpoints: UsaSpendingEndpoints::default(),
        }
    }

    /// Create connector from environment variables (no auth needed, but for API consistency)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to USASpending.gov API
    async fn get(
        &self,
        endpoint: UsaSpendingEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
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

        // Check for API errors
        UsaSpendingParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to USASpending.gov API
    async fn post(
        &self,
        endpoint: UsaSpendingEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .post(&url)
            .json(&body)
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

        // Check for API errors
        UsaSpendingParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // USASPENDING-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of federal agencies
    ///
    /// Returns information about federal agencies including budget and obligations.
    pub async fn get_agencies(&self) -> ExchangeResult<Vec<UsaSpendingAgency>> {
        let params = HashMap::new();
        let response = self.get(UsaSpendingEndpoint::Agencies, params).await?;
        UsaSpendingParser::parse_agencies(&response)
    }

    /// Search for awards by keyword
    ///
    /// # Arguments
    /// - `keyword` - Search keyword (searches in award description, recipient name, etc.)
    /// - `limit` - Optional limit on number of results (default depends on API)
    ///
    /// # Returns
    /// Vector of awards matching the search criteria
    pub async fn search_awards(
        &self,
        keyword: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<UsaSpendingAward>> {
        let mut body = serde_json::json!({
            "filters": {
                "keywords": [keyword]
            }
        });

        if let Some(lim) = limit {
            body["limit"] = serde_json::json!(lim);
        }

        let response = self.post(UsaSpendingEndpoint::AwardSearch, body).await?;
        UsaSpendingParser::parse_awards(&response)
    }

    /// Get state spending data
    ///
    /// # Arguments
    /// - `fips` - Optional FIPS code for specific state (e.g., "06" for California)
    ///   If None, returns data for all states
    ///
    /// # Returns
    /// Vector of state spending data
    pub async fn get_state_spending(
        &self,
        fips: Option<&str>,
    ) -> ExchangeResult<Vec<UsaSpendingState>> {
        let endpoint = if let Some(fips_code) = fips {
            UsaSpendingEndpoint::StateSpecificSpending {
                fips: fips_code.to_string(),
            }
        } else {
            UsaSpendingEndpoint::StateSpending
        };

        let params = HashMap::new();
        let response = self.get(endpoint, params).await?;
        UsaSpendingParser::parse_state_spending(&response)
    }
}

impl Default for UsaSpendingConnector {
    fn default() -> Self {
        Self::new()
    }
}
