# Upbit API Response Formats - Complete Reference

Research conducted: 2026-01-20

## Table of Contents

1. [General Response Structure](#general-response-structure)
2. [Ticker Response](#ticker-response)
3. [Orderbook Response](#orderbook-response)
4. [Candles/Klines Response](#candlesklines-response)
5. [Recent Trades Response](#recent-trades-response)
6. [Order Response](#order-response)
7. [Balance Response](#balance-response)
8. [Trading Pairs Response](#trading-pairs-response)

---

## General Response Structure

### Success Response

Upbit API returns JSON arrays or objects directly (no wrapper).

**Format** (for arrays):
```json
[
  {
    // data here
  }
]
```

**Format** (for objects):
```json
{
  // data here
}
```

### Error Response

**Format**:
```json
{
  "error": {
    "name": "error_code",
    "message": "error description"
  }
}
```

**Key Points**:
- Success responses have no `error` field
- Error responses always include `error` object with `name` and `message`
- HTTP status codes indicate success (2xx) or failure (4xx, 5xx)

### Timestamps

- All timestamps are in **milliseconds** (Unix epoch UTC)
- Date/time strings use **ISO 8601** format: `"2024-06-19T08:31:43+00:00"`
- Korean time (KST) fields also provided: `"2024-06-19T17:31:43+09:00"`

---

## Ticker Response

### Endpoint
`GET /v1/tickers?markets=SGD-BTC`

### Response Format

**Returns**: Array of ticker objects

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

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market identifier (e.g., "SGD-BTC") |
| `trade_date` | string | Recent trade date in UTC (YYYYMMDD) |
| `trade_time` | string | Recent trade time in UTC (HHmmss) |
| `trade_date_kst` | string | Trade date in KST (YYYYMMDD) |
| `trade_time_kst` | string | Trade time in KST (HHmmss) |
| `trade_timestamp` | number | Trade timestamp (milliseconds) |
| `opening_price` | number | Opening price at 00:00:00 UTC |
| `high_price` | number | Highest price in last 24 hours |
| `low_price` | number | Lowest price in last 24 hours |
| `trade_price` | number | Most recent trade price (last price) |
| `prev_closing_price` | number | Previous day closing price (UTC 00:00:00) |
| `change` | string | Price change type: "RISE", "EVEN", or "FALL" |
| `change_price` | number | Absolute price change from previous close |
| `change_rate` | number | Price change rate (decimal, e.g., 0.0197 = 1.97%) |
| `signed_change_price` | number | Signed price change (+ for rise, - for fall) |
| `signed_change_rate` | number | Signed change rate (+ for rise, - for fall) |
| `trade_volume` | number | Volume of most recent trade |
| `acc_trade_price` | number | Accumulated trade value since UTC 00:00:00 (quote currency) |
| `acc_trade_price_24h` | number | Last 24h accumulated trade value |
| `acc_trade_volume` | number | Accumulated trade volume since UTC 00:00:00 (base currency) |
| `acc_trade_volume_24h` | number | Last 24h accumulated trade volume |
| `highest_52_week_price` | number | 52-week high price |
| `highest_52_week_date` | string | Date of 52-week high (YYYY-MM-DD) |
| `lowest_52_week_price` | number | 52-week low price |
| `lowest_52_week_date` | string | Date of 52-week low (YYYY-MM-DD) |
| `timestamp` | number | Response generation timestamp (milliseconds) |

### Notes

- All price/volume values are **numbers** (not strings)
- `change` field: "RISE" (price up), "EVEN" (no change), "FALL" (price down)
- `change_rate` is a decimal (multiply by 100 for percentage)
- Timestamps in milliseconds
- Multiple tickers can be requested in single call (comma-separated markets)

---

## Orderbook Response

### Endpoint
`GET /v1/orderbooks?markets=SGD-BTC`

### Response Format

**Returns**: Array of orderbook objects

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

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market identifier |
| `timestamp` | number | Orderbook snapshot timestamp (milliseconds) |
| `total_ask_size` | number | Total ask volume across all levels |
| `total_bid_size` | number | Total bid volume across all levels |
| `orderbook_units` | array | Array of price level objects (up to 30 levels) |

### Orderbook Unit Fields

| Field | Type | Description |
|-------|------|-------------|
| `ask_price` | number | Ask (sell) price at this level |
| `bid_price` | number | Bid (buy) price at this level |
| `ask_size` | number | Ask (sell) volume at this level |
| `bid_size` | number | Bid (buy) volume at this level |

### Notes

- Maximum 30 price levels per request
- Orderbook units sorted by best price first
- `ask_price` ascending (lowest ask first)
- `bid_price` descending (highest bid first)
- All numeric values are numbers (not strings)

---

## Candles/Klines Response

### Endpoint
`GET /v1/candles/minutes/1?market=SGD-BTC&count=200`

### Response Format

**Returns**: Array of candle objects (sorted descending by time - newest first)

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
  },
  {
    "market": "SGD-BTC",
    "candle_date_time_utc": "2024-06-19T08:30:00",
    "candle_date_time_kst": "2024-06-19T17:30:00",
    "opening_price": 66950.0,
    "high_price": 67100.0,
    "low_price": 66900.0,
    "trade_price": 67000.0,
    "timestamp": 1718788200000,
    "candle_acc_trade_price": 987654.32,
    "candle_acc_trade_volume": 14.73,
    "unit": 1
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market identifier |
| `candle_date_time_utc` | string | Candle start time in UTC (ISO 8601) |
| `candle_date_time_kst` | string | Candle start time in KST (ISO 8601) |
| `opening_price` | number | Opening price (first trade price in interval) |
| `high_price` | number | Highest price in interval |
| `low_price` | number | Lowest price in interval |
| `trade_price` | number | Closing price (last trade price in interval) |
| `timestamp` | number | Timestamp of last trade in candle (milliseconds) |
| `candle_acc_trade_price` | number | Accumulated trade value (quote currency) |
| `candle_acc_trade_volume` | number | Accumulated trade volume (base currency) |
| `unit` | number | Candle interval in minutes |

### Notes

- Sorted **descending** by time (newest candle first)
- All price/volume values are numbers (not strings)
- Maximum 200 candles per request
- `trade_price` is the closing price
- `timestamp` reflects the last trade time within the candle
- Available for: seconds, 1-240 minutes, days, weeks, months, years

---

## Recent Trades Response

### Endpoint
`GET /v1/trades/recent?market=SGD-BTC&count=100`

### Response Format

**Returns**: Array of trade objects (sorted descending by time - newest first)

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
  },
  {
    "market": "SGD-BTC",
    "trade_date_utc": "2024-06-19",
    "trade_time_utc": "08:31:40",
    "timestamp": 1718788300000,
    "trade_price": 67280.0,
    "trade_volume": 0.08,
    "prev_closing_price": 66000.0,
    "change_price": 1280.0,
    "ask_bid": "ASK",
    "sequential_id": 1234567890122
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market identifier |
| `trade_date_utc` | string | Trade execution date in UTC (YYYY-MM-DD) |
| `trade_time_utc` | string | Trade execution time in UTC (HH:mm:ss) |
| `timestamp` | number | Trade timestamp (milliseconds) |
| `trade_price` | number | Execution price |
| `trade_volume` | number | Execution volume (base currency) |
| `prev_closing_price` | number | Previous day closing price (UTC 00:00:00) |
| `change_price` | number | Price change from previous close |
| `ask_bid` | string | Trade side: "ASK" (sell) or "BID" (buy) |
| `sequential_id` | number | Sequential trade ID for pagination |

### Notes

- Sorted **descending** by timestamp (newest trade first)
- `ask_bid` indicates taker side: "ASK" = taker sold, "BID" = taker bought
- `sequential_id` can be used for pagination with `to` parameter
- Maximum 500 trades per request
- Maximum 7 days historical data

---

## Order Response

### Endpoint
`POST /v1/orders` (Create Order)
`GET /v1/orders/{order-id}` (Get Order)
`GET /v1/orders` (List Orders)

### Response Format

**Single Order** (Create/Get):
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
  "remaining_volume": "0.05",
  "reserved_fee": "0.5",
  "remaining_fee": "0.25",
  "paid_fee": "0.25",
  "locked": "3350.25",
  "executed_volume": "0.05",
  "trades_count": 2,
  "trades": [
    {
      "market": "SGD-BTC",
      "uuid": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
      "price": "67000.0",
      "volume": "0.03",
      "funds": "2010.0",
      "side": "bid",
      "created_at": "2024-06-19T08:31:45+00:00"
    },
    {
      "market": "SGD-BTC",
      "uuid": "c3d4e5f6-a7b8-9012-cdef-123456789012",
      "price": "67100.0",
      "volume": "0.02",
      "funds": "1342.0",
      "side": "bid",
      "created_at": "2024-06-19T08:31:50+00:00"
    }
  ]
}
```

**List Orders** (Array):
```json
[
  {
    "uuid": "...",
    "side": "bid",
    // ... same fields as single order
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | string | Order UUID |
| `side` | string | Order side: "bid" (buy) or "ask" (sell) |
| `ord_type` | string | Order type: "limit", "price" (market buy), "market" (market sell) |
| `price` | string | Order price (limit orders) or total amount (market buy) |
| `state` | string | Order state: "wait", "watch", "done", "cancel" |
| `market` | string | Market identifier |
| `created_at` | string | Order creation time (ISO 8601) |
| `volume` | string | Order volume (base currency) |
| `remaining_volume` | string | Remaining unfilled volume |
| `reserved_fee` | string | Reserved fee amount |
| `remaining_fee` | string | Remaining fee to be paid |
| `paid_fee` | string | Fee already paid |
| `locked` | string | Locked amount (price × volume + fee for buy, volume for sell) |
| `executed_volume` | string | Executed volume |
| `trades_count` | number | Number of trades for this order |
| `trades` | array | Array of trade executions (if order is partially/fully filled) |

### Order States

| State | Description |
|-------|-------------|
| `wait` | Order waiting to be filled (active in orderbook) |
| `watch` | Order being monitored (conditional orders) |
| `done` | Order completed (fully filled or canceled) |
| `cancel` | Order canceled |

### Trade Execution Fields

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market identifier |
| `uuid` | string | Trade UUID |
| `price` | string | Execution price |
| `volume` | string | Execution volume |
| `funds` | string | Execution value (price × volume) |
| `side` | string | Order side |
| `created_at` | string | Execution time (ISO 8601) |

### Notes

- All price/volume/fee values are **strings** (for precision)
- `state: "wait"` = active order
- `state: "done"` = completed (check `remaining_volume` to determine if filled or canceled)
- `trades` array populated when order has executions
- Average execution price = total `funds` / total `executed_volume`

---

## Balance Response

### Endpoint
`GET /v1/balances`

### Response Format

**Returns**: Array of balance objects

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
  },
  {
    "currency": "ETH",
    "balance": "15.5",
    "locked": "2.0",
    "avg_buy_price": "3500",
    "avg_buy_price_modified": false,
    "unit_currency": "SGD"
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency code (e.g., "BTC", "SGD", "ETH") |
| `balance` | string | Total balance for this currency |
| `locked` | string | Balance locked in orders or withdrawals |
| `avg_buy_price` | string | Average purchase price in unit currency |
| `avg_buy_price_modified` | boolean | Whether average price was manually modified |
| `unit_currency` | string | Unit currency for valuation (e.g., "SGD") |

### Notes

- All balance values are **strings** (for precision)
- Available balance = `balance` - `locked`
- Only currencies with non-zero balance are returned
- `avg_buy_price` tracked for portfolio valuation
- Fiat currencies (SGD, USD, etc.) typically have `avg_buy_price: "0"`

---

## Trading Pairs Response

### Endpoint
`GET /v1/trading-pairs`

### Response Format

**Returns**: Array of market identifier strings

```json
[
  "SGD-BTC",
  "SGD-ETH",
  "SGD-XRP",
  "BTC-ETH",
  "BTC-XRP",
  "USDT-BTC",
  "USDT-ETH"
]
```

### Notes

- Simple array of strings
- Format: `{QUOTE}-{BASE}` (e.g., "SGD-BTC" = BTC priced in SGD)
- No additional metadata in this endpoint
- Use `/v1/tickers` or `/v1/orderbooks` for detailed market information

---

## Summary

### Data Type Patterns

| Response Type | Price/Volume Format | Timestamp Format |
|---------------|---------------------|------------------|
| Ticker | numbers | milliseconds |
| Orderbook | numbers | milliseconds |
| Candles | numbers | milliseconds |
| Trades | numbers | milliseconds |
| Orders | **strings** | ISO 8601 string |
| Balances | **strings** | N/A |

### Key Observations

1. **Ticker/Orderbook/Candles/Trades**: Use numeric values for prices and volumes
2. **Orders/Balances**: Use string values for prices and volumes (precision)
3. **Timestamps**: Milliseconds for market data, ISO 8601 strings for account data
4. **No Wrapper**: Responses are direct arrays or objects (no `{ "data": ... }` wrapper)
5. **Error Format**: `{ "error": { "name": "...", "message": "..." } }`
6. **Numeric Precision**: Numbers safe for JavaScript/JSON (no overflow)
7. **String Precision**: Critical financial values (orders, balances) use strings

---

## Sources

- [Upbit Open API - REST API Guide](https://global-docs.upbit.com/reference/rest-api-guide)
- [Upbit Open API - WebSocket Orderbook](https://global-docs.upbit.com/v1.2.2/reference/websocket-orderbook)
- [Upbit Open API - Minutes Candles](https://global-docs.upbit.com/v1.2.2/reference/minutes)
- [CCXT - Upbit Implementation](https://github.com/ccxt/ccxt/blob/master/python/ccxt/upbit.py)
- [Tardis.dev - Upbit Data](https://docs.tardis.dev/historical-data-details/upbit)
- [GitHub - Upbit Python Client](https://github.com/sharebook-kr/pyupbit)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
