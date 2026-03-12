//! Whale Alert connector implementation
//!
//! Whale Alert is a blockchain transaction analytics provider, NOT a trading exchange.
//! Therefore, most trading-related traits will return UnsupportedOperation errors.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::*;
use crate::core::traits::*;

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// Whale Alert connector
///
/// Provides access to Whale Alert's blockchain transaction tracking service.
///
/// ## Supported Operations
/// - Status queries (supported blockchains)
/// - Transaction lookups by hash
/// - Transaction streaming by block height
/// - Address transaction history
/// - Address owner attribution
///
/// ## Unsupported Operations (returns UnsupportedOperation)
/// - Price data
/// - Orderbook
/// - Trading operations
/// - Account operations
/// - Position management
pub struct WhaleAlertConnector {
    client: Client,
    auth: WhaleAlertAuth,
    endpoints: WhaleAlertEndpoints,
    use_v1_api: bool,
}

impl WhaleAlertConnector {
    /// Create new connector with authentication
    pub fn new(auth: WhaleAlertAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: WhaleAlertEndpoints::default(),
            use_v1_api: false, // Default to Enterprise API v2
        }
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(WhaleAlertAuth::from_env())
    }

    /// Enable Developer API v1 (deprecated but functional)
    pub fn use_v1_api(mut self) -> Self {
        self.use_v1_api = true;
        self
    }

    /// Internal: Make GET request
    async fn get(
        &self,
        endpoint: WhaleAlertEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Determine base URL based on API version
        let base_url = if endpoint.is_v1() {
            self.endpoints.rest_base_v1
        } else {
            self.endpoints.rest_base
        };

        let url = format!("{}{}", base_url, endpoint.path());

        // Add authentication to query params
        self.auth.sign_query(&mut params);

        let mut request = self.client.get(&url);

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            return match status.as_u16() {
                401 => Err(ExchangeError::Auth("Invalid API key".to_string())),
                403 => Err(ExchangeError::PermissionDenied(
                    "API key lacks permissions for this endpoint".to_string()
                )),
                404 => Err(ExchangeError::NotFound("Resource not found".to_string())),
                429 => Err(ExchangeError::RateLimit),
                _ => Err(ExchangeError::Api {
                    code: status.as_u16() as i32,
                    message: error_text,
                }),
            };
        }

        response.json().await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WHALE ALERT SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get status of all supported blockchains
    pub async fn get_status(&self) -> ExchangeResult<StatusResponse> {
        let endpoint = if self.use_v1_api {
            WhaleAlertEndpoint::StatusV1
        } else {
            WhaleAlertEndpoint::Status
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        WhaleAlertParser::parse_status(&response)
    }

    /// Get blockchain status (block height range)
    pub async fn get_blockchain_status(&self, blockchain: Blockchain) -> ExchangeResult<BlockchainStatus> {
        let endpoint = WhaleAlertEndpoint::BlockchainStatus {
            blockchain: blockchain.into(),
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        WhaleAlertParser::parse_blockchain_status(&response)
    }

    /// Get single transaction by hash
    pub async fn get_transaction(
        &self,
        blockchain: Blockchain,
        hash: &str,
    ) -> ExchangeResult<WhaleTransaction> {
        let endpoint = if self.use_v1_api {
            WhaleAlertEndpoint::TransactionV1 {
                blockchain: blockchain.into(),
                hash: hash.to_string(),
            }
        } else {
            WhaleAlertEndpoint::Transaction {
                blockchain: blockchain.into(),
                hash: hash.to_string(),
            }
        };

        let response = self.get(endpoint, HashMap::new()).await?;

        if self.use_v1_api {
            // v1 API returns different format - not directly supported yet
            Err(ExchangeError::UnsupportedOperation(
                "v1 API transaction parsing not yet implemented - use v2 API".to_string()
            ))
        } else {
            WhaleAlertParser::parse_transaction(&response)
        }
    }

    /// Stream transactions from start block height
    pub async fn get_transactions(
        &self,
        blockchain: Blockchain,
        start_height: u64,
        symbol: Option<&str>,
        transaction_type: Option<TransactionType>,
        limit: Option<u16>,
    ) -> ExchangeResult<Vec<WhaleTransaction>> {
        let mut params = HashMap::new();
        params.insert("start_height".to_string(), start_height.to_string());

        if let Some(sym) = symbol {
            params.insert("symbol".to_string(), sym.to_string());
        }

        if let Some(tx_type) = transaction_type {
            params.insert("transaction_type".to_string(), tx_type.as_str().to_string());
        }

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let endpoint = WhaleAlertEndpoint::Transactions {
            blockchain: blockchain.into(),
        };

        let response = self.get(endpoint, params).await?;
        WhaleAlertParser::parse_transactions(&response)
    }

    /// Get complete block data
    pub async fn get_block(
        &self,
        blockchain: Blockchain,
        height: u64,
    ) -> ExchangeResult<WhaleBlock> {
        let endpoint = WhaleAlertEndpoint::Block {
            blockchain: blockchain.into(),
            height,
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        WhaleAlertParser::parse_block(&response)
    }

    /// Get transaction history for an address (30-day limit)
    pub async fn get_address_transactions(
        &self,
        blockchain: Blockchain,
        address: &str,
    ) -> ExchangeResult<Vec<WhaleTransaction>> {
        let endpoint = WhaleAlertEndpoint::AddressTransactions {
            blockchain: blockchain.into(),
            address: address.to_string(),
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        WhaleAlertParser::parse_address_transactions(&response)
    }

    /// Get owner attribution for an address
    pub async fn get_address_attributions(
        &self,
        blockchain: Blockchain,
        address: &str,
    ) -> ExchangeResult<Vec<OwnerAttribution>> {
        let endpoint = WhaleAlertEndpoint::AddressAttributions {
            blockchain: blockchain.into(),
            address: address.to_string(),
        };

        let response = self.get(endpoint, HashMap::new()).await?;
        WhaleAlertParser::parse_owner_attributions(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity (ALWAYS implement)
// ═══════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for WhaleAlertConnector {
    fn exchange_name(&self) -> &'static str {
        "whale_alert"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::WhaleAlert
    }

    fn is_testnet(&self) -> bool {
        false // Whale Alert has no testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Whale Alert is not a trading platform - no account types
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData (Implement minimal compatibility)
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for WhaleAlertConnector {
    /// Get current price (NOT SUPPORTED)
    async fn get_price(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<f64> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide price data - use a crypto exchange or price oracle".to_string()
        ))
    }

    /// Get ticker (NOT SUPPORTED)
    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide ticker data - use a crypto exchange".to_string()
        ))
    }

    /// Get orderbook (NOT SUPPORTED)
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide orderbook data - use a crypto exchange".to_string()
        ))
    }

    /// Get klines/candles (NOT SUPPORTED)
    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Whale Alert does not provide OHLCV data - use a crypto exchange".to_string()
        ))
    }

    /// Ping server (basic connectivity test)
    async fn ping(&self) -> ExchangeResult<()> {
        // Use status endpoint as ping
        self.get(WhaleAlertEndpoint::Status, HashMap::new()).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Trading (NOT SUPPORTED - data provider only)
// ═══════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Account (NOT SUPPORTED - data provider only)
// ═══════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════
// TRAIT: Positions (NOT SUPPORTED - data provider only)
// ═══════════════════════════════════════════════════════════════════════════


