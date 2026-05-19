# Gate.io API v4 Response Formats

**Research Date**: 2026-01-21

---

## Table of Contents

- [General Response Structure](#general-response-structure)
- [Ticker Response](#ticker-response)
- [Orderbook Response](#orderbook-response)
- [Klines Response](#klines-response)
- [Balance Response](#balance-response)
- [Order Response](#order-response)
- [Position Response (Futures)](#position-response-futures)
- [Funding Rate Response](#funding-rate-response)

---

## General Response Structure

All Gate.io API responses follow a consistent format.

### Success Response

```json
{
  "currency_pair": "BTC_USDT",
  "last": "48600.5",
  "lowest_ask": "48601.0",
  "highest_bid": "48600.0",
  ...
}
```

OR for list endpoints:

```json
[
  {
    "currency_pair": "BTC_USDT",
    ...
  },
  {
    "currency_pair": "ETH_USDT",
    ...
  }
]
```

**Key Points**:
- No wrapper object for successful responses (unlike many exchanges)
- Response is **directly** the data (object or array)
- HTTP status 200 indicates success
- All numeric values are **strings** (need parsing)

### Error Response

```json
{
  "label": "INVALID_PARAM_VALUE",
  "message": "invalid currency_pair"
}
```

**Error Fields**:
- `label`: Error code (e.g., "INVALID_KEY", "INVALID_SIGNATURE")
- `message`: Human-readable error description

**HTTP Status Codes**:
- 200: Success
- 400: Bad request (invalid parameters)
- 401: Unauthorized (authentication failed)
- 403: Forbidden (permission denied)
- 404: Not found
- 429: Rate limit exceeded
- 500: Internal server error

---

## Ticker Response

### Endpoint: GET /spot/tickers

**Single Ticker** (with `currency_pair` parameter):
```json
[
  {
    "currency_pair": "BTC_USDT",
    "last": "48600.5",
    "lowest_ask": "48601.0",
    "highest_bid": "48600.0",
    "change_percentage": "2.5",
    "base_volume": "1234.567",
    "quote_volume": "60000000.00",
    "high_24h": "49000.0",
    "low_24h": "47500.0",
    "etf_net_value": null,
    "etf_pre_net_value": null,
    "etf_pre_timestamp": null,
    "etf_leverage": null
  }
]
```

**All Tickers** (without `currency_pair` parameter):
```json
[
  {
    "currency_pair": "BTC_USDT",
    ...
  },
  {
    "currency_pair": "ETH_USDT",
    ...
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `currency_pair` | string | Trading pair (e.g., "BTC_USDT") |
| `last` | string | Last traded price |
| `lowest_ask` | string | Best ask price (lowest sell price) |
| `highest_bid` | string | Best bid price (highest buy price) |
| `change_percentage` | string | 24h price change % (e.g., "2.5" = 2.5%) |
| `base_volume` | string | 24h trading volume (base currency) |
| `quote_volume` | string | 24h trading volume (quote currency) |
| `high_24h` | string | Highest price in last 24h |
| `low_24h` | string | Lowest price in last 24h |
| `etf_net_value` | string/null | ETF net value (ETF pairs only) |
| `etf_pre_net_value` | string/null | ETF previous net value |
| `etf_pre_timestamp` | integer/null | ETF previous timestamp |
| `etf_leverage` | string/null | ETF leverage |

**Notes**:
- Response is **always an array**, even for single ticker
- All prices/volumes are **strings**
- `change_percentage` is percentage value (not decimal, e.g., "2.5" not "0.025")

### Futures Ticker: GET /futures/{settle}/tickers

Additional fields for futures:
```json
{
  "contract": "BTC_USDT",
  "last": "48600.5",
  "mark_price": "48601.2",
  "index_price": "48599.8",
  "funding_rate": "0.0001",
  "funding_rate_indicative": "0.00012",
  "volume_24h": "123456789",
  "volume_24h_btc": "2545.5",
  "volume_24h_usd": "123700000",
  "volume_24h_base": "123456789",
  "volume_24h_quote": "60000000000",
  "volume_24h_settle": "60000000000",
  ...
}
```

| Field | Type | Description |
|-------|------|-------------|
| `contract` | string | Contract name (e.g., "BTC_USDT") |
| `mark_price` | string | Mark price (for liquidation) |
| `index_price` | string | Index price from spot markets |
| `funding_rate` | string | Current funding rate |
| `funding_rate_indicative` | string | Predicted next funding rate |
| `volume_24h_base` | string | 24h volume (base currency) |
| `volume_24h_quote` | string | 24h volume (quote currency) |
| `volume_24h_settle` | string | 24h volume (settlement currency) |

---

## Orderbook Response

### Endpoint: GET /spot/order_book

```json
{
  "id": 123456789,
  "current": 1623898993123,
  "update": 1623898993121,
  "asks": [
    ["48610.0", "0.5"],
    ["48615.0", "1.2"],
    ["48620.0", "0.8"]
  ],
  "bids": [
    ["48600.0", "0.8"],
    ["48595.0", "2.1"],
    ["48590.0", "1.5"]
  ]
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Order book update ID (optional, if `with_id=true`) |
| `current` | integer | Timestamp when response generated (milliseconds) |
| `update` | integer | Timestamp when order book last changed (milliseconds) |
| `asks` | array | Sell orders [[price, quantity], ...] |
| `bids` | array | Buy orders [[price, quantity], ...] |

**Ask/Bid Format**: `[price, quantity]`
- `[0]`: Price (string)
- `[1]`: Quantity (string)

**Sorting**:
- `asks`: Sorted low to high (best ask first)
- `bids`: Sorted high to low (best bid first)

**Futures Orderbook**: Same structure, but quantity is in **number of contracts**.

---

## Klines Response

### Endpoint: GET /spot/candlesticks

```json
[
  [
    "1566703320",
    "8533.02",
    "8553.74",
    "8550.24",
    "8527.17",
    "8553.74",
    "123.456"
  ],
  [
    "1566703260",
    "8533.02",
    "8553.74",
    "8550.24",
    "8527.17",
    "8553.74",
    "789.012"
  ]
]
```

### Array Structure

**CRITICAL**: Gate.io uses a **different** array order than most exchanges!

**Gate.io format**: `[time, volume, close, high, low, open, quote_volume]`

| Index | Field | Type | Description |
|-------|-------|------|-------------|
| 0 | time | string | Unix timestamp in **seconds** |
| 1 | volume | string | Trading volume (base currency) |
| 2 | close | string | Closing price |
| 3 | high | string | Highest price |
| 4 | low | string | Lowest price |
| 5 | open | string | Opening price |
| 6 | quote_volume | string | Quote currency volume (optional) |

**Notes**:
- Time is in **seconds**, not milliseconds (multiply by 1000 for ms)
- Most recent candle is **first** (descending order)
- All values are **strings**
- Different from typical OHLCV order!

**Comparison with typical exchange**:
- Typical: `[time, open, high, low, close, volume]`
- Gate.io: `[time, volume, close, high, low, open, quote_volume]`

---

## Balance Response

### Spot Balance: GET /spot/accounts

```json
[
  {
    "currency": "BTC",
    "available": "1.2345",
    "locked": "0.0100"
  },
  {
    "currency": "USDT",
    "available": "10000.50",
    "locked": "500.00"
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency code (e.g., "BTC", "USDT") |
| `available` | string | Available balance (free to trade) |
| `locked` | string | Balance locked in orders |

**Total Balance**: `total = parseFloat(available) + parseFloat(locked)`

### Futures Balance: GET /futures/{settle}/accounts

```json
{
  "user": 123456,
  "currency": "USDT",
  "total": "10000.5",
  "unrealised_pnl": "150.25",
  "position_margin": "500.0",
  "order_margin": "200.0",
  "available": "9150.25",
  "point": "0",
  "bonus": "0",
  "in_dual_mode": false,
  "enable_credit": true,
  "position_initial_margin": "0",
  "maintenance_margin": "250.0",
  "enable_evolved_classic": true,
  "history": {
    "dnw": "0",
    "pnl": "150.25",
    "fee": "5.50",
    "refr": "0",
    "fund": "2.10",
    "point_dnw": "0",
    "point_fee": "0",
    "point_refr": "0",
    "bonus_dnw": "0",
    "bonus_offset": "0"
  }
}
```

### Futures Balance Fields

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Settlement currency |
| `total` | string | Total account balance |
| `unrealised_pnl` | string | Unrealized PnL from all positions |
| `position_margin` | string | Margin locked in positions |
| `order_margin` | string | Margin locked in open orders |
| `available` | string | Available balance for trading |
| `maintenance_margin` | string | Required maintenance margin |
| `in_dual_mode` | boolean | Hedge mode enabled? |

---

## Order Response

### Create Order: POST /spot/orders

**Response**:
```json
{
  "id": "123456789",
  "text": "my-order-123",
  "create_time": "1729100692",
  "update_time": "1729100692",
  "create_time_ms": "1729100692123",
  "update_time_ms": "1729100692123",
  "currency_pair": "BTC_USDT",
  "status": "open",
  "type": "limit",
  "account": "spot",
  "side": "buy",
  "iceberg": "0",
  "amount": "0.01",
  "price": "48000",
  "time_in_force": "gtc",
  "left": "0.01",
  "filled_total": "0",
  "fee": "0",
  "fee_currency": "USDT",
  "point_fee": "0",
  "gt_fee": "0",
  "gt_discount": false,
  "rebated_fee": "0",
  "rebated_fee_currency": "USDT"
}
```

### Order Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Server-assigned order ID |
| `text` | string | User-defined label |
| `create_time` | string | Creation time (seconds) |
| `update_time` | string | Last update time (seconds) |
| `create_time_ms` | string | Creation time (milliseconds) |
| `update_time_ms` | string | Last update time (milliseconds) |
| `currency_pair` | string | Trading pair |
| `status` | string | Order status (see below) |
| `type` | string | Order type ("limit", "market") |
| `account` | string | Account type ("spot", "margin") |
| `side` | string | Order side ("buy", "sell") |
| `amount` | string | Order quantity |
| `price` | string | Order price |
| `time_in_force` | string | TIF ("gtc", "ioc", "poc", "fok") |
| `left` | string | Remaining unfilled quantity |
| `filled_total` | string | Total filled value (quote currency) |
| `fee` | string | Trading fee paid |
| `fee_currency` | string | Fee currency |

### Order Status Values

| Status | Description |
|--------|-------------|
| `open` | Active, waiting to be filled |
| `closed` | Fully filled |
| `cancelled` | Cancelled by user |

**Filled Quantity**: `filled_qty = amount - left`

**Average Fill Price**: `avg_price = filled_total / filled_qty`

### List Orders: GET /spot/orders

Returns **array** of order objects:
```json
[
  {
    "id": "123456789",
    "currency_pair": "BTC_USDT",
    "status": "open",
    ...
  },
  {
    "id": "987654321",
    "currency_pair": "ETH_USDT",
    "status": "closed",
    ...
  }
]
```

### Futures Order Response

Similar structure with additional fields:
```json
{
  "id": 123456789,
  "user": 123456,
  "create_time": 1729100692.123,
  "finish_time": 0,
  "finish_as": "",
  "status": "open",
  "contract": "BTC_USDT",
  "size": 10,
  "price": "48000",
  "fill_price": "0",
  "left": 10,
  "text": "my-order-123",
  "tkfr": "0.0006",
  "mkfr": "0.0002",
  "refu": 0,
  "is_reduce_only": false,
  "is_close": false,
  "is_liq": false,
  "tif": "gtc",
  "iceberg": 0
}
```

**Futures-specific fields**:
- `contract`: Contract name (instead of `currency_pair`)
- `size`: Number of contracts (integer, can be negative)
- `fill_price`: Average fill price
- `tkfr`: Taker fee rate
- `mkfr`: Maker fee rate
- `is_reduce_only`: Reduce-only flag
- `is_close`: Close position flag
- `is_liq`: Liquidation order flag

---

## Position Response (Futures)

### Get Positions: GET /futures/{settle}/positions

```json
[
  {
    "user": 123456,
    "contract": "BTC_USDT",
    "size": 10,
    "leverage": "10",
    "risk_limit": "1000000",
    "leverage_max": "100",
    "maintenance_rate": "0.005",
    "value": "4860.5",
    "margin": "486.05",
    "entry_price": "48600.5",
    "liq_price": "43740.45",
    "mark_price": "48605.2",
    "unrealised_pnl": "0.47",
    "realised_pnl": "-1.20",
    "history_pnl": "-1.20",
    "last_close_pnl": "0",
    "realised_point": "0",
    "history_point": "0",
    "adl_ranking": 3,
    "pending_orders": 2,
    "close_order": null,
    "mode": "single",
    "cross_leverage_limit": "0",
    "update_time": 1729100692
  }
]
```

### Position Fields

| Field | Type | Description |
|-------|------|-------------|
| `contract` | string | Contract symbol |
| `size` | integer | Position size (+ = long, - = short, 0 = no position) |
| `leverage` | string | Current leverage |
| `value` | string | Position value (quote currency) |
| `margin` | string | Position margin |
| `entry_price` | string | Average entry price |
| `liq_price` | string | Liquidation price |
| `mark_price` | string | Current mark price |
| `unrealised_pnl` | string | Unrealized profit/loss |
| `realised_pnl` | string | Realized PnL (current session) |
| `history_pnl` | string | Historical realized PnL |
| `adl_ranking` | integer | Auto-deleveraging rank (1-5, 5 = highest risk) |
| `pending_orders` | integer | Number of open orders |
| `mode` | string | Position mode ("single" or "dual" for hedge) |
| `update_time` | integer | Last update timestamp (seconds) |

**Position Side**:
- `size > 0`: Long position
- `size < 0`: Short position
- `size == 0`: No position

---

## Funding Rate Response

### Get Funding Rate: GET /futures/{settle}/funding_rate

```json
[
  {
    "t": 1729129200,
    "r": "0.0001"
  },
  {
    "t": 1729140800,
    "r": "0.00012"
  }
]
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `t` | integer | Funding time (Unix timestamp in **seconds**) |
| `r` | string | Funding rate (decimal, e.g., "0.0001" = 0.01%) |

**Notes**:
- Returns historical funding rates
- Sorted descending (newest first)
- Funding rate is applied every 8 hours (28800 seconds) typically
- Positive rate: Longs pay shorts
- Negative rate: Shorts pay longs

---

## Key Takeaways

### Data Types
- **All numeric values are strings** (prices, volumes, balances, etc.)
- **Timestamps vary**: seconds for most, milliseconds for orderbook
- **Booleans** are actual booleans (not strings)

### Array vs Object
- Tickers: Always **array** (even single ticker)
- Klines: **Array of arrays**
- Balances: **Array** (spot) or **object** (futures)
- Orders: **Array** (list) or **object** (single)
- Positions: **Array**

### No Wrapper
- Gate.io returns data **directly** (no `{"data": ...}` wrapper)
- Errors use `{"label": ..., "message": ...}` format

### Timestamps
- Server time: **seconds**
- Order times: **seconds** (with `*_ms` alternatives)
- Klines: **seconds**
- Orderbook: **milliseconds**

### Gate.io Quirks
1. Klines array order is **different**: `[time, volume, close, high, low, open]`
2. Single ticker returns **array** with one element
3. Futures position `size` can be **negative** (for shorts)
4. No response wrapper (data is top-level)

---

## Sources

- [Gate.io API Documentation](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Spot API](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Futures API](https://www.gate.com/docs/futures/api/index.html)
- [GitHub - gateio/gateapi-python OrderBook](https://github.com/gateio/gateapi-python/blob/master/docs/OrderBook.md)

---

**Research completed**: 2026-01-21
