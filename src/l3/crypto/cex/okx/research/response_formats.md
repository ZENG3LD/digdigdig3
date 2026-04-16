# OKX API v5 Response Formats

## Standard Response Structure

All OKX API v5 REST endpoints follow a unified JSON response format with three main fields:

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      // Response data objects
    }
  ]
}
```

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `code` | String | Status code. `"0"` indicates success, non-zero indicates error |
| `msg` | String | Error message (empty on success, descriptive text on failure) |
| `data` | Array | Array of response objects (even for single-object responses) |

---

## Success Response

### Example: Get Balance

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "totalEq": "41624.32",
      "isoEq": "0",
      "adjEq": "41624.32",
      "details": [
        {
          "ccy": "USDT",
          "eq": "1000.5",
          "cashBal": "1000.5",
          "availBal": "950.25",
          "frozenBal": "50.25",
          "ordFrozen": "50.25",
          "upl": "0"
        }
      ]
    }
  ]
}
```

### Example: Place Order

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "tag": "",
      "sCode": "0",
      "sMsg": ""
    }
  ]
}
```

### Example: Get Server Time

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "ts": "1672841403093"
    }
  ]
}
```

---

## Error Response

When an error occurs, the `code` field contains a non-zero error code, and `msg` contains the error description.

### Example: Rate Limit Error

```json
{
  "code": "50011",
  "msg": "Rate limit reached. Please refer to API documentation and throttle requests accordingly",
  "data": []
}
```

### Example: Invalid Parameter

```json
{
  "code": "51000",
  "msg": "Parameter instId error",
  "data": []
}
```

### Example: Authentication Error

```json
{
  "code": "50111",
  "msg": "Invalid sign",
  "data": []
}
```

---

## Common Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 50011 | Rate limit reached | API rate limit exceeded |
| 50061 | Order rate limit reached | Order placement rate limit (1,000/2s) exceeded |
| 50101 | API frozen | API key blocked or insufficient permissions |
| 50102 | Timestamp request expired | Request older than 30 seconds |
| 50103 | OK-ACCESS-KEY cannot be empty | Missing API key header |
| 50104 | OK-ACCESS-PASSPHRASE cannot be empty | Missing passphrase header |
| 50105 | OK-ACCESS-TIMESTAMP cannot be empty | Missing timestamp header |
| 50106 | OK-ACCESS-SIGN cannot be empty | Missing signature header |
| 50107 | Invalid OK-ACCESS-TIMESTAMP | Timestamp format incorrect |
| 50111 | Invalid sign | Signature verification failed |
| 50113 | Invalid IP | IP not whitelisted |
| 51000 | Parameter {param} error | Invalid parameter value |
| 51001 | Instrument ID does not exist | Invalid instId |
| 51020 | Order placement failed | General order error |

---

## Market Data Responses

### Ticker Response

**Endpoint:** `GET /api/v5/market/ticker`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "last": "43250.5",
      "lastSz": "0.15",
      "askPx": "43251.0",
      "askSz": "2.5",
      "bidPx": "43250.0",
      "bidSz": "3.2",
      "open24h": "42800.0",
      "high24h": "43500.0",
      "low24h": "42500.0",
      "vol24h": "1850.25",
      "volCcy24h": "79852341.25",
      "sodUtc0": "42900.0",
      "sodUtc8": "43000.0",
      "ts": "1672841403093"
    }
  ]
}
```

**Field Descriptions:**

| Field | Description |
|-------|-------------|
| `instType` | Instrument type (SPOT, SWAP, FUTURES, MARGIN, OPTION) |
| `instId` | Instrument ID |
| `last` | Last traded price |
| `lastSz` | Last traded size |
| `askPx` | Best ask price |
| `askSz` | Ask size |
| `bidPx` | Best bid price |
| `bidSz` | Bid size |
| `open24h` | 24h opening price |
| `high24h` | 24h highest price |
| `low24h` | 24h lowest price |
| `vol24h` | 24h trading volume (base currency for SPOT, USD for contracts) |
| `volCcy24h` | 24h trading volume (quote currency) |
| `sodUtc0` | Start of day price (UTC 0) |
| `sodUtc8` | Start of day price (UTC 8) |
| `ts` | Timestamp (milliseconds) |

### Order Book Response

**Endpoint:** `GET /api/v5/market/books`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "asks": [
        ["43251.5", "1.2", "0", "3"],
        ["43252.0", "2.5", "0", "4"]
      ],
      "bids": [
        ["43250.0", "1.8", "0", "2"],
        ["43249.5", "3.1", "0", "5"]
      ],
      "ts": "1672841403093"
    }
  ]
}
```

**Array Format:** `[price, size, deprecated, amount]`

| Index | Field | Description |
|-------|-------|-------------|
| 0 | Price | Price level |
| 1 | Size | Quantity at this price level |
| 2 | (Deprecated) | Ignore this field |
| 3 | Amount | Total amount (in contracts/base currency) |

### Candlestick Response

**Endpoint:** `GET /api/v5/market/candles`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    [
      "1672840800000",
      "43200.0",
      "43350.0",
      "43150.0",
      "43250.5",
      "125.8",
      "5432108.9",
      "5432108.9",
      "1"
    ],
    [
      "1672844400000",
      "43250.5",
      "43400.0",
      "43200.0",
      "43380.0",
      "98.5",
      "4271093.0",
      "4271093.0",
      "0"
    ]
  ]
}
```

**Array Format:** `[timestamp, open, high, low, close, vol, volCcy, volCcyQuote, confirm]`

| Index | Field | Description |
|-------|-------|-------------|
| 0 | Timestamp | Opening time (milliseconds UTC) |
| 1 | Open | Open price |
| 2 | High | High price |
| 3 | Low | Low price |
| 4 | Close | Close price |
| 5 | Vol | Volume (trading currency) |
| 6 | VolCcy | Volume (quote currency) |
| 7 | VolCcyQuote | Volume (USD for contracts) |
| 8 | Confirm | `0` = candle in progress, `1` = candle complete |

---

## Trading Responses

### Place Order Response

**Endpoint:** `POST /api/v5/trade/order`

**Success:**
```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "tag": "",
      "sCode": "0",
      "sMsg": ""
    }
  ]
}
```

**Field Descriptions:**

| Field | Description |
|-------|-------------|
| `ordId` | Exchange-assigned order ID |
| `clOrdId` | Client-supplied order ID |
| `tag` | Order tag |
| `sCode` | Status code (`"0"` = success) |
| `sMsg` | Status message |

**Partial Success (Batch Orders):**
```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "order1",
      "sCode": "0",
      "sMsg": ""
    },
    {
      "ordId": "",
      "clOrdId": "order2",
      "sCode": "51020",
      "sMsg": "Order placement failed"
    }
  ]
}
```

### Get Order Response

**Endpoint:** `GET /api/v5/trade/order`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "px": "43200.0",
      "sz": "0.5",
      "ordType": "limit",
      "side": "buy",
      "tdMode": "cash",
      "state": "filled",
      "avgPx": "43200.0",
      "accFillSz": "0.5",
      "fillPx": "43200.0",
      "fillSz": "0.5",
      "fillTime": "1672841403093",
      "cTime": "1672841400000",
      "uTime": "1672841403093"
    }
  ]
}
```

**Field Descriptions:**

| Field | Description |
|-------|-------------|
| `ordId` | Order ID |
| `clOrdId` | Client order ID |
| `instId` | Instrument ID |
| `instType` | Instrument type |
| `px` | Order price |
| `sz` | Order size |
| `ordType` | Order type (`market`, `limit`, `post_only`, etc.) |
| `side` | Side (`buy`, `sell`) |
| `tdMode` | Trade mode (`cash`, `cross`, `isolated`) |
| `state` | Order state (`live`, `partially_filled`, `filled`, `canceled`) |
| `avgPx` | Average filled price |
| `accFillSz` | Accumulated fill size |
| `fillPx` | Last fill price |
| `fillSz` | Last fill size |
| `fillTime` | Last fill time |
| `cTime` | Creation time (milliseconds) |
| `uTime` | Update time (milliseconds) |

### Cancel Order Response

**Endpoint:** `POST /api/v5/trade/cancel-order`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "sCode": "0",
      "sMsg": ""
    }
  ]
}
```

---

## Account Responses

### Get Balance Response

**Endpoint:** `GET /api/v5/account/balance`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "totalEq": "41624.32",
      "isoEq": "0",
      "adjEq": "41624.32",
      "ordFroz": "0",
      "imr": "0",
      "mmr": "0",
      "mgnRatio": "",
      "notionalUsd": "",
      "uTime": "1672841403093",
      "details": [
        {
          "ccy": "BTC",
          "eq": "1.5",
          "cashBal": "1.5",
          "availBal": "1.2",
          "frozenBal": "0.3",
          "ordFrozen": "0.3",
          "liab": "0",
          "upl": "0",
          "uplLiab": "0",
          "crossLiab": "0",
          "isoLiab": "0",
          "mgnRatio": "",
          "interest": "0",
          "twap": "0",
          "maxLoan": "",
          "eqUsd": "64836.48",
          "notionalLever": "0",
          "stgyEq": "0",
          "isoUpl": "0"
        }
      ]
    }
  ]
}
```

**Field Descriptions:**

| Field | Description |
|-------|-------------|
| `totalEq` | Total equity (USD) |
| `isoEq` | Isolated margin equity (USD) |
| `adjEq` | Adjusted equity (USD) |
| `details` | Array of currency balances |
| `ccy` | Currency |
| `eq` | Equity of currency |
| `cashBal` | Cash balance |
| `availBal` | Available balance |
| `frozenBal` | Frozen balance |
| `ordFrozen` | Margin frozen for open orders |
| `upl` | Unrealized profit and loss |

### Get Positions Response

**Endpoint:** `GET /api/v5/account/positions`

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "instType": "SWAP",
      "instId": "BTC-USDT-SWAP",
      "mgnMode": "isolated",
      "posId": "312269865356374016",
      "posSide": "long",
      "pos": "10",
      "availPos": "10",
      "avgPx": "43000.0",
      "upl": "250.5",
      "uplRatio": "0.0058",
      "lever": "10",
      "liqPx": "39500.0",
      "markPx": "43025.05",
      "margin": "4300.0",
      "mgnRatio": "0.092",
      "cTime": "1672841400000",
      "uTime": "1672841403093"
    }
  ]
}
```

**Field Descriptions:**

| Field | Description |
|-------|-------------|
| `instId` | Instrument ID |
| `instType` | Instrument type |
| `mgnMode` | Margin mode (`cross`, `isolated`) |
| `posId` | Position ID |
| `posSide` | Position side (`long`, `short`, `net`) |
| `pos` | Quantity of positions |
| `availPos` | Available position to close |
| `avgPx` | Average open price |
| `upl` | Unrealized P&L |
| `uplRatio` | Unrealized P&L ratio |
| `lever` | Leverage |
| `liqPx` | Estimated liquidation price |
| `markPx` | Latest mark price |
| `margin` | Margin |
| `mgnRatio` | Margin ratio |

---

## WebSocket Response Formats

### Subscription Acknowledgment

```json
{
  "event": "subscribe",
  "arg": {
    "channel": "tickers",
    "instId": "BTC-USDT"
  },
  "connId": "a4d3ae55"
}
```

### Ticker Update

```json
{
  "arg": {
    "channel": "tickers",
    "instId": "BTC-USDT"
  },
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "last": "43250.5",
      "lastSz": "0.15",
      "askPx": "43251.0",
      "askSz": "2.5",
      "bidPx": "43250.0",
      "bidSz": "3.2",
      "open24h": "42800.0",
      "high24h": "43500.0",
      "low24h": "42500.0",
      "vol24h": "1850.25",
      "volCcy24h": "79852341.25",
      "ts": "1672841403093"
    }
  ]
}
```

### Order Update (Private Channel)

```json
{
  "arg": {
    "channel": "orders",
    "instType": "SPOT"
  },
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "px": "43200.0",
      "sz": "0.5",
      "ordType": "limit",
      "side": "buy",
      "state": "filled",
      "avgPx": "43200.0",
      "accFillSz": "0.5",
      "fillPx": "43200.0",
      "fillSz": "0.5",
      "fillTime": "1672841403093",
      "uTime": "1672841403093"
    }
  ]
}
```

### Error Message

```json
{
  "event": "error",
  "code": "60012",
  "msg": "Invalid request: {\"op\":\"subscribe\",\"argss\":[{\"channel\":\"tickers\",\"instId\":\"BTC-USDT\"}]}"
}
```

---

## Timestamp Format

All timestamps in OKX API responses are in **milliseconds since Unix epoch** (UTC).

**Example:** `"1672841403093"` = `2023-01-04 15:50:03.093 UTC`

**Conversion:**
```rust
use chrono::{DateTime, Utc};

let ts_ms: i64 = 1672841403093;
let dt = DateTime::<Utc>::from_timestamp_millis(ts_ms).unwrap();
println!("{}", dt); // 2023-01-04 15:50:03.093 UTC
```

---

## Notes

1. **Data is always an array**: Even single-object responses wrap the object in an array
2. **Success code is string**: `"0"` (not integer `0`)
3. **Empty data on error**: Failed requests return empty `data: []`
4. **Batch operations**: Use `sCode`/`sMsg` within each data object for individual status
5. **Timestamps**: Always milliseconds (not seconds)
6. **Decimal strings**: Prices/sizes are strings (e.g., `"43250.5"`) to preserve precision
