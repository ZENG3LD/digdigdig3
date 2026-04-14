# HyperLiquid Trading API Specification

Source: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint

## Overview

All trading actions are submitted via a single unified endpoint using HTTP POST with JSON body.

- **Mainnet:** `POST https://api.hyperliquid.xyz/exchange`
- **Testnet:** `POST https://api.hyperliquid-testnet.xyz/exchange`
- **Content-Type:** `application/json`

All requests share the same outer envelope:

```json
{
  "action": { ... },
  "nonce": 1713148990947,
  "signature": { "r": "0x...", "s": "0x...", "v": 28 },
  "vaultAddress": "0x...",
  "expiresAfter": 1713149090947
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | Object | Yes | The specific action payload (varies by type) |
| `nonce` | Number | Yes | Current unix timestamp in milliseconds (must be unique and increasing per signer) |
| `signature` | Object | Yes | ECDSA signature with fields r, s, v |
| `vaultAddress` | String | No | 42-char hex; required when acting on behalf of a sub-account or vault |
| `expiresAfter` | Number | No | Unix ms timestamp after which this action is rejected; stale requests incur 5x rate limit penalty |

---

## Asset Numbering

- **Perpetuals:** asset index = integer from universe metadata (e.g., `0` = BTC, `1` = ETH)
- **Spot:** asset index = `10000 + spot_index`

---

## Order Types Supported

### Time-in-Force (TIF) for Limit Orders

| TIF | Key | Behavior |
|-----|-----|----------|
| Good-till-Canceled | `"Gtc"` | Rests on book until filled or explicitly canceled |
| Immediate-or-Cancel | `"Ioc"` | Fills available quantity immediately; cancels any unfilled remainder |
| Add-Liquidity-Only (Post-Only) | `"Alo"` | Cancels immediately if it would match (never taker) |

FOK and GTD are NOT documented in the official API.

### Trigger Orders (Stop / Take-Profit / Stop-Loss)

Trigger orders have an additional sub-type field `tpsl`:

| `tpsl` value | Meaning |
|-------------|---------|
| `"tp"` | Take-profit trigger |
| `"sl"` | Stop-loss trigger |

`isMarket: true` = market execution when triggered; `isMarket: false` = limit execution at `triggerPx`.

### Advanced Order Types

- **TWAP (Time-Weighted Average Price):** Native TWAP execution over a specified time window (minutes parameter).
- **Dead Man's Switch (scheduleCancel):** Automatically cancels all open orders at a scheduled future timestamp.

---

## Order Placement

### Single or Batch Order Placement

Endpoint: `POST /exchange`

The same `order` action type supports both single and batch placement. Multiple orders are passed in the `orders` array. No explicit documented maximum batch size; IP-based rate limit weight scales as `1 + floor(batch_length / 40)`.

**Action type:** `"order"`

```json
{
  "action": {
    "type": "order",
    "orders": [
      {
        "a": 1,
        "b": true,
        "p": "1891.40",
        "s": "0.02",
        "r": false,
        "t": {
          "limit": { "tif": "Gtc" }
        },
        "c": "0x0000000000000000000000000000000000000000000000000000000000000001"
      }
    ],
    "grouping": "na"
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

**Order fields:**

| Field | Key | Type | Description |
|-------|-----|------|-------------|
| Asset | `a` | Number | Asset index (perp = index, spot = 10000+index) |
| Is Buy | `b` | Boolean | `true` = buy, `false` = sell |
| Price | `p` | String | Limit price as string (e.g., `"1891.40"`) |
| Size | `s` | String | Order size as string (e.g., `"0.02"`) |
| Reduce Only | `r` | Boolean | If true, order can only reduce an existing position |
| Order Type | `t` | Object | Either `{"limit": {"tif": "Gtc|Ioc|Alo"}}` or `{"trigger": {...}}` |
| Client Order ID | `c` | String | Optional; 128-bit hex string for client-side tracking |

**Trigger order type field:**

```json
"t": {
  "trigger": {
    "isMarket": true,
    "triggerPx": "1800.00",
    "tpsl": "sl"
  }
}
```

**Grouping field values:**

| Value | Meaning |
|-------|---------|
| `"na"` | No grouping / standalone orders |
| `"normalTpsl"` | Standard TP/SL pair |
| `"positionTpsl"` | Position-level TP/SL |

**Response examples:**

```json
// Resting on book:
{"status":"ok","response":{"type":"order","data":{"statuses":[{"resting":{"oid":77738308}}]}}}

// Filled:
{"status":"ok","response":{"type":"order","data":{"statuses":[{"filled":{"totalSz":"0.02","avgPx":"1891.4","oid":77747314}}]}}}

// Error:
{"status":"ok","response":{"type":"order","data":{"statuses":[{"error":"Order must have minimum value of $10."}]}}}
```

### TWAP Order Placement

**Action type:** `"twapOrder"`

```json
{
  "action": {
    "type": "twapOrder",
    "twap": {
      "a": 1,
      "b": true,
      "s": "1.00",
      "r": false,
      "m": 10,
      "t": false
    }
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

| Field | Key | Type | Description |
|-------|-----|------|-------------|
| Asset | `a` | Number | Asset index |
| Is Buy | `b` | Boolean | Buy or sell |
| Size | `s` | String | Total size to execute |
| Reduce Only | `r` | Boolean | Position-reducing flag |
| Minutes | `m` | Number | Duration of TWAP execution in minutes |
| Randomize | `t` | Boolean | Randomize slice timing if true |

**Response:**
```json
{"status":"ok","response":{"type":"twapOrder","data":{"status":{"running":{"twapId":77738308}}}}}
```

---

## Order Management

### Cancel by Order ID

**Action type:** `"cancel"`

```json
{
  "action": {
    "type": "cancel",
    "cancels": [
      { "a": 1, "o": 77738308 }
    ]
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

Batch cancellation: pass multiple objects in the `cancels` array.

### Cancel by Client Order ID

**Action type:** `"cancelByCloid"`

```json
{
  "action": {
    "type": "cancelByCloid",
    "cancels": [
      { "asset": 1, "cloid": "0x0000000000000000000000000000000000000000000000000000000000000001" }
    ]
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Cancel TWAP

**Action type:** `"twapCancel"`

```json
{
  "action": {
    "type": "twapCancel",
    "a": 1,
    "t": 77738308
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

| Field | Key | Description |
|-------|-----|-------------|
| Asset | `a` | Asset index |
| TWAP ID | `t` | The twapId returned when placing the TWAP |

### Schedule Cancel (Dead Man's Switch)

**Action type:** `"scheduleCancel"`

Cancels ALL open orders at the specified future timestamp.

```json
{
  "action": {
    "type": "scheduleCancel",
    "time": 1713149090947
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Modify Single Order

**Action type:** `"modify"`

```json
{
  "action": {
    "type": "modify",
    "oid": 77738308,
    "order": {
      "a": 1,
      "b": true,
      "p": "1850.00",
      "s": "0.05",
      "r": false,
      "t": { "limit": { "tif": "Gtc" } },
      "c": "0x..."
    }
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

`oid` can be either a numeric order ID or a client order ID string.

### Batch Modify Multiple Orders

**Action type:** `"batchModify"`

```json
{
  "action": {
    "type": "batchModify",
    "modifies": [
      {
        "oid": 77738308,
        "order": { "a": 1, "b": true, "p": "1850.00", "s": "0.05", "r": false, "t": { "limit": { "tif": "Gtc" } } }
      }
    ]
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

### Query Open Orders

**Endpoint:** `POST https://api.hyperliquid.xyz/info`

```json
{ "type": "openOrders", "user": "0x..." }
```

Response:
```json
[{ "coin": "BTC", "limitPx": "29792.0", "oid": 91490942, "side": "A", "sz": "0.0", "timestamp": 1681247412573 }]
```

`side` field: `"A"` = Ask (sell), `"B"` = Bid (buy).

### Query Order Status by ID

```json
{ "type": "orderStatus", "user": "0x...", "oid": 91490942 }
```

Also accepts a client order ID string in the `oid` field.

### Query Historical Orders

```json
{ "type": "historicalOrders", "user": "0x..." }
```

Returns up to 2000 most recent orders. Response includes order details plus `status` (`"filled"`, `"open"`, `"canceled"`) and `statusTimestamp`.

### Query Fills

```json
{ "type": "userFills", "user": "0x...", "aggregateByTime": false }
```

Returns up to 2000 most recent fills. Each fill includes: `coin`, `px`, `sz`, `fee`, `feeToken`, `tid` (trade ID).

Time-bounded variant:
```json
{ "type": "userFillsByTime", "user": "0x...", "startTime": 1681222254710, "endTime": 1681222354710 }
```

Also up to 2000 fills per response.

### Frontend Open Orders (with metadata)

```json
{ "type": "frontendOpenOrders", "user": "0x..." }
```

Same as `openOrders` but includes additional frontend-specific metadata fields.

---

## Position Management

### Get Positions (Clearinghouse State)

**Endpoint:** `POST https://api.hyperliquid.xyz/info`

```json
{ "type": "clearinghouseState", "user": "0x...", "dex": "" }
```

Response includes:
- `assetPositions`: array of positions per asset, each with:
  - `entryPx`: entry price
  - `leverage`: cross or isolated with leverage value
  - `liquidationPx`: estimated liquidation price (null if cross)
  - `marginUsed`: margin allocated
  - `positionValue`: current value
  - `unrealizedPnl`: unrealized P&L
  - `returnOnEquity`: ROE
  - `szi`: signed size (positive = long, negative = short)
- `crossMarginSummary`: `accountValue`, `totalMarginUsed`, `totalNtlPos`
- `withdrawable`: available for withdrawal

### Set Leverage

**Action type:** `"updateLeverage"` on `/exchange`

```json
{
  "action": {
    "type": "updateLeverage",
    "asset": 1,
    "isCross": false,
    "leverage": 10
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `asset` | Number | Asset index |
| `isCross` | Boolean | `true` = cross margin mode, `false` = isolated |
| `leverage` | Integer | Leverage multiplier (integer) |

### Change Margin Mode

The `isCross` field in `updateLeverage` controls margin mode. Setting `isCross: true` switches to cross; `isCross: false` switches to isolated.

### Add / Remove Isolated Margin

**Action type:** `"updateIsolatedMargin"`

```json
{
  "action": {
    "type": "updateIsolatedMargin",
    "asset": 1,
    "isBuy": true,
    "ntli": 500
  },
  "nonce": 1713148990947,
  "signature": { ... }
}
```

`ntli` (net transfer to leverage in USDC): positive = add margin, negative = remove margin.

Alternative — target a specific leverage ratio for isolated margin:

```json
{
  "action": {
    "type": "topUpIsolatedOnlyMargin",
    "asset": 1,
    "leverage": "10.0"
  },
  "nonce": ...,
  "signature": { ... }
}
```

### Get Funding Rates

Historical funding rates by asset:
```json
{ "type": "fundingHistory", "coin": "ETH", "startTime": 1683849600000, "endTime": 1683849600076 }
```

Response: array of `{ rate, premium, time }`.

Predicted funding rates across venues:
```json
{ "type": "predictedFundings" }
```

Response includes funding predictions for HlPerp, BinPerp, BybitPerp with next funding timestamps.

User funding history:
```json
{ "type": "userFunding", "user": "0x...", "startTime": 1681222254710 }
```

### Get Liquidation Price

Liquidation price is returned in the `clearinghouseState` response under `assetPositions[n].liquidationPx`. Returns `null` for cross-margin positions.

### Active Asset Data (Trade Limits)

```json
{ "type": "activeAssetData", "user": "0x...", "coin": "ETH" }
```

Returns max trade sizes and available-to-trade amounts for that asset.

---

## Advanced Features

### Vault Trading

Trading on behalf of a vault or sub-account is done by setting `vaultAddress` in the request envelope to the vault/sub-account address. The signing key must belong to the master account or an approved agent wallet.

```json
{
  "action": { ... },
  "nonce": ...,
  "signature": { ... },
  "vaultAddress": "0x<vault_or_subaccount_address>"
}
```

No separate endpoint — the `vaultAddress` field works with all trading action types.

### Portfolio / Account Abstraction Modes

**Action type:** `"userSetAbstraction"` or `"agentSetAbstraction"`

```json
{
  "action": {
    "type": "userSetAbstraction",
    "mode": "unifiedAccount"
  },
  "nonce": ...,
  "signature": { ... }
}
```

Mode options: `"disabled"`, `"unifiedAccount"`, `"portfolioMargin"`.

### Reserve Rate Limit Capacity

**Action type:** `"reserveRequestWeight"` — purchase additional rate-limit capacity at 0.0005 USDC per unit.

### Noop Action

**Action type:** `"noop"` — marks a nonce as used without executing any state change. Useful for nonce management.

---

## Order Constraints (Documented)

- Minimum order value: $10 (orders below this are rejected)
- Reduce-only orders with 1,000+ existing open orders may be rejected
- Trigger orders with 1,000+ existing open orders may be rejected
- `expiresAfter` stale requests incur 5x rate limit penalty

---

## Sources

- [Exchange Endpoint | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [Info Endpoint | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Perpetuals | Hyperliquid Docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Place Order | Hyperliquid — Chainstack](https://docs.chainstack.com/reference/hyperliquid-exchange-place-order)
