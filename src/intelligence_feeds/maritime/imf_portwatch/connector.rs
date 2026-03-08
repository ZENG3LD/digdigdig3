//! IMF PortWatch connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    ImfPortWatchParser, PortWatchChokepoint, PortWatchPort, PortWatchTrafficStats,
    PortWatchDisruption,
};

/// IMF PortWatch connector
///
/// Provides access to maritime chokepoint monitoring, port statistics, and trade disruption data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::imf_portwatch::ImfPortWatchConnector;
///
/// let connector = ImfPortWatchConnector::new();
///
/// // Get all chokepoints
/// let chokepoints = connector.get_chokepoints().await?;
///
/// // Get Suez Canal traffic
/// let suez_traffic = connector.get_suez_canal_traffic().await?;
///
/// // Get active disruptions
/// let disruptions = connector.get_disruptions().await?;
/// ```
pub struct ImfPortWatchConnector {
    client: Client,
    auth: ImfPortWatchAuth,
    endpoints: ImfPortWatchEndpoints,
    _testnet: bool,
}

impl ImfPortWatchConnector {
    /// Create new IMF PortWatch connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: ImfPortWatchAuth::new(),
            endpoints: ImfPortWatchEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables (no-op for public API)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to IMF PortWatch API
    async fn get(
        &self,
        endpoint: ImfPortWatchEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
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

        ImfPortWatchParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with ID in path
    async fn get_with_id(
        &self,
        endpoint: ImfPortWatchEndpoint,
        id: &str,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path_with_id(id));

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

        ImfPortWatchParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CHOKEPOINT METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all chokepoints
    ///
    /// Returns list of all 28 global maritime chokepoints monitored by IMF.
    pub async fn get_chokepoints(&self) -> ExchangeResult<Vec<PortWatchChokepoint>> {
        let params = HashMap::new();
        let response = self.get(ImfPortWatchEndpoint::Chokepoints, params).await?;
        ImfPortWatchParser::parse_chokepoints(&response)
    }

    /// Get traffic statistics for a specific chokepoint
    ///
    /// # Arguments
    /// - `id` - Chokepoint ID
    /// - `period` - Optional time period (e.g., "2024-01", "latest")
    pub async fn get_chokepoint_stats(
        &self,
        id: &str,
        period: Option<&str>,
    ) -> ExchangeResult<PortWatchTrafficStats> {
        let mut params = HashMap::new();
        if let Some(p) = period {
            params.insert("period".to_string(), p.to_string());
        }

        let response = self
            .get_with_id(ImfPortWatchEndpoint::ChokepointStats, id, params)
            .await?;
        ImfPortWatchParser::parse_chokepoint_stats(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PORT METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of major ports
    ///
    /// # Arguments
    /// - `limit` - Optional limit on number of results
    pub async fn get_ports(&self, limit: Option<u32>) -> ExchangeResult<Vec<PortWatchPort>> {
        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(ImfPortWatchEndpoint::Ports, params).await?;
        ImfPortWatchParser::parse_ports(&response)
    }

    /// Get traffic statistics for a specific port
    ///
    /// # Arguments
    /// - `id` - Port ID
    /// - `period` - Optional time period
    pub async fn get_port_stats(
        &self,
        id: &str,
        period: Option<&str>,
    ) -> ExchangeResult<PortWatchTrafficStats> {
        let mut params = HashMap::new();
        if let Some(p) = period {
            params.insert("period".to_string(), p.to_string());
        }

        let response = self
            .get_with_id(ImfPortWatchEndpoint::PortStats, id, params)
            .await?;
        ImfPortWatchParser::parse_chokepoint_stats(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TRADE FLOW METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get global trade flow data
    ///
    /// # Arguments
    /// - `period` - Optional time period
    pub async fn get_trade_flows(&self, period: Option<&str>) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(p) = period {
            params.insert("period".to_string(), p.to_string());
        }

        self.get(ImfPortWatchEndpoint::TradeFlows, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DISRUPTION METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get active disruptions
    ///
    /// Returns list of active maritime trade disruptions affecting chokepoints.
    pub async fn get_disruptions(&self) -> ExchangeResult<Vec<PortWatchDisruption>> {
        let params = HashMap::new();
        let response = self.get(ImfPortWatchEndpoint::Disruptions, params).await?;
        ImfPortWatchParser::parse_disruptions(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS - Major Chokepoints
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Suez Canal traffic statistics
    ///
    /// Convenience method for the Suez Canal chokepoint.
    pub async fn get_suez_canal_traffic(&self) -> ExchangeResult<PortWatchTrafficStats> {
        self.get_chokepoint_stats("suez", None).await
    }

    /// Get Panama Canal traffic statistics
    ///
    /// Convenience method for the Panama Canal chokepoint.
    pub async fn get_panama_canal_traffic(&self) -> ExchangeResult<PortWatchTrafficStats> {
        self.get_chokepoint_stats("panama", None).await
    }

    /// Get Strait of Hormuz traffic statistics
    ///
    /// Convenience method for the Strait of Hormuz chokepoint.
    pub async fn get_hormuz_strait_traffic(&self) -> ExchangeResult<PortWatchTrafficStats> {
        self.get_chokepoint_stats("hormuz", None).await
    }

    /// Get global shipping index
    ///
    /// Aggregates shipping activity across all major chokepoints to provide
    /// a global shipping activity indicator.
    pub async fn get_global_shipping_index(&self) -> ExchangeResult<f64> {
        let chokepoints = self.get_chokepoints().await?;

        // Calculate aggregate index from average daily vessels
        let total_vessels: f64 = chokepoints
            .iter()
            .filter_map(|cp| cp.avg_daily_vessels)
            .sum();

        let count = chokepoints
            .iter()
            .filter(|cp| cp.avg_daily_vessels.is_some())
            .count();

        if count == 0 {
            return Err(ExchangeError::NotFound(
                "No vessel data available for global index".to_string(),
            ));
        }

        // Return average daily vessels across all chokepoints
        Ok(total_vessels / count as f64)
    }

    /// Ping (check connection)
    pub async fn ping(&self) -> ExchangeResult<()> {
        let params = HashMap::new();
        let _ = self.get(ImfPortWatchEndpoint::Chokepoints, params).await?;
        Ok(())
    }
}

impl Default for ImfPortWatchConnector {
    fn default() -> Self {
        Self::new()
    }
}
