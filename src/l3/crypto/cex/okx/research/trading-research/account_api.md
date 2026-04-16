# OKX Account API — V5 Complete Reference

Base URL: `https://www.okx.com`

---

## 1. ACCOUNT TYPES — OKX UNIFIED ACCOUNT

OKX uses a **Unified Account** — a single trading account spanning all instrument types simultaneously (Spot, Margin, Futures, Swap, Options). This is the core architectural difference from most other exchanges.

### Account Levels (`acctLv`)

| Value | Mode | Description |
|-------|------|-------------|
| `1` | Simple | Spot only; no margin/derivatives |
| `2` | Single-currency margin | All products, but one settlement currency |
| `3` | Multi-currency margin | All products; multiple collateral currencies |
| `4` | Portfolio margin | Advanced; cross-product margin netting |

### Position Mode (`posMode`)

| Value | Description |
|-------|-------------|
| `long_short_mode` | Separate long/short positions per instrument; `posSide` required on all orders |
| `net_mode` | Single net position per instrument; no `posSide` needed |

### Key Account Parameters

| Parameter | Where Used | Description |
|-----------|-----------|-------------|
| `acctLv` | account/config | Account level (1-4) |
| `posMode` | account/config | long_short_mode or net_mode |
| `tdMode` | trade/order | cash, isolated, cross (per order) |
| `mgnMode` | positions | isolated or cross (per position) |
| `instType` | everywhere | SPOT, MARGIN, SWAP, FUTURES, OPTION |

---

## 2. BALANCE & ACCOUNT INFO

### GET /api/v5/account/balance

**Auth required**: Read permission
**Rate limit**: 10 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ccy` | String | No | Comma-separated currencies to filter (max 20), e.g. `BTC,ETH,USDT` |

**Response — Account Level:**

```json
{
  "code": "0",
  "data": [
    {
      "uTime":              "1695190491421",
      "totalEq":            "55415.62",
      "isoEq":              "0",
      "adjEq":              "55415.62",
      "ordFroz":            "0",
      "imr":                "0",
      "mmr":                "0",
      "borrowFroz":         "0",
      "mgnRatio":           "",
      "notionalUsd":        "0",
      "notionalUsdForBorrow": "0",
      "notionalUsdForSwap":   "0",
      "notionalUsdForFutures": "0",
      "notionalUsdForOption": "0",
      "upl":                "0",
      "delta":              "",
      "deltaLever":         "",
      "deltaNeutralStatus": "",
      "availEq":            "55415.62",
      "details": [
        {
          "ccy":              "USDT",
          "eq":               "55415.62",
          "cashBal":          "55415.62",
          "uTime":            "1695190491421",
          "isoEq":            "0",
          "availEq":          "55415.62",
          "disEq":            "55415.62",
          "availBal":         "55415.62",
          "frozenBal":        "0",
          "ordFrozen":        "0",
          "liab":             "0",
          "upl":              "0",
          "uplLiab":          "0",
          "crossLiab":        "0",
          "isoLiab":          "0",
          "interest":         "0",
          "twap":             "0",
          "maxLoan":          "25000",
          "eqUsd":            "55415.62",
          "borrowFroz":       "0",
          "notionalLever":    "0",
          "stgyEq":           "0",
          "isoUpl":           "0",
          "spotInUseAmt":     "0",
          "clSpotInUseAmt":   "0",
          "maxSpotInUse":     "",
          "spotIsoBal":       "0",
          "imr":              "0",
          "mmr":              "0",
          "smtSyncEq":        "0",
          "collateralEnabled": true,
          "frpType":          "0"
        }
      ]
    }
  ]
}
```

**Account-level balance fields:**

| Field | Type | Description |
|-------|------|-------------|
| `totalEq` | String | Total equity in USD |
| `adjEq` | String | Adjusted equity (for margin calculations) |
| `availEq` | String | Available equity for opening positions |
| `ordFroz` | String | Equity frozen by pending orders |
| `imr` | String | Initial margin requirement |
| `mmr` | String | Maintenance margin requirement |
| `mgnRatio` | String | Margin ratio |
| `notionalUsd` | String | Total notional value of derivatives positions |
| `upl` | String | Total unrealized PnL |
| `borrowFroz` | String | Margin frozen for borrowing |

**Per-currency detail fields:**

| Field | Type | Description |
|-------|------|-------------|
| `ccy` | String | Currency code |
| `eq` | String | Total equity (includes unrealized PnL) |
| `cashBal` | String | Cash balance |
| `availBal` | String | Available balance (spot) |
| `availEq` | String | Available equity (derivatives) |
| `frozenBal` | String | Frozen balance |
| `ordFrozen` | String | Frozen by orders |
| `liab` | String | Borrow liability |
| `interest` | String | Accrued interest |
| `upl` | String | Unrealized PnL in this currency |
| `maxLoan` | String | Maximum borrowable amount |
| `eqUsd` | String | Equity converted to USD |
| `collateralEnabled` | Boolean | Whether this currency is used as collateral |
| `stgyEq` | String | Strategy equity |

---

### GET /api/v5/account/config — Account Configuration

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**No request parameters.**

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "uid":              "44705892343619584",
      "mainUid":          "44705892343619584",
      "acctLv":           "2",
      "posMode":          "long_short_mode",
      "autoLoan":         false,
      "greeksType":       "PA",
      "level":            "Lv1",
      "levelTmp":         "",
      "ctIsoMode":        "automatic",
      "mgnIsoMode":       "automatic",
      "spotOffsetType":   "",
      "roleType":         "0",
      "traderInsts":      [],
      "spotRoleType":     "0",
      "spotTraderInsts":  [],
      "opAuth":           "1",
      "kycLv":            "3",
      "label":            "v5 test",
      "ip":               "",
      "perm":             "read_only,withdraw,trade",
      "type":             "0",
      "stpMode":          "cancel_maker",
      "acctStpMode":      "cancel_maker",
      "feeType":          "0",
      "enableSpotBorrow": false,
      "spotBorrowAutoRepay": false,
      "stgyType":         "0",
      "liquidationGear":  "-1",
      "settleCcy":        "USDC",
      "settleCcyList":    ["USD", "USDC", "USDG"]
    }
  ]
}
```

**Config fields:**

| Field | Type | Description |
|-------|------|-------------|
| `uid` | String | User ID |
| `mainUid` | String | Master account UID (same as uid if not sub-account) |
| `acctLv` | String | Account level: `1`=simple, `2`=single-ccy margin, `3`=multi-ccy margin, `4`=portfolio margin |
| `posMode` | String | Position mode: `long_short_mode` or `net_mode` |
| `autoLoan` | Boolean | Whether auto-borrow is enabled for margin |
| `greeksType` | String | Greeks display type: `PA` (actual), `BS` (Black-Scholes) |
| `level` | String | VIP level: `Lv1`-`Lv8`, `VIP1`-`VIP8` |
| `ctIsoMode` | String | Contract isolated margin mode: `automatic` or `autonomy` |
| `mgnIsoMode` | String | Margin isolated mode: `automatic` or `autonomy` |
| `opAuth` | String | Whether options trading is authorized: `0`=no, `1`=yes |
| `kycLv` | String | KYC verification level |
| `perm` | String | API key permissions (comma-separated) |
| `type` | String | Account type: `0`=main, `1`=standard sub, `2`=managed sub |
| `stpMode` | String | Default self-trade prevention mode |
| `settleCcy` | String | Default settlement currency |

---

## 3. POSITIONS

### GET /api/v5/account/positions — Current Open Positions

**Auth required**: Read permission
**Rate limit**: 10 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | No | `MARGIN`, `SWAP`, `FUTURES`, `OPTION` (not SPOT) |
| `instId` | String | No | Specific instrument |
| `posId` | String | No | Specific position ID |

**Response — Position Object:**

```json
{
  "code": "0",
  "data": [
    {
      "instType":       "SWAP",
      "mgnMode":        "cross",
      "posId":          "307173036051015680",
      "posSide":        "long",
      "pos":            "10",
      "baseBal":        "",
      "quoteBal":       "",
      "baseBorrowed":   "",
      "quoteInterest":  "",
      "posCcy":         "",
      "availPos":       "10",
      "avgPx":          "50000.0",
      "upl":            "1200.0",
      "uplRatio":       "0.024",
      "uplLastPx":      "1210.0",
      "uplRatioLastPx": "0.0242",
      "instId":         "BTC-USDT-SWAP",
      "lever":          "10",
      "liqPx":          "45200.0",
      "markPx":         "50120.0",
      "imr":            "5000.0",
      "margin":         "5000.0",
      "mgnRatio":       "11.24",
      "mmr":            "125.0",
      "liab":           "",
      "liabCcy":        "",
      "interest":       "0",
      "tradeId":        "123456789",
      "notionalUsd":    "50120.0",
      "optVal":         "",
      "adl":            "2",
      "bizRefId":       "",
      "bizRefType":     "",
      "ccy":            "USDT",
      "last":           "50120.0",
      "idxPx":          "50100.0",
      "usdPx":          "",
      "bePx":           "45500.0",
      "deltaBS":        "",
      "deltaPA":        "",
      "gammaBS":        "",
      "gammaPA":        "",
      "thetaBS":        "",
      "thetaPA":        "",
      "vegaBS":         "",
      "vegaPA":         "",
      "spotInUseAmt":   "",
      "clSpotInUseAmt": "",
      "maxSpotInUse":   "",
      "realizedPnl":    "0",
      "pnl":            "0",
      "fee":            "-12.5",
      "fundingFee":     "-5.0",
      "liqPenalty":     "0",
      "closeOrderAlgo": [],
      "cTime":          "1695190491421",
      "uTime":          "1695190491421"
    }
  ]
}
```

**Position fields:**

| Field | Type | Description |
|-------|------|-------------|
| `instType` | String | Instrument type |
| `instId` | String | Instrument ID |
| `mgnMode` | String | `isolated` or `cross` |
| `posId` | String | Position ID |
| `posSide` | String | `long`, `short`, `net` |
| `pos` | String | Position size (contracts for derivatives; base ccy units for margin) |
| `availPos` | String | Available position for closing |
| `avgPx` | String | Average opening price |
| `upl` | String | Unrealized PnL (mark price basis) |
| `uplRatio` | String | Unrealized PnL ratio |
| `liqPx` | String | Estimated liquidation price |
| `markPx` | String | Current mark price |
| `lever` | String | Current leverage |
| `imr` | String | Initial margin requirement |
| `mmr` | String | Maintenance margin requirement |
| `margin` | String | Margin allocated (isolated mode) |
| `mgnRatio` | String | Margin ratio |
| `notionalUsd` | String | Position notional value in USD |
| `adl` | String | Auto-deleveraging rank (1=safest, 5=most at risk) |
| `bePx` | String | Break-even price |
| `realizedPnl` | String | Realized PnL since position opened |
| `fee` | String | Cumulative trading fees |
| `fundingFee` | String | Cumulative funding fees paid/received |
| `cTime` | String | Position creation time (Unix ms) |
| `uTime` | String | Last update time (Unix ms) |

---

### GET /api/v5/account/positions-history — Position History

**Auth required**: Read permission
**Rate limit**: 1 request/10s per User ID (much slower!)

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | No | `MARGIN`, `SWAP`, `FUTURES`, `OPTION` |
| `instId` | String | No | Specific instrument |
| `mgnMode` | String | No | `isolated` or `cross` |
| `type` | String | No | Closing type: `1`=close position, `2`=delivery, `3`=exercised, `4`=expired, `5`=ADL, `6`=liquidation |
| `posId` | String | No | Position ID |
| `after` | String | No | Pagination (closed positions after this posId) |
| `before` | String | No | Pagination |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 (default 100) |

**Position History Object:**

```json
{
  "instId":       "BTC-USDT-SWAP",
  "instType":     "SWAP",
  "mgnMode":      "cross",
  "posId":        "307173036051015680",
  "posSide":      "long",
  "direction":    "long",
  "lever":        "10",
  "openAvgPx":    "50000.0",
  "closeAvgPx":   "51500.0",
  "openMaxPos":   "10",
  "closeTotalPos": "10",
  "pnl":          "1500.0",
  "pnlRatio":     "0.03",
  "realizedPnl":  "1500.0",
  "fee":          "-25.0",
  "fundingFee":   "-10.0",
  "liqPenalty":   "0",
  "settledPnl":   "1465.0",
  "triggerPx":    "",
  "type":         "1",
  "ccy":          "USDT",
  "uly":          "BTC-USD",
  "cTime":        "1695100000000",
  "uTime":        "1695190491421"
}
```

**History position fields:**

| Field | Type | Description |
|-------|------|-------------|
| `direction` | String | `long` or `short` |
| `openAvgPx` | String | Average opening price |
| `closeAvgPx` | String | Average closing price |
| `openMaxPos` | String | Max position size held |
| `closeTotalPos` | String | Total size closed |
| `pnl` | String | Gross PnL |
| `pnlRatio` | String | PnL ratio |
| `realizedPnl` | String | PnL after fees |
| `settledPnl` | String | Actually settled PnL |
| `fee` | String | Trading fees paid |
| `fundingFee` | String | Funding fees paid/received |
| `type` | String | Closing type (see query params for values) |

---

### POST /api/v5/account/set-position-mode

**Auth required**: Trade permission
**Rate limit**: 5 requests/2s per User ID

**Request body:**

```json
{ "posMode": "long_short_mode" }
```

`posMode`: `long_short_mode` or `net_mode`

**Note**: Cannot change while open positions or pending orders exist.

---

## 4. LEVERAGE & MARGIN

### POST /api/v5/account/set-leverage

**Auth required**: Trade permission
**Rate limit**: 40 requests/2s per User ID + Instrument Family

**Request body:**

```json
{
  "instId":  "BTC-USDT-SWAP",
  "lever":   "10",
  "mgnMode": "cross",
  "posSide": "long"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Conditional | Required for MARGIN/SWAP/FUTURES; not for SPOT |
| `lever` | String | Yes | Leverage multiplier (e.g. `"10"`) |
| `mgnMode` | String | Yes | `isolated` or `cross` |
| `posSide` | String | Conditional | `long`, `short` for isolated in long_short_mode; `net` for net_mode |
| `ccy` | String | Conditional | Currency; required for cross margin in multi-ccy mode without instId |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "lever":   "10",
      "mgnMode": "cross",
      "instId":  "BTC-USDT-SWAP",
      "posSide": "long"
    }
  ]
}
```

---

### GET /api/v5/account/leverage-info

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `mgnMode` | String | Yes | `isolated` or `cross` |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "instId":  "BTC-USDT-SWAP",
      "mgnMode": "cross",
      "lever":   "10",
      "posSide": "long"
    }
  ]
}
```

---

### GET /api/v5/account/max-size — Max Order Size

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID (comma-separated, max 5) |
| `tdMode` | String | Yes | Trade mode |
| `ccy` | String | No | Currency for margin |
| `px` | String | No | Price (used for limit order size calculation) |
| `lever` | String | No | Leverage |
| `unSpotOffset` | Boolean | No | Exclude spot holdings from calculation |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "instId":  "BTC-USDT-SWAP",
      "ccy":     "USDT",
      "maxBuy":  "5",
      "maxSell": "5"
    }
  ]
}
```

---

### GET /api/v5/account/max-avail-size — Max Available Tradable Amount

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID (comma-separated) |
| `tdMode` | String | Yes | `isolated` or `cross` |
| `ccy` | String | No | Currency |
| `reduceOnly` | Boolean | No | For reduce-only calculations |
| `unSpotOffset` | Boolean | No | Exclude spot |
| `quickMgnType` | String | No | Quick margin type |

---

### POST /api/v5/account/position-margin — Adjust Isolated Position Margin

**Auth required**: Trade permission
**Rate limit**: 20 requests/2s per User ID

**Request body:**

```json
{
  "instId":  "BTC-USDT-SWAP",
  "posSide": "long",
  "type":    "add",
  "amt":     "100",
  "loanTrans": false
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `posSide` | String | Yes | `long`, `short`, `net` |
| `type` | String | Yes | `add` (add margin) or `reduce` (remove margin) |
| `amt` | String | Yes | Amount to add/remove |
| `loanTrans` | Boolean | No | Whether to borrow funds to add margin |
| `ccy` | String | No | Currency; for MARGIN only |
| `auto` | Boolean | No | Whether to auto-borrow |

---

## 5. MARGIN TRADING (BORROW/REPAY)

### GET /api/v5/account/max-loan — Max Borrowable Amount

**Auth required**: Read permission
**Rate limit**: 20 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instId` | String | Yes | Instrument ID |
| `mgnMode` | String | Yes | `isolated` or `cross` |
| `mgnCcy` | String | Conditional | Required for isolated margin |

---

### GET /api/v5/account/interest-accrued — Interest Records

**Auth required**: Read permission
**Rate limit**: 5 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | String | No | `1`=regular, `2`=VIP loans |
| `ccy` | String | No | Currency filter |
| `instId` | String | No | Instrument ID (for isolated margin) |
| `mgnMode` | String | No | `isolated` or `cross` |
| `after` | String | No | Pagination |
| `before` | String | No | Pagination |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 |

**Response fields:** `type`, `ccy`, `mgnMode`, `instId`, `interest`, `interestRate`, `liab`, `ts`

---

### GET /api/v5/account/interest-rate — Current Interest Rate

**Auth required**: Read permission
**Rate limit**: 5 requests/2s per User ID

**Query parameters:** `ccy` (optional)

**Response fields:** `ccy`, `interestRate`, `nextInterestTime`

---

## 6. TRANSFERS

### POST /api/v5/asset/transfer — Internal Transfer

**Auth required**: Trade permission
**Rate limit**: 1 request/s per User ID

**Account type codes:**

| Code | Account |
|------|---------|
| `1` | Spot (deprecated; now unified) |
| `6` | Funding account |
| `18` | Unified trading account |
| `9` | Options account (legacy) |

**Request body:**

```json
{
  "ccy":     "USDT",
  "amt":     "100.5",
  "from":    "6",
  "to":      "18",
  "type":    "0",
  "subAcct": ""
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ccy` | String | Yes | Currency to transfer |
| `amt` | String | Yes | Amount |
| `from` | String | Yes | Source account code |
| `to` | String | Yes | Destination account code |
| `type` | String | No | `0`=within main account (default), `1`=main→sub, `2`=sub→main, `3`=sub→sub |
| `subAcct` | String | Conditional | Sub-account name; required for type 1/2/3 |
| `loanTrans` | Boolean | No | Allow transfer of borrowed funds |
| `clientId` | String | No | Client-assigned transfer ID |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "transId": "7547797743156224",
      "ccy":     "USDT",
      "clientId": "",
      "from":    "6",
      "amt":     "100.5",
      "to":      "18"
    }
  ]
}
```

---

### GET /api/v5/asset/transfer-state — Transfer Status

**Auth required**: Read permission
**Rate limit**: 10 requests/s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `transId` | String | Conditional | Transfer ID |
| `clientId` | String | Conditional | Client transfer ID |
| `type` | String | No | Transfer type (same as above) |

---

## 7. RISK & METRICS

### GET /api/v5/account/trade-fee — Trading Fees

**Auth required**: Read permission
**Rate limit**: 5 requests/2s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | Yes | `SPOT`, `MARGIN`, `SWAP`, `FUTURES`, `OPTION` |
| `instId` | String | No | Specific instrument |
| `uly` | String | No | Underlying |
| `instFamily` | String | No | Instrument family |

**Response:**

```json
{
  "code": "0",
  "data": [
    {
      "category": "1",
      "delivery":  "",
      "exercise":  "",
      "instType":  "SWAP",
      "level":     "Lv1",
      "maker":     "-0.0002",
      "makerU":    "-0.0002",
      "makerUSDC": "-0.0002",
      "taker":     "0.0005",
      "takerU":    "0.0005",
      "takerUSDC": "0.0005",
      "ts":        "1695190491421"
    }
  ]
}
```

**Fee fields:** `maker` and `taker` are as decimals (e.g. `-0.0002` = 0.02% rebate for maker).

---

### GET /api/v5/account/bills — Account Bills (last 7 days)

**Auth required**: Read permission
**Rate limit**: 5 requests/s per User ID (last 7 days)
**Rate limit**: 5 requests/2s per User ID (archive, last 3 months)

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instType` | String | No | Instrument type filter |
| `ccy` | String | No | Currency filter |
| `mgnMode` | String | No | `isolated` or `cross` |
| `ctType` | String | No | Contract type for FUTURES |
| `type` | String | No | Bill type (see below) |
| `subType` | String | No | Bill sub-type |
| `after` | String | No | Pagination |
| `before` | String | No | Pagination |
| `begin` | String | No | Start time (Unix ms) |
| `end` | String | No | End time (Unix ms) |
| `limit` | String | No | Max 100 (default 100) |

**Bill types (`type`):**

| Value | Description |
|-------|-------------|
| `1` | Transfer |
| `2` | Trade |
| `3` | Delivery |
| `4` | Auto token conversion |
| `5` | Liquidation |
| `6` | Margin transfer |
| `7` | Interest deduction |
| `8` | Funding fee |
| `9` | ADL |
| `10` | Clawback |
| `11` | System token conversion |
| `12` | Strategy transfer |
| `13` | DDH |
| `14` | Block trade |
| `15` | Quick margin |

**Bill Object:**

```json
{
  "billId":   "987654321",
  "ordId":    "312269865356374016",
  "tradeId":  "123456789",
  "clOrdId":  "myorder_001",
  "instType": "SWAP",
  "instId":   "BTC-USDT-SWAP",
  "type":     "2",
  "subType":  "1",
  "ts":       "1695190491421",
  "balChg":   "-0.0254",
  "posBalChg": "0",
  "bal":      "55415.62",
  "posBal":   "5000.0",
  "sz":       "1",
  "px":       "50900.0",
  "side":     "buy",
  "posSide":  "long",
  "execType": "T",
  "fee":      "-0.0254",
  "feeCcy":   "USDT",
  "mgnMode":  "cross",
  "notes":    "",
  "pnl":      "0",
  "ccy":      "USDT",
  "from":     "",
  "to":       "",
  "tag":      ""
}
```

---

### GET /api/v5/account/bills-archive — Account Bills Archive (last 3 months)

Same parameters as `bills` but covers up to 3 months.

---

### GET /api/v5/account/risk-state — Portfolio Margin Risk State

**Auth required**: Read permission
**Rate limit**: 10 requests/2s per User ID
**Applies to**: Portfolio margin accounts (acctLv=4) only

**No request parameters.**

**Response fields:** `atRisk` (Boolean), `atRiskIdx` (index), `atRiskMgn` (margin flag), `pTime` (timestamp)

---

## 8. DEPOSITS & WITHDRAWALS

### GET /api/v5/asset/deposit-address

**Auth required**: Read permission (Withdraw permission)
**Rate limit**: 6 requests/s per User ID

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ccy` | String | Yes | Currency (e.g. `BTC`) |

**Response fields:** `addr`, `chain`, `ccy`, `selected`, `memo`, `tag`, `to`, `ctAddr`, `addrEx`

---

### GET /api/v5/asset/deposit-history

**Auth required**: Read permission
**Rate limit**: 6 requests/s per User ID

**Query parameters:** `ccy`, `depId`, `txId`, `type`, `state`, `after`, `before`, `begin`, `end`, `limit`

**Deposit states:**

| Value | Description |
|-------|-------------|
| `0` | Waiting for confirmation |
| `1` | Credited (not withdrawable) |
| `2` | Successful |
| `8` | Pending (risk review) |
| `11` | Match the address blacklist |
| `12` | Account or deposit frozen |
| `13` | Sub-account deposit interception |
| `14` | KYC limit |

**Response fields:** `actualDepBlkConfirm`, `amt`, `billId`, `ccy`, `chain`, `depId`, `from`, `state`, `to`, `ts`, `txId`

---

### POST /api/v5/asset/withdrawal

**Auth required**: Withdraw permission
**Rate limit**: 6 requests/s per User ID

**Request body:**

```json
{
  "amt":    "1.0",
  "fee":    "0.0001",
  "dest":   "4",
  "ccy":    "BTC",
  "chain":  "BTC-Bitcoin",
  "toAddr": "1A1zP1eP5QGefi2DMPTfTL5SLmv7Divf Na"
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ccy` | String | Yes | Currency |
| `amt` | String | Yes | Withdrawal amount |
| `dest` | String | Yes | `3`=OKX account, `4`=external blockchain address |
| `toAddr` | String | Yes | Destination address |
| `chain` | String | Yes | Chain (e.g. `BTC-Bitcoin`, `ETH-ERC20`) |
| `fee` | String | Yes | Network fee |
| `rcvrInfo` | Object | No | Recipient info for regulated transfers |
| `clientId` | String | No | Client withdrawal ID |
| `pwd` | String | No | Fund password (if set) |

**Response fields:** `amt`, `wdId`, `ccy`, `clientId`, `chain`

---

### GET /api/v5/asset/withdrawal-history

**Auth required**: Read permission (Withdraw)
**Rate limit**: 6 requests/s per User ID

**Query parameters:** `ccy`, `wdId`, `clientId`, `txId`, `type`, `state`, `after`, `before`, `begin`, `end`, `limit`

**Withdrawal states:**

| Value | Description |
|-------|-------------|
| `-3` | Canceling |
| `-2` | Canceled |
| `-1` | Failed |
| `0` | Waiting withdrawal |
| `1` | Withdrawing |
| `2` | Success |
| `7` | Approved |
| `10` | Waiting transfer |
| `4` | Waiting manual review |
| `5` | Waiting identity verification |
| `6` | Under review |
| `8` | Waiting |
| `12` | Sending |
| `3` | Awaiting email verification |
| `9` | Awaiting fund password |

**Response fields:** `chain`, `clientId`, `fee`, `ccy`, `amt`, `txId`, `from`, `to`, `state`, `wdId`, `ts`, `nonTradableAsset`

---

## Sources

- [OKX API v5 Official Docs](https://www.okx.com/docs-v5/en/)
- [OKX API v5 Complete Guide](https://www.okx.com/en-us/learn/complete-guide-to-okex-api-v5-upgrade)
- [OKX Trading Account REST API](https://www.okx.com/docs-v5/en/#trading-account-rest-api)
- [OKX okxAPI R Package Documentation](https://cran.r-project.org/web/packages/okxAPI/okxAPI.pdf)
