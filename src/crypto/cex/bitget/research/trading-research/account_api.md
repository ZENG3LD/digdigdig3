# Bitget V2 Account API

Base URL: `https://api.bitget.com`

All account endpoints require authentication headers (see `auth_levels.md`).

---

## Account Types

| Type | Namespace | Description |
|------|-----------|-------------|
| Spot | `/api/v2/spot/account/` | Spot wallet |
| USDT-M Futures | `/api/v2/mix/account/` (`productType=USDT-FUTURES`) | USDT-margined perpetual |
| USDC-M Futures | `/api/v2/mix/account/` (`productType=USDC-FUTURES`) | USDC-margined perpetual |
| Coin-M Futures | `/api/v2/mix/account/` (`productType=COIN-FUTURES`) | Coin-margined perpetual/delivery |
| Isolated Margin | `/api/v2/margin/isolated/` | Isolated margin borrowing |
| Cross Margin | `/api/v2/margin/crossed/` | Cross margin borrowing |

---

## Spot Account

### 1. Get Spot Account Assets

**GET** `/api/v2/spot/account/assets`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `coin` | string | No | Filter by coin (e.g. `USDT`) |

**Response:**

```json
{
  "code": "00000",
  "msg": "success",
  "data": [
    {
      "coin":       "BTC",
      "available":  "0.1",
      "frozen":     "0.0",
      "locked":     "0.0",
      "limitAvailable": "0.0",
      "uTime":      "1695808690167"
    },
    {
      "coin":       "USDT",
      "available":  "1000.00",
      "frozen":     "0.00",
      "locked":     "0.00",
      "limitAvailable": "0.00",
      "uTime":      "1695808690167"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `coin` | string | Asset symbol |
| `available` | string | Available for trading/withdrawal |
| `frozen` | string | In open orders |
| `locked` | string | Locked (other restrictions) |
| `limitAvailable` | string | Available for limit orders |
| `uTime` | string | Last update timestamp (ms) |

---

### 2. Get Spot Account Info

**GET** `/api/v2/spot/account/info`

Rate limit: 10 req/sec/UID

**Response:**

```json
{
  "code": "00000",
  "data": {
    "userId":    "123456",
    "inviterId": "654321",
    "ips":       "127.0.0.1",
    "authorities": ["trade", "readonly"],
    "parentId":  "0",
    "trader":    false
  }
}
```

---

### 3. Get Spot Bills (Account Ledger)

**GET** `/api/v2/spot/account/bills`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `coin` | string | No | Filter by coin |
| `groupType` | string | No | `deposit`, `withdraw`, `transaction`, `transfer`, `other` |
| `bizType` | string | No | Specific business type |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination cursor |

---

## Futures (Mix) Account

### 4. Get Futures Account List (All Coins)

**GET** `/api/v2/mix/account/accounts`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | `USDT-FUTURES`, `USDC-FUTURES`, `COIN-FUTURES` |

**Response:**

```json
{
  "code": "00000",
  "data": [
    {
      "marginCoin":          "USDT",
      "locked":              "0",
      "available":           "1000.00",
      "crossMaxAvailable":   "1000.00",
      "fixedMaxAvailable":   "1000.00",
      "maxTransferOut":      "1000.00",
      "equity":              "1000.00",
      "usdtEquity":          "1000.00",
      "btcEquity":           "0.0333",
      "crossRiskRate":       "0",
      "crossMarginLeverage": "20",
      "fixedLongLeverage":   "10",
      "fixedShortLeverage":  "10",
      "marginMode":          "isolated",
      "holdMode":            "hedge_mode",
      "unrealizedPL":        "0",
      "bonus":               "0",
      "productType":         "USDT-FUTURES"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `marginCoin` | string | Margin currency |
| `locked` | string | Locked in orders/positions |
| `available` | string | Available balance |
| `crossMaxAvailable` | string | Max available for cross margin |
| `fixedMaxAvailable` | string | Max available for isolated margin |
| `maxTransferOut` | string | Max transferable amount |
| `equity` | string | Total equity (available + unrealized PnL) |
| `usdtEquity` | string | Equity in USDT |
| `btcEquity` | string | Equity in BTC |
| `crossRiskRate` | string | Cross margin risk rate |
| `crossMarginLeverage` | string | Current cross margin leverage |
| `fixedLongLeverage` | string | Long isolated leverage |
| `fixedShortLeverage` | string | Short isolated leverage |
| `marginMode` | string | `isolated` or `crossed` |
| `holdMode` | string | `hedge_mode` or `one_way_mode` |
| `unrealizedPL` | string | Total unrealized PnL |
| `bonus` | string | Bonus balance |

---

### 5. Get Single Futures Account

**GET** `/api/v2/mix/account/account`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | e.g. `BTCUSDT` |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | e.g. `USDT` |

Response structure is identical to a single entry in the account list above.

---

### 6. Get Futures Account Bills

**GET** `/api/v2/mix/account/bill`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `coin` | string | No | Filter by coin |
| `businessType` | string | No | `settle`, `open`, `close`, `trans_from_exchange`, etc. |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination |

---

## Positions

### 7. Get All Positions

**GET** `/api/v2/mix/position/all-position`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `marginCoin` | string | No | Filter by margin coin |

**Response:**

```json
{
  "code": "00000",
  "data": [
    {
      "marginCoin":       "USDT",
      "symbol":           "BTCUSDT",
      "holdSide":         "long",
      "openDelegateSize": "0",
      "marginSize":       "300.00",
      "available":        "0.01",
      "locked":           "0",
      "total":            "0.01",
      "leverage":         "10",
      "achievedProfits":  "0",
      "openPriceAvg":     "30000.00",
      "marginMode":       "isolated",
      "posMode":          "hedge_mode",
      "unrealizedPL":     "5.00",
      "liquidationPrice": "27500.00",
      "keepMarginRate":   "0.004",
      "markPrice":        "30500.00",
      "marginRatio":      "0.02",
      "breakEvenPrice":   "30060.00",
      "totalFee":         "-0.018",
      "deductedFee":      "0",
      "grant":            "0",
      "assetMode":        "single",
      "autoMargin":       "off",
      "takeProfit":       "35000.00",
      "stopLoss":         "28000.00",
      "takeProfitId":     "1088600000000000002",
      "stopLossId":       "1088600000000000003",
      "cTime":            "1695808690167",
      "uTime":            "1695808700000"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `marginCoin` | string | Margin currency |
| `symbol` | string | Trading pair |
| `holdSide` | string | `long` or `short` |
| `openDelegateSize` | string | Size in pending open orders |
| `marginSize` | string | Current margin allocated |
| `available` | string | Available position size (not in close orders) |
| `locked` | string | Locked in close orders |
| `total` | string | Total position size |
| `leverage` | string | Current leverage |
| `achievedProfits` | string | Realized PnL on partial closes |
| `openPriceAvg` | string | Average entry price |
| `marginMode` | string | `isolated` or `crossed` |
| `posMode` | string | `hedge_mode` or `one_way_mode` |
| `unrealizedPL` | string | Unrealized PnL |
| `liquidationPrice` | string | Estimated liquidation price |
| `keepMarginRate` | string | Maintenance margin rate |
| `markPrice` | string | Current mark price |
| `marginRatio` | string | Current margin ratio |
| `breakEvenPrice` | string | Break-even price (includes fees) |
| `totalFee` | string | Cumulative fees paid |
| `autoMargin` | string | `on` or `off` |
| `takeProfit` | string | Attached TP trigger price |
| `stopLoss` | string | Attached SL trigger price |
| `takeProfitId` | string | TP plan order ID |
| `stopLossId` | string | SL plan order ID |
| `cTime` | string | Position creation timestamp (ms) |
| `uTime` | string | Last update timestamp (ms) |

---

### 8. Get Single Position

**GET** `/api/v2/mix/position/single-position`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | Margin coin |

Response is a single position object (same structure as above).

---

### 9. Get Historical Positions

**GET** `/api/v2/mix/position/history-position`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `symbol` | string | No | Filter by symbol |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination |

---

## Leverage & Margin Settings

### 10. Set Leverage

**POST** `/api/v2/mix/account/set-leverage`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "leverage":    "20",
  "holdSide":    "long"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | Margin coin |
| `leverage` | string | Yes | Leverage multiplier (e.g. `"20"`) |
| `holdSide` | string | Cond. | `long` or `short` — required in hedge mode; omit in one-way mode |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "symbol":      "BTCUSDT",
    "marginCoin":  "USDT",
    "longLeverage": "20",
    "shortLeverage": "20",
    "crossMarginLeverage": "20",
    "marginMode":  "isolated"
  }
}
```

Note: In cross margin mode, `leverage` applies to both sides; `holdSide` can be omitted.

---

### 11. Set Margin Mode

**POST** `/api/v2/mix/account/set-margin-mode`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "marginMode":  "isolated"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | Margin coin |
| `marginMode` | string | Yes | `isolated` or `crossed` |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "symbol":      "BTCUSDT",
    "marginCoin":  "USDT",
    "longLeverage": "10",
    "shortLeverage": "10",
    "marginMode":   "isolated"
  }
}
```

---

### 12. Set Position Mode (One-Way / Hedge)

**POST** `/api/v2/mix/account/set-position-mode`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "productType": "USDT-FUTURES",
  "posMode":     "hedge_mode"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `posMode` | string | Yes | `one_way_mode` or `hedge_mode` |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "posMode": "hedge_mode"
  }
}
```

Note: Cannot change position mode while open positions or orders exist.

---

### 13. Adjust Position Margin

**POST** `/api/v2/mix/account/set-margin`

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "holdSide":    "long",
  "amount":      "100.00"
}
```

`amount`: Positive to add margin, negative to reduce margin.

---

## Transfers

### 14. Transfer Between Accounts

**POST** `/api/v2/spot/wallet/transfer`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "fromType": "spot",
  "toType":   "mix_usdt",
  "amount":   "100.00",
  "coin":     "USDT",
  "clientOid": "transfer_001",
  "symbol":   ""
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fromType` | string | Yes | Source account type |
| `toType` | string | Yes | Destination account type |
| `amount` | string | Yes | Transfer amount |
| `coin` | string | Yes | Coin to transfer |
| `clientOid` | string | No | Client transfer ID |
| `symbol` | string | Cond. | Required for isolated margin transfers |

**`fromType` / `toType` values:**

| Value | Description |
|-------|-------------|
| `spot` | Spot account |
| `mix_usdt` | USDT-M futures account |
| `mix_usdc` | USDC-M futures account |
| `mix_usd` | Coin-M futures account |
| `crossed_margin` | Cross margin account |
| `isolated_margin` | Isolated margin account (requires `symbol`) |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "transferId": "1088700000000000001",
    "clientOid":  "transfer_001"
  }
}
```

---

## Fee Rates

### 15. Get VIP Fee Rate (Spot)

**GET** `/api/v2/spot/market/vip-fee-rate`

No authentication required.

**Response:**

```json
{
  "code": "00000",
  "data": [
    {
      "level":        "0",
      "dealAmount":   "0",
      "assetAmount":  "0",
      "takerFeeRate": "0.001",
      "makerFeeRate": "0.001",
      "btcWithdrawAmount": "100",
      "usdtWithdrawAmount": "500000"
    }
  ]
}
```

### 16. Get Trade Rate (Spot — Account Specific)

**GET** `/api/v2/common/trade-rate`

Rate limit: 10 req/sec/UID
Authentication required.

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `businessType` | string | Yes | `spot`, `mix` |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "makerFeeRate": "0.001",
    "takerFeeRate": "0.001"
  }
}
```

---

## Margin Account (Isolated)

### 17. Get Isolated Margin Account Assets

**GET** `/api/v2/margin/isolated/account/assets`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | No | Filter by trading pair |
| `coin` | string | No | Filter by coin |

**Response (per symbol):**

```json
{
  "symbol":          "BTCUSDT",
  "marginRatio":     "999",
  "liquidationPrice": "0",
  "baseCoin":        "BTC",
  "baseTransferable": "0",
  "baseBorrowed":    "0",
  "baseInterest":    "0",
  "quoteTransferable": "500.00",
  "quoteBorrowed":   "0",
  "quoteInterest":   "0",
  "baseAvailable":   "0",
  "baseFreeze":      "0",
  "quoteCoin":       "USDT",
  "quoteAvailable":  "500.00",
  "quoteFreeze":     "0",
  "status":          "normal",
  "cTime":           "1695808690167",
  "uTime":           "1695808690167"
}
```

---

## Cross Margin Account

### 18. Get Cross Margin Account Assets

**GET** `/api/v2/margin/crossed/account/assets`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `coin` | string | No | Filter by coin |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "netAssets":    "1000.00",
    "totalAssets":  "1000.00",
    "borrowed":     "0",
    "interest":     "0",
    "riskRate":     "999",
    "coinList": [
      {
        "coin":       "USDT",
        "available":  "1000.00",
        "locked":     "0",
        "borrowed":   "0",
        "interest":   "0",
        "netAsset":   "1000.00",
        "uTime":      "1695808690167"
      }
    ]
  }
}
```

---

## All-Account Balance Overview

### 19. Get All Account Balances

**GET** `/api/v2/account/all-account-balance`

Returns aggregated balances across spot, futures, and margin accounts.

Authentication required.

---

## Sources

- [Get Spot Account Assets](https://www.bitget.com/api-doc/spot/account/Get-Account-Assets)
- [Get Futures Account List](https://www.bitget.com/api-doc/contract/account/Get-Account-List)
- [Get Single Futures Account](https://www.bitget.com/api-doc/contract/account/Get-Single-Account)
- [Get All Positions](https://www.bitget.com/api-doc/contract/position/get-all-position)
- [Get Single Position](https://www.bitget.com/api-doc/contract/position/get-single-position)
- [Change Leverage](https://www.bitget.com/api-doc/contract/account/Change-Leverage)
- [Change Margin Mode](https://www.bitget.com/api-doc/contract/account/Change-Margin-Mode)
- [Change Position Mode](https://www.bitget.com/api-doc/contract/account/Change-Hold-Mode)
- [Adjust Position Margin](https://www.bitget.com/api-doc/contract/account/Change-Margin)
- [Wallet Transfer](https://www.bitget.com/api-doc/spot/wallet/Wallet-Transfer)
- [Assets Overview](https://www.bitget.com/api-doc/common/account/All-Account-Balance)
- [VIP Fee Rate](https://www.bitget.com/api-doc/spot/market/Get-VIP-Fee-Rate)
