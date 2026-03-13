//! NGA Maritime Warnings connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::Value;
use chrono::Datelike;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{NgaWarningsParser, MaritimeWarning, WarningType, WarningArea};

/// NGA Maritime Warnings connector
///
/// Provides access to maritime broadcast warnings and navigational warnings
/// from the National Geospatial-Intelligence Agency (NGA) Maritime Safety Information (MSI) API.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::maritime::nga_warnings::NgaWarningsConnector;
///
/// let connector = NgaWarningsConnector::new();
///
/// // Get all active warnings
/// let warnings = connector.get_active_warnings().await?;
///
/// // Get warnings for a specific area
/// let atlantic_warnings = connector.get_warnings_by_area("Atlantic").await?;
///
/// // Get HYDROLANT warnings
/// let hydrolant = connector.get_hydrolant_warnings().await?;
/// ```
pub struct NgaWarningsConnector {
    client: Client,
    auth: NgaWarningsAuth,
    endpoints: NgaWarningsEndpoints,
    _testnet: bool,
}

impl NgaWarningsConnector {
    /// Create new NGA Maritime Warnings connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: NgaWarningsAuth::new(),
            endpoints: NgaWarningsEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to NGA MSI API
    async fn get(&self, endpoint: NgaWarningsEndpoint, mut params: HashMap<String, String>) -> ExchangeResult<String> {
        // No authentication needed for NGA MSI
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
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get all active broadcast warnings
    ///
    /// Returns warnings with status=A (Active)
    ///
    /// # Returns
    /// Vector of active maritime warnings
    pub async fn get_active_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());
        params.insert("status".to_string(), "A".to_string());

        let json_text = self.get(NgaWarningsEndpoint::BroadcastWarnings, params).await?;

        let json: Value = serde_json::from_str(&json_text)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NgaWarningsParser::parse_broadcast_warnings(&json)
    }

    /// Get warnings by geographic area
    ///
    /// # Arguments
    /// - `area` - Geographic area (e.g., "Atlantic", "Pacific", "Mediterranean")
    ///
    /// # Returns
    /// Vector of warnings filtered by area
    pub async fn get_warnings_by_area(&self, area: &str) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        let target_area = match area.to_uppercase().as_str() {
            "ATLANTIC" => WarningArea::Atlantic,
            "PACIFIC" => WarningArea::Pacific,
            "MEDITERRANEAN" | "MED" => WarningArea::Mediterranean,
            "INDIAN" => WarningArea::Indian,
            "ARCTIC" => WarningArea::Arctic,
            "CARIBBEAN" => WarningArea::Caribbean,
            "GULF OF MEXICO" | "GULFOFMEXICO" => WarningArea::GulfOfMexico,
            "BALTIC" => WarningArea::Baltic,
            "BLACK SEA" | "BLACKSEA" => WarningArea::BlackSea,
            "RED SEA" | "REDSEA" => WarningArea::RedSea,
            "PERSIAN GULF" | "PERSIANGULF" | "ARABIAN" => WarningArea::PersianGulf,
            _ => return Ok(warnings), // Return all if area not recognized
        };

        Ok(warnings
            .into_iter()
            .filter(|w| w.area == target_area)
            .collect())
    }

    /// Get specific warning by ID
    ///
    /// # Arguments
    /// - `id` - Warning ID or reference number
    ///
    /// # Returns
    /// Single maritime warning or error if not found
    pub async fn get_warning_by_id(&self, id: &str) -> ExchangeResult<MaritimeWarning> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());

        let json_text = self.get(NgaWarningsEndpoint::WarningById { id: id.to_string() }, params).await?;

        let json: Value = serde_json::from_str(&json_text)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NgaWarningsParser::parse_warning(&json)
    }

    /// Get HYDROLANT warnings (Atlantic hydrographic warnings)
    ///
    /// # Returns
    /// Vector of HYDROLANT warnings
    pub async fn get_hydrolant_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        Ok(warnings
            .into_iter()
            .filter(|w| w.warning_type == WarningType::HYDROLANT)
            .collect())
    }

    /// Get HYDROPAC warnings (Pacific hydrographic warnings)
    ///
    /// # Returns
    /// Vector of HYDROPAC warnings
    pub async fn get_hydropac_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        Ok(warnings
            .into_iter()
            .filter(|w| w.warning_type == WarningType::HYDROPAC)
            .collect())
    }

    /// Get NAVAREA warnings (navigational area warnings)
    ///
    /// # Returns
    /// Vector of NAVAREA warnings
    pub async fn get_navarea_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        Ok(warnings
            .into_iter()
            .filter(|w| w.warning_type == WarningType::NAVAREA)
            .collect())
    }

    /// Get coastal warnings
    ///
    /// # Returns
    /// Vector of coastal warnings
    pub async fn get_coastal_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        Ok(warnings
            .into_iter()
            .filter(|w| w.warning_type == WarningType::Coastal)
            .collect())
    }

    /// Get local warnings
    ///
    /// # Returns
    /// Vector of local warnings
    pub async fn get_local_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        Ok(warnings
            .into_iter()
            .filter(|w| w.warning_type == WarningType::Local)
            .collect())
    }

    /// Get warnings issued this year
    ///
    /// # Returns
    /// Vector of warnings from current year
    pub async fn get_current_year_warnings(&self) -> ExchangeResult<Vec<MaritimeWarning>> {
        let warnings = self.get_active_warnings().await?;

        // Get current year
        let current_year = chrono::Utc::now().year() as u64;

        Ok(warnings
            .into_iter()
            .filter(|w| w.year == Some(current_year))
            .collect())
    }

    /// Get warnings by status
    ///
    /// # Arguments
    /// - `status` - Status code (A=Active, C=Cancelled, etc.)
    ///
    /// # Returns
    /// Vector of warnings with specified status
    pub async fn get_warnings_by_status(&self, status: &str) -> ExchangeResult<Vec<MaritimeWarning>> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());
        params.insert("status".to_string(), status.to_string());

        let json_text = self.get(NgaWarningsEndpoint::BroadcastWarnings, params).await?;

        let json: Value = serde_json::from_str(&json_text)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        NgaWarningsParser::parse_broadcast_warnings(&json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get ASAM anti-piracy and maritime security incident reports
    ///
    /// ASAM (Anti-Shipping Activity Messages) are incident reports for
    /// piracy, robbery, and suspicious activity at sea.
    ///
    /// # Arguments
    /// - `limit` - Optional limit on number of results
    ///
    /// # Returns
    /// ASAM incident reports as raw JSON string
    pub async fn get_asam_piracy_reports(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());
        if let Some(l) = limit {
            params.insert("maxRecords".to_string(), l.to_string());
        }

        self.get(NgaWarningsEndpoint::AsamPiracyReports, params).await
    }

    /// Get MODU (Mobile Offshore Drilling Unit) positions
    ///
    /// Returns current positions of offshore drilling platforms registered
    /// in the NGA MODU database.
    ///
    /// # Returns
    /// MODU position data as raw JSON string
    pub async fn get_modu_positions(&self) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());

        self.get(NgaWarningsEndpoint::ModuPositions, params).await
    }

    /// Get World Port Index (WPI) data
    ///
    /// The WPI contains information on approximately 3,700 ports, terminals,
    /// and offshore oil terminals worldwide.
    ///
    /// # Arguments
    /// - `country_code` - Optional 2-letter country code filter (e.g., "US", "GB")
    ///
    /// # Returns
    /// World Port Index entries as raw JSON string
    pub async fn get_world_port_index(
        &self,
        country_code: Option<&str>,
    ) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("output".to_string(), "json".to_string());
        if let Some(cc) = country_code {
            params.insert("countryCode".to_string(), cc.to_string());
        }

        self.get(NgaWarningsEndpoint::WorldPortIndex, params).await
    }
}

impl Default for NgaWarningsConnector {
    fn default() -> Self {
        Self::new()
    }
}
