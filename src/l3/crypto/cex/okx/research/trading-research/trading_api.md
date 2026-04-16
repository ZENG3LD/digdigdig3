# OKX Trading API — V5 Complete Reference

Base URL (production): `https://www.okx.com`
Base URL (demo): `https://www.okx.com` (same host, add header `x-simulated-trading: 1`)

---

## 1. ORDER TYPES

OKX uses a unified endpoint for all instrument types. The `instType` is determined by the `instId` format and `tdMode` combination.

### Instrument Types (`instType`)

| Value | Description |
|-------|-------------|
| `SPOT` | Spot trading (e.g. `BTC-USDT`) |
| `MARGIN` | Margin trading (e.g. `BTC-USDT` with tdMode=isolated/cross) |
| `SWAP` | Perpetual swaps (e.g. `BTC-USDT-SWAP`) |
| `FUTURES` | Delivery futures (e.g. `BTC-USDT-231229`) |
| `OPTION` | Options (e.g. `BTC-USD-231229-30000-C`) |

### Trade Mode (`tdMode`)

| Value | Applies To | Description |
|-------|-----------|-------------|
| `cash` | SPOT | No margin, simple buy/sell |
| `isolated` | MARGIN, SWAP, FUTURES, OPTION | Per-position isolated margin |
| `cross` | MARGIN, SWAP, FUTURES, OPTION | Shared cross margin pool |

### Regular Order Types (`ordType`)

| Value | Description |
|-------|-------------|
| `market` | Market order — immediate execution at best available price |
| `limit` | Limit order — resting at specified price |
| `post_only` | Maker-only limit; rejected if would fill immediately |
| `fok` | Fill or Kill — full fill or full cancel |
| `ioc` | Immediate or Cancel — partial fills allowed, remainder cancelled |
| `optimal_limit_ioc` | Market order with price ceiling (for derivatives) |
| `mmp` | Market maker protection order |
| `mmp_and_post_only` | Combined MMP + post_only |

### Algo Order Types (separate endpoint system)

| Value | Description |
|-------|-------------|
| `conditional` | Single TP or SL order attached to a position |
| `oco` | One-Cancels-Other — both TP and SL defined together |
| `trigger` | Trigger order — fires when trigger price is hit |
| `move_order_stop` | Trailing stop order |
| `iceberg` | Iceberg order — large order split into small visible slices |
| `twap` | Time-Weighted Average Price order |

### Trigger Price Types (`tpTriggerPxType` / `slTriggerPxType`)

| Value | Description |
|-------|-------------|
| `last` | Last traded price (default) |
| `mark` | Mark price (recommended for derivatives) |
| `index` | Index price |

---

## 2. ORDER MANAGEMENT

### POST /api/v5/trade/order — Place Order

**Auth required**: Trade permission
**Rate limit**: Per Instrument ID level (Options: per Instrument Family)
**Sub-account limit**: Max 1000 order requests/2s across new + amend

**Request body:**

```json
{
  "instId":   "BTC-USDT-SWAP",
  "tdMode":   "cross",
  "side":     "buy",
  "ordType":  "limit",
  "px":       "50912.4",
  "sz":       "1",
  "posSide":  "long",
  "clOrdId":  "myorder_001",
  "tag":      "strategyA",
  "reduceOnly": false,
  "tpTriggerPx": "",
  "tpOrdPx":     "",
  "slTriggerPx": "",
  "slOrdPx":     "",
  "tpTriggerPxType": "last",
  "slTriggerPxType": "last",
  "tgtCcy":   "",
  "stpId":    "",
  "stpMode":  "cancel_maker"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID (e.g. `BTC-USDT-SWAP`) |
| `tdMode` | String | Yes | Trade mode: `cash`, `isolated`, `cross` |
| `side` | String | Yes | `buy` or `sell` |
| `ordType` | String | Yes | Order type (see table above) |
| `sz` | String | Yes | Order size (contracts for derivatives, base currency for spot) |
| `px` | String | Conditional | Price; required for `limit`, `post_only`, `fok`, `ioc` |
| `posSide` | String | Conditional | `long`, `short`, `net`; required when posMode=long_short_mode |
| `ccy` | String | No | Margin currency; only for MARGIN cross orders |
| `clOrdId` | String | No | Client order ID (max 32 chars, alphanumeric + `-_`) |
| `tag` | String | No | Order tag (max 16 chars) |
| `tgtCcy` | String | No | Target currency for market orders: `base_ccy` or `quote_ccy` |
| `reduceOnly` | Boolean | No | `true` = reduce position only; for SWAP/FUTURES/OPTION |
| `tpTriggerPx` | String | No | Take-profit trigger price |
| `tpOrdPx` | String | No | Take-profit order price; `-1` = market order on trigger |
| `slTriggerPx` | String | No | Stop-loss trigger price |
| `slOrdPx` | String | No | Stop-loss order price; `-1` = market order on trigger |
| `tpTriggerPxType` | String | No | TP trigger price type: `last`, `mark`, `index` |
| `slTriggerPxType` | String | No | SL trigger price type: `last`, `mark`, `index` |
| `stpId` | String | No | Self-trade prevention group ID |
| `stpMode` | String | No | STP mode: `cancel_maker`, `cancel_taker`, `cancel_both` |
| `expTime` | String | No | Request expiry (Unix ms timestamp) — from HTTP header |
| `quickMgnType` | String | No | Quick margin type for isolated margin: `manual`, `auto_borrow`, `auto_repay` |

**Response:**

```json
{
  "code": "0",
  "msg":  "",
  "data": [
    {
      "clOrdId": "myorder_001",
      "ordId":   "312269865356374016",
      "tag":     "strategyA",
      "sCode":   "0",
      "sMsg":    "",
      "ts":      "1695190491421"
    }
  ],
  "inTime":  "1695190491408",
  "outTime": "1695190491423"
}
```

**Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `ordId` | String | Exchange-assigned order ID |
| `clOrdId` | String | Client order ID (echoed if provided) |
| `tag` | String | Order tag (echoed) |
| `sCode` | String | Status code: `0` = success, non-zero = error |
| `sMsg` | String | Status message (error description if failed) |
| `ts` | String | Timestamp when order was placed (Unix ms) |

---

### POST /api/v5/trade/amend-order — Amend Order

**Auth required**: Trade permission
**Rate limit**: Independent from place/cancel; per Instrument ID level
**Note**: Successful response only confirms request accepted; monitor WebSocket orders channel for final result

**Request body:**

```json
{
  "instId":   "BTC-USDT-SWAP",
  "ordId":    "312269865356374016",
  "clOrdId":  "myorder_001",
  "reqId":    "amend_req_001",
  "newSz":    "2",
  "newPx":    "50500.0",
  "newTpTriggerPx": "",
  "newTpOrdPx":     "",
  "newSlTriggerPx": "",
  "newSlOrdPx":     ""
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `ordId` | String | Conditional | Order ID; one of `ordId`/`clOrdId` required |
| `clOrdId` | String | Conditional | Client order ID; one of `ordId`/`clOrdId` required |
| `reqId` | String | No | Client request ID for tracking amend (echoed in response) |
| `newSz` | String | No | New order size |
| `newPx` | String | No | New order price |
| `newTpTriggerPx` | String | No | New TP trigger price |
| `newTpOrdPx` | String | No | New TP order price |
| `newSlTriggerPx` | String | No | New SL trigger price |
| `newSlOrdPx` | String | No | New SL order price |
| `newTpTriggerPxType` | String | No | New TP trigger price type |
| `newSlTriggerPxType` | String | No | New SL trigger price type |

**Response:**

```json
{
  "code": "0",
  "msg":  "",
  "data": [
    {
      "clOrdId": "myorder_001",
      "ordId":   "312269865356374016",
      "reqId":   "amend_req_001",
      "sCode":   "0",
      "sMsg":    ""
    }
  ]
}
```

---

### POST /api/v5/trade/cancel-order — Cancel Order

**Auth required**: Trade permission
**Rate limit**: Per Instrument ID level; independent from place/amend

**Request body:**

```json
{
  "instId":  "BTC-USDT-SWAP",
  "ordId":   "312269865356374016",
  "clOrdId": "myorder_001"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `ordId` | String | Conditional | One of `ordId`/`clOrdId` required |
| `clOrdId` | String | Conditional | One of `ordId`/`clOrdId` required |

**Response:**

```json
{
  "code": "0",
  "msg":  "",
  "data": [
    {
      "clOrdId": "myorder_001",
      "ordId":   "312269865356374016",
      "sCode":   "0",
      "sMsg":    ""
    }
  ]
}
```

---

### GET /api/v5/trade/order — Get Order Details

**Auth required**: Read permission
**Rate limit**: 60 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `ordId` | String | Conditional | One of `ordId`/`clOrdId` required |
| `clOrdId` | String | Conditional | One of `ordId`/`clOrdId` required |

**Response — Full Order Object:**

```json
{
  "code": "0",
  "data": [
    {
      "instType":    "SWAP",
      "instId":      "BTC-USDT-SWAP",
      "ccy":         "",
      "ordId":       "312269865356374016",
      "clOrdId":     "myorder_001",
      "tag":         "strategyA",
      "px":          "50912.4",
      "sz":          "1",
      "pnl":         "0",
      "ordType":     "limit",
      "side":        "buy",
      "posSide":     "long",
      "tdMode":      "cross",
      "accFillSz":   "0",
      "fillPx":      "",
      "tradeId":     "",
      "fillSz":      "0",
      "fillTime":    "",
      "avgPx":       "",
      "state":       "live",
      "lever":       "10",
      "tpTriggerPx": "",
      "tpOrdPx":     "",
      "slTriggerPx": "",
      "slOrdPx":     "",
      "tpTriggerPxType": "last",
      "slTriggerPxType": "last",
      "feeCcy":      "USDT",
      "fee":         "0",
      "rebateCcy":   "USDT",
      "rebate":      "0",
      "category":    "normal",
      "uTime":       "1695190491421",
      "cTime":       "1695190491421",
      "reqId":       "",
      "source":      "",
      "cancelSource": "",
      "reduceOnly":  "false",
      "quickMgnType": "",
      "stpId":       "",
      "stpMode":     "cancel_maker",
      "attachAlgoOrds": []
    }
  ]
}
```

**Order state values:**

| Value | Description |
|-------|-------------|
| `live` | Active resting order |
| `partially_filled` | Partially filled, still active |
| `filled` | Completely filled |
| `canceled` | Canceled by user or system |

---

### GET /api/v5/trade/orders-pending — Open Orders

**Auth required**: Read permission
**Rate limit**: 60 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | No | Filter by: `SPOT`, `MARGIN`, `SWAP`, `FUTURES`, `OPTION` |
| `uly` | String | No | Underlying (for FUTURES/SWAP/OPTION, e.g. `BTC-USD`) |
| `instFamily` | String | No | Instrument family |
| `instId` | String | No | Specific instrument ID |
| `ordType` | String | No | Filter by order type |
| `state` | String | No | `live` or `partially_filled` |
| `after` | String | No | Pagination: orders before this ordId |
| `before` | String | No | Pagination: orders after this ordId |
| `limit` | String | No | Number of results, max 100 (default 100) |

Returns array of full order objects (same schema as GET /api/v5/trade/order).

---

### GET /api/v5/trade/orders-history — Order History (last 7 days)

**Auth required**: Read permission
**Rate limit**: 40 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | Yes | Instrument type |
| `uly` | String | No | Underlying |
| `instFamily` | String | No | Instrument family |
| `instId` | String | No | Specific instrument |
| `ordType` | String | No | Order type filter |
| `state` | String | No | `filled`, `canceled`, `partially_filled` |
| `category` | String | No | `twap`, `adl`, `full_liquidation`, `partial_liquidation`, `delivery`, `ddh` |
| `after` | String | No | Pagination cursor |
| `before` | String | No | Pagination cursor |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 (default 100) |

---

### GET /api/v5/trade/orders-history-archive — Order History (last 3 months)

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

Same parameters as `orders-history`. Returns data up to 3 months old.

---

### GET /api/v5/trade/fills — Recent Fills (last 3 days)

**Auth required**: Read permission
**Rate limit**: 60 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | No | Instrument type filter |
| `uly` | String | No | Underlying |
| `instFamily` | String | No | Instrument family |
| `instId` | String | No | Specific instrument |
| `ordId` | String | No | Filter by order ID |
| `after` | String | No | Pagination: bills before this billId |
| `before` | String | No | Pagination: bills after this billId |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 (default 100) |

**Fill/Trade Object:**

```json
{
  "instType":  "SWAP",
  "instId":    "BTC-USDT-SWAP",
  "tradeId":   "123456789",
  "ordId":     "312269865356374016",
  "clOrdId":   "myorder_001",
  "billId":    "987654321",
  "tag":       "",
  "fillPx":    "50900.0",
  "fillSz":    "1",
  "fillIdxPx": "50890.0",
  "fillPnl":   "0",
  "side":      "buy",
  "posSide":   "long",
  "execType":  "T",
  "feeCcy":    "USDT",
  "fee":       "-0.0254",
  "ts":        "1695190491421"
}
```

**Fill fields:**

| Field | Type | Description |
|-------|------|-------------|
| `tradeId` | String | Exchange trade ID |
| `billId` | String | Bill/ledger entry ID |
| `fillPx` | String | Fill price |
| `fillSz` | String | Fill size |
| `fillIdxPx` | String | Index price at fill time |
| `fillPnl` | String | Realized PnL from this fill |
| `execType` | String | `T` = taker, `M` = maker |
| `fee` | String | Fee charged (negative = cost, positive = rebate) |
| `feeCcy` | String | Fee currency |

---

### GET /api/v5/trade/fills-history — Historical Fills (last 3 months)

**Auth required**: Read permission
**Rate limit**: 10 requests/2s per User ID

Same parameters as `fills`. Returns data up to 3 months old.

---

## 3. TP/SL & CONDITIONAL (ALGO) ORDERS

OKX maintains a completely separate algo order system with its own endpoints. Algo orders persist independently of regular orders.

**Algo order limits:**
- TP/SL (conditional): Max 100 per instrument
- Trigger: Max 500 per account
- Trailing stop (move_order_stop): Max 50 per account
- Iceberg: Max 100 per account
- TWAP: Max 20 per account

### POST /api/v5/trade/order-algo — Place Algo Order

**Auth required**: Trade permission
**Rate limit**: 20 requests/2s per User ID

**Request body — Conditional (TP/SL):**

```json
{
  "instId":    "BTC-USDT",
  "tdMode":    "cash",
  "side":      "buy",
  "ordType":   "conditional",
  "sz":        "2",
  "tpTriggerPx": "15",
  "tpOrdPx":     "18",
  "tpTriggerPxType": "last",
  "slTriggerPx": "",
  "slOrdPx":     "",
  "slTriggerPxType": "last"
}
```

**Request body — OCO:**

```json
{
  "instId":    "ETH-USDT-SWAP",
  "tdMode":    "cross",
  "side":      "sell",
  "posSide":   "long",
  "ordType":   "oco",
  "sz":        "1",
  "tpTriggerPx":     "1500",
  "tpOrdPx":         "-1",
  "tpTriggerPxType": "last",
  "slTriggerPx":     "1200",
  "slOrdPx":         "-1",
  "slTriggerPxType": "mark"
}
```

**Request body — Trigger Order:**

```json
{
  "instId":       "BTC-USDT-SWAP",
  "tdMode":       "cross",
  "side":         "buy",
  "ordType":      "trigger",
  "sz":           "1",
  "triggerPx":    "45000",
  "orderPx":      "45500",
  "triggerPxType": "last"
}
```

**Request body — Trailing Stop:**

```json
{
  "instId":        "BTC-USDT-SWAP",
  "tdMode":        "cross",
  "side":          "sell",
  "posSide":       "long",
  "ordType":       "move_order_stop",
  "sz":            "1",
  "activePx":      "52000",
  "callbackRatio": "0.01"
}
```

**Parameters (algo orders):**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `tdMode` | String | Yes | Trade mode |
| `side` | String | Yes | `buy` or `sell` |
| `ordType` | String | Yes | Algo type |
| `sz` | String | Yes | Order size |
| `posSide` | String | Conditional | Required in long_short_mode |
| `clOrdId` | String | No | Client algo order ID |
| `tag` | String | No | Tag |
| `tpTriggerPx` | String | Conditional | TP trigger price |
| `tpOrdPx` | String | Conditional | TP order price (`-1` = market) |
| `tpTriggerPxType` | String | No | `last`, `mark`, `index` |
| `slTriggerPx` | String | Conditional | SL trigger price |
| `slOrdPx` | String | Conditional | SL order price (`-1` = market) |
| `slTriggerPxType` | String | No | `last`, `mark`, `index` |
| `triggerPx` | String | Conditional | Trigger price (for `trigger` type) |
| `orderPx` | String | Conditional | Order price after trigger (`-1` = market) |
| `triggerPxType` | String | No | Trigger price type |
| `callbackRatio` | String | Conditional | Trailing ratio (for `move_order_stop`) |
| `callbackSpread` | String | Conditional | Trailing spread (alternative to ratio) |
| `activePx` | String | No | Activation price for trailing stop |
| `pxVar` | String | Conditional | Price variance for iceberg |
| `szLimit` | String | Conditional | Single order size for iceberg/TWAP |
| `pxSpread` | String | Conditional | Price spread for iceberg |
| `timeInterval` | String | Conditional | Time interval for TWAP (in seconds) |

**Response:**

```json
{
  "code": "0",
  "msg":  "",
  "data": [
    {
      "clOrdId": "algo_001",
      "algoId":  "1234567890",
      "sCode":   "0",
      "sMsg":    ""
    }
  ]
}
```

---

### POST /api/v5/trade/cancel-algos — Cancel Algo Orders

**Auth required**: Trade permission
**Rate limit**: 20 requests/2s per User ID

**Request body (array, up to 10):**

```json
[
  {
    "algoId":  "1234567890",
    "instId":  "BTC-USDT-SWAP"
  }
]
```

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "algoId":  "1234567890",
      "clOrdId": "",
      "sCode":   "0",
      "sMsg":    ""
    }
  ]
}
```

---

### POST /api/v5/trade/amend-algos — Amend Algo Order

**Auth required**: Trade permission
**Rate limit**: 20 requests/2s per User ID

**Request body:**

```json
{
  "instId":           "BTC-USDT-SWAP",
  "algoId":           "1234567890",
  "newSz":            "2",
  "newTpTriggerPx":   "55000",
  "newTpOrdPx":       "-1",
  "newSlTriggerPx":   "44000",
  "newSlOrdPx":       "-1"
}
```

---

### GET /api/v5/trade/orders-algo-pending — Pending Algo Orders

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `algoId` | String | No | Specific algo order ID |
| `clOrdId` | String | No | Client algo order ID |
| `instType` | String | No | Instrument type filter |
| `instId` | String | No | Instrument ID filter |
| `ordType` | String | Yes | Algo type filter |
| `after` | String | No | Pagination |
| `before` | String | No | Pagination |
| `limit` | String | No | Max 100 |

---

### GET /api/v5/trade/orders-algo-history — Algo Order History

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ordType` | String | Yes | Algo type |
| `state` | String | Conditional | `effective`, `canceled`, `order_failed`; one of state/algoId required |
| `algoId` | String | Conditional | Specific algo order |
| `instType` | String | No | Instrument type |
| `instId` | String | No | Instrument ID |
| `after` | String | No | Pagination |
| `before` | String | No | Pagination |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 |

**Algo order state values:** `live`, `pause`, `partially_filled`, `filled`, `canceled`, `order_failed`, `effective`

---

## 4. BATCH OPERATIONS

### POST /api/v5/trade/batch-orders — Place Multiple Orders

**Auth required**: Trade permission
**Rate limit**: Per Instrument ID (same shared pool as single order)
**Max batch size**: 20 orders per request
**Note**: If batch contains only 1 order, it counts as a single order against single-order rate limit

**Request body (array):**

```json
[
  {
    "instId":  "BTC-USDT-SWAP",
    "tdMode":  "cross",
    "side":    "buy",
    "ordType": "limit",
    "px":      "50000",
    "sz":      "1",
    "posSide": "long",
    "clOrdId": "batch_001"
  },
  {
    "instId":  "ETH-USDT-SWAP",
    "tdMode":  "cross",
    "side":    "buy",
    "ordType": "limit",
    "px":      "3000",
    "sz":      "2",
    "posSide": "long",
    "clOrdId": "batch_002"
  }
]
```

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "clOrdId": "batch_001",
      "ordId":   "312269865356374016",
      "tag":     "",
      "sCode":   "0",
      "sMsg":    ""
    },
    {
      "clOrdId": "batch_002",
      "ordId":   "312269865356374017",
      "tag":     "",
      "sCode":   "0",
      "sMsg":    ""
    }
  ]
}
```

**Note**: Check individual `sCode` per order. Top-level `code: "0"` only means the batch was accepted, not all orders succeeded.

---

### POST /api/v5/trade/cancel-batch-orders — Cancel Multiple Orders

**Auth required**: Trade permission
**Rate limit**: Per Instrument ID; independent from place/amend
**Max batch size**: 20 orders per request

**Request body (array):**

```json
[
  { "instId": "BTC-USDT-SWAP", "ordId": "312269865356374016" },
  { "instId": "BTC-USDT-SWAP", "clOrdId": "batch_002" }
]
```

---

### POST /api/v5/trade/amend-batch-orders — Amend Multiple Orders

**Auth required**: Trade permission
**Rate limit**: Per Instrument ID
**Max batch size**: 20 orders per request

Request body follows same structure as `amend-order` but as an array.

---

## 5. CLOSE POSITION

### POST /api/v5/trade/close-position

**Auth required**: Trade permission
**Rate limit**: 20 requests/2s per User ID
**Applies to**: SWAP, FUTURES, MARGIN only (not SPOT)
**Note**: If pending orders exist for the same instrument/side, they must be manually canceled first (or use `autoCxl: true`)

**Request body:**

```json
{
  "instId":  "BTC-USDT-SWAP",
  "mgnMode": "cross",
  "posSide": "long",
  "ccy":     "",
  "autoCxl": false,
  "clOrdId": "close_001",
  "tag":     ""
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `mgnMode` | String | Yes | Margin mode: `isolated` or `cross` |
| `posSide` | String | Conditional | `long` or `short`; required in long_short_mode; `net` for net mode |
| `ccy` | String | Conditional | Currency; required for MARGIN cross in multi-currency mode |
| `autoCxl` | Boolean | No | Auto-cancel pending orders for this position before closing |
| `clOrdId` | String | No | Client order ID for the close order |
| `tag` | String | No | Order tag |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "instId":  "BTC-USDT-SWAP",
      "posSide": "long",
      "clOrdId": "close_001",
      "tag":     ""
    }
  ]
}
```

---

## 6. ORDER RESPONSE FORMAT

### Full Order Object (from GET /api/v5/trade/order and orders-pending/history)

```json
{
  "instType":         "SWAP",
  "instId":           "BTC-USDT-SWAP",
  "ccy":              "",
  "ordId":            "312269865356374016",
  "clOrdId":          "myorder_001",
  "tag":              "strategyA",
  "px":               "50912.4",
  "pxUsd":            "",
  "pxVol":            "",
  "sz":               "1",
  "notionalUsd":      "50912.4",
  "ordType":          "limit",
  "side":             "buy",
  "posSide":          "long",
  "tdMode":           "cross",
  "accFillSz":        "1",
  "fillPx":           "50900.0",
  "tradeId":          "123456789",
  "fillSz":           "1",
  "fillTime":         "1695190492000",
  "fillFee":          "-0.0254",
  "fillFeeCcy":       "USDT",
  "fillPnl":          "0",
  "execType":         "T",
  "avgPx":            "50900.0",
  "state":            "filled",
  "lever":            "10",
  "attachAlgoClOrdId": "",
  "tpTriggerPx":      "",
  "tpOrdPx":          "",
  "slTriggerPx":      "",
  "slOrdPx":          "",
  "tpTriggerPxType":  "last",
  "slTriggerPxType":  "last",
  "feeCcy":           "USDT",
  "fee":              "-0.0254",
  "rebateCcy":        "USDT",
  "rebate":           "0",
  "pnl":              "0",
  "source":           "",
  "cancelSource":     "",
  "category":         "normal",
  "uTime":            "1695190492000",
  "cTime":            "1695190491421",
  "reqId":            "",
  "amendResult":      "",
  "reduceOnly":       "false",
  "quickMgnType":     "",
  "stpId":            "",
  "stpMode":          "cancel_maker",
  "algoClOrdId":      "",
  "algoId":           "",
  "attachAlgoOrds":   []
}
```

### Key Order Fields Summary

| Field | Type | Description |
|-------|------|-------------|
| `ordId` | String | Exchange order ID |
| `clOrdId` | String | Client order ID |
| `instType` | String | `SPOT`, `MARGIN`, `SWAP`, `FUTURES`, `OPTION` |
| `instId` | String | Trading pair / contract symbol |
| `tdMode` | String | `cash`, `cross`, `isolated` |
| `ordType` | String | Order type |
| `side` | String | `buy` or `sell` |
| `posSide` | String | `long`, `short`, `net` |
| `px` | String | Order price |
| `sz` | String | Order size |
| `accFillSz` | String | Accumulated filled size |
| `fillPx` | String | Last fill price |
| `avgPx` | String | Average fill price |
| `state` | String | `live`, `partially_filled`, `filled`, `canceled` |
| `fee` | String | Total fee (negative = paid, positive = earned) |
| `feeCcy` | String | Fee currency |
| `rebate` | String | Maker rebate amount |
| `pnl` | String | Realized PnL (derivatives) |
| `lever` | String | Leverage at time of order |
| `cTime` | String | Creation timestamp (Unix ms) |
| `uTime` | String | Last update timestamp (Unix ms) |

---

## 7. WEBSOCKET TRADING

WebSocket order management shares the same rate limits as REST.

**Private endpoint**: `wss://ws.okx.com:8443/ws/v5/private`
**Demo endpoint**: `wss://wspap.okx.com:8443/ws/v5/private`

### Place Order via WebSocket

```json
{
  "id":  "order_ws_001",
  "op":  "order",
  "args": [
    {
      "instId":  "BTC-USDT-SWAP",
      "tdMode":  "cross",
      "side":    "buy",
      "ordType": "limit",
      "px":      "50000",
      "sz":      "1"
    }
  ]
}
```

**WebSocket ops**: `order`, `batch-orders`, `cancel-order`, `batch-cancel-orders`, `amend-order`, `batch-amend-orders`

### Subscribe to Order Updates

```json
{
  "op": "subscribe",
  "args": [
    {
      "channel":  "orders",
      "instType": "SWAP",
      "instId":   "BTC-USDT-SWAP"
    }
  ]
}
```

Order channel pushes full order objects on any state change.

---

## Sources

- [OKX API v5 Official Docs](https://www.okx.com/docs-v5/en/)
- [OKX API v5 Complete Guide](https://www.okx.com/en-us/learn/complete-guide-to-okex-api-v5-upgrade)
- [OKX Sub-Account Rate Limit Announcement](https://www.okx.com/help/fill-ratio-sub-account-rate-limit)
- [OKX Demo Trading Guide](https://www.okx.com/docs-v5/en/#overview-demo-trading)
