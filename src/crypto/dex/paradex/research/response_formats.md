# Paradex API Response Formats

## General Response Structure

All Paradex API responses follow standard HTTP status codes with JSON payloads.

### Success Responses
- **200 OK**: Successful GET request
- **201 Created**: Successful POST request (order creation)
- **204 No Content**: Successful DELETE request (order cancellation)

### Error Responses
- **400 Bad Request**: Invalid parameters
- **401 Unauthorized**: Authentication failure
- **404 Not Found**: Resource not found
- **429 Too Many Requests**: Rate limit exceeded
- **500 Internal Server Error**: Server error

---

## Common Field Types

| Type | Description | Example |
|------|-------------|---------|
| `string` | Decimal numbers (preserve precision) | `"123.45"` |
| `integer` | Whole numbers, timestamps | `1681759756` |
| `enum` | Fixed set of values | `"BUY"`, `"SELL"` |
| `array` | List of items | `["item1", "item2"]` |
| `object` | Nested structure | `{ "key": "value" }` |

**Note**: Prices, sizes, and amounts are returned as **strings** to preserve decimal precision.

---

## Authentication Responses

### POST /auth

**Success (200)**:
```json
{
  "jwt_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error (401)**:
```json
{
  "error": "Unauthorized",
  "message": "Invalid signature"
}
```

---

## Market Data Responses

### GET /markets

**Response (200)**:
```json
{
  "results": [
    {
      "symbol": "BTC-USD-PERP",
      "base_currency": "BTC",
      "quote_currency": "USD",
      "settlement_currency": "USDC",
      "price_tick_size": "0.1",
      "price_feed_id": "0x...",
      "clamp_rate": "0.05",
      "asset_kind": "PERP",
      "market_kind": "cross",
      "open_at": 1681759756000,
      "expiry_at": null,
      "max_order_size": "1000000",
      "max_open_orders": 200,
      "order_size_increment": "0.001",
      "min_notional": "10",
      "position_limit": "10000000",
      "max_slippage": "0.1",
      "fee_config": {
        "api_fees": {
          "maker": "0.0002",
          "taker": "0.0005"
        },
        "interactive_fees": {
          "maker": "0.0003",
          "taker": "0.0007"
        },
        "rpi_fees": {
          "maker": "0.0001",
          "taker": "0.0004"
        }
      },
      "delta1_cross_margin_params": {
        "initial_margin_fraction": "0.05",
        "maintenance_margin_fraction": "0.03"
      },
      "option_cross_margin_params": null,
      "funding_period_hours": 8,
      "funding_multiplier": "1.0",
      "max_funding_rate": "0.0005",
      "interest_rate": "0.0001",
      "max_funding_rate_change": "0.00075",
      "option_type": null,
      "strike_price": null,
      "iv_bands_width": null,
      "contract_address": "0x...",
      "collateral_address": "0x...",
      "oracle_address": "0x...",
      "fee_account_address": "0x..."
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Market identifier (e.g., "BTC-USD-PERP") |
| `base_currency` | string | Base asset symbol |
| `quote_currency` | string | Quote asset symbol |
| `settlement_currency` | string | Settlement asset (usually USDC) |
| `price_tick_size` | string | Minimum price increment |
| `asset_kind` | enum | "PERP" or "PERP_OPTION" |
| `market_kind` | enum | "cross", "isolated", "isolated_margin" |
| `open_at` | integer | Market open timestamp (ms) |
| `expiry_at` | integer/null | Expiration timestamp for options (ms) |
| `max_order_size` | string | Maximum order size |
| `order_size_increment` | string | Minimum size increment |
| `min_notional` | string | Minimum order notional value |
| `position_limit` | string | Maximum position size |
| `fee_config` | object | Fee structure for different access types |
| `funding_period_hours` | integer | Hours between funding payments |
| `max_funding_rate` | string | Maximum funding rate per period |

### GET /markets/summary

**Response (200)**:
```json
{
  "results": [
    {
      "market": "BTC-USD-PERP",
      "best_bid": "65432.1",
      "best_ask": "65432.5",
      "best_bid_iv": null,
      "best_ask_iv": null,
      "last_traded_price": "65432.3",
      "mark_price": "65432.2",
      "spot_price": "65430.5",
      "volume_24h": "123456789.50",
      "total_volume": "987654321.00",
      "open_interest": "45678.5",
      "price_change_rate_24h": "0.0234",
      "funding_rate": "0.0001",
      "predicted_funding_rate": "0.00012",
      "greeks": {
        "delta": "0.65",
        "gamma": "0.012",
        "vega": "0.045",
        "rho": "0.002",
        "vanna": "0.001",
        "volga": "0.0005"
      },
      "created_at": 1681759756000
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `best_bid` | string | Highest bid price |
| `best_ask` | string | Lowest ask price |
| `best_bid_iv` | string/null | Implied volatility for bid (options only) |
| `best_ask_iv` | string/null | Implied volatility for ask (options only) |
| `last_traded_price` | string | Price of last trade |
| `mark_price` | string | Fair price for margin calculations |
| `spot_price` | string | Underlying spot price |
| `volume_24h` | string | 24-hour trading volume (USD) |
| `total_volume` | string | Lifetime total volume (USD) |
| `open_interest` | string | Total open positions (base currency) |
| `price_change_rate_24h` | string | 24-hour price change percentage |
| `funding_rate` | string | Current funding rate |
| `predicted_funding_rate` | string | Next funding rate prediction |
| `greeks` | object | Option greeks (null for perpetuals) |

### GET /orderbook/:market

**Response (200)**:
```json
{
  "market": "BTC-USD-PERP",
  "asks": [
    ["65432.5", "1.234"],
    ["65432.6", "2.456"],
    ["65432.7", "3.789"]
  ],
  "bids": [
    ["65432.4", "1.111"],
    ["65432.3", "2.222"],
    ["65432.2", "3.333"]
  ],
  "best_ask_api": ["65432.5", "1.234"],
  "best_ask_interactive": ["65432.4", "1.500"],
  "best_bid_api": ["65432.4", "1.111"],
  "best_bid_interactive": ["65432.5", "1.300"],
  "last_updated_at": 1681759756789,
  "seq_no": 12345678
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `asks` | array | Array of [price, size] ask orders |
| `bids` | array | Array of [price, size] bid orders |
| `best_ask_api` | array | Best ask excluding RPI (API orders only) |
| `best_ask_interactive` | array | Best ask from UI (includes RPI) |
| `best_bid_api` | array | Best bid excluding RPI |
| `best_bid_interactive` | array | Best bid from UI (includes RPI) |
| `last_updated_at` | integer | Last update timestamp (ms) |
| `seq_no` | integer | Sequence number for ordering updates |

---

## Account Responses

### GET /account

**Response (200)**:
```json
{
  "account": "0x129f3dc1b8962d8a87abc692424c78fda963ade0e1cd17bf3d1c26f8d41ee7a",
  "account_value": "125432.50",
  "free_collateral": "85432.50",
  "initial_margin_requirement": "35000.00",
  "maintenance_margin_requirement": "25000.00",
  "margin_cushion": "100432.50",
  "seq_no": 12345,
  "settlement_asset": "USDC",
  "status": "ACTIVE",
  "total_collateral": "120000.00",
  "updated_at": 1681759756789
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `account` | string | StarkNet account address |
| `account_value` | string | Total account value with unrealized PnL |
| `free_collateral` | string | Available for new positions |
| `initial_margin_requirement` | string | Margin needed to open positions |
| `maintenance_margin_requirement` | string | Margin needed to maintain positions |
| `margin_cushion` | string | Account value above maintenance margin |
| `seq_no` | integer | Sequence number for deduplication |
| `settlement_asset` | string | Settlement currency |
| `status` | string | "ACTIVE", "LIQUIDATION", etc. |
| `total_collateral` | string | Total collateral balance |
| `updated_at` | integer | Last update timestamp (ms) |

### GET /positions

**Response (200)**:
```json
{
  "results": [
    {
      "id": "pos_123456",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "side": "LONG",
      "status": "OPEN",
      "size": "1.5",
      "leverage": "10.0",
      "average_entry_price": "65000.00",
      "average_entry_price_usd": "65000.00",
      "average_exit_price": "0",
      "liquidation_price": "59500.00",
      "cost": "9750.00",
      "cost_usd": "9750.00",
      "unrealized_pnl": "648.00",
      "unrealized_funding_pnl": "-12.50",
      "realized_positional_pnl": "0",
      "realized_positional_funding_pnl": "0",
      "cached_funding_index": "123.456",
      "created_at": 1681759756789,
      "closed_at": null,
      "last_updated_at": 1681759756789,
      "seq_no": 12345,
      "last_fill_id": "fill_789"
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique position identifier |
| `side` | enum | "LONG" or "SHORT" |
| `status` | enum | "OPEN" or "CLOSED" |
| `size` | string | Position size (signed: + for long, - for short) |
| `leverage` | string | Position leverage |
| `average_entry_price` | string | Average entry price (base asset) |
| `average_entry_price_usd` | string | Average entry price (USD) |
| `liquidation_price` | string | Price at which position gets liquidated |
| `unrealized_pnl` | string | Current unrealized PnL with funding |
| `unrealized_funding_pnl` | string | Unrealized funding component |
| `realized_positional_pnl` | string | Realized PnL (resets on close/flip) |
| `cached_funding_index` | string | Last funding index snapshot |

### GET /account/history

**Response (200)**:
```json
{
  "data": [100.5, 102.3, 98.7, 105.2, 110.8],
  "timestamps": [1681759756000, 1681763356000, 1681766956000, 1681770556000, 1681774156000]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `data` | array | Ordered numeric values (PnL, value, volume, or fee savings) |
| `timestamps` | array | Corresponding Unix timestamps (ms) |

**Query Parameter `type`**:
- `"pnl"`: Profit and loss history
- `"value"`: Account value history
- `"volume"`: Trading volume history
- `"fee_savings"`: Fee savings from RPI

---

## Order Responses

### POST /orders

**Success Response (201)**:
```json
{
  "id": "order_123456789",
  "status": "NEW",
  "account": "0x...",
  "market": "BTC-USD-PERP",
  "side": "BUY",
  "type": "LIMIT",
  "size": "0.5",
  "price": "65000.00",
  "remaining_size": "0.5",
  "avg_fill_price": "0",
  "instruction": "GTC",
  "flags": [],
  "stp": null,
  "created_at": 1681759756789,
  "last_updated_at": 1681759756789,
  "seq_no": 12345,
  "client_id": "my_order_123",
  "signature": "...",
  "signature_timestamp": 1681759756789,
  "trigger_price": null,
  "recv_window": null,
  "on_behalf_of_account": null
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Paradex-generated unique identifier |
| `status` | enum | "NEW", "UNTRIGGERED", "OPEN", "CLOSED" |
| `side` | enum | "BUY" or "SELL" |
| `type` | enum | Order type (MARKET, LIMIT, etc.) |
| `remaining_size` | string | Unfilled quantity |
| `avg_fill_price` | string | Average execution price |
| `instruction` | enum | "GTC", "POST_ONLY", "IOC", "RPI" |
| `flags` | array | Order modifiers (e.g., ["REDUCE_ONLY"]) |
| `stp` | enum/null | Self-trade prevention mode |
| `client_id` | string/null | User-provided identifier |
| `seq_no` | integer | Sequence number for ordering |

**Status Progression**:
1. **NEW**: Initial status, queued for risk checks
2. **OPEN**: Passed risk checks, in matching engine
3. **CLOSED**: Fully filled, cancelled, or rejected
4. **UNTRIGGERED**: Stop order waiting for trigger

**Order Types**:
- `MARKET`: Market order (price = "0")
- `LIMIT`: Limit order
- `STOP_LIMIT`: Stop-limit order
- `STOP_MARKET`: Stop-market order
- `TAKE_PROFIT_LIMIT`: Take-profit limit
- `TAKE_PROFIT_MARKET`: Take-profit market
- `STOP_LOSS_LIMIT`: Stop-loss limit
- `STOP_LOSS_MARKET`: Stop-loss market

**Instructions**:
- `GTC`: Good-till-cancel
- `IOC`: Immediate-or-cancel
- `POST_ONLY`: Only make (reject if taker)
- `RPI`: Retail Price Improvement

### GET /orders/:order_id

**Response (200)**: Same structure as POST /orders response

### DELETE /orders/:order_id

**Success Response (204)**: Empty body (No Content)

**Note**: Order queued for cancellation. Check status via GET /orders/:id or WebSocket.

### POST /orders/batch

**Response (201)**:
```json
{
  "results": [
    {
      "id": "order_1",
      "status": "NEW",
      ...
    },
    {
      "id": "order_2",
      "status": "NEW",
      ...
    }
  ]
}
```

---

## Trade History Responses

### GET /fills

**Response (200)**:
```json
{
  "results": [
    {
      "id": "fill_123456",
      "order_id": "order_789",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "side": "BUY",
      "size": "0.5",
      "price": "65000.00",
      "fee": "16.25",
      "fee_currency": "USDC",
      "liquidity": "TAKER",
      "is_rpi": false,
      "is_liquidation": false,
      "created_at": 1681759756789,
      "seq_no": 12345
    }
  ]
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique fill identifier |
| `order_id` | string | Associated order ID |
| `side` | enum | "BUY" or "SELL" |
| `size` | string | Filled quantity |
| `price` | string | Execution price |
| `fee` | string | Trading fee paid |
| `fee_currency` | string | Fee denomination (usually USDC) |
| `liquidity` | enum | "MAKER" or "TAKER" |
| `is_rpi` | boolean | Retail Price Improvement flag |
| `is_liquidation` | boolean | Fill from liquidation |
| `created_at` | integer | Execution timestamp (ms) |

---

## System Responses

### GET /system/config

**Response (200)**:
```json
{
  "chain_id": "SN_MAIN",
  "block_explorer_url": "https://starkscan.co",
  "contract_address": "0x...",
  "maintenance_mode": false,
  "features": {
    "trading_enabled": true,
    "withdrawals_enabled": true,
    "deposits_enabled": true
  }
}
```

### GET /system/time

**Response (200)**:
```json
{
  "server_time": 1681759756789
}
```

### GET /system/state

**Response (200)**:
```json
{
  "status": "operational",
  "components": {
    "api": "operational",
    "matching_engine": "operational",
    "websocket": "operational",
    "blockchain": "operational"
  }
}
```

---

## Error Response Format

### Standard Error Response

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error description",
  "details": {
    "field": "Additional context"
  }
}
```

### Common Error Codes

| Status | Error Code | Description |
|--------|------------|-------------|
| 400 | `INVALID_PARAMETER` | Invalid request parameter |
| 400 | `INVALID_SIGNATURE` | Order signature validation failed |
| 400 | `INSUFFICIENT_MARGIN` | Not enough margin for order |
| 400 | `ORDER_ALREADY_CLOSED` | Cannot cancel closed order |
| 401 | `UNAUTHORIZED` | Missing or invalid JWT |
| 401 | `TOKEN_EXPIRED` | JWT expired (5-minute lifetime) |
| 404 | `NOT_FOUND` | Resource not found |
| 429 | `RATE_LIMIT_EXCEEDED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |

### Example Error Responses

**Invalid Parameter (400)**:
```json
{
  "error": "INVALID_PARAMETER",
  "message": "Invalid market symbol",
  "details": {
    "parameter": "market",
    "value": "INVALID-SYMBOL",
    "valid_format": "BTC-USD-PERP"
  }
}
```

**Unauthorized (401)**:
```json
{
  "error": "UNAUTHORIZED",
  "message": "JWT token has expired",
  "details": {
    "expired_at": 1681759756789
  }
}
```

**Rate Limit (429)**:
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Order endpoint rate limit exceeded",
  "details": {
    "limit": "800 req/s",
    "retry_after": 1000
  }
}
```

---

## Pagination

**Note**: Documentation does not specify pagination format for list endpoints. Typical implementation might use:

```json
{
  "results": [...],
  "pagination": {
    "total": 1000,
    "page": 1,
    "page_size": 100,
    "has_more": true
  }
}
```

Or cursor-based:

```json
{
  "results": [...],
  "cursor": "eyJpZCI6MTIzNDU2fQ==",
  "has_more": true
}
```

**Recommendation**: Check Python SDK implementation for actual pagination behavior.

---

## Timestamp Format

All timestamps are **Unix milliseconds** (not seconds):

```json
{
  "created_at": 1681759756789,  // 2023-04-17 20:22:36.789 UTC
  "updated_at": 1681759756789
}
```

**Conversion**:
```rust
// Rust
use chrono::{DateTime, Utc};

let timestamp_ms = 1681759756789i64;
let datetime = DateTime::from_timestamp_millis(timestamp_ms).unwrap();

// To timestamp
let now_ms = Utc::now().timestamp_millis();
```

---

## Decimal Precision

All numeric values representing prices, sizes, and amounts are **strings**:

**Why strings?**
- Avoid floating-point rounding errors
- Preserve exact decimal values
- Cross-language compatibility

**Parsing**:
```rust
use rust_decimal::Decimal;
use std::str::FromStr;

let price_str = "65432.50";
let price = Decimal::from_str(price_str).unwrap();

// Arithmetic
let quantity = Decimal::from_str("1.5").unwrap();
let total = price * quantity; // 98148.75
```

---

## Enumerations

### Common Enums

**Side**:
- `BUY`
- `SELL`

**Order Status**:
- `NEW`: Queued for risk checks
- `UNTRIGGERED`: Stop order waiting for trigger
- `OPEN`: Active in order book
- `CLOSED`: Filled, cancelled, or rejected

**Order Type**:
- `MARKET`
- `LIMIT`
- `STOP_LIMIT`
- `STOP_MARKET`
- `TAKE_PROFIT_LIMIT`
- `TAKE_PROFIT_MARKET`
- `STOP_LOSS_LIMIT`
- `STOP_LOSS_MARKET`

**Instruction**:
- `GTC`: Good-till-cancel
- `IOC`: Immediate-or-cancel
- `POST_ONLY`: Maker-only
- `RPI`: Retail Price Improvement

**Position Side**:
- `LONG`
- `SHORT`

**Position Status**:
- `OPEN`
- `CLOSED`

**Account Status**:
- `ACTIVE`
- `LIQUIDATION`
- `SUSPENDED`

**Asset Kind**:
- `PERP`: Perpetual futures
- `PERP_OPTION`: Perpetual options

**Market Kind**:
- `cross`: Cross-margin
- `isolated`: Isolated margin
- `isolated_margin`: Isolated margin (alternative name)

---

## Summary

1. **Format**: JSON for all responses
2. **Precision**: Strings for decimal numbers
3. **Timestamps**: Unix milliseconds (not seconds)
4. **Status Codes**: Standard HTTP codes
5. **Errors**: Structured error objects with codes and details
6. **Arrays**: Wrapped in `results` field for list endpoints
7. **Null Values**: Use `null` for absent optional fields
8. **Enums**: Uppercase strings for enumerated values

---

## Additional Resources

- **API Documentation**: https://docs.paradex.trade/api/general-information
- **Python SDK**: https://github.com/tradeparadex/paradex-py (see response models)
- **Code Samples**: https://github.com/tradeparadex/code-samples
