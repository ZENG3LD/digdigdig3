# Upbit API Endpoints Research

**Research Date**: 2026-01-20

This document contains comprehensive research on Upbit's official API endpoints for trading and market data.

---

## Table of Contents

- [Base URLs](#base-urls)
- [Market Data Endpoints (Quotation API)](#market-data-endpoints-quotation-api)
  - [Trading Pairs](#trading-pairs)
  - [Candles/Klines](#candlesklines)
  - [Ticker](#ticker)
  - [Orderbook](#orderbook)
  - [Recent Trades](#recent-trades)
- [Trading Endpoints (Exchange API)](#trading-endpoints-exchange-api)
  - [Orders](#orders)
- [Account Endpoints (Exchange API)](#account-endpoints-exchange-api)
  - [Balances](#balances)
  - [Deposits](#deposits)
  - [Withdrawals](#withdrawals)

---

## Base URLs

### Production (Mainnet)

| Region | Base URL |
|--------|----------|
| **Singapore** | `https://sg-api.upbit.com` |
| **Indonesia** | `https://id-api.upbit.com` |
| **Thailand** | `https://th-api.upbit.com` |

**Note**: Upbit operates regional exchanges. Select the appropriate base URL based on your target market.

---

## Market Data Endpoints (Quotation API)

All Quotation API endpoints are **public** (no authentication required).

### Trading Pairs

#### Get All Trading Pairs

- **Endpoint**: `GET /v1/trading-pairs`
- **Full URL**: `https://sg-api.upbit.com/v1/trading-pairs`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: market group)
- **Description**: Retrieve list of all available trading pairs on the exchange
- **Query Parameters**: None
- **Response**: Array of market identifiers

**Example Response**:
```json
[
  "SGD-BTC",
  "SGD-ETH",
  "BTC-ETH",
  "USDT-BTC"
]
```

---

### Candles/Klines

Upbit provides multiple endpoints for different candlestick intervals.

#### 1. Minute Candles

- **Endpoint**: `GET /v1/candles/minutes/{unit}`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/minutes/1?market=SGD-BTC&count=200`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)
- **Supported Units** (path parameter):
  - `1` - 1 minute
  - `3` - 3 minutes
  - `5` - 5 minutes
  - `10` - 10 minutes
  - `15` - 15 minutes
  - `30` - 30 minutes
  - `60` - 1 hour
  - `240` - 4 hours

**Query Parameters**:
- `market` (required, string): Market identifier (e.g., "SGD-BTC")
- `to` (optional, string): End timestamp in ISO 8601 format (e.g., "2024-06-19T08:31:43Z")
- `count` (optional, integer): Number of candles to retrieve (default: 200, max: 200)

**Response Fields**:
- `market` (string): Market ID
- `candle_date_time_utc` (string): Candle time in UTC (ISO 8601)
- `candle_date_time_kst` (string): Candle time in KST (ISO 8601)
- `opening_price` (number): Opening price
- `high_price` (number): Highest price
- `low_price` (number): Lowest price
- `trade_price` (number): Closing price
- `timestamp` (integer): Last trade timestamp for candle (milliseconds)
- `candle_acc_trade_price` (number): Accumulated trade volume (quote currency)
- `candle_acc_trade_volume` (number): Accumulated trade volume (base currency)
- `unit` (integer): Candle unit in minutes

**Example Response**:
```json
[
  {
    "market": "SGD-BTC",
    "candle_date_time_utc": "2024-06-19T08:31:00",
    "candle_date_time_kst": "2024-06-19T17:31:00",
    "opening_price": 67000.0,
    "high_price": 67500.0,
    "low_price": 66900.0,
    "trade_price": 67300.0,
    "timestamp": 1718788299000,
    "candle_acc_trade_price": 1234567.89,
    "candle_acc_trade_volume": 18.45,
    "unit": 1
  }
]
```

**Notes**:
- Returns array sorted by timestamp descending (newest first)
- All price values are numeric (not strings)
- Maximum 200 candles per request

---

#### 2. Day Candles

- **Endpoint**: `GET /v1/candles/days`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/days?market=SGD-BTC&count=30`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)

**Query Parameters**:
- `market` (required, string): Market identifier
- `to` (optional, string): End timestamp in ISO 8601 format
- `count` (optional, integer): Number of candles (default: 1, max: 200)
- `convertingPriceUnit` (optional, string): Converted price unit (e.g., "SGD")

**Response**: Similar structure to minute candles

---

#### 3. Week Candles

- **Endpoint**: `GET /v1/candles/weeks`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/weeks?market=SGD-BTC&count=10`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)

**Query Parameters**: Same as day candles

---

#### 4. Month Candles

- **Endpoint**: `GET /v1/candles/months`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/months?market=SGD-BTC&count=12`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)

**Query Parameters**: Same as day candles

---

#### 5. Year Candles

- **Endpoint**: `GET /v1/candles/years`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/years?market=SGD-BTC&count=5`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)

**Query Parameters**: Same as day candles

---

#### 6. Second Candles (Intraday)

- **Endpoint**: `GET /v1/candles/seconds`
- **Full URL**: `https://sg-api.upbit.com/v1/candles/seconds?market=SGD-BTC&count=100`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: candle group)
- **Description**: Second-level candlestick data for high-frequency analysis

**Query Parameters**: Same as minute candles

---

### Ticker

#### Get Ticker by Trading Pairs

- **Endpoint**: `GET /v1/tickers`
- **Full URL**: `https://sg-api.upbit.com/v1/tickers?markets=SGD-BTC,SGD-ETH`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: ticker group)
- **Description**: Retrieve current ticker data for specified trading pairs

**Query Parameters**:
- `markets` (required, string): Comma-separated list of market identifiers

**Response Fields**:
- `market` (string): Market identifier
- `trade_date` (string): Recent trade date (YYYYMMDD)
- `trade_time` (string): Recent trade time (HHmmss)
- `trade_date_kst` (string): Trade date in KST
- `trade_time_kst` (string): Trade time in KST
- `trade_timestamp` (integer): Trade timestamp (milliseconds)
- `opening_price` (number): Opening price (00:00:00 UTC)
- `high_price` (number): Highest price in 24h
- `low_price` (number): Lowest price in 24h
- `trade_price` (number): Most recent price
- `prev_closing_price` (number): Previous day closing price
- `change` (string): Price change type ("RISE", "EVEN", "FALL")
- `change_price` (number): Price change from previous close (absolute)
- `change_rate` (number): Price change rate (decimal, e.g., 0.0231)
- `signed_change_price` (number): Signed price change (+ for rise, - for fall)
- `signed_change_rate` (number): Signed change rate
- `trade_volume` (number): Most recent trade volume
- `acc_trade_price` (number): 24h accumulated trade value (quote currency)
- `acc_trade_price_24h` (number): Last 24h accumulated trade value
- `acc_trade_volume` (number): 24h accumulated trade volume (base currency)
- `acc_trade_volume_24h` (number): Last 24h accumulated trade volume
- `highest_52_week_price` (number): 52-week high
- `highest_52_week_date` (string): 52-week high date
- `lowest_52_week_price` (number): 52-week low
- `lowest_52_week_date` (string): 52-week low date
- `timestamp` (integer): Timestamp (milliseconds)

**Example Response**:
```json
[
  {
    "market": "SGD-BTC",
    "trade_date": "20240619",
    "trade_time": "083143",
    "trade_date_kst": "20240619",
    "trade_time_kst": "173143",
    "trade_timestamp": 1718788303000,
    "opening_price": 66000.0,
    "high_price": 68000.0,
    "low_price": 65500.0,
    "trade_price": 67300.0,
    "prev_closing_price": 66000.0,
    "change": "RISE",
    "change_price": 1300.0,
    "change_rate": 0.0197,
    "signed_change_price": 1300.0,
    "signed_change_rate": 0.0197,
    "trade_volume": 0.15,
    "acc_trade_price": 45678901.23,
    "acc_trade_price_24h": 45678901.23,
    "acc_trade_volume": 678.45,
    "acc_trade_volume_24h": 678.45,
    "highest_52_week_price": 85000.0,
    "highest_52_week_date": "2023-11-15",
    "lowest_52_week_price": 25000.0,
    "lowest_52_week_date": "2023-07-01",
    "timestamp": 1718788303000
  }
]
```

**Notes**:
- Supports multiple markets in single request (comma-separated)
- All numeric values are numbers (not strings)
- Timestamps in milliseconds

---

#### Get Ticker by Quote Currency

- **Endpoint**: `GET /v1/tickers/quote`
- **Full URL**: `https://sg-api.upbit.com/v1/tickers/quote?quote=SGD`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: ticker group)
- **Description**: Retrieve tickers for all pairs with specified quote currency

**Query Parameters**:
- `quote` (required, string): Quote currency (e.g., "SGD", "BTC", "USDT")

**Response**: Array of ticker objects (same structure as above)

---

### Orderbook

#### Get Orderbook

- **Endpoint**: `GET /v1/orderbooks`
- **Full URL**: `https://sg-api.upbit.com/v1/orderbooks?markets=SGD-BTC`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: orderbook group)
- **Description**: Retrieve current order book data

**Query Parameters**:
- `markets` (required, string): Comma-separated market identifiers (max: 1)
- `level` (optional, integer): Number of price levels to return (max: 30)

**Response Fields**:
- `market` (string): Market identifier
- `timestamp` (integer): Orderbook timestamp (milliseconds)
- `total_ask_size` (number): Total ask volume
- `total_bid_size` (number): Total bid volume
- `orderbook_units` (array): Array of price level objects
  - `ask_price` (number): Ask price
  - `bid_price` (number): Bid price
  - `ask_size` (number): Ask volume at this level
  - `bid_size` (number): Bid volume at this level

**Example Response**:
```json
[
  {
    "market": "SGD-BTC",
    "timestamp": 1718788303000,
    "total_ask_size": 123.45,
    "total_bid_size": 234.56,
    "orderbook_units": [
      {
        "ask_price": 67500.0,
        "bid_price": 67300.0,
        "ask_size": 5.23,
        "bid_size": 6.78
      },
      {
        "ask_price": 67600.0,
        "bid_price": 67200.0,
        "ask_size": 3.45,
        "bid_size": 4.12
      }
    ]
  }
]
```

**Notes**:
- Orderbook levels sorted by price (asks ascending, bids descending)
- Total sizes represent cumulative volume across all levels
- Maximum 30 price levels per request

---

#### Get Orderbook Instruments

- **Endpoint**: `GET /v1/orderbook-instruments`
- **Full URL**: `https://sg-api.upbit.com/v1/orderbook-instruments`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: orderbook group)
- **Description**: List markets that support orderbook data

**Query Parameters**: None

---

### Recent Trades

#### Get Recent Trades

- **Endpoint**: `GET /v1/trades/recent`
- **Full URL**: `https://sg-api.upbit.com/v1/trades/recent?market=SGD-BTC&count=100`
- **Auth Required**: No (Public)
- **Rate Limit**: 10 requests/second (Quotation: trade group)
- **Description**: Retrieve recent trade execution history

**Query Parameters**:
- `market` (required, string): Market identifier
- `to` (optional, string): Last trade ID (for pagination)
- `count` (optional, integer): Number of trades to return (default: 100, max: 500)
- `daysAgo` (optional, integer): Days ago to retrieve (max: 7)

**Response Fields**:
- `market` (string): Market identifier
- `trade_date_utc` (string): Trade date in UTC (YYYY-MM-DD)
- `trade_time_utc` (string): Trade time in UTC (HH:mm:ss)
- `timestamp` (integer): Trade timestamp (milliseconds)
- `trade_price` (number): Execution price
- `trade_volume` (number): Execution volume
- `prev_closing_price` (number): Previous closing price
- `change_price` (number): Price change from previous close
- `ask_bid` (string): "ASK" (sell) or "BID" (buy)
- `sequential_id` (integer): Sequential trade ID

**Example Response**:
```json
[
  {
    "market": "SGD-BTC",
    "trade_date_utc": "2024-06-19",
    "trade_time_utc": "08:31:43",
    "timestamp": 1718788303000,
    "trade_price": 67300.0,
    "trade_volume": 0.15,
    "prev_closing_price": 66000.0,
    "change_price": 1300.0,
    "ask_bid": "BID",
    "sequential_id": 1234567890123
  }
]
```

**Notes**:
- Sorted by timestamp descending (newest first)
- `sequential_id` can be used for pagination with `to` parameter
- Maximum 500 trades per request

---

## Trading Endpoints (Exchange API)

All Exchange API endpoints require **authentication** via JWT Bearer token.

### Orders

#### 1. Get Order Information

- **Endpoint**: `GET /v1/orders/info`
- **Full URL**: `https://sg-api.upbit.com/v1/orders/info`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **Description**: Get information about available order types and constraints

**Query Parameters**: None

---

#### 2. Create Order

- **Endpoint**: `POST /v1/orders`
- **Full URL**: `https://sg-api.upbit.com/v1/orders`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 8 requests/second (Exchange: order group)
- **Permission**: "Make Orders"
- **HTTP Method**: POST
- **Content-Type**: `application/json; charset=utf-8`

**Request Body** (JSON):
- `market` (required, string): Market identifier (e.g., "SGD-BTC")
- `side` (required, string): "bid" (buy) or "ask" (sell)
- `volume` (optional, string): Order volume (for limit orders or sell market orders)
- `price` (optional, string): Order price per unit (for limit orders)
- `ord_type` (required, string): Order type
  - `"limit"` - Limit order (requires `price` and `volume`)
  - `"price"` - Market buy (requires `price` as total amount to spend)
  - `"market"` - Market sell (requires `volume`)
- `identifier` (optional, string): Custom order identifier (max 40 chars, for idempotency)
- `time_in_force` (optional, string): "IOC" (Immediate or Cancel) or "FOK" (Fill or Kill)

**Response Fields**:
- `uuid` (string): Order UUID
- `side` (string): "bid" or "ask"
- `ord_type` (string): Order type
- `price` (string): Order price
- `state` (string): Order state ("wait", "watch", "done", "cancel")
- `market` (string): Market identifier
- `created_at` (string): Creation timestamp (ISO 8601)
- `volume` (string): Order volume
- `remaining_volume` (string): Remaining unfilled volume
- `reserved_fee` (string): Reserved fee amount
- `remaining_fee` (string): Remaining fee
- `paid_fee` (string): Paid fee
- `locked` (string): Locked amount
- `executed_volume` (string): Executed volume
- `trades_count` (integer): Number of trades

**Example Request**:
```json
{
  "market": "SGD-BTC",
  "side": "bid",
  "volume": "0.1",
  "price": "67000",
  "ord_type": "limit"
}
```

**Example Response**:
```json
{
  "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "side": "bid",
  "ord_type": "limit",
  "price": "67000.0",
  "state": "wait",
  "market": "SGD-BTC",
  "created_at": "2024-06-19T08:31:43+00:00",
  "volume": "0.1",
  "remaining_volume": "0.1",
  "reserved_fee": "0.5",
  "remaining_fee": "0.5",
  "paid_fee": "0.0",
  "locked": "6700.5",
  "executed_volume": "0.0",
  "trades_count": 0
}
```

**Notes**:
- Limit orders require both `price` and `volume`
- Market buy orders use `price` to specify total amount in quote currency
- Market sell orders use `volume` to specify amount in base currency
- `identifier` field enables idempotent order placement

---

#### 3. Test Order Creation

- **Endpoint**: `POST /v1/orders/test`
- **Full URL**: `https://sg-api.upbit.com/v1/orders/test`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 8 requests/second (Exchange: order-test group)
- **Permission**: "Make Orders"
- **Description**: Validate order parameters without placing actual order

**Request Body**: Same as Create Order

**Response**: Validation result (success or error details)

---

#### 4. Get Order by UUID

- **Endpoint**: `GET /v1/orders/{order-id}`
- **Full URL**: `https://sg-api.upbit.com/v1/orders/a1b2c3d4-e5f6-7890-abcd-ef1234567890`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **HTTP Method**: GET

**Path Parameters**:
- `order-id` (string): Order UUID

**Query Parameters**:
- `uuid` (optional, string): Order UUID (alternative to path parameter)
- `identifier` (optional, string): Custom order identifier

**Response**: Same structure as Create Order response, with updated state and execution data

**Notes**:
- Either `uuid` or `identifier` must be provided
- Returns full order details including execution status

---

#### 5. List Orders

- **Endpoint**: `GET /v1/orders`
- **Full URL**: `https://sg-api.upbit.com/v1/orders?market=SGD-BTC&state=wait`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **HTTP Method**: GET

**Query Parameters**:
- `market` (optional, string): Filter by market identifier
- `uuids[]` (optional, array): Filter by order UUIDs (max: 100)
- `identifiers[]` (optional, array): Filter by custom identifiers (max: 100)
- `state` (optional, string): Filter by order state
  - `"wait"` - Orders waiting to be filled
  - `"watch"` - Orders being monitored (conditional orders)
  - `"done"` - Completed orders (filled or canceled)
  - `"cancel"` - Canceled orders
- `states[]` (optional, array): Filter by multiple states
- `page` (optional, integer): Page number (default: 1)
- `limit` (optional, integer): Results per page (default: 100, max: 100)
- `order_by` (optional, string): Sort order ("asc" or "desc", default: "desc")

**Response**: Array of order objects (same structure as Create Order response)

**Notes**:
- Returns paginated results
- Can filter by UUIDs, identifiers, market, or state
- Maximum 100 orders per request

---

#### 6. Cancel Order

- **Endpoint**: `DELETE /v1/orders/{order-id}`
- **Full URL**: `https://sg-api.upbit.com/v1/orders/a1b2c3d4-e5f6-7890-abcd-ef1234567890`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "Make Orders"
- **HTTP Method**: DELETE

**Path Parameters**:
- `order-id` (string): Order UUID to cancel

**Query Parameters**:
- `uuid` (optional, string): Order UUID (alternative to path parameter)
- `identifier` (optional, string): Custom order identifier

**Response**: Canceled order object with `state: "cancel"`

**Notes**:
- Only orders in "wait" or "watch" state can be canceled
- Returns updated order details after cancellation

---

#### 7. Batch Cancel Orders

- **Endpoint**: `DELETE /v1/orders`
- **Full URL**: `https://sg-api.upbit.com/v1/orders?market=SGD-BTC`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 1 request/2 seconds (Exchange: order-cancel-all group)
- **Permission**: "Make Orders"
- **HTTP Method**: DELETE

**Query Parameters**:
- `market` (optional, string): Cancel all orders for specified market
- `side` (optional, string): Cancel orders by side ("bid" or "ask")
- `uuids[]` (optional, array): Cancel specific orders by UUID
- `identifiers[]` (optional, array): Cancel specific orders by identifier

**Response**: Array of canceled order objects

**Notes**:
- Strict rate limit: 1 request per 2 seconds
- Can cancel all orders or filter by market/side
- Maximum 100 orders can be canceled per request

---

## Account Endpoints (Exchange API)

### Balances

#### Get Account Balances

- **Endpoint**: `GET /v1/balances`
- **Full URL**: `https://sg-api.upbit.com/v1/balances`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **HTTP Method**: GET
- **Description**: Retrieve all asset balances in account

**Query Parameters**: None

**Response Fields**:
- `currency` (string): Currency code (e.g., "BTC", "SGD")
- `balance` (string): Total balance
- `locked` (string): Balance locked in orders or withdrawals
- `avg_buy_price` (string): Average purchase price
- `avg_buy_price_modified` (boolean): Whether average price was manually modified
- `unit_currency` (string): Unit currency for valuation (e.g., "SGD")

**Example Response**:
```json
[
  {
    "currency": "SGD",
    "balance": "1000000.0",
    "locked": "0.0",
    "avg_buy_price": "0",
    "avg_buy_price_modified": false,
    "unit_currency": "SGD"
  },
  {
    "currency": "BTC",
    "balance": "2.0",
    "locked": "0.1",
    "avg_buy_price": "67000",
    "avg_buy_price_modified": false,
    "unit_currency": "SGD"
  }
]
```

**Notes**:
- All balance values are strings (not numbers)
- `locked` represents funds in open orders or pending withdrawals
- Available balance = `balance` - `locked`

---

### Deposits

#### 1. Get Deposit Information

- **Endpoint**: `GET /v1/deposits/info`
- **Full URL**: `https://sg-api.upbit.com/v1/deposits/info?currency=BTC`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"

**Query Parameters**:
- `currency` (required, string): Currency code

---

#### 2. List Deposit Addresses

- **Endpoint**: `GET /v1/deposits/addresses`
- **Full URL**: `https://sg-api.upbit.com/v1/deposits/addresses`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **Description**: Get deposit addresses for all currencies

**Query Parameters**: None

---

#### 3. Create Deposit Address

- **Endpoint**: `POST /v1/deposits/addresses`
- **Full URL**: `https://sg-api.upbit.com/v1/deposits/addresses`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "Make Deposits/Withdrawals"
- **HTTP Method**: POST

**Request Body**:
- `currency` (required, string): Currency code

---

#### 4. List Deposits

- **Endpoint**: `GET /v1/deposits`
- **Full URL**: `https://sg-api.upbit.com/v1/deposits?currency=BTC`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **Description**: Retrieve deposit history

**Query Parameters**:
- `currency` (optional, string): Filter by currency
- `state` (optional, string): Filter by deposit state
- `uuids[]` (optional, array): Filter by deposit UUIDs
- `txids[]` (optional, array): Filter by transaction IDs
- `limit` (optional, integer): Results per page (max: 100)
- `page` (optional, integer): Page number
- `order_by` (optional, string): Sort order ("asc" or "desc")

---

### Withdrawals

#### 1. Get Withdrawal Information

- **Endpoint**: `GET /v1/withdrawals/info`
- **Full URL**: `https://sg-api.upbit.com/v1/withdrawals/info?currency=BTC`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"

**Query Parameters**:
- `currency` (required, string): Currency code

---

#### 2. List Withdrawal Addresses

- **Endpoint**: `GET /v1/withdrawals/addresses`
- **Full URL**: `https://sg-api.upbit.com/v1/withdrawals/addresses`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **Description**: Get registered withdrawal addresses

**Query Parameters**: None

---

#### 3. Initiate Withdrawal

- **Endpoint**: `POST /v1/withdrawals`
- **Full URL**: `https://sg-api.upbit.com/v1/withdrawals`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "Make Deposits/Withdrawals"
- **HTTP Method**: POST

**Request Body**:
- `currency` (required, string): Currency code
- `amount` (required, string): Withdrawal amount
- `address` (required, string): Withdrawal address
- `secondary_address` (optional, string): Secondary address (e.g., memo, tag)
- `transaction_type` (optional, string): Transaction type ("default" or "internal")

---

#### 4. List Withdrawals

- **Endpoint**: `GET /v1/withdrawals`
- **Full URL**: `https://sg-api.upbit.com/v1/withdrawals?currency=BTC`
- **Auth Required**: Yes (Private)
- **Rate Limit**: 30 requests/second (Exchange: default group)
- **Permission**: "View Account"
- **Description**: Retrieve withdrawal history

**Query Parameters**:
- `currency` (optional, string): Filter by currency
- `state` (optional, string): Filter by withdrawal state
- `uuids[]` (optional, array): Filter by withdrawal UUIDs
- `txids[]` (optional, array): Filter by transaction IDs
- `limit` (optional, integer): Results per page (max: 100)
- `page` (optional, integer): Page number
- `order_by` (optional, string): Sort order ("asc" or "desc")

---

## Summary

### Key Findings

1. **Regional Architecture**: Upbit operates separate regional exchanges (Singapore, Indonesia, Thailand) with distinct base URLs
2. **Endpoint Structure**: Clear separation between Quotation API (public) and Exchange API (private)
3. **Candle Endpoints**: Multiple endpoints for different intervals rather than single unified endpoint
4. **Symbol Format**: `{QUOTE}-{BASE}` format (e.g., "SGD-BTC", reversed from most exchanges)
5. **Numeric Format**: Response fields use numbers (not strings) for prices and volumes
6. **Timestamps**: All timestamps in milliseconds since Unix epoch
7. **Authentication**: JWT Bearer token for all private endpoints
8. **Rate Limiting**: Separate rate limit groups with per-second enforcement

### Implementation Notes

- All endpoints use `application/json` content type
- TLS 1.2 minimum, TLS 1.3 recommended
- POST requests require JSON body (form-based requests deprecated)
- Query parameters must be URL-encoded (except array brackets `[]`)
- Gzip compression supported via `Accept-Encoding: gzip` header

---

## Sources

Research compiled from official Upbit documentation:

- [Upbit Open API - REST API Guide](https://global-docs.upbit.com/reference/rest-api-guide)
- [Upbit Open API - Minute Candles](https://global-docs.upbit.com/v1.2.2/reference/minutes)
- [Upbit Open API - Market Ask Order Creation](https://global-docs.upbit.com/docs/market-ask-order-creation)
- [Tardis.dev - Upbit Historical Data](https://docs.tardis.dev/historical-data-details/upbit)
- [CCXT - Upbit Integration](https://github.com/ccxt/ccxt/blob/master/python/ccxt/upbit.py)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
