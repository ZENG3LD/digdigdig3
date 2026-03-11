# Binance Account API — Complete Reference

Sources: Binance Spot, Futures USDM, Margin, and Wallet API documentation (developers.binance.com)

---

## 1. ACCOUNT TYPES

| Account Type | API Prefix | Description |
|-------------|------------|-------------|
| Spot | `/api/v3/` | Default trading account |
| Cross Margin | `/sapi/v1/margin/` | Single margin pool across all pairs |
| Isolated Margin | `/sapi/v1/margin/isolated/` | Separate margin pool per trading pair |
| USDM Futures | `/fapi/v1/`, `/fapi/v2/`, `/fapi/v3/` | USDⓈ-Margined perpetual/quarterly contracts |
| COIN-M Futures | `/dapi/v1/` | Coin-margined contracts |
| Options | `/eapi/v1/` | European-style options |
| Portfolio Margin | `/papi/v1/` | Cross-product margin (advanced) |

**These are separate accounts** — balances are NOT shared. Funds must be transferred explicitly between accounts.

**Key differences in API calls:**
- Spot trading: use `/api/v3/` base path.
- Margin trading: use `/sapi/v1/margin/` with `isIsolated` param to toggle between cross/isolated.
- USDM Futures: use `https://fapi.binance.com` as base URL.
- COIN-M Futures: use `https://dapi.binance.com` as base URL.

**Authentication header is the same:** `X-MBX-APIKEY: <api_key>` for all account types.

---

## 2. BALANCE AND ACCOUNT INFO

### Spot Account Info (`GET /api/v3/account`)

**Weight:** 20
**Security:** USER_DATA (signed)

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `omitZeroBalances` | BOOLEAN | NO | If true, hides zero-balance assets (default false) |
| `recvWindow` | LONG | NO | Max 60000 ms |
| `timestamp` | LONG | YES | |

**Response:**
```json
{
  "makerCommission": 15,
  "takerCommission": 15,
  "buyerCommission": 0,
  "sellerCommission": 0,
  "commissionRates": {
    "maker": "0.00150000",
    "taker": "0.00150000",
    "buyer": "0.00000000",
    "seller": "0.00000000"
  },
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "brokered": false,
  "requireSelfTradePrevention": false,
  "preventSor": false,
  "updateTime": 123456789,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "4723846.89208129",
      "locked": "0.00000000"
    },
    {
      "asset": "LTC",
      "free": "4763368.68006011",
      "locked": "0.00000000"
    }
  ],
  "permissions": ["SPOT"],
  "uid": 354937868
}
```

**Notes:**
- `locked` = funds held in open orders.
- `free` = available for trading/withdrawal.
- `permissions` array can contain: `SPOT`, `MARGIN`, `LEVERAGED`, `TRD_GRP_*`.
- `uid` is the user's account UID.

---

### Spot Commission Rates (`GET /api/v3/account/commission`)

**Weight:** 20
**Security:** USER_DATA

**Parameters:** `symbol` (required), `recvWindow`, `timestamp`.

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "standardCommission": {
    "maker": "0.00000010",
    "taker": "0.00000020",
    "buyer": "0.00000000",
    "seller": "0.00000000"
  },
  "taxCommission": {
    "maker": "0.00000112",
    "taker": "0.00000114",
    "buyer": "0.00000000",
    "seller": "0.00000000"
  },
  "discount": {
    "enabledForAccount": true,
    "enabledForSymbol": true,
    "discountAsset": "BNB",
    "discount": "0.75000000"
  }
}
```

---

### Spot Order Rate Limits (`GET /api/v3/rateLimit/order`)

**Weight:** 40
**Security:** USER_DATA

**Response:**
```json
[
  {
    "rateLimitType": "ORDERS",
    "interval": "SECOND",
    "intervalNum": 10,
    "limit": 50,
    "count": 0
  },
  {
    "rateLimitType": "ORDERS",
    "interval": "DAY",
    "intervalNum": 1,
    "limit": 160000,
    "count": 0
  }
]
```

---

### Futures Account Balance V2 (`GET /fapi/v2/balance`)

**Weight:** 5
**Security:** USER_DATA

**Parameters:** `recvWindow` (optional), `timestamp` (required).

**Response:**
```json
[
  {
    "accountAlias": "SgsR",
    "asset": "USDT",
    "balance": "122607.35137903",
    "crossWalletBalance": "23288.02641813",
    "crossUnPnl": "0.00000000",
    "availableBalance": "23288.02641813",
    "maxWithdrawAmount": "23288.02641813",
    "marginAvailable": true,
    "updateTime": 1617939110373
  }
]
```

**Field descriptions:**
- `accountAlias`: Unique account code.
- `balance`: Wallet balance (total funds including unrealized PnL).
- `crossWalletBalance`: Wallet balance in cross-margin mode.
- `crossUnPnl`: Unrealized PnL of all crossed positions.
- `availableBalance`: Can be used to open new positions.
- `maxWithdrawAmount`: Maximum transferable/withdrawable amount.
- `marginAvailable`: Whether this asset can be used as margin in Multi-Assets mode.

---

### Futures Account Information V2 (`GET /fapi/v2/account`)

**Weight:** 5
**Security:** USER_DATA

**Parameters:** `recvWindow` (optional), `timestamp` (required).

**Response (abbreviated):**
```json
{
  "feeTier": 0,
  "feeBurn": true,
  "canTrade": true,
  "canDeposit": true,
  "canWithdraw": true,
  "multiAssetsMargin": false,
  "tradeGroupId": -1,
  "updateTime": 0,
  "totalInitialMargin": "0.00000000",
  "totalMaintMargin": "0.00000000",
  "totalWalletBalance": "23.72469206",
  "totalUnrealizedProfit": "0.00000000",
  "totalMarginBalance": "23.72469206",
  "totalPositionInitialMargin": "0.00000000",
  "totalOpenOrderInitialMargin": "0.00000000",
  "totalCrossWalletBalance": "23.72469206",
  "totalCrossUnPnl": "0.00000000",
  "availableBalance": "23.72469206",
  "maxWithdrawAmount": "23.72469206",
  "assets": [
    {
      "asset": "USDT",
      "walletBalance": "23.72469206",
      "unrealizedProfit": "0.00000000",
      "marginBalance": "23.72469206",
      "maintMargin": "0.00000000",
      "initialMargin": "0.00000000",
      "positionInitialMargin": "0.00000000",
      "openOrderInitialMargin": "0.00000000",
      "crossWalletBalance": "23.72469206",
      "crossUnPnl": "0.00000000",
      "availableBalance": "23.72469206",
      "maxWithdrawAmount": "23.72469206",
      "marginAvailable": true,
      "updateTime": 1625474304765
    }
  ],
  "positions": [
    {
      "symbol": "BTCUSDT",
      "initialMargin": "0",
      "maintMargin": "0",
      "unrealizedProfit": "0.00000000",
      "positionInitialMargin": "0",
      "openOrderInitialMargin": "0",
      "leverage": "100",
      "isolated": false,
      "entryPrice": "0.00000",
      "maxNotional": "250000",
      "bidNotional": "0",
      "askNotional": "0",
      "positionSide": "BOTH",
      "positionAmt": "0",
      "updateTime": 0
    }
  ]
}
```

**Notes:**
- V2 returns ALL symbols, even those with no position/order.
- V3 (`GET /fapi/v3/account`) only returns symbols where user has positions or open orders (lighter response).
- `feeTier`: 0 = regular user, higher = VIP tier.

### Futures Account Information V3 (`GET /fapi/v3/account`)

**Weight:** 5
**Security:** USER_DATA

Same fields as V2 but:
- Only returns symbols with active positions or open orders (not all symbols).
- Supports Multi-Assets mode with USD-converted totals across assets (USDT, USDC, BTC etc.).

---

### Margin Account Info — Cross Margin (`GET /sapi/v1/margin/account`)

**Weight:** 10
**Security:** USER_DATA

**Parameters:** `recvWindow`, `timestamp`.

**Response fields:** `borrowEnabled`, `marginLevel`, `totalAssetOfBtc`, `totalLiabilityOfBtc`, `totalNetAssetOfBtc`, `tradeEnabled`, `transferEnabled`, `userAssets` (array of asset objects with `asset`, `borrowed`, `free`, `interest`, `locked`, `netAsset`).

---

### Margin Account Info — Isolated (`GET /sapi/v1/margin/isolated/account`)

**Weight:** 10
**Security:** USER_DATA

**Parameters:** `symbols` (optional, comma-separated; if omitted returns all isolated pairs), `recvWindow`, `timestamp`.

---

## 3. POSITIONS (FUTURES ONLY)

### Get Positions V2 (`GET /fapi/v2/positionRisk`)

**Weight:** 5
**Security:** USER_DATA

**Parameters:**
| Parameter | Type | Required |
|-----------|------|----------|
| `symbol` | STRING | NO (omit for all positions) |
| `recvWindow` | LONG | NO |
| `timestamp` | LONG | YES |

**Response:**
```json
[
  {
    "symbol": "BTCUSDT",
    "positionAmt": "0.010",
    "entryPrice": "36500.0",
    "breakEvenPrice": "36515.92",
    "markPrice": "36496.80808100",
    "unRealizedProfit": "-0.03191919",
    "liquidationPrice": "35000.00",
    "leverage": "10",
    "maxNotionalValue": "3000000",
    "marginType": "isolated",
    "isolatedMargin": "3.60040606",
    "isAutoAddMargin": "false",
    "positionSide": "BOTH",
    "notional": "364.96808081",
    "isolatedWallet": "3.63232585",
    "updateTime": 1625474304765
  }
]
```

**Notes:**
- `positionSide`: `BOTH` = One-way mode; `LONG`/`SHORT` = Hedge mode.
- `positionAmt`: Negative = short position.
- Only returns positions with non-zero `positionAmt` (held positions).
- Recommendation: use with User Data Stream `ACCOUNT_UPDATE` for real-time accuracy.
- `/fapi/v1/positionRisk` is **retired** — use V2 or V3.

### Get Positions V3 (`GET /fapi/v3/positionRisk`)

Returns held positions AND positions with open orders.

---

### Close a Position

There is **no dedicated close-position endpoint**. To close:

1. **One-way mode:** Place a MARKET order with `side` opposite to current position and `quantity` equal to `positionAmt`. Or use `STOP_MARKET`/`TAKE_PROFIT_MARKET` with `closePosition=true`.

2. **Hedge mode:** Place a MARKET order with correct `positionSide` and `reduceOnly=true`. Or use `closePosition=true` on stop/take-profit orders.

Example (close a long position):
```
POST /fapi/v1/order
symbol=BTCUSDT&side=SELL&type=MARKET&quantity=0.010
```

---

### Position Mode — One-Way vs Hedge

**Query position mode (`GET /fapi/v1/positionSide/dual`):**

**Weight:** 30
Parameters: `recvWindow`, `timestamp`.

Response:
```json
{
  "dualSidePosition": true
}
```
- `true` = Hedge mode (LONG + SHORT positions independent).
- `false` = One-way mode (default, single BOTH position per symbol).

**Change position mode (`POST /fapi/v1/positionSide/dual`):**

**Weight:** 1

Parameters:
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `dualSidePosition` | STRING | YES | `"true"` = Hedge mode, `"false"` = One-way mode |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

Response:
```json
{
  "code": 200,
  "msg": "success"
}
```

**Cannot change mode if any open orders or positions exist.**

---

## 4. LEVERAGE AND MARGIN

### Change Leverage (`POST /fapi/v1/leverage`)

**Weight:** 1
**Security:** TRADE

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `leverage` | INT | YES | 1 to 125 (actual max depends on symbol and notional) |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**Response:**
```json
{
  "leverage": 21,
  "maxNotionalValue": "1000000",
  "symbol": "BTCUSDT"
}
```

**No dedicated GET leverage endpoint.** Current leverage per symbol is returned in:
- `GET /fapi/v2/positionRisk` (field: `leverage`)
- `GET /fapi/v2/account` (field: `positions[].leverage`)

---

### Get Leverage Brackets (`GET /fapi/v1/leverageBracket`)

**Weight:** 1
**Security:** USER_DATA

**Parameters:** `symbol` (optional), `recvWindow`, `timestamp`.

**Response (array when no symbol):**
```json
[
  {
    "symbol": "ETHUSDT",
    "notionalCoef": 1.50,
    "brackets": [
      {
        "bracket": 1,
        "initialLeverage": 75,
        "notionalCap": 10000,
        "notionalFloor": 0,
        "maintMarginRatio": 0.0065,
        "cum": 0.0
      },
      {
        "bracket": 2,
        "initialLeverage": 50,
        "notionalCap": 25000,
        "notionalFloor": 10000,
        "maintMarginRatio": 0.01,
        "cum": 34.5
      }
    ]
  }
]
```

**Field descriptions:**
- `initialLeverage`: Max leverage for this bracket.
- `notionalCap`: Max notional to be in this bracket.
- `notionalFloor`: Min notional for this bracket.
- `maintMarginRatio`: Maintenance margin ratio.
- `cum`: Ignore (computed constant for liquidation price).
- `notionalCoef`: Bracket multiplier vs default leverage bracket.

---

### Change Margin Type (`POST /fapi/v1/marginType`)

**Weight:** 1
**Security:** TRADE

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `marginType` | ENUM | YES | `ISOLATED` or `CROSSED` |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**Response:**
```json
{
  "code": 200,
  "msg": "success"
}
```

**Cannot change margin type if a position or open order exists for that symbol.**

Current margin type per symbol is in `GET /fapi/v2/positionRisk` (field: `marginType`).

---

### Modify Isolated Position Margin (`POST /fapi/v1/positionMargin`)

Add or reduce margin on an isolated position.

**Weight:** 1
**Security:** TRADE

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `positionSide` | ENUM | NO | `BOTH` (one-way), `LONG`, `SHORT` (hedge mode) |
| `amount` | DECIMAL | YES | Amount to add/reduce |
| `type` | INT | YES | `1` = Add margin, `2` = Reduce margin |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

---

### Position Margin Change History (`GET /fapi/v1/positionMargin/history`)

**Weight:** 1
**Security:** USER_DATA

**Parameters:** `symbol` (required), `type` (1 or 2), `startTime`, `endTime`, `limit` (max 500), `recvWindow`, `timestamp`.

---

## 5. MARGIN TRADING (SPOT MARGIN)

### Borrow / Repay (`POST /sapi/v1/margin/borrow-repay`)

**Weight:** 1500 (UID-based)
**Security:** MARGIN

**Note:** Old endpoints `POST /sapi/v1/margin/loan` and `POST /sapi/v1/margin/repay` are **deprecated** as of 2024-03-31. Use this unified endpoint.

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `asset` | STRING | YES | Asset to borrow/repay |
| `isIsolated` | STRING | YES | `"TRUE"` for isolated, `"FALSE"` for cross margin |
| `symbol` | STRING | Conditional | Required for isolated margin |
| `amount` | STRING | YES | Amount |
| `type` | STRING | YES | `"BORROW"` or `"REPAY"` |
| `recvWindow` | LONG | NO | Max 60000 |
| `timestamp` | LONG | YES | |

**Response:**
```json
{
  "tranId": 100000001
}
```

---

### Query Borrow/Repay History (`GET /sapi/v1/margin/borrow-repay`)

**Parameters:** `asset`, `isIsolated`, `symbol`, `txId`, `startTime`, `endTime`, `current` (page), `size` (max 100), `recvWindow`, `timestamp`.

**Notes:**
- Either `txId` or `startTime` must be sent.
- History within last 6 months only.

---

### Max Borrowable (`GET /sapi/v1/margin/maxBorrowable`)

**Weight:** 50
**Parameters:** `asset` (required), `isolatedSymbol` (optional), `recvWindow`, `timestamp`.

**Response:**
```json
{
  "amount": "1.69248805",
  "borrowLimit": "60"
}
```

---

### Max Transferable (`GET /sapi/v1/margin/maxTransferable`)

**Weight:** 50
**Parameters:** `asset` (required), `isolatedSymbol` (optional), `recvWindow`, `timestamp`.

---

### Cross vs Isolated Margin Differences

| Feature | Cross Margin | Isolated Margin |
|---------|-------------|-----------------|
| Margin pool | Shared across all pairs | Per trading pair |
| Liquidation | Can use entire balance | Only isolated balance |
| API `isIsolated` | `"FALSE"` | `"TRUE"` |
| Symbol required for borrow | No | Yes |
| Account endpoint | `/sapi/v1/margin/account` | `/sapi/v1/margin/isolated/account` |
| Transfer endpoint | `/sapi/v1/asset/transfer` with MAIN_MARGIN type | with ISOLATEDMARGIN types |

---

## 6. TRANSFERS

### Universal Transfer (`POST /sapi/v1/asset/transfer`)

**Weight:** 900 (UID-based)
**Security:** USER_DATA (requires `permitsUniversalTransfer` API key permission)

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `type` | ENUM | YES | Transfer type (see list below) |
| `asset` | STRING | YES | Asset symbol |
| `amount` | DECIMAL | YES | |
| `fromSymbol` | STRING | Conditional | Required for ISOLATEDMARGIN_MARGIN and ISOLATEDMARGIN_ISOLATEDMARGIN |
| `toSymbol` | STRING | Conditional | Required for MARGIN_ISOLATEDMARGIN and ISOLATEDMARGIN_ISOLATEDMARGIN |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**Response:**
```json
{
  "tranId": 13526853623
}
```

**Transfer Types (complete list):**

| Type | Direction |
|------|-----------|
| `MAIN_UMFUTURE` | Spot → USDM Futures |
| `MAIN_CMFUTURE` | Spot → COIN-M Futures |
| `MAIN_MARGIN` | Spot → Cross Margin |
| `MAIN_FUNDING` | Spot → Funding |
| `MAIN_OPTION` | Spot → Options |
| `MAIN_PORTFOLIO_MARGIN` | Spot → Portfolio Margin |
| `UMFUTURE_MAIN` | USDM Futures → Spot |
| `UMFUTURE_MARGIN` | USDM Futures → Cross Margin |
| `UMFUTURE_FUNDING` | USDM Futures → Funding |
| `UMFUTURE_OPTION` | USDM Futures → Options |
| `CMFUTURE_MAIN` | COIN-M Futures → Spot |
| `CMFUTURE_MARGIN` | COIN-M Futures → Cross Margin |
| `CMFUTURE_FUNDING` | COIN-M Futures → Funding |
| `MARGIN_MAIN` | Cross Margin → Spot |
| `MARGIN_UMFUTURE` | Cross Margin → USDM Futures |
| `MARGIN_CMFUTURE` | Cross Margin → COIN-M Futures |
| `MARGIN_FUNDING` | Cross Margin → Funding |
| `MARGIN_OPTION` | Cross Margin → Options |
| `ISOLATEDMARGIN_MARGIN` | Isolated Margin → Cross Margin |
| `MARGIN_ISOLATEDMARGIN` | Cross Margin → Isolated Margin |
| `ISOLATEDMARGIN_ISOLATEDMARGIN` | Isolated → Isolated (different pairs) |
| `FUNDING_MAIN` | Funding → Spot |
| `FUNDING_UMFUTURE` | Funding → USDM Futures |
| `FUNDING_MARGIN` | Funding → Cross Margin |
| `FUNDING_CMFUTURE` | Funding → COIN-M Futures |
| `FUNDING_OPTION` | Funding → Options |
| `OPTION_MAIN` | Options → Spot |
| `OPTION_UMFUTURE` | Options → USDM Futures |
| `OPTION_MARGIN` | Options → Cross Margin |
| `OPTION_FUNDING` | Options → Funding |
| `PORTFOLIO_MARGIN_MAIN` | Portfolio Margin → Spot |

---

### Query Transfer History (`GET /sapi/v1/asset/transfer`)

**Weight:** 1
**Parameters:** `type` (required), `startTime`, `endTime`, `current` (page, default 1), `size` (max 100), `fromSymbol`, `toSymbol`, `recvWindow`, `timestamp`.

---

## 7. RISK AND ACCOUNT METRICS

### Futures Commission Rate (`GET /fapi/v1/commissionRate`)

**Weight:** 20
**Security:** USER_DATA

**Parameters:** `symbol` (required), `recvWindow`, `timestamp`.

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "makerCommissionRate": "0.0002",
  "takerCommissionRate": "0.0004",
  "rpiCommissionRate": "0.00005"
}
```
- `rpiCommissionRate`: RPI (Retail Price Index) commission rate, added 2025.

---

### Futures Income History (`GET /fapi/v1/income`)

**Weight:** 30
**Security:** USER_DATA

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | NO | Filter by symbol |
| `incomeType` | STRING | NO | Filter by type (see enum below) |
| `startTime` | LONG | NO | Inclusive; default = recent 7 days |
| `endTime` | LONG | NO | Inclusive |
| `page` | INT | NO | Pagination |
| `limit` | INT | NO | Default 100, max 1000 |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**`incomeType` Enum Values:**
`TRANSFER`, `WELCOME_BONUS`, `REALIZED_PNL`, `FUNDING_FEE`, `COMMISSION`, `INSURANCE_CLEAR`, `REFERRAL_KICKBACK`, `COMMISSION_REBATE`, `API_REBATE`, `CONTEST_REWARD`, `CROSS_COLLATERAL_TRANSFER`, `OPTIONS_PREMIUM_FEE`, `OPTIONS_SETTLE_PROFIT`, `INTERNAL_TRANSFER`, `AUTO_EXCHANGE`, `DELIVERED_SETTELMENT`, `COIN_SWAP_DEPOSIT`, `COIN_SWAP_WITHDRAW`, `POSITION_LIMIT_INCREASE_FEE`, `STRATEGY_UMFUTURES_TRANSFER`, `FEE_RETURN`, `BFUSD_REWARD`

**Response:**
```json
[
  {
    "symbol": "",
    "incomeType": "TRANSFER",
    "income": "-0.37500000",
    "asset": "USDT",
    "info": "TRANSFER",
    "time": 1570608000000,
    "tranId": 9689322392,
    "tradeId": ""
  },
  {
    "symbol": "BTCUSDT",
    "incomeType": "COMMISSION",
    "income": "-0.01000000",
    "asset": "USDT",
    "info": "COMMISSION",
    "time": 1570636800000,
    "tranId": 9689322392,
    "tradeId": "2059192"
  }
]
```

**Notes:**
- Last 3 months of data only.
- If neither startTime nor endTime sent, returns last 7 days.

---

### API Key Permissions (`GET /sapi/v1/account/apiRestrictions`)

**Weight:** 1

**Response:**
```json
{
  "ipRestrict": false,
  "createTime": 1698645219000,
  "enableReading": true,
  "enableSpotAndMarginTrading": false,
  "enableWithdrawals": false,
  "enableInternalTransfer": false,
  "permitsUniversalTransfer": false,
  "enableVanillaOptions": false,
  "enableFutures": false,
  "enableMargin": false,
  "enablePortfolioMarginTrading": false
}
```

---

### Futures Leverage Brackets (`GET /fapi/v1/leverageBracket`)

See Section 4 above.

---

## 8. DEPOSITS AND WITHDRAWALS (READ-ONLY)

### Deposit Address (`GET /sapi/v1/capital/deposit/address`)

**Weight:** 10
**Security:** USER_DATA

**Parameters:** `coin` (required), `network` (optional), `amount` (optional), `recvWindow`, `timestamp`.

**Response:**
```json
{
  "address": "1HPn8Rx2y...",
  "coin": "BTC",
  "tag": "",
  "url": "https://btc.com/1HPn8Rx2y..."
}
```

---

### Deposit History (`GET /sapi/v1/capital/deposit/hisrec`)

**Weight:** 1
**Security:** USER_DATA

**Parameters:** `coin`, `status` (0=pending, 6=credited, 1=success), `startTime`, `endTime`, `offset`, `limit` (max 1000), `txId`, `recvWindow`, `timestamp`.

**Response fields:** `id`, `amount`, `coin`, `network`, `status`, `address`, `addressTag`, `txId`, `insertTime`, `transferType` (0=external, 1=internal), `confirmTimes`, `unlockConfirm`, `walletType`.

---

### Withdrawal History (`GET /sapi/v1/capital/withdraw/history`)

**Weight:** 1
**Security:** USER_DATA

**Parameters:** `coin`, `withdrawOrderId`, `status` (0=email_sent, 1=cancelled, 2=awaiting_approval, 3=rejected, 4=processing, 5=failure, 6=completed), `offset`, `limit` (max 1000), `startTime`, `endTime`, `recvWindow`, `timestamp`.

---

### Supported Networks / Deposit Config (`GET /sapi/v1/capital/config/getall`)

**Weight:** 10

Returns all coins with their networks, deposit/withdraw capabilities, min/max amounts, fees, and confirmation requirements.

---

## Sources

- [Binance Spot Account Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints)
- [Binance Futures Account Balance V2](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Futures-Account-Balance-V2)
- [Binance Futures Account Information V2](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V2)
- [Binance Futures Account Information V3](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3)
- [Binance Futures Position Information V2](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-Information-V2)
- [Binance Futures Change Initial Leverage](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Initial-Leverage)
- [Binance Futures Leverage Brackets](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Notional-and-Leverage-Brackets)
- [Binance Futures Change Position Mode](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Position-Mode)
- [Binance Futures User Commission Rate](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/User-Commission-Rate)
- [Binance Futures Income History](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Income-History)
- [Binance Margin Borrow/Repay](https://developers.binance.com/docs/margin_trading/borrow-and-repay/Margin-Account-Borrow-Repay)
- [Binance Universal Transfer](https://developers.binance.com/docs/wallet/asset/user-universal-transfer)
- [Binance API Key Permissions](https://developers.binance.com/docs/wallet/account/api-key-permission)
