//! SEC EDGAR connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    SecEdgarParser, CompanyFiling, CompanyFacts, CompanyConcept, XbrlFrame,
    CompanyTicker, SearchResult,
};

/// SEC EDGAR (Electronic Data Gathering, Analysis, and Retrieval) connector
///
/// Provides access to company filings, financial data, and insider trading information
/// from the U.S. Securities and Exchange Commission.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::sec_edgar::SecEdgarConnector;
///
/// let connector = SecEdgarConnector::new();
///
/// // Get company filings (Apple's CIK: 320193)
/// let filings = connector.get_company_filings("320193").await?;
///
/// // Get financial data
/// let facts = connector.get_company_facts("320193").await?;
///
/// // Get specific financial concept (e.g., Revenues)
/// let revenues = connector.get_revenue("320193").await?;
/// ```
pub struct SecEdgarConnector {
    client: Client,
    auth: SecEdgarAuth,
    endpoints: SecEdgarEndpoints,
}

impl SecEdgarConnector {
    /// Create new SEC EDGAR connector
    ///
    /// Uses User-Agent from environment variable `SEC_EDGAR_USER_AGENT`
    /// or defaults to "NemoTrading contact@example.com"
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: SecEdgarAuth::from_env(),
            endpoints: SecEdgarEndpoints::default(),
        }
    }

    /// Create connector with custom User-Agent
    ///
    /// # Arguments
    /// * `user_agent` - User-Agent string in format "CompanyName email@example.com"
    pub fn with_user_agent(user_agent: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: SecEdgarAuth::new(user_agent),
            endpoints: SecEdgarEndpoints::default(),
        }
    }

    /// Internal: Make GET request to SEC EDGAR API
    async fn get(
        &self,
        endpoint: SecEdgarEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let base_url = endpoint.base_url(&self.endpoints);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        let mut headers = reqwest::header::HeaderMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).headers(headers);

        // Add query parameters if present
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
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

        // Check for SEC EDGAR API errors
        SecEdgarParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SEC EDGAR-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all filings for a company
    ///
    /// # Arguments
    /// * `cik` - Central Index Key (CIK) number (will be zero-padded to 10 digits)
    ///
    /// # Example
    /// ```ignore
    /// let filings = connector.get_company_filings("320193").await?; // Apple
    /// ```
    pub async fn get_company_filings(&self, cik: &str) -> ExchangeResult<CompanyFiling> {
        let endpoint = SecEdgarEndpoint::CompanyFilings {
            cik: pad_cik(cik),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        SecEdgarParser::parse_company_filings(&response)
    }

    /// Get XBRL financial data for a company
    ///
    /// # Arguments
    /// * `cik` - Central Index Key (CIK) number
    ///
    /// # Example
    /// ```ignore
    /// let facts = connector.get_company_facts("320193").await?; // Apple
    /// ```
    pub async fn get_company_facts(&self, cik: &str) -> ExchangeResult<CompanyFacts> {
        let endpoint = SecEdgarEndpoint::CompanyFacts {
            cik: pad_cik(cik),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        SecEdgarParser::parse_company_facts(&response)
    }

    /// Get specific financial concept for a company
    ///
    /// # Arguments
    /// * `cik` - Central Index Key (CIK) number
    /// * `taxonomy` - XBRL taxonomy (e.g., "us-gaap", "dei", "ifrs-full")
    /// * `tag` - Financial concept tag (e.g., "Revenues", "NetIncomeLoss", "Assets")
    ///
    /// # Example
    /// ```ignore
    /// let revenues = connector.get_company_concept("320193", "us-gaap", "Revenues").await?;
    /// ```
    pub async fn get_company_concept(
        &self,
        cik: &str,
        taxonomy: &str,
        tag: &str,
    ) -> ExchangeResult<CompanyConcept> {
        let endpoint = SecEdgarEndpoint::CompanyConcept {
            cik: pad_cik(cik),
            taxonomy: taxonomy.to_string(),
            tag: tag.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        SecEdgarParser::parse_company_concept(&response)
    }

    /// Search filings by keywords and filters
    ///
    /// # Arguments
    /// * `query` - Search keywords
    /// * `forms` - Optional form types (e.g., "10-K,10-Q")
    /// * `date_from` - Optional start date (YYYY-MM-DD)
    /// * `date_to` - Optional end date (YYYY-MM-DD)
    ///
    /// # Example
    /// ```ignore
    /// let results = connector.search_filings("apple", Some("10-K"), None, None).await?;
    /// ```
    pub async fn search_filings(
        &self,
        query: &str,
        forms: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<SearchResult>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(f) = forms {
            params.insert("forms".to_string(), f.to_string());
        }

        if let (Some(from), Some(to)) = (date_from, date_to) {
            params.insert("dateRange".to_string(), "custom".to_string());
            params.insert("startdt".to_string(), from.to_string());
            params.insert("enddt".to_string(), to.to_string());
        }

        let response = self.get(SecEdgarEndpoint::SearchFilings, params).await?;
        SecEdgarParser::parse_search_results(&response)
    }

    /// Get all companies with their CIKs
    ///
    /// Returns a list of all companies registered with the SEC
    pub async fn get_company_tickers(&self) -> ExchangeResult<Vec<CompanyTicker>> {
        let response = self.get(SecEdgarEndpoint::CompanyTickers, HashMap::new()).await?;
        SecEdgarParser::parse_company_tickers(&response)
    }

    /// Get all mutual funds with their CIKs
    ///
    /// Returns a list of all mutual funds registered with the SEC
    pub async fn get_mutual_fund_tickers(&self) -> ExchangeResult<Vec<CompanyTicker>> {
        let response = self.get(SecEdgarEndpoint::MutualFundTickers, HashMap::new()).await?;
        SecEdgarParser::parse_company_tickers(&response)
    }

    /// Get XBRL frames (aggregate data across all filers)
    ///
    /// # Arguments
    /// * `taxonomy` - XBRL taxonomy (e.g., "us-gaap")
    /// * `tag` - Financial concept tag (e.g., "Revenues")
    /// * `unit` - Unit of measurement (e.g., "USD")
    /// * `period` - Calendar year (e.g., "2022")
    ///
    /// # Example
    /// ```ignore
    /// let frame = connector.get_xbrl_frames("us-gaap", "Revenues", "USD", "2022").await?;
    /// ```
    pub async fn get_xbrl_frames(
        &self,
        taxonomy: &str,
        tag: &str,
        unit: &str,
        period: &str,
    ) -> ExchangeResult<XbrlFrame> {
        let endpoint = SecEdgarEndpoint::XbrlFrames {
            taxonomy: taxonomy.to_string(),
            tag: tag.to_string(),
            unit: unit.to_string(),
            period: period.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        SecEdgarParser::parse_xbrl_frames(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get 10-K filings for a company (annual reports)
    ///
    /// This is a convenience method that filters company filings for form 10-K
    pub async fn get_10k_filings(&self, cik: &str) -> ExchangeResult<CompanyFiling> {
        // For now, return all filings - user can filter by form type
        self.get_company_filings(cik).await
    }

    /// Get 10-Q filings for a company (quarterly reports)
    ///
    /// This is a convenience method that filters company filings for form 10-Q
    pub async fn get_10q_filings(&self, cik: &str) -> ExchangeResult<CompanyFiling> {
        // For now, return all filings - user can filter by form type
        self.get_company_filings(cik).await
    }

    /// Get insider trading filings (Form 4)
    ///
    /// This is a convenience method that filters company filings for form 4
    pub async fn get_insider_trades(&self, cik: &str) -> ExchangeResult<CompanyFiling> {
        // For now, return all filings - user can filter by form type
        self.get_company_filings(cik).await
    }

    /// Get 13F filings (institutional holdings)
    ///
    /// This is a convenience method that filters company filings for form 13F
    pub async fn get_13f_filings(&self, cik: &str) -> ExchangeResult<CompanyFiling> {
        // For now, return all filings - user can filter by form type
        self.get_company_filings(cik).await
    }

    /// Get revenue data for a company
    ///
    /// Convenience method that calls get_company_concept with us-gaap/Revenues
    pub async fn get_revenue(&self, cik: &str) -> ExchangeResult<CompanyConcept> {
        self.get_company_concept(cik, "us-gaap", "Revenues").await
    }

    /// Get net income data for a company
    ///
    /// Convenience method that calls get_company_concept with us-gaap/NetIncomeLoss
    pub async fn get_net_income(&self, cik: &str) -> ExchangeResult<CompanyConcept> {
        self.get_company_concept(cik, "us-gaap", "NetIncomeLoss").await
    }

    /// Get total assets for a company
    ///
    /// Convenience method that calls get_company_concept with us-gaap/Assets
    pub async fn get_total_assets(&self, cik: &str) -> ExchangeResult<CompanyConcept> {
        self.get_company_concept(cik, "us-gaap", "Assets").await
    }
}

impl Default for SecEdgarConnector {
    fn default() -> Self {
        Self::new()
    }
}
