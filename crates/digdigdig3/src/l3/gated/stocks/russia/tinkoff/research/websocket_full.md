# Tinkoff Invest API - WebSocket Documentation

## Availability: Yes

## Connection

### URLs
- **Production**: `wss://invest-public-api.tinkoff.ru/ws/`
- **Sandbox**: Not available via WebSocket (use gRPC for sandbox)
- **Regional**: Single global endpoint

### Connection Process
1. Establish WebSocket connection to `wss://invest-public-api.tinkoff.ru/ws/`
2. Send authentication in one of two ways:
   - **Header**: `Authorization: Bearer <token>`
   - **Protocol header**: `Web-Socket-Protocol: json, <token>`
3. Server sends welcome/ping message with timestamp
4. Subscribe to desired channels via subscription messages
5. Receive real-time data updates

### Protocol Header Requirements
- **Required**: `Web-Socket-Protocol: json` (or `json-proto` for proto-compatible field names)
- **Message format**: JSON (supports both camelCase and snake_case)

## ALL Available Channels/Topics

| Channel/Topic | Type | Description | Auth | Free | Update Frequency | Subscription Message |
|---------------|------|-------------|------|------|------------------|---------------------|
| candles | Public | Real-time OHLC updates | Yes | Yes | Per candle close | `{"subscribeCandlesRequest": {...}}` |
| orderbook | Public | Order book L2 updates | Yes | Yes | Real-time | `{"subscribeOrderBookRequest": {...}}` |
| trades | Public | Anonymous trade feed | Yes | Yes | Real-time | `{"subscribeTradesRequest": {...}}` |
| info | Public | Instrument trading status | Yes | Yes | On status change | `{"subscribeInfoRequest": {...}}` |
| lastPrice | Public | Last trade price updates | Yes | Yes | Real-time | `{"subscribeLastPriceRequest": {...}}` |
| positions | Private | Portfolio positions stream | Yes | Yes | On position change | `{"accounts": ["account_id"]}` (PositionsStream) |
| portfolio | Private | Portfolio holdings stream | Yes | Yes | On portfolio change | `{"accounts": ["account_id"]}` (PortfolioStream) |
| trades_stream | Private | Order execution events | Yes | Yes | On trade execution | `{"accounts": ["account_id"]}` (TradesStream) |

**Important Notes**:
- All channels require authentication (Bearer token)
- All data is free for Tinkoff Investments clients
- No separate public/private WebSocket URLs - authentication determines access
- Dynamic rate limiting applies (based on trading activity)

## Subscription Format

### Subscribe to Candles

**Subscribe Message**:
```json
{
  "subscribeCandlesRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_SUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88",
        "interval": "SUBSCRIPTION_INTERVAL_ONE_MINUTE"
      },
      {
        "instrumentId": "6afa6f80-03a7-4d83-9cf0-c19d7d366297",
        "interval": "SUBSCRIPTION_INTERVAL_FIVE_MINUTES"
      }
    ]
  }
}
```

**Unsubscribe Message**:
```json
{
  "subscribeCandlesRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_UNSUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88",
        "interval": "SUBSCRIPTION_INTERVAL_ONE_MINUTE"
      }
    ]
  }
}
```

**Subscription Intervals** (CandleInterval enum):
- `SUBSCRIPTION_INTERVAL_ONE_MINUTE` - 1-minute candles
- `SUBSCRIPTION_INTERVAL_FIVE_MINUTES` - 5-minute candles
- Additional intervals available (15m, 1h, etc.) - check current API docs

### Subscribe to Order Book

**Subscribe Message**:
```json
{
  "subscribeOrderBookRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_SUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88",
        "depth": 10
      }
    ]
  }
}
```

**Depth options**: 1, 10, 20, 30, 40, 50 levels

### Subscribe to Trades

**Subscribe Message**:
```json
{
  "subscribeTradesRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_SUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88"
      },
      {
        "instrumentId": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
      }
    ]
  }
}
```

### Subscribe to Trading Status (Info)

**Subscribe Message**:
```json
{
  "subscribeInfoRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_SUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88"
      }
    ]
  }
}
```

### Subscribe to Last Price

**Subscribe Message**:
```json
{
  "subscribeLastPriceRequest": {
    "subscriptionAction": "SUBSCRIPTION_ACTION_SUBSCRIBE",
    "instruments": [
      {
        "figi": "BBG004730N88"
      }
    ]
  }
}
```

### Subscription Confirmation

**Generic confirmation format**:
```json
{
  "subscribeCandlesResponse": {
    "trackingId": "unique-tracking-id-12345",
    "candlesSubscriptions": [
      {
        "figi": "BBG004730N88",
        "interval": "SUBSCRIPTION_INTERVAL_ONE_MINUTE",
        "subscriptionStatus": "SUBSCRIPTION_STATUS_SUCCESS"
      }
    ]
  }
}
```

**Subscription Status Enum**:
- `SUBSCRIPTION_STATUS_UNSPECIFIED` - Undefined
- `SUBSCRIPTION_STATUS_SUCCESS` - Successfully subscribed
- `SUBSCRIPTION_STATUS_INSTRUMENT_NOT_FOUND` - Invalid FIGI/instrument_id
- `SUBSCRIPTION_STATUS_SUBSCRIPTION_ACTION_IS_INVALID` - Invalid action
- `SUBSCRIPTION_STATUS_DEPTH_IS_INVALID` - Invalid depth (orderbook)
- `SUBSCRIPTION_STATUS_INTERVAL_IS_INVALID` - Invalid interval (candles)

## Message Formats (for EVERY channel)

### Candle Update

```json
{
  "candle": {
    "figi": "BBG004730N88",
    "interval": "SUBSCRIPTION_INTERVAL_ONE_MINUTE",
    "open": {
      "units": 150,
      "nano": 250000000
    },
    "high": {
      "units": 150,
      "nano": 500000000
    },
    "low": {
      "units": 150,
      "nano": 100000000
    },
    "close": {
      "units": 150,
      "nano": 350000000
    },
    "volume": 1234567,
    "time": "2026-01-26T10:30:00Z",
    "lastTradeTs": "2026-01-26T10:30:59.999Z",
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

**Fields**:
- `figi` - Instrument FIGI
- `interval` - Candle interval
- `open`, `high`, `low`, `close` - OHLC prices (Quotation type)
- `volume` - Volume in lots
- `time` - Candle start time
- `lastTradeTs` - Last trade timestamp in candle
- `instrumentUid` - Instrument UID

### Trade Update

```json
{
  "trade": {
    "figi": "BBG004730N88",
    "direction": "TRADE_DIRECTION_BUY",
    "price": {
      "units": 150,
      "nano": 250000000
    },
    "quantity": 100,
    "time": "2026-01-26T10:30:45.123Z",
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

**Fields**:
- `figi` - Instrument FIGI
- `direction` - Trade direction: `TRADE_DIRECTION_BUY` or `TRADE_DIRECTION_SELL`
- `price` - Trade execution price (Quotation)
- `quantity` - Quantity in lots
- `time` - Trade timestamp
- `instrumentUid` - Instrument UID

### Order Book Snapshot

```json
{
  "orderbook": {
    "figi": "BBG004730N88",
    "depth": 10,
    "isConsistent": true,
    "bids": [
      {
        "price": {
          "units": 150,
          "nano": 240000000
        },
        "quantity": 1000
      },
      {
        "price": {
          "units": 150,
          "nano": 230000000
        },
        "quantity": 2500
      }
    ],
    "asks": [
      {
        "price": {
          "units": 150,
          "nano": 250000000
        },
        "quantity": 800
      },
      {
        "price": {
          "units": 150,
          "nano": 260000000
        },
        "quantity": 1500
      }
    ],
    "time": "2026-01-26T10:30:45.123Z",
    "limitUp": {
      "units": 165,
      "nano": 0
    },
    "limitDown": {
      "units": 135,
      "nano": 0
    },
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

**Fields**:
- `figi` - Instrument FIGI
- `depth` - Order book depth (levels per side)
- `isConsistent` - Data consistency flag
- `bids` - Array of bid levels (buy orders)
- `asks` - Array of ask levels (sell orders)
- `time` - Snapshot timestamp
- `limitUp` - Upper price limit
- `limitDown` - Lower price limit
- `instrumentUid` - Instrument UID

**Note**: Full snapshot sent on subscription and updates, not delta updates.

### Trading Status Update

```json
{
  "tradingStatus": {
    "figi": "BBG004730N88",
    "tradingStatus": "SECURITY_TRADING_STATUS_NORMAL_TRADING",
    "time": "2026-01-26T10:30:00Z",
    "limitOrderAvailableFlag": true,
    "marketOrderAvailableFlag": true,
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

**SecurityTradingStatus enum** (17 possible values):
- `SECURITY_TRADING_STATUS_UNSPECIFIED` - Undefined
- `SECURITY_TRADING_STATUS_NOT_AVAILABLE_FOR_TRADING` - Not tradable
- `SECURITY_TRADING_STATUS_OPENING_PERIOD` - Opening auction
- `SECURITY_TRADING_STATUS_CLOSING_PERIOD` - Closing auction
- `SECURITY_TRADING_STATUS_BREAK_IN_TRADING` - Trading halt
- `SECURITY_TRADING_STATUS_NORMAL_TRADING` - Normal trading
- `SECURITY_TRADING_STATUS_CLOSING_AUCTION` - Closing auction period
- `SECURITY_TRADING_STATUS_DARK_POOL_AUCTION` - Dark pool auction
- `SECURITY_TRADING_STATUS_DISCRETE_AUCTION` - Discrete auction
- `SECURITY_TRADING_STATUS_OPENING_AUCTION_PERIOD` - Opening auction period
- `SECURITY_TRADING_STATUS_TRADING_AT_CLOSING_AUCTION_PRICE` - At closing price
- `SECURITY_TRADING_STATUS_SESSION_ASSIGNED` - Session assigned
- `SECURITY_TRADING_STATUS_SESSION_CLOSE` - Session closed
- `SECURITY_TRADING_STATUS_SESSION_OPEN` - Session open
- `SECURITY_TRADING_STATUS_DEALER_NORMAL_TRADING` - Dealer normal
- `SECURITY_TRADING_STATUS_DEALER_BREAK_IN_TRADING` - Dealer halt
- `SECURITY_TRADING_STATUS_DEALER_NOT_AVAILABLE_FOR_TRADING` - Dealer not available

### Last Price Update

```json
{
  "lastPrice": {
    "figi": "BBG004730N88",
    "price": {
      "units": 150,
      "nano": 250000000
    },
    "time": "2026-01-26T10:30:45.123Z",
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

### Portfolio Stream Update

```json
{
  "portfolio": {
    "accountId": "2000123456",
    "totalAmountShares": {
      "currency": "RUB",
      "units": 500000,
      "nano": 0
    },
    "totalAmountBonds": {
      "currency": "RUB",
      "units": 100000,
      "nano": 0
    },
    "totalAmountEtf": {
      "currency": "RUB",
      "units": 50000,
      "nano": 0
    },
    "totalAmountCurrencies": {
      "currency": "RUB",
      "units": 25000,
      "nano": 0
    },
    "totalAmountFutures": {
      "currency": "RUB",
      "units": 0,
      "nano": 0
    },
    "expectedYield": {
      "units": 15000,
      "nano": 500000000
    },
    "positions": [
      {
        "figi": "BBG004730N88",
        "instrumentType": "share",
        "quantity": {
          "units": 100,
          "nano": 0
        },
        "averagePositionPrice": {
          "currency": "RUB",
          "units": 145,
          "nano": 0
        },
        "expectedYield": {
          "units": 525,
          "nano": 0
        },
        "currentNkd": {
          "currency": "RUB",
          "units": 0,
          "nano": 0
        },
        "currentPrice": {
          "currency": "RUB",
          "units": 150,
          "nano": 250000000
        },
        "averagePositionPriceFifo": {
          "currency": "RUB",
          "units": 145,
          "nano": 0
        }
      }
    ]
  }
}
```

### Positions Stream Update

```json
{
  "position": {
    "accountId": "2000123456",
    "money": [
      {
        "availableValue": {
          "currency": "RUB",
          "units": 25000,
          "nano": 0
        },
        "blockedValue": {
          "currency": "RUB",
          "units": 5000,
          "nano": 0
        }
      }
    ],
    "securities": [
      {
        "figi": "BBG004730N88",
        "blocked": 0,
        "balance": 100,
        "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
      }
    ],
    "futures": [],
    "options": []
  }
}
```

### Trade Execution Event (TradesStream)

```json
{
  "orderTrades": {
    "orderId": "12345678",
    "createdAt": "2026-01-26T10:30:00Z",
    "direction": "ORDER_DIRECTION_BUY",
    "figi": "BBG004730N88",
    "trades": [
      {
        "dateTime": "2026-01-26T10:30:45.123Z",
        "price": {
          "units": 150,
          "nano": 250000000
        },
        "quantity": 50
      },
      {
        "dateTime": "2026-01-26T10:30:46.456Z",
        "price": {
          "units": 150,
          "nano": 260000000
        },
        "quantity": 50
      }
    ],
    "accountId": "2000123456",
    "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
  }
}
```

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client ping**: Yes
- **Client → Server ping**: Not required (but client can send)

### Message Format
- **Text messages**: JSON format
- **Server ping**: Contains timestamp

### Timing
- **Ping interval**: Variable (sent by server periodically)
- **Timeout**: Connection may be closed if ping not responded to
- **Client must respond**: Not explicitly required, but recommended for connection health

### Example

**Server → Client**:
```json
{
  "ping": {
    "time": "2026-01-26T10:30:00.000Z"
  }
}
```

**Client → Server** (optional response):
```json
{
  "pong": {
    "time": "2026-01-26T10:30:00.000Z"
  }
}
```

**Note**: Connection health monitoring uses these ping messages with server timestamps.

## Connection Limits

- **Max connections per IP**: Dynamic (based on trading activity)
- **Max connections per API key**: Dynamic (rate limiting applies)
- **Max subscriptions per connection**: Not explicitly documented (likely high limit)
- **Message rate limit**: Dynamic (part of overall rate limiting)
- **Auto-disconnect after**: Not specified (idle connections may timeout)

**Note**: Active traders get higher limits; low-volume traders have standard limits.

## Authentication (for private channels)

### Method
- **Header-based**: `Authorization: Bearer <token>`
- **Or protocol header**: `Web-Socket-Protocol: json, <token>`

### Auth Message Format
Authentication is done during connection handshake via headers, NOT via message after connect.

**Header example**:
```
Authorization: Bearer t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
Web-Socket-Protocol: json
```

**Alternative protocol header**:
```
Web-Socket-Protocol: json, t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```

### Auth Success/Failure

**Success**: Connection established, server sends ping/welcome message

**Failure**: Connection closed with error, typically HTTP 401 Unauthorized or WebSocket close code

**Error Code**: 40003 (invalid/expired token) via gRPC error mapping

## Subscription Actions

**SubscriptionAction enum**:
- `SUBSCRIPTION_ACTION_UNSPECIFIED` - Undefined (not valid for requests)
- `SUBSCRIPTION_ACTION_SUBSCRIBE` - Subscribe to updates
- `SUBSCRIPTION_ACTION_UNSUBSCRIBE` - Unsubscribe from updates

## Important Implementation Notes

1. **Field naming**: API accepts both `camelCase` and `snake_case` in JSON
2. **Protocol header options**:
   - `Web-Socket-Protocol: json` - Standard JSON field names (camelCase)
   - `Web-Socket-Protocol: json-proto` - Proto-compatible field names (snake_case)
3. **Quotation type**: Prices use `{units: int64, nano: int32}` format
4. **MoneyValue type**: Same as Quotation but includes `currency` field
5. **Instrument identification**: Use `figi` or `instrumentId` (UID preferred for options)
6. **Tracking ID**: Server includes `trackingId` in responses for technical support
7. **Error handling**: Subscription failures return status in confirmation message
8. **Full snapshots**: Order books send full snapshots, not incremental deltas
9. **Real-time updates**: All data is real-time (no delayed data)
10. **Free access**: All WebSocket streams are free for Tinkoff Investments clients

## Testing

- **Production**: Use production token with `wss://invest-public-api.tinkoff.ru/ws/`
- **Sandbox**: WebSocket not available for sandbox - use gRPC streaming instead
- **Recommended tools**: Browser DevTools, wscat, Postman (WebSocket support)

## Rate Limiting on WebSocket

- Dynamic rate limiting applies to WebSocket connections
- Based on overall account trading activity
- Error code 80001: Concurrent stream limit exceeded
- Error code 80002: Request rate exceeded
- Recommended: Implement exponential backoff for reconnections
- See `tiers_and_limits.md` for detailed rate limit information
