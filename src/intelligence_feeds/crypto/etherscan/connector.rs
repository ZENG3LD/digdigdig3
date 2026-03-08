//! Etherscan connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    EtherscanParser, EtherscanResponse, EthBalance, EthTransaction, TokenTransfer,
    EthPrice, GasOracle, BlockReward,
};

/// Etherscan (Ethereum Blockchain Explorer) connector
///
/// Provides access to Ethereum blockchain data including:
/// - Account balances and transactions
/// - Token transfers (ERC20)
/// - Gas prices and block data
/// - Smart contract ABIs
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::etherscan::EtherscanConnector;
///
/// let connector = EtherscanConnector::from_env();
///
/// // Get ETH balance
/// let balance = connector.get_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb").await?;
///
/// // Get ETH price
/// let price = connector.get_eth_price().await?;
///
/// // Get gas prices
/// let gas = connector.get_gas_oracle().await?;
/// ```
pub struct EtherscanConnector {
    client: Client,
    auth: EtherscanAuth,
    endpoints: EtherscanEndpoints,
    _testnet: bool,
}

impl EtherscanConnector {
    /// Create new Etherscan connector with authentication
    pub fn new(auth: EtherscanAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: EtherscanEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `ETHERSCAN_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(EtherscanAuth::from_env())
    }

    /// Create testnet connector (Sepolia)
    pub fn testnet(auth: EtherscanAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: EtherscanEndpoints::testnet(),
            _testnet: true,
        }
    }

    /// Internal: Make GET request to Etherscan API
    async fn get(
        &self,
        endpoint: EtherscanEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add module and action parameters
        let (module, action) = endpoint.params();
        params.insert("module".to_string(), module.to_string());
        params.insert("action".to_string(), action.to_string());

        // Add API key authentication
        self.auth.sign_query(&mut params);

        let url = self.endpoints.rest_base;

        let response = self
            .client
            .get(url)
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

        // Check for Etherscan API errors
        EtherscanParser::check_response_generic(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request and deserialize to typed response
    async fn get_typed<T>(&self, endpoint: EtherscanEndpoint, params: HashMap<String, String>) -> ExchangeResult<EtherscanResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let json = self.get(endpoint, params).await?;

        serde_json::from_value(json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to deserialize response: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCOUNT METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get ETH balance for a single address
    ///
    /// # Arguments
    /// - `address` - Ethereum address (0x...)
    ///
    /// # Returns
    /// Balance in Wei as a string (use web3 utils to convert to ETH)
    pub async fn get_balance(&self, address: &str) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), format_address(address));
        params.insert("tag".to_string(), "latest".to_string());

        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::Balance, params).await?;
        EtherscanParser::parse_balance(&response)
    }

    /// Get ETH balances for multiple addresses
    ///
    /// # Arguments
    /// - `addresses` - Slice of Ethereum addresses
    ///
    /// # Returns
    /// Vector of balances (account + balance in Wei)
    pub async fn get_multi_balance(&self, addresses: &[&str]) -> ExchangeResult<Vec<EthBalance>> {
        let addresses_str = addresses
            .iter()
            .map(|addr| format_address(addr))
            .collect::<Vec<_>>()
            .join(",");

        let mut params = HashMap::new();
        params.insert("address".to_string(), addresses_str);
        params.insert("tag".to_string(), "latest".to_string());

        let response: EtherscanResponse<Vec<EthBalance>> = self.get_typed(EtherscanEndpoint::BalanceMulti, params).await?;
        EtherscanParser::parse_multi_balance(&response)
    }

    /// Get list of normal transactions for an address
    ///
    /// # Arguments
    /// - `address` - Ethereum address
    /// - `start_block` - Optional starting block number (0 for genesis)
    /// - `end_block` - Optional ending block number (999999999 for latest)
    /// - `page` - Optional page number (1-based)
    /// - `limit` - Optional page size (max 10000)
    ///
    /// # Returns
    /// Vector of transactions
    pub async fn get_transactions(
        &self,
        address: &str,
        start_block: Option<u64>,
        end_block: Option<u64>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<EthTransaction>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), format_address(address));
        params.insert("startblock".to_string(), start_block.unwrap_or(0).to_string());
        params.insert("endblock".to_string(), end_block.unwrap_or(99999999).to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());
        params.insert("offset".to_string(), limit.unwrap_or(10000).to_string());
        params.insert("sort".to_string(), "desc".to_string());

        let response: EtherscanResponse<Vec<EthTransaction>> = self.get_typed(EtherscanEndpoint::TxList, params).await?;
        EtherscanParser::parse_transactions(&response)
    }

    /// Get list of ERC20 token transfer events for an address
    ///
    /// # Arguments
    /// - `address` - Ethereum address
    /// - `contract_address` - Optional token contract address (filter by specific token)
    /// - `page` - Optional page number (1-based)
    /// - `limit` - Optional page size (max 10000)
    ///
    /// # Returns
    /// Vector of token transfers
    pub async fn get_token_transfers(
        &self,
        address: &str,
        contract_address: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<TokenTransfer>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), format_address(address));

        if let Some(contract) = contract_address {
            params.insert("contractaddress".to_string(), format_address(contract));
        }

        params.insert("page".to_string(), page.unwrap_or(1).to_string());
        params.insert("offset".to_string(), limit.unwrap_or(10000).to_string());
        params.insert("sort".to_string(), "desc".to_string());

        let response: EtherscanResponse<Vec<TokenTransfer>> = self.get_typed(EtherscanEndpoint::TokenTx, params).await?;
        EtherscanParser::parse_token_transfers(&response)
    }

    /// Get list of internal transactions for an address
    ///
    /// # Arguments
    /// - `address` - Ethereum address
    /// - `start_block` - Optional starting block number
    /// - `end_block` - Optional ending block number
    /// - `page` - Optional page number (1-based)
    /// - `limit` - Optional page size (max 10000)
    ///
    /// # Returns
    /// Vector of internal transactions
    pub async fn get_internal_transactions(
        &self,
        address: &str,
        start_block: Option<u64>,
        end_block: Option<u64>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<EthTransaction>> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), format_address(address));
        params.insert("startblock".to_string(), start_block.unwrap_or(0).to_string());
        params.insert("endblock".to_string(), end_block.unwrap_or(99999999).to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());
        params.insert("offset".to_string(), limit.unwrap_or(10000).to_string());
        params.insert("sort".to_string(), "desc".to_string());

        let response: EtherscanResponse<Vec<EthTransaction>> = self.get_typed(EtherscanEndpoint::TxListInternal, params).await?;
        EtherscanParser::parse_transactions(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STATS METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get current ETH price (USD + BTC)
    ///
    /// # Returns
    /// ETH price in USD and BTC with timestamps
    pub async fn get_eth_price(&self) -> ExchangeResult<EthPrice> {
        let params = HashMap::new();
        let response: EtherscanResponse<EthPrice> = self.get_typed(EtherscanEndpoint::EthPrice, params).await?;
        EtherscanParser::parse_eth_price(&response)
    }

    /// Get total ETH supply
    ///
    /// # Returns
    /// Total ETH supply in Wei as a string
    pub async fn get_eth_supply(&self) -> ExchangeResult<String> {
        let params = HashMap::new();
        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::EthSupply, params).await?;
        EtherscanParser::parse_string_result(&response)
    }

    /// Get total size of the Ethereum blockchain
    ///
    /// # Returns
    /// Chain size in bytes as a string
    pub async fn get_chain_size(&self) -> ExchangeResult<String> {
        let params = HashMap::new();
        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::ChainSize, params).await?;
        EtherscanParser::parse_string_result(&response)
    }

    /// Get total supply of an ERC20 token
    ///
    /// # Arguments
    /// - `contract_address` - Token contract address
    ///
    /// # Returns
    /// Total token supply as a string
    pub async fn get_token_supply(&self, contract_address: &str) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("contractaddress".to_string(), format_address(contract_address));

        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::TokenSupply, params).await?;
        EtherscanParser::parse_string_result(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GAS TRACKER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get current gas price oracle
    ///
    /// # Returns
    /// Gas prices (safe, propose, fast) in Gwei
    pub async fn get_gas_oracle(&self) -> ExchangeResult<GasOracle> {
        let params = HashMap::new();
        let response: EtherscanResponse<GasOracle> = self.get_typed(EtherscanEndpoint::GasOracle, params).await?;
        EtherscanParser::parse_gas_oracle(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PROXY METHODS (JSON-RPC)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get latest block number
    ///
    /// # Returns
    /// Latest block number as hex string (0x...)
    pub async fn get_latest_block_number(&self) -> ExchangeResult<String> {
        let params = HashMap::new();
        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::EthBlockNumber, params).await?;
        EtherscanParser::parse_string_result(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BLOCK METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get block mining reward
    ///
    /// # Arguments
    /// - `block_number` - Block number
    ///
    /// # Returns
    /// Block reward details including miner and reward amount
    pub async fn get_block_reward(&self, block_number: u64) -> ExchangeResult<BlockReward> {
        let mut params = HashMap::new();
        params.insert("blockno".to_string(), block_number.to_string());

        let response: EtherscanResponse<BlockReward> = self.get_typed(EtherscanEndpoint::BlockReward, params).await?;
        EtherscanParser::parse_block_reward(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONTRACT METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get smart contract ABI
    ///
    /// # Arguments
    /// - `address` - Contract address
    ///
    /// # Returns
    /// Contract ABI as JSON string
    pub async fn get_contract_abi(&self, address: &str) -> ExchangeResult<String> {
        let mut params = HashMap::new();
        params.insert("address".to_string(), format_address(address));

        let response: EtherscanResponse<String> = self.get_typed(EtherscanEndpoint::GetAbi, params).await?;
        EtherscanParser::parse_contract_abi(&response)
    }
}
