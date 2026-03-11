# KuCoin Account API Specification

## CRITICAL: KuCoin Account Architecture

KuCoin uses **completely separate account pools** — unlike exchanges with a unified account. Funds must be explicitly transferred between account types:

```
┌─────────────┐     transfer     ┌─────────────┐
│    MAIN     │ ◄────────────► │    TRADE    │
│  (Funding)  │                 │   (Spot HF) │
└─────────────┘                 └─────────────┘
       │                               │
       │ transfer                      │ transfer
       ▼                               ▼
┌─────────────┐                 ┌─────────────┐
│   MARGIN    │                 │  CONTRACT   │
│  (Spot Mgn) │                 │  (Futures)  │
└─────────────┘                 └─────────────┘
```

Additionally, KuCoin has **Isolated Margin** accounts — one per trading pair (e.g., `BTC-USDT` isolated).

---

## 1. ACCOUNT TYPES

| Account Type | API Name | Use Case |
|-------------|----------|----------|
| Main (Funding) | `main` | Deposits, withdrawals, default landing |
| Trade (Spot) | `trade` | Regular spot trading |
| Spot HF | `trade_hf` | High-frequency spot trading |
| Margin (Cross) | `margin` | Cross-margin trading |
| Isolated Margin | `isolated` | Isolated margin (per pair); also `margin_v2` / `isolated_v2` |
| Futures (Contract) | `contract` | Futures/perpetuals |

**Important**: Spot HF (`trade_hf`) and regular Spot (`trade`) are separate balance pools.

---

## 2. BALANCE & ACCOUNT INFO

### Get Account List (Spot/Margin)
```
GET https://api.kucoin.com/api/v1/accounts
```
**Required Permission**: General

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | No | Filter by currency (e.g., `BTC`) |
| `type` | string | No | Filter by type: `main`, `trade`, `trade_hf`, `margin` |

**Response**:
```json
{
  "code": "200000",
  "data": [
    {
      "id": "5bd6e9286d99522a52e458de",
      "currency": "BTC",
      "type": "main",
      "balance": "237582.04299",
      "available": "237582.04299",
      "holds": "0"
    },
    {
      "id": "5bd6e9216d99522a52e458d6",
      "currency": "BTC",
      "type": "trade",
      "balance": "1234356",
      "available": "1234356",
      "holds": "0"
    }
  ]
}
```

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Account ID |
| `currency` | string | Currency code |
| `type` | string | Account type: `main`, `trade`, `trade_hf`, `margin` |
| `balance` | string | Total balance |
| `available` | string | Available balance (balance - holds) |
| `holds` | string | Funds on hold (frozen for open orders) |

### Get Account Detail (Spot/Margin/HF)
```
GET https://api.kucoin.com/api/v1/accounts/{accountId}
```
Returns same fields as account list but for single account.

### Get Isolated Margin Account (All Pairs)
```
GET https://api.kucoin.com/api/v1/isolated/accounts?balanceCurrency=USDT
```

### Get Margin Account (Cross)
```
GET https://api.kucoin.com/api/v1/margin/account
```

**Response includes**:
- `debtRatio` — current debt ratio
- Per-currency: `currency`, `totalBalance`, `availableBalance`, `holdBalance`, `liability`, `interest`, `borrowEnabled`, `repayEnabled`, `transferEnabled`, `borrowed`, `totalAsset`, `net`, `maxBorrowSize`

---

## 3. FUTURES ACCOUNT

### Get Futures Account Overview
```
GET https://api-futures.kucoin.com/api/v1/account-overview?currency=USDT
```
**Required Permission**: General

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | No | Settlement currency (e.g., `USDT`, `XBT`) |

**Response**:
```json
{
  "code": "200000",
  "data": {
    "accountEquity": 198.733127406,
    "unrealisedPNL": 0.0,
    "marginBalance": 198.733127406,
    "positionMargin": 0.0,
    "orderMargin": 0.0,
    "frozenFunds": 0.0,
    "availableBalance": 198.733127406,
    "currency": "USDT",
    "riskRatio": 0.0,
    "maxWithdrawAmount": 198.733127406
  }
}
```

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `accountEquity` | float | Total equity = marginBalance + unrealisedPNL |
| `unrealisedPNL` | float | Unrealized profit/loss |
| `marginBalance` | float | Margin balance = positionMargin + orderMargin + frozenFunds + availableBalance - unrealisedPNL |
| `positionMargin` | float | Margin allocated to open positions |
| `orderMargin` | float | Margin frozen for open orders |
| `frozenFunds` | float | Frozen for withdrawals/transfers |
| `availableBalance` | float | Available for new orders/withdrawals |
| `currency` | string | Settlement currency |
| `riskRatio` | float | Current risk ratio |
| `maxWithdrawAmount` | float | Maximum withdrawable amount |

---

## 4. POSITIONS

### Get Single Futures Position
```
GET https://api-futures.kucoin.com/api/v1/position?symbol=XBTUSDTM
```
**Required Permission**: General

**Response — Key Fields**:
```json
{
  "code": "200000",
  "data": {
    "id": "5ce3cda60c19fc0d4e9ae7cd",
    "userId": "5ce3cda60c19fc0d4e9ae7ce",
    "symbol": "XBTUSDTM",
    "autoDeposit": false,
    "crossMode": false,
    "delevPercentage": 0.52,
    "openingTimestamp": 1558433191000,
    "currentTimestamp": 1558507727807,
    "currentQty": 1,
    "currentCost": 0.001265,
    "currentComm": 0.0000012635,
    "unrealisedCost": 0.001265,
    "realisedGrossCost": 0.0,
    "realisedCost": 0.0,
    "isOpen": true,
    "markPrice": 7935.01,
    "markValue": 0.00126010,
    "posCost": 0.001265,
    "posCross": 1.2e-7,
    "posInit": 0.0001265,
    "posComm": 0.0000012635,
    "posLoss": 0.0,
    "posMargin": 0.0001277635,
    "posMaint": 0.0000225,
    "maintMargin": 0.0001277635,
    "realisedGrossPnl": 0.0,
    "realisedPnl": 0.0,
    "unrealisedPnl": -0.00000490,
    "unrealisedPnlPcnt": -0.0038,
    "unrealisedRoePcnt": -0.0388,
    "avgEntryPrice": 7909.68,
    "liquidationPrice": 7831.17,
    "bankruptPrice": 7777.09,
    "settleCurrency": "XBT",
    "isInverse": true,
    "maintMarginReq": 0.005,
    "riskLimit": 200,
    "realLeverage": 10.0,
    "crossMode": false,
    "marginMode": "ISOLATED",
    "positionSide": "BOTH",
    "leverage": "10"
  }
}
```

**Key Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Contract symbol |
| `currentQty` | integer | Current position size (contracts; negative = short) |
| `avgEntryPrice` | float | Average entry price |
| `unrealisedPnl` | float | Unrealized PnL in settlement currency |
| `unrealisedPnlPcnt` | float | Unrealized PnL as percentage of cost |
| `unrealisedRoePcnt` | float | Return on equity percentage |
| `realisedPnl` | float | Realized PnL |
| `liquidationPrice` | float | Liquidation price |
| `bankruptPrice` | float | Bankruptcy price |
| `markPrice` | float | Current mark price |
| `isOpen` | boolean | Whether position is open |
| `marginMode` | string | `ISOLATED` or `CROSS` |
| `leverage` | string | Current leverage |
| `realLeverage` | float | Actual leverage based on current price |
| `maintMarginReq` | float | Maintenance margin requirement rate |
| `autoDeposit` | boolean | Auto-deposit margin enabled |
| `settleCurrency` | string | Settlement currency |
| `isInverse` | boolean | Inverse contract flag |

### Get All Futures Positions
```
GET https://api-futures.kucoin.com/api/v1/positions
```
Returns array of position objects (same fields as single position above).

### Get Positions History
```
GET https://api-futures.kucoin.com/api/v1/history-positions?symbol=XBTUSDTM&currentPage=1&pageSize=10
```

### Close Position
No dedicated "close position" endpoint. Close by placing an order with:
- `closeOrder: true` — closes entire position at market
- Or `reduceOnly: true` with matching `side` and `size`

---

## 5. LEVERAGE & MARGIN

### Change Position Leverage
```
POST https://api-futures.kucoin.com/api/v1/position/risk-limit-level/change
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | Yes | Contract symbol |
| `level` | integer | Yes | Risk limit level |

### Add Isolated Margin to Position
```
POST https://api-futures.kucoin.com/api/v1/position/margin/deposit-margin
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | Yes | Contract symbol |
| `bizNo` | string | Yes | Unique business ID |
| `margin` | float | Yes | Margin amount to add |

### Toggle Auto-Deposit Margin
```
POST https://api-futures.kucoin.com/api/v1/position/margin/auto-deposit-status
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | Yes | Contract symbol |
| `status` | boolean | Yes | `true` to enable, `false` to disable |

---

## 6. MARGIN TRADING (Spot)

### Borrow Funds (Cross Margin)
```
POST https://api.kucoin.com/api/v3/margin/borrow
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | Yes | Currency to borrow |
| `size` | string | Yes | Borrow amount |
| `timeInForce` | string | Yes | `IOC` or `FOK` |
| `isIsolated` | boolean | No | `true` for isolated margin |
| `symbol` | string | Cond. | Required if `isIsolated: true` (e.g., `BTC-USDT`) |
| `isHf` | boolean | No | `true` for HF account |

### Repay Funds (Cross Margin)
```
POST https://api.kucoin.com/api/v3/margin/repay
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | Yes | Currency to repay |
| `size` | string | Yes | Repay amount |
| `isIsolated` | boolean | No | `true` for isolated |
| `symbol` | string | Cond. | Required if isolated |
| `isHf` | boolean | No | `true` for HF |

### Get Borrow History
```
GET https://api.kucoin.com/api/v3/margin/borrow?currency=BTC&isIsolated=false
```

### Cross vs Isolated Margin
- **Cross Margin**: Shares full account balance as margin. All cross-margin assets support each other.
- **Isolated Margin**: Each trading pair has its own margin pool. Losses limited to pair's margin.

---

## 7. TRANSFERS

### Universal Transfer (Flex Transfer) — CURRENT
```
POST https://api.kucoin.com/api/v3/accounts/universal-transfer
```
**Required Permission**: FlexTransfers

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | Yes | Unique client transfer ID |
| `currency` | string | Yes | Currency to transfer |
| `amount` | string | Yes | Transfer amount |
| `fromAccountType` | string | Yes | Source account type |
| `toAccountType` | string | Yes | Destination account type |
| `type` | string | Yes | `INTERNAL` (within account), `PARENT_TO_SUB`, `SUB_TO_PARENT` |
| `fromUserId` | string | Cond. | Required for `SUB_TO_PARENT` |
| `toUserId` | string | Cond. | Required for `PARENT_TO_SUB` |
| `fromTag` | string | Cond. | Required if `fromAccountType` is `ISOLATED`/`ISOLATED_V2` (trading pair, e.g. `BTC-USDT`) |
| `toTag` | string | Cond. | Required if `toAccountType` is `ISOLATED`/`ISOLATED_V2` |

**Account Type Values**: `MAIN`, `TRADE`, `CONTRACT`, `MARGIN`, `ISOLATED`, `MARGIN_V2`, `ISOLATED_V2`

### Legacy Inner Transfer (Deprecated but still functional)
```
POST https://api.kucoin.com/api/v2/accounts/inner-transfer
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | Yes | Unique client ID |
| `currency` | string | Yes | Currency |
| `amount` | string | Yes | Amount |
| `from` | string | Yes | Source account: `main`, `trade`, `margin`, `isolated`, `contract` |
| `to` | string | Yes | Destination account |
| `fromTag` | string | Cond. | Trading pair for `isolated` (e.g., `BTC-USDT`) |
| `toTag` | string | Cond. | Trading pair for `isolated` |

**Rate Limit Weight**: 10

### Transfer TO Futures Account (Legacy)
```
POST https://api.kucoin.com/api/v1/accounts/sub-transfer
POST https://api-futures.kucoin.com/api/v1/transfer-out  # Transfer FROM futures
```

---

## 8. RISK & METRICS

### Get Spot Account Fees
```
GET https://api.kucoin.com/api/v1/trade-fees?symbols=BTC-USDT,ETH-USDT
```
**Response**:
```json
{
  "code": "200000",
  "data": [
    {
      "symbol": "BTC-USDT",
      "takerFeeRate": "0.001",
      "makerFeeRate": "0.001"
    }
  ]
}
```

### Get Basic User Fee (Spot/Margin)
```
GET https://api.kucoin.com/api/v1/base-fee?currencyType=0
```
`currencyType`: `0` for crypto-to-crypto, `1` for FIAT-to-crypto

### Get Futures Funding Rate History
```
GET https://api-futures.kucoin.com/api/v1/funding-history?symbol=XBTUSDTM&startAt=&endAt=&currentPage=1&pageSize=50
```

### Get Spot Fills (Trade History)
```
GET https://api.kucoin.com/api/v1/hf/fills?symbol=BTC-USDT&side=buy&type=limit&startAt=&endAt=&lastId=&limit=100
```

### Get Futures Trade History
```
GET https://api-futures.kucoin.com/api/v1/fills?symbol=XBTUSDTM&side=buy&type=limit&startAt=&endAt=&currentPage=1&pageSize=50
```

### Get Account Ledger (Spot)
```
GET https://api.kucoin.com/api/v1/accounts/ledgers?currency=BTC&direction=out&bizType=TRADE&startAt=&endAt=&currentPage=1&pageSize=50
```

---

## 9. DEPOSITS & WITHDRAWALS

### Get Deposit Address
```
GET https://api.kucoin.com/api/v2/deposit-addresses?currency=BTC&chain=BTC
```
**Response**:
```json
{
  "code": "200000",
  "data": {
    "address": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
    "memo": "",
    "chain": "BTC",
    "contractAddress": ""
  }
}
```

### Get Deposit List
```
GET https://api.kucoin.com/api/v1/deposits?currency=BTC&startAt=&endAt=&status=SUCCESS&currentPage=1&pageSize=50
```

**Status values**: `PROCESSING`, `SUCCESS`, `FAILURE`

### Request Withdrawal
```
POST https://api.kucoin.com/api/v1/withdrawals
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | Yes | Currency code |
| `address` | string | Yes | Withdrawal address |
| `amount` | float | Yes | Withdrawal amount |
| `memo` | string | No | Address memo/tag |
| `chain` | string | No | Chain type (e.g., `ETH`, `TRC20`) |
| `remark` | string | No | Remarks |
| `feeDeductType` | string | No | `INTERNAL` or `EXTERNAL` |

### Get Withdrawal List
```
GET https://api.kucoin.com/api/v1/withdrawals?currency=BTC&status=SUCCESS&currentPage=1&pageSize=50
```

---

## Sources
- [KuCoin Get Account List - docs-new](https://www.kucoin.com/docs-new/rest/account-info/account-funding/get-account-list-spot)
- [KuCoin Get Account Detail (Spot/Margin/HF)](https://www.kucoin.com/docs/rest/account/basic-info/get-account-detail-spot-margin-trade_hf)
- [KuCoin Get Account - Futures - docs-new](https://www.kucoin.com/docs-new/rest/account-info/account-funding/get-account-futures)
- [KuCoin Get Position List - docs-new](https://www.kucoin.com/docs-new/rest/futures-trading/positions/get-position-list)
- [KuCoin Get Position List (legacy)](https://www.kucoin.com/docs/rest/futures-trading/positions/get-position-list)
- [KuCoin Flex Transfer - docs-new](https://www.kucoin.com/docs-new/rest/account-info/transfer/flex-transfer)
- [KuCoin Inner Transfer (legacy)](https://www.kucoin.com/docs/rest/funding/transfer/inner-transfer)
- [KuCoin FlexTransfer (legacy)](https://www.kucoin.com/docs/rest/funding/transfer/flextransfer)
- [KuCoin Transfer to Futures](https://www.kucoin.com/docs/rest/funding/transfer/transfer-to-futures-account)
- [KuCoin Futures Account Overview](https://www.kucoin.com/docs/rest/funding/funding-overview/get-account-detail-futures)
