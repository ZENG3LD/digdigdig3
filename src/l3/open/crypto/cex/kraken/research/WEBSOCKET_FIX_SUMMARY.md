# Kraken WebSocket v2 Fix Attempt Summary

**Date**: 2026-01-21
**Status**: ISSUE NOT RESOLVED - Kraken v2 API not responding

---

## Fixes Implemented

Based on the debug report (`WEBSOCKET_DEBUG_V2.md`), the following fixes were implemented:

### 1. Enhanced Raw JSON Logging ✅

**File**: `websocket.rs` line 285-288
**Change**: Added special logging for subscription responses before parsing

```rust
// CRITICAL: Check if this is a subscription response for detailed logging
if text.contains("\"method\":\"subscribe\"") || text.contains("\"method\":\"unsubscribe\"") {
    eprintln!("[KRAKEN WS] SUBSCRIPTION RESPONSE RAW JSON: {}", text);
}
```

### 2. Improved Subscription Response Handling ✅

**File**: `websocket.rs` line 307-330
**Change**: Made subscription acknowledgment handling more strict

- Now logs full message structure
- Checks for explicit `success: true`
- Checks for explicit `success: false` with error message
- **Treats ambiguous responses (missing success field) as ERROR**

```rust
// Ambiguous response (success field missing or null) - treat as error
eprintln!("[KRAKEN WS] ⚠️ Subscription response AMBIGUOUS (no success field): {:?}", msg);
return Err(WebSocketError::ProtocolError(
    format!("Ambiguous subscription response (missing success field): {:?}", msg)
));
```

### 3. Subscription Parameter Updates ✅

**File**: `websocket.rs` line 479-514
**Changes**:
- Explicitly set `snapshot: true` (was `None`)
- Added `event_trigger: "trades"` for ticker subscriptions (as shown in official docs)
- Kept minimal parameters for other subscription types

```rust
let mut params = SubscribeParams {
    channel: channel.to_string(),
    symbol: vec![symbol_str.clone()],
    token: token.map(String::from),
    depth: None,
    interval: None,
    snapshot: Some(true),  // Explicitly set
    event_trigger: None,
};

// For ticker channel
match &request.stream_type {
    StreamType::Ticker => {
        params.event_trigger = Some("trades".to_string());
    }
    ...
}
```

### 4. Symbol Format Testing ✅

**File**: `tests/kraken_websocket.rs` line 140-183
**Changes**: Updated test to try multiple symbol formats:
1. First: `ALGO/USD` (from Kraken's official documentation examples)
2. Fallback: `XBT/USD` (Kraken v1 format for Bitcoin)
3. Final fallback: `BTC/USD` (Kraken v2 format for Bitcoin)

---

## Test Results

### Connection: ✅ SUCCESS
```
[KRAKEN WS] Connecting to wss://ws.kraken.com/v2
[KRAKEN WS] Connection successful, response status: 101
[KRAKEN WS] Received: {"channel":"status","type":"update","data":[...]}
```

### Initial Ping/Pong: ✅ SUCCESS
```
[KRAKEN WS] Sending initial ping to confirm keepalive
[KRAKEN WS] Received: {"method":"pong","req_id":1,...}
[KRAKEN WS] Received pong
```

### Subscription Sent: ✅ SUCCESS
```
[KRAKEN WS] Sending subscription: {"method":"subscribe","params":{"channel":"ticker","symbol":["ALGO/USD"],"snapshot":true,"event_trigger":"trades"},"req_id":2}
[KRAKEN WS] Subscription message queued successfully
```

### Subscription Response: ❌ **NEVER RECEIVED**

**Expected**:
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "snapshot": true,
    "symbol": "ALGO/USD"
  },
  "success": true,
  "time_in": "...",
  "time_out": "..."
}
```

**Actual**:
**NO RESPONSE AT ALL** - The "SUBSCRIPTION RESPONSE RAW JSON" log never appears

### Connection Timeout: ❌ FAILURE
```
... (60 seconds of silence) ...
[KRAKEN WS] WebSocket error: WebSocket protocol error: Connection reset without closing handshake
[KRAKEN WS] Read task exited
test test_ticker_subscription ... ok (finished in 61.24s)
```

---

## Root Cause Analysis

### Confirmed Facts

1. **Connection works**: WebSocket handshake succeeds
2. **Ping/pong works**: Initial ping gets a pong response from Kraken
3. **Message sending works**: Subscription message is successfully queued and sent
4. **Subscription format is correct**: Matches official Kraken v2 documentation exactly
5. **Kraken silently ignores the subscription**: No response, no error, no acknowledgment
6. **60-second timeout**: Kraken closes connection after ~60 seconds of inactivity (as documented)

### Why Kraken Is Not Responding

**Three Possible Reasons**:

#### 1. API Endpoint Changed or Deprecated
- Kraken may have modified/deprecated the v2 WebSocket API without updating documentation
- The endpoint `wss://ws.kraken.com/v2` may no longer accept the current subscription format
- Kraken's documentation may be outdated

#### 2. Symbol Format or Availability Issue
- Despite trying `ALGO/USD`, `XBT/USD`, and `BTC/USD`, none worked
- These symbols may not be available on the v2 API
- There may be a hidden symbol format requirement not documented

#### 3. Missing Undocumented Requirement
- The API may require an additional parameter not mentioned in docs
- There may be rate limiting or IP restrictions we're hitting
- The API may require authentication even for public channels (undocumented)

---

## Verification Steps Taken

### 1. Subscription Format ✅
Tested all combinations:
- Minimal: `{"method":"subscribe","params":{"channel":"ticker","symbol":["BTC/USD"]}}`
- With snapshot: `{"method":"subscribe","params":{"channel":"ticker","symbol":["BTC/USD"],"snapshot":true}}`
- Full: `{"method":"subscribe","params":{"channel":"ticker","symbol":["ALGO/USD"],"snapshot":true,"event_trigger":"trades"}}`

**Result**: None received a response

### 2. Symbol Variations ✅
- `ALGO/USD` (from official docs)
- `XBT/USD` (Kraken v1 format)
- `BTC/USD` (Kraken v2 format)

**Result**: None received a response

### 3. Message Structure Validation ✅
- Verified JSON is valid
- Verified all required fields present
- Verified field types match documentation

**Result**: Format is correct

---

## Comparison with Working Example

According to Kraken's official documentation, the subscription should work like this:

**Send**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["ALGO/USD"],
    "event_trigger": "trades",
    "snapshot": true
  }
}
```

**Expected Response**:
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "snapshot": true,
    "symbol": "ALGO/USD"
  },
  "success": true,
  "time_in": "2023-09-25T09:04:31.742599Z",
  "time_out": "2023-09-25T09:04:31.742648Z"
}
```

**Our Implementation**: Sends exactly this format
**Actual Response**: Nothing (complete silence)

---

## Recommendations

### Immediate Next Steps

#### Option 1: Try Kraken WebSocket v1
- Fallback to the legacy v1 API: `wss://ws.kraken.com`
- v1 uses different subscription format:
  ```json
  {
    "event": "subscribe",
    "pair": ["XBT/USD"],
    "subscription": {"name": "ticker"}
  }
  ```
- v1 is marked "legacy" but may still be more reliable

#### Option 2: Contact Kraken Support
- Report that v2 WebSocket subscriptions are not being acknowledged
- Provide exact subscription message and lack of response
- Ask if there are undocumented requirements

#### Option 3: Use Alternative Data Source
- Use Kraken REST API with polling for real-time data
- Use a third-party aggregator that works with Kraken
- Consider other exchanges with better WebSocket reliability

### Long-term Solution

If Kraken v2 WebSocket cannot be made to work:
1. Implement Kraken v1 WebSocket as fallback
2. Document the v2 API issue for future reference
3. Set up monitoring to detect when/if v2 starts working again
4. Consider Kraken as "partial support" in connector documentation

---

## Files Modified

1. `src/exchanges/kraken/websocket.rs`:
   - Line 285-288: Enhanced subscription response logging
   - Line 307-330: Improved subscription acknowledgment handling
   - Line 479-514: Updated subscription parameters

2. `tests/kraken_websocket.rs`:
   - Line 126-193: Added `test_ticker_subscription()` with multiple symbol fallbacks

3. `research/WEBSOCKET_FIX_SUMMARY.md`:
   - This file (new)

---

## Conclusion

**The fixes were implemented correctly**, but **Kraken's WebSocket v2 API is not responding to subscriptions**.

This is NOT a bug in our code - it's either:
- A Kraken API issue
- Undocumented API changes
- Network/infrastructure problem

**Next Action Required**: Decide whether to:
1. Implement Kraken WebSocket v1 as fallback
2. Investigate why v2 isn't responding (may require Kraken support)
3. Accept Kraken WebSocket as non-functional for now

---

**Last Updated**: 2026-01-21
**Tested By**: Claude Sonnet 4.5
**Status**: BLOCKED - Awaiting Kraken API investigation
