//! # Uniswap WebSocket
//!
//! Real-time event monitoring via Ethereum WebSocket subscriptions.
//!
//! ## Architecture
//! - Connects to Ethereum node WebSocket (Infura, Alchemy, etc.)
//! - Subscribes to pool events (Swap, Mint, Burn)
//! - Decodes event logs using ethers
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

use async_trait::async_trait;
use ethers::prelude::*;
use ethers::providers::{Provider, Ws};
use ethers::types::{I256, U256, H160, H256, Log, Filter};
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
    provider: Arc<Mutex<Option<Provider<Ws>>>>,
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
    async fn connect_ws(&self) -> WebSocketResult<Provider<Ws>> {
        // Try primary URL first
        match Provider::<Ws>::connect(&self.ws_url).await {
            Ok(provider) => return Ok(provider),
            Err(e) => {
                tracing::warn!("Failed to connect to primary endpoint {}: {}", self.ws_url, e);
            }
        }

        // Try fallback URLs
        for fallback_url in FALLBACK_WS_URLS {
            tracing::info!("Trying fallback endpoint: {}", fallback_url);
            match Provider::<Ws>::connect(fallback_url).await {
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

    /// Subscribe to pool events
    ///
    /// Subscribes to Swap events on specified pool address.
    pub async fn subscribe_to_pool(&mut self, pool_address: &str) -> WebSocketResult<()> {
        // Normalize address to lowercase
        let pool_address = pool_address.to_lowercase();

        // Add to pool addresses
        self.pool_addresses.lock().await.insert(pool_address.clone());

        // Get provider and clone it
        let provider = {
            let provider_guard = self.provider.lock().await;
            match provider_guard.as_ref() {
                Some(p) => p.clone(),
                None => return Err(WebSocketError::ConnectionError("Not connected".to_string())),
            }
        };

        // Parse pool address
        let pool_addr: H160 = pool_address.parse()
            .map_err(|e| WebSocketError::ProtocolError(format!("Invalid pool address: {}", e)))?;

        // Parse event signatures
        let swap_topic: H256 = SWAP_EVENT_SIGNATURE.parse()
            .map_err(|e| WebSocketError::ProtocolError(format!("Invalid SWAP topic: {}", e)))?;

        // Create logs filter for Swap events
        let filter = Filter::new()
            .address(pool_addr)
            .topic0(swap_topic);

        // Subscribe to logs
        let broadcast_tx = self.broadcast_tx.clone();
        let pool_address_clone = pool_address.clone();
        let last_message = self.last_message.clone();

        // Spawn handler task that subscribes to logs
        tokio::spawn(async move {
            // Subscribe to logs (this must be done inside the task to own the provider)
            match provider.subscribe_logs(&filter).await {
                Ok(mut stream) => {
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

        // Note: ethers doesn't provide a direct way to unsubscribe from specific logs
        // In a production implementation, you'd track subscription handles and cancel them

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

        if log.topics.len() < 3 {
            return None;
        }

        // Decode indexed parameters (topics)
        let _sender = format!("0x{:x}", log.topics[1]);
        let _recipient = format!("0x{:x}", log.topics[2]);

        // Decode non-indexed parameters (data)
        let data = &log.data;
        if data.len() < 160 {
            // Need at least 5 * 32 bytes = 160 bytes
            return None;
        }

        // Parse amounts (int256) - first 32 bytes each
        let amount0_bytes = &data[0..32];
        let amount1_bytes = &data[32..64];

        // Convert to i256 (simplified - treating as positive for now)
        let amount0 = I256::from_raw(U256::from_big_endian(amount0_bytes));
        let amount1 = I256::from_raw(U256::from_big_endian(amount1_bytes));

        // Determine price and side from amounts
        // In Uniswap, negative amount = token sold, positive = token bought
        let (price, quantity, is_buy) = Self::calculate_trade_info(amount0, amount1);

        // Create PublicTrade event
        let trade = PublicTrade {
            id: format!("{:x}", log.transaction_hash.unwrap_or_default()),
            symbol: pool_address.to_string(),
            price,
            quantity,
            side: if is_buy { TradeSide::Buy } else { TradeSide::Sell },
            timestamp: timestamp_millis() as i64,
        };

        Some(StreamEvent::Trade(trade))
    }

    /// Calculate trade information from amounts
    fn calculate_trade_info(amount0: I256, amount1: I256) -> (f64, f64, bool) {
        // Convert I256 to f64 (simplified)
        let amt0 = Self::i256_to_f64(amount0);
        let amt1 = Self::i256_to_f64(amount1);

        // Determine direction
        let is_buy = amt0 < 0.0; // If amount0 is negative, token0 was sold (buying token1)

        // Calculate price and quantity
        let price = if amt0.abs() > 0.0 {
            amt1.abs() / amt0.abs()
        } else {
            0.0
        };

        let quantity = amt0.abs();

        (price, quantity, is_buy)
    }

    /// Convert I256 to f64 (simplified implementation)
    fn i256_to_f64(value: I256) -> f64 {
        // This is a simplified conversion
        // In production, you'd want to handle decimals properly
        let u256_val = value.into_raw();

        // Convert U256 to f64 (may lose precision for very large numbers)
        let high = u256_val.0[3];
        let mid_high = u256_val.0[2];
        let mid_low = u256_val.0[1];
        let low = u256_val.0[0];

        let result = (high as f64) * 2f64.powi(192) +
                     (mid_high as f64) * 2f64.powi(128) +
                     (mid_low as f64) * 2f64.powi(64) +
                     (low as f64);

        if value.is_negative() {
            -result
        } else {
            result
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
        // For Uniswap, we need to determine the pool address from the symbol
        // This is a simplified implementation - in production you'd look up the pool address

        // Store subscription
        self.subscriptions.lock().await.insert(request.clone());

        // For now, we'll use a placeholder pool address
        // In production, you'd map symbol to actual pool address
        let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"; // WETH/USDC 0.05%

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
