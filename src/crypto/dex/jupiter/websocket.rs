//! # Jupiter WebSocket Implementation
//!
//! Jupiter does NOT have a native WebSocket API.
//!
//! This implementation provides a **polling-based pseudo-WebSocket** that:
//! - Polls Jupiter Price API at configurable intervals (default: 2 seconds)
//! - Wraps polling in async stream to match WebSocketConnector interface
//! - Emits StreamEvent::Ticker on price updates
//!
//! ## Architecture
//!
//! Since Jupiter is an aggregator without native WebSocket:
//! 1. Use polling of Price API (every 1-2 seconds)
//! 2. Broadcast price updates to subscribers
//! 3. Maintain subscription state
//!
//! ## Alternative Approaches
//! 1. Solana RPC WebSocket for on-chain monitoring (complex, low-level)
//! 2. Third-party providers (bloXroute, Bitquery)

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use tokio::sync::{broadcast, Mutex};
use tokio::time::interval;

use crate::core::types::{
    AccountType, ConnectionStatus, StreamEvent, SubscriptionRequest,
    WebSocketError, WebSocketResult,
};
use crate::core::traits::WebSocketConnector;
use crate::core::{HttpClient, Ticker, Symbol};

use super::endpoints::{JupiterUrls, JupiterEndpoint, MintRegistry};
use super::auth::JupiterAuth;
use super::parser::JupiterParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default polling interval (2 seconds)
const DEFAULT_POLL_INTERVAL_MS: u64 = 2000;

/// Maximum number of symbols to request per API call
const MAX_SYMBOLS_PER_REQUEST: usize = 50;

// ═══════════════════════════════════════════════════════════════════════════════
// JUPITER WEBSOCKET (POLLING-BASED)
// ═══════════════════════════════════════════════════════════════════════════════

/// Jupiter WebSocket connector (polling-based)
///
/// This implementation polls the Jupiter Price API at regular intervals
/// and emits ticker updates via a broadcast channel.
pub struct JupiterWebSocket {
    /// HTTP client for API requests
    http: Arc<HttpClient>,
    /// Authentication (optional - for Price API access)
    auth: Option<JupiterAuth>,
    /// URLs
    urls: JupiterUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions (symbol -> mint address mapping)
    subscriptions: Arc<Mutex<HashMap<Symbol, String>>>,
    /// Broadcast channel for events (dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// Polling interval in milliseconds
    poll_interval_ms: u64,
    /// Polling task handle
    polling_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl JupiterWebSocket {
    /// Create new Jupiter WebSocket connector
    ///
    /// # Arguments
    /// * `api_key` - Optional API key (required for Price API)
    /// * `poll_interval_ms` - Optional polling interval in milliseconds (default: 2000)
    pub fn new(api_key: Option<String>, poll_interval_ms: Option<u64>) -> Self {
        let http = Arc::new(HttpClient::new(30_000).expect("Failed to create HTTP client"));
        let auth = api_key.map(JupiterAuth::new);
        let urls = JupiterUrls::MAINNET;

        Self {
            http,
            auth,
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            poll_interval_ms: poll_interval_ms.unwrap_or(DEFAULT_POLL_INTERVAL_MS),
            polling_task: Arc::new(Mutex::new(None)),
        }
    }

    /// Create connector with API key
    pub fn with_api_key(api_key: String) -> Self {
        Self::new(Some(api_key), None)
    }

    /// Create public connector (requires fallback to Quote API for each symbol)
    pub fn public() -> Self {
        Self::new(None, None)
    }

    /// Start polling task
    async fn start_polling(&self) {
        let http = self.http.clone();
        let auth = self.auth.clone();
        let urls = self.urls.clone();
        let status = self.status.clone();
        let subscriptions = self.subscriptions.clone();
        let broadcast_tx = self.broadcast_tx.clone();
        let poll_interval_ms = self.poll_interval_ms;

        let task = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(poll_interval_ms));

            loop {
                ticker.tick().await;

                // Check if still connected
                if *status.lock().await == ConnectionStatus::Disconnected {
                    break;
                }

                // Get current subscriptions
                let subs = subscriptions.lock().await.clone();

                if subs.is_empty() {
                    continue;
                }

                // Collect mint addresses
                let mint_addresses: Vec<String> = subs.values().cloned().collect();

                // Poll prices for all subscribed symbols
                match Self::poll_prices(&http, &auth, &urls, &mint_addresses).await {
                    Ok(tickers) => {
                        // Emit ticker events
                        if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                            for ticker in tickers {
                                let _ = tx.send(Ok(StreamEvent::Ticker(ticker)));
                            }
                        }
                    }
                    Err(e) => {
                        // Emit error event
                        if let Some(tx) = broadcast_tx.lock().unwrap().as_ref() {
                            let _ = tx.send(Err(e));
                        }
                    }
                }
            }
        });

        *self.polling_task.lock().await = Some(task);
    }

    /// Stop polling task
    async fn stop_polling(&self) {
        if let Some(task) = self.polling_task.lock().await.take() {
            task.abort();
        }
    }

    /// Poll prices from Jupiter API
    async fn poll_prices(
        http: &HttpClient,
        auth: &Option<JupiterAuth>,
        urls: &JupiterUrls,
        mint_addresses: &[String],
    ) -> WebSocketResult<Vec<Ticker>> {
        // Check if we have API key
        if auth.is_none() {
            return Err(WebSocketError::Auth(
                "API key required for Price API polling. Use JupiterWebSocket::with_api_key().".to_string()
            ));
        }

        // Build request URL
        let endpoint = JupiterEndpoint::Price;
        let url = endpoint.url(urls);

        // Build query params (max 50 symbols per request)
        let mut tickers = Vec::new();

        for chunk in mint_addresses.chunks(MAX_SYMBOLS_PER_REQUEST) {
            let ids = chunk.join(",");
            let query = format!("?ids={}", ids);
            let full_url = format!("{}{}", url, query);

            // Add auth headers
            let headers = if let Some(ref auth) = auth {
                auth.auth_headers()
            } else {
                HashMap::new()
            };

            // Make request
            let response = http
                .get_with_headers(&full_url, &HashMap::new(), &headers)
                .await
                .map_err(|e| WebSocketError::ConnectionError(format!("HTTP request failed: {}", e)))?;

            // Parse response
            JupiterParser::check_error(&response)
                .map_err(|e| WebSocketError::ProtocolError(format!("API error: {}", e)))?;

            // Parse tickers for each mint in chunk
            for mint in chunk {
                match JupiterParser::parse_ticker_from_price(&response, mint) {
                    Ok(ticker) => tickers.push(ticker),
                    Err(_) => {
                        // Skip symbols that failed to parse
                        continue;
                    }
                }
            }
        }

        Ok(tickers)
    }

    /// Convert Symbol to mint address
    fn symbol_to_mint(&self, symbol: &Symbol) -> WebSocketResult<String> {
        // Try to resolve quote symbol to mint address (base is what we're getting price for)
        let mint = if super::endpoints::is_valid_mint_address(&symbol.quote) {
            symbol.quote.clone()
        } else {
            MintRegistry::symbol_to_mint(&symbol.quote)
                .ok_or_else(|| {
                    WebSocketError::Subscription(format!(
                        "Unknown token symbol: {}. Use mint address instead.",
                        symbol.quote
                    ))
                })?
                .to_string()
        };

        Ok(mint)
    }
}

impl Default for JupiterWebSocket {
    fn default() -> Self {
        Self::public()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for JupiterWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Check if we have API key
        if self.auth.is_none() {
            return Err(WebSocketError::Auth(
                "API key required for Jupiter WebSocket (polling). Use JupiterWebSocket::with_api_key().".to_string()
            ));
        }

        *self.status.lock().await = ConnectionStatus::Connecting;

        // Create broadcast channel and store
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start polling task
        self.start_polling().await;

        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Stop polling
        self.stop_polling().await;

        let _ = self.broadcast_tx.lock().unwrap().take();

        // Clear subscriptions
        self.subscriptions.lock().await.clear();

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
        // Only support Ticker stream type
        if !matches!(request.stream_type, crate::core::types::StreamType::Ticker) {
            return Err(WebSocketError::UnsupportedOperation(
                "Jupiter WebSocket only supports Ticker stream. For other data, use REST API.".to_string()
            ));
        }

        // Convert symbol to mint address
        let mint = self.symbol_to_mint(&request.symbol)?;

        // Add to subscriptions
        self.subscriptions.lock().await.insert(request.symbol.clone(), mint);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Remove from subscriptions
        self.subscriptions.lock().await.remove(&request.symbol);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.broadcast_tx.lock().unwrap().as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| broadcast::channel(1).1);

        Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|result| async move {
            match result {
                Ok(event) => Some(event),
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                    Some(Err(WebSocketError::ConnectionError("Event stream lagged behind".to_string())))
                }
            }
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Use try_lock to avoid blocking
        match self.subscriptions.try_lock() {
            Ok(subs) => subs
                .keys()
                .map(|symbol| SubscriptionRequest::new(symbol.clone(), crate::core::types::StreamType::Ticker))
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
