//! C2IntelFeeds connector implementation

use reqwest::Client;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{C2IntelFeedsParser, C2Indicator};

/// C2IntelFeeds (Command & Control Threat Intelligence) connector
///
/// Provides access to C2 IP and domain threat intelligence feeds from GitHub.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::c2intel_feeds::C2IntelFeedsConnector;
///
/// let connector = C2IntelFeedsConnector::new();
///
/// // Get 30-day IP indicators
/// let ip_indicators = connector.get_ip_feed_30day().await?;
///
/// // Get domain indicators
/// let domain_indicators = connector.get_domain_feed().await?;
///
/// // Get all indicators (IPs + domains)
/// let all_indicators = connector.get_all_indicators().await?;
/// ```
pub struct C2IntelFeedsConnector {
    client: Client,
    _auth: C2IntelFeedsAuth,
    endpoints: C2IntelFeedsEndpoints,
    _testnet: bool,
}

impl C2IntelFeedsConnector {
    /// Create new C2IntelFeeds connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: C2IntelFeedsAuth::new(),
            endpoints: C2IntelFeedsEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to GitHub and return raw text
    async fn get_text(&self, endpoint: C2IntelFeedsEndpoint) -> ExchangeResult<String> {
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

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get all-time IP C2 indicators
    ///
    /// Returns complete historical dataset of IP addresses associated with
    /// Command & Control servers.
    ///
    /// # Returns
    /// Vector of IP indicators
    pub async fn get_all_ip_indicators(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::IpFeedAll).await?;
        C2IntelFeedsParser::parse_ip_feed(&csv)
    }

    /// Get 30-day IP C2 indicators
    ///
    /// Returns IP addresses from the last 30 days.
    ///
    /// # Returns
    /// Vector of recent IP indicators
    pub async fn get_ip_feed_30day(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::IpFeed30Day).await?;
        C2IntelFeedsParser::parse_ip_feed(&csv)
    }

    /// Get 7-day IP C2 indicators
    ///
    /// Returns IP addresses from the last 7 days.
    ///
    /// # Returns
    /// Vector of very recent IP indicators
    pub async fn get_ip_feed_7day(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::IpFeed7Day).await?;
        C2IntelFeedsParser::parse_ip_feed(&csv)
    }

    /// Get 90-day IP C2 indicators
    ///
    /// Returns IP addresses from the last 90 days.
    ///
    /// # Returns
    /// Vector of 90-day IP indicators
    pub async fn get_ip_feed_90day(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::IpFeed90Day).await?;
        C2IntelFeedsParser::parse_ip_feed(&csv)
    }

    /// Get all-time domain C2 indicators
    ///
    /// Returns complete historical dataset of domains associated with
    /// Command & Control servers.
    ///
    /// # Returns
    /// Vector of domain indicators
    pub async fn get_domain_feed(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::DomainFeed).await?;
        C2IntelFeedsParser::parse_domain_feed(&csv)
    }

    /// Get 30-day domain C2 indicators
    ///
    /// Returns domains from the last 30 days.
    ///
    /// # Returns
    /// Vector of recent domain indicators
    pub async fn get_domain_feed_30day(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::DomainFeed30Day).await?;
        C2IntelFeedsParser::parse_domain_feed(&csv)
    }

    /// Get 90-day domain C2 indicators
    ///
    /// Returns domains from the last 90 days.
    ///
    /// # Returns
    /// Vector of 90-day domain indicators
    pub async fn get_domain_feed_90day(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let csv = self.get_text(C2IntelFeedsEndpoint::DomainFeed90Day).await?;
        C2IntelFeedsParser::parse_domain_feed(&csv)
    }

    /// Get all indicators (IPs + domains) from 30-day feeds
    ///
    /// Convenience method that combines both IP and domain feeds from the
    /// last 30 days into a single vector.
    ///
    /// # Returns
    /// Vector of all indicators (IPs and domains combined)
    pub async fn get_all_indicators(&self) -> ExchangeResult<Vec<C2Indicator>> {
        let mut all_indicators = Vec::new();

        // Fetch both feeds in parallel would be better, but for simplicity do sequential
        let ip_indicators = self.get_ip_feed_30day().await?;
        let domain_indicators = self.get_domain_feed_30day().await?;

        all_indicators.extend(ip_indicators);
        all_indicators.extend(domain_indicators);

        Ok(all_indicators)
    }
}

impl Default for C2IntelFeedsConnector {
    fn default() -> Self {
        Self::new()
    }
}
