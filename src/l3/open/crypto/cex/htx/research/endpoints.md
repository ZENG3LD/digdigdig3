# HTX API Endpoints

Complete endpoint documentation for HTX (formerly Huobi) exchange V5 connector implementation.

## Base URLs

**REST API:**
- Primary: `https://api.huobi.pro`
- AWS Optimized: `https://api-aws.huobi.pro`

**WebSocket:**
- Market Data: `wss://api.huobi.pro/ws`
- MBP Incremental: `wss://api.huobi.pro/feed`
- Account/Orders: `wss://api.huobi.pro/ws/v2`

## MarketData Trait Endpoints

### Get Server Time
```
GET /v1/common/timestamp
```
**Authentication:** None
**Response:**
```json
{
  "status": "ok",
  "data": 1234567890000
}
```

### Get Symbols / Trading Pairs
```
GET /v2/settings/common/symbols
```
**Authentication:** None
**Parameters:**
- `symbols` (optional): Comma-separated symbol list
- `ts` (optional): Response generation time

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "symbol": "btcusdt",
      "state": "online",
      "bc": "btc",
      "qc": "usdt",
      "pp": 2,
      "ap": 6,
      "sp": "main",
      "vp": 8,
      "minov": "5",
      "maxov": "200000",
      "lominoa": "0.0001",
      "lomaxoa": "1000",
      "lomaxba": "10000000",
      "lomaxsa": "1000",
      "smminoa": "0.0001",
      "blmlt": "3",
      "slmlt": "3",
      "smmaxoa": "100",
      "bmmaxov": "1000000",
      "msormlt": "0.01",
      "mbormlt": "50",
      "maxov": "200000",
      "u": "btcusdt",
      "mfr": "0.001",
      "ct": "0.01",
      "tags": "etp,holdinglimit"
    }
  ]
}
```

**Field Descriptions:**
- `symbol`: Trading symbol
- `state`: Trading status (online/offline/suspend)
- `bc`: Base currency
- `qc`: Quote currency
- `pp`: Price precision
- `ap`: Amount precision
- `vp`: Value precision
- `minov`: Min order value
- `maxov`: Max order value
- `lominoa`: Limit order min amount
- `lomaxoa`: Limit order max amount
- `lomaxba`: Limit order max buy amount
- `lomaxsa`: Limit order max sell amount
- `smminoa`: Sell-market min order amount
- `blmlt`: Buy limit order max leverage
- `slmlt`: Sell limit order max leverage
- `smmaxoa`: Sell-market max order amount
- `bmmaxov`: Buy-market max order value
- `msormlt`: Min sell-market order rate limit
- `mbormlt`: Min buy-market order rate limit
- `mfr`: Maker fee rate
- `ct`: Control type
- `tags`: Symbol tags

### Get Ticker (24hr Stats)
```
GET /market/detail/merged
```
**Authentication:** None
**Parameters:**
- `symbol` (required): Trading symbol (e.g., btcusdt)

**Response:**
```json
{
  "status": "ok",
  "ch": "market.btcusdt.detail.merged",
  "ts": 1234567890,
  "tick": {
    "id": 100001234567,
    "amount": 12224.2922,
    "count": 15195,
    "open": 9790.52,
    "close": 10195.00,
    "low": 9657.00,
    "high": 10300.00,
    "vol": 121906001.7922,
    "bid": [10195.00, 1.3],
    "ask": [10195.01, 2.5]
  }
}
```

### Get All Tickers
```
GET /market/tickers
```
**Authentication:** None
**Response:**
```json
{
  "status": "ok",
  "ts": 1234567890,
  "data": [
    {
      "symbol": "btcusdt",
      "open": 9790.52,
      "high": 10300.00,
      "low": 9657.00,
      "close": 10195.00,
      "amount": 12224.2922,
      "vol": 121906001.7922,
      "count": 15195,
      "bid": 10195.00,
      "bidSize": 1.3,
      "ask": 10195.01,
      "askSize": 2.5
    }
  ]
}
```

### Get Order Book
```
GET /market/depth
```
**Authentication:** None
**Parameters:**
- `symbol` (required): Trading symbol
- `depth` (optional): Levels - 5, 10, 20 (default: 20)
- `type` (required): step0-step5 (aggregation level)

**Response:**
```json
{
  "status": "ok",
  "ch": "market.btcusdt.depth.step0",
  "ts": 1234567890,
  "tick": {
    "version": 100001234567,
    "ts": 1234567890,
    "bids": [
      [10195.00, 1.3],
      [10194.99, 2.5]
    ],
    "asks": [
      [10195.01, 2.5],
      [10195.02, 1.8]
    ]
  }
}
```

### Get Recent Trades
```
GET /market/trade
```
**Authentication:** None
**Parameters:**
- `symbol` (required): Trading symbol

**Response:**
```json
{
  "status": "ok",
  "ch": "market.btcusdt.trade.detail",
  "ts": 1234567890,
  "tick": {
    "id": 100001234567,
    "ts": 1234567890,
    "data": [
      {
        "id": 100001234567,
        "ts": 1234567890,
        "trade-id": 100001234567,
        "amount": 0.1,
        "price": 10195.00,
        "direction": "buy"
      }
    ]
  }
}
```

### Get Trade History
```
GET /market/history/trade
```
**Authentication:** None
**Parameters:**
- `symbol` (required): Trading symbol
- `size` (optional): Number of records (1-2000, default: 1)

**Response:**
```json
{
  "status": "ok",
  "ch": "market.btcusdt.trade.detail",
  "ts": 1234567890,
  "data": [
    {
      "id": 100001234567,
      "ts": 1234567890,
      "data": [
        {
          "id": 100001234567,
          "ts": 1234567890,
          "trade-id": 100001234567,
          "amount": 0.1,
          "price": 10195.00,
          "direction": "buy"
        }
      ]
    }
  ]
}
```

### Get Klines/Candlesticks
```
GET /market/history/kline
```
**Authentication:** None
**Parameters:**
- `symbol` (required): Trading symbol
- `period` (required): 1min, 5min, 15min, 30min, 60min, 4hour, 1day, 1mon, 1week, 1year
- `size` (optional): Number of records (1-2000, default: 150)

**Response:**
```json
{
  "status": "ok",
  "ch": "market.btcusdt.kline.1day",
  "ts": 1234567890,
  "data": [
    {
      "id": 1234567800,
      "amount": 12224.2922,
      "count": 15195,
      "open": 9790.52,
      "close": 10195.00,
      "low": 9657.00,
      "high": 10300.00,
      "vol": 121906001.7922
    }
  ]
}
```

## Account Trait Endpoints

### Get Account List
```
GET /v1/account/accounts
```
**Authentication:** Required
**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 123456,
      "type": "spot",
      "subtype": "",
      "state": "working"
    }
  ]
}
```

**Account Types:**
- `spot`: Spot account
- `margin`: Cross-margin account
- `otc`: OTC account
- `point`: Point card account
- `super-margin`: Isolated margin account
- `investment`: Wealth management account
- `borrow`: Borrow account
- `grid-trading`: Grid trading account

### Get Account Balance
```
GET /v1/account/accounts/{account-id}/balance
```
**Authentication:** Required
**Path Parameters:**
- `account-id` (required): Account ID from accounts list

**Response:**
```json
{
  "status": "ok",
  "data": {
    "id": 123456,
    "type": "spot",
    "state": "working",
    "list": [
      {
        "currency": "usdt",
        "type": "trade",
        "balance": "10000.123"
      },
      {
        "currency": "usdt",
        "type": "frozen",
        "balance": "0.00"
      }
    ]
  }
}
```

**Balance Types:**
- `trade`: Available balance
- `frozen`: Frozen balance

### Get Asset Valuation
```
GET /v2/account/asset-valuation
```
**Authentication:** Required
**Parameters:**
- `accountType` (required): spot, margin, otc, super-margin, etc.
- `valuationCurrency` (optional): BTC, USD, CNY (default: BTC)
- `subUid` (optional): Sub-user ID

**Response:**
```json
{
  "code": 200,
  "data": {
    "balance": "10000.50",
    "timestamp": 1234567890
  }
}
```

### Get Account Ledger
```
GET /v2/account/ledger
```
**Authentication:** Required
**Parameters:**
- `accountId` (required): Account ID
- `currency` (optional): Currency filter
- `transactTypes` (optional): Transaction types (comma-separated)
- `startTime` (optional): Start timestamp (ms)
- `endTime` (optional): End timestamp (ms)
- `sort` (optional): asc, desc
- `limit` (optional): Records per page (1-500, default: 100)
- `fromId` (optional): First record ID for next page

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "accountId": 123456,
      "currency": "usdt",
      "transactAmt": "100.00",
      "transactType": "trade",
      "transferType": "spot-to-margin",
      "transactId": 100001234567,
      "transactTime": 1234567890,
      "transferer": 789012,
      "transferee": 123456
    }
  ],
  "nextId": 100001234568
}
```

### Asset Transfer
```
POST /v1/account/transfer
```
**Authentication:** Required
**Parameters (JSON body):**
- `from-user` (required): User ID
- `from-account-type` (required): spot, margin, otc, etc.
- `from-account` (required): Account ID
- `to-user` (required): User ID
- `to-account-type` (required): spot, margin, otc, etc.
- `to-account` (required): Account ID
- `currency` (required): Currency code
- `amount` (required): Transfer amount

**Response:**
```json
{
  "status": "ok",
  "data": 100001234567
}
```

## Trading Trait Endpoints

### Place Order
```
POST /v1/order/orders/place
```
**Authentication:** Required
**Parameters (JSON body):**
- `account-id` (required): Account ID
- `symbol` (required): Trading symbol
- `type` (required): Order type
- `amount` (required): Order amount
- `price` (optional): Order price (required for limit orders)
- `source` (optional): Order source (default: api)
- `client-order-id` (optional): Client order ID (max 64 chars)
- `stop-price` (optional): Stop trigger price (for stop orders)
- `operator` (optional): gte, lte (for stop orders)
- `self-match-prevent` (optional): 0 (disabled), 1 (enabled)

**Order Types:**
- `buy-market`: Market buy
- `sell-market`: Market sell
- `buy-limit`: Limit buy
- `sell-limit`: Limit sell
- `buy-ioc`: IOC buy
- `sell-ioc`: IOC sell
- `buy-limit-maker`: Maker-only buy
- `sell-limit-maker`: Maker-only sell
- `buy-stop-limit`: Stop-limit buy
- `sell-stop-limit`: Stop-limit sell
- `buy-limit-fok`: FOK buy
- `sell-limit-fok`: FOK sell

**Response:**
```json
{
  "status": "ok",
  "data": "100001234567"
}
```

### Place Batch Orders
```
POST /v1/order/batch-orders
```
**Authentication:** Required
**Parameters (JSON body):**
Array of order objects with same fields as single order.

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "order-id": 100001234567,
      "client-order-id": "my-order-1"
    }
  ]
}
```

### Cancel Order
```
POST /v1/order/orders/{order-id}/submitcancel
```
**Authentication:** Required
**Path Parameters:**
- `order-id` (required): Order ID

**Response:**
```json
{
  "status": "ok",
  "data": "100001234567"
}
```

### Cancel Order by Client ID
```
POST /v1/order/orders/submitCancelClientOrder
```
**Authentication:** Required
**Parameters (JSON body):**
- `client-order-id` (required): Client order ID

**Response:**
```json
{
  "status": "ok",
  "data": -1
}
```

### Batch Cancel Orders
```
POST /v1/order/orders/batchcancel
```
**Authentication:** Required
**Parameters (JSON body):**
- `order-ids` (optional): Array of order IDs (max 50)
- `client-order-ids` (optional): Array of client order IDs (max 50)

**Response:**
```json
{
  "status": "ok",
  "data": {
    "success": ["100001234567"],
    "failed": [
      {
        "order-id": "100001234568",
        "err-code": "order-orderstate-error",
        "err-msg": "Invalid order state"
      }
    ]
  }
}
```

### Cancel All Orders
```
POST /v1/order/orders/batchCancelOpenOrders
```
**Authentication:** Required
**Parameters (JSON body):**
- `account-id` (required): Account ID
- `symbol` (optional): Trading symbol
- `side` (optional): buy, sell
- `size` (optional): Number of orders to cancel (max 100)

**Response:**
```json
{
  "status": "ok",
  "data": {
    "success-count": 10,
    "failed-count": 0,
    "next-id": 100001234567
  }
}
```

### Get Order Details
```
GET /v1/order/orders/{order-id}
```
**Authentication:** Required
**Path Parameters:**
- `order-id` (required): Order ID

**Response:**
```json
{
  "status": "ok",
  "data": {
    "id": 100001234567,
    "symbol": "btcusdt",
    "account-id": 123456,
    "client-order-id": "my-order-1",
    "amount": "0.1",
    "price": "10000.00",
    "created-at": 1234567890,
    "type": "buy-limit",
    "field-amount": "0.05",
    "field-cash-amount": "500.00",
    "field-fees": "0.0001",
    "finished-at": 1234567900,
    "source": "api",
    "state": "filled",
    "canceled-at": 0
  }
}
```

**Order States:**
- `submitted`: Submitted
- `partial-filled`: Partially filled
- `partial-canceled`: Partially canceled
- `filled`: Fully filled
- `canceled`: Canceled

### Get Order by Client ID
```
GET /v1/order/orders/getClientOrder
```
**Authentication:** Required
**Parameters:**
- `clientOrderId` (required): Client order ID

**Response:**
```json
{
  "status": "ok",
  "data": {
    "id": 100001234567,
    "symbol": "btcusdt",
    "account-id": 123456,
    "client-order-id": "my-order-1",
    "amount": "0.1",
    "price": "10000.00",
    "created-at": 1234567890,
    "type": "buy-limit",
    "field-amount": "0.05",
    "field-cash-amount": "500.00",
    "field-fees": "0.0001",
    "state": "filled"
  }
}
```

### Get Open Orders
```
GET /v1/order/openOrders
```
**Authentication:** Required
**Parameters:**
- `account-id` (required): Account ID
- `symbol` (optional): Trading symbol
- `side` (optional): buy, sell
- `from` (optional): Start order ID
- `direct` (optional): prev, next
- `size` (optional): Number of records (max 500)

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "symbol": "btcusdt",
      "account-id": 123456,
      "client-order-id": "my-order-1",
      "amount": "0.1",
      "price": "10000.00",
      "created-at": 1234567890,
      "type": "buy-limit",
      "filled-amount": "0.05",
      "filled-cash-amount": "500.00",
      "filled-fees": "0.0001",
      "source": "api",
      "state": "partial-filled"
    }
  ]
}
```

### Query Orders
```
GET /v1/order/orders
```
**Authentication:** Required
**Parameters:**
- `symbol` (required): Trading symbol
- `types` (optional): Order types (comma-separated)
- `start-time` (optional): Start time filter (ms, -48h to -1ms from current)
- `end-time` (optional): End time filter (ms, -48h to -1ms from current)
- `states` (optional): Order states (comma-separated)
- `from` (optional): Start order ID
- `direct` (optional): prev, next
- `size` (optional): Number of records (max 100)

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "symbol": "btcusdt",
      "account-id": 123456,
      "client-order-id": "my-order-1",
      "amount": "0.1",
      "price": "10000.00",
      "created-at": 1234567890,
      "type": "buy-limit",
      "filled-amount": "0.1",
      "filled-cash-amount": "1000.00",
      "filled-fees": "0.0002",
      "source": "api",
      "state": "filled",
      "canceled-at": 0
    }
  ]
}
```

### Get Order Match Results
```
GET /v1/order/orders/{order-id}/matchresults
```
**Authentication:** Required
**Path Parameters:**
- `order-id` (required): Order ID

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "order-id": 100001234567,
      "match-id": 100001234567,
      "symbol": "btcusdt",
      "type": "buy-limit",
      "source": "api",
      "price": "10000.00",
      "filled-amount": "0.1",
      "filled-fees": "0.0002",
      "created-at": 1234567890
    }
  ]
}
```

### Query Match Results
```
GET /v1/order/matchresults
```
**Authentication:** Required
**Parameters:**
- `symbol` (required): Trading symbol
- `types` (optional): Order types (comma-separated)
- `start-time` (optional): Start time (ms, -48h to -1ms)
- `end-time` (optional): End time (ms, -48h to -1ms)
- `from` (optional): Start trade ID
- `direct` (optional): prev, next
- `size` (optional): Number of records (max 500)

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "order-id": 100001234567,
      "match-id": 100001234567,
      "symbol": "btcusdt",
      "type": "buy-limit",
      "source": "api",
      "price": "10000.00",
      "filled-amount": "0.1",
      "filled-fees": "0.0002",
      "fee-currency": "btc",
      "created-at": 1234567890
    }
  ]
}
```

### Get Trading Fees
```
GET /v2/reference/transact-fee-rate
```
**Authentication:** Required
**Parameters:**
- `symbols` (required): Comma-separated symbols

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "symbol": "btcusdt",
      "actualMakerRate": "0.002",
      "actualTakerRate": "0.002",
      "takerFeeRate": "0.002",
      "makerFeeRate": "0.002"
    }
  ]
}
```

## Positions Trait Endpoints

HTX spot trading does not have dedicated positions endpoints. Position data is managed through:

1. Account balances (`GET /v1/account/accounts/{account-id}/balance`)
2. Open orders (`GET /v1/order/openOrders`)

For futures/margin trading positions, use separate futures/margin endpoints:

### Isolated Margin Position
```
GET /v1/margin/accounts/balance
```
**Authentication:** Required
**Parameters:**
- `symbol` (optional): Trading symbol
- `sub-uid` (optional): Sub-user ID

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 123456,
      "type": "margin",
      "state": "working",
      "symbol": "btcusdt",
      "fl-price": "9500.00",
      "fl-type": "safe",
      "risk-rate": "1.5",
      "list": [
        {
          "currency": "btc",
          "type": "trade",
          "balance": "0.1"
        }
      ]
    }
  ]
}
```

### Cross Margin Position
```
GET /v1/cross-margin/accounts/balance
```
**Authentication:** Required
**Parameters:**
- `sub-uid` (optional): Sub-user ID

**Response:**
```json
{
  "status": "ok",
  "data": {
    "id": 123456,
    "type": "cross-margin",
    "state": "working",
    "risk-rate": "1.5",
    "acct-balance-sum": "10000.00",
    "debt-balance-sum": "5000.00",
    "list": [
      {
        "currency": "btc",
        "type": "trade",
        "balance": "0.1"
      }
    ]
  }
}
```

## Wallet Endpoints

### Get Deposit Address
```
GET /v2/account/deposit/address
```
**Authentication:** Required
**Parameters:**
- `currency` (required): Currency code

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "currency": "usdt",
      "address": "0x1234567890abcdef",
      "addressTag": "",
      "chain": "trc20usdt"
    }
  ]
}
```

### Create Withdrawal
```
POST /v1/dw/withdraw/api/create
```
**Authentication:** Required
**Parameters (JSON body):**
- `address` (required): Withdrawal address
- `amount` (required): Withdrawal amount
- `currency` (required): Currency code
- `fee` (required): Transaction fee
- `chain` (optional): Blockchain name
- `addr-tag` (optional): Address tag
- `client-order-id` (optional): Client order ID

**Response:**
```json
{
  "status": "ok",
  "data": 100001234567
}
```

### Get Withdrawal Quota
```
GET /v2/account/withdraw/quota
```
**Authentication:** Required
**Parameters:**
- `currency` (required): Currency code

**Response:**
```json
{
  "code": 200,
  "data": {
    "currency": "btc",
    "chains": [
      {
        "chain": "btc",
        "maxWithdrawAmt": "100.00",
        "withdrawQuotaPerDay": "100.00",
        "remainWithdrawQuotaPerDay": "100.00",
        "withdrawQuotaPerYear": "36500.00",
        "remainWithdrawQuotaPerYear": "36500.00",
        "withdrawQuotaTotal": "100000.00",
        "remainWithdrawQuotaTotal": "100000.00"
      }
    ]
  }
}
```

### Cancel Withdrawal
```
POST /v1/dw/withdraw/{withdraw-id}/cancel
```
**Authentication:** Required
**Path Parameters:**
- `withdraw-id` (required): Withdrawal ID

**Response:**
```json
{
  "status": "ok",
  "data": 100001234567
}
```

### Query Deposit/Withdrawal History
```
GET /v1/query/deposit-withdraw
```
**Authentication:** Required
**Parameters:**
- `type` (required): deposit, withdraw
- `currency` (optional): Currency code
- `from` (optional): Starting record ID
- `size` (optional): Number of records (max 500)
- `direct` (optional): prev, next

**Response:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "type": "deposit",
      "currency": "usdt",
      "tx-hash": "0xabcdef...",
      "chain": "trc20usdt",
      "amount": 1000.00,
      "address": "0x1234567890abcdef",
      "address-tag": "",
      "fee": 0,
      "state": "safe",
      "created-at": 1234567890,
      "updated-at": 1234567890
    }
  ]
}
```

**Deposit States:**
- `unknown`: Unknown
- `confirming`: Confirming
- `confirmed`: Confirmed
- `safe`: Completed
- `orphan`: Orphan

**Withdrawal States:**
- `submitted`: Submitted
- `reexamine`: Under review
- `canceled`: Canceled
- `pass`: Approved
- `reject`: Rejected
- `pre-transfer`: Pre-transfer
- `wallet-transfer`: Wallet transfer
- `wallet-reject`: Wallet rejected
- `confirmed`: Confirmed
- `confirm-error`: Confirmation error
- `repealed`: Repealed

## Notes

1. All timestamps are in milliseconds (Unix epoch)
2. All prices and amounts are strings to preserve precision
3. Authentication required endpoints need HMAC SHA256 signature
4. Rate limits apply per UID across all API keys
5. Response status "ok" indicates success, "error" indicates failure
6. Maximum query window for orders is 48 hours
7. Client order IDs are optional but recommended for idempotency
