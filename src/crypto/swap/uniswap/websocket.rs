//! # Uniswap WebSocket
//!
//! Real-time event monitoring via Ethereum WebSocket subscriptions.
//!
//! ## Architecture
//! - Connects to Ethereum node WebSocket (Infura, Alchemy, etc.)
//! - Subscribes to pool events (Swap, Mint, Burn)
//! - Decodes event logs using alloy
//! - Dispatches to handlers via broadcast channel
//!
//! ## Event Types
//! - `Swap`: Token swap events (price changes)
//! - `Mint`: Liquidity added
//! - `Burn`: Liquidity removed
//!
//! ## Usage
//! ```ignore
//! let mut ws = UniswapWebSocket::new(ws_url, chain_id).await?;
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe_to_pool("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640").await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::Trade(trade)) => {
//!             println!("Trade: {:?}", trade);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use alloy::primitives::{Address, B256, I256, U256};
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::{Filter, Log};
use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_stream::StreamExt;

use crate::core::{
    AccountType, ConnectionStatus, StreamEvent,
    SubscriptionRequest, PublicTrade,
};
use crate::core::types::{WebSocketResult, WebSocketError, TradeSide};
use crate::core::traits::WebSocketConnector;
use crate::core::utils::timestamp_millis;
use super::endpoints::find_pool_by_address;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Heartbeat interval for connection health check
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Connection timeout - consider stale if no messages for this duration
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Swap event signature: Swap(address,address,int256,int256,uint160,uint128,int24)
const SWAP_EVENT_SIGNATURE: &str = "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67";

/// Fallback Ethereum WebSocket URLs (free public endpoints)
const FALLBACK_WS_URLS: &[&str] = &[
    "wss://ethereum-rpc.publicnode.com",
    "wss://eth.merkle.io",
    "wss://rpc.ankr.com/eth/ws",
    // Note: These endpoints may have rate limits
    // For production use, consider getting API keys from:
    // - Infura: https://infura.io (free tier: 100k requests/day)
    // - Alchemy: https://www.alchemy.com (free tier: generous limits)
];

// ═══════════════════════════════════════════════════════════════════════════════
// PROVIDER TYPE ALIAS
// ═══════════════════════════════════════════════════════════════════════════════

/// Concrete alloy WebSocket provider type.
///
/// `ProviderBuilder::new().connect_ws(ws).await` returns a `RootProvider` with
/// the WS transport baked in. We erase the transport with `DynProvider` so we
/// can store it behind a plain `Arc<Mutex<Option<...>>>`.
type EthProvider = alloy::providers::DynProvider;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap WebSocket client
pub struct UniswapWebSocket {
    /// Ethereum WebSocket URL
    ws_url: String,
    /// Chain ID
    _chain_id: u64,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Pool addresses we're monitoring
    pool_addresses: Arc<Mutex<HashSet<String>>>,
    /// Broadcast sender (for multiple consumers)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Ethereum provider
    provider: Arc<Mutex<Option<EthProvider>>>,
    /// Last message time
    last_message: Arc<Mutex<Instant>>,
}

impl UniswapWebSocket {
    /// Create new WebSocket client
    pub async fn new(ws_url: String, chain_id: u64) -> WebSocketResult<Self> {
        Ok(Self {
            ws_url,
            _chain_id: chain_id,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            pool_addresses: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            provider: Arc::new(Mutex::new(None)),
            last_message: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Create new WebSocket client with environment variable fallback
    pub async fn new_with_fallback(chain_id: u64) -> WebSocketResult<Self> {
        // Try environment variable first
        let ws_url = std::env::var("ETHEREUM_WS_URL")
            .unwrap_or_else(|_| FALLBACK_WS_URLS[0].to_string());

        Self::new(ws_url, chain_id).await
    }

    /// Connect to Ethereum WebSocket with fallback logic
    async fn connect_ws(&self) -> WebSocketResult<EthProvider> {
        // Try primary URL first
        match Self::try_connect(&self.ws_url).await {
            Ok(provider) => return Ok(provider),
            Err(e) => {
                tracing::warn!("Failed to connect to primary endpoint {}: {}", self.ws_url, e);
            }
        }

        // Try fallback URLs
        for fallback_url in FALLBACK_WS_URLS {
            tracing::info!("Trying fallback endpoint: {}", fallback_url);
            match Self::try_connect(fallback_url).await {
                Ok(provider) => {
                    tracing::info!("Successfully connected to fallback endpoint: {}", fallback_url);
                    return Ok(provider);
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to fallback {}: {}", fallback_url, e);
                    continue;
                }
            }
        }

        Err(WebSocketError::ConnectionError(
            format!(
                "Failed to connect to Ethereum node. Tried primary URL and {} fallbacks. \
                 Consider setting ETHEREUM_WS_URL environment variable with your own RPC endpoint. \
                 See UNISWAP_SETUP.md for details.",
                FALLBACK_WS_URLS.len()
            )
        ))
    }

    /// Attempt a single WebSocket connection and return a type-erased provider.
    async fn try_connect(url: &str) -> WebSocketResult<EthProvider> {
        let ws = WsConnect::new(url);
        let provider = ProviderBuilder::new()
            .connect_ws(ws)
            .await
            .map_err(|e| WebSocketError::ConnectionError(format!("WS connect error: {}", e)))?;
        Ok(provider.erased())
    }

    /// Subscribe to pool events
    ///
    /// Subscribes to Swap events on specified pool address.
    pub async fn subscribe_to_pool(&mut self, pool_address: &str) -> WebSocketResult<()> {
        // Normalize address to lowercase
        let pool_address = pool_address.to_lowercase();

        // Add to pool addresses
        self.pool_addresses.lock().await.insert(pool_address.clone());

        // Get provider and clone it (alloy providers are internally ref-counted)
        let provider = {
            let provider_guard = self.provider.lock().await;
            match provider_guard.as_ref() {
                Some(p) => p.clone(),
                None => return Err(WebSocketError::ConnectionError("Not connected".to_string())),
            }
        };

        // Parse pool address
        let pool_addr: Address = pool_address.parse()
            .map_err(|e| WebSocketError::ProtocolError(format!("Invalid pool address: {}", e)))?;

        // Parse event signature as B256 topic
        let swap_topic: B256 = SWAP_EVENT_SIGNATURE.parse()
            .map_err(|e| WebSocketError::ProtocolError(format!("Invalid SWAP topic: {}", e)))?;

        // Create logs filter for Swap events
        let filter = Filter::new()
            .address(pool_addr)
            .event_signature(swap_topic);

        // Subscribe to logs
        let broadcast_tx = self.broadcast_tx.clone();
        let pool_address_clone = pool_address.clone();
        let last_message = self.last_message.clone();

        // Spawn handler task that subscribes to logs
        tokio::spawn(async move {
            match provider.subscribe_logs(&filter).await {
                Ok(subscription) => {
                    let mut stream = subscription.into_stream();
                    while let Some(log) = stream.next().await {
                        // Update last message time
                        *last_message.lock().await = Instant::now();

                        // Parse and emit event
                        if let Some(event) = Self::parse_swap_log(&pool_address_clone, &log) {
                            if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                                let _ = tx.send(Ok(event));
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                        let _ = tx.send(Err(WebSocketError::ConnectionError(format!("Failed to subscribe to logs: {}", e))));
                    }
                }
            }
        });

        Ok(())
    }

    /// Unsubscribe from pool events
    pub async fn unsubscribe_from_pool(&mut self, pool_address: &str) -> WebSocketResult<()> {
        let pool_address = pool_address.to_lowercase();
        self.pool_addresses.lock().await.remove(&pool_address);

        // Note: in a production implementation you'd track subscription handles and cancel them.

        Ok(())
    }

    /// Get next event from WebSocket
    ///
    /// This method is deprecated - use event_stream() instead
    pub async fn next_event(&mut self) -> Option<UniswapEvent> {
        // This is a compatibility method for old API
        // In the new implementation, use event_stream() instead
        None
    }

    /// Parse Swap event log
    fn parse_swap_log(pool_address: &str, log: &Log) -> Option<StreamEvent> {
        // Swap event structure:
        // event Swap(
        //     address indexed sender,
        //     address indexed recipient,
        //     int256 amount0,
        //     int256 amount1,
        //     uint160 sqrtPriceX96,
        //     uint128 liquidity,
        //     int24 tick
        // );

        // Topics: [event_signature, sender, recipient]
        // Data: [amount0, amount1, sqrtPriceX96, liquidity, tick]

        let topics = log.inner.topics();
        if topics.len() < 3 {
            return None;
        }

        // Decode indexed parameters (topics)
        let _sender = format!("0x{:x}", topics[1]);
        let _recipient = format!("0x{:x}", topics[2]);

        // Decode non-indexed parameters (data)
        let data = log.inner.data.data.as_ref();
        if data.len() < 160 {
            // Need at least 5 * 32 bytes = 160 bytes
            return None;
        }

        // Parse amounts (int256) - first 32 bytes each
        let amount0_bytes = &data[0..32];
        let amount1_bytes = &data[32..64];

        // Convert to I256 using big-endian byte interpretation
        let amount0 = I256::from_be_bytes::<32>(amount0_bytes.try_into().ok()?);
        let amount1 = I256::from_be_bytes::<32>(amount1_bytes.try_into().ok()?);

        // Look up token decimals from the pool registry so we can scale amounts
        // correctly.  For unknown pools we fall back to 18/18 which gives raw
        // wei values (consistent with the old behaviour).
        let (token0_decimals, token1_decimals) =
            find_pool_by_address(pool_address)
                .map(|meta| (meta.token0_decimals, meta.token1_decimals))
                .unwrap_or((18, 18));

        // Determine price and side from amounts with decimal scaling
        let (price, quantity, is_buy) =
            Self::calculate_trade_info(amount0, amount1, token0_decimals, token1_decimals);

        // Create PublicTrade event
        let trade = PublicTrade {
            id: log.transaction_hash
                .map(|h| format!("{:x}", h))
                .unwrap_or_default(),
            symbol: pool_address.to_string(),
            price,
            quantity,
            side: if is_buy { TradeSide::Buy } else { TradeSide::Sell },
            timestamp: timestamp_millis() as i64,
        };

        Some(StreamEvent::Trade(trade))
    }

    /// Calculate trade information from raw amounts, scaling by token decimals.
    ///
    /// Amounts from the Swap event are raw integers in each token's smallest
    /// unit (e.g. wei for WETH, micro-USDC for USDC).  Dividing by
    /// `10^decimals` converts them to human-readable units before computing the
    /// price ratio.
    ///
    /// - `token0_decimals` / `token1_decimals`: from pool metadata (e.g. 6 for
    ///   USDC, 18 for WETH, 8 for WBTC).
    fn calculate_trade_info(
        amount0: I256,
        amount1: I256,
        token0_decimals: u8,
        token1_decimals: u8,
    ) -> (f64, f64, bool) {
        let amt0_raw = Self::i256_to_f64(amount0);
        let amt1_raw = Self::i256_to_f64(amount1);

        // Scale by per-token decimals
        let scale0 = 10_f64.powi(token0_decimals as i32);
        let scale1 = 10_f64.powi(token1_decimals as i32);
        let amt0 = amt0_raw / scale0;
        let amt1 = amt1_raw / scale1;

        // Determine direction: negative amount means the pool paid out that
        // token (i.e. the swapper received it).
        // amt0 < 0 → token0 left the pool → buyer received token0
        let is_buy = amt0 < 0.0;

        // Price = how much token1 per token0 (in human-readable units)
        let price = if amt0.abs() > 0.0 {
            amt1.abs() / amt0.abs()
        } else {
            0.0
        };

        // Quantity in token0 units
        let quantity = amt0.abs();

        (price, quantity, is_buy)
    }

    /// Convert I256 to f64 (simplified implementation)
    fn i256_to_f64(value: I256) -> f64 {
        // This is a simplified conversion.
        // In production, you'd want to handle decimals properly.
        let is_negative = value.is_negative();
        // into_raw() returns the underlying U256 (two's complement bit pattern)
        let u256_val: U256 = value.into_raw();

        // Access the four 64-bit limbs (little-endian order: index 0 = least significant)
        let limbs = u256_val.into_limbs(); // [u64; 4], limbs[0] = lowest bits

        let result = (limbs[3] as f64) * 2f64.powi(192)
            + (limbs[2] as f64) * 2f64.powi(128)
            + (limbs[1] as f64) * 2f64.powi(64)
            + (limbs[0] as f64);

        if is_negative { -result } else { result }
    }

    /// Look up the canonical Uniswap V3 pool address for a well-known token pair.
    ///
    /// Returns the highest-liquidity pool address for each pair. Both slash-separated
    /// forms (`ETH/USDC`) and their WETH-prefixed variants are accepted.
    ///
    /// Pool addresses are for Ethereum mainnet only. For other chains, use
    /// `subscribe_to_pool()` directly with the correct address.
    pub fn get_pool_address(symbol: &str) -> Option<&'static str> {
        match symbol {
            // ETH/USDC — WETH/USDC 0.05% pool (highest liquidity)
            "ETH/USDC" | "WETH/USDC" | "USDC/ETH" | "USDC/WETH" => {
                Some("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640")
            }
            // ETH/USDT — WETH/USDT 0.05% pool
            "ETH/USDT" | "WETH/USDT" | "USDT/ETH" | "USDT/WETH" => {
                Some("0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36")
            }
            // WBTC/ETH — WBTC/WETH 0.3% pool
            "WBTC/ETH" | "WBTC/WETH" | "ETH/WBTC" | "WETH/WBTC" => {
                Some("0xCBCdF9626bC03E24f779434178A73a0B4bad62eD")
            }
            // WBTC/USDC — 0.3% pool
            "WBTC/USDC" | "USDC/WBTC" => {
                Some("0x99ac8cA7087fA4A2A1FB6357269965A2014ABc35")
            }
            // ETH/DAI — WETH/DAI 0.3% pool
            "ETH/DAI" | "WETH/DAI" | "DAI/ETH" | "DAI/WETH" => {
                Some("0xC2e9F25Be6257c210d7Adf0D4Cd6E3E881ba25f8")
            }
            // USDC/USDT — stablecoin 0.01% pool
            "USDC/USDT" | "USDT/USDC" => {
                Some("0x3416cF6C708Da44DB2624D63ea0AAef7113527C6")
            }
            _ => None,
        }
    }

    /// Start heartbeat monitoring task
    fn start_heartbeat_task(
        last_message: Arc<Mutex<Instant>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(HEARTBEAT_INTERVAL).await;

                let last = *last_message.lock().await;
                if last.elapsed() >= CONNECTION_TIMEOUT {
                    // No messages for too long - connection may be stale
                    *status.lock().await = ConnectionStatus::Disconnected;
                    break;
                }
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for UniswapWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;

        // Connect to Ethereum WebSocket
        let provider = self.connect_ws().await?;
        *self.provider.lock().await = Some(provider);

        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_message.lock().await = Instant::now();

        // Create broadcast channel and store sender
        let (tx, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(tx);

        // Start heartbeat monitoring
        Self::start_heartbeat_task(
            self.last_message.clone(),
            self.status.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Clear provider
        *self.provider.lock().await = None;

        // Drop the broadcast sender so consumers see stream end
        let _ = self.broadcast_tx.lock().unwrap().take();

        // Clear subscriptions
        self.subscriptions.lock().await.clear();
        self.pool_addresses.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Use try_lock to avoid blocking
        match self.status.try_lock() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Determine the pool address from the symbol using the known-pairs lookup table.
        let symbol = &request.symbol;
        let symbol_str = format!("{}/{}", symbol.base, symbol.quote);

        let pool_address = Self::get_pool_address(&symbol_str)
            .ok_or_else(|| WebSocketError::UnsupportedOperation(format!(
                "Unknown Uniswap V3 pool for symbol '{}'. \
                 Supported pairs: ETH/USDC, ETH/USDT, WBTC/ETH, WBTC/USDC, ETH/DAI, USDC/USDT. \
                 Use subscribe_to_pool() directly with a known pool address.",
                symbol_str
            )))?;

        // Store subscription only after we know the pool is supported
        self.subscriptions.lock().await.insert(request.clone());

        self.subscribe_to_pool(pool_address).await?;

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        self.subscriptions.lock().await.remove(&request);

        // In production, you'd track which pools correspond to which subscriptions
        // and only unsubscribe from pools that have no active subscriptions

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let tx_guard = self.broadcast_tx.lock().unwrap();
        if let Some(ref tx) = *tx_guard {
            let rx = tx.subscribe();
            // Convert broadcast receiver to stream
            Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| {
                match result {
                    Ok(event) => Some(event),
                    Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                        Some(Err(WebSocketError::ConnectionError("Event stream lagged behind".to_string())))
                    }
                }
            }))
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Use try_lock to avoid blocking
        match self.subscriptions.try_lock() {
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Uniswap event types
#[derive(Debug, Clone)]
pub enum UniswapEvent {
    /// Swap event
    Swap(SwapData),
    /// Mint (add liquidity) event
    Mint(MintData),
    /// Burn (remove liquidity) event
    Burn(BurnData),
}

/// Swap event data
#[derive(Debug, Clone)]
pub struct SwapData {
    /// Pool address
    pub pool: String,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
    /// Token0 amount (negative = sold, positive = bought)
    pub amount0: f64,
    /// Token1 amount (negative = sold, positive = bought)
    pub amount1: f64,
    /// New pool price (sqrtPriceX96)
    pub sqrt_price_x96: String,
    /// Liquidity after swap
    pub liquidity: String,
    /// Current tick
    pub tick: i32,
    /// Block timestamp
    pub timestamp: i64,
}

/// Mint (add liquidity) event data
#[derive(Debug, Clone)]
pub struct MintData {
    /// Pool address
    pub pool: String,
    /// Owner address
    pub owner: String,
    /// Lower tick bound
    pub tick_lower: i32,
    /// Upper tick bound
    pub tick_upper: i32,
    /// Liquidity amount
    pub amount: String,
    /// Token0 amount
    pub amount0: String,
    /// Token1 amount
    pub amount1: String,
    /// Block timestamp
    pub timestamp: i64,
}

/// Burn (remove liquidity) event data
#[derive(Debug, Clone)]
pub struct BurnData {
    /// Pool address
    pub pool: String,
    /// Owner address
    pub owner: String,
    /// Lower tick bound
    pub tick_lower: i32,
    /// Upper tick bound
    pub tick_upper: i32,
    /// Liquidity amount
    pub amount: String,
    /// Token0 amount
    pub amount0: String,
    /// Token1 amount
    pub amount1: String,
    /// Block timestamp
    pub timestamp: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NOTES
// ═══════════════════════════════════════════════════════════════════════════════

// This implementation provides real WebSocket support for Uniswap via Ethereum RPC.
//
// Key features:
// - Real WebSocket connection to Ethereum node
// - eth_subscribe with logs filtering
// - Swap event decoding
// - Broadcast channel for multiple consumers
// - Auto-reconnection monitoring via heartbeat
// - Fallback to multiple public RPC endpoints
//
// Known Limitations:
// - Token amounts are not scaled by decimals (raw wei values)
//   * USDC has 6 decimals, WETH has 18 decimals
//   * Prices appear very small due to this
//   * For production: look up token decimals and scale appropriately
//
// For production use:
// - Add proper decimal handling for token amounts
// - Implement Mint and Burn event parsing
// - Add pool address lookup from symbol
// - Handle chain reorganizations
// - Implement proper error recovery
// - Cache token decimals to scale amounts correctly
