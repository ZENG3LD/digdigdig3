# Bybit V5 Account & Position API

Sources:
- https://bybit-exchange.github.io/docs/v5/account/wallet-balance
- https://bybit-exchange.github.io/docs/v5/account/account-info
- https://bybit-exchange.github.io/docs/v5/position
- https://bybit-exchange.github.io/docs/v5/position/set-leverage
- https://bybit-exchange.github.io/docs/v5/position/trading-stop

---

## 1. ACCOUNT TYPES

### Unified Trading Account (UTA)

Bybit's primary account type since 2022. Key characteristics:
- **Single margin pool**: Spot, USDT Perpetual, USDC Perpetual, and Options share one collateral pool
- **Cross-collateralization**: BTC, ETH, and other assets can serve as margin for derivatives
- **Three margin modes** (configurable per UTA account):
  - `ISOLATED_MARGIN` — per-position isolated margin
  - `REGULAR_MARGIN` (cross margin) — shared margin across positions in same category
  - `PORTFOLIO_MARGIN` — unified risk across all products, requires minimum equity

UTA accounts have `unifiedMarginStatus` field in account info response.

### Classic Account

The legacy account structure with separate sub-accounts:
- `SPOT` account — spot trading only
- `CONTRACT` account — derivatives trading only
- No cross-collateralization between sub-accounts
- Requires explicit transfers between sub-accounts to move funds

### Account Type Identifiers Used in API

| Value | Used In | Meaning |
|-------|---------|---------|
| `UNIFIED` | wallet-balance, transfers | UTA unified account |
| `CONTRACT` | wallet-balance, transfers | Classic derivatives account |
| `SPOT` | wallet-balance, transfers | Classic spot account |
| `FUND` | transfers | Funding/asset wallet |
| `OPTION` | transfers | Options sub-account (classic) |

### `category` vs `accountType`

These are separate concepts:
- `category` = product type used in ORDER and POSITION endpoints: `spot`, `linear`, `inverse`, `option`
- `accountType` = account sub-type used in ACCOUNT and TRANSFER endpoints: `UNIFIED`, `CONTRACT`, `SPOT`

---

## 2. BALANCE & ACCOUNT INFO

### GET /v5/account/wallet-balance

Retrieves wallet balance for a given account type.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `accountType` | string | YES | `UNIFIED` — primary value for UTA. For funding wallet use dedicated endpoint. |
| `coin` | string | NO | Specific coin filter (e.g. `BTC`, `USDT`). Comma-separated for multiple. |

NOTE: For **classic accounts**, use `CONTRACT` or `SPOT` as `accountType`. For **funding wallet**, use the dedicated asset endpoint `/v5/asset/transfer/query-asset-info`.

**Account-level response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `accountType` | string | Account type |
| `accountIMRate` | string | Initial margin rate of the account |
| `accountMMRate` | string | Maintenance margin rate |
| `totalEquity` | string | Total equity in USD |
| `totalWalletBalance` | string | Total wallet balance in USD |
| `totalMarginBalance` | string | Total margin balance in USD |
| `totalAvailableBalance` | string | Available balance for opening new positions |
| `totalPerpUPL` | string | Total unrealized PnL from perpetuals |
| `totalInitialMargin` | string | Total initial margin required |
| `totalMaintenanceMargin` | string | Total maintenance margin required |

**Coin-level response fields (within `coin` array):**

| Field | Type | Description |
|-------|------|-------------|
| `coin` | string | Coin name (e.g. `USDT`, `BTC`) |
| `equity` | string | Coin equity |
| `usdValue` | string | USD value of coin equity |
| `walletBalance` | string | Total wallet balance for this coin |
| `locked` | string | Locked by open orders |
| `spotHedgingQty` | string | Spot hedging quantity (UTA spot hedging) |
| `borrowAmount` | string | Total borrow amount |
| `accruedInterest` | string | Accrued borrow interest |
| `totalOrderIM` | string | Initial margin used by open orders |
| `totalPositionIM` | string | Initial margin used by open positions |
| `totalPositionMM` | string | Maintenance margin used by positions |
| `unrealisedPnl` | string | Unrealized PnL |
| `cumRealisedPnl` | string | Cumulative realized PnL |
| `bonus` | string | Trial fund bonus amount |
| `marginCollateral` | string | Whether coin can be used as margin collateral |
| `collateralSwitch` | boolean | Whether collateral is enabled for this coin |
| `spotBorrow` | string | Spot borrow amount |
| `availableToWithdraw` | string | DEPRECATED — use `availableBalance` |
| `availableToBorrow` | string | DEPRECATED |

**Response structure:**
```json
{
  "retCode": 0,
  "result": {
    "list": [
      {
        "accountType": "UNIFIED",
        "accountIMRate": "0.02",
        "totalEquity": "10500.5",
        "totalWalletBalance": "10000.0",
        "totalAvailableBalance": "8000.0",
        "coin": [
          {
            "coin": "USDT",
            "walletBalance": "5000.0",
            "equity": "5200.0",
            "unrealisedPnl": "200.0",
            "locked": "100.0",
            "borrowAmount": "0",
            "usdValue": "5200.0"
          },
          {
            "coin": "BTC",
            "walletBalance": "0.1",
            "equity": "0.1",
            "usdValue": "5300.5",
            "marginCollateral": "true"
          }
        ]
      }
    ]
  }
}
```

---

### GET /v5/account/info

No request parameters required.

**Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `unifiedMarginStatus` | integer | Account upgrade status (1=Classic, 2=UTA1.0, 3=UTA Pro, 4=UTA2.0) |
| `marginMode` | string | `ISOLATED_MARGIN`, `REGULAR_MARGIN`, or `PORTFOLIO_MARGIN` |
| `isMasterTrader` | boolean | Whether account is a copy trading leader |
| `spotHedgingStatus` | string | `ON` or `OFF` — UTA spot hedging feature |
| `dcpStatus` | string | DEPRECATED, always `OFF` |
| `smpGroup` | integer | DEPRECATED, always `0` |
| `updatedTime` | string | Last update timestamp (ms) |

---

## 3. POSITIONS

### GET /v5/position/list

**Supported categories:** `linear`, `inverse`, `option` (NOT spot — spot has no positions concept)

**Parameters:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear`, `inverse`, or `option` |
| `symbol` | NO | Filter by symbol. For linear: symbol OR settleCoin required |
| `baseCoin` | NO | Base coin filter (options only) |
| `settleCoin` | NO | Settle coin filter |
| `limit` | NO | [1-200], default 20 |
| `cursor` | NO | Pagination cursor |

**Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `positionIdx` | integer | `0` = one-way mode; `1` = buy-side hedge; `2` = sell-side hedge |
| `symbol` | string | Trading pair |
| `side` | string | `Buy` (long), `Sell` (short), or `""` (empty = no position) |
| `size` | string | Position size (always positive) |
| `avgPrice` | string | Average entry price |
| `unrealisedPnl` | string | Unrealized profit/loss |
| `cumRealisedPnl` | string | Cumulative realized PnL |
| `leverage` | string | Current leverage (empty for portfolio margin) |
| `liqPrice` | string | Liquidation price (empty when invalid or portfolio margin) |
| `bustPrice` | string | Bankruptcy price |
| `markPrice` | string | Current mark price |
| `positionStatus` | string | `Normal`, `Liq` (liquidation in progress), `Adl` (auto-deleveraging) |
| `tradeMode` | integer | DEPRECATED, always `0` |
| `autoAddMargin` | integer | `0` = disabled, `1` = enabled (isolated margin auto top-up) |
| `positionIM` | string | Initial margin of position |
| `positionMM` | string | Maintenance margin of position |
| `takeProfit` | string | Current TP price |
| `stopLoss` | string | Current SL price |
| `trailingStop` | string | Trailing stop distance |
| `sessionAvgPrice` | string | USDC contract session average price |
| `delta` | string | Delta (options) |
| `gamma` | string | Gamma (options) |
| `vega` | string | Vega (options) |
| `theta` | string | Theta (options) |
| `createdTime` | string | First position creation timestamp (ms) |
| `updatedTime` | string | Last position update timestamp (ms) |
| `seq` | long | Cross-sequence number for ordering |

### Position Mode

**One-way mode (MergedSingle):** `positionIdx = 0` — only one position per symbol, long or short.

**Hedge mode (BothSide):** `positionIdx = 1` (long) and `positionIdx = 2` (short) — can hold both directions simultaneously.

### POST /v5/position/switch-mode (switch-mode endpoint)

Actually: `POST /v5/position/switch-mode`

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | Conditional | Symbol name. Required if `coin` not specified |
| `coin` | Conditional | Settlement coin. Required if `symbol` not specified |
| `mode` | YES | `0` = One-way mode (MergedSingle); `3` = Hedge mode (BothSide) |

**Constraint:** Mode switch only possible when there are no open orders or positions for the symbol.

### Closing a Position

There is no dedicated "close position" endpoint in Bybit V5. Options:

1. **Via `reduceOnly` order:** Place a Market or Limit order with `reduceOnly: true` and `qty` equal to position size.
2. **Via `POST /v5/position/trading-stop`:** Set TP/SL to trigger closure at desired price.

---

## 4. LEVERAGE & MARGIN

### POST /v5/position/set-leverage

**Supported categories:** `linear`, `inverse` ONLY

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | YES | Trading pair |
| `buyLeverage` | YES | Leverage for buy/long side (string, e.g. `"10"`) |
| `sellLeverage` | YES | Leverage for sell/short side (string, e.g. `"10"`) |

**Constraints:**
- One-way mode: `buyLeverage` must equal `sellLeverage`
- Hedge mode with cross margin: `buyLeverage` must equal `sellLeverage`
- Hedge mode with isolated margin: can differ

**Response:** Empty result `{}` on success.

### Leverage in Position Response

The `leverage` field in `/v5/position/list` response shows the current leverage. Empty string when portfolio margin is active.

---

### POST /v5/position/switch-isolated — Switch Cross/Isolated Margin

Path: `POST /v5/position/switch-isolated`

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | YES | Trading pair |
| `tradeMode` | YES | `0` = cross margin; `1` = isolated margin |
| `buyLeverage` | YES | Required even when switching to cross margin |
| `sellLeverage` | YES | Required even when switching to cross margin |

**Constraint:** Cannot switch when there is an open position or open orders on the symbol.

---

### POST /v5/position/set-auto-add-margin

Enables or disables automatic margin addition when position approaches liquidation (isolated margin mode).

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | YES | Trading pair |
| `autoAddMargin` | YES | `0` = disable; `1` = enable |
| `positionIdx` | NO | Required in hedge mode |

---

### POST /v5/position/add-margin — Manual Add/Reduce Margin

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | YES | Trading pair |
| `margin` | YES | Amount to add (positive) or reduce (negative). Supports up to 4 decimals. |
| `positionIdx` | NO | Required in hedge mode: `0`, `1`, or `2` |

**Response:** Returns updated position details including new margin, leverage, liqPrice, etc.

---

## 5. MARGIN TRADING (Spot)

### Spot Margin in UTA

Spot margin trading in UTA does NOT use separate "margin account" — it is done within the Unified account itself.

**Enable margin on spot order:**
- Set `isLeverage: 1` in the order create request
- This signals the order should use margin (borrow if needed)

**Spot margin mode:**
- UTA accounts have a global `marginMode` that controls cross/isolated for all categories
- For spot specifically, the `spotHedgingStatus` can be `ON` or `OFF`

**NOTE:** For **classic accounts**, spot margin is NOT supported through the V5 unified API. Only UTA accounts support spot margin via `isLeverage`.

---

## 6. TRANSFERS

### POST /v5/asset/transfer/inter-transfer — Internal Transfer

Transfers between different account types under the same UID.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `transferId` | YES | UUID (client-generated unique ID for idempotency) |
| `coin` | YES | Coin to transfer (e.g. `USDT`) |
| `amount` | YES | Transfer amount (string) |
| `fromAccountType` | YES | Source account type |
| `toAccountType` | YES | Destination account type |

**`fromAccountType` / `toAccountType` valid values:**

| Value | Description |
|-------|-------------|
| `UNIFIED` | Unified Trading Account |
| `CONTRACT` | Classic derivatives account |
| `SPOT` | Classic spot account |
| `FUND` | Funding/asset wallet |
| `OPTION` | Classic options account |

**Response:**
```json
{
  "retCode": 0,
  "result": {
    "transferId": "42c0cfb0-6bca-c242-bc76-4e6df6cbab16"
  }
}
```

### GET /v5/asset/transfer/query-inter-transfer-list — Transfer History

| Parameter | Required | Description |
|-----------|----------|-------------|
| `transferId` | NO | Filter by specific transfer ID |
| `coin` | NO | Filter by coin |
| `status` | NO | `SUCCESS`, `PENDING`, `FAILED` |
| `startTime` / `endTime` | NO | Timestamp range (ms) |
| `limit` | NO | [1-50], default 20 |
| `cursor` | NO | Pagination cursor |

---

## 7. RISK & METRICS

### GET /v5/account/fee-rate

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `spot`, `linear`, `inverse`, `option` |
| `symbol` | NO | Trading pair (valid for linear, inverse, spot) |
| `baseCoin` | NO | Base coin filter (options only) |

**Response fields (per symbol):**

| Field | Description |
|-------|-------------|
| `symbol` | Trading pair (empty for options) |
| `baseCoin` | Base coin (options only) |
| `takerFeeRate` | Taker fee rate (e.g. `"0.0006"` = 0.06%) |
| `makerFeeRate` | Maker fee rate (e.g. `"0.0001"` = 0.01%) |

**Rate limit:** ~5-10 req/s depending on category.

---

### GET /v5/account/transaction-log

Income, expense, and settlement history for the UNIFIED account.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `accountType` | NO | `UNIFIED` (default) |
| `category` | NO | `spot`, `linear`, `option`, `inverse` |
| `currency` | NO | Coin denomination filter |
| `type` | NO | Transaction type filter |
| `startTime` / `endTime` | NO | Max 7-day span |
| `limit` | NO | [1-50], default 20 |
| `cursor` | NO | Pagination cursor |

**Response fields:**

| Field | Description |
|-------|-------------|
| `transactionTime` | Timestamp (ms) |
| `type` | `TRADE`, `SETTLEMENT`, `DELIVERY`, `BLOCK_TRADE`, `BONUS`, etc. |
| `symbol` | Trading pair |
| `qty` | Quantity change (negative = decrease) |
| `size` | Position size after transaction |
| `currency` | Settlement currency |
| `tradePrice` | Execution price |
| `funding` | Funding fee (positive = received) |
| `fee` | Trading fee (positive = expense) |
| `cashFlow` | Cash change excluding fee |
| `change` | Total change = cashFlow + funding - fee |
| `cashBalance` | Wallet balance after transaction |

**Rate limit:** 30 req/s

---

### GET /v5/user/query-api — API Key Info

Returns information about the API key making the request.

Key fields in response: `id`, `note`, `apiKey`, `readOnly`, `secret` (masked), `permissions` (object with permission categories), `ips` (allowed IP list), `type`, `deadlineDay`, `expiredAt`, `createdAt`, `unified`, `uta`, `userID`, `inviterID`, `vipLevel`, `mktMakerLevel`, `affiliateID`.

**Permission categories in `permissions` field:**

| Category | Description |
|----------|-------------|
| `ContractTrade` | Futures/derivatives order & position management |
| `Spot` | Spot trading |
| `Wallet` | AccountTransfer, SubMemberTransfer, Withdraw |
| `Options` | Options trading |
| `Derivatives` | Derivatives trading |
| `Exchange` | Convert history |
| `Earn` | Earn products |
| `FiatP2P` | P2P trading (master accounts only) |
| `BlockTrade` | Block trade |

`readOnly`: `0` = Read + Write; `1` = Read only

---

## 8. DEPOSITS & WITHDRAWALS

### Deposit Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v5/asset/deposit/query-record` | GET | Deposit history |
| `/v5/asset/deposit/query-address` | GET | Get deposit address for coin/chain |
| `/v5/asset/deposit/query-sub-member-record` | GET | Sub-account deposit history |

### Withdrawal Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v5/asset/withdraw/create` | POST | Submit withdrawal request |
| `/v5/asset/withdraw/query-record` | GET | Withdrawal history |
| `/v5/asset/withdraw/withdraw-address` | GET | Whitelisted withdrawal addresses |
| `/v5/asset/withdraw/cancel` | POST | Cancel pending withdrawal |

**Withdrawal requirements:**
- Wallet address must be whitelisted in the Bybit account settings
- Only master UID API keys can initiate withdrawals
- **Rate limit:** 5 req/s; secondary limit: 1 withdrawal per 10 seconds per chain/coin

**Key withdrawal params:** `coin`, `chain`, `address`, `tag` (memo), `amount`, `accountType` (source account), `feeType` (0=fee deducted from withdrawal amount, 1=fee charged separately)

### GET /v5/position/close-pnl — Closed PnL History

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear` or `inverse` |
| `symbol` | NO | Filter by symbol |
| `startTime` / `endTime` | NO | Max 7-day span per query |
| `limit` | NO | [1-100], default 50 |
| `cursor` | NO | Pagination cursor |

Returns up to 2 years of closed position PnL history.
