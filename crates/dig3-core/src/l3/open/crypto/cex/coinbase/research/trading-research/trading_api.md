# Coinbase Advanced Trade API — Trading API Reference

Base URL: `https://api.coinbase.com/api/v3/brokerage/`

> Coinbase Advanced Trade is the successor to Coinbase Pro. All new integrations must use v3 endpoints.

---

## 1. ORDER TYPES

Coinbase uses a **nested `order_configuration` object** rather than a flat `type` field. The key name within `order_configuration` encodes both order type and time-in-force simultaneously.

### Supported Order Configuration Keys

| Config Key | Type | TIF | Notes |
|---|---|---|---|
| `market_market_ioc` | Market | IOC | Market buy/sell at best available price |
| `market_market_fok` | Market | FOK | Market fill-or-kill |
| `sor_limit_ioc` | Limit | IOC | Smart order routing, limit price |
| `limit_limit_gtc` | Limit | GTC | Standard limit, good-til-canceled |
| `limit_limit_gtd` | Limit | GTD | Limit with explicit expiry `end_time` |
| `limit_limit_fok` | Limit | FOK | Limit fill-or-kill (taker only) |
| `twap_limit_gtd` | TWAP | GTD | Time-weighted average price execution |
| `stop_limit_stop_limit_gtc` | Stop-Limit | GTC | Triggers at stop price, rests as limit |
| `stop_limit_stop_limit_gtd` | Stop-Limit | GTD | Stop-limit with expiry |
| `trigger_bracket_gtc` | Bracket | GTC | Entry + TP + SL in ONE order (unique!) |
| `trigger_bracket_gtd` | Bracket | GTD | Bracket order with expiry |
| `scaled_limit_gtc` | Scaled | GTC | Splits large order into N limit orders |

### Time-In-Force Enum Values (used in `list_orders`)
- `GOOD_UNTIL_CANCELLED` (GTC)
- `GOOD_UNTIL_DATE_TIME` (GTD)
- `IMMEDIATE_OR_CANCEL` (IOC)
- `FILL_OR_KILL` (FOK)

---

## 2. ORDER MANAGEMENT ENDPOINTS

### 2.1 Create Order

```
POST /api/v3/brokerage/orders
```

**Required Auth**: Yes (Bearer JWT)
**Permission**: `trade`

#### Request Body

```json
{
  "client_order_id": "0000-00000-000000",
  "product_id": "BTC-USD",
  "side": "BUY",
  "order_configuration": {
    // ONE of the config variants below
  },
  "leverage": "1.0",
  "margin_type": "CROSS",
  "preview_id": "optional-preview-id",
  "attached_order_configuration": {
    // optional, only for limit_limit_gtc with bracket TP/SL
  }
}
```

#### order_configuration Variants

**Market (IOC):**
```json
"market_market_ioc": {
  "quote_size": "10.00"   // for BUY: spend this much quote currency
  // OR
  "base_size": "0.001"    // for SELL: sell this much base currency
}
```

**Limit GTC:**
```json
"limit_limit_gtc": {
  "base_size": "0.001",
  "limit_price": "50000.00",
  "post_only": false
}
```

**Limit GTD:**
```json
"limit_limit_gtd": {
  "base_size": "0.001",
  "limit_price": "50000.00",
  "end_time": "2024-12-31T23:59:59Z",
  "post_only": false
}
```

**Limit FOK:**
```json
"limit_limit_fok": {
  "base_size": "0.001",
  "limit_price": "50000.00"
}
```

**Stop-Limit GTC:**
```json
"stop_limit_stop_limit_gtc": {
  "base_size": "0.001",
  "limit_price": "49000.00",
  "stop_price": "49500.00",
  "stop_direction": "STOP_DIRECTION_STOP_DOWN"
}
```

Stop direction values:
- `STOP_DIRECTION_STOP_UP` — trigger when price rises to stop_price
- `STOP_DIRECTION_STOP_DOWN` — trigger when price falls to stop_price

**Stop-Limit GTD:**
```json
"stop_limit_stop_limit_gtd": {
  "base_size": "0.001",
  "limit_price": "49000.00",
  "stop_price": "49500.00",
  "stop_direction": "STOP_DIRECTION_STOP_DOWN",
  "end_time": "2024-12-31T23:59:59Z"
}
```

**Bracket GTC (entry + TP + SL):**
```json
"trigger_bracket_gtc": {
  "base_size": "0.001",
  "limit_price": "50000.00",
  "stop_trigger_price": "48000.00"
}
```
Note: `stop_trigger_price` sets the stop-loss level. The bracket order is a SELL with built-in downside protection.

**Bracket GTD:**
```json
"trigger_bracket_gtd": {
  "base_size": "0.001",
  "limit_price": "50000.00",
  "stop_trigger_price": "48000.00",
  "end_time": "2024-12-31T23:59:59Z"
}
```

**TWAP GTD:**
```json
"twap_limit_gtd": {
  "base_size": "1.0",
  "limit_price": "50000.00",
  "start_time": "2024-06-01T10:00:00Z",
  "end_time": "2024-06-01T12:00:00Z",
  "number_buckets": "12",
  "bucket_size": "0.083",
  "bucket_duration": "600s"
}
```

**Scaled Limit GTC:**
```json
"scaled_limit_gtc": {
  "base_size": "1.0",
  "num_orders": "10",
  "min_price": "45000.00",
  "max_price": "50000.00",
  "price_distribution": "FLAT",
  "size_distribution": "EVENLY_SPLIT"
}
```
Price distribution values: `FLAT`, `LINEAR_INCREASING`, `LINEAR_DECREASING`
Size distribution values: `INCREASING`, `DECREASING`, `EVENLY_SPLIT`

#### Attached TP/SL via attached_order_configuration

To attach a bracket TP/SL to an existing limit order entry (used for a BUY entry with automated SELL bracket):

```json
{
  "client_order_id": "YOUR_CLIENT_ORDER_ID",
  "product_id": "ETH-USDC",
  "side": "BUY",
  "order_configuration": {
    "limit_limit_gtc": {
      "base_size": "0.01",
      "limit_price": "1500.00"
    }
  },
  "attached_order_configuration": {
    "trigger_bracket_gtc": {
      "limit_price": "1600.00",
      "stop_trigger_price": "1300.00"
    }
  }
}
```

Rules for `attached_order_configuration`:
- Only `limit_limit_gtc` orders are eligible to carry an attachment
- Do NOT include `base_size` in the attached config — it inherits from the parent order
- `limit_price` is the take-profit target
- `stop_trigger_price` is the stop-loss trigger

#### Create Order Response

**Success (HTTP 200, `success: true`):**
```json
{
  "success": true,
  "success_response": {
    "order_id": "11111-00000-000000",
    "product_id": "BTC-USD",
    "side": "BUY",
    "client_order_id": "0000-00000-000000"
  },
  "order_configuration": {
    "limit_limit_gtc": {
      "base_size": "0.001",
      "limit_price": "50000.00",
      "post_only": false
    }
  }
}
```

**Error (HTTP 200, `success: false`):**
```json
{
  "success": false,
  "error_response": {
    "message": "Generic error explanation",
    "error_details": "Specific reason for failure",
    "new_order_failure_reason": "INSUFFICIENT_FUND"
  },
  "order_configuration": { }
}
```

Common `new_order_failure_reason` values:
- `INSUFFICIENT_FUND`
- `INVALID_PRODUCT_ID`
- `ORDER_ENTRY_DISABLED`
- `INTRADAY_MARGIN_NOT_READY`
- `UNKNOWN_FAILURE_REASON`

Note: Coinbase returns HTTP 200 even for order failures. Always check `success` field.

---

### 2.2 Edit Order

```
POST /api/v3/brokerage/orders/{order_id}/edit
```

**Restrictions**: Only `limit_limit_gtc` orders can be edited.

#### Request Body

```json
{
  "order_id": "11111-00000-000000",
  "new_base_size": "0.002",
  "new_limit_price": "51000.00"
}
```

#### Response

Returns updated order object. The `edit_history` field is appended to the order (also visible in `get_order` and `list_orders` responses after an edit).

---

### 2.3 Edit Order Preview

```
POST /api/v3/brokerage/orders/{order_id}/edit_preview
```

Same body as Edit Order. Returns projected results without committing changes. Use to validate before editing.

---

### 2.4 Cancel Orders (Batch)

```
POST /api/v3/brokerage/orders/batch_cancel
```

Note: Cancel uses **POST**, not DELETE.

#### Request Body

```json
{
  "order_ids": [
    "11111-00000-000000",
    "22222-00000-000000"
  ]
}
```

**Maximum**: 100 `order_ids` per request. Exceeding returns `InvalidArgument` error.

#### Response

```json
{
  "results": [
    {
      "success": true,
      "failure_reason": "UNKNOWN_CANCEL_FAILURE_REASON",
      "order_id": "11111-00000-000000"
    },
    {
      "success": false,
      "failure_reason": "COMMANDER_REJECTED_CANCEL_ORDER",
      "order_id": "22222-00000-000000"
    }
  ]
}
```

Each entry in `results` corresponds to one order_id and has its own `success` flag.

---

### 2.5 Get Order

```
GET /api/v3/brokerage/orders/historical/{order_id}
```

#### Response (Order Object)

```json
{
  "order": {
    "order_id": "11111-00000-000000",
    "product_id": "BTC-USD",
    "user_id": "1234567",
    "order_configuration": { },
    "side": "BUY",
    "client_order_id": "0000-00000-000000",
    "status": "OPEN",
    "time_in_force": "GOOD_UNTIL_CANCELLED",
    "created_time": "2021-05-31T09:59:59.000Z",
    "last_update_time": "2021-05-31T09:59:59.000Z",
    "completion_percentage": "50",
    "filled_size": "0.0005",
    "average_filled_price": "50100.00",
    "fee": "0.25",
    "number_of_fills": "2",
    "filled_value": "25.05",
    "pending_cancel": false,
    "size_in_quote": false,
    "total_fees": "0.50",
    "size_inclusive_of_fees": false,
    "total_value_after_fees": "25.30",
    "trigger_status": "INVALID_ORDER_TYPE",
    "order_type": "LIMIT",
    "reject_reason": "REJECT_REASON_UNSPECIFIED",
    "settled": false,
    "product_type": "SPOT",
    "reject_message": "",
    "cancel_message": "",
    "order_placement_source": "RETAIL_ADVANCED",
    "outstanding_hold_amount": "24.75",
    "is_liquidation": false,
    "last_fill_time": "2021-05-31T09:59:59.000Z",
    "edit_history": [],
    "leverage": "1",
    "margin_type": "CROSS"
  }
}
```

#### Order Status Values
- `PENDING` — order received but not yet open
- `OPEN` — resting on the order book
- `FILLED` — fully executed
- `CANCELLED` — canceled by user or system
- `EXPIRED` — GTD order past its end_time
- `FAILED` — order failed to place
- `QUEUED` — queued for processing
- `CANCEL_QUEUED` — cancel request received, pending
- `EDIT_QUEUED` — edit request received, pending
- `UNKNOWN_ORDER_STATUS`

#### Order Type Values (for filtering)
`MARKET`, `LIMIT`, `STOP`, `STOP_LIMIT`, `BRACKET`, `TWAP`, `ROLL_OPEN`, `ROLL_CLOSE`, `LIQUIDATION`, `SCALED`

---

### 2.6 List Orders

```
GET /api/v3/brokerage/orders/historical/batch
```

#### Query Parameters

| Parameter | Type | Description |
|---|---|---|
| `order_ids` | string[] | Filter by specific order IDs |
| `product_ids` | string[] | Filter by product (defaults to all) |
| `product_type` | enum | `SPOT`, `FUTURE`, `UNKNOWN_PRODUCT_TYPE` |
| `order_status` | string[] | Filter by status (see status values above) |
| `order_types` | string[] | Filter by type (MARKET, LIMIT, etc.) |
| `order_side` | enum | `BUY` or `SELL` |
| `time_in_forces` | string[] | Filter by TIF |
| `start_date` | RFC3339 | Inclusive start time |
| `end_date` | RFC3339 | Exclusive end time |
| `limit` | integer | Results per page |
| `cursor` | string | Pagination cursor |
| `sort_by` | enum | `LIMIT_PRICE`, `LAST_FILL_TIME`, `LAST_UPDATE_TIME` |
| `contract_expiry_type` | enum | `EXPIRING` or `PERPETUAL` (futures only) |
| `asset_filters` | string[] | Filter by base asset symbol |
| `order_placement_source` | enum | `RETAIL_SIMPLE` or `RETAIL_ADVANCED` |

#### Response

```json
{
  "orders": [ /* array of Order objects (same structure as Get Order) */ ],
  "has_next": true,
  "cursor": "cursor_value_for_next_page",
  "sequence": "1"
}
```

---

### 2.7 List Fills

```
GET /api/v3/brokerage/orders/historical/fills
```

#### Query Parameters

| Parameter | Type | Description |
|---|---|---|
| `order_id` | string | Filter by specific order |
| `product_id` | string | Filter by product |
| `start_sequence_timestamp` | RFC3339 | Start time |
| `end_sequence_timestamp` | RFC3339 | End time |
| `retail_portfolio_id` | string | (Deprecated) |
| `limit` | integer | Results per page |
| `cursor` | string | Pagination cursor |

#### Fill Object Response

```json
{
  "fills": [
    {
      "entry_id": "22222-2222222-22222222",
      "trade_id": "1111-11111-111111",
      "order_id": "11111-00000-000000",
      "trade_time": "2021-05-31T09:59:59.000Z",
      "trade_type": "FILL",
      "price": "50100.00",
      "size": "0.0005",
      "commission": "1.25",
      "product_id": "BTC-USD",
      "sequence_timestamp": "2021-05-31T09:59:59.000Z",
      "liquidity_indicator": "MAKER",
      "size_in_quote": false,
      "user_id": "1234567",
      "side": "BUY",
      "retail_portfolio_id": "portfolio-uuid",
      "fill_source": "FILL_SOURCE_ADVANCED",
      "commission_detail_total": {
        "total_commission": "1.25",
        "gst_commission": "0.00",
        "withholding_commission": "0.00",
        "client_commission": "1.25"
      }
    }
  ],
  "cursor": "cursor_for_next_page"
}
```

`liquidity_indicator` values: `MAKER`, `TAKER`, `UNKNOWN_LIQUIDITY_INDICATOR`
`trade_type` values: `FILL`, `REVERSAL`, `CORRECTION`, `SYNTHETIC`

---

## 3. TP/SL & CONDITIONAL ORDERS

### Method 1: Standalone Bracket Order (trigger_bracket)

A `trigger_bracket_gtc` or `trigger_bracket_gtd` order is a **SELL** limit order with an embedded stop-loss trigger. It encodes both the take-profit limit price and a stop-loss trigger price in a single API call.

```json
{
  "client_order_id": "bracket-001",
  "product_id": "BTC-USD",
  "side": "SELL",
  "order_configuration": {
    "trigger_bracket_gtc": {
      "base_size": "0.001",
      "limit_price": "55000.00",
      "stop_trigger_price": "46000.00"
    }
  }
}
```

Behavior:
- `limit_price` = take-profit: order fills at or above this price
- `stop_trigger_price` = stop-loss trigger: if price drops to this, activates as market sell
- This is unique to Coinbase — no other major exchange offers TP+SL in a single call

### Method 2: Attached TP/SL on Entry Order

When placing a BUY limit entry, attach a bracket for the corresponding exit:

```json
{
  "client_order_id": "entry-001",
  "product_id": "BTC-USD",
  "side": "BUY",
  "order_configuration": {
    "limit_limit_gtc": {
      "base_size": "0.001",
      "limit_price": "50000.00"
    }
  },
  "attached_order_configuration": {
    "trigger_bracket_gtc": {
      "limit_price": "55000.00",
      "stop_trigger_price": "46000.00"
    }
  }
}
```

Note: `base_size` is omitted from `attached_order_configuration` (inherits from parent).
Only `limit_limit_gtc` entries support `attached_order_configuration`.

### Method 3: Standalone Stop-Limit

For a traditional stop-limit order (does not embed TP):

```json
{
  "order_configuration": {
    "stop_limit_stop_limit_gtc": {
      "base_size": "0.001",
      "limit_price": "48500.00",
      "stop_price": "49000.00",
      "stop_direction": "STOP_DIRECTION_STOP_DOWN"
    }
  }
}
```

---

## 4. BATCH OPERATIONS

| Operation | Supported | Max Per Call | Endpoint |
|---|---|---|---|
| Batch cancel | YES | 100 orders | `POST /orders/batch_cancel` |
| Batch create | NO | — | — |
| Batch edit | NO | — | — |

Only cancellation is batched. Creates and edits are single-order operations.

---

## 5. ORDER PREVIEW

```
POST /api/v3/brokerage/orders/preview
```

Same body as Create Order. Returns fee estimate, size info, and estimated total without placing the order. Use before submitting to show the user cost breakdown.

```json
{
  "client_order_id": "preview-id",
  "product_id": "BTC-USD",
  "side": "BUY",
  "order_configuration": {
    "market_market_ioc": {
      "quote_size": "100.00"
    }
  }
}
```

---

## 6. COMPLETE ORDER OBJECT — ALL FIELDS

```json
{
  "order_id": "11111-00000-000000",
  "product_id": "BTC-USD",
  "user_id": "1234567",
  "order_configuration": { },
  "side": "BUY",
  "client_order_id": "0000-00000-000000",
  "status": "OPEN",
  "time_in_force": "GOOD_UNTIL_CANCELLED",
  "created_time": "2021-05-31T09:59:59.000Z",
  "last_update_time": "2021-05-31T09:59:59.000Z",
  "completion_percentage": "50",
  "filled_size": "0.0005",
  "average_filled_price": "50100.00",
  "fee": "0.25",
  "number_of_fills": "2",
  "filled_value": "25.05",
  "pending_cancel": false,
  "size_in_quote": false,
  "total_fees": "0.50",
  "size_inclusive_of_fees": false,
  "total_value_after_fees": "25.30",
  "trigger_status": "INVALID_ORDER_TYPE",
  "order_type": "LIMIT",
  "reject_reason": "REJECT_REASON_UNSPECIFIED",
  "settled": false,
  "product_type": "SPOT",
  "reject_message": "",
  "cancel_message": "",
  "order_placement_source": "RETAIL_ADVANCED",
  "outstanding_hold_amount": "24.75",
  "is_liquidation": false,
  "last_fill_time": "2021-05-31T09:59:59.000Z",
  "edit_history": [
    {
      "price": "50500.00",
      "size": "0.001",
      "replace_accept_timestamp": "2021-05-31T09:58:00.000Z"
    }
  ],
  "leverage": "1",
  "margin_type": "CROSS",
  "retail_portfolio_id": "portfolio-uuid"
}
```

---

## Sources

- [Advanced API Order Management](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/orders)
- [Create Order Reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
- [List Orders Reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/list-orders)
- [List Fills Reference](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/list-fills)
- [Advanced Trade API Endpoints](https://docs.cdp.coinbase.com/advanced-trade/docs/api-overview/)
