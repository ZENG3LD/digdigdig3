# Binance WebSocket Subscription Debug Report

## Problem Summary

Binance WebSocket tests show:
- **Connection works**: 2/2 tests pass (connect/disconnect)
- **Subscriptions fail**: 6/12 tests fail
  - Error: "Should receive at least one ticker event" - receives empty []
  - WebSocket CONNECTS successfully but subscriptions don't deliver events

## Root Cause Analysis

After analyzing the implementation and official Binance documentation, I've identified **TWO critical issues**:

### Issue 1: event_stream() Implementation Bug

**Location**: `src/exchanges/binance/websocket.rs` lines 922-934

**Problem**: The `event_stream()` method creates a NEW empty channel instead of returning the existing `event_tx` channel:

```rust
fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
    let (_tx, rx) = mpsc::unbounded_channel();  // ❌ Creates NEW channel!

    // Clone the event_tx to forward events
    let _event_tx = self.event_tx.clone();

    tokio::spawn(async move {
        // ❌ This is a simplified implementation
        // In production, we'd properly forward events from event_tx to tx
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))  // ❌ Returns empty channel!
}
```

**Why this breaks**:
- The message handler (line 285-339) sends events to `self.event_tx`
- But `event_stream()` returns a completely different channel that never receives anything
- Tests wait for events on the new channel, get nothing, timeout

### Issue 2: Subscription Message Format (Potentially)

**Current implementation** (lines 867-871):
```rust
let msg = SubscribeMessage {
    method: "SUBSCRIBE".to_string(),
    params: vec![stream_name],
    id: self.next_msg_id().await,
};
```

**Official Binance format** (from official docs):
```json
{
  "method": "SUBSCRIBE",
  "params": ["btcusdt@ticker", "btcusdt@depth"],
  "id": 1
}
```

**Status**: This looks CORRECT ✅

The subscription message format matches Binance spec exactly.

## Binance WebSocket Subscription Methods

Based on official documentation research (https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams):

### Method 1: Direct URL Connection

Connect to specific stream URL and receive data automatically:
```
wss://stream.binance.com:9443/ws/<streamName>
```

Example:
```
wss://stream.binance.com:9443/ws/btcusdt@ticker
```

- No subscription message needed
- Stream starts immediately upon connection
- Receives raw payloads directly

### Method 2: Combined Stream with Dynamic Subscriptions

Connect to combined stream endpoint and send SUBSCRIBE messages:
```
wss://stream.binance.com:9443/stream?streams=<stream1>/<stream2>
```

Or just:
```
wss://stream.binance.com:9443/stream
```

Then send subscription messages:
```json
{
  "method": "SUBSCRIBE",
  "params": ["btcusdt@ticker", "ethusdt@depth"],
  "id": 1
}
```

**Response**:
```json
{
  "result": null,
  "id": 1
}
```

**Message format for combined streams**:
```json
{
  "stream": "btcusdt@ticker",
  "data": {
    "e": "24hrTicker",
    "E": 1672515782136,
    ...
  }
}
```

### Current Implementation Analysis

**What we do** (line 275):
```rust
// Public stream URL (we'll use combined stream format)
format!("{}/stream", ws_base)
```

This connects to: `wss://stream.binance.com:9443/stream`

**Then** (lines 867-887):
```rust
let msg = SubscribeMessage {
    method: "SUBSCRIBE".to_string(),
    params: vec![stream_name],
    id: self.next_msg_id().await,
};

stream.send(Message::Text(msg_json)).await?;
```

**Status**: This approach is CORRECT ✅

We connect to the combined stream endpoint and send SUBSCRIBE messages, which is a valid Binance pattern.

## Stream Name Format Requirements

### Critical Rule: LOWERCASE ONLY

From official docs:
> All stream names must be **lowercase**

**Examples**:
- ✅ `btcusdt@ticker`
- ❌ `BTCUSDT@ticker`

### Current Implementation (line 475)

```rust
fn build_stream_name(request: &SubscriptionRequest, account_type: AccountType) -> String {
    let symbol = format_symbol(&request.symbol.base, &request.symbol.quote, account_type).to_lowercase();

    match &request.stream_type {
        StreamType::Ticker => format!("{}@ticker", symbol),
        StreamType::Trade => format!("{}@trade", symbol),
        StreamType::Orderbook => format!("{}@depth20@100ms", symbol),
        StreamType::OrderbookDelta => format!("{}@depth@100ms", symbol),
        StreamType::Kline { interval } => format!("{}@kline_{}", symbol, interval),
        ...
    }
}
```

**Status**: CORRECT ✅

We call `.to_lowercase()` on the symbol, so stream names are correctly lowercase.

## Message Parsing

### Combined Stream Format

When connecting to `/stream`, Binance wraps messages like:
```json
{
  "stream": "btcusdt@ticker",
  "data": {
    "e": "24hrTicker",
    ...
  }
}
```

### Current Implementation (lines 348-353)

```rust
// Try to parse as combined stream format first
if let Ok(combined) = serde_json::from_str::<CombinedStreamMessage>(text) {
    if let Some(event) = Self::parse_stream_data(&combined.data, account_type)? {
        let _ = event_tx.send(Ok(event));
    }
    return Ok(());
}
```

**Status**: CORRECT ✅

We parse combined stream format correctly.

## Subscription Response Handling

### What Binance Sends

When you send a SUBSCRIBE message, Binance responds with:
```json
{
  "result": null,
  "id": 1
}
```

### Current Implementation

**Problem**: We don't handle subscription responses!

The message handler (lines 304-373) only processes:
- `Message::Text` → Tries to parse as event data
- `Message::Ping` → Sends pong
- `Message::Close` → Disconnects

**What happens to subscription response**:
1. We send `{"method": "SUBSCRIBE", "params": ["btcusdt@ticker"], "id": 1}`
2. Binance sends `{"result": null, "id": 1}`
3. We try to parse this as event data
4. It doesn't match `CombinedStreamMessage` or `SingleStreamMessage` or have event type
5. We silently ignore it ✅ (This is fine!)

**Status**: OK ✅

Ignoring subscription responses is fine - we only care about actual stream events.

## Connection URL Analysis

### Spot Trading

**Current** (from endpoints.rs):
```rust
pub const WS_SPOT: &str = "wss://stream.binance.com:9443";
```

**Official**:
```
wss://stream.binance.com:9443/ws/<streamName>
wss://stream.binance.com:9443/stream?streams=...
```

**Our usage**:
```rust
format!("{}/stream", ws_base)  // wss://stream.binance.com:9443/stream
```

**Status**: CORRECT ✅

### Futures USDT-M

**Current** (from endpoints.rs):
```rust
pub const WS_FUTURES: &str = "wss://fstream.binance.com";
```

**Official**:
```
wss://fstream.binance.com/ws/<streamName>
wss://fstream.binance.com/stream?streams=...
```

**Our usage**:
```rust
format!("{}/stream", ws_base)  // wss://fstream.binance.com/stream
```

**Status**: CORRECT ✅

## Exact Subscription Flow

Based on official docs, here's what SHOULD happen:

### 1. Connect
```
Connection → wss://stream.binance.com:9443/stream
```

### 2. Send Subscribe Message
```json
{
  "method": "SUBSCRIBE",
  "params": ["btcusdt@ticker"],
  "id": 1
}
```

### 3. Receive Acknowledgment
```json
{
  "result": null,
  "id": 1
}
```

### 4. Receive Stream Events
```json
{
  "stream": "btcusdt@ticker",
  "data": {
    "e": "24hrTicker",
    "E": 1672515782136,
    "s": "BTCUSDT",
    "c": "42000.00",
    ...
  }
}
```

### What Our Code Does

1. ✅ Connects to `wss://stream.binance.com:9443/stream` (line 275)
2. ✅ Sends SUBSCRIBE with lowercase stream name (lines 867-887)
3. ✅ Parses acknowledgment (silently ignored, which is fine)
4. ✅ Parses stream events as `CombinedStreamMessage` (lines 348-353)
5. ❌ **SENDS EVENTS TO THE WRONG CHANNEL** (line 350)

## The REAL Problem: Event Stream Implementation

Looking at the event flow:

```
[WebSocket Message] → [handle_message()] → [event_tx.send(Ok(event))]
                                                     ↓
                                            [self.event_tx channel]
                                                     ↓
                                                  NOWHERE!

[Test calls event_stream()] → [Creates NEW channel] → [Returns empty rx]
                                                     ↓
                                            [Test waits here]
                                                     ↓
                                              [Gets nothing]
                                                     ↓
                                            [Timeout/Empty []]
```

**The channels are disconnected!**

## Solution

Fix the `event_stream()` method to properly return the receiver from `self.event_tx`:

```rust
fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>> {
    let (tx, rx) = mpsc::unbounded_channel();

    let event_tx = self.event_tx.clone();

    // Forward events from event_tx to the returned stream
    tokio::spawn(async move {
        if let Some(sender) = event_tx.lock().await.as_ref() {
            let mut receiver = sender.subscribe(); // Need to make event_tx a broadcast channel
            while let Some(event) = receiver.recv().await {
                if tx.send(event).is_err() {
                    break;
                }
            }
        }
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
```

**OR** use a different channel type (broadcast channel) that supports multiple receivers.

## Test Verification

Once fixed, the flow should be:

1. Test connects → `event_tx` channel created
2. Message handler starts → Sends events to `event_tx`
3. Test calls `event_stream()` → Gets receiver from `event_tx`
4. Binance sends ticker event → Handler parses → Sends to `event_tx`
5. Test receives event from stream → ✅ Test passes!

## Additional Findings

### WebSocket Connection Limits

From official docs:
- **Maximum 1,024 streams** per connection
- **5 incoming messages per second** per connection
- **300 connections per 5 minutes** per IP
- **24-hour auto-disconnect**

### Ping/Pong

- Server sends ping frame every 20 seconds
- Client must respond within 1 minute
- ✅ We handle this correctly (lines 310-316)

### Stream Types We Support

All stream formats are correct:

| Stream Type | Format | Status |
|-------------|--------|--------|
| Ticker | `btcusdt@ticker` | ✅ |
| Trade | `btcusdt@trade` | ✅ |
| Orderbook Snapshot | `btcusdt@depth20@100ms` | ✅ |
| Orderbook Delta | `btcusdt@depth@100ms` | ✅ |
| Kline | `btcusdt@kline_1m` | ✅ |
| Mark Price | `btcusdt@markPrice` | ✅ |

## Conclusion

### What's Wrong

**ONLY ONE ISSUE**: `event_stream()` implementation bug (lines 922-934)

The method creates a new empty channel instead of returning the receiver from the actual `event_tx` channel where events are being sent.

### What's Right

Everything else is implemented correctly:
- ✅ Connection URL format
- ✅ Subscription message format
- ✅ Stream name formatting (lowercase)
- ✅ Combined stream message parsing
- ✅ Ping/pong handling
- ✅ Event parsing

### Fix Priority

**HIGH**: Fix `event_stream()` to properly forward events from `event_tx` to the returned stream.

Once this is fixed, all subscription tests should pass.

## References

- [Binance WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Binance WebSocket API](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api)
- [WebSocket Market Streams](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md)
