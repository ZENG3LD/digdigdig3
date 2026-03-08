//! # GMX WebSocket Implementation (Polling-Based)
//!
//! GMX does not provide a native WebSocket API. This module implements
//! a polling-based solution that mimics WebSocket behavior by periodically
//! fetching data from the REST API and broadcasting it via channels.
//!
//! ## Features
//! - Ticker polling (2-5 second intervals)
//! - Kline polling (based on interval)
//! - Broadcast channel for event distribution
//! - Automatic reconnection on errors
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = GmxWebSocket::new("arbitrum").await?;
//! ws.connect(AccountType::FuturesCross).await?;
//! ws.subscribe_ticker(Symbol::new("ETH", "USD")).await?;
//!
//! let mut stream = ws.event_stream();
//! while let Some(event) = stream.recv().await {
//!     match event {
//!         Ok(StreamEvent::Ticker { symbol, last_price, .. }) => {
//!             println!("{}: {}", symbol, last_price);
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, sleep};

use crate::core::{
    AccountType, Symbol, ExchangeResult,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError};
use crate::core::traits::{WebSocketConnector, MarketData};

use super::connector::GmxConnector;
use super::endpoints::format_symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// GMX WEBSOCKET (POLLING-BASED)
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX WebSocket connector (polling-based)
pub struct GmxWebSocket {
    /// Underlying REST connector
    connector: Arc<GmxConnector>,
    /// Broadcast channel for events
    event_tx: broadcast::Sender<WebSocketResult<StreamEvent>>,
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashSet<String>>>,
    /// Polling task handles
    #[allow(dead_code)]
    tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl GmxWebSocket {
    /// Create new WebSocket connector
    pub async fn new(chain: Option<String>) -> ExchangeResult<Self> {
        let connector = Arc::new(GmxConnector::new(chain).await?);
        let (event_tx, _) = broadcast::channel(1000);
        let status = Arc::new(RwLock::new(ConnectionStatus::Disconnected));
        let subscriptions = Arc::new(RwLock::new(HashSet::new()));
        let tasks = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            connector,
            event_tx,
            status,
            subscriptions,
            tasks,
        })
    }

    /// Get event stream receiver (for manual subscription)
    /// Most users should use the WebSocketConnector trait's event_stream() instead
    #[allow(dead_code)]
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<WebSocketResult<StreamEvent>> {
        self.event_tx.subscribe()
    }

    /// Poll ticker at regular intervals
    async fn poll_ticker_loop(
        connector: Arc<GmxConnector>,
        symbol: Symbol,
        event_tx: broadcast::Sender<Result<StreamEvent, WebSocketError>>,
        subscriptions: Arc<RwLock<HashSet<String>>>,
    ) {
        let sub_key = format!("ticker:{}/{}", symbol.base, symbol.quote);
        let mut ticker_interval = interval(Duration::from_secs(3)); // 3-second polling

        loop {
            ticker_interval.tick().await;

            // Check if still subscribed
            {
                let subs = subscriptions.read().await;
                if !subs.contains(&sub_key) {
                    break; // Unsubscribed, exit loop
                }
            }

            // Fetch ticker using MarketData trait method
            match MarketData::get_ticker(&*connector, symbol.clone(), AccountType::FuturesCross).await {
                Ok(ticker) => {
                    let event = StreamEvent::Ticker(ticker);

                    // Broadcast event (ignore send errors if no receivers)
                    let _ = event_tx.send(Ok(event));
                }
                Err(e) => {
                    eprintln!("GMX Ticker poll failed for {}: {}", sub_key, e);
                    // Don't send error events, just log and continue
                    sleep(Duration::from_secs(5)).await; // Backoff on error
                }
            }
        }
    }

    /// Poll klines at regular intervals
    async fn poll_kline_loop(
        connector: Arc<GmxConnector>,
        symbol: Symbol,
        interval_str: String,
        event_tx: broadcast::Sender<Result<StreamEvent, WebSocketError>>,
        subscriptions: Arc<RwLock<HashSet<String>>>,
    ) {
        let sub_key = format!("kline:{}:{}",  format_symbol(&symbol.base, &symbol.quote, AccountType::FuturesCross), interval_str);

        // Poll interval based on kline interval
        let poll_duration = match interval_str.as_str() {
            "1m" => Duration::from_secs(10),
            "5m" => Duration::from_secs(30),
            "15m" => Duration::from_secs(60),
            "1h" => Duration::from_secs(120),
            "4h" => Duration::from_secs(300),
            "1d" => Duration::from_secs(600),
            _ => Duration::from_secs(60),
        };

        let mut kline_interval = interval(poll_duration);

        loop {
            kline_interval.tick().await;

            // Check if still subscribed
            {
                let subs = subscriptions.read().await;
                if !subs.contains(&sub_key) {
                    break;
                }
            }

            // Fetch latest kline using MarketData trait method
            match MarketData::get_klines(&*connector, symbol.clone(), &interval_str, Some(1), AccountType::FuturesCross, None).await {
                Ok(mut klines) => {
                    if let Some(kline) = klines.pop() {
                        let event = StreamEvent::Kline(kline);
                        let _ = event_tx.send(Ok(event));
                    }
                }
                Err(e) => {
                    eprintln!("GMX Kline poll failed for {}: {}", sub_key, e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

#[async_trait]
impl WebSocketConnector for GmxWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // "Connect" by setting status to connected
        // No actual WebSocket connection is made
        let mut status = self.status.write().await;
        *status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Stop all polling tasks by clearing subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            subs.clear();
        }

        // Update status
        let mut status = self.status.write().await;
        *status = ConnectionStatus::Disconnected;

        Ok(())
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Check if connected
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }
        drop(status); // Release the lock

        let sub_key = match &request.stream_type {
            StreamType::Ticker => {
                format!("ticker:{}/{}", request.symbol.base, request.symbol.quote)
            }
            StreamType::Trade => {
                return Err(WebSocketError::Subscription(
                    "GMX doesn't provide real-time trade streams".to_string()
                ));
            }
            StreamType::Kline { interval } => {
                format!("kline:{}:{}", format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross), interval)
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                return Err(WebSocketError::Subscription(
                    "GMX uses oracle pricing, not orderbooks".to_string()
                ));
            }
            _ => {
                return Err(WebSocketError::Subscription(
                    "Stream type not supported for GMX".to_string()
                ));
            }
        };

        // Add to subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            if subs.contains(&sub_key) {
                return Ok(()); // Already subscribed
            }
            subs.insert(sub_key.clone());
        }

        // Start polling task based on stream type
        match &request.stream_type {
            StreamType::Ticker => {
                let task = tokio::spawn(Self::poll_ticker_loop(
                    self.connector.clone(),
                    request.symbol.clone(),
                    self.event_tx.clone(),
                    self.subscriptions.clone(),
                ));

                let mut tasks = self.tasks.write().await;
                tasks.push(task);
            }
            StreamType::Kline { interval } => {
                let task = tokio::spawn(Self::poll_kline_loop(
                    self.connector.clone(),
                    request.symbol.clone(),
                    interval.clone(),
                    self.event_tx.clone(),
                    self.subscriptions.clone(),
                ));

                let mut tasks = self.tasks.write().await;
                tasks.push(task);
            }
            _ => {}
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let sub_key = match &request.stream_type {
            StreamType::Ticker => {
                format!("ticker:{}/{}", request.symbol.base, request.symbol.quote)
            }
            StreamType::Kline { interval } => {
                format!("kline:{}:{}", format_symbol(&request.symbol.base, &request.symbol.quote, AccountType::FuturesCross), interval)
            }
            _ => return Ok(()),
        };

        // Remove from subscriptions (polling task will exit automatically)
        let mut subs = self.subscriptions.write().await;
        subs.remove(&sub_key);

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Use try_read to avoid async in sync method
        match self.status.try_read() {
            Ok(status) => *status,
            Err(_) => ConnectionStatus::Disconnected, // Default to disconnected if can't acquire lock
        }
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        use futures_util::stream::unfold;

        let receiver = self.event_tx.subscribe();

        Box::pin(unfold(receiver, |mut rx| async move {
            match rx.recv().await {
                Ok(event) => Some((event, rx)),
                Err(_) => None,
            }
        }))
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        // Convert internal subscription keys back to SubscriptionRequest
        // This is a simplified implementation - may not preserve all original data
        match self.subscriptions.try_read() {
            Ok(subs) => {
                subs.iter()
                    .filter_map(|key| {
                        // Parse subscription keys back to requests
                        if let Some(ticker_symbol) = key.strip_prefix("ticker:") {
                            let parts: Vec<&str> = ticker_symbol.split('/').collect();
                            if parts.len() == 2 {
                                return Some(SubscriptionRequest::ticker(Symbol::new(parts[0], parts[1])));
                            }
                        }
                        None
                    })
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = GmxWebSocket::new(Some("arbitrum".to_string())).await.unwrap();
        assert!(matches!(ws.connection_status(), ConnectionStatus::Disconnected));
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut ws = GmxWebSocket::new(Some("arbitrum".to_string())).await.unwrap();

        ws.connect(AccountType::FuturesCross).await.unwrap();
        assert!(matches!(ws.connection_status(), ConnectionStatus::Connected));

        ws.disconnect().await.unwrap();
        assert!(matches!(ws.connection_status(), ConnectionStatus::Disconnected));
    }
}
