//! # Deribit WebSocket Implementation
//!
//! WebSocket connector for Deribit using JSON-RPC 2.0 over WebSocket.
//!
//! ## Features
//! - Public and private channel subscriptions
//! - Automatic authentication (OAuth 2.0 over WebSocket)
//! - Heartbeat (test/ping every 30s)
//! - Broadcast channel pattern for event distribution
//! - JSON-RPC 2.0 message routing
//!
//! ## Architecture
//!
//! The WebSocket stream is split into independent read and write halves on connect.
//! The write half is stored behind a mutex for shared access by `send_request`,
//! `subscribe`, and the heartbeat task. The read half is owned exclusively by the
//! message loop task — no mutex contention on reads, which eliminates the deadlock
//! that occurred when both the reader and writer held the same mutex simultaneously.
//!
//! ## Usage
//!
//! ```ignore
//! let mut ws = DeribitWebSocket::new(Some(credentials), false, AccountType::FuturesCross).await?;
//! ws.connect(AccountType::FuturesCross).await?;
//! ws.subscribe_ticker(Symbol::new("BTC", "USD")).await?;
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use std::sync::Mutex as StdMutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    Credentials, AccountType,
    ExchangeResult, ExchangeError,
    ConnectionStatus, StreamEvent, StreamType, SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities, WsBookChannel};
use crate::core::traits::WebSocketConnector;

use super::endpoints::DeribitUrls;
use super::auth::DeribitAuth;
use super::parser::DeribitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by send_request, subscribe, and heartbeat
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Deribit WebSocket connector
pub struct DeribitWebSocket {
    /// Authentication handler
    auth: Option<DeribitAuth>,
    /// URLs (mainnet/testnet)
    urls: DeribitUrls,
    /// Testnet mode
    _testnet: bool,
    /// Current account type
    _account_type: AccountType,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Event broadcast sender — uses std::sync::Mutex so `subscribe()` can be called
    /// lock-free from `event_stream()` without contending with the async message loop.
    event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by send_request, subscribe, and heartbeat task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_writer: Arc<Mutex<Option<WsSink>>>,
    /// Request ID counter
    request_id: Arc<Mutex<u64>>,
    /// Access token for authenticated requests
    access_token: Arc<Mutex<Option<String>>>,
    /// Last time a WS-level ping was sent (for RTT measurement)
    last_ping: Arc<Mutex<Instant>>,
    /// Round-trip time of the last WebSocket ping/pong in milliseconds
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl DeribitWebSocket {
    /// Create new WebSocket connector
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
        account_type: AccountType,
    ) -> ExchangeResult<Self> {
        let urls = if testnet {
            DeribitUrls::TESTNET
        } else {
            DeribitUrls::MAINNET
        };

        let auth = credentials
            .as_ref()
            .map(DeribitAuth::new)
            .transpose()?;

        Ok(Self {
            auth,
            urls,
            _testnet: testnet,
            _account_type: account_type,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            event_tx: Arc::new(StdMutex::new(None)),
            ws_writer: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(1)),
            access_token: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        })
    }

    /// Get next request ID
    async fn next_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    /// Build JSON-RPC request
    fn build_request(&self, id: u64, method: &str, params: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
    }

    /// Send JSON-RPC request over WebSocket.
    ///
    /// Only locks `ws_writer` — the reader half is owned separately by the
    /// message loop task, so there is no deadlock risk here.
    async fn send_request(&self, method: &str, params: Value) -> ExchangeResult<u64> {
        let id = self.next_id().await;
        let request = self.build_request(id, method, params);

        let mut writer_guard = self.ws_writer.lock().await;
        let writer = writer_guard.as_mut()
            .ok_or_else(|| ExchangeError::Network("WebSocket not connected".to_string()))?;

        let msg_text = serde_json::to_string(&request)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize request: {}", e)))?;

        writer.send(Message::Text(msg_text)).await
            .map_err(|e| ExchangeError::Network(format!("Failed to send message: {}", e)))?;

        Ok(id)
    }

    /// Authenticate via WebSocket
    async fn authenticate(&self) -> ExchangeResult<()> {
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("No credentials provided".to_string()))?;

        let params = auth.client_credentials_params();

        let params_json = serde_json::to_value(params)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize auth params: {}", e)))?;

        let _id = self.send_request("public/auth", params_json).await?;

        // The auth response (with access_token) is handled asynchronously in the message loop.

        Ok(())
    }

    /// Subscribe to channels
    async fn subscribe_channels(&self, channels: Vec<String>, is_private: bool) -> ExchangeResult<()> {
        let method = if is_private {
            "private/subscribe"
        } else {
            "public/subscribe"
        };

        let params = json!({
            "channels": channels
        });

        self.send_request(method, params).await?;
        Ok(())
    }

    /// Unsubscribe from channels
    async fn unsubscribe_channels(&self, channels: Vec<String>, is_private: bool) -> ExchangeResult<()> {
        let method = if is_private {
            "private/unsubscribe"
        } else {
            "public/unsubscribe"
        };

        let params = json!({
            "channels": channels
        });

        self.send_request(method, params).await?;
        Ok(())
    }

    /// Build channel name from subscription request
    fn build_channel_name(&self, request: &SubscriptionRequest) -> String {
        // Format symbol: BTC-PERPETUAL, ETH-PERPETUAL, etc.
        let instrument = if request.symbol.base.is_empty() {
            // Private channels don't need instrument
            String::new()
        } else {
            format!("{}-PERPETUAL", request.symbol.base.to_uppercase())
        };

        match &request.stream_type {
            StreamType::Ticker => format!("ticker.{}.100ms", instrument),
            StreamType::Trade => format!("trades.{}.100ms", instrument),
            StreamType::Orderbook => format!("book.{}.100ms", instrument),
            StreamType::OrderbookDelta => format!("book.{}.100ms", instrument),
            StreamType::Kline { interval } => {
                // Deribit uses chart.trades.{instrument}.{resolution}
                format!("chart.trades.{}.{}", instrument, interval)
            },
            StreamType::OrderUpdate => "user.orders.any.any.raw".to_string(),
            // BalanceUpdate: Deribit uses per-currency portfolio channels.
            // Subscribing to a single channel (e.g. user.portfolio.BTC) would miss
            // balances in ETH, USDC, USDT, SOL etc.  We return a comma-joined list of
            // the five major settlement currencies so the caller can fan out.
            // The subscribe() method sends all returned channels in one request.
            StreamType::BalanceUpdate => {
                "user.portfolio.BTC,user.portfolio.ETH,user.portfolio.USDC,user.portfolio.USDT,user.portfolio.SOL".to_string()
            }
            StreamType::PositionUpdate => "user.changes.any.any.raw".to_string(),
            // OptionGreeks: subscribe to ticker.<instrument>.100ms — same channel that
            // carries the `greeks` field for option instruments.
            StreamType::OptionGreeks => format!("ticker.{}.100ms", instrument),
            _ => String::new(),
        }
    }

    /// Check if subscription is private
    fn is_private_subscription(&self, request: &SubscriptionRequest) -> bool {
        matches!(
            request.stream_type,
            StreamType::OrderUpdate | StreamType::BalanceUpdate | StreamType::PositionUpdate
        )
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// `ws_writer` is passed separately so the loop can send heartbeat replies
    /// without touching the reader.
    ///
    /// The loop runs until the WebSocket connection closes or errors naturally.
    /// There is no shutdown channel — the loop exits when the connection drops,
    /// which is the correct behaviour. Keeping a shutdown sender in the struct
    /// would cause the receiver to see a closed channel immediately when the
    /// struct is dropped (e.g. in bridge.rs after calling `event_stream()`),
    /// terminating the loop before any events are delivered.
    fn start_message_loop(
        mut reader: WsReader,
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        access_token: Arc<Mutex<Option<String>>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse JSON-RPC message
                        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                            // Check if it's an auth response
                            if let Some(result) = parsed.get("result") {
                                if let Some(token) = result.get("access_token") {
                                    if let Some(token_str) = token.as_str() {
                                        let mut token_guard = access_token.lock().await;
                                        *token_guard = Some(token_str.to_string());
                                    }
                                }
                            }

                            // Check if it's a test_request heartbeat from server.
                            // Deribit sends: {"method": "heartbeat", "params": {"type": "test_request"}, "id": <N>}
                            // Client MUST reply with public/test echoing the same id, or Deribit
                            // closes the connection after ~10 seconds.
                            if let Some(method) = parsed.get("method") {
                                if method == "heartbeat" {
                                    let is_test_request = parsed
                                        .get("params")
                                        .and_then(|p| p.get("type"))
                                        .and_then(|t| t.as_str())
                                        == Some("test_request");

                                    if is_test_request {
                                        // Echo back the original id so Deribit accepts the reply.
                                        let original_id = parsed
                                            .get("id")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0);

                                        let response = json!({
                                            "jsonrpc": "2.0",
                                            "id": original_id,
                                            "method": "public/test"
                                        });

                                        if let Ok(response_text) = serde_json::to_string(&response) {
                                            let mut writer_guard = ws_writer.lock().await;
                                            if let Some(ref mut writer) = *writer_guard {
                                                let _ = writer.send(Message::Text(response_text)).await;
                                            }
                                        }
                                    }
                                } else if method == "subscription" {
                                    // Parse and broadcast all events (may be multiple per message)
                                    let events = Self::parse_events(&parsed);
                                    if !events.is_empty() {
                                        let tx_guard = event_tx.lock().unwrap();
                                        if let Some(ref tx) = *tx_guard {
                                            for event in events {
                                                let _ = tx.send(Ok(event));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Record RTT for the WS-level ping sent by start_ws_ping_task
                        let rtt = last_ping.lock().await.elapsed().as_millis() as u64;
                        *ws_ping_rtt_ms.lock().await = rtt;
                    }
                    Ok(Message::Close(_)) => {
                        *status.lock().await = ConnectionStatus::Disconnected;
                        break;
                    }
                    Err(e) => {
                        let tx_guard = event_tx.lock().unwrap();
                        if let Some(ref tx) = *tx_guard {
                            let _ = tx.send(Err(WebSocketError::ConnectionError(format!("WebSocket error: {}", e))));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            // from .next(). Without this, a clean close leaves the sender alive
            // and the bridge hangs forever instead of reconnecting.
            let _ = event_tx.lock().unwrap().take();
            // Stream exhausted — connection closed
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Parse all events from a JSON-RPC subscription notification.
    ///
    /// `ticker.*` on perpetuals emits Ticker + FundingRate (current_funding) + MarkPrice.
    /// `deribit_price_index.*` emits IndexPrice.
    fn parse_events(msg: &Value) -> Vec<StreamEvent> {
        let params = match msg.get("params") {
            Some(p) => p,
            None => return vec![],
        };
        let channel = match params.get("channel").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => return vec![],
        };
        let data = match params.get("data") {
            Some(d) => d,
            None => return vec![],
        };

        if channel.starts_with("ticker.") {
            let mut events: Vec<StreamEvent> = Vec::with_capacity(5);

            if let Ok(ticker) = DeribitParser::parse_ws_ticker(data) {
                let symbol = ticker.symbol.clone();
                let timestamp = ticker.timestamp;
                events.push(StreamEvent::Ticker(ticker));

                // Emit FundingRate from current_funding (perpetuals only)
                let get_f64 = |key: &str| -> Option<f64> {
                    data.get(key).and_then(|v| v.as_f64())
                };
                if let Some(rate) = get_f64("current_funding") {
                    events.push(StreamEvent::FundingRate {
                        symbol: symbol.clone(),
                        rate,
                        next_funding_time: None,
                        timestamp,
                    });
                }

                // Emit MarkPrice from mark_price
                if let Some(mark) = get_f64("mark_price") {
                    let index = get_f64("index_price");
                    events.push(StreamEvent::MarkPrice {
                        symbol: symbol.clone(),
                        mark_price: mark,
                        index_price: index,
                        timestamp,
                    });
                }

                // Emit OptionGreeks if this is an option instrument.
                // Option names match: <CURRENCY>-<DDMMMYY>-<STRIKE>-C or -P
                // e.g. BTC-27DEC24-50000-C, ETH-29MAR24-2000-P
                // Also emit if the `greeks` field is present in the data.
                let has_greeks_field = data.get("greeks").is_some();
                let is_option_name = {
                    let s = &symbol;
                    s.ends_with("-C") || s.ends_with("-P")
                };
                if has_greeks_field || is_option_name {
                    let greeks = data.get("greeks");
                    let gf64 = |obj: Option<&serde_json::Value>, key: &str| -> Option<f64> {
                        obj.and_then(|g| g.get(key)).and_then(|v| v.as_f64())
                    };
                    let delta = gf64(greeks, "delta");
                    let gamma = gf64(greeks, "gamma");
                    let vega = gf64(greeks, "vega");
                    let theta = gf64(greeks, "theta");
                    let rho = gf64(greeks, "rho");
                    let mark_iv = get_f64("mark_iv");
                    let bid_iv = get_f64("bid_iv");
                    let ask_iv = get_f64("ask_iv");
                    events.push(StreamEvent::OptionGreeks {
                        symbol,
                        delta,
                        gamma,
                        vega,
                        theta,
                        rho,
                        mark_iv,
                        bid_iv,
                        ask_iv,
                        timestamp,
                    });
                }
            }

            events
        } else if channel.starts_with("quote.") {
            // Deribit quote.<instrument> — high-frequency best bid/ask.
            // data: { best_bid_price, best_bid_amount, best_ask_price,
            //         best_ask_amount, instrument_name, timestamp }
            let instrument = data.get("instrument_name")
                .and_then(|v| v.as_str())
                .unwrap_or(channel.strip_prefix("quote.").unwrap_or(channel));
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let bid_price = data.get("best_bid_price").and_then(|v| v.as_f64());
            let ask_price = data.get("best_ask_price").and_then(|v| v.as_f64());

            use crate::core::types::Ticker as TickerData;
            let ticker = TickerData {
                symbol: instrument.to_string(),
                bid_price,
                ask_price,
                last_price: bid_price.unwrap_or(0.0),
                volume_24h: None,
                high_24h: None,
                low_24h: None,
                price_change_24h: None,
                price_change_percent_24h: None,
                quote_volume_24h: None,
                timestamp,
            };
            vec![StreamEvent::Ticker(ticker)]
        } else if channel.starts_with("estimated_expiration_price.") {
            // Deribit estimated_expiration_price.<index> — settlement estimate.
            // data: { price, index_name, timestamp }  (same shape as deribit_price_index)
            let price = data.get("price").and_then(|v| v.as_f64());
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let index_name = data.get("index_name")
                .and_then(|v| v.as_str())
                .unwrap_or(
                    channel.strip_prefix("estimated_expiration_price.").unwrap_or(channel)
                );
            if let Some(px) = price {
                vec![StreamEvent::IndexPrice {
                    symbol: index_name.to_string(),
                    price: px,
                    timestamp,
                }]
            } else {
                vec![]
            }
        } else if channel.starts_with("deribit_price_index.") {
            // data: { "timestamp": 1234, "price": 9050.25, "index_name": "btc_usd" }
            let price = data.get("price").and_then(|v| v.as_f64());
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let index_name = data.get("index_name")
                .and_then(|v| v.as_str())
                .unwrap_or(channel.strip_prefix("deribit_price_index.").unwrap_or(channel));

            if let Some(px) = price {
                vec![StreamEvent::IndexPrice {
                    symbol: index_name.to_string(),
                    price: px,
                    timestamp,
                }]
            } else {
                vec![]
            }
        } else if channel.starts_with("perpetual.") {
            // Deribit perpetual.<instrument>.<interval> — interest rate for perpetuals.
            // data: { interest_rate, timestamp, instrument_name }
            // Emit as FundingRate (Deribit calls it interest_rate, equivalent concept).
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let instrument = data.get("instrument_name")
                .and_then(|v| v.as_str())
                .unwrap_or(channel.split('.').nth(1).unwrap_or(""));
            if let Some(rate) = data.get("interest_rate").and_then(|v| v.as_f64()) {
                vec![StreamEvent::FundingRate {
                    symbol: instrument.to_string(),
                    rate,
                    next_funding_time: None,
                    timestamp,
                }]
            } else {
                vec![]
            }
        } else if channel.starts_with("book.") {
            DeribitParser::parse_ws_orderbook(data).ok().into_iter().collect()
        } else if channel.starts_with("trades.") {
            DeribitParser::parse_ws_trade(data).ok()
                .map(StreamEvent::Trade)
                .into_iter()
                .collect()
        } else if channel.starts_with("user.orders.") {
            DeribitParser::parse_ws_order_update(data).ok()
                .map(StreamEvent::OrderUpdate)
                .into_iter()
                .collect()
        } else if channel.starts_with("user.portfolio.") {
            // data is the portfolio object: {"currency":"BTC","equity":...,"balance":...}
            let currency = channel.strip_prefix("user.portfolio.").unwrap_or("");
            let balance_val = |key: &str| -> f64 {
                data.get(key)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };
            let total = balance_val("equity");
            let available = balance_val("available_funds");
            let event = crate::core::BalanceUpdateEvent {
                asset: currency.to_string(),
                free: available,
                locked: (total - available).max(0.0),
                total,
                delta: None,
                reason: Some(crate::core::BalanceChangeReason::Other),
                timestamp: Utc::now().timestamp_millis(),
            };
            vec![StreamEvent::BalanceUpdate(event)]
        } else if channel.starts_with("deribit_volatility_index.") {
            // Deribit deribit_volatility_index.<index_name> — DVOL index value.
            // data: { "index_name": "btc_usd", "volatility": 62.5, "timestamp": 1234567890 }
            let index_name = data.get("index_name")
                .and_then(|v| v.as_str())
                .unwrap_or(
                    channel.strip_prefix("deribit_volatility_index.").unwrap_or(channel)
                );
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            if let Some(value) = data.get("volatility").and_then(|v| v.as_f64()) {
                vec![StreamEvent::VolatilityIndex {
                    symbol: index_name.to_string(),
                    value,
                    timestamp,
                }]
            } else {
                vec![]
            }
        } else if channel.starts_with("markprice.options.") {
            // Deribit markprice.options.<index_name> — array of option mark prices.
            // data is an array: [{ "instrument_name": "BTC-27DEC24-50000-C",
            //                      "mark_price": 0.015, "mark_iv": 62.5,
            //                      "timestamp": 1234 }, ...]
            if let Some(arr) = data.as_array() {
                arr.iter().filter_map(|item| {
                    let symbol = item.get("instrument_name")
                        .and_then(|v| v.as_str())?
                        .to_string();
                    let mark = item.get("mark_price").and_then(|v| v.as_f64())?;
                    let timestamp = item.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
                    Some(StreamEvent::MarkPrice {
                        symbol,
                        mark_price: mark,
                        index_price: None,
                        timestamp,
                    })
                }).collect()
            } else {
                vec![]
            }
        } else if channel == "block_trade_confirmations" {
            // Deribit block_trade_confirmations — confirmed block trades.
            // data: { "trade_id": "BT-123", "instrument_name": "BTC-PERPETUAL",
            //         "price": 81000.0, "amount": 10.0, "direction": "buy",
            //         "block_trade_id": "BT-123", "iv": null, "timestamp": 1234 }
            let symbol = data.get("instrument_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let block_id = data.get("block_trade_id")
                .or_else(|| data.get("trade_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let price = data.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let quantity = data.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let timestamp = data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let is_iv = data.get("iv").and_then(|v| v.as_f64()).is_some();
            let side = match data.get("direction").and_then(|v| v.as_str()) {
                Some("buy") => crate::core::types::TradeSide::Buy,
                _ => crate::core::types::TradeSide::Sell,
            };
            if !symbol.is_empty() {
                vec![StreamEvent::BlockTrade {
                    symbol,
                    block_id,
                    price,
                    quantity,
                    side,
                    timestamp,
                    is_iv,
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Start WS-level ping task for RTT measurement (every 5 seconds).
    ///
    /// Separate from the JSON heartbeat task — sends `Message::Ping` frames so
    /// the server responds with `Message::Pong`, allowing RTT measurement.
    fn start_ws_ping_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            // Skip immediate first tick
            interval.tick().await;

            loop {
                interval.tick().await;

                let mut writer_guard = ws_writer.lock().await;
                if let Some(ref mut writer) = *writer_guard {
                    *last_ping.lock().await = Instant::now();
                    if writer.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    /// Start heartbeat task (send public/test every 30 seconds).
    ///
    /// Uses only `ws_writer` — no contention with the reader half.
    ///
    /// The task exits naturally when the writer send fails (connection closed).
    /// No shutdown channel is used — see `start_message_loop` for rationale.
    fn start_heartbeat_task(
        ws_writer: Arc<Mutex<Option<WsSink>>>,
        request_id: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            // Skip the immediate first tick so we don't send a heartbeat before
            // the connection is fully established.
            interval.tick().await;

            loop {
                interval.tick().await;

                let id = {
                    let mut id_guard = request_id.lock().await;
                    let current = *id_guard;
                    *id_guard += 1;
                    current
                };

                let test_msg = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "public/test"
                });

                if let Ok(msg_text) = serde_json::to_string(&test_msg) {
                    let mut writer_guard = ws_writer.lock().await;
                    if let Some(ref mut writer) = *writer_guard {
                        if writer.send(Message::Text(msg_text)).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });
    }
}

impl DeribitWebSocket {
    /// Subscribe to the Deribit volatility index channel for an index (e.g. `"btc_usd"`).
    ///
    /// Channel name: `deribit_volatility_index.<index_name>`.
    /// Events arrive as `StreamEvent::VolatilityIndex`.
    pub async fn subscribe_volatility_index(&mut self, index_name: &str) -> ExchangeResult<()> {
        let channel = format!("deribit_volatility_index.{}", index_name);
        self.subscribe_channels(vec![channel], false).await
    }

    /// Subscribe to mark prices for all options on an index (e.g. `"btc_usd"`).
    ///
    /// Channel name: `markprice.options.<index_name>`.
    /// Events arrive as `StreamEvent::MarkPrice` per option instrument.
    pub async fn subscribe_options_mark_prices(&mut self, index_name: &str) -> ExchangeResult<()> {
        let channel = format!("markprice.options.{}", index_name);
        self.subscribe_channels(vec![channel], false).await
    }

    /// Subscribe to the public block trade confirmations channel.
    ///
    /// Channel name: `block_trade_confirmations`.
    /// Events arrive as `StreamEvent::BlockTrade`.
    pub async fn subscribe_block_trades(&mut self) -> ExchangeResult<()> {
        self.subscribe_channels(vec!["block_trade_confirmations".to_string()], false).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for DeribitWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Update status
        *self.status.lock().await = ConnectionStatus::Connecting;

        // Connect to WebSocket
        let ws_url = self.urls.ws_url();
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("Failed to connect: {}", e)))?;

        // Split into independent read and write halves.
        // The write half goes behind a mutex for shared use.
        // The read half is passed directly to the message loop — no mutex needed.
        let (write, read) = ws_stream.split();
        *self.ws_writer.lock().await = Some(write);

        // Create event broadcast channel
        let (tx, _) = broadcast::channel(1000);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Authenticate if credentials present.
        // Must happen after ws_writer is stored but before we start the read loop,
        // because authenticate() calls send_request() which uses ws_writer.
        if self.auth.is_some() {
            self.authenticate().await
                .map_err(|e| WebSocketError::Auth(format!("Authentication failed: {}", e)))?;
        }

        // Start message loop — reader is moved in, never shared via mutex.
        // The loop runs until the connection closes naturally; no shutdown channel
        // is needed or used.
        Self::start_message_loop(
            read,
            self.ws_writer.clone(),
            self.event_tx.clone(),
            self.status.clone(),
            self.access_token.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start WS-level ping task for RTT measurement
        Self::start_ws_ping_task(
            self.ws_writer.clone(),
            self.last_ping.clone(),
        );

        // Start heartbeat task — uses ws_writer only.
        // Exits naturally when the connection drops.
        Self::start_heartbeat_task(
            self.ws_writer.clone(),
            self.request_id.clone(),
        );

        // Update status
        *self.status.lock().await = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        // Close the write half. The message loop task owns the read half and will
        // detect the close frame / stream termination naturally and exit on its own.
        // The heartbeat task will fail on its next send attempt and also exit.
        if let Some(mut writer) = self.ws_writer.lock().await.take() {
            let _ = writer.close().await;
        }

        // Update status
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Clear subscriptions
        self.subscriptions.lock().await.clear();

        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        // Note: We need to use blocking here since the trait method is not async
        // In production, we'd use a different pattern (e.g., Arc<AtomicU8>)
        match self.status.try_lock() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Check connection
        if self.connection_status() != ConnectionStatus::Connected {
            return Err(WebSocketError::NotConnected);
        }

        // Build channel name
        let channel = self.build_channel_name(&request);

        if channel.is_empty() {
            return Err(WebSocketError::Subscription("Unsupported stream type".to_string()));
        }

        // Channel may be a comma-joined list (e.g. BalanceUpdate → multiple portfolio channels)
        let channels: Vec<String> = channel.split(',').map(|s| s.trim().to_string()).collect();

        // Subscribe
        let is_private = self.is_private_subscription(&request);
        self.subscribe_channels(channels, is_private).await
            .map_err(|e| WebSocketError::Subscription(format!("Subscribe failed: {}", e)))?;

        // Track subscription
        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        // Build channel name
        let channel = self.build_channel_name(&request);

        if channel.is_empty() {
            return Ok(());
        }

        // Channel may be a comma-joined list (e.g. BalanceUpdate → multiple portfolio channels)
        let channels: Vec<String> = channel.split(',').map(|s| s.trim().to_string()).collect();

        // Unsubscribe
        let is_private = self.is_private_subscription(&request);
        self.unsubscribe_channels(channels, is_private).await
            .map_err(|e| WebSocketError::Subscription(format!("Unsubscribe failed: {}", e)))?;

        // Remove from tracking
        self.subscriptions.lock().await.remove(&request);

        Ok(())
    }

    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        // std::sync::Mutex::lock() never contends long — `send()` and `subscribe()` are
        // both instant operations.  This replaces the old tokio try_lock() which would
        // return an empty stream whenever the message loop held the lock (i.e. almost
        // always, at 100 ms ticker frequency).
        let tx_guard = self.event_tx.lock().unwrap();

        if let Some(ref tx) = *tx_guard {
            let rx = tx.subscribe();
            Box::pin(tokio_stream::wrappers::BroadcastStream::new(rx).map(|r| {
                r.map_err(|e| WebSocketError::ConnectionError(format!("Broadcast error: {}", e)))
                    .and_then(|x| x)
            }))
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }

    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_lock() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static DERIBIT_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("book.{instr}.{group}.{depth}.100ms", None, Some(100)),
            WsBookChannel::delta("book.{instr}.{group}.{depth}.agg2",  None, None    ),
            WsBookChannel::delta("book.{instr}.{group}.{depth}.raw",   None, None    ).with_auth_tier(),
        ];
        OrderbookCapabilities {
            ws_depths: &[1, 10, 20],
            ws_default_depth: Some(20),
            rest_max_depth: Some(10000),
            rest_depth_values: &[1, 5, 10, 20, 50, 100, 1000, 10000],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[100],
            default_speed_ms: Some(100),
            ws_channels: DERIBIT_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: true,
            supports_aggregation: true,
            aggregation_levels: &["none", "1", "2", "5", "10", "25", "100", "250"],
        }
    }
}
