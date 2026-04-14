# MEXC Account API

## Account Types

MEXC supports two distinct account types with entirely separate APIs:

| Account Type | API | Base URL |
|-------------|-----|----------|
| Spot | Spot V3 | `https://api.mexc.com` |
| Futures (Contract) | Contract V1 | `https://contract.mexc.com` |

**NOTE:** There is **no cross-margin or isolated-margin lending account** documented. No margin trading endpoints exist in the public API documentation. `isMarginTradingAllowed` may appear in exchange info but no margin trading API is provided.

---

## SPOT ACCOUNT ENDPOINTS

### GET /api/v3/account — Spot Account Information

**Permission:** `SPOT_ACCOUNT_READ`
**Weight:** 10 (IP)
**Rate Limit:** 2 times/second

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `recvWindow` | LONG | NO | Max 60000 ms |
| `timestamp` | LONG | YES | Unix milliseconds |

#### Response JSON

```json
{
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "updateTime": null,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "0.00100000",
      "locked": "0.00000000"
    },
    {
      "asset": "USDT",
      "free": "5000.00000000",
      "locked": "125.00000000"
    }
  ],
  "permissions": ["SPOT"]
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `canTrade` | BOOL | Account can place orders |
| `canWithdraw` | BOOL | Account can withdraw |
| `canDeposit` | BOOL | Account can deposit |
| `updateTime` | LONG or null | Last update timestamp |
| `accountType` | STRING | Always `"SPOT"` |
| `balances` | ARRAY | Per-asset balance list |
| `balances[].asset` | STRING | Asset symbol |
| `balances[].free` | STRING | Available (unlocked) balance |
| `balances[].locked` | STRING | Frozen balance (in open orders) |
| `permissions` | ARRAY | Allowed account types, e.g. `["SPOT"]` |

**NOTE:** Only assets with non-zero balances are returned. Zero-balance assets are omitted.

---

### GET /api/v3/tradeFee — Symbol Commission Rates

**Permission:** `SPOT_ACCOUNT_READ`
**Weight:** Not specified

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | Trading pair |
| `timestamp` | LONG | YES | |

#### Response JSON

```json
{
  "symbol": "MXUSDT",
  "makerCommission": "0.002",
  "takerCommission": "0.002"
}
```

Rates are in decimal form (0.002 = 0.2%).

---

### GET /api/v3/mxDeduct/enable — MX Token Fee Deduction Status

**Permission:** `SPOT_ACCOUNT_READ`

Returns whether MX token fee deduction is enabled for the account.

```json
{
  "mxDeductEnable": true
}
```

### POST /api/v3/mxDeduct/enable — Toggle MX Fee Deduction

**Permission:** `SPOT_ACCOUNT_WRITE`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `mxDeductEnable` | BOOL | YES | `true` to enable, `false` to disable |
| `timestamp` | LONG | YES | |

---

### POST /api/v3/capital/transfer — Transfer Between Accounts

**Permission:** `SPOT_TRANSFER_WRITE`
**Weight:** 1 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `fromAccountType` | STRING | YES | `"SPOT"` or `"FUTURES"` |
| `toAccountType` | STRING | YES | `"SPOT"` or `"FUTURES"` |
| `asset` | STRING | YES | Asset to transfer, e.g. `"USDT"` |
| `amount` | STRING | YES | Transfer amount |
| `timestamp` | LONG | YES | |

#### Response JSON

```json
{
  "tranId": "c45d800a47ba4cbc876a5cd29388319"
}
```

---

### GET /api/v3/capital/transfer — Transfer History

**Permission:** `SPOT_TRANSFER_READ`
**Weight:** 1 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `fromAccountType` | STRING | YES | |
| `toAccountType` | STRING | YES | |
| `startTime` | LONG | NO | |
| `endTime` | LONG | NO | |
| `page` | INT | NO | Default 1 |
| `size` | INT | NO | Default 10; max 100 |
| `timestamp` | LONG | YES | |

#### Response JSON (each transfer record)

```json
{
  "tranId": "c45d800a47ba4cbc876a5cd29388319",
  "asset": "USDT",
  "amount": "100.00",
  "fromAccountType": "SPOT",
  "toAccountType": "FUTURES",
  "status": "CONFIRMED",
  "timestamp": 1699999999999
}
```

---

## FUTURES (CONTRACT) ACCOUNT ENDPOINTS

### GET /api/v1/private/account/assets — All Futures Balances

**Permission:** Account reading access
**Rate Limit:** Standard

Returns array of per-currency asset objects.

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "currency": "USDT",
      "positionMargin": "150.00",
      "availableBalance": "4850.00",
      "cashBalance": "5000.00",
      "frozenBalance": "0.00",
      "equity": "5010.50",
      "unrealized": "10.50",
      "bonus": "0.00"
    }
  ]
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `currency` | STRING | Asset symbol (e.g. `"USDT"`) |
| `positionMargin` | DECIMAL | Margin allocated to open positions |
| `availableBalance` | DECIMAL | Free balance available for new orders |
| `cashBalance` | DECIMAL | Withdrawable balance |
| `frozenBalance` | DECIMAL | Temporarily locked (pending orders) |
| `equity` | DECIMAL | Total account equity (cashBalance + unrealized) |
| `unrealized` | DECIMAL | Unrealized PnL across all open positions |
| `bonus` | DECIMAL | Promotional bonus amount |

---

### GET /api/v1/private/account/asset/{currency} — Single Currency Balance

Returns same fields as above but for a single currency.

**Example:** `GET /api/v1/private/account/asset/USDT`

---

### GET /api/v1/private/position/open_positions — Open Positions

**Permission:** Trade reading access

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | NO | Filter by symbol (e.g. `BTC_USDT`) |

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "positionId": 102015012431820288,
      "symbol": "BTC_USDT",
      "holdVol": "0.01",
      "positionType": 1,
      "openType": 1,
      "liquidatePrice": "25000.00",
      "holdAvgPrice": "35000.00",
      "realised": "0.00",
      "leverage": 20,
      "createTime": "2023-01-01T00:00:00Z"
    }
  ]
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `positionId` | LONG | Unique position ID |
| `symbol` | STRING | Trading pair (e.g. `BTC_USDT`) |
| `holdVol` | DECIMAL | Current position size in contracts |
| `positionType` | INT | `1`=long, `2`=short |
| `openType` | INT | `1`=isolated margin, `2`=cross margin |
| `liquidatePrice` | DECIMAL | Liquidation price |
| `holdAvgPrice` | DECIMAL | Average entry price |
| `realised` | DECIMAL | Realized PnL |
| `leverage` | INT | Current leverage multiplier |
| `createTime` | DATE | Position open timestamp |

---

### GET /api/v1/private/position/list/history_positions — Position History

Returns closed positions with same core fields plus close price and close time.

---

## LEVERAGE MANAGEMENT (FUTURES)

### GET /api/v1/private/position/leverage — Get Current Leverage

**Rate Limit:** 20 times/2 seconds

#### Request Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| `symbol` | STRING | YES |

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": {
    "positionType": 1,
    "level": 5,
    "imr": "0.05",
    "mmr": "0.025",
    "leverage": 20
  }
}
```

| Field | Description |
|-------|-------------|
| `positionType` | `1`=long, `2`=short |
| `level` | Risk tier |
| `imr` | Initial margin rate for this tier |
| `mmr` | Maintenance margin rate |
| `leverage` | Current leverage |

---

### POST /api/v1/private/position/change_leverage — Set Leverage

**Rate Limit:** 20 times/2 seconds

#### Request (with existing position)

```json
{
  "positionId": 102015012431820288,
  "leverage": 20
}
```

#### Request (without existing position)

```json
{
  "openType": 1,
  "leverage": 20,
  "symbol": "BTC_USDT",
  "positionType": 1
}
```

---

## POSITION MODE (FUTURES)

### GET /api/v1/private/position/position_mode

Returns current mode: `1`=Hedge mode, `2`=One-way mode.

### POST /api/v1/private/position/change_position_mode

```json
{
  "positionMode": 1
}
```

**Restriction:** Cannot change mode when open orders or positions exist.

---

## NOT SUPPORTED — Account Features

- **Margin trading (lending/borrowing)** — No margin loan endpoints documented
- **Isolated margin account** — Not available via API
- **Portfolio margin** — Not documented
- **Income history (Futures)** — No equivalent to Binance's `/fapi/v1/income` endpoint
- **Commission rate tiers** — Only per-symbol fee query available (not volume-based tier info)
- **Sub-account balance aggregation via API** — Sub-account creation allowed (max 30), but balance aggregation API not confirmed in docs

---

## Sources

- MEXC Spot V3 API: https://mexcdevelop.github.io/apidocs/spot_v3_en/
- MEXC Contract V1 API: https://mexcdevelop.github.io/apidocs/contract_v1_en/
