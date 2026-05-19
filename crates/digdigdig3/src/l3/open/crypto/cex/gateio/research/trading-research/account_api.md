# Gate.io APIv4 — Account API Reference

**Base URL:** `https://api.gateio.ws/api/v4`

---

## 1. Account Types Overview

Gate.io has multiple account types, each with separate balances:

| Account Type | API Label | Description |
|---|---|---|
| Spot | `spot` | Default trading account |
| Margin (Isolated) | `margin` | Per-pair isolated margin |
| Cross Margin | `cross_margin` | Shared margin across pairs |
| Unified | `unified` | Combined spot + margin + futures in one account |
| Futures USDT-settled | `futures` (settle=`usdt`) | USDT perpetual swaps |
| Futures BTC-settled | `futures` (settle=`btc`) | BTC coin-margined perpetual swaps |
| Delivery Futures | `delivery` | Fixed-expiry futures |
| Options | `options` | Options contracts |

> The **Unified Account** is Gate.io's modern consolidated account that combines spot, margin, and futures positions under a single margin pool.

---

## 2. Spot Account

### 2.1 Get Spot Account Balances

```
GET /spot/accounts
```

**Query Params:**
- `currency` (optional): filter by specific currency (e.g. `"BTC"`)

**Response:** Array of SpotAccount objects.

**SpotAccount fields:**

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency symbol (e.g. `"BTC"`, `"USDT"`) |
| `available` | string | Available balance for trading |
| `locked` | string | Locked balance (in open orders) |
| `update_id` | int64 | Version number for consistency checks |

**Example Response:**
```json
[
  {
    "currency": "BTC",
    "available": "0.5",
    "locked": "0.1",
    "update_id": 1234567890
  },
  {
    "currency": "USDT",
    "available": "10000.00",
    "locked": "2000.00",
    "update_id": 1234567891
  }
]
```

---

### 2.2 Spot Fee Rates

```
GET /spot/fee
```

**Query Params:**
- `currency_pair` (optional): get fee for specific pair

**Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | int64 | User ID |
| `taker_fee` | string | Taker fee rate (e.g. `"0.001"` = 0.1%) |
| `maker_fee` | string | Maker fee rate |
| `gt_discount` | bool | Whether GT discount is applied |
| `gt_taker_fee` | string | GT discounted taker fee |
| `gt_maker_fee` | string | GT discounted maker fee |
| `loan_fee` | string | Loan fee rate (margin) |
| `point_type` | string | POINT token type |
| `currency_pair` | string | Pair for pair-specific fee |
| `derivs_taker_fee` | string | Derivatives taker fee |
| `derivs_maker_fee` | string | Derivatives maker fee |

---

## 3. Margin Account (Isolated)

### 3.1 List Margin Accounts

```
GET /margin/accounts
```

**Query Params:**
- `currency_pair` (optional): filter by pair

Returns per-pair isolated margin account details including borrowed amounts and interest.

### 3.2 Borrow (Isolated Margin)

```
POST /margin/loans
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency to borrow |
| `amount` | string | Borrow amount |
| `currency_pair` | string | Trading pair (isolated margin) |
| `side` | string | `"lend"` or `"borrow"` |
| `rate` | string | Interest rate (for lending) |
| `days` | int32 | Loan duration days (for lending) |

### 3.3 Repay (Isolated Margin)

```
DELETE /margin/loans/{loan_id}
```

Or via `auto_repay` field in order placement.

### 3.4 Unified/Cross Margin Borrow or Repay

```
POST /unified/loans
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency to borrow/repay |
| `amount` | string | Amount |
| `type` | string | `"borrow"` or `"repay"` |

```
GET /unified/loans
```

Query current loans in unified account.

```
GET /unified/interest_records
```

Query interest deduction history.

---

## 4. Cross Margin Account

### 4.1 Get Cross Margin Account

```
GET /margin/cross/accounts
```

Returns cross-margin balance with collateral details.

### 4.2 Cross Margin Borrow

```
POST /margin/cross/loans
```

### 4.3 Cross Margin Repay

```
POST /margin/cross/repayments
```

---

## 5. Unified Account

### 5.1 Get Unified Account Balances

```
GET /unified/accounts
```

Returns combined balances across all sub-accounts.

**Response fields (partial — key fields):**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | int64 | User ID |
| `refresh_time` | int64 | Last refresh timestamp |
| `locked` | bool | Account locked status |
| `balances` | object | Map of currency to balance details |
| `total` | string | Total equity in USDT |
| `borrowed` | string | Total borrowed amount |
| `total_initial_margin` | string | Total initial margin |
| `total_margin_balance` | string | Total margin balance |
| `total_maintenance_margin` | string | Total maintenance margin |
| `total_initial_margin_rate` | string | IMR ratio |
| `total_maintenance_margin_rate` | string | MMR ratio |
| `total_available_margin` | string | Available margin |
| `unified_account_total` | string | Total portfolio value |
| `unified_account_total_liab` | string | Total liabilities |
| `unified_account_total_equity` | string | Total equity |
| `leverage` | string | Current leverage |
| `spot_order_loss` | string | Loss from spot orders |
| `spot_hedge` | bool | Spot hedging enabled |
| `is_all_collateral` | bool | All currencies used as collateral |

Each entry in `balances` map:

| Field | Type | Description |
|-------|------|-------------|
| `available` | string | Available amount |
| `freeze` | string | Frozen/locked amount |
| `borrowed` | string | Borrowed amount |
| `negative_liab` | string | Negative balance liability |
| `futures_pos_liab` | string | Futures position liability |
| `equity` | string | Currency equity value |
| `total_freeze` | string | Total frozen |
| `total_liab` | string | Total liabilities |
| `spot_in_use` | string | Spot used as margin |
| `margin_freeze` | string | Margin frozen |
| `enabled_collateral` | bool | Whether currency is used as collateral |

---

## 6. Futures Account

### 6.1 Get Futures Account (USDT-settled)

```
GET /futures/usdt/accounts
```

### 6.2 Get Futures Account (BTC-settled)

```
GET /futures/btc/accounts
```

**FuturesAccount response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total` | string | Total balance (deposits + withdrawals + P&L) |
| `unrealised_pnl` | string | Unrealized profit/loss |
| `position_margin` | string | Margin used by open positions |
| `order_margin` | string | Margin reserved for pending orders |
| `available` | string | Available balance (includes bonuses) |
| `point` | string | POINT card balance |
| `currency` | string | Settlement currency (`"USDT"` or `"BTC"`) |
| `in_dual_mode` | bool | Whether dual position mode is active |
| `enable_credit` | bool | Portfolio margin (credit) enabled |
| `position_initial_margin` | string | Initial margin for positions |
| `maintenance_margin` | string | Maintenance margin requirement |
| `bonus` | string | Bonus balance |
| `enable_evolved_classic` | bool | New classic margin mode |
| `cross_order_margin` | string | Cross-margin reserved for orders |
| `cross_initial_margin` | string | Cross-margin initial margin |
| `cross_maintenance_margin` | string | Cross-margin maintenance margin |
| `cross_unrealised_pnl` | string | Cross-margin unrealized P&L |
| `cross_available` | string | Cross-margin available balance |
| `isolated_position_margin` | string | Isolated position margin |
| `enable_new_dual_mode` | bool | New dual mode enabled |
| `margin_mode` | int32 | `0` = classic, `1` = cross-currency, `2` = combined |
| `history` | object | FuturesAccountHistory sub-object |

**Example Response:**
```json
{
  "total": "15000.5",
  "unrealised_pnl": "120.3",
  "position_margin": "2000.0",
  "order_margin": "500.0",
  "available": "12380.2",
  "currency": "USDT",
  "in_dual_mode": false,
  "maintenance_margin": "100.5",
  "margin_mode": 0
}
```

---

## 7. Positions

### 7.1 Get All Open Positions

```
GET /futures/{settle}/positions
```

Returns all open positions for the account in that settlement currency.

### 7.2 Get Single Position

```
GET /futures/{settle}/positions/{contract}
```

**Position fields:**

| Field | Type | Description |
|-------|------|-------------|
| `user` | int64 | User ID |
| `contract` | string | Contract name (e.g. `"BTC_USDT"`) |
| `size` | int64 | Position size (positive = long, negative = short) |
| `leverage` | string | Current leverage |
| `risk_limit` | string | Risk limit |
| `leverage_max` | string | Max allowed leverage |
| `maintenance_rate` | string | Maintenance margin rate |
| `value` | string | Position value |
| `margin` | string | Current margin |
| `entry_price` | string | Average entry price |
| `liq_price` | string | Liquidation price |
| `mark_price` | string | Current mark price |
| `unrealised_pnl` | string | Unrealized P&L |
| `realised_pnl` | string | Realized P&L |
| `history_pnl` | string | Historical P&L |
| `last_close_pnl` | string | P&L from last close |
| `realised_point` | string | Realized POINT income |
| `history_point` | string | Historical POINT income |
| `adl_ranking` | int32 | Auto-deleveraging ranking |
| `pending_orders` | int32 | Number of pending orders |
| `close_order` | object | Active close order info |
| `mode` | string | Position mode: `"single"`, `"dual_long"`, `"dual_short"` |
| `cross_leverage_limit` | string | Max leverage in cross-margin mode |
| `update_time` | int64 | Last update timestamp |
| `open_time` | int64 | Position open timestamp |

---

## 8. Leverage

### 8.1 Update Position Leverage

```
POST /futures/{settle}/positions/{contract}/leverage
```

**Query Params:**
- `leverage` (required): New leverage value as string (e.g. `"10"`)
- `cross_leverage_limit` (optional): Max leverage for cross-margin mode

**Response:** Updated Position object.

**Example:**
```
POST /futures/usdt/positions/BTC_USDT/leverage?leverage=10
```

---

## 9. Position Margin

### 9.1 Update Position Margin (Isolated)

```
POST /futures/{settle}/positions/{contract}/margin
```

**Query Params:**
- `change` (required): Margin change amount (positive = add, negative = remove)

---

## 10. Risk Limit

### 10.1 Update Position Risk Limit

```
POST /futures/{settle}/positions/{contract}/risk_limit
```

**Query Params:**
- `risk_limit` (required): New risk limit value

---

## 11. Wallet Transfers

### 11.1 Transfer Between Accounts

```
POST /wallet/transfers
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `currency` | string | Yes | Currency to transfer (e.g. `"USDT"`, `"BTC"`) or `"POINT"` |
| `from` | string | Yes | Source account type |
| `to` | string | Yes | Destination account type |
| `amount` | string | Yes | Transfer amount |
| `currency_pair` | string | Cond. | Required when transferring to/from `margin` |
| `settle` | string | Cond. | Required when transferring to/from `futures` or `delivery` |

**Account type values for `from`/`to`:**

| Value | Description |
|-------|-------------|
| `spot` | Spot trading account |
| `margin` | Isolated margin account (requires `currency_pair`) |
| `futures` | Perpetual futures (requires `settle`) |
| `delivery` | Delivery futures (requires `settle`) |
| `options` | Options account |
| `cross_margin` | Cross-margin account |

**Example (Spot to Futures USDT):**
```json
{
  "currency": "USDT",
  "from": "spot",
  "to": "futures",
  "amount": "1000",
  "settle": "usdt"
}
```

**Example (Spot to Isolated Margin):**
```json
{
  "currency": "USDT",
  "from": "spot",
  "to": "margin",
  "amount": "500",
  "currency_pair": "BTC_USDT"
}
```

---

### 11.2 Get Transfer History

```
GET /wallet/transfers
```

**Query Params:**
- `currency` (optional)
- `from` / `to` (optional): Unix timestamp
- `limit` (optional, default 100)
- `offset` (optional)

---

## 12. Wallet — Deposits and Withdrawals

### 12.1 Get Deposit Records

```
GET /wallet/deposits
```

### 12.2 Get Withdrawal Records

```
GET /wallet/withdrawals
```

### 12.3 Create Withdrawal

```
POST /withdrawals
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `currency` | string | Yes | Currency |
| `amount` | string | Yes | Amount |
| `address` | string | Yes | Withdrawal address |
| `memo` | string | No | Memo/tag for supported chains |
| `chain` | string | No | Blockchain network |

---

## 13. Account Detail

### 13.1 Get Account Info

```
GET /account/detail
```

Returns user account overview including VIP level, GT balance, user ID.

**Key response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | int64 | User numeric ID |
| `ip_whitelist` | []string | Whitelisted IP addresses |
| `currency_pairs` | []string | Available trading pairs |
| `tier` | int32 | VIP tier level |

---

## 14. Income / P&L History

### 14.1 Futures Income History

```
GET /futures/{settle}/account_book
```

**Query Params:**
- `contract` (optional): filter by contract
- `limit` (optional, default 100, max 1000)
- `from` / `to` (optional): Unix timestamp
- `type` (optional): income type filter — `"dnw"` (deposit/withdrawal), `"pnl"`, `"fee"`, `"refr"` (referral rebate), `"fund"` (funding fee), `"point_dnw"`, `"point_fee"`, `"point_refr"`

**Response fields per entry:**

| Field | Type | Description |
|-------|------|-------------|
| `time` | float64 | Timestamp |
| `change` | string | Balance change amount |
| `balance` | string | Balance after change |
| `type` | string | Income type |
| `text` | string | Description |
| `contract` | string | Related contract |
| `trade_id` | string | Related trade ID |

---

## 15. Sub-Account Management

### 15.1 List Sub-Accounts

```
GET /sub_accounts
```

### 15.2 Transfer Between Main and Sub-Account

```
POST /wallet/sub_account_transfers
```

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency |
| `sub_account` | string | Sub-account UID |
| `direction` | string | `"to"` (main to sub) or `"from"` (sub to main) |
| `amount` | string | Transfer amount |
| `client_order_id` | string | Idempotency key |

### 15.3 Sub-Account Balance Query

```
GET /wallet/sub_account_balances
```

---

## Sources

- [Gate API v4 Official Docs](https://www.gate.com/docs/developers/apiv4/en/)
- [gateapi-go model_futures_account.go](https://github.com/gateio/gateapi-go/blob/master/model_futures_account.go)
- [gateapi-go model_spot_account.go](https://github.com/gateio/gateapi-go/blob/master/model_spot_account.go)
- [gateapi-go model_position.go](https://github.com/gateio/gateapi-go/blob/master/model_position.go)
- [gateapi-go model_transfer.go](https://github.com/gateio/gateapi-go/blob/master/model_transfer.go)
- [Gate.io WalletApi.md](https://github.com/gateio/gateapi-python/blob/master/docs/WalletApi.md)
