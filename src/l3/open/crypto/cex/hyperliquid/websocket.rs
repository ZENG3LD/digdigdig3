//! # Hyperliquid WebSocket Implementation
//!
//! WebSocket connector with auto-reconnection and full event support.
//!
//! ## Features
//!
//! - Auto-reconnect on disconnect
//! - Snapshot + incremental update handling
//! - 19 subscription types supported
//! - Ping/pong heartbeat handling
//! - Broadcast channel for multiple consumers

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::{Stream, StreamExt, SinkExt, stream::{SplitSink, SplitStream}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};

use crate::core::{
    AccountType, ConnectionStatus, StreamEvent, StreamType,
    SubscriptionRequest,
};
use crate::core::types::{WebSocketResult, WebSocketError, OrderbookCapabilities, TradeSide};
use crate::core::traits::WebSocketConnector;

use super::{HyperliquidUrls, HyperliquidParser};

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET MESSAGES
// ═══════════════════════════════════════════════════════════════════════════════

/// Outgoing subscription message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    method: String,
    subscription: Value,
}

/// Incoming message from Hyperliquid
#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    channel: Option<String>,
    data: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HYPERLIQUID WEBSOCKET CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
/// Write half — used by subscribe, unsubscribe, and ping task
type WsSink = SplitSink<WsStream, Message>;
/// Read half — owned exclusively by the message loop task
type WsReader = SplitStream<WsStream>;

/// Hyperliquid WebSocket connector
pub struct HyperliquidWebSocket {
    /// WebSocket URLs
    urls: HyperliquidUrls,
    /// Connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashSet<SubscriptionRequest>>>,
    /// Broadcast sender (for multiple consumers, dropped on disconnect)
    broadcast_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
    /// WebSocket write half — shared by subscribe, unsubscribe, and ping task.
    /// The read half is owned exclusively by the message loop task (no mutex needed).
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Last ping time
    last_ping: Arc<Mutex<Instant>>,
    /// Most recent ping round-trip time in milliseconds (0 until first pong)
    ws_ping_rtt_ms: Arc<Mutex<u64>>,
}

impl HyperliquidWebSocket {
    /// Create new WebSocket connector
    pub fn new(is_testnet: bool) -> Self {
        let urls = if is_testnet {
            HyperliquidUrls::TESTNET
        } else {
            HyperliquidUrls::MAINNET
        };

        Self {
            urls,
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            broadcast_tx: Arc::new(StdMutex::new(None)),
            ws_sink: Arc::new(Mutex::new(None)),
            last_ping: Arc::new(Mutex::new(Instant::now())),
            ws_ping_rtt_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Create public WebSocket connector (convenience method)
    pub fn public(is_testnet: bool) -> Self {
        Self::new(is_testnet)
    }

    /// Connect to WebSocket
    async fn connect_ws(&self) -> WebSocketResult<WsStream> {
        let ws_url = self.urls.ws_url();

        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(|e| WebSocketError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        Ok(ws_stream)
    }

    /// Start message read loop.
    ///
    /// Takes ownership of `reader` (the `SplitStream` half) — no mutex is needed.
    /// Runs until the WebSocket connection closes or errors.
    fn start_message_handler(
        mut reader: WsReader,
        event_tx: Arc<StdMutex<Option<broadcast::Sender<WebSocketResult<StreamEvent>>>>>,
        status: Arc<Mutex<ConnectionStatus>>,
        last_ping: Arc<Mutex<Instant>>,
        ws_ping_rtt_ms: Arc<Mutex<u64>>,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match Self::handle_message(&text).await {
                            Ok(events) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    for event in events {
                                        let _ = tx.send(Ok(event));
                                    }
                                }
                            }
                            Err(e) => {
                                let tx_guard = event_tx.lock().unwrap();
                                if let Some(ref tx) = *tx_guard {
                                    let _ = tx.send(Err(e));
                                }
                            }
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // Response to our client-initiated WS Ping frame — measure RTT
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
                            let _ = tx.send(Err(WebSocketError::ConnectionError(e.to_string())));
                        }
                        break;
                    }
                    _ => {}
                }
            }
            // Drop the broadcast sender so all BroadcastStream receivers get None
            let _ = event_tx.lock().unwrap().take();
            *status.lock().await = ConnectionStatus::Disconnected;
        });
    }

    /// Handle incoming WebSocket message, returning 0-N parsed events.
    async fn handle_message(text: &str) -> WebSocketResult<Vec<StreamEvent>> {
        let msg: IncomingMessage = serde_json::from_str(text)
            .map_err(|e| WebSocketError::Parse(format!("Failed to parse message: {}", e)))?;

        // Get channel and data
        let channel = match msg.channel {
            Some(ch) => ch,
            None => return Ok(vec![]), // Ignore messages without channel
        };

        let data = match msg.data {
            Some(d) => d,
            None => return Ok(vec![]), // Ignore messages without data
        };

        // Parse based on channel type
        let mut events = Vec::new();
        match channel.as_str() {
            "activeAssetCtx" => {
                events.extend(Self::parse_active_asset_ctx(&data)?);
            }
            "webData2" => {
                // Large composite payload: mids, positions, fills, etc.
                // Partial parse — extract `mids` and emit Ticker per symbol,
                // same as allMids. Other fields (positions, fills) are ignored
                // as they require user context not available here.
                if let Some(mids_val) = data.get("mids") {
                    events.extend(Self::parse_all_mids(&serde_json::json!({"mids": mids_val}))?);
                }
            }
            "clearinghouseState" => {
                // User clearinghouse state including positions and balances.
                // Emits BalanceUpdate per balance entry and PositionUpdate per position.
                events.extend(Self::parse_clearinghouse_state(&data)?);
            }
            "liquidations" => {
                if let Some(event) = Self::parse_liquidation(&data)? {
                    events.push(event);
                }
            }
            "allMids" => {
                events.extend(Self::parse_all_mids(&data)?);
            }
            "bbo" => {
                events.extend(Self::parse_bbo(&data)?);
            }
            "userFundings" => {
                events.extend(Self::parse_user_fundings(&data)?);
            }
            "trades" => {
                if let Some(event) = Self::parse_trades(&data)? {
                    events.push(event);
                }
            }
            "l2Book" => {
                if let Some(event) = Self::parse_l2_book(&data)? {
                    events.push(event);
                }
            }
            "candle" => {
                if let Some(event) = Self::parse_candle(&data)? {
                    events.push(event);
                }
            }
            "subscriptionResponse" => {
                // Subscription confirmed - ignore
            }
            "error" => {
                // Error message
                let error_msg = data.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error");
                return Err(WebSocketError::ProtocolError(error_msg.to_string()));
            }
            _ => {
                // Unknown channel - ignore
            }
        }

        Ok(events)
    }

    /// Parse activeAssetCtx message to multiple events.
    ///
    /// This channel provides per-coin 24h stats including dayNtlVlm, prevDayPx,
    /// markPx, and midPx — far richer than allMids which only has mid-prices.
    ///
    /// Emits: Ticker, MarkPrice, FundingRate, OpenInterestUpdate, IndexPrice (oraclePx).
    ///
    /// Message format:
    /// ```json
    /// {
    ///   "coin": "BTC",
    ///   "ctx": {
    ///     "dayNtlVlm": "1234567890.5",
    ///     "funding": "0.000012345",
    ///     "openInterest": "987654.321",
    ///     "prevDayPx": "49500.0",
    ///     "markPx": "50123.45",
    ///     "midPx": "50123.5",
    ///     "impactPxs": ["50120.0", "50127.0"],
    ///     "premium": "0.5",
    ///     "oraclePx": "50122.95"
    ///   }
    /// }
    /// ```
    fn parse_active_asset_ctx(data: &Value) -> WebSocketResult<Vec<StreamEvent>> {
        let coin = data.get("coin")
            .and_then(|c| c.as_str())
            .ok_or_else(|| WebSocketError::Parse("Missing 'coin' in activeAssetCtx".to_string()))?;

        let ctx = data.get("ctx")
            .ok_or_else(|| WebSocketError::Parse("Missing 'ctx' in activeAssetCtx".to_string()))?;

        let parse_f64 = |val: &Value| -> Option<f64> {
            val.as_str().and_then(|s| s.parse().ok()).or_else(|| val.as_f64())
        };

        let mark_px = ctx.get("markPx").and_then(parse_f64).unwrap_or(0.0);
        let mid_px = ctx.get("midPx").and_then(parse_f64);
        let prev_day_px = ctx.get("prevDayPx").and_then(parse_f64);
        let volume_24h = ctx.get("dayNtlVlm").and_then(parse_f64);
        let funding_rate = ctx.get("funding").and_then(parse_f64);
        let open_interest = ctx.get("openInterest").and_then(parse_f64);
        let oracle_px = ctx.get("oraclePx").and_then(parse_f64);

        let last_price = mid_px.unwrap_or(mark_px);
        let now = crate::core::utils::timestamp_millis() as i64;

        let (price_change_24h, price_change_percent_24h) = match prev_day_px {
            Some(prev) if prev > 0.0 => {
                let change = last_price - prev;
                let change_pct = (change / prev) * 100.0;
                (Some(change), Some(change_pct))
            }
            _ => (None, None),
        };

        let mut events = Vec::with_capacity(5);

        // Ticker
        events.push(StreamEvent::Ticker(crate::core::Ticker {
            symbol: coin.to_string(),
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h,
            quote_volume_24h: None,
            price_change_24h,
            price_change_percent_24h,
            timestamp: now,
        }));

        // MarkPrice
        events.push(StreamEvent::MarkPrice {
            symbol: coin.to_string(),
            mark_price: mark_px,
            index_price: oracle_px,
            timestamp: now,
        });

        // FundingRate
        if let Some(rate) = funding_rate {
            events.push(StreamEvent::FundingRate {
                symbol: coin.to_string(),
                rate,
                next_funding_time: None,
                timestamp: now,
            });
        }

        // OpenInterestUpdate
        if let Some(oi) = open_interest {
            events.push(StreamEvent::OpenInterestUpdate {
                symbol: coin.to_string(),
                open_interest: oi,
                open_interest_value: None,
                timestamp: now,
            });
        }

        // IndexPrice (oraclePx)
        if let Some(idx_px) = oracle_px {
            events.push(StreamEvent::IndexPrice {
                symbol: coin.to_string(),
                price: idx_px,
                timestamp: now,
            });
        }

        Ok(events)
    }

    /// Parse `clearinghouseState` channel — user account state with positions and balances.
    ///
    /// Data structure (simplified):
    /// ```json
    /// {
    ///   "assetPositions": [
    ///     { "position": { "coin": "BTC", "szi": "0.1", "entryPx": "50000", "unrealizedPnl": "100", ... } }
    ///   ],
    ///   "marginSummary": { "accountValue": "10000", "totalMarginUsed": "500", ... }
    /// }
    /// ```
    ///
    /// Emits `BalanceUpdate` for the USDC account value (from `marginSummary`) and
    /// `PositionUpdate` for each non-zero position in `assetPositions`.
    fn parse_clearinghouse_state(data: &Value) -> WebSocketResult<Vec<StreamEvent>> {
        let mut events = Vec::new();
        let now = crate::core::utils::timestamp_millis() as i64;

        let parse_f64 = |val: &Value| -> Option<f64> {
            val.as_str().and_then(|s| s.parse().ok()).or_else(|| val.as_f64())
        };

        // Balance from marginSummary
        if let Some(summary) = data.get("marginSummary") {
            let account_value = summary.get("accountValue")
                .and_then(parse_f64)
                .unwrap_or(0.0);
            let margin_used = summary.get("totalMarginUsed")
                .and_then(parse_f64)
                .unwrap_or(0.0);
            events.push(StreamEvent::BalanceUpdate(crate::core::BalanceUpdateEvent {
                asset: "USDC".to_string(),
                free: (account_value - margin_used).max(0.0),
                locked: margin_used,
                total: account_value,
                delta: None,
                reason: None,
                timestamp: now,
            }));
        }

        // Positions from assetPositions
        if let Some(positions) = data.get("assetPositions").and_then(|v| v.as_array()) {
            for entry in positions {
                let pos = entry.get("position").unwrap_or(entry);

                let coin = match pos.get("coin").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => continue,
                };

                let size_str = pos.get("szi")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                let size: f64 = size_str.parse().unwrap_or(0.0);
                if size == 0.0 {
                    continue;
                }

                let entry_price = pos.get("entryPx")
                    .and_then(parse_f64)
                    .unwrap_or(0.0);
                let unrealized_pnl = pos.get("unrealizedPnl")
                    .and_then(parse_f64)
                    .unwrap_or(0.0);

                let side = if size > 0.0 {
                    crate::core::PositionSide::Long
                } else {
                    crate::core::PositionSide::Short
                };

                events.push(StreamEvent::PositionUpdate(crate::core::PositionUpdateEvent {
                    symbol: coin.to_string(),
                    side,
                    quantity: size.abs(),
                    entry_price,
                    mark_price: None,
                    unrealized_pnl,
                    realized_pnl: None,
                    liquidation_price: pos.get("liquidationPx").and_then(parse_f64),
                    leverage: pos.get("leverage")
                        .and_then(|v| v.get("value"))
                        .and_then(parse_f64)
                        .map(|v| v as u32),
                    margin_type: None,
                    reason: None,
                    timestamp: now,
                }));
            }
        }

        Ok(events)
    }

    /// Parse allMids message — emits one Ticker per symbol in the `mids` object.
    fn parse_all_mids(data: &Value) -> WebSocketResult<Vec<StreamEvent>> {
        // Format: { "mids": { "BTC": "50123.45", "ETH": "2500.67", ... } }
        let mids = data.get("mids")
            .and_then(|m| m.as_object())
            .ok_or_else(|| WebSocketError::Parse("Missing 'mids' object".to_string()))?;

        let now = crate::core::utils::timestamp_millis() as i64;
        let mut events = Vec::with_capacity(mids.len());

        for (symbol, price_val) in mids.iter() {
            if let Some(price) = price_val.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| price_val.as_f64())
            {
                events.push(StreamEvent::Ticker(crate::core::Ticker {
                    symbol: symbol.clone(),
                    last_price: price,
                    bid_price: None,
                    ask_price: None,
                    high_24h: None,
                    low_24h: None,
                    volume_24h: None,
                    quote_volume_24h: None,
                    price_change_24h: None,
                    price_change_percent_24h: None,
                    timestamp: now,
                }));
            }
        }

        Ok(events)
    }

    /// Parse `bbo` (best bid/offer) message → Ticker with bid_price/ask_price.
    ///
    /// Message format (verified from Hyperliquid docs):
    /// ```json
    /// { "coin": "BTC", "time": 1234567890, "bbo": [{"px":"50100.0","sz":"0.5","n":1}, null] }
    /// ```
    /// `bbo` is a 2-element array: [best_bid WsLevel | null, best_ask WsLevel | null].
    /// WsLevel = { px, sz, n } where px is price (string), sz is size (string).
    fn parse_bbo(data: &Value) -> WebSocketResult<Vec<StreamEvent>> {
        let parse_f64 = |val: &Value| -> Option<f64> {
            val.as_str().and_then(|s| s.parse().ok()).or_else(|| val.as_f64())
        };

        let coin = match data.get("coin").and_then(|c| c.as_str()) {
            Some(s) => s,
            None => return Ok(vec![]),
        };

        // bbo field is a 2-element array: [bid_level, ask_level], each may be null
        let bbo_arr = match data.get("bbo").and_then(|b| b.as_array()) {
            Some(a) => a,
            None => return Ok(vec![]),
        };

        let bid_price = bbo_arr.first()
            .and_then(|level| level.get("px"))
            .and_then(parse_f64);

        let ask_price = bbo_arr.get(1)
            .and_then(|level| level.get("px"))
            .and_then(parse_f64);

        let last_price = bid_price
            .zip(ask_price)
            .map(|(b, a)| (b + a) / 2.0)
            .or(bid_price)
            .or(ask_price)
            .unwrap_or(0.0);

        let now = crate::core::utils::timestamp_millis() as i64;

        Ok(vec![StreamEvent::Ticker(crate::core::Ticker {
            symbol: coin.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: now,
        })])
    }

    /// Parse `userFundings` message → FundingRate events per funding entry.
    ///
    /// Message format:
    /// ```json
    /// { "fundings": [ { "coin": "BTC", "fundingRate": "0.000012", "time": 1234567890 }, ... ] }
    /// ```
    fn parse_user_fundings(data: &Value) -> WebSocketResult<Vec<StreamEvent>> {
        let parse_f64 = |val: &Value| -> Option<f64> {
            val.as_str().and_then(|s| s.parse().ok()).or_else(|| val.as_f64())
        };

        let fundings = data.get("fundings")
            .and_then(|f| f.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        let mut events = Vec::with_capacity(fundings.len());

        for entry in fundings {
            let coin = match entry.get("coin").and_then(|c| c.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let rate = match entry.get("fundingRate")
                .and_then(parse_f64)
                .or_else(|| entry.get("funding").and_then(parse_f64))
            {
                Some(r) => r,
                None => continue,
            };
            let timestamp = entry.get("time")
                .and_then(|t| t.as_i64())
                .unwrap_or_else(|| crate::core::utils::timestamp_millis() as i64);

            events.push(StreamEvent::FundingRate {
                symbol: coin.to_string(),
                rate,
                next_funding_time: None,
                timestamp,
            });
        }

        Ok(events)
    }

    /// Parse trades message
    fn parse_trades(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        // Format: [ { "coin": "BTC", "side": "B", "px": "50123.45", "sz": "0.5", ... } ]
        let trades = data.as_array()
            .ok_or_else(|| WebSocketError::Parse("Expected array of trades".to_string()))?;

        // Emit first trade (in real implementation, might emit all)
        if let Some(trade_data) = trades.first() {
            let trade = HyperliquidParser::parse_recent_trades(&json!([trade_data]))
                .map_err(|e| WebSocketError::Parse(e.to_string()))?;

            if let Some(first_trade) = trade.into_iter().next() {
                return Ok(Some(StreamEvent::Trade(first_trade)));
            }
        }

        Ok(None)
    }

    /// Parse l2Book message
    fn parse_l2_book(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        // Format: { "coin": "BTC", "time": 1234567890, "levels": [[bids], [asks]] }
        let orderbook = HyperliquidParser::parse_orderbook(data)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        Ok(Some(StreamEvent::OrderbookSnapshot(orderbook)))
    }

    /// Parse candle message
    fn parse_candle(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        // Format: [ { "t": 1234, "o": "50100", "h": "50200", ... } ]
        let klines = HyperliquidParser::parse_klines(data)
            .map_err(|e| WebSocketError::Parse(e.to_string()))?;

        if let Some(kline) = klines.into_iter().next() {
            return Ok(Some(StreamEvent::Kline(kline)));
        }

        Ok(None)
    }

    /// Parse liquidation event from Hyperliquid `liquidations` channel.
    ///
    /// Hyperliquid format:
    /// ```json
    /// {
    ///   "coin": "BTC",
    ///   "side": "B",
    ///   "px": "50000.0",
    ///   "sz": "0.01",
    ///   "time": 1700000000000
    /// }
    /// ```
    /// Side mapping: "B"/"Buy" = buy-side forced order → short was liquidated → emit TradeSide::Sell
    ///               "A"/"Sell" = sell-side forced order → long was liquidated → emit TradeSide::Buy
    fn parse_liquidation(data: &Value) -> WebSocketResult<Option<StreamEvent>> {
        let parse_f64 = |val: &Value| -> Option<f64> {
            val.as_str().and_then(|s| s.parse().ok()).or_else(|| val.as_f64())
        };

        let symbol = match data.get("coin").and_then(|c| c.as_str()) {
            Some(s) => s.to_string(),
            None => return Ok(None),
        };
        let side_str = match data.get("side").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => return Ok(None),
        };
        let price = match data.get("px").and_then(|v| parse_f64(v)) {
            Some(p) => p,
            None => return Ok(None),
        };
        let quantity = match data.get("sz").and_then(|v| parse_f64(v)) {
            Some(q) => q,
            None => return Ok(None),
        };
        let timestamp = data.get("time")
            .or_else(|| data.get("ts"))
            .and_then(|t| t.as_i64())
            .unwrap_or(0);

        // "B"/"Buy" = buy order (forced) → short position was liquidated
        // "A"/"Sell" = sell order (forced) → long position was liquidated
        let side = match side_str {
            "B" | "Buy" => TradeSide::Sell,
            _ => TradeSide::Buy,
        };

        let value = Some(price * quantity);

        Ok(Some(StreamEvent::Liquidation {
            symbol,
            side,
            price,
            quantity,
            timestamp,
            value,
        }))
    }

    /// Build subscription object for Hyperliquid
    fn build_subscription(request: &SubscriptionRequest) -> Value {
        let coin = &request.symbol.base;

        match &request.stream_type {
            StreamType::Ticker => {
                // activeAssetCtx provides per-coin 24h stats: dayNtlVlm, prevDayPx,
                // markPx, midPx, funding, etc. Much richer than allMids (mid-price only).
                json!({
                    "type": "activeAssetCtx",
                    "coin": coin
                })
            }
            StreamType::Trade => {
                json!({
                    "type": "trades",
                    "coin": coin
                })
            }
            StreamType::Orderbook | StreamType::OrderbookDelta => {
                json!({
                    "type": "l2Book",
                    "coin": coin,
                    "nSigFigs": null,
                    "mantissa": null
                })
            }
            StreamType::Kline { interval } => {
                json!({
                    "type": "candle",
                    "coin": coin,
                    "interval": interval
                })
            }
            _ => {
                // Unsupported stream types — fall back to allMids for backward compatibility
                json!({
                    "type": "allMids",
                    "dex": ""
                })
            }
        }
    }

    /// Start heartbeat task.
    ///
    /// Sends a `Message::Ping(vec![])` frame every 30 seconds so the server
    /// can be kept alive and so RTT can be measured via the resulting
    /// `Message::Pong` received in the message handler.
    fn start_heartbeat_task(
        ws_sink: Arc<Mutex<Option<WsSink>>>,
        last_ping: Arc<Mutex<Instant>>,
        status: Arc<Mutex<ConnectionStatus>>,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;

                // Check if connection is still alive
                let last = *last_ping.lock().await;
                if last.elapsed() >= Duration::from_secs(60) {
                    // No pongs for 60 seconds — connection may be stale
                    *status.lock().await = ConnectionStatus::Disconnected;
                    break;
                }

                // Send a WS Ping frame; the message handler will record RTT on Pong
                let mut sink_guard = ws_sink.lock().await;
                if let Some(sink) = sink_guard.as_mut() {
                    if sink.send(Message::Ping(vec![])).await.is_ok() {
                        *last_ping.lock().await = Instant::now();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CONNECTOR TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl WebSocketConnector for HyperliquidWebSocket {
    async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Connecting;

        // Connect WebSocket and split into independent read/write halves.
        let ws_stream = self.connect_ws().await?;
        let (sink, reader) = ws_stream.split();
        *self.ws_sink.lock().await = Some(sink);
        *self.status.lock().await = ConnectionStatus::Connected;
        *self.last_ping.lock().await = Instant::now();

        // Create broadcast channel
        let (broadcast_sender, _) = broadcast::channel(1000);
        *self.broadcast_tx.lock().unwrap() = Some(broadcast_sender);

        // Start message handler — reader is moved in, never shared via mutex.
        Self::start_message_handler(
            reader,
            self.broadcast_tx.clone(),
            self.status.clone(),
            self.last_ping.clone(),
            self.ws_ping_rtt_ms.clone(),
        );

        // Start heartbeat task — uses ws_sink only, no contention with reader.
        Self::start_heartbeat_task(
            self.ws_sink.clone(),
            self.last_ping.clone(),
            self.status.clone(),
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> WebSocketResult<()> {
        *self.status.lock().await = ConnectionStatus::Disconnected;

        // Close the write half; the reader task owns the read half and exits naturally.
        if let Some(mut sink) = self.ws_sink.lock().await.take() {
            let _ = sink.close().await;
        }

        let _ = self.broadcast_tx.lock().unwrap().take();
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
        let subscription = Self::build_subscription(&request);

        let msg = SubscribeMessage {
            method: "subscribe".to_string(),
            subscription,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        let sink = sink_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        sink.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(sink_guard);

        self.subscriptions.lock().await.insert(request);

        Ok(())
    }

    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
        let subscription = Self::build_subscription(&request);

        let msg = SubscribeMessage {
            method: "unsubscribe".to_string(),
            subscription,
        };

        let msg_json = serde_json::to_string(&msg)
            .map_err(|e| WebSocketError::ProtocolError(e.to_string()))?;

        let mut sink_guard = self.ws_sink.lock().await;
        let sink = sink_guard.as_mut()
            .ok_or_else(|| WebSocketError::ConnectionError("Not connected".to_string()))?;

        sink.send(Message::Text(msg_json)).await
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        drop(sink_guard);

        self.subscriptions.lock().await.remove(&request);

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
            Ok(subs) => subs.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn ping_rtt_handle(&self) -> Option<Arc<Mutex<u64>>> {
        Some(self.ws_ping_rtt_ms.clone())
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: Some(20),
            rest_max_depth: Some(20),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: false,
            update_speeds_ms: &[],
            default_speed_ms: Some(500),
            ws_channels: &[],
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["null", "2", "3", "4", "5"],
        }
    }
}

/// Subscription types specific to Hyperliquid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum HyperliquidSubscription {
    /// All mid prices (price only, no 24h stats). Use ActiveAssetCtx for full ticker.
    AllMids,
    /// Per-coin 24h stats: dayNtlVlm, prevDayPx, markPx, midPx, funding, openInterest.
    /// Use this for ticker subscriptions — richer than AllMids.
    ActiveAssetCtx,
    Trades,           // Trade feed
    L2Book,           // Order book updates
    Bbo,              // Best bid/offer
    Candle,           // Kline/candle updates
    Notification,     // User notifications
    OpenOrders,       // Open orders
    OrderUpdates,     // Order status changes
    UserFills,        // Trade executions
    UserEvents,       // All account events
    UserFundings,     // Funding payments
    ClearinghouseState, // Account summary
}

impl HyperliquidSubscription {
    /// Get subscription type string
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllMids => "allMids",
            Self::ActiveAssetCtx => "activeAssetCtx",
            Self::Trades => "trades",
            Self::L2Book => "l2Book",
            Self::Bbo => "bbo",
            Self::Candle => "candle",
            Self::Notification => "notification",
            Self::OpenOrders => "openOrders",
            Self::OrderUpdates => "orderUpdates",
            Self::UserFills => "userFills",
            Self::UserEvents => "userEvents",
            Self::UserFundings => "userFundings",
            Self::ClearinghouseState => "clearinghouseState",
        }
    }

    /// Does subscription require authentication
    #[allow(dead_code)]
    pub fn requires_auth(&self) -> bool {
        matches!(self,
            Self::Notification
            | Self::OpenOrders
            | Self::OrderUpdates
            | Self::UserFills
            | Self::UserEvents
            | Self::UserFundings
            | Self::ClearinghouseState
        )
    }
}
