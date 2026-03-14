//! # Raydium WebSocket Implementation
//!
//! WebSocket connector using Solana RPC account subscriptions to monitor Raydium pool updates.
//!
//! ## Implementation Details
//!
//! This implementation uses raw WebSocket connections to Solana RPC endpoints,
//! avoiding the need for `solana-client` dependencies which require OpenSSL.
//!
//! **Architecture:**
//! - Connects to Solana RPC WebSocket endpoint (wss://api.mainnet-beta.solana.com)
//! - First fetches the AMM pool account via REST to extract vault token account addresses
//! - Subscribes to both vault SPL token accounts via `accountSubscribe`
//! - Parses SPL token account data (165 bytes) to extract current balances
//! - Calculates price from vault reserves ratio
//! - Emits StreamEvent::Ticker on pool updates
//!
//! **Why vault accounts instead of pool account?**
//! - Vault token accounts update on EVERY swap (balance changes)
//! - SPL token layout is simple and stable (amount at offset 64)
//! - No need to parse complex AmmInfo struct with version-dependent offsets
//!
//! **Benefits:**
//! - No OpenSSL dependency (uses native-tls from tokio-tungstenite)
//! - Real-time pool state monitoring via vault balance changes
//! - Automatic price calculation from pool reserves
//! - Broadcast channel for multiple consumers
//!
//! ## Usage
//!
//! ```no_run
//! use connectors_v5::exchanges::raydium::{RaydiumWebSocket, well_known_mints};
//! use connectors_v5::core::{AccountType, Symbol, SubscriptionRequest, StreamType};
//! use connectors_v5::core::traits::WebSocketConnector;
//! use futures_util::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ws = RaydiumWebSocket::new(false).await?;
//! ws.connect(AccountType::Spot).await?;
//!
//! let sol_usdc = Symbol::new(
//!     well_known_mints::SOL,
//!     well_known_mints::USDC,
//! );
//!
//! let request = SubscriptionRequest {
//!     symbol: sol_usdc,
//!     stream_type: StreamType::Ticker,
//! };
//!
//! ws.subscribe(request).await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(stream_event) => println!("{:?}", stream_event),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::pin::Pin;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::core::{
    AccountType, ConnectionStatus, StreamEvent, Ticker,
    SubscriptionRequest, Symbol,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::WebSocketConnector;
use crate::core::utils;

use super::well_known_mints;

// For parsing Solana addresses
use bs58;
use base64::Engine as _;

// =====================================================================
// SOLANA RPC JSON-RPC TYPES
// =====================================================================

#[derive(Debug, Serialize)]
struct SolanaRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default, rename = "method")]
    _method: Option<String>,
    #[serde(default)]
    params: Option<SolanaRpcParams>,
}

#[derive(Debug, Deserialize)]
struct SolanaRpcParams {
    result: AccountUpdateResult,
    subscription: u64,
}

#[derive(Debug, Deserialize)]
struct AccountUpdateResult {
    value: AccountInfo,
}

#[derive(Debug, Deserialize)]
struct AccountInfo {
    data: Vec<String>,
    #[allow(dead_code)]
    executable: bool,
    #[allow(dead_code)]
    lamports: u64,
    #[allow(dead_code)]
    owner: String,
    #[serde(rename = "rentEpoch")]
    #[allow(dead_code)]
    rent_epoch: u64,
}

// =====================================================================
// RAYDIUM AMM V4 ACCOUNT PARSER
// =====================================================================

/// Raydium AMM V4 AmmInfo account layout offsets.
///
/// The AmmInfo struct is repr(C, packed) with the following layout:
///   Bytes 0-127:   16 x u64 fields (status, nonce, order_num, depth, coin_decimals,
///                  pc_decimals, state, reset_flag, min_size, vol_max_cut_ratio,
///                  amount_wave, coin_lot_size, pc_lot_size, min_price_multiplier,
///                  max_price_multiplier, sys_decimal_value)
///   Bytes 128-191: Fees struct (8 x u64 = 64 bytes)
///   Bytes 192-335: StateData struct (144 bytes, contains u128 fields)
///   Bytes 336-367: coin_vault (Pubkey, 32 bytes) - SPL token account for base token
///   Bytes 368-399: pc_vault (Pubkey, 32 bytes) - SPL token account for quote token
///   Bytes 400-431: coin_vault_mint (Pubkey, 32 bytes) - base token mint
///   Bytes 432-463: pc_vault_mint (Pubkey, 32 bytes) - quote token mint
///   ... more fields follow
mod amm_offsets {
    /// coin_decimals field (u64): number of decimals for base token
    pub const COIN_DECIMALS: usize = 32;
    /// pc_decimals field (u64): number of decimals for quote token
    pub const PC_DECIMALS: usize = 40;
    /// coin_vault (Pubkey): SPL token account holding base token reserves
    pub const COIN_VAULT: usize = 336;
    /// pc_vault (Pubkey): SPL token account holding quote token reserves
    pub const PC_VAULT: usize = 368;
    /// coin_vault_mint (Pubkey): base token mint address
    pub const COIN_VAULT_MINT: usize = 400;
    /// pc_vault_mint (Pubkey): quote token mint address
    pub const PC_VAULT_MINT: usize = 432;
    /// Minimum account data size for AmmInfo
    pub const MIN_SIZE: usize = 464;
}

/// SPL Token Account layout offsets.
/// Standard SPL token account is 165 bytes.
mod spl_token_offsets {
    /// Amount (u64, 8 bytes) at offset 64
    pub const AMOUNT: usize = 64;
    /// Minimum account data size
    pub const MIN_SIZE: usize = 72;
}

/// Parsed vault info from AmmInfo account
#[derive(Debug, Clone)]
struct PoolVaultInfo {
    pub coin_vault: String,
    pub pc_vault: String,
    pub coin_mint: String,
    pub pc_mint: String,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
}

impl PoolVaultInfo {
    /// Parse vault addresses and decimals from AmmInfo account data
    fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < amm_offsets::MIN_SIZE {
            return Err(format!(
                "AmmInfo data too short: {} bytes (need {})",
                data.len(),
                amm_offsets::MIN_SIZE
            ));
        }

        let coin_decimals = u64::from_le_bytes(
            data[amm_offsets::COIN_DECIMALS..amm_offsets::COIN_DECIMALS + 8]
                .try_into()
                .map_err(|e| format!("Failed to parse coin_decimals: {:?}", e))?,
        );
        let pc_decimals = u64::from_le_bytes(
            data[amm_offsets::PC_DECIMALS..amm_offsets::PC_DECIMALS + 8]
                .try_into()
                .map_err(|e| format!("Failed to parse pc_decimals: {:?}", e))?,
        );

        let coin_vault =
            bs58::encode(&data[amm_offsets::COIN_VAULT..amm_offsets::COIN_VAULT + 32])
                .into_string();
        let pc_vault =
            bs58::encode(&data[amm_offsets::PC_VAULT..amm_offsets::PC_VAULT + 32]).into_string();
        let coin_mint =
            bs58::encode(&data[amm_offsets::COIN_VAULT_MINT..amm_offsets::COIN_VAULT_MINT + 32])
                .into_string();
        let pc_mint =
            bs58::encode(&data[amm_offsets::PC_VAULT_MINT..amm_offsets::PC_VAULT_MINT + 32])
                .into_string();

        Ok(Self {
            coin_vault,
            pc_vault,
            coin_mint,
            pc_mint,
            coin_decimals,
            pc_decimals,
        })
    }
}

/// Parse token amount from SPL token account data
fn parse_spl_token_amount(data: &[u8]) -> Result<u64, String> {
    if data.len() < spl_token_offsets::MIN_SIZE {
        return Err(format!(
            "SPL token data too short: {} bytes (need {})",
            data.len(),
            spl_token_offsets::MIN_SIZE
        ));
    }
    Ok(u64::from_le_bytes(
        data[spl_token_offsets::AMOUNT..spl_token_offsets::AMOUNT + 8]
            .try_into()
            .map_err(|e| format!("Failed to parse amount: {:?}", e))?,
    ))
}

// =====================================================================
// WEBSOCKET CONNECTOR
// =====================================================================

/// Raydium WebSocket connector using Solana RPC.
///
/// Subscribes to the vault SPL token accounts of Raydium AMM pools to
/// get real-time balance updates, then calculates price from the ratio.
pub struct RaydiumWebSocket {
    ws_url: String,
    rpc_url: String,
    status: Arc<Mutex<ConnectionStatus>>,
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender created at construction time — always valid, never None.
    /// Receivers are created on demand via `tx.subscribe()`.
    broadcast_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    sub_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    last_ping: Arc<Mutex<Instant>>,
    /// Cache mapping "{base_mint}/{quote_mint}" → pool_address to avoid
    /// redundant REST lookups when subscribing to the same pair repeatedly.
    pool_cache: Arc<StdMutex<HashMap<String, String>>>,
}

impl RaydiumWebSocket {
    /// Create new Raydium WebSocket connector
    pub async fn new(is_testnet: bool) -> WebSocketResult<Self> {
        let (ws_url, rpc_url) = if is_testnet {
            (
                "wss://api.devnet.solana.com".to_string(),
                "https://api.devnet.solana.com".to_string(),
            )
        } else {
            (
                "wss://api.mainnet-beta.solana.com".to_string(),
                "https://api.mainnet-beta.solana.com".to_string(),
            )
        };

        // Create the broadcast channel here so `event_stream()` can be called
        // safely before any subscription task is spawned.
        let (broadcast_tx, _) = broadcast::channel(1000);

        // Pre-populate the pool cache with known pairs to avoid REST lookups
        // for the most common pairs.
        let mut cache = HashMap::new();
        let sol_usdc_key = format!(
            "{}/{}",
            well_known_mints::SOL,
            well_known_mints::USDC
        );
        cache.insert(sol_usdc_key, well_known_mints::SOL_USDC_POOL.to_string());

        Ok(Self {
            ws_url,
            rpc_url,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx,
            sub_handles: Arc::new(Mutex::new(Vec::new())),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            pool_cache: Arc::new(StdMutex::new(cache)),
        })
    }

    /// Fetch pool account via REST RPC to get vault addresses
    async fn fetch_pool_vault_info(
        rpc_url: &str,
        pool_address: &str,
    ) -> Result<PoolVaultInfo, String> {
        let client = reqwest::Client::new();

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                pool_address,
                {"encoding": "base64"}
            ]
        });

        let response = client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("RPC request failed: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse RPC response: {}", e))?;

        let data_str = json["result"]["value"]["data"][0]
            .as_str()
            .ok_or_else(|| "Missing account data in RPC response".to_string())?;

        use base64::Engine;
        let account_data = base64::engine::general_purpose::STANDARD
            .decode(data_str)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

        PoolVaultInfo::parse(&account_data)
    }

    /// Subscribe to a specific Raydium pool by monitoring its vault token accounts
    async fn subscribe_to_pool(
        &self,
        pool_address: &str,
        request: SubscriptionRequest,
    ) -> WebSocketResult<()> {
        // First, fetch vault info from the pool account
        let vault_info = Self::fetch_pool_vault_info(&self.rpc_url, pool_address)
            .await
            .map_err(|e| {
                WebSocketError::ConnectionError(format!(
                    "Failed to fetch pool vault info: {}",
                    e
                ))
            })?;

        tracing::info!(
            "Pool {} vaults: coin={} pc={} coin_mint={} pc_mint={} decimals=({},{})",
            pool_address,
            vault_info.coin_vault,
            vault_info.pc_vault,
            vault_info.coin_mint,
            vault_info.pc_mint,
            vault_info.coin_decimals,
            vault_info.pc_decimals
        );

        let ws_url = self.ws_url.clone();
        // Clone the sender — the background task broadcasts on the same channel
        // that was created in `new()`.  Receivers obtained via `event_stream()`
        // before or after spawning will both receive events correctly.
        let broadcast_tx = self.broadcast_tx.clone();
        let last_ping = self.last_ping.clone();
        let status = self.status.clone();
        let symbol = request.symbol.clone();

        let handle = tokio::spawn(async move {
            let mut reconnect_delay = Duration::from_secs(1);
            let max_delay = Duration::from_secs(60);

            loop {
                match Self::run_vault_subscription(
                    &ws_url,
                    &vault_info,
                    &symbol,
                    &broadcast_tx,
                    &last_ping,
                    &status,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("Subscription loop ended normally");
                        reconnect_delay = Duration::from_secs(1);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Subscription error: {:?}, reconnecting in {:?}",
                            e,
                            reconnect_delay
                        );
                        let _ = broadcast_tx.send(Err(WebSocketError::ConnectionError(
                            format!("Subscription error: {}", e),
                        )));
                    }
                }

                *status.lock().await = ConnectionStatus::Disconnected;
                sleep(reconnect_delay).await;

                // Exponential backoff
                reconnect_delay = std::cmp::min(reconnect_delay * 2, max_delay);
            }
        });

        self.sub_handles.lock().await.push(handle);
        Ok(())
    }

    /// Run subscription to both vault token accounts on a single WS connection
    async fn run_vault_subscription(
        ws_url: &str,
        vault_info: &PoolVaultInfo,
        symbol: &Symbol,
        broadcast_tx: &broadcast::Sender<WebSocketResult<StreamEvent>>,
        last_ping: &Arc<Mutex<Instant>>,
        status: &Arc<Mutex<ConnectionStatus>>,
    ) -> WebSocketResult<()> {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to connect: {}", e)))?;

        tracing::info!("Connected to Solana RPC WebSocket");
        *status.lock().await = ConnectionStatus::Connected;

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to coin vault (base token account)
        let coin_subscribe = SolanaRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "accountSubscribe".to_string(),
            params: vec![
                serde_json::json!(vault_info.coin_vault),
                serde_json::json!({
                    "encoding": "base64",
                    "commitment": "confirmed"
                }),
            ],
        };

        let msg = serde_json::to_string(&coin_subscribe)
            .map_err(|e| WebSocketError::ProtocolError(format!("Failed to serialize: {}", e)))?;
        write
            .send(Message::Text(msg))
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to send: {}", e)))?;
        tracing::info!(
            "Sent accountSubscribe for coin vault: {}",
            vault_info.coin_vault
        );

        // Subscribe to pc vault (quote token account)
        let pc_subscribe = SolanaRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 2,
            method: "accountSubscribe".to_string(),
            params: vec![
                serde_json::json!(vault_info.pc_vault),
                serde_json::json!({
                    "encoding": "base64",
                    "commitment": "confirmed"
                }),
            ],
        };

        let msg = serde_json::to_string(&pc_subscribe)
            .map_err(|e| WebSocketError::ProtocolError(format!("Failed to serialize: {}", e)))?;
        write
            .send(Message::Text(msg))
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to send: {}", e)))?;
        tracing::info!(
            "Sent accountSubscribe for pc vault: {}",
            vault_info.pc_vault
        );

        // Track subscription IDs and current reserves
        let mut coin_sub_id: Option<u64> = None;
        let mut pc_sub_id: Option<u64> = None;
        let mut coin_amount: Option<u64> = None;
        let mut pc_amount: Option<u64> = None;
        let coin_scale = 10_f64.powi(vault_info.coin_decimals as i32);
        let pc_scale = 10_f64.powi(vault_info.pc_decimals as i32);

        // Read messages
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    *last_ping.lock().await = Instant::now();

                    match serde_json::from_str::<SolanaRpcResponse>(&text) {
                        Ok(response) => {
                            // Subscription confirmation
                            if let Some(ref result) = response.result {
                                if let Some(sub_id) = result.as_u64() {
                                    if coin_sub_id.is_none() {
                                        coin_sub_id = Some(sub_id);
                                        tracing::info!(
                                            "Coin vault subscription confirmed: ID {}",
                                            sub_id
                                        );
                                    } else if pc_sub_id.is_none() {
                                        pc_sub_id = Some(sub_id);
                                        tracing::info!(
                                            "PC vault subscription confirmed: ID {}",
                                            sub_id
                                        );
                                    }
                                }
                                continue;
                            }

                            // Account update notification
                            if let Some(params) = response.params {
                                let sub_id = params.subscription;

                                if params.result.value.data.is_empty() {
                                    tracing::warn!("Empty account data in update");
                                    continue;
                                }

                                let data_str = &params.result.value.data[0];
                                let account_data = match base64::engine::general_purpose::STANDARD.decode(data_str) {
                                    Ok(d) => d,
                                    Err(e) => {
                                        tracing::warn!("Failed to decode base64: {}", e);
                                        continue;
                                    }
                                };

                                let amount = match parse_spl_token_amount(&account_data) {
                                    Ok(a) => a,
                                    Err(e) => {
                                        tracing::warn!("Failed to parse token amount: {}", e);
                                        continue;
                                    }
                                };

                                // Determine which vault updated
                                let is_coin =
                                    coin_sub_id.is_some_and(|id| id == sub_id);
                                let is_pc =
                                    pc_sub_id.is_some_and(|id| id == sub_id);

                                if is_coin {
                                    coin_amount = Some(amount);
                                    tracing::debug!(
                                        "Coin vault update: {} raw ({:.6} scaled)",
                                        amount,
                                        amount as f64 / coin_scale
                                    );
                                } else if is_pc {
                                    pc_amount = Some(amount);
                                    tracing::debug!(
                                        "PC vault update: {} raw ({:.6} scaled)",
                                        amount,
                                        amount as f64 / pc_scale
                                    );
                                } else {
                                    tracing::warn!("Unknown subscription ID: {}", sub_id);
                                    continue;
                                }

                                // Emit price when we have both reserves
                                if let (Some(coin), Some(pc)) = (coin_amount, pc_amount) {
                                    let coin_scaled = coin as f64 / coin_scale;
                                    let pc_scaled = pc as f64 / pc_scale;

                                    if coin_scaled > 0.0 {
                                        let price = pc_scaled / coin_scaled;

                                        tracing::debug!(
                                            "Pool price update: coin={:.4}, pc={:.4}, price={:.4}",
                                            coin_scaled,
                                            pc_scaled,
                                            price
                                        );

                                        let ticker = Ticker {
                                            symbol: format!(
                                                "{}/{}",
                                                symbol.base, symbol.quote
                                            ),
                                            last_price: price,
                                            bid_price: None,
                                            ask_price: None,
                                            high_24h: None,
                                            low_24h: None,
                                            volume_24h: Some(coin_scaled),
                                            quote_volume_24h: Some(pc_scaled),
                                            price_change_24h: None,
                                            price_change_percent_24h: None,
                                            timestamp: utils::timestamp_millis() as i64,
                                        };

                                        let _ = broadcast_tx.send(Ok(StreamEvent::Ticker(ticker)));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse RPC response: {} - {}",
                                e,
                                &text[..text.len().min(200)]
                            );
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    write.send(Message::Pong(data)).await.ok();
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket closed by server");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    return Err(WebSocketError::ConnectionError(format!(
                        "WebSocket error: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Resolve pool address for a symbol.
    ///
    /// Resolution order:
    /// 1. In-memory cache (populated with well-known pairs at startup and with
    ///    every successful REST lookup).
    /// 2. Raydium REST API `GET /pools/info/mint` — returns the highest-TVL
    ///    pool for the given mint pair.
    ///
    /// The result is cached so subsequent calls for the same pair are free.
    async fn resolve_pool_address(&self, symbol: &Symbol) -> Result<String, String> {
        let cache_key = format!("{}/{}", symbol.base, symbol.quote);

        // 1. Check cache first
        if let Ok(cache) = self.pool_cache.lock() {
            if let Some(addr) = cache.get(&cache_key) {
                return Ok(addr.clone());
            }
        }

        // 2. Dynamic REST lookup via Raydium /pools/info/mint
        let pool_address = Self::fetch_pool_address_by_mints(
            &self.rpc_url,
            &symbol.base,
            &symbol.quote,
        )
        .await?;

        // Store in cache
        if let Ok(mut cache) = self.pool_cache.lock() {
            cache.insert(cache_key, pool_address.clone());
        }

        Ok(pool_address)
    }

    /// Fetch the highest-TVL pool address for a mint pair via the Raydium REST API.
    ///
    /// Note: `rpc_url` here refers to the Solana RPC used for WS connections, but
    /// this REST call goes to the Raydium API V3 base URL derived from the same
    /// network (mainnet/devnet).
    async fn fetch_pool_address_by_mints(
        rpc_url: &str,
        mint_a: &str,
        mint_b: &str,
    ) -> Result<String, String> {
        // Derive the Raydium API V3 base from the RPC URL convention
        let api_base = if rpc_url.contains("devnet") {
            "https://api-v3-devnet.raydium.io"
        } else {
            "https://api-v3.raydium.io"
        };

        let url = format!(
            "{}/pools/info/mint?mint1={}&mint2={}&sort=liquidity&order=desc&page=1",
            api_base, mint_a, mint_b
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("REST pool lookup failed: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse pool lookup response: {}", e))?;

        // Response shape: { "success": true, "data": { "data": [ { "id": "...", ... } ] } }
        let pool_id = json
            .get("data")
            .and_then(|d| d.get("data"))
            .and_then(|arr| arr.as_array())
            .and_then(|arr| arr.first())
            .and_then(|pool| pool.get("id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                format!("No pool found for mints {}/{}", mint_a, mint_b)
            })?;

        Ok(pool_id.to_string())
    }
}

#[async_trait]
impl WebSocketConnector for RaydiumWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_ping.lock().await = Instant::now();
        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;
        let mut handles = self.sub_handles.lock().await;
        for handle in handles.drain(..) {
            handle.abort();
        }
        self.subscriptions.lock().await.clear();
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let pool_address = self
            .resolve_pool_address(&request.symbol)
            .await
            .map_err(WebSocketError::ProtocolError)?;
        self.subscribe_to_pool(&pool_address, request.clone())
            .await?;
        self.subscriptions.lock().await.insert(request);
        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.remove(&request);
        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.subscribe();
        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
                match result {
                    Ok(event) => Some(event),
                    Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                        Some(Err(WebSocketError::ConnectionError(
                            "Event stream lagged behind".to_string(),
                        )))
                    }
                }
            }),
        )
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}
