//! # MOEX ISS WebSocket Implementation (STOMP Protocol)
//!
//! WebSocket connector for MOEX using STOMP (Simple Text Oriented Messaging Protocol)
//! over WebSocket. Provides streaming market data with 15-minute delay on free tier.
//!
//! ## STOMP Protocol
//! MOEX uses STOMP v1.2 over WebSocket. Frame format:
//! ```text
//! COMMAND\n
//! header1:value1\n
//! header2:value2\n
//! \n
//! body\x00
//! ```
//!
//! ## Connection Flow
//! 1. WebSocket connect to `wss://iss.moex.com/infocx/v3/websocket`
//! 2. Send STOMP CONNECT frame
//! 3. Receive CONNECTED frame
//! 4. Send SUBSCRIBE frames for market data topics
//! 5. Receive MESSAGE frames with JSON body
//! 6. Send periodic heartbeat (newline `\n`)
//!
//! ## Usage
//! ```ignore
//! let mut ws = MoexWebSocket::new_public();
//! ws.connect(AccountType::Spot).await?;
//! ws.subscribe(SubscriptionRequest::ticker(Symbol::new("SBER", "RUB"))).await?;
//!
//! let stream = ws.event_stream();
//! pin_mut!(stream);
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(StreamEvent::Ticker(ticker)) => println!("{:?}", ticker),
//!         _ => {}
//!     }
//! }
//! ```

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::{Message, client::IntoClientRequest, http::header}};

use crate::core::types::{
    AccountType, ConnectionStatus, OrderbookCapabilities, StreamEvent, StreamType,
    SubscriptionRequest, Symbol, Ticker, WebSocketError, WebSocketResult,
};
use crate::core::traits::WebSocketConnector;

use super::auth::MoexAuth;
use super::endpoints::{MoexEndpoints, format_symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// STOMP FRAME TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// STOMP command types we handle
#[derive(Debug, Clone, PartialEq, Eq)]
enum StompCommand {
    Connected,
    Message,
    Receipt,
    Error,
    Unknown(String),
}

/// A parsed STOMP frame
#[derive(Debug, Clone)]
struct StompFrame {
    command: StompCommand,
    headers: Vec<(String, String)>,
    body: String,
}

impl StompFrame {
    /// Parse a STOMP frame from a text string
    fn parse(text: &str) -> Option<Self> {
        // STOMP frames end with \x00 (null byte)
        let text = text.trim_end_matches('\0').trim();
        if text.is_empty() {
            return None;
        }

        let mut lines = text.lines();

        // First line is the command
        let command_str = lines.next()?.trim();
        let command = match command_str {
            "CONNECTED" => StompCommand::Connected,
            "MESSAGE" => StompCommand::Message,
            "RECEIPT" => StompCommand::Receipt,
            "ERROR" => StompCommand::Error,
            other => StompCommand::Unknown(other.to_string()),
        };

        // Parse headers until empty line
        let mut headers = Vec::new();
        let mut body_start = false;
        let mut body_lines = Vec::new();

        for line in lines {
            if body_start {
                body_lines.push(line);
            } else if line.is_empty() {
                body_start = true;
            } else if let Some((key, value)) = line.split_once(':') {
                headers.push((key.to_string(), value.to_string()));
            }
        }

        let body = body_lines.join("\n");

        Some(StompFrame {
            command,
            headers,
            body,
        })
    }

    /// Get a header value by key
    fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STOMP FRAME BUILDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a STOMP CONNECT frame
///
/// MOEX requires a non-standard `domain` header to specify the authentication realm:
/// - `domain:DEMO` with `login:guest`, `passcode:guest` for 15-minute delayed data
/// - `domain:passport` with real credentials for real-time data
///
/// **CRITICAL**: MOEX server requires `\n` line endings (LF), NOT `\r\n` (CRLF).
/// While STOMP spec allows both, MOEX rejects CRLF frames with immediate disconnect.
fn build_connect_frame(host: &str, login: Option<&str>, passcode: Option<&str>) -> String {
    // MOEX requires domain:DEMO + login:guest + passcode:guest even for anonymous access
    let domain = if login.is_some() && passcode.is_some() {
        "passport"
    } else {
        "DEMO"
    };

    let login = login.unwrap_or("guest");
    let passcode = passcode.unwrap_or("guest");

    // Use \n line endings (LF only) - MOEX server rejects \r\n (CRLF)
    format!(
        "CONNECT\ndomain:{}\nlogin:{}\npasscode:{}\naccept-version:1.2\nhost:{}\nheart-beat:10000,10000\n\n\0",
        domain, login, passcode, host
    )
}

/// Build a STOMP SUBSCRIBE frame
///
/// MOEX uses custom subscription format:
/// - `destination`: Stream name like `MXSE.securities`, `MXSE.orderbooks`
/// - `selector`: SQL-like WHERE clause like `TICKER="MXSE.TQBR.SBER"`
fn build_subscribe_frame(id: &str, destination: &str, selector: &str) -> String {
    format!(
        "SUBSCRIBE\nid:{}\ndestination:{}\nselector:{}\nack:auto\n\n\0",
        id, destination, selector
    )
}

/// Build a STOMP UNSUBSCRIBE frame
fn build_unsubscribe_frame(id: &str) -> String {
    format!("UNSUBSCRIBE\nid:{}\n\n\0", id)
}

/// Build a STOMP DISCONNECT frame
fn build_disconnect_frame() -> String {
    "DISCONNECT\nreceipt:disconnect-receipt\n\n\0".to_string()
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOPIC HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Map a subscription request to a MOEX STOMP destination and selector
///
/// MOEX InfoCX uses custom subscription format:
/// - `destination`: Stream name (e.g., `MXSE.securities`, `MXSE.orderbooks`)
/// - `selector`: SQL-like filter (e.g., `TICKER="MXSE.TQBR.SBER"`)
///
/// Returns: (destination, selector)
fn subscription_to_destination(symbol: &Symbol, stream_type: &StreamType) -> (&'static str, String) {
    let ticker = format_symbol(symbol);
    // MOEX ticker format: MXSE.{BOARD}.{SECID}
    let moex_ticker = format!("MXSE.TQBR.{}", ticker);

    match stream_type {
        StreamType::Ticker | StreamType::Trade => {
            // Both ticker and trade data come from MXSE.securities stream
            ("MXSE.securities", format!("TICKER=\"{}\"", moex_ticker))
        }
        StreamType::Orderbook | StreamType::OrderbookDelta => {
            // Orderbook data from MXSE.orderbooks stream
            ("MXSE.orderbooks", format!("TICKER=\"{}\"", moex_ticker))
        }
        StreamType::Kline { .. } => {
            // MOEX STOMP doesn't have native kline streams; use security data
            ("MXSE.securities", format!("TICKER=\"{}\"", moex_ticker))
        }
        _ => {
            // Fallback to general security data
            ("MXSE.securities", format!("TICKER=\"{}\"", moex_ticker))
        }
    }
}

/// Generate a subscription ID from destination
fn subscription_id(request: &SubscriptionRequest) -> String {
    let ticker = format_symbol(&request.symbol);
    let stream_name = match &request.stream_type {
        StreamType::Ticker => "ticker",
        StreamType::Trade => "trade",
        StreamType::Orderbook => "orderbook",
        StreamType::OrderbookDelta => "orderbook-delta",
        StreamType::Kline { interval } => return format!("kline-{}-{}", ticker, interval),
        StreamType::MarkPrice => "mark-price",
        StreamType::FundingRate => "funding-rate",
        StreamType::OrderUpdate => "order-update",
        StreamType::BalanceUpdate => "balance-update",
        StreamType::PositionUpdate => "position-update",
    };
    format!("{}-{}", stream_name, ticker)
}

// ═══════════════════════════════════════════════════════════════════════════════
// MOEX WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal commands for the WebSocket loop
enum WsCommand {
    Subscribe(SubscriptionRequest),
    Unsubscribe(SubscriptionRequest),
    Disconnect,
}

/// MOEX WebSocket connector using STOMP protocol
///
/// This connector implements the `WebSocketConnector` trait for MOEX ISS streaming data.
/// It uses STOMP v1.2 protocol over WebSocket for real-time (or 15-min delayed) market data.
///
/// ## Notes
/// - Free tier provides 15-minute delayed data
/// - Real-time data requires MOEX ISS subscription credentials
/// - STOMP protocol is implemented manually (simple text-based protocol)
pub struct MoexWebSocket {
    /// Authentication credentials
    auth: MoexAuth,
    /// Endpoint configuration
    endpoints: MoexEndpoints,
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashSet<SubscriptionRequest>>>,
    /// Command sender for the WebSocket loop
    command_tx: Option<mpsc::UnboundedSender<WsCommand>>,
    /// Event receiver (wrapped for sharing)
    event_rx: Option<Arc<Mutex<mpsc::UnboundedReceiver<WebSocketResult<StreamEvent>>>>>,
    /// Debug mode
    debug: bool,
}

impl MoexWebSocket {
    /// Create new MOEX WebSocket connector with authentication
    pub fn new(auth: MoexAuth) -> Self {
        let debug = std::env::var("DEBUG_WS").is_ok();

        Self {
            auth,
            endpoints: MoexEndpoints::default(),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            command_tx: None,
            event_rx: None,
            debug,
        }
    }

    /// Create public MOEX WebSocket connector (15-minute delayed data)
    pub fn new_public() -> Self {
        Self::new(MoexAuth::public())
    }

    /// Create connector from environment variables
    pub fn from_env() -> Self {
        Self::new(MoexAuth::from_env())
    }

    /// Parse STOMP MESSAGE body into StreamEvent
    fn parse_message_event(frame: &StompFrame, _debug: bool) -> Result<StreamEvent, String> {
        let destination = frame.header("destination").unwrap_or("");

        // Try to parse JSON body
        let json: Value = serde_json::from_str(&frame.body)
            .map_err(|e| format!("Failed to parse STOMP message body as JSON: {}", e))?;

        // Determine event type from destination
        if destination.contains("/trades") {
            Self::parse_trade_event(&json, destination)
        } else if destination.contains("/orderbook") {
            Self::parse_orderbook_event(&json, destination)
        } else {
            // Default: treat as ticker/security data
            Self::parse_ticker_event(&json, destination)
        }
    }

    /// Parse ticker/security data from MOEX STOMP message
    fn parse_ticker_event(json: &Value, destination: &str) -> Result<StreamEvent, String> {
        // MOEX WebSocket sends data in columns+data format (same as REST API)
        // Each data value can be either a simple value or a [value, precision] tuple.

        // Try columns+data format first (primary format for MOEX WebSocket)
        // Can be at top level or nested in "marketdata"
        let (columns_opt, data_opt) = if let Some(marketdata) = json.get("marketdata") {
            // REST API format: nested in marketdata
            (
                marketdata.get("columns").and_then(|c| c.as_array()),
                marketdata.get("data").and_then(|d| d.as_array()),
            )
        } else {
            // WebSocket format: top-level columns and data
            (
                json.get("columns").and_then(|c| c.as_array()),
                json.get("data").and_then(|d| d.as_array()),
            )
        };

        if let (Some(columns), Some(data)) = (columns_opt, data_opt) {
            if let Some(row) = data.first() {
                // Extract symbol from the TICKER column (first column)
                let row_arr = row.as_array().ok_or("Data row is not an array")?;
                let ticker_col = columns.iter().position(|c| c.as_str() == Some("TICKER"))
                    .ok_or("No TICKER column found")?;
                let ticker_value = row_arr.get(ticker_col)
                    .and_then(|v| v.as_str())
                    .ok_or("TICKER value is not a string")?;

                // MOEX ticker format: MXSE.TQBR.SBER -> extract SBER
                let symbol = ticker_value.split('.').next_back().unwrap_or(ticker_value);

                return Self::parse_ticker_from_columns_data(symbol, columns, row);
            }
        }

        // Fallback: try flat JSON format
        let symbol = json.get("SECID")
            .or_else(|| json.get("secid"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                // Last resort: extract from destination
                destination.rsplit('/').next().unwrap_or("UNKNOWN")
            });

        // Try flat JSON format
        let last_price = json.get("LAST")
            .or_else(|| json.get("last"))
            .or_else(|| json.get("price"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("No price field found in message for {}", symbol))?;

        let bid = json.get("BID").or_else(|| json.get("bid")).and_then(|v| v.as_f64());
        let ask = json.get("ASK").or_else(|| json.get("ask")).and_then(|v| v.as_f64());
        let high = json.get("HIGH").or_else(|| json.get("high")).and_then(|v| v.as_f64());
        let low = json.get("LOW").or_else(|| json.get("low")).and_then(|v| v.as_f64());
        let volume = json.get("VOLUME")
            .or_else(|| json.get("volume"))
            .and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)));
        let change = json.get("LASTCHANGE")
            .or_else(|| json.get("change"))
            .and_then(|v| v.as_f64());
        let change_pct = json.get("LASTCHANGEPRCNT")
            .or_else(|| json.get("change_pct"))
            .and_then(|v| v.as_f64());

        let timestamp = chrono::Utc::now().timestamp_millis();

        Ok(StreamEvent::Ticker(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: bid,
            ask_price: ask,
            high_24h: high,
            low_24h: low,
            volume_24h: volume,
            quote_volume_24h: None,
            price_change_24h: change,
            price_change_percent_24h: change_pct,
            timestamp,
        }))
    }

    /// Parse ticker from MOEX columns+data format (same as REST response)
    fn parse_ticker_from_columns_data(
        symbol: &str,
        columns: &[Value],
        row: &Value,
    ) -> Result<StreamEvent, String> {
        let row_arr = row.as_array().ok_or("Data row is not an array")?;

        let find_col = |name: &str| -> Option<usize> {
            columns.iter().position(|c| c.as_str() == Some(name))
        };

        // MOEX data can be either a simple value or [value, precision] array
        let get_f64 = |name: &str| -> Option<f64> {
            find_col(name).and_then(|i| row_arr.get(i)).and_then(|v| {
                // Try as direct number first
                v.as_f64().or_else(|| {
                    // Try as [value, precision] array
                    v.as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|val| val.as_f64())
                })
            })
        };

        // Get integer value (for volume which might be large)
        let get_volume = |name: &str| -> Option<f64> {
            find_col(name).and_then(|i| row_arr.get(i)).and_then(|v| {
                // Try as direct number
                v.as_f64()
                    .or_else(|| v.as_i64().map(|i| i as f64))
                    .or_else(|| {
                        // Try as [value, precision] array
                        v.as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|val| val.as_f64().or_else(|| val.as_i64().map(|i| i as f64)))
                    })
            })
        };

        let last_price = get_f64("LAST")
            .ok_or_else(|| "Missing LAST price in columns data".to_string())?;

        let timestamp = chrono::Utc::now().timestamp_millis();

        Ok(StreamEvent::Ticker(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price: get_f64("BID").or_else(|| get_f64("OFFER")),  // MOEX uses OFFER instead of ASK sometimes
            ask_price: get_f64("ASK").or_else(|| get_f64("OFFER")),
            high_24h: get_f64("HIGH"),
            low_24h: get_f64("LOW"),
            volume_24h: get_volume("VOLTODAY").or_else(|| get_volume("VOLUME")),
            quote_volume_24h: get_f64("VALTODAY").or_else(|| get_f64("VALUE")),
            price_change_24h: get_f64("CHANGE").or_else(|| get_f64("LASTCHANGE")),
            price_change_percent_24h: get_f64("LASTCHANGEPRCNT"),
            timestamp,
        }))
    }

    /// Parse trade event from MOEX STOMP message
    fn parse_trade_event(json: &Value, destination: &str) -> Result<StreamEvent, String> {
        let symbol = destination
            .split("/securities/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("UNKNOWN");

        // Try to extract trade data
        let price = json.get("PRICE")
            .or_else(|| json.get("price"))
            .and_then(|v| v.as_f64())
            .ok_or("Missing price in trade message")?;

        let quantity = json.get("QUANTITY")
            .or_else(|| json.get("quantity"))
            .or_else(|| json.get("SIZE"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let timestamp = chrono::Utc::now().timestamp_millis();

        // Use Ticker event as a carrier for trade data since PublicTrade
        // may require additional fields
        Ok(StreamEvent::Ticker(Ticker {
            symbol: symbol.to_string(),
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: Some(quantity),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        }))
    }

    /// Parse orderbook event from MOEX STOMP message
    fn parse_orderbook_event(_json: &Value, _destination: &str) -> Result<StreamEvent, String> {
        // Placeholder: orderbook parsing from STOMP
        // MOEX orderbook data may come as full snapshots
        let timestamp = chrono::Utc::now().timestamp_millis();

        Ok(StreamEvent::OrderbookSnapshot(
            crate::core::types::OrderBook {
                bids: Vec::new(),
                asks: Vec::new(),
                timestamp,
                sequence: None,
                last_update_id: None,
                first_update_id: None,
                prev_update_id: None,
                event_time: None,
                transaction_time: None,
                checksum: None,
            },
        ))
    }

    /// Handle a parsed STOMP frame (common logic for text and binary messages)
    async fn handle_stomp_frame(
        frame: StompFrame,
        event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
        status: &Arc<RwLock<ConnectionStatus>>,
        debug: bool,
    ) {
        match frame.command {
            StompCommand::Message => {
                if debug {
                    let dest = frame.header("destination").unwrap_or("<no dest>");
                    eprintln!("[MOEX-WS] MESSAGE frame received");
                    eprintln!("[MOEX-WS]   destination: {}", dest);
                    eprintln!("[MOEX-WS]   body length: {} bytes", frame.body.len());
                    let body_preview: String = frame.body.chars().take(200).collect();
                    eprintln!("[MOEX-WS]   body preview: {}", body_preview);
                }
                match Self::parse_message_event(&frame, debug) {
                    Ok(event) => {
                        if debug {
                            eprintln!("[MOEX-WS] Successfully parsed event: {:?}", event);
                        }
                        let _ = event_tx.send(Ok(event));
                    }
                    Err(e) => {
                        if debug {
                            eprintln!("[MOEX-WS] Parse error: {}", e);
                        }
                    }
                }
            }
            StompCommand::Error => {
                let error_msg = if frame.body.is_empty() {
                    frame.header("message").unwrap_or("Unknown error").to_string()
                } else {
                    frame.body.clone()
                };
                if debug {
                    eprintln!("[MOEX-WS] STOMP ERROR: {}", error_msg);
                }
                let _ = event_tx.send(Err(WebSocketError::ProtocolError(
                    format!("STOMP error: {}", error_msg),
                )));
            }
            StompCommand::Receipt => {
                if debug {
                    let receipt_id = frame.header("receipt-id").unwrap_or("unknown");
                    eprintln!("[MOEX-WS] Receipt: {}", receipt_id);
                }
                // Check for disconnect receipt
                if frame.header("receipt-id") == Some("disconnect-receipt") {
                    let mut guard = status.write().await;
                    *guard = ConnectionStatus::Disconnected;
                }
            }
            StompCommand::Connected => {
                if debug {
                    eprintln!("[MOEX-WS] Received CONNECTED frame in message loop (expected only during handshake)");
                }
            }
            _ => {
                if debug {
                    eprintln!("[MOEX-WS] Unexpected frame: {:?}", frame.command);
                }
            }
        }
    }

    /// Run the WebSocket message loop with STOMP protocol
    async fn run_stomp_loop(
        auth: MoexAuth,
        ws_url: String,
        debug: bool,
        status: Arc<RwLock<ConnectionStatus>>,
        subscriptions: Arc<RwLock<HashSet<SubscriptionRequest>>>,
        mut command_rx: mpsc::UnboundedReceiver<WsCommand>,
        event_tx: mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
    ) {
        // Update status to connecting
        {
            let mut guard = status.write().await;
            *guard = ConnectionStatus::Connecting;
        }

        if debug {
            eprintln!("[MOEX-WS] Connecting to {}", ws_url);
        }

        // Build WebSocket request with STOMP subprotocol header.
        // MOEX InfoCX requires `Sec-WebSocket-Protocol: v12.stomp` during the
        // WebSocket handshake. Without this header the server accepts the TCP
        // upgrade but never responds to STOMP frames.
        let ws_request = match ws_url.as_str().into_client_request() {
            Ok(mut req) => {
                req.headers_mut().insert(
                    header::SEC_WEBSOCKET_PROTOCOL,
                    "v12.stomp".parse().expect("valid header value"),
                );
                if debug {
                    eprintln!("[MOEX-WS] WebSocket request headers:");
                    for (key, value) in req.headers().iter() {
                        eprintln!("[MOEX-WS]   {:?}: {:?}", key, value);
                    }
                }
                req
            }
            Err(e) => {
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::ConnectionError(
                    format!("Failed to build WebSocket request: {}", e),
                )));
                return;
            }
        };

        // Connect WebSocket
        let ws_result = tokio::time::timeout(
            Duration::from_secs(15),
            connect_async(ws_request),
        )
        .await;

        let ws_stream = match ws_result {
            Ok(Ok((stream, response))) => {
                if debug {
                    eprintln!("[MOEX-WS] WebSocket handshake successful");
                    eprintln!("[MOEX-WS] Response status: {:?}", response.status());
                    eprintln!("[MOEX-WS] Response headers:");
                    for (key, value) in response.headers().iter() {
                        eprintln!("[MOEX-WS]   {:?}: {:?}", key, value);
                    }
                }
                stream
            },
            Ok(Err(e)) => {
                if debug {
                    eprintln!("[MOEX-WS] Connection failed: {}", e);
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::ConnectionError(
                    format!("WebSocket connection failed: {}", e),
                )));
                return;
            }
            Err(_) => {
                if debug {
                    eprintln!("[MOEX-WS] Connection timeout");
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::Timeout));
                return;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        // Send STOMP CONNECT frame
        let (login, passcode) = auth.credentials();
        let connect_frame = build_connect_frame(
            "iss.moex.com",
            login.as_deref(),
            passcode.as_deref(),
        );

        if debug {
            eprintln!("[MOEX-WS] Sending STOMP CONNECT");
            eprintln!("[MOEX-WS] Frame length: {} bytes", connect_frame.len());
            eprintln!("[MOEX-WS] Frame bytes: {:?}", connect_frame.as_bytes());
            eprintln!("[MOEX-WS] Frame (escaped): {}", connect_frame.escape_debug());
        }

        if let Err(e) = write.send(Message::Text(connect_frame)).await {
            let mut guard = status.write().await;
            *guard = ConnectionStatus::Disconnected;
            let _ = event_tx.send(Err(WebSocketError::ConnectionError(
                format!("Failed to send STOMP CONNECT: {}", e),
            )));
            return;
        }

        // Wait for CONNECTED response
        let connected = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if debug {
                            let preview: String = text.chars().take(200).collect();
                            eprintln!("[MOEX-WS] Received text frame: {:?}", preview);
                        }
                        // Skip empty heartbeat frames
                        let trimmed = text.trim_matches(|c: char| c == '\n' || c == '\r' || c == '\0');
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Some(frame) = StompFrame::parse(&text) {
                            return Some(frame);
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        if debug {
                            eprintln!("[MOEX-WS] Received binary frame ({} bytes)", data.len());
                        }
                        // MOEX sends STOMP frames as binary WebSocket messages
                        // Try to parse as UTF-8 text
                        if let Ok(text) = String::from_utf8(data) {
                            if debug {
                                let preview: String = text.chars().take(200).collect();
                                eprintln!("[MOEX-WS] Binary decoded as text: {:?}", preview);
                            }
                            let trimmed = text.trim_matches(|c: char| c == '\n' || c == '\r' || c == '\0');
                            if !trimmed.is_empty() {
                                if let Some(frame) = StompFrame::parse(&text) {
                                    return Some(frame);
                                }
                            }
                        }
                        continue;
                    }
                    Ok(Message::Ping(data)) => {
                        if debug {
                            eprintln!("[MOEX-WS] Received ping during handshake");
                        }
                        // Cannot send pong here since we don't have write access
                        // but continue waiting
                        let _ = data;
                        continue;
                    }
                    Ok(Message::Close(close_frame)) => {
                        if debug {
                            eprintln!("[MOEX-WS] Received Close frame during handshake: {:?}", close_frame);
                        }
                        // Server rejected connection
                        return None;
                    }
                    Ok(other) => {
                        if debug {
                            eprintln!("[MOEX-WS] Received unexpected frame type during handshake: {:?}", other);
                        }
                        continue;
                    }
                    Err(e) => {
                        if debug {
                            eprintln!("[MOEX-WS] Error waiting for CONNECTED: {}", e);
                        }
                        return None;
                    }
                }
            }
            None
        })
        .await;

        match connected {
            Ok(Some(frame)) if frame.command == StompCommand::Connected => {
                if debug {
                    let version = frame.header("version").unwrap_or("unknown");
                    let server = frame.header("server").unwrap_or("unknown");
                    eprintln!(
                        "[MOEX-WS] STOMP CONNECTED (version: {}, server: {})",
                        version, server
                    );
                }
            }
            Ok(Some(frame)) if frame.command == StompCommand::Error => {
                let error_msg = if frame.body.is_empty() {
                    frame.header("message").unwrap_or("Unknown error").to_string()
                } else {
                    frame.body.clone()
                };
                if debug {
                    eprintln!("[MOEX-WS] STOMP ERROR: {}", error_msg);
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::ConnectionError(
                    format!("STOMP connection error: {}", error_msg),
                )));
                return;
            }
            Ok(_) => {
                if debug {
                    eprintln!("[MOEX-WS] Unexpected response to STOMP CONNECT");
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::ProtocolError(
                    "Expected CONNECTED frame but got something else".to_string(),
                )));
                return;
            }
            Err(_) => {
                if debug {
                    eprintln!("[MOEX-WS] Timeout waiting for STOMP CONNECTED");
                }
                let mut guard = status.write().await;
                *guard = ConnectionStatus::Disconnected;
                let _ = event_tx.send(Err(WebSocketError::Timeout));
                return;
            }
        }

        // Successfully connected
        {
            let mut guard = status.write().await;
            *guard = ConnectionStatus::Connected;
        }

        // Restore any existing subscriptions
        {
            let subs = subscriptions.read().await;
            for sub in subs.iter() {
                let sub_id = subscription_id(sub);
                let (destination, selector) = subscription_to_destination(&sub.symbol, &sub.stream_type);
                let frame = build_subscribe_frame(&sub_id, destination, &selector);
                if let Err(e) = write.send(Message::Text(frame)).await {
                    if debug {
                        eprintln!("[MOEX-WS] Failed to restore subscription {}: {}", sub_id, e);
                    }
                }
            }
        }

        // Heartbeat interval (STOMP heartbeat is a single newline)
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(10));

        // Main message loop
        loop {
            tokio::select! {
                // Incoming messages from MOEX
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            // Check for heartbeat (empty or just newline)
                            let trimmed = text.trim_matches(|c: char| c == '\n' || c == '\r' || c == '\0');
                            if trimmed.is_empty() {
                                if debug {
                                    eprintln!("[MOEX-WS] Received heartbeat (text)");
                                }
                                continue;
                            }

                            if debug {
                                let preview: String = text.chars().take(300).collect();
                                eprintln!("[MOEX-WS] Received text message ({} bytes): {}", text.len(), preview);
                            }

                            if let Some(frame) = StompFrame::parse(&text) {
                                Self::handle_stomp_frame(frame, &event_tx, &status, debug).await;
                            } else if debug {
                                eprintln!("[MOEX-WS] Failed to parse text as STOMP frame");
                            }
                        }
                        Some(Ok(Message::Binary(data))) => {
                            if debug {
                                eprintln!("[MOEX-WS] Received binary message ({} bytes)", data.len());
                            }
                            // MOEX sends STOMP frames as binary WebSocket messages
                            if let Ok(text) = String::from_utf8(data) {
                                let trimmed = text.trim_matches(|c: char| c == '\n' || c == '\r' || c == '\0');
                                if !trimmed.is_empty() {
                                    if debug {
                                        let preview: String = text.chars().take(300).collect();
                                        eprintln!("[MOEX-WS] Binary decoded as text: {}", preview);
                                    }
                                    if let Some(frame) = StompFrame::parse(&text) {
                                        Self::handle_stomp_frame(frame, &event_tx, &status, debug).await;
                                    } else if debug {
                                        eprintln!("[MOEX-WS] Failed to parse binary as STOMP frame");
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Ok(Message::Close(_))) => {
                            if debug {
                                eprintln!("[MOEX-WS] Connection closed by server");
                            }
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return;
                        }
                        Some(Ok(_)) => {
                            // Pong, Frame - ignore
                        }
                        Some(Err(e)) => {
                            if debug {
                                eprintln!("[MOEX-WS] WebSocket error: {}", e);
                            }
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            let _ = event_tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                            return;
                        }
                        None => {
                            if debug {
                                eprintln!("[MOEX-WS] WebSocket stream ended");
                            }
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return;
                        }
                    }
                }

                // Commands from user
                cmd = command_rx.recv() => {
                    match cmd {
                        Some(WsCommand::Subscribe(req)) => {
                            let sub_id = subscription_id(&req);
                            let (destination, selector) = subscription_to_destination(&req.symbol, &req.stream_type);
                            let frame = build_subscribe_frame(&sub_id, destination, &selector);

                            if debug {
                                eprintln!("[MOEX-WS] SUBSCRIBE id={} dest={} selector={}", sub_id, destination, selector);
                                eprintln!("[MOEX-WS] SUBSCRIBE frame ({} bytes): {}", frame.len(), frame.escape_debug());
                            }

                            if let Err(e) = write.send(Message::Text(frame)).await {
                                let _ = event_tx.send(Err(WebSocketError::SendError(
                                    format!("Failed to send SUBSCRIBE: {}", e),
                                )));
                            } else {
                                subscriptions.write().await.insert(req);
                            }
                        }
                        Some(WsCommand::Unsubscribe(req)) => {
                            let sub_id = subscription_id(&req);
                            let frame = build_unsubscribe_frame(&sub_id);

                            if debug {
                                eprintln!("[MOEX-WS] UNSUBSCRIBE id={}", sub_id);
                            }

                            if let Err(e) = write.send(Message::Text(frame)).await {
                                let _ = event_tx.send(Err(WebSocketError::SendError(
                                    format!("Failed to send UNSUBSCRIBE: {}", e),
                                )));
                            } else {
                                subscriptions.write().await.remove(&req);
                            }
                        }
                        Some(WsCommand::Disconnect) => {
                            if debug {
                                eprintln!("[MOEX-WS] Disconnecting...");
                            }
                            let frame = build_disconnect_frame();
                            let _ = write.send(Message::Text(frame)).await;
                            // Wait briefly for receipt
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            let _ = write.close().await;
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return;
                        }
                        None => {
                            // Command channel closed
                            if debug {
                                eprintln!("[MOEX-WS] Command channel closed");
                            }
                            let mut guard = status.write().await;
                            *guard = ConnectionStatus::Disconnected;
                            return;
                        }
                    }
                }

                // Heartbeat timer (STOMP heartbeat = newline character)
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = write.send(Message::Text("\n".to_string())).await {
                        if debug {
                            eprintln!("[MOEX-WS] Failed to send heartbeat: {}", e);
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WebSocketConnector TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for MoexWebSocket {
    /// Connect to MOEX STOMP WebSocket
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        // Check current status
        {
            let guard = self.status.read().await;
            if matches!(*guard, ConnectionStatus::Connected | ConnectionStatus::Connecting) {
                return Ok(());
            }
        }

        let ws_url = self
            .endpoints
            .ws_base
            .ok_or_else(|| {
                WebSocketError::ConnectionError("No WebSocket URL configured".to_string())
            })?
            .to_string();

        // Create channels
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        self.command_tx = Some(command_tx);
        self.event_rx = Some(Arc::new(Mutex::new(event_rx)));

        // Clone state for the spawned task
        let auth = self.auth.clone();
        let debug = self.debug;
        let status = self.status.clone();
        let subscriptions = self.subscriptions.clone();

        // Spawn the STOMP loop
        tokio::spawn(async move {
            Self::run_stomp_loop(
                auth,
                ws_url,
                debug,
                status,
                subscriptions,
                command_rx,
                event_tx,
            )
            .await;
        });

        // Wait for connection with timeout.
        // The spawned task transitions status: Disconnected -> Connecting -> Connected (or back to Disconnected on failure).
        // We poll status and also peek the event channel for early error reports.
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(20);
        let mut saw_connecting = false;

        while start.elapsed() < timeout_duration {
            {
                let guard = self.status.read().await;
                match *guard {
                    ConnectionStatus::Connected => return Ok(()),
                    ConnectionStatus::Connecting => {
                        saw_connecting = true;
                    }
                    ConnectionStatus::Disconnected if saw_connecting => {
                        // Was connecting but went back to disconnected - connection failed.
                        // Try to extract the actual error from the event channel.
                        if let Some(rx) = &self.event_rx {
                            let mut rx_guard = rx.lock().await;
                            if let Ok(Err(ws_err)) = rx_guard.try_recv() {
                                return Err(ws_err);
                            }
                        }
                        return Err(WebSocketError::ConnectionError(
                            "STOMP connection failed - server rejected or timed out".to_string(),
                        ));
                    }
                    _ => {}
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(WebSocketError::Timeout)
    }

    /// Disconnect from MOEX WebSocket
    async fn disconnect(&mut self) -> WebSocketResult<()> {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(WsCommand::Disconnect);
        }

        // Wait for disconnection
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            let guard = self.status.read().await;
            if matches!(*guard, ConnectionStatus::Disconnected) {
                break;
            }
            drop(guard);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        self.event_rx = None;
        Ok(())
    }

    /// Get connection status
    fn connection_status(&self) -> ConnectionStatus {
        match self.status.try_read() {
            Ok(guard) => *guard,
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    /// Subscribe to a data stream
    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(WsCommand::Subscribe(request))
                .map_err(|_| WebSocketError::SendError("Command channel closed".to_string()))?;
            Ok(())
        } else {
            Err(WebSocketError::NotConnected)
        }
    }

    /// Unsubscribe from a data stream
    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(WsCommand::Unsubscribe(request))
                .map_err(|_| WebSocketError::SendError("Command channel closed".to_string()))?;
            Ok(())
        } else {
            Err(WebSocketError::NotConnected)
        }
    }

    /// Get event stream
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
        let rx = self.event_rx.clone();

        Box::pin(futures_util::stream::unfold(rx, |rx| async move {
            if let Some(rx) = rx {
                let mut guard = rx.lock().await;
                guard.recv().await.map(|event| (event, Some(rx.clone())))
            } else {
                None
            }
        }))
    }

    /// Get active subscriptions
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest> {
        match self.subscriptions.try_read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// MOEX orderbook capabilities
    ///
    /// REST (ISS): ~20 levels per side, no depth parameter, column-array format.
    /// WS (STOMP): `MXSE.orderbooks` channel delivers full snapshots, no deltas.
    /// Free tier (DEMO/guest): 15-minute delayed data. Real-time requires paid subscription.
    /// SEQNUM field present in REST responses.
    /// No checksum. No aggregation.
    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[20],
            ws_default_depth: Some(20),
            rest_max_depth: Some(20),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: &[],
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stomp_frame_parse_connected() {
        let raw = "CONNECTED\nversion:1.2\nserver:MOEX-ISS\nheart-beat:10000,10000\n\n\0";
        let frame = StompFrame::parse(raw).unwrap();
        assert_eq!(frame.command, StompCommand::Connected);
        assert_eq!(frame.header("version"), Some("1.2"));
        assert_eq!(frame.header("server"), Some("MOEX-ISS"));
    }

    #[test]
    fn test_stomp_frame_parse_message() {
        let raw = "MESSAGE\ndestination:/topic/engines/stock/markets/shares/boards/TQBR/securities/SBER\nsubscription:ticker-SBER\nmessage-id:msg-001\n\n{\"LAST\":308.31,\"BID\":308.26,\"ASK\":308.35}\0";
        let frame = StompFrame::parse(raw).unwrap();
        assert_eq!(frame.command, StompCommand::Message);
        assert_eq!(
            frame.header("destination"),
            Some("/topic/engines/stock/markets/shares/boards/TQBR/securities/SBER")
        );
        assert!(frame.body.contains("308.31"));
    }

    #[test]
    fn test_stomp_frame_parse_error() {
        let raw = "ERROR\nmessage:Authentication failed\n\nInvalid credentials\0";
        let frame = StompFrame::parse(raw).unwrap();
        assert_eq!(frame.command, StompCommand::Error);
        assert_eq!(frame.header("message"), Some("Authentication failed"));
        assert_eq!(frame.body, "Invalid credentials");
    }

    #[test]
    fn test_build_connect_frame() {
        let frame = build_connect_frame("iss.moex.com", None, None);
        assert!(frame.starts_with("CONNECT\n"));
        assert!(frame.contains("domain:DEMO"));
        assert!(frame.contains("login:guest"));
        assert!(frame.contains("passcode:guest"));
        assert!(frame.contains("accept-version:1.2"));
        assert!(frame.contains("host:iss.moex.com"));
        assert!(frame.contains("heart-beat:10000,10000"));
        assert!(frame.ends_with("\n\0"));

        // Verify exact format - MOEX requires \n (LF) not \r\n (CRLF)
        let expected = "CONNECT\ndomain:DEMO\nlogin:guest\npasscode:guest\naccept-version:1.2\nhost:iss.moex.com\nheart-beat:10000,10000\n\n\0";
        assert_eq!(frame, expected);

        // Verify no CRLF sequences exist
        assert!(!frame.contains("\r\n"), "Frame must use LF (\\n) not CRLF (\\r\\n)");
    }

    #[test]
    fn test_build_connect_frame_with_auth() {
        let frame = build_connect_frame("iss.moex.com", Some("user@example.com"), Some("password123"));
        assert!(frame.contains("domain:passport"));
        assert!(frame.contains("login:user@example.com"));
        assert!(frame.contains("passcode:password123"));
    }

    #[test]
    fn test_build_subscribe_frame() {
        let frame = build_subscribe_frame("ticker-SBER", "MXSE.securities", "TICKER=\"MXSE.TQBR.SBER\"");
        assert!(frame.starts_with("SUBSCRIBE\n"));
        assert!(frame.contains("id:ticker-SBER"));
        assert!(frame.contains("destination:MXSE.securities"));
        assert!(frame.contains("selector:TICKER=\"MXSE.TQBR.SBER\""));
        assert!(frame.contains("ack:auto"));
        assert!(frame.ends_with("\n\0"));
        assert!(!frame.contains("\r\n"), "Frame must use LF (\\n) not CRLF (\\r\\n)");
    }

    #[test]
    fn test_subscription_to_destination() {
        let symbol = Symbol::new("SBER", "RUB");

        let (dest, selector) = subscription_to_destination(&symbol, &StreamType::Ticker);
        assert_eq!(dest, "MXSE.securities");
        assert_eq!(selector, "TICKER=\"MXSE.TQBR.SBER\"");

        let (dest, selector) = subscription_to_destination(&symbol, &StreamType::Trade);
        assert_eq!(dest, "MXSE.securities");
        assert_eq!(selector, "TICKER=\"MXSE.TQBR.SBER\"");

        let (dest, selector) = subscription_to_destination(&symbol, &StreamType::Orderbook);
        assert_eq!(dest, "MXSE.orderbooks");
        assert_eq!(selector, "TICKER=\"MXSE.TQBR.SBER\"");
    }

    #[test]
    fn test_subscription_id() {
        let req = SubscriptionRequest::ticker(Symbol::new("SBER", "RUB"));
        assert_eq!(subscription_id(&req), "ticker-SBER");

        let req = SubscriptionRequest::trade(Symbol::new("GAZP", "RUB"));
        assert_eq!(subscription_id(&req), "trade-GAZP");
    }

    #[test]
    fn test_parse_ticker_event_flat_json() {
        let json: Value = serde_json::from_str(
            r#"{"LAST": 308.31, "BID": 308.26, "ASK": 308.35, "HIGH": 310.0, "LOW": 306.0, "VOLUME": 1000000}"#
        ).unwrap();

        let dest = "/topic/engines/stock/markets/shares/boards/TQBR/securities/SBER";
        let event = MoexWebSocket::parse_ticker_event(&json, dest).unwrap();

        if let StreamEvent::Ticker(ticker) = event {
            assert_eq!(ticker.symbol, "SBER");
            assert_eq!(ticker.last_price, 308.31);
            assert_eq!(ticker.bid_price, Some(308.26));
            assert_eq!(ticker.ask_price, Some(308.35));
        } else {
            panic!("Expected Ticker event");
        }
    }

    #[test]
    fn test_create_websocket() {
        let ws = MoexWebSocket::new_public();
        assert_eq!(ws.connection_status(), ConnectionStatus::Disconnected);
        assert!(ws.active_subscriptions().is_empty());
    }
}
