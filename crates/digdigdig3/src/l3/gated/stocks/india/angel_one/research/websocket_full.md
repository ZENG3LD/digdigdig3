# Angel One SmartAPI - WebSocket Documentation

## Availability: Yes (WebSocket V2)

## Connection

### URLs
- **WebSocket V2 URL**: wss://smartapisocket.angelone.in/smart-stream
- **No separate public/private streams**: Single WebSocket connection for all data
- **No regional endpoints**: Single global WebSocket server

### Connection Process
1. **Obtain Feed Token**: Call REST API endpoint `/rest/secure/angelbroking/user/v1/getfeedToken` after login
2. **Connect to WebSocket URL**: wss://smartapisocket.angelone.in/smart-stream
3. **Authenticate**: Send authentication message with API key, client code, and feed token
4. **Subscribe to Channels**: Send subscription messages for desired symbols and modes
5. **Receive Data**: Handle incoming market data messages

### Authentication Requirements
- **AUTH_TOKEN**: JWT token from login
- **API_KEY**: API key from SmartAPI dashboard
- **CLIENT_CODE**: Angel One account client code
- **FEED_TOKEN**: Special token obtained from getFeedToken() REST endpoint

**Note**: Feed token is separate from the JWT authentication token and specifically required for WebSocket connections.

## WebSocket V2 Implementation

### Python Example
```python
from SmartApi.smartWebSocketV2 import SmartWebSocketV2

# Initialize WebSocket
sws = SmartWebSocketV2(AUTH_TOKEN, API_KEY, CLIENT_CODE, FEED_TOKEN)

# Define callbacks
def on_data(wsapp, message):
    print("Received:", message)

def on_open(wsapp):
    print("Connected")
    # Subscribe to symbols
    sws.subscribe(correlation_id, mode, token_list)

def on_error(wsapp, error):
    print("Error:", error)

def on_close(wsapp):
    print("Disconnected")

# Assign callbacks
sws.on_open = on_open
sws.on_data = on_data
sws.on_error = on_error
sws.on_close = on_close

# Connect
sws.connect()
```

## ALL Available Channels/Topics

WebSocket V2 uses a mode-based subscription system rather than named channels.

| Mode | Code | Description | Auth? | Free? | Update Frequency | Data Included |
|------|------|-------------|-------|-------|------------------|---------------|
| LTP | 1 | Last Traded Price | Yes | Yes | Real-time | LTP only |
| Quote | 2 | Quote data | Yes | Yes | Real-time | LTP, Open, High, Low, Close, Volume |
| Snap Quote | 3 | Snapshot quote | Yes | Yes | Real-time | Full market depth snapshot |
| Depth 20 | 4 | 20-level order book | Yes | Yes | Real-time | 20 best bid/ask levels with price & quantity |

### Mode Details

**Mode 1 (LTP)**:
- Minimal data - only last traded price
- Lowest bandwidth consumption
- Fastest updates

**Mode 2 (Quote)**:
- OHLC data + volume
- Current LTP
- Best bid/ask

**Mode 3 (Snap Quote)**:
- Complete market snapshot
- Full order book (5 levels typically)
- All ticker data

**Mode 4 (Depth 20)**:
- Extended order book depth
- 20 best bid levels
- 20 best ask levels
- Price and quantity for each level
- Introduced in WebSocket V2 (beta testing in 2024)

## Subscription Format

### Subscribe Message
```python
correlation_id = "your_correlation_id"  # Any string to track subscription
mode = 1  # 1=LTP, 2=Quote, 3=SnapQuote, 4=Depth20
token_list = [
    {
        "exchangeType": 1,  # 1=NSE, 2=NFO, 3=BSE, etc.
        "tokens": ["3045", "1594"]  # Symbol tokens
    }
]

sws.subscribe(correlation_id, mode, token_list)
```

### Exchange Type Codes
| Exchange | Code |
|----------|------|
| NSE | 1 |
| NFO | 2 |
| BSE | 3 |
| BFO | 4 |
| MCX | 5 |
| CDS | 7 |
| NCDEX | 13 |

### Token Format
Tokens are obtained from:
- **Instrument Master File**: https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json
- **SearchScrip API**: Search endpoint returns symbol token

### Unsubscribe Message
```python
correlation_id = "your_correlation_id"
mode = 1
token_list = [
    {
        "exchangeType": 1,
        "tokens": ["3045"]
    }
]

sws.unsubscribe(correlation_id, mode, token_list)
```

### Subscription Confirmation
Confirmation is not explicitly documented in a separate message format. The `on_open` callback indicates successful connection, and data flow confirms subscription success.

## Message Formats (for EVERY mode)

### Mode 1: LTP Update
```json
{
  "subscription_mode": 1,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 12345,
  "exchange_timestamp": 1234567890000,
  "last_traded_price": 50025
}
```

**Fields**:
- `subscription_mode`: Mode number (1)
- `exchange_type`: Exchange code
- `token`: Symbol token
- `sequence_number`: Message sequence
- `exchange_timestamp`: Exchange timestamp (Unix ms)
- `last_traded_price`: LTP in paise (divide by 100 for rupees)

### Mode 2: Quote Update
```json
{
  "subscription_mode": 2,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 12346,
  "exchange_timestamp": 1234567890000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 1234567,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50100,
  "low_price_of_the_day": 49900,
  "closed_price": 49975,
  "last_traded_timestamp": 1234567890,
  "open_interest": 0,
  "open_interest_change_percentage": 0
}
```

**Fields** (prices in paise):
- All LTP mode fields
- `last_traded_quantity`: Size of last trade
- `average_traded_price`: VWAP
- `volume_trade_for_the_day`: Total volume
- `total_buy_quantity`: Total buy orders quantity
- `total_sell_quantity`: Total sell orders quantity
- `open_price_of_the_day`: Open price
- `high_price_of_the_day`: High price
- `low_price_of_the_day`: Low price
- `closed_price`: Previous close
- `last_traded_timestamp`: Trade timestamp
- `open_interest`: OI (for derivatives)
- `open_interest_change_percentage`: OI change %

### Mode 3: Snap Quote Update
```json
{
  "subscription_mode": 3,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 12347,
  "exchange_timestamp": 1234567890000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 1234567,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50100,
  "low_price_of_the_day": 49900,
  "closed_price": 49975,
  "last_traded_timestamp": 1234567890,
  "open_interest": 0,
  "open_interest_change_percentage": 0,
  "upper_circuit_limit": 52500,
  "lower_circuit_limit": 47500,
  "52_week_high": 55000,
  "52_week_low": 40000,
  "best_5_buy_data": [
    {
      "flag": 0,
      "quantity": 1000,
      "price": 50020,
      "no_of_orders": 5
    },
    // ... 4 more levels
  ],
  "best_5_sell_data": [
    {
      "flag": 1,
      "quantity": 800,
      "price": 50030,
      "no_of_orders": 3
    },
    // ... 4 more levels
  ]
}
```

**Additional Fields**:
- All Quote mode fields
- `upper_circuit_limit`: Upper price limit
- `lower_circuit_limit`: Lower price limit
- `52_week_high`: 52-week high
- `52_week_low`: 52-week low
- `best_5_buy_data`: Top 5 bid levels
- `best_5_sell_data`: Top 5 ask levels

**Order Book Entry**:
- `flag`: 0=buy, 1=sell
- `quantity`: Total quantity at level
- `price`: Price level (in paise)
- `no_of_orders`: Number of orders at level

### Mode 4: Depth 20 Update
```json
{
  "subscription_mode": 4,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 12348,
  "exchange_timestamp": 1234567890000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 1234567,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50100,
  "low_price_of_the_day": 49900,
  "closed_price": 49975,
  "last_traded_timestamp": 1234567890,
  "open_interest": 0,
  "open_interest_change_percentage": 0,
  "upper_circuit_limit": 52500,
  "lower_circuit_limit": 47500,
  "52_week_high": 55000,
  "52_week_low": 40000,
  "best_20_buy_data": [
    {
      "flag": 0,
      "quantity": 1000,
      "price": 50020,
      "no_of_orders": 5
    },
    // ... 19 more levels
  ],
  "best_20_sell_data": [
    {
      "flag": 1,
      "quantity": 800,
      "price": 50030,
      "no_of_orders": 3
    },
    // ... 19 more levels
  ]
}
```

**Additional Fields** (compared to Mode 3):
- `best_20_buy_data`: Top 20 bid levels (instead of 5)
- `best_20_sell_data`: Top 20 ask levels (instead of 5)

## Order Update WebSocket

**Separate WebSocket Class**: `SmartWebSocketOrderUpdate`

### Purpose
Real-time order status updates for your own orders.

### Connection
```python
from SmartApi.smartWebSocketOrderUpdate import SmartWebSocketOrderUpdate

order_socket = SmartWebSocketOrderUpdate(AUTH_TOKEN, API_KEY, CLIENT_CODE, FEED_TOKEN)
order_socket.connect()
```

### Message Format
Order update messages contain order status changes in real-time (exact format not fully documented in available sources, similar to order book API response).

## Heartbeat / Ping-Pong

**CRITICAL**: Exact heartbeat mechanism not explicitly documented in available sources.

### Likely Implementation
Based on WebSocket V2 standard practices:

- **Server → Client ping**: Likely yes (standard WebSocket ping frames)
- **Client → Server ping**: May be required for keep-alive
- **Binary ping/pong frames**: Likely (standard WebSocket protocol)
- **Timeout**: Not publicly specified

### Connection Stability
- Callbacks available: `on_reconnect` and `on_no_reconnect`
- Automatic reconnection logic built into SDK
- Max reconnection attempts configurable

### Recommended Practice
- Implement connection monitoring
- Handle reconnection via SDK callbacks
- Track last message timestamp
- Reconnect if no data received for extended period (e.g., 60 seconds)

## Connection Limits

- **Max connections per IP**: Not publicly specified
- **Max connections per API key**: Not publicly specified
- **Max subscriptions per connection**: **1000 tokens** (documented limit)
- **Message rate limit**: Not publicly specified (server may throttle)
- **Auto-disconnect after**: Not publicly specified (sessions valid until midnight)

**Important**: If 1000 token subscription limit is exceeded, ticks won't be received for tokens beyond the limit.

## Authentication (WebSocket V2)

### Method
Authentication via parameters during WebSocket client initialization.

### Authentication Parameters
```python
SmartWebSocketV2(AUTH_TOKEN, API_KEY, CLIENT_CODE, FEED_TOKEN)
```

**Parameters**:
1. **AUTH_TOKEN**: JWT token from REST API login (`generateSession` response)
2. **API_KEY**: API key from SmartAPI dashboard
3. **CLIENT_CODE**: Angel One trading account client code
4. **FEED_TOKEN**: Special token from `getfeedToken()` REST endpoint

### Getting Feed Token
```python
# After successful login via REST API
feed_token = smartApi.getfeedToken()
```

**Note**: Feed token must be obtained fresh for each WebSocket session.

### Auth Success/Failure
- **Success**: `on_open` callback triggered, ready to subscribe
- **Failure**: `on_error` callback triggered with error details

## Error Handling

### Callbacks
- `on_error(wsapp, error)`: Handle connection and data errors
- `on_close(wsapp)`: Handle connection closure
- `on_reconnect(wsapp, attempt_count)`: Reconnection attempt
- `on_no_reconnect(wsapp)`: Reconnection failed after max attempts

### Common Issues
1. **Invalid Feed Token**: Ensure fresh feed token from REST API
2. **Token Limit Exceeded**: Keep subscriptions under 1000 tokens
3. **Connection Drops**: Implement reconnection logic via callbacks
4. **No Data Received**: Check subscription format and token validity

## WebSocket V2 Features (2024-2026)

### Recent Enhancements
- **Depth 20**: 20-level order book (beta testing launched in 2024)
- **Enhanced stability**: Improved reconnection handling
- **Multiple modes**: Flexible subscription modes for bandwidth optimization

### Best Practices
1. **Use appropriate mode**: LTP for minimal bandwidth, Depth 20 for full book
2. **Manage subscriptions**: Stay under 1000 token limit
3. **Handle reconnection**: Implement robust reconnection logic
4. **Monitor timestamps**: Track exchange timestamps for data freshness
5. **Price conversion**: Remember prices are in paise (divide by 100 for rupees)

## Token List Management

### Obtaining Tokens
Download instrument master:
```python
import requests
import pandas as pd

url = 'https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json'
data = requests.get(url).json()
df = pd.DataFrame(data)

# Filter for specific symbols
nse_stocks = df[df['exch_seg'] == 'NSE']
sbin_token = df[(df['symbol'] == 'SBIN-EQ') & (df['exch_seg'] == 'NSE')]['token'].values[0]
```

### Feed Format
Tokens can also be formatted as string for certain use cases:
```
"nse_cm|17963&nse_cm|3499"
```

Format: `exchange_segment|token&exchange_segment|token...`

## Notes

1. **Separate Feed Token**: Feed token is different from JWT auth token
2. **1000 Token Limit**: Hard limit on concurrent subscriptions
3. **Prices in Paise**: All price values are in paise (Indian cents), divide by 100 for rupees
4. **Real-time Updates**: All modes provide real-time market data
5. **Free for All**: WebSocket access included free for all Angel One SmartAPI users
6. **Depth 20 Beta**: Extended depth feature launched in 2024, now stable
7. **No Historical via WebSocket**: WebSocket is real-time only, use REST API for historical data
8. **Order Updates**: Separate WebSocket class for order status updates
