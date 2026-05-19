# Kraken Trading API — Order Types, Management, and Execution

## Critical Architecture Note

Kraken Spot and Kraken Futures are **completely separate systems** with different:
- Base URLs
- Authentication mechanisms
- Request formats
- Order types and parameters
- Rate limit systems

---

## 1. ORDER TYPES

### Spot Order Types

Parameter name: `ordertype` (form-encoded in POST body)

| Value | Description |
|-------|-------------|
| `market` | Market order — execute immediately at best available price |
| `limit` | Limit order — execute at specified price or better |
| `stop-loss` | Stop order — triggers market order when stop price reached |
| `take-profit` | Take profit — triggers market order when take-profit price reached |
| `stop-loss-limit` | Stop order that places a limit order when triggered |
| `take-profit-limit` | Take-profit that places a limit order when triggered |
| `trailing-stop` | Trailing stop — stop price trails market by offset |
| `trailing-stop-limit` | Trailing stop that places a limit order when triggered |
| `settle-position` | Settle margin position — effectively closes a margin position |

### Spot Time-In-Force (`timeinforce`)

| Value | Description |
|-------|-------------|
| `GTC` | Good Till Canceled (default) — order stays open until filled or cancelled |
| `IOC` | Immediate Or Cancel — fill immediately, cancel any unfilled remainder |
| `GTD` | Good Till Date — order stays open until `expiretm` timestamp |

### Spot Order Flags (`oflags`) — comma-delimited list

| Flag | Description |
|------|-------------|
| `post` | Post-Only — order will only be placed as maker; rejected if would immediately fill |
| `fcib` | Fee in base currency — prefer to pay fees in base asset |
| `fciq` | Fee in quote currency — prefer to pay fees in quote asset |
| `nompp` | No Market Price Protection — disable price deviation protection for market orders |
| `viqc` | Volume in quote currency — `volume` param is denominated in quote asset |

### Spot Leverage

- `leverage` param: integer string like `"2"`, `"3"`, `"5"` (max varies per pair)
- Leverage is specified **per-order**, not per-position — there is no separate set_leverage endpoint
- Adding leverage makes it a margin trade; Kraken handles borrow/repay automatically
- Default: no leverage (spot trade)

### Spot Conditional Close Order (OTO)

Attach a take-profit or stop-loss to an order at creation time:

| Param | Description |
|-------|-------------|
| `close[ordertype]` | Order type for the close leg (e.g. `stop-loss-limit`) |
| `close[price]` | Trigger/limit price for the close leg |
| `close[price2]` | Secondary limit price (for `stop-loss-limit`, `take-profit-limit`) |

### Spot Reduce-Only

- `reduce_only` — boolean, only reduces existing positions (margin trading)

---

### Futures Order Types

Parameter name: `orderType` (camelCase, JSON body)

| Value | Description |
|-------|-------------|
| `lmt` | Limit order — rests in book at specified price |
| `post` | Post-Only limit — rejected if would immediately fill (maker-only) |
| `mkt` | Market order — execute immediately at best available price |
| `ioc` | Immediate-Or-Cancel — fill immediately, cancel remainder |
| `stp` | Stop order — triggers when trigger price is reached |
| `take_profit` | Take-profit order — triggers when take-profit price is reached |
| `trailing_stop` | Trailing stop — stop price dynamically trails market |

### Futures Side

| Value | Description |
|-------|-------------|
| `buy` | Long / buy |
| `sell` | Short / sell |

### Futures Trigger Signals (`triggerSignal`)

Used with `stp` and `take_profit` order types:

| Value | Description |
|-------|-------------|
| `mark` | Mark price = index price + 30-second EMA of future's basis |
| `index` | Index price (default for `take_profit`) |
| `last` | Last executed trade price |

### Futures Trailing Stop Params

| Param | Description |
|-------|-------------|
| `trailingStopDeviationUnit` | Unit for trailing deviation (e.g. `"PERCENT"`, `"QUOTE_CURRENCY"`) |
| `trailingStopMaxDeviation` | Maximum deviation amount |

### Futures Reduce-Only

- `reduceOnly` (bool) — if `true`, order will only reduce existing positions, not open new ones

---

## 2. ORDER MANAGEMENT

### Spot — Base URL: `https://api.kraken.com/0`

All Spot private endpoints use **POST** (even for queries). Authentication via HMAC-SHA512 signature.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `POST /0/private/AddOrder` | POST | Place a new order |
| `POST /0/private/AddOrderBatch` | POST | Place up to 15 orders in one request |
| `POST /0/private/EditOrder` | POST | Modify an existing open order (deprecated — prefer AmendOrder via WebSocket v2) |
| `POST /0/private/CancelOrder` | POST | Cancel open order by txid, userref, or cl_ord_id |
| `POST /0/private/CancelAll` | POST | Cancel ALL open orders |
| `POST /0/private/CancelAllOrdersAfter` | POST | Dead man's switch — cancel all after timeout |
| `POST /0/private/QueryOrders` | POST | Get info on specific orders by txid |
| `POST /0/private/OpenOrders` | POST | Get all open orders |
| `POST /0/private/ClosedOrders` | POST | Get closed/cancelled order history (50 per page) |
| `POST /0/private/TradesHistory` | POST | Get executed trade history |

### Futures — Base URL: `https://futures.kraken.com/derivatives/api/v3`

Futures endpoints use standard REST (GET for queries, POST for mutations). Different auth mechanism.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `POST /derivatives/api/v3/sendorder` | POST | Place a new Futures order |
| `POST /derivatives/api/v3/editorder` | POST | Modify an existing Futures order |
| `POST /derivatives/api/v3/cancelorder` | POST | Cancel a specific Futures order |
| `POST /derivatives/api/v3/cancelallorders` | POST | Cancel all Futures orders (optionally for a symbol) |
| `POST /derivatives/api/v3/batchorder` | POST | Batch place/cancel Futures orders |
| `GET /derivatives/api/v3/openorders` | GET | Get all open Futures orders |
| `GET /derivatives/api/v3/orders` | GET | Get specific orders by ID |
| `GET /derivatives/api/v3/fills` | GET | Get trade fill history |

---

## 3. ADD ORDER (Spot) — Full Parameter Reference

**Endpoint:** `POST /0/private/AddOrder`

**Content-Type:** `application/x-www-form-urlencoded` OR `application/json`

**Required Permission:** `Orders and trades — Create & modify orders`

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `nonce` | integer | Yes | Monotonically increasing 64-bit unsigned int; recommend ms timestamp |
| `pair` | string | Yes | Asset pair (e.g. `XBTUSD`, `ETHUSD`) |
| `type` | string | Yes | `buy` or `sell` |
| `ordertype` | string | Yes | See order types table above |
| `price` | decimal | Conditional | Limit price (for `limit`, `stop-loss-limit`, `take-profit-limit`); stop price (for `stop-loss`, `take-profit`, `trailing-stop`); trailing offset (for `trailing-stop`, `trailing-stop-limit`) |
| `price2` | decimal | Conditional | Secondary price — limit price for `stop-loss-limit` and `take-profit-limit`; stop offset for `trailing-stop-limit` |
| `volume` | decimal | Yes | Order quantity (in base asset unless `viqc` flag set) |
| `displayvol` | decimal | No | Iceberg order visible quantity (must be less than `volume`) |
| `leverage` | string | No | Margin leverage (e.g. `"2"`, `"5"`); omit for non-margin |
| `reduce_only` | boolean | No | Reduce-only for margin positions |
| `stptype` | string | No | Self-trade prevention type: `cancel-newest`, `cancel-oldest`, `cancel-both` |
| `oflags` | string | No | Comma-delimited order flags: `post`, `fcib`, `fciq`, `nompp`, `viqc` |
| `timeinforce` | string | No | `GTC` (default), `IOC`, `GTD` |
| `starttm` | string | No | Scheduled start time: `0` = now; `+<n>` = relative offset; Unix timestamp |
| `expiretm` | string | No | Expiry time (for GTD): same format as `starttm` |
| `close[ordertype]` | string | No | Conditional close order type |
| `close[price]` | decimal | Conditional | Conditional close trigger/limit price |
| `close[price2]` | decimal | Conditional | Conditional close secondary price |
| `deadline` | string | No | RFC3339 timestamp deadline for order matching (prevents slippage) |
| `validate` | boolean | No | If `true`, validate only — do not submit order |
| `userref` | integer | No | User-defined integer order reference |
| `cl_ord_id` | string | No | Client order ID (alphanumeric string) |

### Response JSON

```json
{
  "error": [],
  "result": {
    "descr": {
      "order": "buy 1.25000000 XBTUSD @ limit 27500.0",
      "close": "close position @ stop loss 25000.0 -> limit 24000.0"
    },
    "txid": ["OWFYJG-DJUOO-F5BISK"]
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `error` | array | Empty on success; error strings on failure |
| `result.descr.order` | string | Human-readable order description |
| `result.descr.close` | string | Close order description (if conditional close specified) |
| `result.txid` | array[string] | Transaction IDs assigned by Kraken (may be empty if `validate=true`) |

**Note on txid:** In rare cases (exchange overload), the response may be `result: {}` with an empty `txid`. Clients must handle this case.

---

## 4. ADD ORDER BATCH (Spot)

**Endpoint:** `POST /0/private/AddOrderBatch`

- Supports **up to 15 orders** per request
- All orders share the same `pair`
- Response contains an array of per-order results/errors

### Request Structure

```json
{
  "nonce": "1616492376594",
  "pair": "XBTUSD",
  "orders": [
    {
      "type": "buy",
      "ordertype": "limit",
      "price": "27500",
      "volume": "0.5"
    },
    {
      "type": "sell",
      "ordertype": "limit",
      "price": "30000",
      "volume": "0.5",
      "oflags": "post"
    }
  ],
  "deadline": "2023-01-01T00:00:00Z"
}
```

### Response

```json
{
  "error": [],
  "result": {
    "orders": [
      {
        "txid": "ABCDE-FGHIJ-KLMNO",
        "descr": { "order": "buy 0.50000000 XBTUSD @ limit 27500.0" },
        "close": ""
      },
      {
        "txid": "PQRST-UVWXY-ZABCD",
        "descr": { "order": "sell 0.50000000 XBTUSD @ limit 30000.0" },
        "close": ""
      }
    ]
  }
}
```

---

## 5. EDIT ORDER (Spot)

**Endpoint:** `POST /0/private/EditOrder`

**Deprecated** — Kraken recommends using the newer `AmendOrder` via WebSocket v2 which:
- Preserves queue position
- Supports `cl_ord_id`
- Has better performance

### EditOrder Limitations
- Triggered stop-loss/take-profit orders: NOT supported
- Orders with conditional close terms: NOT supported
- Cannot reduce volume below already-executed amount
- Does NOT preserve queue position
- Execution history not transferred to amended order

### EditOrder Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `nonce` | integer | Monotonically increasing nonce |
| `txid` | string | Transaction ID of order to edit |
| `volume` | decimal | New total volume |
| `displayvol` | decimal | New iceberg visible volume |
| `price` | decimal | New limit/stop price |
| `price2` | decimal | New secondary price |
| `oflags` | string | New order flags |
| `deadline` | string | New deadline timestamp |
| `cancel_response` | boolean | If true, cancel-and-replace semantics |
| `validate` | boolean | Validate only, do not submit |
| `userref` | integer | New user reference |

### EditOrder Response

```json
{
  "error": [],
  "result": {
    "descr": {
      "order": "buy 1.25000000 XBTUSD @ limit 27000.0"
    },
    "txid": "NEWORDER-TXID-12345",
    "originaltxid": "OWFYJG-DJUOO-F5BISK",
    "volume": "1.25000000",
    "price": "27000.0",
    "orders_cancelled": 1,
    "newuserref": 0
  }
}
```

---

## 6. CANCEL ORDER (Spot)

**Endpoint:** `POST /0/private/CancelOrder`

Accepts one of `txid`, `userref`, or `cl_ord_id`.

### Response

```json
{
  "error": [],
  "result": {
    "count": 1,
    "pending": false
  }
}
```

| Field | Description |
|-------|-------------|
| `count` | Number of orders cancelled |
| `pending` | Whether a cancel was queued but not yet processed |

---

## 7. CANCEL ALL ORDERS AFTER — Dead Man's Switch (Spot)

**Endpoint:** `POST /0/private/CancelAllOrdersAfter`

**Required Permission:** `Orders and trades — Create & modify orders` OR `Cancel & close orders`

A countdown timer that automatically cancels ALL open orders if not reset in time.

### How It Works
1. Client sends request with `timeout` in seconds — starts the countdown
2. Client must call endpoint again before timer expires to reset it
3. Sending `timeout=0` disables the timer
4. If timer expires, all open orders are immediately cancelled
5. Timer remains inactive until explicitly reactivated

### Recommended Usage
- Call every 15-30 seconds with `timeout=60`
- Disable before scheduled exchange maintenance windows

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `nonce` | integer | Monotonically increasing nonce |
| `timeout` | integer | Seconds until auto-cancel; `0` = disable timer |

### Response

```json
{
  "error": [],
  "result": {
    "currentTime": "2023-03-24T17:41:56Z",
    "triggerTime": "2023-03-24T17:42:56Z"
  }
}
```

| Field | Description |
|-------|-------------|
| `currentTime` | Server timestamp |
| `triggerTime` | Timestamp when orders will be cancelled (null if disabled) |

---

## 8. OPEN ORDERS / QUERY ORDERS (Spot)

**OpenOrders:** `POST /0/private/OpenOrders`

**QueryOrders:** `POST /0/private/QueryOrders` — takes `txid` param

### Order Object Fields

```json
{
  "ABCDE-FGHIJ-KLMNO": {
    "refid": null,
    "userref": 0,
    "cl_ord_id": "my-order-1",
    "status": "open",
    "opentm": 1688992000.123,
    "starttm": 0,
    "expiretm": 0,
    "descr": {
      "pair": "XBTUSD",
      "type": "buy",
      "ordertype": "limit",
      "price": "27500.00",
      "price2": "0",
      "leverage": "none",
      "order": "buy 1.25000000 XBTUSD @ limit 27500.0",
      "close": ""
    },
    "vol": "1.25000000",
    "vol_exec": "0.50000000",
    "cost": "13750.000",
    "fee": "27.500",
    "price": "27500.0",
    "stopprice": "0.00000",
    "limitprice": "0.00000",
    "misc": "",
    "oflags": "fciq",
    "timeinforce": "GTC",
    "amended": false
  }
}
```

| Field | Description |
|-------|-------------|
| `refid` | Originating order txid (if this is a close order) |
| `userref` | User-defined integer reference |
| `cl_ord_id` | Client order ID |
| `status` | `open`, `closed`, `canceled`, `expired`, `pending` |
| `opentm` | Unix timestamp when order was placed |
| `expiretm` | Unix expiry timestamp (0 = none) |
| `descr.pair` | Asset pair |
| `descr.type` | `buy` or `sell` |
| `descr.ordertype` | Order type string |
| `descr.price` | Primary price |
| `descr.price2` | Secondary price |
| `descr.leverage` | Leverage or `"none"` |
| `descr.order` | Human-readable order description |
| `vol` | Total order volume |
| `vol_exec` | Volume already executed |
| `cost` | Total cost/proceeds of executed fills |
| `fee` | Fees paid |
| `price` | Average execution price |
| `stopprice` | Stop price (for stop orders) |
| `limitprice` | Limit price (for stop-limit orders) |
| `oflags` | Order flags |
| `timeinforce` | Time-in-force |
| `amended` | Whether order has been amended |

---

## 9. SEND ORDER (Futures) — Full Parameter Reference

**Endpoint:** `POST /derivatives/api/v3/sendorder`

**Content-Type:** `application/x-www-form-urlencoded` (default) or JSON

### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `orderType` | string | Yes | `lmt`, `post`, `mkt`, `ioc`, `stp`, `take_profit`, `trailing_stop` |
| `symbol` | string | Yes | Futures symbol (e.g. `PI_XBTUSD`, `PF_ETHUSD`) |
| `side` | string | Yes | `buy` or `sell` |
| `size` | decimal | Yes | Order quantity in contracts |
| `limitPrice` | decimal | Conditional | Required for `lmt`, `post`, `ioc`, `stp`, `take_profit` |
| `stopPrice` | decimal | Conditional | Required for `stp`, `take_profit`, `trailing_stop` |
| `triggerSignal` | string | No | `mark`, `index`, `last` (for `stp`, `take_profit`) |
| `cliOrdId` | string | No | Client-defined order ID (max 100 chars) |
| `reduceOnly` | boolean | No | If true, only reduces existing position |
| `trailingStopDeviationUnit` | string | Conditional | Required for `trailing_stop` |
| `trailingStopMaxDeviation` | string | Conditional | Required for `trailing_stop` |

### Response JSON

```json
{
  "result": "success",
  "serverTime": "2024-01-15T10:30:00.000Z",
  "sendStatus": {
    "order_id": "c18f0c17-9971-40e6-8e5b-10df05d422f0",
    "cliOrdId": "my-futures-order-1",
    "receivedTime": "2024-01-15T10:30:00.000Z",
    "status": "placed",
    "orderEvents": [
      {
        "executionId": "some-exec-id",
        "price": 27500.0,
        "amount": 1000,
        "orderPriorExecution": {
          "orderId": "c18f0c17-9971-40e6-8e5b-10df05d422f0",
          "symbol": "PI_XBTUSD",
          "side": "buy",
          "orderType": "lmt",
          "limitPrice": 27500.0,
          "unfilledSize": 1000,
          "receivedTime": "2024-01-15T10:30:00.000Z",
          "status": "untouched",
          "filledSize": 0,
          "reduceOnly": false
        },
        "type": "EXECUTION"
      }
    ]
  }
}
```

| Field | Description |
|-------|-------------|
| `result` | `"success"` means request was received and assessed; check `sendStatus.status` for actual state |
| `serverTime` | Server timestamp (ISO 8601) |
| `sendStatus.order_id` | UUID assigned by Kraken Futures |
| `sendStatus.cliOrdId` | Client order ID if provided |
| `sendStatus.receivedTime` | Timestamp when order was received |
| `sendStatus.status` | `placed`, `insufficientAvailableFunds`, `invalidOrderType`, `tooManySmallOrders`, etc. |
| `sendStatus.orderEvents` | Array of execution events (for fills) |

**Important:** `result: "success"` only means the API call succeeded. It does NOT mean the order was placed — check `sendStatus.status`.

---

## 10. FUTURES ORDER MANAGEMENT

### Edit Order (Futures)

**Endpoint:** `POST /derivatives/api/v3/editorder`

| Parameter | Type | Description |
|-----------|------|-------------|
| `orderId` | string | Kraken-assigned order UUID |
| `cliOrdId` | string | Client order ID (alternative to orderId) |
| `limitPrice` | decimal | New limit price |
| `size` | decimal | New size |
| `stopPrice` | decimal | New stop price |

### Cancel Order (Futures)

**Endpoint:** `POST /derivatives/api/v3/cancelorder`

| Parameter | Type | Description |
|-----------|------|-------------|
| `order_id` | string | Kraken-assigned order UUID |
| `cliOrdId` | string | Client order ID (alternative) |

**Response:**

```json
{
  "result": "success",
  "serverTime": "2024-01-15T10:30:00.000Z",
  "cancelStatus": {
    "status": "cancelled",
    "order_id": "c18f0c17-9971-40e6-8e5b-10df05d422f0",
    "receivedTime": "2024-01-15T10:30:00.000Z"
  }
}
```

### Cancel All Orders (Futures)

**Endpoint:** `POST /derivatives/api/v3/cancelallorders`

Optional `symbol` param to limit to a single contract. Response includes count of cancelled orders.

### Batch Order (Futures)

**Endpoint:** `POST /derivatives/api/v3/batchorder`

Accepts a `batchOrder` array of actions:

```json
{
  "batchOrder": [
    {
      "order": "send",
      "orderType": "lmt",
      "symbol": "PI_XBTUSD",
      "side": "buy",
      "size": 1000,
      "limitPrice": 27000
    },
    {
      "order": "cancel",
      "order_id": "existing-order-uuid"
    }
  ]
}
```

### Get Open Orders (Futures)

**Endpoint:** `GET /derivatives/api/v3/openorders`

No parameters required.

### Get Fills (Futures)

**Endpoint:** `GET /derivatives/api/v3/fills`

| Parameter | Type | Description |
|-----------|------|-------------|
| `lastFillTime` | string | ISO 8601 timestamp — return fills after this time |

**Fill Response Fields:**

```json
{
  "result": "success",
  "fills": [
    {
      "fill_id": "uuid-fill-id",
      "order_id": "uuid-order-id",
      "symbol": "PI_XBTUSD",
      "side": "buy",
      "size": 1000,
      "price": 27500.0,
      "fillTime": "2024-01-15T10:30:00.000Z",
      "fillType": "taker"
    }
  ]
}
```

---

## 11. TP/SL & CONDITIONAL ORDERS

### Spot Conditional Close (OTO — One-Triggers-Other)

Attach a close order at the time of opening:

```
close[ordertype] = "stop-loss-limit"
close[price]     = "25000"    # stop trigger price
close[price2]    = "24800"    # limit price after trigger
```

When the opening order fills, a closing order is automatically created as a linked order.

### Futures TP/SL

- Use `stp` orderType with `stopPrice` and `triggerSignal` for stop-loss
- Use `take_profit` orderType with `limitPrice`, `stopPrice`, and `triggerSignal` for take-profit
- `reduceOnly = true` ensures these only close positions

### Dead Man's Switch (Spot Only)

- `CancelAllOrdersAfter` — unique to Kraken Spot
- Fires after a configurable timeout (in seconds)
- Must be continuously reset to keep orders alive
- Futures equivalent: `dead_mans_switch` endpoint (same concept, different endpoint)

---

## 12. ALGO ORDERS

- **Native TWAP:** Not available
- **Grid trading:** Not available natively via API
- **Trailing Stop:** Available natively for both Spot and Futures
- **Iceberg orders:** Available on Spot via `displayvol` parameter
- **Conditional OTO:** Available on Spot via `close[]` params

---

## 13. ORDER ID FORMAT

### Spot (`txid`)
- Format: `AAAAA-BBBBB-CCCCC` (5-char groups separated by hyphens)
- Example: `OWFYJG-DJUOO-F5BISK`
- Also supports `userref` (integer) and `cl_ord_id` (string) as alternatives

### Futures (`order_id`)
- Format: UUID v4
- Example: `c18f0c17-9971-40e6-8e5b-10df05d422f0`
- Also supports `cliOrdId` (user-defined string, max 100 chars)

---

## 14. SPOT PRIVATE ENDPOINT STRUCTURE

All Spot private endpoints follow this pattern:
- **URL:** `https://api.kraken.com/0/private/<EndpointName>`
- **Method:** POST (always, even for queries)
- **Headers:**
  - `API-Key: <public_key>`
  - `API-Sign: <computed_signature>`
- **Body:** `nonce=<value>&<other_params>` (form-encoded) or JSON

---

## Sources

- [Add Order | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/add-order/)
- [Add Order Batch | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/add-order-batch/)
- [Edit Order | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/edit-order/)
- [Cancel All Orders After | Kraken API Center](https://docs.kraken.com/api/docs/rest-api/cancel-all-orders-after/)
- [Send Order | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/send-order/)
- [Batch Order Management | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/send-batch-order/)
- [Order Management | Kraken API Center](https://docs.kraken.com/api/docs/futures-api/trading/order-management/)
- [Futures REST | Kraken API Center](https://docs.kraken.com/api/docs/guides/futures-rest/)
- [Amend Order WebSocket v2 | Kraken API Center](https://docs.kraken.com/api/docs/websocket-v2/amend_order/)
- [Derivatives Order Types | Kraken Support](https://support.kraken.com/articles/360031471211-derivatives-order-types)
- [Examples of Placing Orders | Kraken Support](https://support.kraken.com/articles/360000920786-examples-of-placing-orders-with-different-parameters)
- [Futures REST Python SDK Docs](https://python-kraken-sdk.readthedocs.io/en/v2.0.0/src/futures/rest.html)
