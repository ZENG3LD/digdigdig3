//! OpenFIGI connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::json;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    OpenFigiParser, FigiResult, FigiMappingResponse, FigiSearchResponse, FigiEnumValues,
};

/// OpenFIGI (Financial Instrument Global Identifier) connector
///
/// Provides access to FIGI mapping and search APIs.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::openfigi::OpenFigiConnector;
///
/// let connector = OpenFigiConnector::from_env();
///
/// // Map ticker to FIGI
/// let results = connector.map_ticker("AAPL", Some("US")).await?;
///
/// // Search for instruments
/// let results = connector.search("Apple", Some("Common Stock"), None).await?;
///
/// // Get enum values
/// let values = connector.get_enum_values("exchCode").await?;
/// ```
pub struct OpenFigiConnector {
    client: Client,
    auth: OpenFigiAuth,
    endpoints: OpenFigiEndpoints,
    _testnet: bool,
}

impl OpenFigiConnector {
    /// Create new OpenFIGI connector with authentication
    pub fn new(auth: OpenFigiAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenFigiEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENFIGI_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(OpenFigiAuth::from_env())
    }

    /// Create connector without authentication (free tier with lower limits)
    pub fn no_auth() -> Self {
        Self::new(OpenFigiAuth::no_auth())
    }

    /// Internal: Make POST request to OpenFIGI API
    async fn post(
        &self,
        endpoint: OpenFigiEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Add API key authentication if available
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self
            .client
            .post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
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

        // Check for OpenFIGI API errors
        OpenFigiParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request to OpenFIGI API
    async fn get(
        &self,
        endpoint: OpenFigiEndpoint,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();

        // Add API key authentication if available
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self
            .client
            .get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
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

        // Check for OpenFIGI API errors
        OpenFigiParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OPENFIGI-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Map ticker to FIGI
    ///
    /// # Arguments
    /// - `ticker` - Ticker symbol (e.g., "AAPL")
    /// - `exchange_code` - Optional exchange code (e.g., "US")
    ///
    /// # Returns
    /// Vector of FIGI results (may contain multiple matches)
    pub async fn map_ticker(
        &self,
        ticker: &str,
        exchange_code: Option<&str>,
    ) -> ExchangeResult<Vec<FigiResult>> {
        let mut job = json!({
            "idType": "TICKER",
            "idValue": ticker,
        });

        if let Some(exch) = exchange_code {
            job["exchCode"] = json!(exch);
        }

        let body = json!([job]);
        let response = self.post(OpenFigiEndpoint::Mapping, body).await?;
        let mapping_response = OpenFigiParser::parse_mapping_response(&response)?;

        Ok(mapping_response.data.into_iter().flatten().collect())
    }

    /// Map ISIN to FIGI
    ///
    /// # Arguments
    /// - `isin` - ISIN code (e.g., "US0378331005")
    ///
    /// # Returns
    /// Vector of FIGI results
    pub async fn map_isin(&self, isin: &str) -> ExchangeResult<Vec<FigiResult>> {
        let job = json!({
            "idType": "ID_ISIN",
            "idValue": isin,
        });

        let body = json!([job]);
        let response = self.post(OpenFigiEndpoint::Mapping, body).await?;
        let mapping_response = OpenFigiParser::parse_mapping_response(&response)?;

        Ok(mapping_response.data.into_iter().flatten().collect())
    }

    /// Map CUSIP to FIGI
    ///
    /// # Arguments
    /// - `cusip` - CUSIP code (e.g., "037833100")
    ///
    /// # Returns
    /// Vector of FIGI results
    pub async fn map_cusip(&self, cusip: &str) -> ExchangeResult<Vec<FigiResult>> {
        let job = json!({
            "idType": "ID_CUSIP",
            "idValue": cusip,
        });

        let body = json!([job]);
        let response = self.post(OpenFigiEndpoint::Mapping, body).await?;
        let mapping_response = OpenFigiParser::parse_mapping_response(&response)?;

        Ok(mapping_response.data.into_iter().flatten().collect())
    }

    /// Map SEDOL to FIGI
    ///
    /// # Arguments
    /// - `sedol` - SEDOL code (e.g., "2046251")
    ///
    /// # Returns
    /// Vector of FIGI results
    pub async fn map_sedol(&self, sedol: &str) -> ExchangeResult<Vec<FigiResult>> {
        let job = json!({
            "idType": "ID_SEDOL",
            "idValue": sedol,
        });

        let body = json!([job]);
        let response = self.post(OpenFigiEndpoint::Mapping, body).await?;
        let mapping_response = OpenFigiParser::parse_mapping_response(&response)?;

        Ok(mapping_response.data.into_iter().flatten().collect())
    }

    /// Map batch of identifiers to FIGIs
    ///
    /// # Arguments
    /// - `jobs` - Array of mapping jobs (up to 100)
    ///
    /// Each job should be a JSON object with:
    /// - `idType`: Type of identifier (e.g., "TICKER", "ID_ISIN")
    /// - `idValue`: Value of identifier
    /// - Optional fields: `exchCode`, `micCode`, `currency`, `marketSecDes`
    ///
    /// # Returns
    /// Mapping response with array of arrays (each job returns array of results)
    pub async fn map_batch(
        &self,
        jobs: Vec<serde_json::Value>,
    ) -> ExchangeResult<FigiMappingResponse> {
        if jobs.len() > 100 {
            return Err(ExchangeError::Parse(
                "Maximum 100 jobs per request".to_string(),
            ));
        }

        let body = json!(jobs);
        let response = self.post(OpenFigiEndpoint::Mapping, body).await?;
        OpenFigiParser::parse_mapping_response(&response)
    }

    /// Search for instruments by text query
    ///
    /// # Arguments
    /// - `query` - Search query (e.g., "Apple")
    /// - `security_type` - Optional security type filter (e.g., "Common Stock")
    /// - `limit` - Optional result limit (default 10, max 100)
    ///
    /// # Returns
    /// Search response with matching instruments
    pub async fn search(
        &self,
        query: &str,
        security_type: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<FigiSearchResponse> {
        let mut body = json!({
            "query": query,
        });

        if let Some(sec_type) = security_type {
            body["securityType"] = json!(sec_type);
        }

        if let Some(lim) = limit {
            body["limit"] = json!(lim);
        }

        let response = self.post(OpenFigiEndpoint::Search, body).await?;
        OpenFigiParser::parse_search_response(&response)
    }

    /// Get valid enum values for a field
    ///
    /// # Arguments
    /// - `field` - Field name (e.g., "exchCode", "securityType", "marketSector")
    ///
    /// # Returns
    /// List of valid values for the field
    pub async fn get_enum_values(&self, field: &str) -> ExchangeResult<FigiEnumValues> {
        let endpoint = OpenFigiEndpoint::MappingValues(field.to_string());
        let response = self.get(endpoint).await?;
        OpenFigiParser::parse_enum_values(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Lookup stock ticker (convenience method)
    ///
    /// # Arguments
    /// - `ticker` - Stock ticker (e.g., "AAPL")
    ///
    /// # Returns
    /// Vector of FIGI results filtered to Common Stock
    pub async fn lookup_stock(&self, ticker: &str) -> ExchangeResult<Vec<FigiResult>> {
        let results = self.map_ticker(ticker, None).await?;

        // Filter to Common Stock only
        let stocks: Vec<FigiResult> = results
            .into_iter()
            .filter(|r| {
                r.security_type
                    .as_ref()
                    .map(|t| t.contains("Common Stock") || t.contains("Equity"))
                    .unwrap_or(false)
            })
            .collect();

        Ok(stocks)
    }

    /// Lookup bond by ISIN (convenience method)
    ///
    /// # Arguments
    /// - `isin` - Bond ISIN code
    ///
    /// # Returns
    /// Vector of FIGI results filtered to bonds
    pub async fn lookup_bond(&self, isin: &str) -> ExchangeResult<Vec<FigiResult>> {
        let results = self.map_isin(isin).await?;

        // Filter to bonds only
        let bonds: Vec<FigiResult> = results
            .into_iter()
            .filter(|r| {
                r.security_type
                    .as_ref()
                    .map(|t| t.contains("Bond") || t.contains("Note"))
                    .unwrap_or(false)
            })
            .collect();

        Ok(bonds)
    }

    /// Lookup futures contract (convenience method)
    ///
    /// # Arguments
    /// - `ticker` - Futures ticker
    /// - `exchange` - Exchange code
    ///
    /// # Returns
    /// Vector of FIGI results filtered to futures
    pub async fn lookup_futures(
        &self,
        ticker: &str,
        exchange: &str,
    ) -> ExchangeResult<Vec<FigiResult>> {
        let results = self.map_ticker(ticker, Some(exchange)).await?;

        // Filter to futures only
        let futures: Vec<FigiResult> = results
            .into_iter()
            .filter(|r| {
                r.security_type
                    .as_ref()
                    .map(|t| t.contains("Future"))
                    .unwrap_or(false)
            })
            .collect();

        Ok(futures)
    }
}
