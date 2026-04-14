# Bitstamp Trading API — Full Specification

> **CRITICAL NOTE**: Bitstamp is SPOT ONLY. No futures, no perpetuals, no margin trading, no leverage.
> Buy and sell use SEPARATE endpoints — this is unusual and must be modeled explicitly in the V5 trait.

---

## 1. ORDER TYPES

Bitstamp supports the following order types on the REST API:

### 1.1 Limit Order
- Buy: `POST /api/v2/buy/{currency_pair}/`
- Sell: `POST /api/v2/sell/{currency_pair}/`
- Specify `amount` (base currency) and `price`
- Optional modifiers: `daily_order`, `ioc_order`, `fok_order`, `moc_order`, `gtd_order`

### 1.2 Market Order
- Buy: `POST /api/v2/buy/market/{currency_pair}/`
- Sell: `POST /api/v2/sell/market/{currency_pair}/`
- Specify `amount` in base currency (e.g., BTC quantity)
- No `price` parameter required or accepted

### 1.3 Instant Order (Quote-Currency Amount)
- Buy: `POST /api/v2/buy/instant/{currency_pair}/`
- Sell: `POST /api/v2/sell/instant/{currency_pair}/`
- Specify `amount` in COUNTER (quote) currency (e.g., USD you want to spend)
- Distinct from market orders — you specify how much fiat to spend, not how much crypto to buy
- Fills against order book at market price; eats all available orders until amount is spent

### 1.4 Stop-Limit Order
- Bitstamp supports stop-limit orders (stop price triggers a limit order)
- Parameter: `stop_price` — when market hits this price, a limit order is placed at `price`
- Stop-limit buy and stop-limit sell are supported
- Used through the standard buy/sell limit endpoint with `stop_price` parameter

### 1.5 DISCONTINUED (as of May 14, 2025)
- **Stop Market Buy Orders** — removed April 16, 2025; auto-closed May 14, 2025
- **Trailing Stop Orders** (all types) — removed April 16, 2025

### 1.6 Time-in-Force Modifiers (optional flags on limit orders)
| Parameter | Description |
|-----------|-------------|
| `daily_order` | Good-Till-Day — order expires at midnight UTC |
| `ioc_order` | Immediate or Cancel — fills what it can, cancels remainder |
| `fok_order` | Fill or Kill — fills entirely or cancels entirely |
| `moc_order` | Market on Close |
| `gtd_order` | Good-Till-Date — requires `expire_time` parameter (Unix timestamp) |

> **NOTE**: Only ONE time-in-force modifier may be used per order.

---

## 2. ORDER MANAGEMENT ENDPOINTS

All private endpoints use:
- Method: `POST`
- Content-Type: `application/x-www-form-urlencoded`
- Base URL: `https://www.bitstamp.net`

### 2.1 Place Limit Buy Order
```
POST /api/v2/buy/{currency_pair}/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Amount in base currency |
| `price` | string/decimal | Yes | Limit price |
| `client_order_id` | string | No | Custom order ID (no longer enforced unique as of Nov 2023) |
| `limit_price` | string/decimal | No | Max slippage limit price (safety) |
| `stop_price` | string/decimal | No | Stop trigger price (makes it a stop-limit order) |
| `daily_order` | bool (True/False) | No | Good-Till-Day |
| `ioc_order` | bool | No | Immediate or Cancel |
| `fok_order` | bool | No | Fill or Kill |
| `moc_order` | bool | No | Market on Close |
| `gtd_order` | bool | No | Good-Till-Date (requires `expire_time`) |
| `expire_time` | integer | No | Unix timestamp for GTD orders |

**Response:**
```json
{
  "id": "1234567890",
  "datetime": "2024-01-15 12:34:56.789000",
  "type": "0",
  "price": "45000.00",
  "amount": "0.10000000",
  "client_order_id": "my-order-001",
  "market": "btcusd",
  "status": "Open"
}
```

### 2.2 Place Limit Sell Order
```
POST /api/v2/sell/{currency_pair}/
```
Same parameters as buy limit. `type` in response will be `"1"` (sell).

### 2.3 Place Market Buy Order
```
POST /api/v2/buy/market/{currency_pair}/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Base currency amount to buy |
| `client_order_id` | string | No | Custom order ID |

**Response:** Same structure as limit order response but with `status: "Finished"` typically.

### 2.4 Place Market Sell Order
```
POST /api/v2/sell/market/{currency_pair}/
```
Same as market buy. `type` = `"1"`.

### 2.5 Place Instant Buy Order (quote-currency amount)
```
POST /api/v2/buy/instant/{currency_pair}/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | string/decimal | Yes | Amount in COUNTER (quote) currency (e.g., USD) |
| `client_order_id` | string | No | Custom order ID |

### 2.6 Place Instant Sell Order
```
POST /api/v2/sell/instant/{currency_pair}/
```
Same as instant buy. Sell `amount` in base currency, specify in counter currency terms.

### 2.7 Cancel Order
```
POST /api/v2/cancel_order/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string/integer | Yes (or `client_order_id`) | Order ID |
| `client_order_id` | string | Yes (or `id`) | Custom order ID |

**Response:**
```json
{
  "id": "1234567890",
  "amount": "0.10000000",
  "price": "45000.00",
  "type": "0",
  "market": "btcusd"
}
```
> Returns the cancelled order details on success.

**Error response (if already cancelled/filled):**
```json
{
  "status": "error",
  "reason": "Order not found.",
  "code": "Order not found."
}
```

### 2.8 Cancel All Orders (all pairs)
```
POST /api/v2/cancel_all_orders/
```
No parameters required beyond authentication.

**Response:** `true` (boolean) on success.

### 2.9 Cancel All Orders for a Pair
```
POST /api/v2/cancel_all_orders/{currency_pair}/
```
No body parameters required.

**Response:** `true` on success.

### 2.10 Get Order Status
```
POST /api/v2/order_status/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string/integer | Yes (or `client_order_id`) | Order ID |
| `client_order_id` | string | Yes (or `id`) | Custom order ID |

**Response:**
```json
{
  "status": "Open",
  "id": "1234567890",
  "market": "btcusd",
  "created_datetime": "2024-01-15 12:34:56.789000",
  "updated_datetime": "2024-01-15 12:34:57.000000",
  "type": "0",
  "price": "45000.00",
  "amount": "0.10000000",
  "remaining_amount": "0.05000000",
  "client_order_id": "my-order-001",
  "transactions": [
    {
      "tid": 9876543,
      "price": "44999.50",
      "amount": "0.05000000",
      "fee": "0.50",
      "datetime": "2024-01-15 12:34:57.000000",
      "type": "2"
    }
  ]
}
```

**Order status values:**
| Value | Meaning |
|-------|---------|
| `"Open"` | Active, waiting to fill |
| `"Finished"` | Fully filled |
| `"Canceled"` | Cancelled by user |
| `"Queue"` | Queued for processing |
| `"Expired"` | GTD/GTT order that expired |

> **NOTE**: `type` field: `"0"` = buy, `"1"` = sell

### 2.11 Get All Open Orders
```
POST /api/v2/open_orders/all/
```
No body parameters required beyond authentication.

**Response:** Array of order objects:
```json
[
  {
    "id": "1234567890",
    "client_order_id": "my-order-001",
    "datetime": "2024-01-15 12:34:56.789000",
    "type": "0",
    "price": "45000.00",
    "amount": "0.10000000",
    "market": "btcusd",
    "currency_pair": "btcusd"
  }
]
```

### 2.12 Get Open Orders for a Pair
```
POST /api/v2/open_orders/{currency_pair}/
```
Same response as above but filtered to one pair.

### 2.13 Get User Transactions (Trade History)
```
POST /api/v2/user_transactions/{currency_pair}/
```
**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `offset` | integer | No | Pagination offset (default 0) |
| `limit` | integer | No | Number of results (default 100, max 1000) |
| `sort` | string | No | `"asc"` or `"desc"` (default `"desc"`) |
| `since_id` | integer | No | Return transactions with ID >= this value |
| `since_timestamp` | integer | No | Unix timestamp filter |

**Response (one transaction):**
```json
{
  "id": 51366122,
  "order_id": 1234567890,
  "datetime": "2024-01-15 12:34:57",
  "type": "2",
  "btc": "0.05000000",
  "usd": "-2250.00",
  "fee": "0.50",
  "btc_usd": "45000.00"
}
```
Transaction `type` values: `"0"` = deposit, `"1"` = withdrawal, `"2"` = trade.

### 2.14 Replace Order (Modify)
```
POST /api/v2/replace_order/
```
Atomically cancels an existing order and places a new one.

**Parameters:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string/integer | Yes (or `orig_client_order_id`) | Order ID to replace |
| `orig_client_order_id` | string | Yes (or `id`) | Client ID of order to replace |
| `amount` | string/decimal | Yes | New amount |
| `price` | string/decimal | Yes | New price |
| `client_order_id` | string | No | New client order ID |

**Response:** New order object (same format as place order response).

---

## 3. TP/SL AND CONDITIONAL ORDERS

### What IS Supported:
- **Stop-Limit Orders**: `stop_price` parameter on limit buy/sell endpoints
  - When market price reaches `stop_price`, a limit order at `price` is placed
  - Available for both buy and sell sides
- **Stop-Limit Sell** remains available (as of May 2025)

### What is NOT Supported:
- Take-Profit orders — NOT available via API
- Stop-Market orders — DISCONTINUED May 14, 2025
- Trailing Stop orders — DISCONTINUED May 14, 2025
- OCO (One-Cancels-Other) — NOT available
- Bracket orders — NOT available

### Stop-Limit Example (Sell):
```
POST /api/v2/sell/btcusd/
amount=0.1
price=43000.00       # limit price (execute at this price or better)
stop_price=43500.00  # trigger: when BTC drops to 43500, place limit sell at 43000
```

---

## 4. BATCH OPERATIONS

- **No batch order placement** — each order must be submitted individually
- **Cancel all** is available: `/api/v2/cancel_all_orders/` and `/api/v2/cancel_all_orders/{currency_pair}/`
- No batch status query endpoint

---

## 5. ORDER MODIFICATION

Bitstamp does NOT support direct order modification (amend price/quantity on existing order).

The `replace_order` endpoint (`POST /api/v2/replace_order/`) provides atomic cancel+replace:
- Atomically cancels the original order
- Places a new order with new parameters
- If placement of new order fails, the original order is NOT restored
- Returns the new order object

For V5 trait implementation: `modify_order()` should be implemented via `replace_order`.

---

## 6. ORDER RESPONSE FORMAT — FIELD REFERENCE

### Standard Order Object
| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Bitstamp-assigned order ID (numeric string) |
| `client_order_id` | string | User-supplied custom ID (optional) |
| `datetime` | string | Creation timestamp `"YYYY-MM-DD HH:MM:SS.ffffff"` |
| `updated_datetime` | string | Last update timestamp (in order_status only) |
| `created_datetime` | string | Creation timestamp (in order_status only) |
| `type` | string | `"0"` = buy, `"1"` = sell |
| `price` | string | Limit price (decimal string) |
| `amount` | string | Original order amount in base currency |
| `remaining_amount` | string | Unfilled amount remaining |
| `market` | string | Market symbol e.g. `"btcusd"` (replaces deprecated `currency_pair`) |
| `status` | string | `"Open"`, `"Finished"`, `"Canceled"`, `"Queue"`, `"Expired"` |
| `transactions` | array | List of fills (in order_status response) |

### Transaction Object (inside order_status.transactions)
| Field | Type | Description |
|-------|------|-------------|
| `tid` | integer | Trade ID |
| `price` | string | Fill price |
| `amount` | string | Fill amount in base currency |
| `fee` | string | Fee charged for this fill |
| `datetime` | string | Fill timestamp |
| `type` | string | `"2"` for trade |

### Symbol Format
- Use lowercase: `btcusd`, `ethusd`, `ltcbtc`
- In URL path: `{currency_pair}` segment
- In response: `market` field (the `currency_pair` field is deprecated)

---

## 7. NOTABLE QUIRKS FOR V5 TRAIT DESIGN

1. **Separate buy/sell endpoints** — unlike most exchanges which have a single `/order` endpoint with `side` param. Must handle routing in `place_order()`.
2. **No unified order endpoint** — no single `POST /order` with `{"side": "buy", "type": "limit"}` pattern.
3. **Instant vs Market distinction** — instant = specify quote amount; market = specify base amount. Both are "market" orders semantically.
4. **POST for all private calls** — including queries (open_orders, order_status). Not REST-idiomatic.
5. **String decimals** — all prices and amounts are returned as strings, not floats.
6. **`market` field replacing `currency_pair`** — as of 2023 changelog, `currency_pair` field is deprecated.
7. **`client_order_id` no longer enforced unique** — as of November 2023.
8. **Spot only** — `place_order()` for futures/perpetuals should return `UnsupportedOperation`.
