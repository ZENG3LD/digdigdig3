//! Bank of England connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{BoeParser, BoeObservation};

/// Bank of England (BoE) connector
///
/// Provides access to economic time series from the Bank of England.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::boe::BoeConnector;
///
/// let connector = BoeConnector::new();
///
/// // Get Bank Rate data
/// let observations = connector.get_data(&["IUDBEDR"], Some("01/Jan/2020"), Some("31/Dec/2023")).await?;
///
/// // Get multiple series at once (up to 300)
/// let multi_data = connector.get_data(&["IUDBEDR", "LPMAUZI"], Some("01/Jan/2020"), None).await?;
/// ```
pub struct BoeConnector {
    client: Client,
    auth: BoeAuth,
    endpoints: BoeEndpoints,
}

impl BoeConnector {
    /// Create new BoE connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: BoeAuth::new(),
            endpoints: BoeEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for BoE since no auth needed)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to BoE API
    async fn get(
        &self,
        endpoint: BoeEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<String> {
        // Add authentication (no-op for BoE)
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

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Text parse error: {}", e)))?;

        // Check for BoE API errors
        BoeParser::check_error(&text)?;

        Ok(text)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BOE-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get observations (data values) for one or more economic data series
    ///
    /// This is the CORE endpoint for retrieving time series data.
    ///
    /// # Arguments
    /// - `series_codes` - Array of BoE series codes (e.g., ["IUDBEDR", "LPMAUZI"]). Max 300.
    /// - `date_from` - Optional start date in BoE format: "DD/Mon/YYYY" (e.g., "01/Jan/2020")
    /// - `date_to` - Optional end date in BoE format: "DD/Mon/YYYY" (e.g., "31/Dec/2023")
    ///
    /// # Returns
    /// Vector of observations with date and value
    ///
    /// # Examples
    /// ```ignore
    /// // Get Bank Rate from 2020 to 2023
    /// let data = connector.get_data(&["IUDBEDR"], Some("01/Jan/2020"), Some("31/Dec/2023")).await?;
    ///
    /// // Get multiple series
    /// let data = connector.get_data(&["IUDBEDR", "LPMAUZI"], Some("01/Jan/2020"), None).await?;
    /// ```
    pub async fn get_data(
        &self,
        series_codes: &[&str],
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        if series_codes.is_empty() {
            return Err(ExchangeError::Parse("series_codes cannot be empty".to_string()));
        }

        if series_codes.len() > 300 {
            return Err(ExchangeError::Parse("Maximum 300 series codes allowed".to_string()));
        }

        let mut params = HashMap::new();

        // CSV export flag
        params.insert("csv.x".to_string(), "yes".to_string());

        // Series codes (comma-separated)
        params.insert("SeriesCodes".to_string(), series_codes.join(","));

        // Date range (BoE format: DD/Mon/YYYY)
        if let Some(from) = date_from {
            params.insert("Datefrom".to_string(), from.to_string());
        }
        if let Some(to) = date_to {
            params.insert("Dateto".to_string(), to.to_string());
        }

        // CSV format type
        params.insert("CSVF".to_string(), "TN".to_string());

        let response = self.get(BoeEndpoint::GetData, params).await?;
        BoeParser::parse_csv_data(&response)
    }

    /// Get observations using ISO date format (YYYY-MM-DD)
    ///
    /// This is a convenience wrapper around `get_data()` that accepts ISO dates
    /// and automatically converts them to BoE format.
    ///
    /// # Arguments
    /// - `series_codes` - Array of BoE series codes. Max 300.
    /// - `date_from` - Optional start date in ISO format: "YYYY-MM-DD"
    /// - `date_to` - Optional end date in ISO format: "YYYY-MM-DD"
    ///
    /// # Examples
    /// ```ignore
    /// let data = connector.get_data_iso(&["IUDBEDR"], Some("2020-01-01"), Some("2023-12-31")).await?;
    /// ```
    pub async fn get_data_iso(
        &self,
        series_codes: &[&str],
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        // Convert ISO dates to BoE format
        let boe_from = date_from
            .map(BoeParser::format_boe_date)
            .transpose()?;
        let boe_to = date_to
            .map(BoeParser::format_boe_date)
            .transpose()?;

        self.get_data(
            series_codes,
            boe_from.as_deref(),
            boe_to.as_deref(),
        ).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMMON SERIES CODES (CONVENIENCE METHODS)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Bank Rate (official interest rate)
    ///
    /// Series: IUDBEDR
    pub async fn get_bank_rate(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["IUDBEDR"], date_from, date_to).await
    }

    /// Get Bank Rate monthly average
    ///
    /// Series: IUMABEDR
    pub async fn get_bank_rate_monthly(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["IUMABEDR"], date_from, date_to).await
    }

    /// Get CPI annual rate (inflation)
    ///
    /// Series: LPMAUZI
    pub async fn get_cpi_annual_rate(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["LPMAUZI"], date_from, date_to).await
    }

    /// Get CPI index
    ///
    /// Series: D7BT
    pub async fn get_cpi_index(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["D7BT"], date_from, date_to).await
    }

    /// Get GDP at market prices (quarterly)
    ///
    /// Series: LPQAUYN
    pub async fn get_gdp(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["LPQAUYN"], date_from, date_to).await
    }

    /// Get GDP estimated quarterly growth rate
    ///
    /// Series: MGSX
    pub async fn get_gdp_growth(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["MGSX"], date_from, date_to).await
    }

    /// Get Money supply M4
    ///
    /// Series: A8L4
    pub async fn get_m4(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["A8L4"], date_from, date_to).await
    }

    /// Get USD/GBP exchange rate
    ///
    /// Series: XUMAUSS
    pub async fn get_usd_gbp(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["XUMAUSS"], date_from, date_to).await
    }

    /// Get GBP/USD exchange rate
    ///
    /// Series: XUDLGBD
    pub async fn get_gbp_usd(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["XUDLGBD"], date_from, date_to).await
    }

    /// Get employment rate
    ///
    /// Series: LPMB55S
    pub async fn get_employment_rate(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["LPMB55S"], date_from, date_to).await
    }

    /// Get unemployment rate
    ///
    /// Series: MGSC
    pub async fn get_unemployment_rate(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["MGSC"], date_from, date_to).await
    }

    /// Get 10-year government bond yield
    ///
    /// Series: IUMAER10
    pub async fn get_bond_yield_10y(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["IUMAER10"], date_from, date_to).await
    }

    /// Get mortgage rate (variable, 75% LTV)
    ///
    /// Series: IUMALNPY
    pub async fn get_mortgage_rate(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> ExchangeResult<Vec<BoeObservation>> {
        self.get_data(&["IUMALNPY"], date_from, date_to).await
    }
}

impl Default for BoeConnector {
    fn default() -> Self {
        Self::new()
    }
}
