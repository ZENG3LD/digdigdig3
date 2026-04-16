# Upstox - WebSocket Documentation

## Availability: Yes

Upstox provides two separate WebSocket feeds:
1. **Market Data Feed** - Real-time market quotes, trades, depth
2. **Portfolio Stream Feed** - Order, position, and holding updates

---

# Market Data Feed WebSocket

## Connection

### URLs
- Market Data Feed: wss://api.upstox.com/v2/feed/market-data-feed/protobuf
- Authorization endpoint: GET https://api.upstox.com/v2/feed/market-data-feed/authorize

### Connection Process
1. Get authorized WebSocket URL via REST API: GET /v2/feed/market-data-feed/authorize
2. Connect to the returned WebSocket URL (includes auth token)
3. WebSocket uses automatic redirection (client must support `followRedirects`)
4. Connection established, ready to subscribe
5. **IMPORTANT:** Messages are in **binary Protocol Buffers format**, not JSON

### Authentication
- **Method:** Bearer token in Authorization header when calling authorize endpoint
- Headers for authorize request:
  - `Authorization: Bearer {access_token}`
  - `Accept: */*`
- WebSocket URL returned contains embedded auth token
- No additional auth needed after connecting to returned URL

---

## ALL Available Channels/Topics

Market Data Feed V3 supports real-time streaming with multiple data modes:

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Instrument Limit |
|---------------|------|-------------|-------|-------|------------------|------------------|
| ltpc | Public | Last Trade Price & Close | Yes | Yes | Real-time | 5000 per mode |
| option_greeks | Public | Option Greeks (delta, theta, etc.) | Yes | Yes | Real-time | 3000 per mode |
| full | Public | LTPC + 5 depth levels + metadata + Greeks | Yes | Yes | Real-time | 2000 per mode |
| full_d30 | Premium | LTPC + 30 depth levels + metadata + Greeks | Yes | Plus only | Real-time | 50 (Plus users) |

**Connection Limits:**
- Standard users: 2 WebSocket connections max
- Upstox Plus users: 5 WebSocket connections max
- Standard: 2000 combined instrument limit across all modes
- Plus: 1500 combined instrument limit for Full D30

---

## Subscription Format

### Subscribe Message (Binary Protocol Buffers)
Since the API uses binary Protocol Buffers, here's the conceptual JSON structure (actual message is binary):

```json
{
  "guid": "13syxu852ztodyqncwt0",
  "method": "sub",
  "data": {
    "mode": "full",
    "instrumentKeys": ["NSE_EQ|INE669E01016", "BSE_EQ|INE002A01018"]
  }
}
```

**Methods:**
- `sub` - Subscribe to instruments
- `unsub` - Unsubscribe from instruments
- `change_mode` - Change data mode for subscribed instruments

**Modes:**
- `ltpc` - Latest Trade Price & Close only
- `option_greeks` - Option Greeks data
- `full` - Full market data with 5 depth levels
- `full_d30` - Full data with 30 depth levels (Upstox Plus only)

### Unsubscribe Message
```json
{
  "guid": "unique-request-id",
  "method": "unsub",
  "data": {
    "mode": "full",
    "instrumentKeys": ["NSE_EQ|INE669E01016"]
  }
}
```

### Change Mode Message
```json
{
  "guid": "unique-request-id",
  "method": "change_mode",
  "data": {
    "mode": "ltpc",
    "instrumentKeys": ["NSE_EQ|INE669E01016"]
  }
}
```

---

## Message Formats (for EVERY channel)

**IMPORTANT:** All WebSocket messages are in **binary Protocol Buffers format**, not JSON text.

### First Message: Market Status (market_info)
Provides current status of market segments:

```json
{
  "type": "market_info",
  "segments": {
    "NSE_EQ": "open",
    "BSE_EQ": "closed",
    "NSE_FO": "open",
    "BSE_FO": "pre_open",
    "MCX_FO": "open",
    "NSE_INDEX": "open",
    "BSE_INDEX": "open"
  }
}
```

**Market Status Values:**
- `open` - Regular trading
- `closed` - Market closed
- `pre_open` - Pre-market session
- `post_close` - Post-market

### Second Message: Data Snapshot
Initial snapshot of current market state for subscribed instruments

### Subsequent Messages: Live Feed Updates

#### LTPC Mode Update
```json
{
  "type": "live_feed",
  "mode": "ltpc",
  "feeds": {
    "NSE_EQ|INE669E01016": {
      "ltpc": {
        "ltp": 2750.50,
        "ltt": "2024-01-26T10:15:30+05:30",
        "ltq": 100,
        "cp": 2725.00
      }
    }
  }
}
```

**LTPC Fields:**
- `ltp` - Last Traded Price
- `ltt` - Last Trade Time (ISO 8601)
- `ltq` - Last Trade Quantity
- `cp` - Close Price (previous day)

#### Full Mode Update (5 Depth Levels)
```json
{
  "type": "live_feed",
  "mode": "full",
  "feeds": {
    "NSE_EQ|INE669E01016": {
      "ltpc": {
        "ltp": 2750.50,
        "ltt": "2024-01-26T10:15:30+05:30",
        "ltq": 100,
        "cp": 2725.00
      },
      "depth": {
        "buy": [
          {"price": 2750.25, "quantity": 500, "orders": 5},
          {"price": 2750.00, "quantity": 1000, "orders": 10},
          {"price": 2749.75, "quantity": 750, "orders": 8},
          {"price": 2749.50, "quantity": 600, "orders": 6},
          {"price": 2749.25, "quantity": 450, "orders": 4}
        ],
        "sell": [
          {"price": 2750.50, "quantity": 300, "orders": 3},
          {"price": 2750.75, "quantity": 800, "orders": 7},
          {"price": 2751.00, "quantity": 1200, "orders": 12},
          {"price": 2751.25, "quantity": 650, "orders": 6},
          {"price": 2751.50, "quantity": 500, "orders": 5}
        ]
      },
      "ohlc": {
        "open": 2730.00,
        "high": 2755.00,
        "low": 2728.50,
        "close": 2725.00
      },
      "volume": 1234567,
      "oi": 456789,
      "total_buy_qty": 123456,
      "total_sell_qty": 234567,
      "avg_price": 2742.35,
      "lower_circuit": 2588.75,
      "upper_circuit": 2861.25,
      "oi_day_high": 478000,
      "oi_day_low": 445000
    }
  }
}
```

**Full Mode Fields:**
- `ltpc` - Last trade price & close
- `depth.buy` - Bid side market depth (5 levels)
- `depth.sell` - Ask side market depth (5 levels)
- `ohlc` - Open, High, Low, Close
- `volume` - Total traded volume
- `oi` - Open Interest (for F&O)
- `total_buy_qty` - Total buy quantity in orderbook
- `total_sell_qty` - Total sell quantity in orderbook
- `avg_price` - Average traded price
- `lower_circuit` - Lower circuit limit
- `upper_circuit` - Upper circuit limit
- `oi_day_high` - Highest OI during day
- `oi_day_low` - Lowest OI during day

#### Full D30 Mode Update (30 Depth Levels)
Same as Full mode but with 30 levels of market depth instead of 5 (Upstox Plus only)

#### Option Greeks Mode Update
```json
{
  "type": "live_feed",
  "mode": "option_greeks",
  "feeds": {
    "NSE_FO|12345": {
      "ltpc": {
        "ltp": 125.50,
        "ltt": "2024-01-26T10:15:30+05:30",
        "ltq": 50,
        "cp": 123.00
      },
      "greeks": {
        "delta": 0.5234,
        "theta": -0.0123,
        "gamma": 0.0045,
        "vega": 0.1234,
        "rho": 0.0234,
        "iv": 18.45
      },
      "spot_price": 18500.00
    }
  }
}
```

**Option Greeks Fields:**
- `delta` - Rate of change of option price vs underlying
- `theta` - Time decay
- `gamma` - Rate of change of delta
- `vega` - Sensitivity to volatility
- `rho` - Sensitivity to interest rate
- `iv` - Implied Volatility (%)
- `spot_price` - Underlying spot price

---

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client ping:** Yes
- **Client → Server ping:** No (not required, but optional)

### Message Format
- **Binary ping/pong frames:** Yes (standard WebSocket ping/pong)
- **Text messages:** No
- **JSON messages:** No

### Timing
- **Ping interval:** Server sends periodic ping frames
- **Timeout:** Connection closed if client doesn't respond to ping
- **Client must send ping:** Not required (auto-handled by WebSocket libraries)

### Example
Standard WebSocket libraries automatically handle ping/pong frames. No manual implementation needed.

**Note:** "The API sends periodic `ping` frames to maintain connection aliveness; standard WebSocket libraries automatically respond with `pong` frames, requiring no manual handling."

---

## Connection Limits

### Market Data Feed
- **Max connections per IP:** Not specified
- **Max connections per API key (Standard):** 2
- **Max connections per API key (Plus):** 5
- **Max subscriptions per connection:**
  - LTPC mode: 5000 instruments
  - Option Greeks mode: 3000 instruments
  - Full mode: 2000 instruments
  - Full D30 mode: 50 instruments (Plus only)
- **Combined limit (Standard):** 2000 instruments across all modes
- **Combined limit (Plus):** 1500 instruments for Full D30
- **Message rate limit:** Not specified
- **Auto-disconnect after:** Not specified

---

# Portfolio Stream Feed WebSocket

## Connection

### URLs
- Portfolio Stream Feed: wss://api.upstox.com/v2/feed/portfolio-stream-feed
- Authorization endpoint: GET https://api.upstox.com/v2/feed/portfolio-stream-feed/authorize

### Connection Process
1. Get authorized WebSocket URL: GET /v2/feed/portfolio-stream-feed/authorize?update_types=order,position,holding
2. Connect to returned WebSocket URL (with followRedirects support)
3. Connection established, start receiving updates
4. No explicit subscription needed - updates are pushed automatically

### Authentication
- **Method:** Bearer token via authorize endpoint
- Headers:
  - `Authorization: Bearer {access_token}`
  - `Accept: */*`
- WebSocket URL contains embedded auth

---

## Available Update Types

| Update Type | Description | Query Parameter | Auth? | Free? |
|-------------|-------------|-----------------|-------|-------|
| order | Order status updates | update_types=order | Yes | Paid |
| gtt_order | GTT order updates | update_types=gtt_order | Yes | Paid |
| position | Position updates | update_types=position | Yes | Paid |
| holding | Holdings updates | update_types=holding | Yes | Paid |

**Query Parameter Format:**
- Single: `?update_types=order`
- Multiple: `?update_types=order,position,holding` (URL-encoded: `order%2Cposition%2Cholding`)
- Default: `order` (if not specified)

---

## Message Formats

### Order Update
```json
{
  "type": "order",
  "data": {
    "order_id": "240126000123456",
    "trading_symbol": "RELIANCE",
    "exchange": "NSE",
    "instrument_token": "NSE_EQ|INE002A01018",
    "product": "D",
    "order_type": "LIMIT",
    "transaction_type": "BUY",
    "quantity": 10,
    "disclosed_quantity": 0,
    "price": 2750.00,
    "trigger_price": 0,
    "validity": "DAY",
    "status": "complete",
    "status_message": "Order executed",
    "filled_quantity": 10,
    "pending_quantity": 0,
    "average_price": 2748.50,
    "order_timestamp": "2024-01-26T09:30:15+05:30",
    "exchange_timestamp": "26-Jan-2024 09:30:15",
    "exchange_order_id": "1100000012345678",
    "parent_order_id": "",
    "is_amo": false,
    "tag": "my-strategy-001"
  }
}
```

**Order Status Values:**
- `open pending` - Order placed, awaiting exchange
- `validation pending` - Order validation in progress
- `put order req received` - Modification request received
- `trigger pending` - SL order waiting for trigger
- `open` - Order open at exchange
- `complete` - Fully executed
- `rejected` - Rejected by exchange
- `cancelled` - Cancelled
- `after market order req received` - AMO received

### GTT Order Update
```json
{
  "type": "gtt_order",
  "data": {
    "gtt_order_id": "GTT123456",
    "instrument_key": "NSE_EQ|INE669E01016",
    "product": "D",
    "order_type": "LIMIT",
    "transaction_type": "BUY",
    "quantity": 5,
    "price": 0,
    "trigger_price": 2700.00,
    "status": "active",
    "created_at": "2024-01-26T08:00:00+05:30",
    "updated_at": "2024-01-26T08:00:00+05:30",
    "expires_at": "2024-02-26T15:30:00+05:30",
    "rules": [
      {
        "id": 1,
        "strategy": "ENTRY",
        "status": "pending",
        "trigger_price": 2700.00,
        "order_type": "LIMIT",
        "price": 2705.00,
        "quantity": 5
      },
      {
        "id": 2,
        "strategy": "TARGET",
        "status": "pending",
        "trigger_price": 2800.00,
        "order_type": "LIMIT",
        "price": 2795.00,
        "quantity": 5
      },
      {
        "id": 3,
        "strategy": "STOPLOSS",
        "status": "pending",
        "trigger_price": 2650.00,
        "order_type": "SL",
        "price": 2645.00,
        "quantity": 5
      }
    ]
  }
}
```

**GTT Strategies:**
- `ENTRY` - Entry trigger
- `TARGET` - Target/profit booking
- `STOPLOSS` - Stop loss

**GTT Status:**
- `active` - Active, waiting for trigger
- `triggered` - Triggered, order placed
- `cancelled` - Cancelled by user
- `expired` - Expired without trigger

### Position Update
```json
{
  "type": "position",
  "data": {
    "exchange": "NSE",
    "product": "I",
    "trading_symbol": "NIFTY24JAN18500CE",
    "instrument_token": "NSE_FO|54321",
    "quantity": 50,
    "last_price": 125.50,
    "pnl": 2500.00,
    "unrealised": 2500.00,
    "realised": 0.00,
    "day_buy_quantity": 50,
    "day_buy_value": 6000.00,
    "day_buy_price": 120.00,
    "day_sell_quantity": 0,
    "day_sell_value": 0.00,
    "day_sell_price": 0.00,
    "overnight_quantity": 0,
    "overnight_buy_amount": 0.00,
    "overnight_sell_amount": 0.00,
    "multiplier": 1.0,
    "average_price": 120.00
  }
}
```

**Position Products:**
- `I` - Intraday
- `D` - Delivery
- `CO` - Cover Order

### Holding Update
```json
{
  "type": "holding",
  "data": {
    "trading_symbol": "RELIANCE",
    "exchange": "NSE",
    "instrument_token": "NSE_EQ|INE002A01018",
    "isin": "INE002A01018",
    "product": "D",
    "quantity": 100,
    "average_price": 2500.00,
    "last_price": 2750.00,
    "pnl": 25000.00,
    "collateral_quantity": 100,
    "collateral_type": "WC",
    "company_name": "Reliance Industries Limited",
    "close_price": 2725.00,
    "haircut": 0.10,
    "t1_quantity": 0,
    "buy_quantity": 100,
    "sell_quantity": 0
  }
}
```

**Collateral Types:**
- `WC` - With Collateral
- `WOC` - Without Collateral

---

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client ping:** Yes
- **Client → Server ping:** Optional

### Message Format
- **Binary ping/pong frames:** Yes (standard WebSocket)
- Auto-handled by WebSocket libraries

### Timing
- **Ping interval:** Automatic (server-initiated)
- **Timeout:** Connection closed if no pong response
- **Client must send ping:** No (auto-handled)

---

## Connection Limits

### Portfolio Stream Feed
- **Max connections per API key:** Not specified
- **Message rate:** Real-time (as events occur)
- **Auto-disconnect:** No automatic disconnect

---

## Authentication (for private channels)

Both WebSocket feeds are private and require authentication.

### Method
1. Call REST authorize endpoint with Bearer token
2. Receive WebSocket URL with embedded auth token
3. Connect to WebSocket URL (no additional auth needed)

### Auth Flow
```bash
# Step 1: Get WebSocket URL
GET https://api.upstox.com/v2/feed/market-data-feed/authorize
Authorization: Bearer {access_token}
Accept: */*

# Response
{
  "status": "success",
  "data": {
    "authorizedRedirectUri": "wss://api.upstox.com/v2/feed/market-data-feed/protobuf?token=..."
  }
}

# Step 2: Connect to WebSocket
Connect to: wss://api.upstox.com/v2/feed/market-data-feed/protobuf?token=...
```

### Requirements
- Valid OAuth 2.0 access token
- WebSocket client must support automatic redirects (`followRedirects`)
- Connection must use `wss://` (secure WebSocket)

---

## Special Notes

1. **Binary Protocol Buffers:** Market Data Feed uses binary format, not JSON text
2. **Auto-reconnect:** Client should implement reconnection logic
3. **Redirect Support:** Client must support automatic WebSocket redirects
4. **Market Status:** First message always contains market segment statuses
5. **Combined Limits:** Instrument subscriptions count across all modes
6. **Real-time Sync:** Updates sync across mobile app, web, and API
7. **Timestamp Formats:** Both ISO 8601 and exchange formats provided
