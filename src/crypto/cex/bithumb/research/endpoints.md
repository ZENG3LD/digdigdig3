# Bithumb API Endpoints

## Overview

Bithumb operates two distinct platforms:
1. **Bithumb Korea** (main exchange) - `https://api.bithumb.com`
2. **Bithumb Pro** (global) - `https://global-openapi.bithumb.pro`

This document covers both platforms for comprehensive connector implementation.

---

## Bithumb Korea API

### Base URLs
- **Public API**: `https://api.bithumb.com/public`
- **Private API**: `https://api.bithumb.com`

### Symbol Format
- Format: `{base}_{quote}` (e.g., `BTC_KRW`)
- Parameters: `order_currency` (base) and `payment_currency` (quote)
- Primary quote currency: `KRW` (Korean Won)

---

## MarketData Trait Endpoints

### 1. Get Ticker
**Endpoint**: `GET /public/ticker/{symbol}`

**Example**: `GET /public/ticker/BTC_KRW`

**Parameters**:
- `symbol`: Trading pair (e.g., `BTC_KRW`) or `ALL_KRW` for all tickers

**Response Fields**:
- `opening_price`: Opening price
- `closing_price`: Current/closing price
- `min_price`: 24h low
- `max_price`: 24h high
- `units_traded`: 24h volume (base currency)
- `acc_trade_value`: 24h volume (quote currency)
- `prev_closing_price`: Previous closing price
- `units_traded_24H`: 24h volume
- `acc_trade_value_24H`: 24h quote volume
- `fluctate_24H`: 24h price change
- `fluctate_rate_24H`: 24h price change percentage

### 2. Get Order Book
**Endpoint**: `GET /public/orderbook/{symbol}`

**Example**: `GET /public/orderbook/BTC_KRW`

**Parameters**:
- `symbol`: Trading pair or `ALL_{quote}`
- `count`: Number of levels (default: 30, max: 30)

**Response Fields**:
- `payment_currency`: Quote currency (KRW)
- `order_currency`: Base currency (BTC)
- `bids`: Array of bid orders `[{quantity, price}, ...]`
- `asks`: Array of ask orders `[{quantity, price}, ...]`

### 3. Get Recent Trades
**Endpoint**: `GET /public/transaction_history/{symbol}`

**Example**: `GET /public/transaction_history/BTC_KRW`

**Parameters**:
- `symbol`: Trading pair
- `count`: Number of trades (default: 20, max: 100)

**Response Fields** (array):
- `transaction_date`: Timestamp (format: `YYYY-MM-DD HH:mm:ss`)
- `type`: Trade side (`bid` or `ask`)
- `units_traded`: Quantity
- `price`: Trade price
- `total`: Total value (price * quantity)

### 4. Get Candlestick/OHLCV Data
**Endpoint**: `GET /public/candlestick/{base}_{quote}/{interval}`

**Example**: `GET /public/candlestick/BTC_KRW/24h`

**Parameters**:
- `base`: Base currency (e.g., BTC)
- `quote`: Quote currency (e.g., KRW)
- `interval`: Time interval
  - Minutes: `1m`, `3m`, `5m`, `10m`, `30m`
  - Hours: `1h`, `6h`, `12h`, `24h`

**Response Fields** (array of arrays):
```
[
  [timestamp, open, close, high, low, volume],
  ...
]
```

### 5. Get Server Time
**Bithumb Korea**: No dedicated endpoint (use system time)

**Bithumb Pro**: `GET /openapi/v1/serverTime`

**Response**: Unix timestamp in milliseconds

### 6. Get Exchange Info / Config
**Endpoint**: `GET /public/assetsstatus/{currency}`

**Parameters**:
- `currency`: Currency code or `ALL` for all currencies

**Response Fields**:
- `deposit_status`: Deposit status (0=unavailable, 1=available)
- `withdrawal_status`: Withdrawal status (0=unavailable, 1=available)

---

## Trading Trait Endpoints

### 1. Create Limit Order
**Endpoint**: `POST /trade/place`

**Authentication**: Required (JWT)

**Parameters**:
- `order_currency`: Base currency (e.g., BTC)
- `payment_currency`: Quote currency (e.g., KRW)
- `units`: Order quantity
- `price`: Limit price
- `type`: Order side (`bid` for buy, `ask` for sell)

**Response**:
```json
{
  "status": "0000",
  "order_id": "1234567890"
}
```

### 2. Create Market Order
**Endpoint**:
- Buy: `POST /trade/market_buy`
- Sell: `POST /trade/market_sell`

**Authentication**: Required (JWT)

**Parameters**:
- `order_currency`: Base currency
- `payment_currency`: Quote currency
- `units`: Quantity (for sell) OR `total`: Total value in quote currency (for buy)

**Response**:
```json
{
  "status": "0000",
  "order_id": "1234567890"
}
```

### 3. Cancel Order
**Endpoint**: `POST /trade/cancel`

**Authentication**: Required (JWT)

**Parameters**:
- `order_id`: Order ID to cancel
- `type`: Order type (`bid` or `ask`)
- `order_currency`: Base currency
- `payment_currency`: Quote currency

**Response**:
```json
{
  "status": "0000"
}
```

### 4. Query Order Details
**Endpoint**: `POST /info/order_detail`

**Authentication**: Required (JWT)

**Parameters**:
- `order_id`: Order ID
- `order_currency`: Base currency
- `payment_currency`: Quote currency

**Response Fields**:
- `order_id`: Order ID
- `order_currency`: Base currency
- `payment_currency`: Quote currency
- `order_date`: Order timestamp
- `type`: Order side (`bid`/`ask`)
- `units`: Original quantity
- `units_remaining`: Remaining quantity
- `price`: Order price
- `fee`: Trading fee
- `contract`: Array of fills

### 5. Query Open Orders
**Endpoint**: `POST /info/orders`

**Authentication**: Required (JWT)

**Parameters**:
- `order_currency`: Base currency
- `payment_currency`: Quote currency
- `count`: Number of orders (default: 100, max: 1000)
- `after`: Order ID for pagination (optional)

**Response**: Array of open orders with same fields as order detail

### 6. Query Order History
**Endpoint**: `POST /info/orders` (filtered by status)

**Note**: Same endpoint as open orders, but can be filtered by order status

---

## Account Trait Endpoints

### 1. Get Balance
**Endpoint**: `POST /info/balance`

**Authentication**: Required (JWT)

**Parameters**:
- `currency`: Currency code or `ALL` for all balances

**Response Fields**:
```json
{
  "status": "0000",
  "data": {
    "total_{currency}": "total balance",
    "in_use_{currency}": "locked balance",
    "available_{currency}": "available balance",
    "xcoin_last_{currency}": "average purchase price"
  }
}
```

### 2. Get Account Information
**Endpoint**: `POST /info/account`

**Authentication**: Required (JWT)

**Response Fields**:
- `created`: Account creation timestamp
- `account_id`: User account ID
- `trade_fee`: Trading fee rate
- `balance`: Balance in KRW

### 3. Get Wallet Address
**Endpoint**: `POST /info/wallet_address`

**Authentication**: Required (JWT)

**Parameters**:
- `currency`: Currency code (e.g., BTC, ETH)

**Response Fields**:
- `wallet_address`: Deposit address
- `currency`: Currency code

### 4. Get Deposit History
**Bithumb Korea**: No dedicated public endpoint
**Recommended**: Use transaction history and filter deposits

### 5. Get Withdrawal History
**Bithumb Korea**: No dedicated public endpoint
**Recommended**: Use transaction history and filter withdrawals

### 6. Withdraw
**Endpoint**: `POST /trade/btc_withdrawal`

**Authentication**: Required (JWT + Withdrawal permission)

**Parameters**:
- `units`: Withdrawal amount
- `address`: Destination address
- `currency`: Currency code
- `destination`: Tag/memo (for XRP, XMR, EOS, STEEM, TON)

**Response**:
```json
{
  "status": "0000",
  "message": "Withdrawal request registered"
}
```

---

## Bithumb Pro API (Global Platform)

### Base URL
`https://global-openapi.bithumb.pro/openapi/v1`

### IMPORTANT: Infrastructure Reliability Warning

> **KNOWN ISSUE: REST API Timeouts**
>
> The Bithumb Pro REST API has a known infrastructure problem with 504 Gateway Timeout errors:
> - **Failure Rate**: ~20% of requests timeout (2 out of 10)
> - **Affected**: ALL REST endpoints (public and private)
> - **Status**: Ongoing issue since June 2023 ([GitHub Issue #114](https://github.com/bithumb-pro/bithumb.pro-official-api-docs/issues/114))
> - **Cause**: Backend server capacity/configuration issues behind Cloudflare
> - **WebSocket Alternative**: WebSocket API (`wss://global-api.bithumb.pro`) is on different infrastructure and works reliably
>
> **Recommendations**:
> 1. Use WebSocket for market data (ticker, orderbook, trades)
> 2. Implement aggressive retry logic for REST (7+ retries)
> 3. Use exponential backoff with jitter
> 4. Fall back to WebSocket when REST times out
> 5. See `debug_notes.md` for detailed analysis and solutions

### Symbol Format
- Format: `{BASE}-{QUOTE}` (e.g., `BTC-USDT`)
- Hyphen-separated instead of underscore

### Key Differences from Bithumb Korea:
1. Uses USDT as primary quote currency (not KRW)
2. Different authentication method (HMAC-SHA256 vs JWT)
3. Different response format (JSON with `code`, `msg`, `data`)
4. Supports futures/contract trading
5. **REST API has reliability issues** (see warning above)

### MarketData Endpoints (Bithumb Pro)

#### Get Ticker
**Endpoint**: `GET /spot/ticker`

**Reliability**: Low (20% timeout rate) - **prefer WebSocket**

**Parameters**:
- `symbol`: Trading pair (e.g., `BTC-USDT`) or `ALL`

**Response**:
```json
{
  "code": "0",
  "msg": "success",
  "data": {
    "c": "current_price",
    "h": "24h_high",
    "l": "24h_low",
    "p": "24h_change_percent",
    "v": "24h_volume",
    "s": "symbol"
  }
}
```

#### Get Order Book
**Endpoint**: `GET /spot/orderBook`

**Reliability**: Low (20% timeout rate) - **prefer WebSocket**

**Parameters**:
- `symbol`: Trading pair (required)

**Response**:
```json
{
  "code": "0",
  "data": {
    "b": [[price, quantity], ...],  // bids
    "s": [[price, quantity], ...],  // asks
    "ver": "version_number"
  }
}
```

#### Get Recent Trades
**Endpoint**: `GET /spot/trades`

**Reliability**: Low (20% timeout rate) - **prefer WebSocket**

**Parameters**:
- `symbol`: Trading pair (required)

**Response**: Last 100 trades
```json
{
  "code": "0",
  "data": [
    {
      "p": "price",
      "s": "side",
      "v": "volume",
      "t": "timestamp"
    }
  ]
}
```

#### Get Candlestick Data
**Endpoint**: `GET /spot/kline`

**Reliability**: Low (20% timeout rate)

**Parameters**:
- `symbol`: Trading pair (required)
- `type`: Interval (required) - `m1`, `m5`, `m15`, `m30`, `h1`, `h4`, `d1`, `w1`, `M1`
- `start`: Start time in seconds (required)
- `end`: End time in seconds (required)

**Response**: OHLCV array

### Trading Endpoints (Bithumb Pro)

**Note**: All trading endpoints subject to 20% timeout rate. Implement retry logic.

#### Create Order
**Endpoint**: `POST /spot/placeOrder`

**Authentication**: Required

**Parameters**:
- `symbol`: Trading pair
- `type`: Order type (`limit`, `market`)
- `side`: Order side (`buy`, `sell`)
- `price`: Price (for limit orders)
- `quantity`: Quantity

**Rate Limit**: 10 requests/second

#### Cancel Order
**Endpoint**: `POST /spot/cancelOrder`

**Authentication**: Required

**Parameters**:
- `orderId`: Order ID to cancel

#### Query Account
**Endpoint**: `POST /spot/account`

**Authentication**: Required

**Response**: Account balances

#### Query Order Detail
**Endpoint**: `POST /spot/orderDetail`

**Authentication**: Required

**Parameters**:
- `orderId`: Order ID

#### Query Open Orders
**Endpoint**: `POST /spot/openOrders`

**Authentication**: Required

#### Query Order History
**Endpoint**: `POST /spot/historyOrders`

**Authentication**: Required

**Parameters**:
- `symbol`: Trading pair
- Pagination options

### Account Endpoints (Bithumb Pro)

#### Withdraw
**Endpoint**: `POST /withdraw`

**Authentication**: Required (withdraw permission)

**Parameters**:
- `coinType`: Currency (e.g., BTC, USDT)
- `address`: Destination wallet
- `quantity`: Amount
- `mark`: Description (max 250 chars)
- `extendParam`: Memo/tag (optional)

#### Deposit History
**Endpoint**: `POST /wallet/depositHistory`

**Authentication**: Required

**Parameters**:
- `start`: Start timestamp (required)
- `end`: End timestamp (optional, max 90-day range)
- `coin`: Currency filter (optional)
- `limit`: Max records (default: 50, max: 50)

#### Withdrawal History
**Endpoint**: `POST /wallet/withdrawHistory`

**Authentication**: Required

**Parameters**:
- `start`: Start time (required)
- `end`: End time (optional)
- `coin`: Currency filter (optional)
- `limit`: Max records (default: 50)

**Response**: Status codes
- `0/1/2/3`: Pending
- `7`: Success
- `8`: Failed

---

## Response Format Differences

### Bithumb Korea
```json
{
  "status": "0000",  // "0000" = success, others = error codes
  "data": {...}
}
```

### Bithumb Pro
```json
{
  "code": "0",      // "0" or codes < 10000 = success
  "success": true,
  "msg": "",
  "data": {...},
  "params": []
}
```

---

## Implementation Notes

1. **Choose Platform**: Decide whether to support Bithumb Korea (KRW markets) or Bithumb Pro (USDT markets), or both
2. **Symbol Formatting**:
   - Korea: `BTC_KRW` with separate `order_currency`/`payment_currency`
   - Pro: `BTC-USDT` hyphen-separated
3. **Authentication**: Different methods (see authentication.md)
4. **Rate Limits**: See rate_limits.md
5. **Error Handling**: Different status code schemes
6. **Market Buy Orders**: Korea platform allows specifying total value in quote currency
7. **REST API Reliability**: Bithumb Pro REST API has infrastructure issues (see warning above)
   - **Strongly recommend using WebSocket for market data**
   - Implement robust retry logic for REST trading operations
   - See `debug_notes.md` for detailed solutions
