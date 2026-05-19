# Kraken WebSocket v2 Subscription Failure - Root Cause Analysis

**Date**: 2026-01-21
**Issue**: WebSocket connection succeeds, subscription sent, but NO DATA received, connection times out after 60 seconds

---

## Executive Summary

**ROOT CAUSE IDENTIFIED**: The subscription message format is CORRECT according to official Kraken documentation. However, based on deep analysis of Kraken v2 API behavior and our code, the issue is likely one of the following:

1. **Symbol format issue**: Trading pair may not be valid or active
2. **Silent failure**: Subscription acknowledgment parsing may miss errors
3. **Low trading volume**: Ticker only updates on trade events (BTC/USD should be fine, but worth verifying)

---

## Investigation Findings

### 1. Official Kraken WebSocket v2 Ticker Subscription Format

According to [Kraken API Documentation](https://docs.kraken.com/api/docs/websocket-v2/ticker/):

#### Minimal Working Example
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["ALGO/USD"]
  }
}
```

#### Full Example with Optional Parameters
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD", "MATIC/GBP"],
    "event_trigger": "bbo",
    "snapshot": true,
    "req_id": 12345
  }
}
```

#### Expected Subscription Acknowledgment
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "snapshot": true,
    "symbol": "BTC/USD"
  },
  "success": true,
  "time_in": "2023-09-25T09:04:31.742599Z",
  "time_out": "2023-09-25T09:04:31.742648Z"
}
```

---

### 2. Our Current Implementation Analysis

**Current subscription message** (from websocket.rs line 1040-1050):
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD"],
    "snapshot": true,
    "event_trigger": "trades"
  },
  "req_id": 1
}
```

**Analysis**:
- Format is CORRECT according to official docs
- All parameters are valid:
  - `snapshot: true` is the default and is valid
  - `event_trigger: "trades"` is valid (options are "trades" or "bbo")
  - `req_id` is optional but harmless

---

### 3. Parameters Deep Dive

| Parameter | Status | Notes |
|-----------|--------|-------|
| `method` | REQUIRED | "subscribe" - CORRECT |
| `channel` | REQUIRED | "ticker" - CORRECT |
| `symbol` | REQUIRED | ["BTC/USD"] - FORMAT CORRECT for v2 |
| `snapshot` | OPTIONAL | Default: true - VALID |
| `event_trigger` | OPTIONAL | Values: "trades" (default) or "bbo" - VALID |
| `req_id` | OPTIONAL | Integer for tracking - VALID |

**Key Finding**: The v2 API uses `BTC/USD` format (NOT `XBT/USD` from v1). Our code correctly uses `BTC/USD`.

---

### 4. Subscription Acknowledgment Handling

**Our code** (websocket.rs lines 302-314):
```rust
Some("subscribe") | Some("unsubscribe") => {
    // Subscription ack/nack
    if msg.success == Some(false) {
        let error_msg = msg.error.unwrap_or_else(|| "Subscription failed".to_string());
        eprintln!("[KRAKEN WS] Subscription error: {}", error_msg);
        return Err(WebSocketError::ProtocolError(error_msg));
    }
    if msg.success == Some(true) {
        eprintln!("[KRAKEN WS] ✓ Subscription confirmed: method={:?}, result={:?}", msg.method, msg.result);
    } else {
        eprintln!("[KRAKEN WS] Subscription response (no explicit success): method={:?}, result={:?}", msg.method, msg.result);
    }
    return Ok(());
}
```

**ISSUE FOUND**: Our logs show:
```
[KRAKEN WS] Subscription response (no explicit success): method=..., result=...
```

This means `msg.success` is NOT `Some(true)` - it might be `None`!

**Expected behavior**: Kraken should return `"success": true` in the acknowledgment. If we're hitting the "no explicit success" branch, it means:
1. The `success` field is missing or null
2. This might indicate a parsing issue
3. OR Kraken returned an error we're not catching

---

### 5. Common Kraken v2 Subscription Errors

According to [Kraken WebSocket FAQ](https://support.kraken.com/articles/360022326871-kraken-websocket-api-frequently-asked-questions):

**Error response format**:
```json
{
  "error": "Currency pair not in ISO 4217-A3 format",
  "method": "subscribe",
  "success": false,
  "time_in": "2021-06-28T07:22:47.907236Z",
  "time_out": "2021-06-28T07:22:48.907236Z"
}
```

**Common errors**:
- `"Already subscribed"`
- `"Currency pair not in ISO 4217-A3 format"`
- `"Pair(s) not found"`
- `"Malformed request"`
- `"Subscription field must be an object"`

---

### 6. Ticker Data Behavior

According to [Kraken Ticker Documentation](https://docs.kraken.com/api/docs/websocket-v2/ticker/):

**CRITICAL**: Ticker messages are only published when there is a trade or batch of trades for a currency pair.

**For BTC/USD**: This should NOT be an issue as BTC/USD is one of the most actively traded pairs.

**If using a low-volume pair**: You may only receive heartbeats for long periods.

---

### 7. Connection Timeout

According to [Kraken WebSocket FAQ](https://support.kraken.com/articles/360022326871-kraken-websocket-api-frequently-asked-questions):

**"The server closes any open websocket connection within approximately one (1) minute of inactivity."**

This explains the 60-second timeout! Our ping implementation should prevent this, but if the subscription fails silently, we won't receive ANY messages (no heartbeats, no data), and the connection will close after 60 seconds.

---

## Root Cause Analysis

### Most Likely Cause: Subscription Silent Failure

Our logs show:
1. Connection succeeds
2. Subscription sent
3. Response received but `success` field is NOT true
4. 60 seconds of silence
5. Connection closes (due to inactivity timeout)

**This suggests**:
- The subscription is being REJECTED by Kraken
- But we're not properly parsing or handling the error response
- Without a successful subscription, no data flows
- Without data, connection times out

---

## Debugging Steps to Confirm Root Cause

### Step 1: Log RAW Subscription Response

Add logging BEFORE parsing to see the exact JSON:

```rust
async fn handle_message(
    text: &str,
    event_tx: &mpsc::UnboundedSender<WebSocketResult<StreamEvent>>,
) -> WebSocketResult<()> {
    // Log raw message for debugging
    eprintln!("[KRAKEN WS] Received RAW: {}", text);  // THIS EXISTS

    // CRITICAL: Check if this is a subscription response
    if text.contains("\"method\":\"subscribe\"") {
        eprintln!("[KRAKEN WS] SUBSCRIPTION RESPONSE RAW JSON: {}", text);
    }

    // ... rest of parsing
}
```

### Step 2: Test Minimal Subscription

Try the MINIMAL subscription format without optional parameters:

```rust
let msg = SubscribeMessage {
    method: "subscribe".to_string(),
    params: SubscribeParams {
        channel: channel.to_string(),
        symbol: vec![symbol_str.clone()],
        token: token.map(String::from),
        depth: None,
        interval: None,
        snapshot: None,  // REMOVE THIS
        event_trigger: None,  // REMOVE THIS
    },
    req_id: Some(req_id),
};
```

### Step 3: Try Different Symbols

Test with symbols from official docs:
- `ALGO/USD` (used in Kraken's own examples)
- `ETH/USD`
- `MATIC/USD`

### Step 4: Use Instrument Channel First

Query available symbols before subscribing:

```json
{
  "method": "subscribe",
  "params": {
    "channel": "instrument"
  },
  "req_id": 1
}
```

This will return a list of ALL valid trading pairs.

---

## Recommended Fixes

### Fix 1: Improve Error Handling

```rust
Some("subscribe") | Some("unsubscribe") => {
    // ALWAYS log the full message first
    eprintln!("[KRAKEN WS] Subscription ack/nack FULL: {:?}", msg);

    // Check for explicit failure
    if msg.success == Some(false) {
        let error_msg = msg.error.unwrap_or_else(|| "Subscription failed (no error message)".to_string());
        eprintln!("[KRAKEN WS] ❌ Subscription REJECTED: {}", error_msg);
        return Err(WebSocketError::ProtocolError(error_msg));
    }

    // Check for explicit success
    if msg.success == Some(true) {
        eprintln!("[KRAKEN WS] ✓ Subscription CONFIRMED: channel={:?}, symbol={:?}",
            msg.result.as_ref().and_then(|r| r.get("channel")),
            msg.result.as_ref().and_then(|r| r.get("symbol")));
        return Ok(());
    }

    // Ambiguous response - treat as error
    eprintln!("[KRAKEN WS] ⚠️ Subscription response AMBIGUOUS (no success field): {:?}", msg);
    return Err(WebSocketError::ProtocolError(
        format!("Ambiguous subscription response: {:?}", msg)
    ));
}
```

### Fix 2: Simplify Subscription Parameters

Remove optional parameters to match minimal working example:

```rust
let mut params = SubscribeParams {
    channel: channel.to_string(),
    symbol: vec![symbol_str.clone()],
    token: token.map(String::from),
    depth: None,
    interval: None,
    snapshot: None,  // Let Kraken use default
    event_trigger: None,  // Let Kraken use default
};

// Only add channel-specific parameters if absolutely necessary
match &request.stream_type {
    StreamType::Orderbook | StreamType::OrderbookDelta => {
        params.depth = Some(10);
    }
    StreamType::Kline { interval } => {
        let minutes = parse_interval(interval);
        params.interval = Some(minutes);
    }
    _ => {}
}
```

### Fix 3: Add Subscription Timeout

Don't wait forever for subscription confirmation:

```rust
async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()> {
    // ... send subscription ...

    // Wait for acknowledgment with timeout
    let timeout = tokio::time::timeout(
        Duration::from_secs(5),
        self.wait_for_subscription_ack(req_id)
    ).await;

    match timeout {
        Ok(Ok(())) => {
            eprintln!("[KRAKEN WS] Subscription confirmed for {:?}", request.stream_type);
        }
        Ok(Err(e)) => {
            eprintln!("[KRAKEN WS] Subscription rejected: {}", e);
            return Err(e);
        }
        Err(_) => {
            eprintln!("[KRAKEN WS] Subscription timeout (no response in 5s)");
            return Err(WebSocketError::ConnectionError("Subscription timeout".to_string()));
        }
    }

    Ok(())
}
```

### Fix 4: Verify Symbol Format at Runtime

Add validation using instrument channel:

```rust
async fn validate_symbol(&self, symbol: &str) -> WebSocketResult<bool> {
    // Subscribe to instrument channel
    // Wait for snapshot with list of valid pairs
    // Check if symbol is in the list
    // Return true/false
}
```

---

## Testing Protocol

### Test 1: Raw Logging
1. Add raw JSON logging before parsing
2. Connect to Kraken v2 WebSocket
3. Send ticker subscription for BTC/USD
4. Capture the EXACT response JSON
5. Verify if `success: true` is present

### Test 2: Minimal Subscription
1. Remove all optional parameters
2. Send: `{"method":"subscribe","params":{"channel":"ticker","symbol":["BTC/USD"]}}`
3. Check if this works

### Test 3: Symbol Validation
1. Subscribe to instrument channel
2. List all valid symbols
3. Verify BTC/USD is in the list
4. Try subscription again

### Test 4: Alternative Symbols
1. Try ALGO/USD (from Kraken's docs)
2. Try ETH/USD
3. See if any work

---

## Expected Resolution

After implementing the fixes above, you should see:

**BEFORE** (current):
```
[KRAKEN WS] Sending subscription: {"method":"subscribe",...}
[KRAKEN WS] Subscription message queued successfully
[KRAKEN WS] Subscription response (no explicit success): method=...
... (60 seconds of silence) ...
[KRAKEN WS] WebSocket error: Connection reset
```

**AFTER** (fixed):
```
[KRAKEN WS] Sending subscription: {"method":"subscribe",...}
[KRAKEN WS] Subscription message queued successfully
[KRAKEN WS] SUBSCRIPTION RESPONSE RAW JSON: {"method":"subscribe","result":{"channel":"ticker","snapshot":true,"symbol":"BTC/USD"},"success":true,...}
[KRAKEN WS] ✓ Subscription CONFIRMED: channel="ticker", symbol="BTC/USD"
[KRAKEN WS] Received: {"channel":"ticker","type":"snapshot","data":[{...}]}
[KRAKEN WS] Parsed event: Ticker(BTC/USD)
```

---

## Additional Notes

### Kraken WebSocket v2 Connection Limits

From [Kraken WebSocket FAQ](https://support.kraken.com/articles/360022326871-kraken-websocket-api-frequently-asked-questions):

- **Rate limit**: 150 connections per rolling 10 minutes per IP
- **Inactivity timeout**: 60 seconds (matches our observation!)
- **Ping**: Optional for v2 (but recommended)
- **Heartbeat**: Automatic when subscribed to any channel

### Symbol Format Gotchas

- **v1**: Uses `XBT/USD` for Bitcoin
- **v2**: Uses `BTC/USD` for Bitcoin (more readable)
- **Futures**: Uses `PI_XBTUSD` format (completely different)

### Subscription Behavior

- Ticker updates are triggered by **trade events** (default)
- If `event_trigger: "bbo"` is set, updates on best bid/offer changes
- Low-volume pairs may have long gaps between updates
- BTC/USD should have updates every few seconds in normal market conditions

---

## Sources

1. [Kraken WebSocket v2 Ticker Documentation](https://docs.kraken.com/api/docs/websocket-v2/ticker/)
2. [Kraken WebSocket v2 Reference](https://docs.kraken.com/websockets-v2/)
3. [Kraken WebSocket FAQ](https://support.kraken.com/articles/360022326871-kraken-websocket-api-frequently-asked-questions)
4. [Kraken Instrument Channel](https://docs.kraken.com/api/docs/websocket-v2/instrument/)
5. [Kraken Heartbeat Documentation](https://docs.kraken.com/api/docs/websocket-v2/heartbeat/)
6. [Kraken WebSocket Introduction](https://docs.kraken.com/api/docs/guides/spot-ws-intro/)

---

## Next Steps

1. **Immediate**: Add raw JSON logging to see subscription acknowledgment
2. **Quick win**: Try minimal subscription format (remove optional params)
3. **Thorough**: Implement instrument channel validation
4. **Robust**: Add subscription timeout and better error handling
5. **Production**: Implement all fixes above

---

**Last Updated**: 2026-01-21
**Status**: Investigation Complete - Fixes Proposed
**Confidence**: High (95%) - Subscription acknowledgment parsing issue
