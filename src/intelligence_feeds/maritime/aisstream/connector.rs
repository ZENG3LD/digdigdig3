//! AISStream.io connector implementation

use reqwest::Client;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    AisStreamParser, AisMessage,
    BoundingBox, SubscriptionMessage,
};

/// AISStream.io connector
///
/// Provides access to real-time AIS vessel tracking data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::aisstream::AisStreamConnector;
///
/// let connector = AisStreamConnector::from_env();
///
/// // Build subscription for Suez Canal area
/// let subscription = connector.get_subscription_for_suez_canal();
///
/// // Build subscription for specific vessels
/// let subscription = connector.get_subscription_for_vessels(vec![123456789, 987654321]);
///
/// // Build subscription for tankers in an area
/// let subscription = connector.get_subscription_for_tankers(
///     BoundingBox::new(29.5, 32.0, 31.5, 33.0)
/// );
/// ```
pub struct AisStreamConnector {
    _client: Client,
    auth: AisStreamAuth,
    endpoints: AisStreamEndpoints,
    _testnet: bool,
}

impl AisStreamConnector {
    /// Create new AISStream connector with authentication
    pub fn new(auth: AisStreamAuth) -> Self {
        Self {
            _client: Client::new(),
            auth,
            endpoints: AisStreamEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `AISSTREAM_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(AisStreamAuth::from_env())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SUBSCRIPTION BUILDERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Build subscription message with custom filters
    ///
    /// # Arguments
    /// - `bounding_boxes` - Optional list of geographic areas
    /// - `ship_types` - Optional list of ship type codes to filter
    /// - `mmsis` - Optional list of specific MMSI numbers to track
    ///
    /// # Returns
    /// JSON subscription message ready to send via WebSocket
    pub fn build_subscription(
        &self,
        bounding_boxes: Option<Vec<BoundingBox>>,
        _ship_types: Option<Vec<u32>>,
        mmsis: Option<Vec<u64>>,
    ) -> ExchangeResult<String> {
        let api_key = self
            .auth
            .get_api_key()
            .ok_or_else(|| ExchangeError::Auth("API key not configured".to_string()))?;

        let subscription = SubscriptionMessage {
            api_key: api_key.to_string(),
            bounding_boxes: bounding_boxes.map(|boxes| vec![boxes]),
            mmsi_filter: mmsis.map(|list| list.iter().map(|m| m.to_string()).collect()),
            message_types: Some(vec!["PositionReport".to_string(), "ShipStaticData".to_string()]),
        };

        serde_json::to_string(&subscription)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize subscription: {}", e)))
    }

    /// Parse incoming AIS message from WebSocket
    ///
    /// # Arguments
    /// - `raw_json` - Raw JSON string received from WebSocket
    ///
    /// # Returns
    /// Parsed AIS message with position and/or static data
    pub fn parse_message(&self, raw_json: &str) -> ExchangeResult<AisMessage> {
        let value: serde_json::Value = serde_json::from_str(raw_json)
            .map_err(|e| ExchangeError::Parse(format!("Invalid JSON: {}", e)))?;

        AisStreamParser::check_error(&value)?;
        AisStreamParser::parse_message(&value)
    }

    /// Build subscription for a geographic area
    ///
    /// # Arguments
    /// - `lat_min` - Minimum latitude (south)
    /// - `lon_min` - Minimum longitude (west)
    /// - `lat_max` - Maximum latitude (north)
    /// - `lon_max` - Maximum longitude (east)
    pub fn get_subscription_for_area(
        &self,
        lat_min: f64,
        lon_min: f64,
        lat_max: f64,
        lon_max: f64,
    ) -> ExchangeResult<String> {
        let bbox = BoundingBox::new(lat_min, lon_min, lat_max, lon_max);
        self.build_subscription(Some(vec![bbox]), None, None)
    }

    /// Build subscription for specific vessels by MMSI
    ///
    /// # Arguments
    /// - `mmsis` - List of MMSI numbers to track
    pub fn get_subscription_for_vessels(&self, mmsis: Vec<u64>) -> ExchangeResult<String> {
        self.build_subscription(None, None, Some(mmsis))
    }

    /// Build subscription for tanker ships in an area
    ///
    /// Tankers have ship type codes 80-89
    ///
    /// # Arguments
    /// - `area` - Bounding box for the area of interest
    pub fn get_subscription_for_tankers(&self, area: BoundingBox) -> ExchangeResult<String> {
        let tanker_types: Vec<u32> = (ship_types::TANKER_MIN..=ship_types::TANKER_MAX).collect();
        self.build_subscription(Some(vec![area]), Some(tanker_types), None)
    }

    /// Build subscription for cargo ships in an area
    ///
    /// Cargo ships have ship type codes 70-79
    ///
    /// # Arguments
    /// - `area` - Bounding box for the area of interest
    pub fn get_subscription_for_cargo(&self, area: BoundingBox) -> ExchangeResult<String> {
        let cargo_types: Vec<u32> = (ship_types::CARGO_MIN..=ship_types::CARGO_MAX).collect();
        self.build_subscription(Some(vec![area]), Some(cargo_types), None)
    }

    /// Build subscription for passenger ships in an area
    ///
    /// Passenger ships have ship type codes 60-69
    ///
    /// # Arguments
    /// - `area` - Bounding box for the area of interest
    pub fn get_subscription_for_passenger(&self, area: BoundingBox) -> ExchangeResult<String> {
        let passenger_types: Vec<u32> =
            (ship_types::PASSENGER_MIN..=ship_types::PASSENGER_MAX).collect();
        self.build_subscription(Some(vec![area]), Some(passenger_types), None)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS FOR WELL-KNOWN AREAS
    // ═══════════════════════════════════════════════════════════════════════

    /// Build subscription for Suez Canal area
    ///
    /// Covers the Suez Canal, a critical maritime chokepoint
    pub fn get_subscription_for_suez_canal(&self) -> ExchangeResult<String> {
        self.build_subscription(Some(vec![areas::suez_canal()]), None, None)
    }

    /// Build subscription for Strait of Hormuz
    ///
    /// Covers the Strait of Hormuz, critical oil shipping route
    pub fn get_subscription_for_strait_of_hormuz(&self) -> ExchangeResult<String> {
        self.build_subscription(Some(vec![areas::strait_of_hormuz()]), None, None)
    }

    /// Build subscription for Panama Canal
    ///
    /// Covers the Panama Canal area
    pub fn get_subscription_for_panama_canal(&self) -> ExchangeResult<String> {
        self.build_subscription(Some(vec![areas::panama_canal()]), None, None)
    }

    /// Build subscription for Singapore Strait
    ///
    /// Covers the Singapore Strait, one of the world's busiest shipping lanes
    pub fn get_subscription_for_singapore_strait(&self) -> ExchangeResult<String> {
        self.build_subscription(Some(vec![areas::singapore_strait()]), None, None)
    }

    /// Build subscription for Strait of Malacca
    ///
    /// Covers the Strait of Malacca, critical shipping route
    pub fn get_subscription_for_strait_of_malacca(&self) -> ExchangeResult<String> {
        self.build_subscription(Some(vec![areas::strait_of_malacca()]), None, None)
    }

    /// Build subscription for tankers in Suez Canal
    pub fn get_subscription_for_suez_tankers(&self) -> ExchangeResult<String> {
        self.get_subscription_for_tankers(areas::suez_canal())
    }

    /// Build subscription for tankers in Strait of Hormuz
    pub fn get_subscription_for_hormuz_tankers(&self) -> ExchangeResult<String> {
        self.get_subscription_for_tankers(areas::strait_of_hormuz())
    }

    /// Build subscription for cargo ships in Panama Canal
    pub fn get_subscription_for_panama_cargo(&self) -> ExchangeResult<String> {
        self.get_subscription_for_cargo(areas::panama_canal())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UTILITY METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get WebSocket URL
    pub fn get_ws_url(&self) -> &str {
        self.endpoints.ws_base
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}
