//! Cloudflare Radar connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    CloudflareRadarParser, RadarTimeSeries, RadarTopLocation, RadarTopAs,
    RadarBotSummary, RadarDeviceSummary, RadarProtocolSummary, RadarOsSummary,
    RadarBrowserSummary, RadarAttackSummary, RadarTopDomain,
};

/// Cloudflare Radar connector
///
/// Provides access to Cloudflare's internet traffic and security data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::cloudflare_radar::CloudflareRadarConnector;
///
/// let connector = CloudflareRadarConnector::from_env();
///
/// // Get HTTP traffic by location
/// let locations = connector.get_http_top_locations("7d", Some(10)).await?;
///
/// // Get bot vs human traffic
/// let bot_summary = connector.get_bot_summary("7d").await?;
///
/// // Get DDoS attack data
/// let attacks = connector.get_l3_attack_summary("7d").await?;
/// ```
pub struct CloudflareRadarConnector {
    client: Client,
    auth: CloudflareRadarAuth,
    endpoints: CloudflareRadarEndpoints,
    _testnet: bool,
}

impl CloudflareRadarConnector {
    /// Create new Cloudflare Radar connector with authentication
    pub fn new(auth: CloudflareRadarAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: CloudflareRadarEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `CLOUDFLARE_RADAR_TOKEN` environment variable
    pub fn from_env() -> Self {
        Self::new(CloudflareRadarAuth::from_env())
    }

    /// Internal: Make GET request to Cloudflare Radar API
    async fn get(
        &self,
        endpoint: CloudflareRadarEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query params
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

        // Check for Cloudflare API errors
        CloudflareRadarParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HTTP TRAFFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get top locations by HTTP traffic
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    /// - `limit` - Optional number of results (default 5)
    pub async fn get_http_top_locations(
        &self,
        date_range: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<RadarTopLocation>> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(CloudflareRadarEndpoint::HttpTopLocations, params).await?;
        CloudflareRadarParser::parse_top_locations(&response)
    }

    /// Get top ASes by traffic
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    /// - `limit` - Optional number of results (default 5)
    pub async fn get_http_top_ases(
        &self,
        date_range: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<RadarTopAs>> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(CloudflareRadarEndpoint::HttpTopAses, params).await?;
        CloudflareRadarParser::parse_top_ases(&response)
    }

    /// Get bot vs human traffic summary
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_bot_summary(&self, date_range: &str) -> ExchangeResult<RadarBotSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpSummaryBotClass, params).await?;
        CloudflareRadarParser::parse_bot_summary(&response)
    }

    /// Get device type breakdown
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_device_summary(&self, date_range: &str) -> ExchangeResult<RadarDeviceSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpSummaryDeviceType, params).await?;
        CloudflareRadarParser::parse_device_summary(&response)
    }

    /// Get HTTP protocol version distribution
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_http_protocol_summary(&self, date_range: &str) -> ExchangeResult<RadarProtocolSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpSummaryHttpProtocol, params).await?;
        CloudflareRadarParser::parse_protocol_summary(&response)
    }

    /// Get OS distribution
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_os_summary(&self, date_range: &str) -> ExchangeResult<RadarOsSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpSummaryOs, params).await?;
        CloudflareRadarParser::parse_os_summary(&response)
    }

    /// Get browser distribution
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_browser_summary(&self, date_range: &str) -> ExchangeResult<RadarBrowserSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpSummaryBrowser, params).await?;
        CloudflareRadarParser::parse_browser_summary(&response)
    }

    /// Get HTTP traffic time series
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_http_timeseries(&self, date_range: &str) -> ExchangeResult<RadarTimeSeries> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::HttpTimeseries, params).await?;
        CloudflareRadarParser::parse_timeseries(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DDOS ATTACK METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Layer 3 DDoS attack summary
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_l3_attack_summary(&self, date_range: &str) -> ExchangeResult<RadarAttackSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::AttacksLayer3Summary, params).await?;
        CloudflareRadarParser::parse_attack_summary(&response)
    }

    /// Get Layer 7 DDoS attack summary
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_l7_attack_summary(&self, date_range: &str) -> ExchangeResult<RadarAttackSummary> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::AttacksLayer7Summary, params).await?;
        CloudflareRadarParser::parse_attack_summary(&response)
    }

    /// Get Layer 3 attack time series
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    pub async fn get_l3_attack_timeseries(&self, date_range: &str) -> ExchangeResult<RadarTimeSeries> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        let response = self.get(CloudflareRadarEndpoint::AttacksLayer3Timeseries, params).await?;
        CloudflareRadarParser::parse_timeseries(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DNS METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get DNS query top locations
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    /// - `limit` - Optional number of results (default 5)
    pub async fn get_dns_top_locations(
        &self,
        date_range: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<RadarTopLocation>> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(CloudflareRadarEndpoint::DnsTopLocations, params).await?;
        CloudflareRadarParser::parse_top_locations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RANKING METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get top domains ranking
    ///
    /// # Arguments
    /// - `date_range` - Time range (1d, 7d, 14d, 28d)
    /// - `limit` - Optional number of results (default 10)
    pub async fn get_top_domains(
        &self,
        date_range: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<RadarTopDomain>> {
        let mut params = HashMap::new();
        params.insert("dateRange".to_string(), date_range.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(CloudflareRadarEndpoint::RankingTop, params).await?;
        CloudflareRadarParser::parse_top_domains(&response)
    }
}
