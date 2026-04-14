# dYdX v4 Response Formats

## General Response Structure

All Indexer HTTP API responses follow a consistent JSON format:

```json
{
  "data": { /* response data */ },
  "error": null  // or error object if request failed
}
```

For errors:
```json
{
  "data": null,
  "error": {
    "message": "Error description",
    "code": "ERROR_CODE"
  }
}
```

---

## Market Data Responses

### Perpetual Markets Response

**Endpoint**: `GET /v4/perpetualMarkets`

```json
{
  "markets": {
    "BTC-USD": {
      "clobPairId": "0",
      "ticker": "BTC-USD",
      "status": "ACTIVE",
      "baseAsset": "BTC",
      "quoteAsset": "USDC",
      "stepSize": "0.0001",
      "tickSize": "1",
      "indexPrice": "50000.5",
      "oraclePrice": "50000.0",
      "priceChange24H": "1250.75",
      "volume24H": "125000000.50",
      "trades24H": 12543,
      "nextFundingRate": "0.00001",
      "initialMarginFraction": "0.05",
      "maintenanceMarginFraction": "0.03",
      "openInterest": "10000.5",
      "atomicResolution": -10,
      "quantumConversionExponent": -9,
      "subticksPerTick": 100000,
      "stepBaseQuantums": 1000000,
      "marketType": "PERPETUAL"
    },
    "ETH-USD": {
      // Similar structure
    }
  }
}
```

**Field Types**:
- `clobPairId`: string (internal integer as string)
- `ticker`: string
- `status`: enum - "ACTIVE", "PAUSED", "CANCEL_ONLY", "POST_ONLY"
- Prices/sizes: string (decimal representation)
- Integer counts: number
- Fractions: string (decimal, e.g., "0.05" for 5%)
- `atomicResolution`: integer (negative value, e.g., -10)
- `quantumConversionExponent`: integer (negative value, e.g., -9)
- `subticksPerTick`: integer
- `marketType`: enum - "PERPETUAL", "CROSS", "ISOLATED"

### Orderbook Response

**Endpoint**: `GET /v4/orderbooks/perpetualMarket/{market}`

```json
{
  "bids": [
    {
      "price": "50000.0",
      "size": "1.5"
    },
    {
      "price": "49999.0",
      "size": "2.3"
    }
  ],
  "asks": [
    {
      "price": "50001.0",
      "size": "0.8"
    },
    {
      "price": "50002.0",
      "size": "1.2"
    }
  ]
}
```

**Field Types**:
- `price`: string (decimal)
- `size`: string (decimal)

**Notes**:
- Bids are sorted descending by price (highest first)
- Asks are sorted ascending by price (lowest first)

### Trades Response

**Endpoint**: `GET /v4/trades/perpetualMarket/{market}`

```json
{
  "trades": [
    {
      "id": "8f7d6e5c-4b3a-2c1d-0e9f-8a7b6c5d4e3f",
      "side": "BUY",
      "size": "0.5",
      "price": "50000.0",
      "type": "LIMIT",
      "createdAt": "2026-01-20T12:34:56.789Z",
      "createdAtHeight": "12345678"
    }
  ]
}
```

**Field Types**:
- `id`: string (UUID)
- `side`: enum - "BUY", "SELL"
- `size`: string (decimal)
- `price`: string (decimal)
- `type`: enum - "LIMIT", "MARKET", "STOP_LIMIT", "STOP_MARKET", "TRAILING_STOP", "TAKE_PROFIT", "TAKE_PROFIT_MARKET"
- `createdAt`: string (ISO 8601 timestamp)
- `createdAtHeight`: string (block height)

### Candles Response

**Endpoint**: `GET /v4/candles/perpetualMarkets/{market}`

```json
{
  "candles": [
    {
      "startedAt": "2026-01-20T12:00:00.000Z",
      "ticker": "BTC-USD",
      "resolution": "1MIN",
      "low": "49950.0",
      "high": "50100.0",
      "open": "50000.0",
      "close": "50050.0",
      "baseTokenVolume": "125.5",
      "usdVolume": "6277500.0",
      "trades": 543,
      "startingOpenInterest": "10000.0"
    }
  ]
}
```

**Field Types**:
- `startedAt`: string (ISO 8601 timestamp)
- `ticker`: string
- `resolution`: enum - "1MIN", "5MINS", "15MINS", "30MINS", "1HOUR", "4HOURS", "1DAY"
- OHLC prices: string (decimal)
- Volumes: string (decimal)
- `trades`: integer
- `startingOpenInterest`: string (decimal)

### Historical Funding Response

**Endpoint**: `GET /v4/historicalFunding/{market}`

```json
{
  "historicalFunding": [
    {
      "ticker": "BTC-USD",
      "rate": "0.00001",
      "price": "50000.0",
      "effectiveAt": "2026-01-20T08:00:00.000Z",
      "effectiveAtHeight": "12345000"
    }
  ]
}
```

**Field Types**:
- `ticker`: string
- `rate`: string (decimal, can be negative)
- `price`: string (decimal, index price)
- `effectiveAt`: string (ISO 8601 timestamp)
- `effectiveAtHeight`: string (block height)

### Server Time Response

**Endpoint**: `GET /v4/time`

```json
{
  "iso": "2026-01-20T12:34:56.789Z",
  "epoch": 1737378896.789
}
```

**Field Types**:
- `iso`: string (ISO 8601 timestamp)
- `epoch`: number (Unix timestamp with decimals)

### Block Height Response

**Endpoint**: `GET /v4/height`

```json
{
  "height": "12345678",
  "time": "2026-01-20T12:34:56.789Z"
}
```

**Field Types**:
- `height`: string (block height)
- `time`: string (ISO 8601 timestamp)

---

## Account Responses

### Subaccounts Response

**Endpoint**: `GET /v4/addresses/{address}`

```json
{
  "subaccounts": [
    {
      "address": "dydx1abc123...",
      "subaccountNumber": 0,
      "equity": "100000.50",
      "freeCollateral": "75000.25",
      "marginEnabled": true,
      "updatedAtHeight": "12345678"
    }
  ]
}
```

**Field Types**:
- `address`: string (Cosmos address format)
- `subaccountNumber`: integer (0-128000)
- `equity`: string (decimal, total account value)
- `freeCollateral`: string (decimal, available for trading)
- `marginEnabled`: boolean
- `updatedAtHeight`: string (block height)

### Specific Subaccount Response

**Endpoint**: `GET /v4/addresses/{address}/subaccountNumber/{subaccount_number}`

```json
{
  "subaccount": {
    "address": "dydx1abc123...",
    "subaccountNumber": 0,
    "equity": "100000.50",
    "freeCollateral": "75000.25",
    "marginEnabled": true,
    "openPerpetualPositions": {
      "BTC-USD": {
        "market": "BTC-USD",
        "status": "OPEN",
        "side": "LONG",
        "size": "2.5",
        "maxSize": "3.0",
        "entryPrice": "48000.0",
        "exitPrice": null,
        "realizedPnl": "0.0",
        "unrealizedPnl": "5000.0",
        "createdAt": "2026-01-15T10:00:00.000Z",
        "createdAtHeight": "12300000",
        "sumOpen": "2.5",
        "sumClose": "0.0",
        "netFunding": "-50.0"
      }
    },
    "assetPositions": {
      "USDC": {
        "symbol": "USDC",
        "side": "LONG",
        "size": "100000.50",
        "assetId": "0"
      }
    }
  }
}
```

**Field Types**:
- Position fields: same as Positions responses (see below)
- Asset positions: simple balance tracking

### Asset Positions Response

**Endpoint**: `GET /v4/assetPositions`

```json
{
  "positions": [
    {
      "symbol": "USDC",
      "side": "LONG",
      "size": "100000.50",
      "assetId": "0"
    }
  ]
}
```

**Field Types**:
- `symbol`: string (asset symbol, usually "USDC")
- `side`: enum - "LONG" (positive balance), "SHORT" (negative, debt)
- `size`: string (decimal)
- `assetId`: string (internal asset ID)

### Transfer History Response

**Endpoint**: `GET /v4/transfers`

```json
{
  "transfers": [
    {
      "id": "transfer-uuid-123",
      "sender": {
        "address": "dydx1abc123...",
        "subaccountNumber": 0
      },
      "recipient": {
        "address": "dydx1xyz789...",
        "subaccountNumber": 0
      },
      "size": "1000.0",
      "symbol": "USDC",
      "type": "TRANSFER_OUT",
      "createdAt": "2026-01-20T10:00:00.000Z",
      "createdAtHeight": "12345000",
      "transactionHash": "0xabc123..."
    },
    {
      "id": "deposit-uuid-456",
      "sender": {
        "address": "noble1abc...",
        "subaccountNumber": null
      },
      "recipient": {
        "address": "dydx1abc123...",
        "subaccountNumber": 0
      },
      "size": "5000.0",
      "symbol": "USDC",
      "type": "DEPOSIT",
      "createdAt": "2026-01-19T15:30:00.000Z",
      "createdAtHeight": "12340000",
      "transactionHash": "0xdef456..."
    }
  ]
}
```

**Field Types**:
- `id`: string (UUID)
- `sender`/`recipient`: object with `address` and `subaccountNumber`
- `size`: string (decimal)
- `symbol`: string
- `type`: enum - "DEPOSIT", "WITHDRAWAL", "TRANSFER_OUT", "TRANSFER_IN"
- `createdAt`: string (ISO 8601 timestamp)
- `createdAtHeight`: string (block height)
- `transactionHash`: string (blockchain tx hash)

### Trading Rewards Response

**Endpoint**: `GET /v4/historicalBlockTradingRewards/{address}`

```json
{
  "rewards": [
    {
      "tradingReward": "15.50",
      "createdAt": "2026-01-20T12:00:00.000Z",
      "createdAtHeight": "12345678"
    }
  ]
}
```

**Field Types**:
- `tradingReward`: string (decimal, DYDX tokens)
- `createdAt`: string (ISO 8601 timestamp)
- `createdAtHeight`: string (block height)

---

## Positions Responses

### Perpetual Positions Response

**Endpoint**: `GET /v4/perpetualPositions`

```json
{
  "positions": [
    {
      "market": "BTC-USD",
      "status": "OPEN",
      "side": "LONG",
      "size": "2.5",
      "maxSize": "3.0",
      "entryPrice": "48000.0",
      "exitPrice": null,
      "realizedPnl": "0.0",
      "unrealizedPnl": "5000.0",
      "createdAt": "2026-01-15T10:00:00.000Z",
      "createdAtHeight": "12300000",
      "closedAt": null,
      "sumOpen": "2.5",
      "sumClose": "0.0",
      "netFunding": "-50.0"
    },
    {
      "market": "ETH-USD",
      "status": "CLOSED",
      "side": "SHORT",
      "size": "0.0",
      "maxSize": "10.0",
      "entryPrice": "3000.0",
      "exitPrice": "2950.0",
      "realizedPnl": "500.0",
      "unrealizedPnl": "0.0",
      "createdAt": "2026-01-10T08:00:00.000Z",
      "createdAtHeight": "12200000",
      "closedAt": "2026-01-18T16:00:00.000Z",
      "sumOpen": "10.0",
      "sumClose": "10.0",
      "netFunding": "25.0"
    }
  ]
}
```

**Field Types**:
- `market`: string (ticker)
- `status`: enum - "OPEN", "CLOSED", "LIQUIDATED"
- `side`: enum - "LONG", "SHORT"
- `size`: string (decimal, current position size)
- `maxSize`: string (decimal, max size reached)
- `entryPrice`: string (decimal, average entry price)
- `exitPrice`: string or null (decimal, average exit price)
- `realizedPnl`: string (decimal, realized profit/loss)
- `unrealizedPnl`: string (decimal, unrealized profit/loss)
- `createdAt`: string (ISO 8601 timestamp)
- `createdAtHeight`: string (block height)
- `closedAt`: string or null (ISO 8601 timestamp)
- `sumOpen`: string (decimal, total opening volume)
- `sumClose`: string (decimal, total closing volume)
- `netFunding`: string (decimal, net funding payments)

### Historical PnL Response

**Endpoint**: `GET /v4/historical-pnl`

```json
{
  "historicalPnl": [
    {
      "id": "pnl-uuid-123",
      "equity": "105000.50",
      "totalPnl": "5000.50",
      "netTransfers": "0.0",
      "createdAt": "2026-01-20T00:00:00.000Z",
      "blockHeight": "12345000",
      "blockTime": "2026-01-20T00:00:00.000Z"
    }
  ]
}
```

**Field Types**:
- `id`: string (UUID)
- `equity`: string (decimal, total account value)
- `totalPnl`: string (decimal, cumulative P&L)
- `netTransfers`: string (decimal, net deposits - withdrawals)
- `createdAt`: string (ISO 8601 timestamp)
- `blockHeight`: string (block height)
- `blockTime`: string (ISO 8601 timestamp)

### Funding Payments Response

**Endpoint**: `GET /v4/fundingPayments`

```json
{
  "fundingPayments": [
    {
      "market": "BTC-USD",
      "payment": "-12.50",
      "rate": "0.00001",
      "price": "50000.0",
      "positionSize": "2.5",
      "effectiveAt": "2026-01-20T08:00:00.000Z"
    }
  ]
}
```

**Field Types**:
- `market`: string (ticker)
- `payment`: string (decimal, negative = paid, positive = received)
- `rate`: string (decimal, funding rate applied)
- `price`: string (decimal, index price)
- `positionSize`: string (decimal, position size at the time)
- `effectiveAt`: string (ISO 8601 timestamp)

**Note**: Funding payments occur every 8 hours on dYdX v4

---

## Trading Responses

### Orders Response

**Endpoint**: `GET /v4/orders`

```json
{
  "orders": [
    {
      "id": "order-uuid-123",
      "subaccountId": "dydx1abc123.../0",
      "clientId": 12345,
      "clobPairId": "0",
      "side": "BUY",
      "size": "1.0",
      "totalFilled": "0.5",
      "price": "50000.0",
      "type": "LIMIT",
      "status": "OPEN",
      "timeInForce": "GTT",
      "postOnly": false,
      "reduceOnly": false,
      "orderFlags": "0",
      "goodTilBlock": null,
      "goodTilBlockTime": "1737400000",
      "createdAtHeight": "12345000",
      "clientMetadata": "0",
      "triggerPrice": null,
      "updatedAt": "2026-01-20T12:00:00.000Z",
      "updatedAtHeight": "12345100"
    },
    {
      "id": "order-uuid-456",
      "subaccountId": "dydx1abc123.../0",
      "clientId": 12346,
      "clobPairId": "0",
      "side": "SELL",
      "size": "0.5",
      "totalFilled": "0.5",
      "price": "51000.0",
      "type": "LIMIT",
      "status": "FILLED",
      "timeInForce": "IOC",
      "postOnly": false,
      "reduceOnly": true,
      "orderFlags": "0",
      "goodTilBlock": "12345050",
      "goodTilBlockTime": null,
      "createdAtHeight": "12345020",
      "clientMetadata": "0",
      "triggerPrice": null,
      "updatedAt": "2026-01-20T12:05:00.000Z",
      "updatedAtHeight": "12345025"
    }
  ]
}
```

**Field Types**:
- `id`: string (UUID, server-assigned)
- `subaccountId`: string (format: "address/subaccountNumber")
- `clientId`: integer (client-assigned, unique per subaccount)
- `clobPairId`: string (internal market ID)
- `side`: enum - "BUY", "SELL"
- `size`: string (decimal, total order size)
- `totalFilled`: string (decimal, amount filled)
- `price`: string (decimal)
- `type`: enum - "LIMIT", "MARKET", "STOP_LIMIT", "STOP_MARKET", "TRAILING_STOP", "TAKE_PROFIT", "TAKE_PROFIT_MARKET"
- `status`: enum - "OPEN", "FILLED", "CANCELED", "BEST_EFFORT_CANCELED", "UNTRIGGERED"
- `timeInForce`: enum - "GTT", "IOC", "FOK", "POST_ONLY"
- `postOnly`: boolean
- `reduceOnly`: boolean
- `orderFlags`: string - "0" (short-term), "32" (conditional), "64" (long-term)
- `goodTilBlock`: string or null (block height for short-term orders)
- `goodTilBlockTime`: string or null (Unix timestamp for stateful orders)
- `createdAtHeight`: string (block height)
- `clientMetadata`: string (integer as string)
- `triggerPrice`: string or null (for conditional orders)
- `updatedAt`: string (ISO 8601 timestamp)
- `updatedAtHeight`: string (block height)

### Specific Order Response

**Endpoint**: `GET /v4/orders/{orderId}`

Same structure as individual order in Orders Response above.

### Fills Response

**Endpoint**: `GET /v4/fills`

```json
{
  "fills": [
    {
      "id": "fill-uuid-123",
      "side": "BUY",
      "liquidity": "TAKER",
      "type": "LIMIT",
      "market": "BTC-USD",
      "marketType": "PERPETUAL",
      "price": "50000.0",
      "size": "0.5",
      "fee": "5.0",
      "createdAt": "2026-01-20T12:00:00.000Z",
      "createdAtHeight": "12345020",
      "orderId": "order-uuid-456",
      "clientMetadata": "0"
    }
  ]
}
```

**Field Types**:
- `id`: string (UUID)
- `side`: enum - "BUY", "SELL"
- `liquidity`: enum - "TAKER", "MAKER"
- `type`: enum - "LIMIT", "MARKET", etc.
- `market`: string (ticker)
- `marketType`: enum - "PERPETUAL", "CROSS", "ISOLATED"
- `price`: string (decimal, execution price)
- `size`: string (decimal, fill amount)
- `fee`: string (decimal, fee in USDC)
- `createdAt`: string (ISO 8601 timestamp)
- `createdAtHeight`: string (block height)
- `orderId`: string (associated order UUID)
- `clientMetadata`: string (integer as string)

**Fee Structure**:
- Maker fees: Typically negative (rebate)
- Taker fees: Positive fee
- Fees are in USDC

---

## gRPC Responses (Node API)

### Place Order Response

```protobuf
message MsgPlaceOrderResponse {
  string order_id = 1;  // Server-assigned order ID
  bool success = 2;
  string error = 3;  // Error message if !success
}
```

**Note**: gRPC responses use Protobuf binary format, not JSON. After broadcasting a transaction:
1. Response includes transaction hash
2. Query Indexer API to get order details by ID
3. Short-term orders may not appear immediately (gossip delay)

### Cancel Order Response

```protobuf
message MsgCancelOrderResponse {
  bool success = 1;
  string error = 2;
}
```

**Note**: Cancellation confirmation doesn't guarantee the order won't fill:
- Short-term cancellations are best-effort
- Order is only truly unfillable after expiry or after cancellation block for stateful orders

---

## WebSocket Responses

### Subscription Acknowledgement

```json
{
  "type": "subscribed",
  "connection_id": "conn-uuid-123",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "message_id": 0
}
```

### Orderbook Update

```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "message_id": 1,
  "contents": {
    "bids": [
      ["50000.0", "1.5"]
    ],
    "asks": [
      ["50001.0", "0.8"]
    ]
  },
  "version": "1.0"
}
```

### Trades Update

```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_trades",
  "id": "BTC-USD",
  "message_id": 2,
  "contents": {
    "trades": [
      {
        "id": "trade-uuid-123",
        "side": "BUY",
        "size": "0.5",
        "price": "50000.0",
        "createdAt": "2026-01-20T12:34:56.789Z"
      }
    ]
  },
  "version": "1.0"
}
```

### Subaccount Update

```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_subaccounts",
  "id": "dydx1abc123.../0",
  "message_id": 3,
  "contents": {
    "orders": [/* order updates */],
    "fills": [/* new fills */],
    "positions": {/* position updates */}
  },
  "version": "1.0"
}
```

### Markets Update

```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_markets",
  "message_id": 4,
  "contents": {
    "trading": {
      "BTC-USD": {
        "oraclePrice": "50000.5",
        "priceChange24H": "1250.75"
      }
    }
  },
  "version": "1.0"
}
```

---

## Error Responses

### HTTP Error Response

```json
{
  "errors": [
    {
      "msg": "Invalid market ticker",
      "param": "market",
      "location": "params"
    }
  ]
}
```

### Common HTTP Status Codes
- `200` - Success
- `400` - Bad Request (invalid parameters)
- `404` - Not Found (resource doesn't exist)
- `429` - Too Many Requests (rate limited)
- `500` - Internal Server Error
- `503` - Service Unavailable

### gRPC Error Codes
- `OK` (0) - Success
- `INVALID_ARGUMENT` (3) - Invalid parameters
- `NOT_FOUND` (5) - Resource not found
- `ALREADY_EXISTS` (6) - Duplicate order
- `RESOURCE_EXHAUSTED` (8) - Rate limit exceeded
- `UNAVAILABLE` (14) - Service temporarily unavailable

---

## Important Notes

### String vs Number Types
- **All prices, sizes, and financial values are strings** (not numbers)
- Prevents floating-point precision issues
- Must parse to Decimal type in Rust

### Timestamp Formats
- **ISO 8601**: `"2026-01-20T12:34:56.789Z"` (always UTC)
- **Unix epoch**: Number of seconds since 1970-01-01 (can include decimals)
- **Block height**: String representation of integer

### Null vs Missing Fields
- Fields can be `null` when not applicable (e.g., `exitPrice` for open positions)
- Optional fields may be missing entirely from response

### Pagination
- Some endpoints support pagination via `limit` and `page` parameters
- Default limits vary by endpoint (typically 100)
- No explicit pagination tokens - use incremental page numbers

### Data Freshness
- Indexer API may lag behind blockchain by 0-2 seconds
- WebSocket updates are typically faster than REST API
- Use block height to verify data recency
