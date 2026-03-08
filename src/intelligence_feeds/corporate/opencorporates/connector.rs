//! OpenCorporates connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{OcParser, OcCompany, OcOfficer, OcFiling, OcSearchResult};

/// OpenCorporates connector
///
/// Provides access to global corporate data including companies, officers, and filings.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::opencorporates::OpenCorporatesConnector;
///
/// let connector = OpenCorporatesConnector::from_env();
///
/// // Search companies
/// let results = connector.search_companies("Apple Inc", None, None, None).await?;
///
/// // Get specific company
/// let company = connector.get_company("us_ca", "C0806592").await?;
///
/// // Get company officers
/// let officers = connector.get_officers("us_ca", "C0806592").await?;
/// ```
pub struct OpenCorporatesConnector {
    client: Client,
    auth: OpenCorporatesAuth,
    endpoints: OpenCorporatesEndpoints,
    _testnet: bool,
}

impl OpenCorporatesConnector {
    /// Create new OpenCorporates connector with authentication
    pub fn new(auth: OpenCorporatesAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenCorporatesEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENCORPORATES_API_TOKEN` environment variable (optional)
    pub fn from_env() -> Self {
        let auth = OpenCorporatesAuth::from_env();
        Self::new(auth)
    }

    /// Create connector without authentication (free tier)
    pub fn anonymous() -> Self {
        let auth = OpenCorporatesAuth::anonymous();
        Self::new(auth)
    }

    /// Internal: Make GET request to OpenCorporates API
    async fn get(
        &self,
        endpoint: &OpenCorporatesEndpoint,
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

        // Check for rate limiting
        if response.status().as_u16() == 429 {
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

        // Check for OpenCorporates API errors
        OcParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMPANIES API
    // ═══════════════════════════════════════════════════════════════════════

    /// Search companies
    ///
    /// # Arguments
    /// - `query` - Search query (company name, number, etc.)
    /// - `jurisdiction` - Jurisdiction code (e.g., "us_ca", "gb", optional)
    /// - `status` - Current status filter: "active" or "inactive" (optional)
    /// - `page` - Page number (optional, default: 1)
    ///
    /// # Returns
    /// Search results with pagination metadata and companies
    pub async fn search_companies(
        &self,
        query: &str,
        jurisdiction: Option<&str>,
        status: Option<&str>,
        page: Option<u32>,
    ) -> ExchangeResult<OcSearchResult<OcCompany>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(j) = jurisdiction {
            params.insert("jurisdiction_code".to_string(), j.to_string());
        }

        if let Some(s) = status {
            params.insert("current_status".to_string(), s.to_string());
        }

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(&OpenCorporatesEndpoint::CompaniesSearch, params).await?;
        OcParser::parse_companies_search(&response)
    }

    /// Get specific company
    ///
    /// # Arguments
    /// - `jurisdiction` - Jurisdiction code (e.g., "us_ca", "gb")
    /// - `company_number` - Company registration number
    ///
    /// # Returns
    /// Full company details
    pub async fn get_company(
        &self,
        jurisdiction: &str,
        company_number: &str,
    ) -> ExchangeResult<OcCompany> {
        let endpoint = OpenCorporatesEndpoint::Company {
            jurisdiction: jurisdiction.to_string(),
            company_number: company_number.to_string(),
        };

        let params = HashMap::new();
        let response = self.get(&endpoint, params).await?;
        OcParser::parse_company_response(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OFFICERS API
    // ═══════════════════════════════════════════════════════════════════════

    /// Get officers for a specific company
    ///
    /// # Arguments
    /// - `jurisdiction` - Jurisdiction code (e.g., "us_ca", "gb")
    /// - `company_number` - Company registration number
    ///
    /// # Returns
    /// List of officers/directors for the company
    pub async fn get_officers(
        &self,
        jurisdiction: &str,
        company_number: &str,
    ) -> ExchangeResult<Vec<OcOfficer>> {
        let endpoint = OpenCorporatesEndpoint::CompanyOfficers {
            jurisdiction: jurisdiction.to_string(),
            company_number: company_number.to_string(),
        };

        let params = HashMap::new();
        let response = self.get(&endpoint, params).await?;
        OcParser::parse_company_officers(&response)
    }

    /// Search officers
    ///
    /// # Arguments
    /// - `query` - Search query (officer name)
    /// - `jurisdiction` - Jurisdiction code (optional)
    /// - `page` - Page number (optional, default: 1)
    ///
    /// # Returns
    /// Search results with pagination metadata and officers
    pub async fn search_officers(
        &self,
        query: &str,
        jurisdiction: Option<&str>,
        page: Option<u32>,
    ) -> ExchangeResult<OcSearchResult<OcOfficer>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(j) = jurisdiction {
            params.insert("jurisdiction_code".to_string(), j.to_string());
        }

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(&OpenCorporatesEndpoint::OfficersSearch, params).await?;
        OcParser::parse_officers_search(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FILINGS API
    // ═══════════════════════════════════════════════════════════════════════

    /// Get filings for a specific company
    ///
    /// # Arguments
    /// - `jurisdiction` - Jurisdiction code (e.g., "us_ca", "gb")
    /// - `company_number` - Company registration number
    ///
    /// # Returns
    /// List of corporate filings for the company
    pub async fn get_filings(
        &self,
        jurisdiction: &str,
        company_number: &str,
    ) -> ExchangeResult<Vec<OcFiling>> {
        let endpoint = OpenCorporatesEndpoint::CompanyFilings {
            jurisdiction: jurisdiction.to_string(),
            company_number: company_number.to_string(),
        };

        let params = HashMap::new();
        let response = self.get(&endpoint, params).await?;
        OcParser::parse_company_filings(&response)
    }
}
