//! UK Companies House connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UkCompaniesHouseParser, ChCompany, ChOfficer, ChPsc, ChFiling, ChSearchResult,
};

/// UK Companies House API connector
///
/// Provides access to UK company registry data including company profiles,
/// officers, beneficial ownership (PSC), and filing history.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::uk_companies_house::UkCompaniesHouseConnector;
///
/// let connector = UkCompaniesHouseConnector::from_env();
///
/// // Search for companies
/// let results = connector.search_companies("Tesla", None, None).await?;
///
/// // Get company profile
/// let company = connector.get_company("00000006").await?;
///
/// // Get beneficial owners (PSC)
/// let psc = connector.get_psc("00000006").await?;
/// ```
pub struct UkCompaniesHouseConnector {
    client: Client,
    auth: UkCompaniesHouseAuth,
    endpoints: UkCompaniesHouseEndpoints,
}

impl UkCompaniesHouseConnector {
    /// Create new connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: UkCompaniesHouseAuth::new(api_key),
            endpoints: UkCompaniesHouseEndpoints::default(),
        }
    }

    /// Create connector from environment variable (COMPANIES_HOUSE_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: UkCompaniesHouseAuth::from_env(),
            endpoints: UkCompaniesHouseEndpoints::default(),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INTERNAL REQUEST METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Internal: Make GET request to Companies House API
    async fn get(
        &self,
        endpoint: UkCompaniesHouseEndpoint,
        params: Option<HashMap<String, String>>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add Basic Auth (API key as username, empty password)
        if let Some(api_key) = self.auth.get_basic_auth() {
            request = request.basic_auth(api_key, Some(""));
        }

        // Add query parameters if provided
        if let Some(params) = params {
            request = request.query(&params);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();

            // Handle rate limiting
            if status.as_u16() == 429 {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: "Rate limit exceeded: 600 requests per 5 minutes".to_string(),
                });
            }

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}", status),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for API errors in response
        if let Some(errors) = json.get("errors") {
            if let Some(first_error) = errors.as_array().and_then(|arr| arr.first()) {
                let error_msg = first_error
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(ExchangeError::Api {
                    code: 400,
                    message: error_msg.to_string(),
                });
            }
        }

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SEARCH ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for companies by name
    ///
    /// # Arguments
    /// * `query` - Search query (company name or keywords)
    /// * `items_per_page` - Number of results per page (default: 20, max: 100)
    /// * `start_index` - Starting index for pagination (default: 0)
    ///
    /// # Example
    /// ```ignore
    /// let results = connector.search_companies("Tesla", Some(50), Some(0)).await?;
    /// println!("Found {} companies", results.total_results.unwrap_or(0));
    /// ```
    pub async fn search_companies(
        &self,
        query: &str,
        items_per_page: Option<u32>,
        start_index: Option<u32>,
    ) -> ExchangeResult<ChSearchResult> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(items) = items_per_page {
            params.insert("items_per_page".to_string(), items.to_string());
        }

        if let Some(start) = start_index {
            params.insert("start_index".to_string(), start.to_string());
        }

        let response = self.get(UkCompaniesHouseEndpoint::SearchCompanies, Some(params)).await?;
        UkCompaniesHouseParser::parse_search_results(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMPANY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get company profile by company number
    ///
    /// # Arguments
    /// * `company_number` - UK company registration number (e.g., "00000006")
    ///
    /// # Example
    /// ```ignore
    /// let company = connector.get_company("00000006").await?;
    /// println!("Company: {}", company.company_name);
    /// ```
    pub async fn get_company(&self, company_number: &str) -> ExchangeResult<ChCompany> {
        let endpoint = UkCompaniesHouseEndpoint::Company {
            company_number: company_number.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        UkCompaniesHouseParser::parse_company(&response)
    }

    /// Get company officers (directors, secretaries)
    ///
    /// # Arguments
    /// * `company_number` - UK company registration number
    ///
    /// # Example
    /// ```ignore
    /// let officers = connector.get_officers("00000006").await?;
    /// for officer in officers {
    ///     println!("{} - {}", officer.name, officer.officer_role.unwrap_or_default());
    /// }
    /// ```
    pub async fn get_officers(&self, company_number: &str) -> ExchangeResult<Vec<ChOfficer>> {
        let endpoint = UkCompaniesHouseEndpoint::CompanyOfficers {
            company_number: company_number.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        UkCompaniesHouseParser::parse_officers(&response)
    }

    /// Get persons with significant control (beneficial owners)
    ///
    /// PSC data reveals the ultimate beneficial ownership and control structure of a company.
    ///
    /// # Arguments
    /// * `company_number` - UK company registration number
    ///
    /// # Example
    /// ```ignore
    /// let psc = connector.get_psc("00000006").await?;
    /// for person in psc {
    ///     println!("Beneficial owner: {}", person.name);
    ///     println!("Control: {:?}", person.natures_of_control);
    /// }
    /// ```
    pub async fn get_psc(&self, company_number: &str) -> ExchangeResult<Vec<ChPsc>> {
        let endpoint = UkCompaniesHouseEndpoint::CompanyPsc {
            company_number: company_number.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        UkCompaniesHouseParser::parse_psc(&response)
    }

    /// Get filing history for a company
    ///
    /// # Arguments
    /// * `company_number` - UK company registration number
    ///
    /// # Example
    /// ```ignore
    /// let filings = connector.get_filing_history("00000006").await?;
    /// for filing in filings {
    ///     println!("{}: {}", filing.date.unwrap_or_default(), filing.description.unwrap_or_default());
    /// }
    /// ```
    pub async fn get_filing_history(&self, company_number: &str) -> ExchangeResult<Vec<ChFiling>> {
        let endpoint = UkCompaniesHouseEndpoint::CompanyFilingHistory {
            company_number: company_number.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        UkCompaniesHouseParser::parse_filing_history(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OFFICER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all appointments for an officer across all companies
    ///
    /// This endpoint is useful for tracking an individual across multiple companies.
    ///
    /// # Arguments
    /// * `officer_id` - Officer ID from Companies House
    ///
    /// # Example
    /// ```ignore
    /// let appointments = connector.get_officer_appointments("abc123xyz").await?;
    /// println!("Officer has {} appointments", appointments.len());
    /// ```
    pub async fn get_officer_appointments(
        &self,
        officer_id: &str,
    ) -> ExchangeResult<Vec<serde_json::Value>> {
        let endpoint = UkCompaniesHouseEndpoint::OfficerAppointments {
            officer_id: officer_id.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        UkCompaniesHouseParser::parse_officer_appointments(&response)
    }
}
