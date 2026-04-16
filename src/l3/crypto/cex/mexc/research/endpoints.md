# MEXC API Endpoints

## Base URLs

### Spot Trading
- **REST API**: `https://api.mexc.com`
- **WebSocket**: `wss://wbs.mexc.com/ws`

### Futures Trading
- **REST API**: `https://contract.mexc.com`
- **WebSocket**: `wss://contract.mexc.com/edge`

**Note**: Futures API trading is currently available to institutional users only. Contact institution@mexc.com for institutional access.

---

## MarketData Trait Endpoints

All market data endpoints are public and do not require authentication.

### Test Connectivity
```
GET /api/v3/ping
```
**Weight**: 1
**Parameters**: None
**Response**: `{}`

### Server Time
```
GET /api/v3/time
```
**Weight**: 1
**Parameters**: None
**Response**:
```json
{
  "serverTime": 1640080800000
}
```

### Exchange Information
```
GET /api/v3/exchangeInfo
```
**Weight**: 10
**Parameters**:
- `symbol` (optional): Query specific symbol
- `symbols` (optional): Array of symbols `["BTCUSDT","ETHUSDT"]`

**Response**:
```json
{
  "timezone": "UTC",
  "serverTime": 1640080800000,
  "rateLimits": [...],
  "exchangeFilters": [],
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "status": "ENABLED",
      "baseAsset": "BTC",
      "baseAssetPrecision": 8,
      "quoteAsset": "USDT",
      "quotePrecision": 8,
      "quoteAssetPrecision": 8,
      "orderTypes": ["LIMIT", "MARKET", "LIMIT_MAKER"],
      "permissions": ["SPOT"],
      "filters": [
        {
          "filterType": "PERCENT_PRICE_BY_SIDE",
          "bidMultiplierUp": "5",
          "bidMultiplierDown": "0.2",
          "askMultiplierUp": "5",
          "askMultiplierDown": "0.2"
        }
      ],
      "maxQuoteAmount": "5000000",
      "makerCommission": "0.002",
      "takerCommission": "0.002"
    }
  ]
}
```

### Default Symbols
```
GET /api/v3/defaultSymbols
```
**Weight**: 1
**Description**: Retrieve list of default trading pairs
**Response**: Array of default symbol names

### Order Book Depth
```
GET /api/v3/depth
```
**Weight**: Adjusted based on limit:
- Limit 1-100: 1
- Limit 101-500: 5
- Limit 501-1000: 10
- Limit 1001-5000: 50

**Parameters**:
- `symbol` (required): Trading pair (e.g., "BTCUSDT")
- `limit` (optional): Default 100, max 5000

**Response**:
```json
{
  "lastUpdateId": 123456789,
  "bids": [
    ["93220.00", "0.5"],
    ["93210.00", "1.2"]
  ],
  "asks": [
    ["93230.00", "0.8"],
    ["93240.00", "2.1"]
  ]
}
```

### Recent Trades
```
GET /api/v3/trades
```
**Weight**: 1
**Parameters**:
- `symbol` (required): Trading pair
- `limit` (optional): Default 500, max 1000

**Response**:
```json
[
  {
    "id": 28457,
    "price": "93220.00",
    "qty": "0.04438243",
    "quoteQty": "4137.85",
    "time": 1736409765051,
    "isBuyerMaker": false,
    "isBestMatch": true
  }
]
```

### Aggregate Trades
```
GET /api/v3/aggTrades
```
**Weight**: 1
**Parameters**:
- `symbol` (required): Trading pair
- `fromId` (optional): Trade ID to fetch from
- `startTime` (optional): Timestamp in ms
- `endTime` (optional): Timestamp in ms
- `limit` (optional): Default 500, max 1000

**Response**:
```json
[
  {
    "a": 26129,
    "p": "93220.00",
    "q": "1.5",
    "f": 100,
    "l": 105,
    "T": 1498793709153,
    "m": true,
    "M": true
  }
]
```

### Klines/Candlesticks
```
GET /api/v3/klines
```
**Weight**: 1
**Parameters**:
- `symbol` (required): Trading pair
- `interval` (required): Kline interval
  - `1m`, `5m`, `15m`, `30m`, `60m` (minutes)
  - `4h`, `8h` (hours)
  - `1d` (day)
  - `1w` (week)
  - `1M` (month)
- `startTime` (optional): Timestamp in ms
- `endTime` (optional): Timestamp in ms
- `limit` (optional): Default 500, max 1000

**Response**:
```json
[
  [
    1640804880000,     // Open time
    "47482.36",        // Open
    "47482.36",        // High
    "47416.57",        // Low
    "47436.1",         // Close
    "3.550717",        // Volume
    1640804940000,     // Close time
    "168387.3"         // Quote asset volume
  ]
]
```

### Current Average Price
```
GET /api/v3/avgPrice
```
**Weight**: 1
**Parameters**:
- `symbol` (required): Trading pair

**Response**:
```json
{
  "mins": 5,
  "price": "93150.25"
}
```

### 24hr Ticker Price Change Statistics
```
GET /api/v3/ticker/24hr
```
**Weight**:
- 1 for single symbol
- 40 when symbol parameter is omitted

**Parameters**:
- `symbol` (optional): Trading pair. If omitted, returns all symbols

**Response** (single symbol):
```json
{
  "symbol": "BTCUSDT",
  "priceChange": "1200.50",
  "priceChangePercent": "1.3",
  "weightedAvgPrice": "93000.00",
  "prevClosePrice": "92000.00",
  "lastPrice": "93200.50",
  "lastQty": "0.5",
  "bidPrice": "93200.00",
  "bidQty": "1.2",
  "askPrice": "93210.00",
  "askQty": "0.8",
  "openPrice": "92000.00",
  "highPrice": "93500.00",
  "lowPrice": "91800.00",
  "volume": "12345.67",
  "quoteVolume": "1147893256.23",
  "openTime": 1640080800000,
  "closeTime": 1640167200000,
  "firstId": 100,
  "lastId": 200,
  "count": 100
}
```

### Symbol Price Ticker
```
GET /api/v3/ticker/price
```
**Weight**:
- 1 for single symbol
- 2 when symbol parameter is omitted

**Parameters**:
- `symbol` (optional): Trading pair

**Response** (single):
```json
{
  "symbol": "BTCUSDT",
  "price": "93200.50"
}
```

### Symbol Order Book Ticker
```
GET /api/v3/ticker/bookTicker
```
**Weight**:
- 1 for single symbol
- 2 when symbol parameter is omitted

**Parameters**:
- `symbol` (optional): Trading pair

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "bidPrice": "93200.00",
  "bidQty": "1.5",
  "askPrice": "93210.00",
  "askQty": "2.3"
}
```

---

## Trading Trait Endpoints

All trading endpoints require authentication.

### New Order (TRADE)
```
POST /api/v3/order
```
**Weight**: 1
**Permission**: SPOT_DEAL_WRITE

**Parameters**:
- `symbol` (required): Trading pair
- `side` (required): `BUY` or `SELL`
- `type` (required): Order type
  - `LIMIT`: Limit order
  - `MARKET`: Market order
  - `LIMIT_MAKER`: Post-only limit order
- `quantity` (required for most types): Order quantity
- `quoteOrderQty` (optional): Quote asset quantity for MARKET orders
- `price` (required for LIMIT orders): Order price
- `newClientOrderId` (optional): Unique client order ID
- `recvWindow` (optional): Max 60000, default 5000
- `timestamp` (required): Current timestamp in milliseconds
- `signature` (required): HMAC SHA256 signature

**Response**:
```json
{
  "symbol": "MXUSDT",
  "orderId": "06a480e69e604477bfb48dddd5f0b750",
  "orderListId": -1,
  "price": "0.1",
  "origQty": "50",
  "type": "LIMIT",
  "side": "BUY",
  "transactTime": 1666676533741
}
```

### Test New Order
```
POST /api/v3/order/test
```
**Weight**: 1
**Permission**: SPOT_DEAL_WRITE
**Description**: Test order placement without executing. Same parameters as POST /api/v3/order

**Response**: `{}`

### Batch Orders
```
POST /api/v3/batchOrders
```
**Weight**: 5
**Permission**: SPOT_DEAL_WRITE

**Parameters**:
- `batchOrders` (required): JSON array of order objects (max 20 orders with same symbol)
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Example**:
```json
{
  "batchOrders": [
    {
      "symbol": "BTCUSDT",
      "side": "BUY",
      "type": "LIMIT",
      "quantity": "0.1",
      "price": "90000"
    },
    {
      "symbol": "BTCUSDT",
      "side": "SELL",
      "type": "LIMIT",
      "quantity": "0.1",
      "price": "95000"
    }
  ]
}
```

### Cancel Order
```
DELETE /api/v3/order
```
**Weight**: 1
**Permission**: SPOT_DEAL_WRITE

**Parameters**:
- `symbol` (required): Trading pair
- `orderId` (optional): Either orderId or origClientOrderId required
- `origClientOrderId` (optional): Either orderId or origClientOrderId required
- `newClientOrderId` (optional): New unique identifier
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "origClientOrderId": "myOrder1",
  "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
  "orderListId": -1,
  "clientOrderId": "cancelMyOrder1",
  "price": "90000",
  "origQty": "0.1",
  "executedQty": "0",
  "cummulativeQuoteQty": "0",
  "status": "CANCELED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY"
}
```

### Cancel All Open Orders
```
DELETE /api/v3/openOrders
```
**Weight**: 1
**Permission**: SPOT_DEAL_WRITE

**Parameters**:
- `symbol` (required): Trading pair
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**: Array of cancelled orders

### Query Order
```
GET /api/v3/order
```
**Weight**: 2
**Permission**: SPOT_DEAL_READ

**Parameters**:
- `symbol` (required): Trading pair
- `orderId` (optional): Either orderId or origClientOrderId required
- `origClientOrderId` (optional): Either orderId or origClientOrderId required
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**:
```json
{
  "symbol": "BTCUSDT",
  "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
  "orderListId": -1,
  "clientOrderId": "myOrder1",
  "price": "90000",
  "origQty": "0.1",
  "executedQty": "0.05",
  "cummulativeQuoteQty": "4500",
  "status": "PARTIALLY_FILLED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY",
  "stopPrice": "0",
  "icebergQty": "0",
  "time": 1640080800000,
  "updateTime": 1640084400000,
  "isWorking": true,
  "origQuoteOrderQty": "0"
}
```

### Current Open Orders
```
GET /api/v3/openOrders
```
**Weight**:
- 3 with symbol
- 40 without symbol

**Permission**: SPOT_DEAL_READ

**Parameters**:
- `symbol` (optional): Trading pair. Careful when accessing without symbol
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**: Array of open orders

### All Orders
```
GET /api/v3/allOrders
```
**Weight**: 10
**Permission**: SPOT_DEAL_READ

**Parameters**:
- `symbol` (required): Trading pair
- `orderId` (optional): Order ID to start from
- `startTime` (optional): Timestamp in ms
- `endTime` (optional): Timestamp in ms
- `limit` (optional): Default 500, max 1000
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Description**: Get all orders (active, cancelled, completed). Query period is latest 24 hours by default. Maximum query range is 7 days.

**Response**: Array of orders

---

## Account Trait Endpoints

All account endpoints require authentication.

### Account Information
```
GET /api/v3/account
```
**Weight**: 10
**Permission**: SPOT_ACCOUNT_READ

**Parameters**:
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**:
```json
{
  "makerCommission": 20,
  "takerCommission": 20,
  "buyerCommission": 0,
  "sellerCommission": 0,
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "updateTime": 1640080800000,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "0.5",
      "locked": "0.1"
    },
    {
      "asset": "USDT",
      "free": "10000.0",
      "locked": "500.0"
    }
  ],
  "permissions": ["SPOT"]
}
```

### Account Trade List
```
GET /api/v3/myTrades
```
**Weight**: 10
**Permission**: SPOT_DEAL_READ

**Parameters**:
- `symbol` (required): Trading pair
- `orderId` (optional): Return trades for this order only
- `startTime` (optional): Timestamp in ms
- `endTime` (optional): Timestamp in ms
- `fromId` (optional): Trade ID to fetch from
- `limit` (optional): Default 500, max 1000
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Description**: Get trades for specific account and symbol. Maximum query period is 30 days.

**Response**:
```json
[
  {
    "symbol": "BTCUSDT",
    "id": 28457,
    "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
    "orderListId": -1,
    "price": "93220.00",
    "qty": "0.04438243",
    "quoteQty": "4137.85",
    "commission": "0.0000443824",
    "commissionAsset": "BTC",
    "time": 1736409765051,
    "isBuyer": true,
    "isMaker": false,
    "isBestMatch": true
  }
]
```

### Trade Fee
```
GET /api/v3/tradeFee
```
**Weight**: 1
**Permission**: SPOT_ACCOUNT_READ

**Parameters**:
- `symbol` (optional): Trading pair
- `recvWindow` (optional)
- `timestamp` (required)
- `signature` (required)

**Response**:
```json
[
  {
    "symbol": "BTCUSDT",
    "makerCommission": "0.002",
    "takerCommission": "0.002"
  }
]
```

---

## Positions Trait Endpoints (Futures Only)

**Note**: Futures API trading is currently restricted to institutional users. Query endpoints are available but trading functionality may be limited for retail users.

### Get Position Information
```
GET /api/v1/private/position/list/query_all
```
**Permission**: Account and Trading

**Response**:
```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "positionId": 123456,
      "symbol": "BTC_USDT",
      "positionType": 1,
      "holdVol": "1.5",
      "holdAvgPrice": "92000.00",
      "leverage": 10,
      "liquidatePrice": "83000.00",
      "oim": "13800.00",
      "im": "13800.00",
      "holdFee": "5.2",
      "autoAddIm": false
    }
  ]
}
```

### Get Account Balance (Futures)
```
GET /api/v1/private/account/asset
```
**Permission**: Account and Trading

**Response**:
```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "currency": "USDT",
      "positionMargin": 13800.00,
      "availableBalance": 32809.85,
      "cashBalance": 32809.85,
      "frozenBalance": 0,
      "equity": 32809.85,
      "unrealized": 0,
      "bonus": 0
    }
  ]
}
```

### Modify Leverage
```
POST /api/v1/private/position/change_leverage
```
**Permission**: Account and Trading

**Parameters**:
- `symbol` (required): Contract symbol (e.g., "BTC_USDT")
- `leverage` (required): New leverage value
- Other authentication parameters

### Change Margin
```
POST /api/v1/private/position/change_margin
```
**Permission**: Account and Trading

**Parameters**:
- `positionId` (required): Position ID
- `amount` (required): Margin amount to add/remove
- `type` (required): Operation type (e.g., "ADD")

---

## Wallet Endpoints

All wallet endpoints require authentication.

### Get All Coins' Information
```
GET /api/v3/capital/config/getall
```
**Weight**: 10
**Permission**: SPOT_ACCOUNT_READ

**Description**: Get currency details including networks, fees, deposit/withdrawal status

**Response**:
```json
[
  {
    "coin": "BTC",
    "name": "Bitcoin",
    "networkList": [
      {
        "network": "BTC",
        "coin": "BTC",
        "withdrawIntegerMultiple": "0.00000001",
        "isDefault": true,
        "depositEnable": true,
        "withdrawEnable": true,
        "withdrawFee": "0.0005",
        "withdrawMin": "0.001",
        "withdrawMax": "9000"
      }
    ]
  }
]
```

### Withdraw
```
POST /api/v3/capital/withdraw
```
**Weight**: 1
**Permission**: SPOT_ACCOUNT_WRITE

**Parameters**:
- `coin` (required): Coin name
- `network` (required): Network name
- `address` (required): Withdrawal address
- `amount` (required): Withdrawal amount
- `withdrawOrderId` (optional): Client custom ID
- `remark` (optional): Withdrawal note
- Other auth parameters

### Deposit History
```
GET /api/v3/capital/deposit/hisrec
```
**Weight**: 1
**Permission**: SPOT_ACCOUNT_READ

**Parameters**:
- `coin` (optional): Coin name
- `status` (optional): Deposit status (0: pending, 1: success, 2: failed)
- `startTime` (optional): Default 7 days ago
- `endTime` (optional): Default now
- `limit` (optional): Default 500, max 1000

**Description**: Query deposit history (7-90 days)

### Withdrawal History
```
GET /api/v3/capital/withdraw/history
```
**Weight**: 1
**Permission**: SPOT_ACCOUNT_READ

**Parameters**: Similar to deposit history

**Description**: Query withdrawal history (7-90 days)

### Deposit Address
```
GET /api/v3/capital/deposit/address
```
**Weight**: 10
**Permission**: SPOT_ACCOUNT_READ

**Parameters**:
- `coin` (required): Coin name
- `network` (optional): Network name

**Description**: Retrieve active deposit addresses

### Transfer Between Accounts
```
POST /api/v3/capital/transfer
```
**Weight**: 1
**Permission**: SPOT_ACCOUNT_WRITE

**Parameters**:
- `fromAccount` (required): Source account type (e.g., "SPOT", "FUTURES")
- `toAccount` (required): Destination account type
- `asset` (required): Asset to transfer
- `amount` (required): Transfer amount

**Description**: Transfer between spot/futures accounts

---

## Symbol Format

### Spot Trading
Format: Concatenated without separator
- Example: `BTCUSDT`, `ETHUSDT`, `MXUSDT`

### Futures Trading
Format: Underscore separator
- Example: `BTC_USDT`, `ETH_USDT`, `DOGE_USDT`

---

## Notes

1. **Rate Limits**: Each endpoint has IP or UID-based limits. See rate_limits.md for details.
2. **Authentication**: All SIGNED endpoints require X-MEXC-APIKEY header and HMAC SHA256 signature.
3. **Timestamps**: All timestamps are in milliseconds.
4. **Weight**: Each endpoint has a weight that counts toward rate limits.
5. **Futures Restriction**: Futures trading API is currently limited to institutional users as of 2026.
