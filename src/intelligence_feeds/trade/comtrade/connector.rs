//! UN COMTRADE connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{ComtradeParser, TradeRecord, MetadataEntry};

/// UN COMTRADE (International Trade Statistics) connector
///
/// Provides access to international trade data from the United Nations.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::comtrade::ComtradeConnector;
///
/// let connector = ComtradeConnector::from_env();
///
/// // Get US imports from China in 2024
/// let trade_data = connector.get_trade_data(
///     TYPE_COMMODITIES,
///     FREQ_ANNUAL,
///     CL_HS,
///     842,  // US
///     "2024",
///     156,  // China
///     "TOTAL",
///     FLOW_IMPORT
/// ).await?;
///
/// // Get list of countries
/// let reporters = connector.get_reporters().await?;
/// ```
pub struct ComtradeConnector {
    client: Client,
    auth: ComtradeAuth,
    endpoints: ComtradeEndpoints,
}

impl ComtradeConnector {
    /// Create new COMTRADE connector with authentication
    pub fn new(auth: ComtradeAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: ComtradeEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `COMTRADE_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(ComtradeAuth::from_env())
    }

    /// Internal: Make GET request to COMTRADE API (authenticated)
    async fn get(
        &self,
        endpoint: ComtradeEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query parameters
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

        // Check for COMTRADE API errors
        ComtradeParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request to COMTRADE API (public, no auth)
    async fn get_public(
        &self,
        endpoint: ComtradeEndpoint,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
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

        ComtradeParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get trade data (authenticated)
    ///
    /// # Arguments
    /// - `type_code` - Type: "C" (commodities) or "S" (services)
    /// - `freq_code` - Frequency: "A" (annual) or "M" (monthly)
    /// - `cl_code` - Classification: "HS", "SITC", etc.
    /// - `reporter_code` - Reporting country code (e.g., 842 for US)
    /// - `period` - Time period (e.g., "2024" or "202401" for monthly)
    /// - `partner_code` - Partner country code (e.g., 156 for China)
    /// - `cmd_code` - Commodity code (e.g., "TOTAL", "27", "84")
    /// - `flow_code` - Flow: "X" (export), "M" (import), "RX" (re-export)
    #[allow(clippy::too_many_arguments)]
    pub async fn get_trade_data(
        &self,
        type_code: &str,
        freq_code: &str,
        cl_code: &str,
        reporter_code: u32,
        period: &str,
        partner_code: u32,
        cmd_code: &str,
        flow_code: &str,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        let mut params = HashMap::new();
        params.insert("reporterCode".to_string(), reporter_code.to_string());
        params.insert("period".to_string(), period.to_string());
        params.insert("partnerCode".to_string(), partner_code.to_string());
        params.insert("cmdCode".to_string(), cmd_code.to_string());
        params.insert("flowCode".to_string(), flow_code.to_string());

        let endpoint = ComtradeEndpoint::GetTradeData {
            type_code: type_code.to_string(),
            freq_code: freq_code.to_string(),
            cl_code: cl_code.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        ComtradeParser::parse_trade_data(&response)
    }

    /// Preview trade data (no authentication needed)
    ///
    /// Limited preview of trade data without requiring API key.
    ///
    /// # Arguments
    /// - `type_code` - Type: "C" (commodities) or "S" (services)
    /// - `freq_code` - Frequency: "A" (annual) or "M" (monthly)
    /// - `cl_code` - Classification: "HS", "SITC", etc.
    /// - `reporter_code` - Reporting country code
    /// - `period` - Time period
    pub async fn preview_trade_data(
        &self,
        type_code: &str,
        freq_code: &str,
        cl_code: &str,
        reporter_code: u32,
        period: &str,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        let mut params = HashMap::new();
        params.insert("reporterCode".to_string(), reporter_code.to_string());
        params.insert("period".to_string(), period.to_string());

        let endpoint = ComtradeEndpoint::PreviewTradeData {
            type_code: type_code.to_string(),
            freq_code: freq_code.to_string(),
            cl_code: cl_code.to_string(),
        };

        let response = self.get(endpoint, params).await?;
        ComtradeParser::parse_trade_data(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS (PUBLIC)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of reporter countries
    pub async fn get_reporters(&self) -> ExchangeResult<Vec<MetadataEntry>> {
        let response = self.get_public(ComtradeEndpoint::GetReporters).await?;
        ComtradeParser::parse_metadata(&response)
    }

    /// Get list of partner countries
    pub async fn get_partners(&self) -> ExchangeResult<Vec<MetadataEntry>> {
        let response = self.get_public(ComtradeEndpoint::GetPartners).await?;
        ComtradeParser::parse_metadata(&response)
    }

    /// Get commodity codes for a classification system
    ///
    /// # Arguments
    /// - `classification` - Classification system: "HS", "SITC", etc.
    pub async fn get_commodity_codes(
        &self,
        classification: &str,
    ) -> ExchangeResult<Vec<MetadataEntry>> {
        let endpoint = ComtradeEndpoint::GetCommodityCodes {
            classification: classification.to_string(),
        };
        let response = self.get_public(endpoint).await?;
        ComtradeParser::parse_metadata(&response)
    }

    /// Get flow codes (import/export/re-export)
    pub async fn get_flow_codes(&self) -> ExchangeResult<Vec<MetadataEntry>> {
        let response = self.get_public(ComtradeEndpoint::GetFlowCodes).await?;
        ComtradeParser::parse_metadata(&response)
    }

    /// Get type codes (commodities/services)
    pub async fn get_type_codes(&self) -> ExchangeResult<Vec<MetadataEntry>> {
        let response = self.get_public(ComtradeEndpoint::GetTypeCodes).await?;
        ComtradeParser::parse_metadata(&response)
    }

    /// Get frequency codes (annual/monthly)
    pub async fn get_freq_codes(&self) -> ExchangeResult<Vec<MetadataEntry>> {
        let response = self.get_public(ComtradeEndpoint::GetFreqCodes).await?;
        ComtradeParser::parse_metadata(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get exports for a reporter country
    ///
    /// # Arguments
    /// - `reporter_code` - Reporting country code (e.g., 842 for US)
    /// - `period` - Time period (e.g., "2024")
    /// - `commodity_code` - Commodity code (e.g., "TOTAL", "27")
    pub async fn get_exports(
        &self,
        reporter_code: u32,
        period: &str,
        commodity_code: &str,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        self.get_trade_data(
            TYPE_COMMODITIES,
            FREQ_ANNUAL,
            CL_HS,
            reporter_code,
            period,
            0, // 0 = World (all partners)
            commodity_code,
            FLOW_EXPORT,
        )
        .await
    }

    /// Get imports for a reporter country
    ///
    /// # Arguments
    /// - `reporter_code` - Reporting country code (e.g., 842 for US)
    /// - `period` - Time period (e.g., "2024")
    /// - `commodity_code` - Commodity code (e.g., "TOTAL", "27")
    pub async fn get_imports(
        &self,
        reporter_code: u32,
        period: &str,
        commodity_code: &str,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        self.get_trade_data(
            TYPE_COMMODITIES,
            FREQ_ANNUAL,
            CL_HS,
            reporter_code,
            period,
            0, // 0 = World (all partners)
            commodity_code,
            FLOW_IMPORT,
        )
        .await
    }

    /// Get bilateral trade between two countries
    ///
    /// # Arguments
    /// - `reporter_code` - Reporting country code
    /// - `partner_code` - Partner country code
    /// - `period` - Time period
    pub async fn get_bilateral_trade(
        &self,
        reporter_code: u32,
        partner_code: u32,
        period: &str,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        self.get_trade_data(
            TYPE_COMMODITIES,
            FREQ_ANNUAL,
            CL_HS,
            reporter_code,
            period,
            partner_code,
            "TOTAL",
            FLOW_IMPORT, // Can be either import or export
        )
        .await
    }

    /// Get top exports by commodity
    ///
    /// Returns trade data for all commodities, sorted by primary value.
    /// Caller should sort results by `primary_value` to get top exports.
    ///
    /// # Arguments
    /// - `reporter_code` - Reporting country code
    /// - `period` - Time period
    /// - `limit` - Number of top exports to return
    pub async fn get_top_exports(
        &self,
        reporter_code: u32,
        period: &str,
        limit: usize,
    ) -> ExchangeResult<Vec<TradeRecord>> {
        let mut records = self
            .get_trade_data(
                TYPE_COMMODITIES,
                FREQ_ANNUAL,
                CL_HS,
                reporter_code,
                period,
                0, // World
                "AG2", // 2-digit HS codes
                FLOW_EXPORT,
            )
            .await?;

        // Sort by primary value (descending)
        records.sort_by(|a, b| {
            let a_val = a.primary_value.unwrap_or(0.0);
            let b_val = b.primary_value.unwrap_or(0.0);
            b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top N
        records.truncate(limit);

        Ok(records)
    }
}
