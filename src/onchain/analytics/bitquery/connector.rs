//! # Bitquery Connector
//!
//! GraphQL-based blockchain data provider connector.
//!
//! ## Important Notes
//!
//! Bitquery is a BLOCKCHAIN DATA provider, not a trading exchange:
//! - NO standard price/OHLC data (use exchanges for CEX data)
//! - NO trading operations (Trading trait returns UnsupportedOperation)
//! - NO account balances (Account trait returns UnsupportedOperation)
//! - Focus: DEX trades, token transfers, on-chain balances, smart contracts
//!
//! ## Custom Methods
//!
//! Since Bitquery uses GraphQL and doesn't fit standard REST patterns,
//! custom methods are provided as direct connector methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{
    BitqueryUrls, BitqueryDataset,
    build_dex_trades_query, build_transfers_query, build_balance_updates_query,
    build_blocks_query, build_transactions_query, build_events_query,
};
use super::auth::BitqueryAuth;
use super::parser::{
    BitqueryParser,
    DexTrade, TokenTransfer, BalanceUpdate, BlockData, TransactionData, SmartContractEvent,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitquery connector
pub struct BitqueryConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication
    auth: BitqueryAuth,
    /// URLs
    urls: BitqueryUrls,
    /// Rate limiter (10 req/min for free tier)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitqueryConnector {
    /// Create new connector
    ///
    /// # Arguments
    /// * `credentials` - OAuth credentials (token in api_key field)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let credentials = Credentials {
    ///     api_key: "ory_at_YOUR_OAUTH_TOKEN".to_string(),
    ///     api_secret: String::new(),
    ///     passphrase: None,
    /// };
    /// let connector = BitqueryConnector::new(credentials).await?;
    /// ```
    pub async fn new(credentials: Credentials) -> ExchangeResult<Self> {
        let auth = BitqueryAuth::new(&credentials)?;
        let urls = BitqueryUrls::default();
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        // Initialize rate limiter: 10 requests per 60 seconds (free tier)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(10, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Create connector with custom rate limit
    ///
    /// # Arguments
    /// * `credentials` - OAuth credentials
    /// * `rate_limit_per_min` - Rate limit (10 for free, custom for commercial)
    pub async fn new_with_rate_limit(
        credentials: Credentials,
        rate_limit_per_min: u32,
    ) -> ExchangeResult<Self> {
        let mut connector = Self::new(credentials).await?;
        connector.rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(rate_limit_per_min, Duration::from_secs(60))
        ));
        Ok(connector)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire() {
                    return;
                }
                limiter.time_until_ready()
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Execute GraphQL query
    async fn execute_query(&self, query: &str) -> ExchangeResult<Value> {
        // Wait for rate limit
        self.rate_limit_wait().await;

        let url = self.urls.graphql;

        // Add auth headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        // Build request body
        let body = json!({
            "query": query
        });

        let response = self.http.post(url, &body, &headers).await?;

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - DEX TRADES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get DEX trades
    ///
    /// # Arguments
    /// * `network` - Blockchain network (e.g., "eth", "bsc", "polygon")
    /// * `protocol` - DEX protocol (e.g., "uniswap_v2", "pancakeswap", optional)
    /// * `buy_currency` - Buy currency smart contract address (optional)
    /// * `sell_currency` - Sell currency smart contract address (optional)
    /// * `limit` - Maximum number of results
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get Uniswap V2 WETH/USDT trades on Ethereum
    /// let trades = connector.get_dex_trades(
    ///     "eth",
    ///     Some("uniswap_v2"),
    ///     Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
    ///     Some("0xdAC17F958D2ee523a2206206994597C13D831ec7"), // USDT
    ///     100,
    /// ).await?;
    /// ```
    pub async fn get_dex_trades(
        &self,
        network: &str,
        protocol: Option<&str>,
        buy_currency: Option<&str>,
        sell_currency: Option<&str>,
        limit: u32,
    ) -> ExchangeResult<Vec<DexTrade>> {
        let query = build_dex_trades_query(
            network,
            BitqueryDataset::Archive.as_str(),
            protocol,
            buy_currency,
            sell_currency,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_dex_trades(&response)
    }

    /// Get real-time DEX trades (uses realtime dataset)
    pub async fn get_realtime_dex_trades(
        &self,
        network: &str,
        protocol: Option<&str>,
        limit: u32,
    ) -> ExchangeResult<Vec<DexTrade>> {
        let query = build_dex_trades_query(
            network,
            BitqueryDataset::Realtime.as_str(),
            protocol,
            None,
            None,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_dex_trades(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - TOKEN TRANSFERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get token transfers
    ///
    /// # Arguments
    /// * `network` - Blockchain network
    /// * `currency_address` - Token contract address (optional)
    /// * `sender` - Sender address (optional)
    /// * `receiver` - Receiver address (optional)
    /// * `limit` - Maximum number of results
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get USDT transfers
    /// let transfers = connector.get_token_transfers(
    ///     "eth",
    ///     Some("0xdac17f958d2ee523a2206206994597c13d831ec7"), // USDT
    ///     None,
    ///     None,
    ///     100,
    /// ).await?;
    /// ```
    pub async fn get_token_transfers(
        &self,
        network: &str,
        currency_address: Option<&str>,
        sender: Option<&str>,
        receiver: Option<&str>,
        limit: u32,
    ) -> ExchangeResult<Vec<TokenTransfer>> {
        let query = build_transfers_query(
            network,
            BitqueryDataset::Archive.as_str(),
            currency_address,
            sender,
            receiver,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_transfers(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - BALANCE UPDATES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get balance updates for an address
    ///
    /// # Arguments
    /// * `network` - Blockchain network
    /// * `address` - Wallet address
    /// * `limit` - Maximum number of results
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get balance updates for Vitalik's address
    /// let balances = connector.get_balance_updates(
    ///     "eth",
    ///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    ///     100,
    /// ).await?;
    /// ```
    pub async fn get_balance_updates(
        &self,
        network: &str,
        address: &str,
        limit: u32,
    ) -> ExchangeResult<Vec<BalanceUpdate>> {
        let query = build_balance_updates_query(
            network,
            BitqueryDataset::Archive.as_str(),
            address,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_balance_updates(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - BLOCKS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get blocks
    ///
    /// # Arguments
    /// * `network` - Blockchain network
    /// * `limit` - Maximum number of results
    pub async fn get_blocks(
        &self,
        network: &str,
        limit: u32,
    ) -> ExchangeResult<Vec<BlockData>> {
        let query = build_blocks_query(
            network,
            BitqueryDataset::Archive.as_str(),
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_blocks(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - TRANSACTIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get transactions
    ///
    /// # Arguments
    /// * `network` - Blockchain network
    /// * `tx_hash` - Transaction hash (optional)
    /// * `from_address` - From address (optional)
    /// * `to_address` - To address (optional)
    /// * `limit` - Maximum number of results
    pub async fn get_transactions(
        &self,
        network: &str,
        tx_hash: Option<&str>,
        from_address: Option<&str>,
        to_address: Option<&str>,
        limit: u32,
    ) -> ExchangeResult<Vec<TransactionData>> {
        let query = build_transactions_query(
            network,
            BitqueryDataset::Archive.as_str(),
            tx_hash,
            from_address,
            to_address,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_transactions(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - SMART CONTRACT EVENTS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get smart contract events
    ///
    /// # Arguments
    /// * `network` - Blockchain network
    /// * `contract_address` - Smart contract address
    /// * `event_name` - Event name (e.g., "Transfer", optional)
    /// * `limit` - Maximum number of results
    pub async fn get_smart_contract_events(
        &self,
        network: &str,
        contract_address: &str,
        event_name: Option<&str>,
        limit: u32,
    ) -> ExchangeResult<Vec<SmartContractEvent>> {
        let query = build_events_query(
            network,
            BitqueryDataset::Archive.as_str(),
            contract_address,
            event_name,
            limit,
        );

        let response = self.execute_query(&query).await?;
        BitqueryParser::parse_events(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: ExchangeIdentity
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitqueryConnector {
    fn exchange_name(&self) -> &'static str {
        "bitquery"
    }

    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitquery
    }

    fn is_testnet(&self) -> bool {
        false // Bitquery doesn't have testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Data provider - no account types
        vec![]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: MarketData - UnsupportedOperation (not a CEX)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BitqueryConnector {
    async fn get_price(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a blockchain data provider - no CEX price data. Use get_dex_trades() for DEX prices.".to_string()
        ))
    }

    async fn get_ticker(
        &self,
        _symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a blockchain data provider - no ticker data. Use get_dex_trades() for DEX data.".to_string()
        ))
    }

    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a blockchain data provider - no orderbook data.".to_string()
        ))
    }

    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a blockchain data provider - no klines data. Use get_dex_trades() with aggregation.".to_string()
        ))
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping: execute minimal query
        let query = r#"{ __typename }"#;
        self.execute_query(query).await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Trading - UnsupportedOperation (data provider only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitqueryConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - trading not supported".to_string()
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - trading not supported".to_string()
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - trading not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Account - UnsupportedOperation (data provider only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitqueryConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - account operations not supported. Use get_balance_updates() for on-chain balances.".to_string()
        ))
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - account operations not supported".to_string()
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - account operations not supported. Use get_balance_updates() for on-chain balances.".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT: Positions - UnsupportedOperation (data provider only)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BitqueryConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - position tracking not supported".to_string()
        ))
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Bitquery is a data provider - position tracking not supported".to_string()
        ))
    }
}
