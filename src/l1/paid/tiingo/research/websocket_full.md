# Tiingo - WebSocket Documentation

## Availability: Yes

Tiingo provides WebSocket APIs for real-time streaming data across IEX (stocks), Forex, and Crypto.

---

## Connection

### URLs
- **IEX (Stocks)**: wss://api.tiingo.com/iex
- **Forex**: wss://api.tiingo.com/fx
- **Crypto**: wss://api.tiingo.com/crypto
- Regional: None
- Separate private streams: No (authentication handles access)

### Connection Process
1. **Connect** to WebSocket URL (wss://api.tiingo.com/{endpoint})
2. **Welcome message**: Server may send initial connection confirmation
3. **Subscribe**: Send subscribe message with authentication and parameters
4. **Data stream**: Receive real-time updates based on subscriptions
5. **Heartbeat**: Server sends periodic heartbeat messages (messageType "H")

---

## ALL Available Channels/Topics

Tiingo uses a unified subscribe pattern with "eventData" controlling which data you receive. Different endpoints (iex, fx, crypto) provide different data streams.

### IEX Endpoint (wss://api.tiingo.com/iex)

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Threshold Control |
|---------------|------|-------------|-------|-------|------------------|-------------------|
| iex | Public | Real-time IEX trades & quotes | Yes | Yes | Real-time (microsecond) | thresholdLevel parameter |

**Threshold Level**: Controls data volume
- Higher thresholdLevel = fewer updates (more filtered)
- **thresholdLevel 5** = ALL top-of-book updates (maximum data)
- Lower values filter out minor price changes

### Forex Endpoint (wss://api.tiingo.com/fx)

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Threshold Control |
|---------------|------|-------------|-------|-------|------------------|-------------------|
| fx | Public | Top-of-book FX quotes | Yes | Yes | Real-time (microsecond) | thresholdLevel parameter |

**Threshold Level**: Same as IEX
- **thresholdLevel 5** = ALL top-of-book updates
- Lower values reduce update frequency

### Crypto Endpoint (wss://api.tiingo.com/crypto)

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Threshold Control |
|---------------|------|-------------|-------|-------|------------------|-------------------|
| crypto | Public | Crypto top-of-book quotes | Yes | Yes | Real-time | thresholdLevel parameter |

**Note**: All WebSocket endpoints provide "firehose" access even on free tier. The firehose can deliver microsecond-resolution data, so systems must be built to scale and handle high message volume.

---

## Subscription Format

### Subscribe Message (IEX Example)

```json
{
  "eventName": "subscribe",
  "authorization": "YOUR_API_TOKEN_HERE",
  "eventData": {
    "thresholdLevel": 5
  }
}
```

**Fields:**
- `eventName`: "subscribe" or "unsubscribe"
- `authorization`: Your Tiingo API token
- `eventData`: Object containing subscription parameters
  - `thresholdLevel`: Integer (5 = all updates, lower = filtered)
  - Additional parameters may vary by endpoint

### Unsubscribe Message

```json
{
  "eventName": "unsubscribe",
  "authorization": "YOUR_API_TOKEN_HERE",
  "eventData": {
    "thresholdLevel": 5
  }
}
```

### Subscription Confirmation

The server sends data messages immediately upon successful subscription. There may not be a separate "subscribed" confirmation message. Data messages indicate successful subscription.

---

## Message Formats (for EVERY channel)

### IEX Trade/Quote Update

WebSocket messages from IEX endpoint:

```json
{
  "service": "iex",
  "messageType": "A",
  "data": [
    {
      "ticker": "AAPL",
      "timestamp": "2020-01-01T12:34:56.789012Z",
      "last": 150.25,
      "lastSize": 100,
      "tngoLast": 150.25,
      "prevClose": 149.50,
      "open": 149.80,
      "high": 150.50,
      "low": 149.70,
      "mid": 150.24,
      "volume": 1234567,
      "bidSize": 200,
      "bidPrice": 150.23,
      "askSize": 150,
      "askPrice": 150.26,
      "quoteTimestamp": "2020-01-01T12:34:56.789012Z",
      "lastSaleTimestamp": "2020-01-01T12:34:56.500000Z"
    }
  ]
}
```

**Top-level fields:**
- `service`: Always "iex" for IEX endpoint
- `messageType`: "A" = price quote/trade update, "H" = heartbeat
- `data`: Array of ticker updates (can contain multiple tickers)

**Data object fields:**
- `ticker`: Stock ticker symbol
- `timestamp`: ISO8601 timestamp with microsecond precision
- `last`: Last trade price
- `lastSize`: Last trade size (shares)
- `tngoLast`: Tiingo's last trade price
- `prevClose`: Previous day's closing price
- `open`: Today's opening price
- `high`: Today's high price
- `low`: Today's low price
- `mid`: Mid-price (bid + ask) / 2
- `volume`: Cumulative volume
- `bidSize`: Best bid size
- `bidPrice`: Best bid price
- `askSize`: Best ask size
- `askPrice`: Best ask price
- `quoteTimestamp`: Quote timestamp
- `lastSaleTimestamp`: Last sale timestamp

### Forex Top-of-Book Update

WebSocket messages from Forex endpoint:

```json
{
  "service": "fx",
  "messageType": "A",
  "data": [
    {
      "ticker": "eurusd",
      "quoteTimestamp": "2020-01-01T12:34:56.789012Z",
      "bidPrice": 1.1234,
      "bidSize": 1000000,
      "askPrice": 1.1236,
      "askSize": 1000000,
      "midPrice": 1.1235
    }
  ]
}
```

**Top-level fields:**
- `service`: Always "fx" for Forex endpoint
- `messageType`: "A" = new quote, "H" = heartbeat
- `data`: Array of FX pair updates

**Data object fields:**
- `ticker`: FX pair ticker (e.g., eurusd, gbpjpy)
- `quoteTimestamp`: ISO8601 timestamp with microsecond precision
- `bidPrice`: Best bid price
- `bidSize`: Bid size (notional amount)
- `askPrice`: Best ask price
- `askSize`: Ask size (notional amount)
- `midPrice`: (bid + ask) / 2 (null if either bid or ask is null)

### Crypto Top-of-Book Update

WebSocket messages from Crypto endpoint (structure similar to Forex):

```json
{
  "service": "crypto",
  "messageType": "A",
  "data": [
    {
      "ticker": "btcusd",
      "exchange": "binance",
      "quoteTimestamp": "2020-01-01T12:34:56.789Z",
      "bidPrice": 45000.50,
      "bidSize": 0.5,
      "askPrice": 45001.50,
      "askSize": 0.3,
      "midPrice": 45001.00,
      "lastPrice": 45000.75,
      "lastSize": 0.1
    }
  ]
}
```

**Top-level fields:**
- `service`: Always "crypto" for Crypto endpoint
- `messageType`: "A" = new quote, "H" = heartbeat
- `data`: Array of crypto pair updates (may include multiple exchanges)

**Data object fields:**
- `ticker`: Crypto pair (e.g., btcusd, ethusd)
- `exchange`: Exchange name (binance, coinbase, etc.)
- `quoteTimestamp`: ISO8601 timestamp
- `bidPrice`: Best bid price
- `bidSize`: Bid size (crypto units)
- `askPrice`: Best ask price
- `askSize`: Ask size (crypto units)
- `midPrice`: (bid + ask) / 2
- `lastPrice`: Last trade price
- `lastSize`: Last trade size

---

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client**: Yes (server sends heartbeat messages)
- **Client → Server**: Not required (client does not need to send pings)

### Message Format
- **Binary ping/pong frames**: No
- **Text messages**: No ("ping"/"pong" strings not used)
- **JSON messages**: Yes

### Heartbeat Message

Server periodically sends heartbeat to keep connection alive:

```json
{
  "service": "iex",
  "messageType": "H",
  "data": []
}
```

**Fields:**
- `service`: Endpoint identifier (iex, fx, or crypto)
- `messageType`: "H" = heartbeat
- `data`: Empty array

### Timing
- **Server ping interval**: Variable (server-determined)
- **Timeout**: Not explicitly documented (connection dropped if client disconnects)
- **Client must send ping**: No (server-initiated heartbeats only)
- **Client should handle heartbeats**: Yes (detect and ignore "H" messages)

### Example Flow

```
Server → Client: {"service":"iex","messageType":"H","data":[]}
Client: (no response required, continue listening)

Server → Client: {"service":"iex","messageType":"A","data":[{...}]}
Client: (process data update)

Server → Client: {"service":"iex","messageType":"H","data":[]}
Client: (no response required)
```

---

## Connection Limits

### Free Tier
- **Max connections per IP**: Not explicitly documented (reasonable use expected)
- **Max connections per API key**: Not explicitly documented
- **Max subscriptions per connection**: Unlimited (firehose provides all data)
- **Message rate limit**: No limit (firehose can deliver microsecond-resolution data)
- **Auto-disconnect after**: Not specified (likely 24 hours or connection failure)

### Premium Tiers
- Same as free tier (WebSocket access included in all tiers)
- Higher REST API rate limits do not directly affect WebSocket
- All tiers have access to full firehose data

### Important Notes
- Tiingo emphasizes that the firehose delivers **very high volumes** of data (microsecond resolution)
- Systems must be built to **scale and handle high message rates**
- Free tier has same WebSocket access as paid tiers
- ThresholdLevel parameter allows controlling data volume (if needed)

---

## Authentication

### Method
- **Message after connect**: Yes
- Subscribe message contains `authorization` field with API token
- No URL parameter authentication
- No separate auth handshake

### Auth Message Format

Authentication is embedded in the subscribe message:

```json
{
  "eventName": "subscribe",
  "authorization": "YOUR_API_TOKEN_HERE",
  "eventData": {
    "thresholdLevel": 5
  }
}
```

**Fields:**
- `eventName`: "subscribe"
- `authorization`: API token (from https://api.tiingo.com/account/api/token)
- `eventData`: Subscription parameters

### Auth Success/Failure

**Success**: Server immediately starts sending data messages (messageType "A")

**Failure**: Connection may be closed or error message returned (exact format not documented, likely connection drop or error JSON)

---

## Example Usage

### Connect and Subscribe (Python-like pseudocode)

```python
import websocket
import json

# Connect to IEX WebSocket
ws = websocket.WebSocketApp(
    "wss://api.tiingo.com/iex",
    on_message=on_message,
    on_error=on_error,
    on_close=on_close
)

def on_open(ws):
    # Subscribe with authentication
    subscribe_msg = {
        "eventName": "subscribe",
        "authorization": "YOUR_API_TOKEN",
        "eventData": {
            "thresholdLevel": 5  # All updates
        }
    }
    ws.send(json.dumps(subscribe_msg))

def on_message(ws, message):
    data = json.loads(message)

    if data["messageType"] == "H":
        # Heartbeat - ignore or log
        print("Heartbeat received")
    elif data["messageType"] == "A":
        # Price update - process data
        for ticker_data in data["data"]:
            print(f"{ticker_data['ticker']}: {ticker_data['last']}")

ws.on_open = on_open
ws.run_forever()
```

### Forex WebSocket Example

```python
# Connect to Forex WebSocket
ws = websocket.WebSocketApp("wss://api.tiingo.com/fx")

# Subscribe message (same pattern)
subscribe = {
    "eventName": "subscribe",
    "authorization": "YOUR_API_TOKEN",
    "eventData": {
        "thresholdLevel": 5
    }
}
```

### Crypto WebSocket Example

```python
# Connect to Crypto WebSocket
ws = websocket.WebSocketApp("wss://api.tiingo.com/crypto")

# Subscribe message (same pattern)
subscribe = {
    "eventName": "subscribe",
    "authorization": "YOUR_API_TOKEN",
    "eventData": {
        "thresholdLevel": 5
    }
}
```

---

## Advanced Configuration

### Threshold Level Tuning

- **thresholdLevel 5**: Maximum data (all top-of-book updates)
  - Use for: Low-latency trading, complete market data capture
  - Warning: Very high message volume (microsecond updates)

- **thresholdLevel 4**: Slightly filtered
  - Use for: High-frequency applications with some filtering

- **thresholdLevel 3**: Moderate filtering
  - Use for: Real-time applications with reduced bandwidth

- **thresholdLevel 1-2**: Heavy filtering
  - Use for: Applications that only need significant price changes
  - Lower bandwidth requirements

### Handling High Message Volume

The Tiingo WebSocket firehose can deliver extremely high message rates:

1. **Buffer messages**: Use queues to handle bursts
2. **Process asynchronously**: Don't block message reception
3. **Sample if needed**: Use lower thresholdLevel or client-side sampling
4. **Monitor performance**: Track message rates and processing latency
5. **Scale infrastructure**: Prepare for microsecond-resolution data streams

---

## Reconnection Strategy

Recommended reconnection logic (not officially documented):

1. **Detect disconnect**: Monitor connection state
2. **Wait before reconnect**: Exponential backoff (1s, 2s, 4s, 8s, max 60s)
3. **Reconnect**: Open new WebSocket connection
4. **Resubscribe**: Send subscribe message again with same parameters
5. **Resume data processing**: Continue from current state (no replay available)

**Note**: No official reconnection protocol documented. Follow best practices for WebSocket resilience.

---

## Data Continuity

- **No snapshot on connect**: WebSocket provides updates only (no initial state)
- **No replay/historical**: WebSocket is real-time only
- **Gaps on disconnect**: Use REST API to fill gaps after reconnection
- **Sequence numbers**: Not provided (no guaranteed ordering across reconnects)

---

## Summary

- **3 WebSocket endpoints**: IEX (stocks), Forex, Crypto
- **Simple subscribe pattern**: eventName, authorization, eventData
- **Unified message format**: service, messageType, data
- **Server heartbeats**: messageType "H" to keep connection alive
- **ThresholdLevel control**: Adjust data volume (5 = all updates)
- **Firehose access**: All tiers get full data stream (microsecond resolution)
- **High performance required**: Systems must handle high message rates
- **No separate private streams**: Authentication controls access, all data public
