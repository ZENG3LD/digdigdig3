# Coinbase Advanced Trade API Response Formats - Complete Reference

Research conducted: 2026-01-20

## Table of Contents

1. [General Response Structure](#general-response-structure)
2. [Price Response (Best Bid/Ask)](#price-response-best-bidask)
3. [Klines/Candles Response](#klinescandles-response)
4. [Orderbook Response](#orderbook-response)
5. [Ticker (Market Trades) Response](#ticker-market-trades-response)
6. [Order Response](#order-response)
7. [Balance Response](#balance-response)
8. [Transaction Summary Response](#transaction-summary-response)

---

## General Response Structure

### Success Response

**Format:**
Unlike KuCoin which wraps responses in `{code, data}`, Coinbase returns direct JSON objects:

```json
{
  "field1": "value1",
  "field2": "value2"
}
```

**Key Points:**
- **No wrapper object** - responses are direct JSON
- **HTTP 200** for success
- **4xx/5xx** for errors
- **No success code field** like KuCoin's `"code": "200000"`

### Error Response

**Format:**
```json
{
  "error": "error_type",
  "message": "Human-readable error description",
  "error_details": "Additional error context"
}
```

**Common Error Types:**
- `invalid_signature` - Authentication failed
- `invalid_api_key` - API key not found
- `token_expired` - JWT expired
- `rate_limit_exceeded` - Too many requests
- `insufficient_funds` - Not enough balance
- `invalid_product_id` - Unknown trading pair

### Timestamps

- **Format**: RFC3339 (e.g., `"2023-10-26T10:05:30.123Z"`)
- **Alternative**: Unix seconds as string (e.g., `"1698315930"`)
- **Milliseconds**: Some fields use Unix milliseconds as string

---

## Price Response (Best Bid/Ask)

### Endpoint
`GET /api/v3/brokerage/best_bid_ask?product_ids=BTC-USD`

### Response Format

```json
{
  "pricebooks": [
    {
      "product_id": "BTC-USD",
      "bids": [
        {
          "price": "49999.99",
          "size": "0.75"
        }
      ],
      "asks": [
        {
          "price": "50000.01",
          "size": "0.50"
        }
      ],
      "time": "2023-10-26T10:05:30.123456Z"
    }
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `product_id` | string | Trading pair |
| `bids` | array | Best bid price/size |
| `bids[].price` | string | Bid price |
| `bids[].size` | string | Bid size |
| `asks` | array | Best ask price/size |
| `asks[].price` | string | Ask price |
| `asks[].size` | string | Ask size |
| `time` | string | Timestamp (RFC3339) |

### Notes
- All price/size fields are **strings** (need parsing)
- `time` uses RFC3339 format with microsecond precision
- Single best bid/ask only (not full orderbook)

---

## Klines/Candles Response

### Endpoint
`GET /api/v3/brokerage/products/BTC-USD/candles?start=1609459200&end=1609545600&granularity=ONE_HOUR`

### Response Format

```json
{
  "candles": [
    {
      "start": "1639508050",
      "low": "48000.00",
      "high": "49000.00",
      "open": "48500.00",
      "close": "48800.00",
      "volume": "123.45"
    },
    {
      "start": "1639504450",
      "low": "47500.00",
      "high": "48500.00",
      "open": "48000.00",
      "close": "48500.00",
      "volume": "105.32"
    }
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `start` | string | Start time (Unix seconds) |
| `low` | string | Lowest price |
| `high` | string | Highest price |
| `open` | string | Opening price |
| `close` | string | Closing price |
| `volume` | string | Trading volume (base currency) |

### Data Ordering

- **Returns up to 300 candles per request**
- **Sort order**: **Descending by time** (newest first)
- **Note**: Must reverse for ascending time order

### Notes
- Time is in **Unix seconds** as string (not milliseconds)
- All numeric fields are **strings**
- No turnover/quote volume field (only base volume)
- Order: `[start, low, high, open, close, volume]` in object format (not array like KuCoin)

**Comparison with KuCoin:**
- KuCoin uses arrays: `[time, open, close, high, low, volume, turnover]`
- Coinbase uses objects with named fields
- KuCoin time in seconds (needs * 1000), Coinbase also in seconds
- KuCoin has `turnover` (quote volume), Coinbase only has `volume` (base)

---

## Orderbook Response

### Endpoint
`GET /api/v3/brokerage/product_book?product_id=BTC-USD&limit=10`

### Response Format

```json
{
  "pricebook": {
    "product_id": "BTC-USD",
    "bids": [
      {
        "price": "49999.99",
        "size": "0.75"
      },
      {
        "price": "49999.98",
        "size": "1.20"
      }
    ],
    "asks": [
      {
        "price": "50000.01",
        "size": "0.50"
      },
      {
        "price": "50000.02",
        "size": "1.00"
      }
    ],
    "time": "2023-10-26T10:05:30.123Z"
  }
}
```

### Alternative Format (with num_orders)

Some implementations show:
```json
{
  "bids": [
    ["49999.99", "0.75", 3]
  ],
  "asks": [
    ["50000.01", "0.50", 2]
  ]
}
```
Where array format is `[price, size, num_orders]`

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `product_id` | string | Trading pair |
| `bids` | array | Bid orders (buy side) |
| `bids[].price` | string | Bid price |
| `bids[].size` | string | Total size at price |
| `asks` | array | Ask orders (sell side) |
| `asks[].price` | string | Ask price |
| `asks[].size` | string | Total size at price |
| `time` | string | Timestamp (RFC3339) |

### Ordering

- **Bids:** Sorted high to low (best bid first)
- **Asks:** Sorted low to high (best ask first)

### Notes
- Level 2 aggregates all orders at each price level
- Max 500 levels per side (with `limit=500`)
- Default limit is 50 levels
- No sequence numbers like KuCoin (use WebSocket for synced orderbook)

**Comparison with KuCoin:**
- KuCoin has `sequence` field for synchronization
- KuCoin uses nested `data` object wrapper
- Coinbase simpler structure, no sequence tracking in REST

---

## Ticker (Market Trades) Response

### Endpoint
`GET /api/v3/brokerage/products/BTC-USD/ticker?limit=100`

### Response Format

```json
{
  "trades": [
    {
      "trade_id": "12345678",
      "product_id": "BTC-USD",
      "price": "50000.00",
      "size": "0.01",
      "time": "2023-10-26T10:05:30.123456Z",
      "side": "BUY",
      "bid": "49999.99",
      "ask": "50000.01"
    }
  ],
  "best_bid": "49999.99",
  "best_ask": "50000.01"
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `trade_id` | string | Trade ID |
| `product_id` | string | Trading pair |
| `price` | string | Trade price |
| `size` | string | Trade size |
| `time` | string | Trade time (RFC3339) |
| `side` | string | Trade side ("BUY" or "SELL") |
| `bid` | string | Best bid at time of trade |
| `ask` | string | Best ask at time of trade |
| `best_bid` | string | Current best bid |
| `best_ask` | string | Current best ask |

### Notes
- Returns recent market trades
- **NOT** 24h ticker stats (different from KuCoin's ticker endpoint)
- For 24h stats, use product details endpoint
- Limit: 1-1000 trades

**Comparison with KuCoin:**
- KuCoin `/api/v1/market/stats` returns 24h stats (high, low, vol, change)
- Coinbase ticker is recent trades, not 24h aggregation
- Different semantic meaning of "ticker"

---

## Order Response

### Create Order Response

**Endpoint:** `POST /api/v3/brokerage/orders`

**Response:**
```json
{
  "success": true,
  "success_response": {
    "order_id": "11111-00000-000000",
    "product_id": "BTC-USD",
    "side": "BUY",
    "client_order_id": "0000-00000-000000"
  },
  "failure_response": {
    "error": "INVALID_ORDER_CONFIG",
    "message": "Invalid order configuration",
    "error_details": "quote_size must be positive"
  },
  "order_configuration": {
    "market_market_ioc": {
      "quote_size": "1000.00"
    }
  }
}
```

**Response can contain EITHER `success_response` OR `failure_response`.**

### Get Order Details Response

**Endpoint:** `GET /api/v3/brokerage/orders/historical/{order_id}`

**Response:**
```json
{
  "order": {
    "order_id": "11111-00000-000000",
    "product_id": "BTC-USD",
    "user_id": "user-uuid",
    "order_configuration": {
      "limit_limit_gtc": {
        "base_size": "0.01",
        "limit_price": "50000.00",
        "post_only": false
      }
    },
    "side": "BUY",
    "client_order_id": "0000-00000-000000",
    "status": "OPEN",
    "time_in_force": "GOOD_UNTIL_CANCELLED",
    "created_time": "2023-10-26T10:05:30.123456Z",
    "completion_percentage": "50",
    "filled_size": "0.005",
    "average_filled_price": "49950.00",
    "fee": "0.50",
    "number_of_fills": "2",
    "filled_value": "249.75",
    "pending_cancel": false,
    "size_in_quote": false,
    "total_fees": "0.50",
    "size_inclusive_of_fees": false,
    "total_value_after_fees": "250.25",
    "trigger_status": "UNKNOWN_TRIGGER_STATUS",
    "order_type": "LIMIT",
    "reject_reason": "REJECT_REASON_UNSPECIFIED",
    "settled": false,
    "product_type": "SPOT",
    "reject_message": "",
    "cancel_message": ""
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `order_id` | string | Server-assigned order ID |
| `client_order_id` | string | Client-provided order ID |
| `product_id` | string | Trading pair |
| `side` | string | "BUY" or "SELL" |
| `status` | string | Order status |
| `filled_size` | string | Filled quantity |
| `average_filled_price` | string | Average fill price |
| `fee` | string | Trading fee for this fill |
| `total_fees` | string | Total fees for order |
| `filled_value` | string | Total filled value |
| `created_time` | string | Creation time (RFC3339) |
| `completion_percentage` | string | Fill percentage (0-100) |
| `number_of_fills` | string | Number of fills |
| `time_in_force` | string | Time in force |
| `order_type` | string | Order type |

### Order Status Values

| Status | Description |
|--------|-------------|
| `OPEN` | Order is active |
| `FILLED` | Order completely filled |
| `CANCELLED` | Order cancelled |
| `EXPIRED` | Order expired |
| `FAILED` | Order failed |
| `UNKNOWN_ORDER_STATUS` | Status unknown |
| `QUEUED` | Order queued |
| `CANCEL_QUEUED` | Cancellation queued |

**Comparison with KuCoin:**
- KuCoin uses `isActive`, `cancelExist` boolean flags
- Coinbase uses explicit status strings
- Coinbase has more granular status values

### List Orders Response

**Endpoint:** `GET /api/v3/brokerage/orders/historical/batch`

**Response:**
```json
{
  "orders": [
    { /* order object */ },
    { /* order object */ }
  ],
  "sequence": "0",
  "has_next": true,
  "cursor": "next_page_cursor"
}
```

**Pagination Fields:**
- `has_next`: Boolean indicating more pages
- `cursor`: Cursor for next page
- `sequence`: Sequence number (optional)

---

## Balance Response

### List Accounts Response

**Endpoint:** `GET /api/v3/brokerage/accounts`

**Response:**
```json
{
  "accounts": [
    {
      "uuid": "8bfc20d7-f7c6-4422-bf07-8243ca4169fe",
      "name": "BTC Wallet",
      "currency": "BTC",
      "available_balance": {
        "value": "1.23456789",
        "currency": "BTC"
      },
      "default": true,
      "active": true,
      "created_at": "2021-05-31T09:59:59Z",
      "updated_at": "2021-05-31T09:59:59Z",
      "deleted_at": null,
      "type": "ACCOUNT_TYPE_CRYPTO",
      "ready": true,
      "hold": {
        "value": "0.1",
        "currency": "BTC"
      }
    }
  ],
  "has_next": false,
  "cursor": "",
  "size": 1
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | string | Account ID |
| `name` | string | Account name |
| `currency` | string | Asset/coin symbol |
| `available_balance` | object | Available balance |
| `available_balance.value` | string | Balance amount |
| `available_balance.currency` | string | Currency code |
| `hold` | object | Frozen/locked balance |
| `hold.value` | string | Held amount |
| `hold.currency` | string | Currency code |
| `type` | string | Account type |
| `active` | boolean | Account active |
| `ready` | boolean | Account ready |
| `created_at` | string | Creation time (RFC3339) |

### Balance Calculation
```
total_balance = available_balance.value + hold.value
```

**Comparison with KuCoin:**
- KuCoin: `balance`, `available`, `holds` as simple strings
- Coinbase: Nested objects with `value` and `currency` fields
- KuCoin simpler structure
- Coinbase more verbose but type-safe

---

## Transaction Summary Response

### Endpoint
`GET /api/v3/brokerage/transaction_summary`

### Response Format

```json
{
  "total_volume": 10000.50,
  "total_fees": 15.25,
  "fee_tier": {
    "pricing_tier": "Advanced <$10K",
    "usd_from": "0",
    "usd_to": "10000",
    "taker_fee_rate": "0.006",
    "maker_fee_rate": "0.004",
    "aop_from": "0",
    "aop_to": "10000"
  },
  "margin_rate": {
    "value": "0.5"
  },
  "goods_and_services_tax": {
    "rate": "0.1",
    "type": "INCLUSIVE"
  },
  "advanced_trade_only_volume": 10000.50,
  "advanced_trade_only_fees": 15.25,
  "coinbase_pro_volume": 0,
  "coinbase_pro_fees": 0,
  "total_balance": "12345.67",
  "has_promo_fee": false
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `total_volume` | number | Total trading volume (30-day) |
| `total_fees` | number | Total fees paid (30-day) |
| `fee_tier` | object | Current fee tier |
| `fee_tier.pricing_tier` | string | Tier name |
| `fee_tier.taker_fee_rate` | string | Taker fee rate (decimal) |
| `fee_tier.maker_fee_rate` | string | Maker fee rate (decimal) |
| `advanced_trade_only_volume` | number | Advanced Trade volume |
| `advanced_trade_only_fees` | number | Advanced Trade fees |

### Fee Tiers

Coinbase uses volume-based fee tiers:
- $0 - $10K: 0.60% taker / 0.40% maker
- $10K - $50K: 0.40% taker / 0.25% maker
- $50K - $100K: 0.25% taker / 0.15% maker
- Higher tiers available with more volume

---

## Summary of Key Differences from KuCoin

| Feature | KuCoin | Coinbase |
|---------|--------|----------|
| **Response Wrapper** | `{code: "200000", data: {...}}` | Direct object |
| **Timestamp Format** | Milliseconds | RFC3339 + Unix seconds |
| **Price/Size Types** | Strings | Strings (same) |
| **Kline Format** | Arrays | Objects with named fields |
| **Kline Order** | Ascending (oldest first) | Descending (newest first) |
| **Order Status** | Boolean flags (`isActive`, `cancelExist`) | Status strings (`OPEN`, `FILLED`) |
| **Balance Structure** | Simple strings | Nested objects |
| **Ticker Endpoint** | 24h stats (high, low, vol) | Recent trades |
| **Sequence Numbers** | Yes (for orderbook sync) | No (REST only) |

---

## Sources

- [Coinbase Advanced Trade API - Welcome](https://docs.cdp.coinbase.com/advanced-trade/docs/welcome)
- [Advanced Trade REST API Endpoints](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/rest-api)
- [Get Product Book](https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/products/get-product-candles)
- [Create Order](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
- [List Orders](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/list-orders)
- [Get Order](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/get-order)
- [Coinbase Advanced Python SDK](https://github.com/coinbase/coinbase-advanced-py)
- [Coinbase API Cheat Sheet](https://vezgo.com/blog/coinbase-api-cheat-sheet-for-developers/)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
