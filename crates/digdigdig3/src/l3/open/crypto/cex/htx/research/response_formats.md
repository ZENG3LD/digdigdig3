# HTX API Response Formats

Complete response format documentation for HTX (formerly Huobi) exchange API.

## General Response Structure

### REST API Response Format

HTX uses two different response formats depending on API version:

#### V1 Format (Most endpoints)

```json
{
  "status": "ok|error",
  "ch": "channel name",
  "ts": 1234567890,
  "data": {}
}
```

**Fields:**
- `status`: "ok" for success, "error" for failure
- `ch` (optional): Channel/topic name (market data endpoints)
- `ts`: Response timestamp (milliseconds)
- `data`: Response payload (varies by endpoint)

#### V2 Format (Newer endpoints)

```json
{
  "code": 200,
  "message": "success",
  "data": {}
}
```

**Fields:**
- `code`: HTTP-like status code (200 = success)
- `message` (optional): Status message
- `data`: Response payload

### Error Response Format

#### V1 Error Format

```json
{
  "status": "error",
  "err-code": "order-limitorder-amount-min-error",
  "err-msg": "limit order amount error, min: 0.001",
  "data": null
}
```

**Fields:**
- `status`: Always "error"
- `err-code`: Error code string
- `err-msg`: Human-readable error message
- `data`: Usually null

#### V2 Error Format

```json
{
  "code": 1002,
  "message": "invalid.param.symbol",
  "data": null
}
```

**Fields:**
- `code`: Error code number
- `message`: Error message
- `data`: Usually null

## Market Data Response Formats

### Ticker (24hr Stats)

```json
{
  "status": "ok",
  "ch": "market.btcusdt.detail.merged",
  "ts": 1629384000000,
  "tick": {
    "id": 311869842476,
    "amount": 18344.5126,
    "count": 89472,
    "open": 48000.00,
    "close": 49500.00,
    "low": 47500.00,
    "high": 50000.00,
    "vol": 896748251.2574,
    "bid": [49499.00, 1.5],
    "ask": [49500.00, 2.3]
  }
}
```

**Field Types:**
- `id`: long - Ticker ID
- `amount`: float - 24h volume in base currency
- `count`: int - Number of trades
- `open`: float - Opening price
- `close`: float - Latest price
- `low`: float - Lowest price
- `high`: float - Highest price
- `vol`: float - 24h volume in quote currency
- `bid`: [price, size] - Best bid
- `ask`: [price, size] - Best ask

### Order Book Depth

```json
{
  "status": "ok",
  "ch": "market.btcusdt.depth.step0",
  "ts": 1629384000000,
  "tick": {
    "version": 311869842476,
    "ts": 1629384000000,
    "bids": [
      [49499.00, 1.5234],
      [49498.00, 2.4567],
      [49497.00, 0.8901]
    ],
    "asks": [
      [49500.00, 2.3456],
      [49501.00, 1.7890],
      [49502.00, 3.1234]
    ]
  }
}
```

**Field Types:**
- `version`: long - Order book version number
- `ts`: long - Timestamp
- `bids`: [[price, size]] - Buy orders (descending price)
- `asks`: [[price, size]] - Sell orders (ascending price)

### Trades

```json
{
  "status": "ok",
  "ch": "market.btcusdt.trade.detail",
  "ts": 1629384000000,
  "tick": {
    "id": 311869842476,
    "ts": 1629384000000,
    "data": [
      {
        "id": 311869842476123,
        "ts": 1629384000000,
        "trade-id": 100001234567,
        "amount": 0.1234,
        "price": 49500.00,
        "direction": "buy"
      }
    ]
  }
}
```

**Field Types:**
- `id`: long - Tick ID
- `ts`: long - Timestamp
- `trade-id`: long - Unique trade ID
- `amount`: float - Trade size
- `price`: float - Trade price
- `direction`: string - "buy" or "sell"

### Klines/Candlesticks

```json
{
  "status": "ok",
  "ch": "market.btcusdt.kline.1day",
  "ts": 1629384000000,
  "data": [
    {
      "id": 1629331200,
      "amount": 18344.5126,
      "count": 89472,
      "open": 48000.00,
      "close": 49500.00,
      "low": 47500.00,
      "high": 50000.00,
      "vol": 896748251.2574
    }
  ]
}
```

**Field Types:**
- `id`: long - Kline start time (Unix timestamp in seconds)
- `amount`: float - Volume in base currency
- `count`: int - Number of trades
- `open`: float - Opening price
- `close`: float - Closing price
- `low`: float - Lowest price
- `high`: float - Highest price
- `vol`: float - Volume in quote currency

### All Tickers

```json
{
  "status": "ok",
  "ts": 1629384000000,
  "data": [
    {
      "symbol": "btcusdt",
      "open": 48000.00,
      "high": 50000.00,
      "low": 47500.00,
      "close": 49500.00,
      "amount": 18344.5126,
      "vol": 896748251.2574,
      "count": 89472,
      "bid": 49499.00,
      "bidSize": 1.5234,
      "ask": 49500.00,
      "askSize": 2.3456
    }
  ]
}
```

## Account Response Formats

### Account List

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

**Field Types:**
- `id`: long - Account ID
- `type`: string - Account type (spot, margin, otc, etc.)
- `subtype`: string - Account subtype
- `state`: string - Account state (working, lock)

**Account Types:**
- `spot`: Spot trading account
- `margin`: Cross-margin account
- `otc`: OTC account
- `point`: Point card account
- `super-margin`: Isolated margin account
- `investment`: Wealth management account
- `borrow`: Borrow account
- `grid-trading`: Grid trading account

### Account Balance

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
        "balance": "10000.1234567890"
      },
      {
        "currency": "usdt",
        "type": "frozen",
        "balance": "500.0000000000"
      },
      {
        "currency": "btc",
        "type": "trade",
        "balance": "0.5000000000"
      }
    ]
  }
}
```

**Field Types:**
- `id`: long - Account ID
- `type`: string - Account type
- `state`: string - Account state
- `currency`: string - Currency code
- `balance`: string - Balance amount (high precision string)

**Balance Types:**
- `trade`: Available balance
- `frozen`: Frozen/locked balance

### Asset Valuation

```json
{
  "code": 200,
  "data": {
    "balance": "125000.5432",
    "timestamp": 1629384000000
  }
}
```

**Field Types:**
- `balance`: string - Total asset value in specified currency
- `timestamp`: long - Calculation timestamp

### Account Ledger

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
      "transactTime": 1629384000000,
      "transferer": 123456,
      "transferee": 789012
    }
  ],
  "nextId": 100001234568
}
```

**Field Types:**
- `accountId`: long - Account ID
- `currency`: string - Currency code
- `transactAmt`: string - Transaction amount
- `transactType`: string - Transaction type
- `transferType`: string - Transfer type
- `transactId`: long - Transaction ID
- `transactTime`: long - Transaction timestamp
- `transferer`: long - Source account
- `transferee`: long - Destination account
- `nextId`: long - Next page start ID

## Trading Response Formats

### Order Placement

```json
{
  "status": "ok",
  "data": "100001234567"
}
```

**Field Types:**
- `data`: string - Order ID

### Batch Order Placement

```json
{
  "status": "ok",
  "data": [
    {
      "order-id": 100001234567,
      "client-order-id": "my-order-1"
    },
    {
      "err-code": "invalid-amount",
      "err-msg": "Invalid amount",
      "client-order-id": "my-order-2"
    }
  ]
}
```

**Success Entry:**
- `order-id`: long - Order ID
- `client-order-id`: string - Client order ID (if provided)

**Error Entry:**
- `err-code`: string - Error code
- `err-msg`: string - Error message
- `client-order-id`: string - Client order ID (if provided)

### Order Cancellation

```json
{
  "status": "ok",
  "data": "100001234567"
}
```

**Field Types:**
- `data`: string - Canceled order ID

### Batch Order Cancellation

```json
{
  "status": "ok",
  "data": {
    "success": [
      "100001234567",
      "100001234568"
    ],
    "failed": [
      {
        "order-id": "100001234569",
        "err-code": "order-orderstate-error",
        "err-msg": "Invalid order state"
      }
    ]
  }
}
```

**Field Types:**
- `success`: [string] - Successfully canceled order IDs
- `failed`: [object] - Failed cancellation entries

### Order Details

```json
{
  "status": "ok",
  "data": {
    "id": 100001234567,
    "symbol": "btcusdt",
    "account-id": 123456,
    "client-order-id": "my-order-1",
    "amount": "0.1000",
    "price": "50000.00",
    "created-at": 1629384000000,
    "type": "buy-limit",
    "field-amount": "0.0500",
    "field-cash-amount": "2500.00",
    "field-fees": "0.0001",
    "finished-at": 1629384060000,
    "source": "api",
    "state": "partial-filled",
    "canceled-at": 0
  }
}
```

**Field Types:**
- `id`: long - Order ID
- `symbol`: string - Trading symbol
- `account-id`: long - Account ID
- `client-order-id`: string - Client order ID
- `amount`: string - Order amount
- `price`: string - Order price
- `created-at`: long - Creation timestamp
- `type`: string - Order type
- `field-amount`: string - Filled amount
- `field-cash-amount`: string - Filled value (quote currency)
- `field-fees`: string - Trading fees paid
- `finished-at`: long - Completion timestamp (0 if not finished)
- `source`: string - Order source (api, web, app, etc.)
- `state`: string - Order state
- `canceled-at`: long - Cancellation timestamp (0 if not canceled)

**Order States:**
- `submitted`: Order submitted
- `partial-filled`: Partially filled
- `partial-canceled`: Partially filled then canceled
- `filled`: Fully filled
- `canceled`: Canceled

**Order Types:**
- `buy-market`: Market buy
- `sell-market`: Market sell
- `buy-limit`: Limit buy
- `sell-limit`: Limit sell
- `buy-ioc`: IOC buy
- `sell-ioc`: IOC sell
- `buy-limit-maker`: Post-only buy
- `sell-limit-maker`: Post-only sell
- `buy-stop-limit`: Stop-limit buy
- `sell-stop-limit`: Stop-limit sell
- `buy-limit-fok`: FOK buy
- `sell-limit-fok`: FOK sell

### Open Orders

```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "symbol": "btcusdt",
      "account-id": 123456,
      "client-order-id": "my-order-1",
      "amount": "0.1000",
      "price": "50000.00",
      "created-at": 1629384000000,
      "type": "buy-limit",
      "filled-amount": "0.0500",
      "filled-cash-amount": "2500.00",
      "filled-fees": "0.0001",
      "source": "api",
      "state": "partial-filled"
    }
  ]
}
```

### Match Results

```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "order-id": 100001234567,
      "match-id": 200001234567,
      "symbol": "btcusdt",
      "type": "buy-limit",
      "source": "api",
      "price": "50000.00",
      "filled-amount": "0.0500",
      "filled-fees": "0.0001",
      "fee-currency": "btc",
      "created-at": 1629384000000,
      "role": "taker"
    }
  ]
}
```

**Field Types:**
- `id`: long - Trade ID
- `order-id`: long - Order ID
- `match-id`: long - Match ID
- `symbol`: string - Trading symbol
- `type`: string - Order type
- `source`: string - Order source
- `price`: string - Fill price
- `filled-amount`: string - Fill amount
- `filled-fees`: string - Fees paid
- `fee-currency`: string - Fee currency
- `created-at`: long - Fill timestamp
- `role`: string - "maker" or "taker"

### Trading Fees

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

**Field Types:**
- `symbol`: string - Trading symbol
- `actualMakerRate`: string - Actual maker fee rate
- `actualTakerRate`: string - Actual taker fee rate
- `takerFeeRate`: string - Base taker fee rate
- `makerFeeRate`: string - Base maker fee rate

## Wallet Response Formats

### Deposit Address

```json
{
  "code": 200,
  "data": [
    {
      "currency": "usdt",
      "address": "TXmF9kJxQrESTgTQ4v8KjCkZBKRTwHvW1X",
      "addressTag": "",
      "chain": "trc20usdt",
      "depositAddress": "TXmF9kJxQrESTgTQ4v8KjCkZBKRTwHvW1X"
    }
  ]
}
```

**Field Types:**
- `currency`: string - Currency code
- `address`: string - Deposit address
- `addressTag`: string - Address tag/memo (if applicable)
- `chain`: string - Blockchain network
- `depositAddress`: string - Deposit address (same as address)

### Withdrawal Creation

```json
{
  "status": "ok",
  "data": 100001234567
}
```

**Field Types:**
- `data`: long - Withdrawal ID

### Withdrawal Quota

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
        "remainWithdrawQuotaPerDay": "95.00",
        "withdrawQuotaPerYear": "36500.00",
        "remainWithdrawQuotaPerYear": "36450.00",
        "withdrawQuotaTotal": "100000.00",
        "remainWithdrawQuotaTotal": "99900.00"
      }
    ]
  }
}
```

**Field Types:**
- `currency`: string - Currency code
- `chain`: string - Blockchain network
- `maxWithdrawAmt`: string - Maximum withdrawal amount
- `withdrawQuotaPerDay`: string - Daily quota
- `remainWithdrawQuotaPerDay`: string - Remaining daily quota
- `withdrawQuotaPerYear`: string - Yearly quota
- `remainWithdrawQuotaPerYear`: string - Remaining yearly quota
- `withdrawQuotaTotal`: string - Total quota
- `remainWithdrawQuotaTotal`: string - Remaining total quota

### Deposit/Withdrawal History

```json
{
  "status": "ok",
  "data": [
    {
      "id": 100001234567,
      "type": "deposit",
      "sub-type": "NORMAL",
      "currency": "usdt",
      "tx-hash": "0x1234567890abcdef...",
      "chain": "trc20usdt",
      "amount": 1000.00,
      "from-addr-tag": "",
      "address": "TXmF9kJxQrESTgTQ4v8KjCkZBKRTwHvW1X",
      "address-tag": "",
      "fee": 0,
      "state": "safe",
      "error-code": "",
      "error-msg": "",
      "created-at": 1629384000000,
      "updated-at": 1629384060000
    }
  ]
}
```

**Field Types:**
- `id`: long - Transaction ID
- `type`: string - "deposit" or "withdraw"
- `sub-type`: string - Transaction subtype
- `currency`: string - Currency code
- `tx-hash`: string - Blockchain transaction hash
- `chain`: string - Blockchain network
- `amount`: float - Transaction amount
- `from-addr-tag`: string - Source address tag
- `address`: string - Deposit/withdrawal address
- `address-tag`: string - Address tag
- `fee`: float - Transaction fee
- `state`: string - Transaction state
- `error-code`: string - Error code (if failed)
- `error-msg`: string - Error message (if failed)
- `created-at`: long - Creation timestamp
- `updated-at`: long - Update timestamp

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

## Error Response Formats

### Common Error Codes

| Error Code | Message | Description |
|------------|---------|-------------|
| `invalid-parameter` | Invalid parameter | Missing or invalid parameter |
| `invalid-command` | Invalid command | Invalid API endpoint |
| `api-signature-not-valid` | Signature verification failed | HMAC signature invalid |
| `gateway-internal-error` | Internal error | Server error, retry |
| `account-frozen-balance-insufficient-error` | Insufficient balance | Not enough available balance |
| `order-limitorder-amount-min-error` | Amount too small | Below minimum order size |
| `order-limitorder-amount-max-error` | Amount too large | Above maximum order size |
| `order-orderstate-error` | Invalid order state | Order cannot be canceled |
| `order-queryorder-invalid` | Order not found | Order ID doesn't exist |
| `order-update-error` | Order update failed | Order modification failed |
| `api-key-invalid` | Invalid API key | API key not found |
| `login-required` | Login required | Missing authentication |
| `method-not-allowed` | Method not allowed | Invalid HTTP method |
| `invalid-timestamp` | Invalid timestamp | Timestamp out of range |

## Data Type Reference

### Primitive Types

- **string**: Text values (quoted in JSON)
- **int**: 32-bit integer
- **long**: 64-bit integer (IDs, timestamps)
- **float**: Decimal number (prices, amounts)
- **boolean**: true/false

### Precision Handling

**IMPORTANT:** All prices, amounts, and balances are returned as **strings** to preserve full precision.

```json
{
  "amount": "0.1234567890",  // String, not number
  "price": "50000.12345678", // String, not number
  "balance": "10000.00"      // String, not number
}
```

**Why strings?**
- Avoids floating-point precision loss
- Preserves exact decimal values
- Required for financial calculations

**Implementation:**
```rust
// Parse strings to Decimal for calculations
use rust_decimal::Decimal;
use std::str::FromStr;

let amount = Decimal::from_str("0.1234567890")?;
let price = Decimal::from_str("50000.12345678")?;
let total = amount * price;
```

### Timestamp Format

All timestamps are Unix epoch in **milliseconds** (not seconds):

```json
{
  "created-at": 1629384000000,  // 2021-08-19 12:00:00 UTC
  "ts": 1629384000000
}
```

**Conversion:**
```rust
use chrono::{DateTime, Utc, TimeZone};

let timestamp_ms = 1629384000000i64;
let datetime = Utc.timestamp_millis_opt(timestamp_ms).unwrap();
```

## Response Headers

### Rate Limit Headers

```
X-HB-RateLimit-Requests-Remain: 95
X-HB-RateLimit-Requests-Expire: 1629384060000
```

- `X-HB-RateLimit-Requests-Remain`: Remaining requests in window
- `X-HB-RateLimit-Requests-Expire`: Window expiration (ms)

### Standard Headers

```
Content-Type: application/json
Content-Encoding: gzip
```

## Pagination

Endpoints supporting pagination use cursor-based pagination:

**Request Parameters:**
- `from`: Starting record ID
- `direct`: "prev" (older) or "next" (newer)
- `size`: Number of records

**Response:**
```json
{
  "status": "ok",
  "data": [...],
  "next-id": 100001234568
}
```

Use `next-id` as `from` parameter for next page.

## Notes

1. Always check `status` field ("ok" or "error")
2. Parse numeric fields from strings for precision
3. Handle both V1 and V2 response formats
4. Timestamps are in milliseconds
5. Error responses include `err-code` and `err-msg`
6. Rate limit headers present in all responses
7. Null values may be omitted or explicit null
