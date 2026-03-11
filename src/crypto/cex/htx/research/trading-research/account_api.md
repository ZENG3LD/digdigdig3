# HTX (Huobi) Account API — Spot, Margin & Futures

## Account Architecture Overview

HTX uses **account-id** as the primary routing key. A single user has multiple accounts
of different types. You must query `/v1/account/accounts` first to discover account IDs.

---

## SPOT & MARGIN ACCOUNTS

### GET /v1/account/accounts — List All Accounts

**Auth required**: Yes (Read permission)

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100009,
      "type": "spot",
      "subtype": "",
      "state": "working"
    },
    {
      "id": 100010,
      "type": "margin",
      "subtype": "btcusdt",
      "state": "working"
    },
    {
      "id": 100011,
      "type": "super-margin",
      "subtype": "",
      "state": "working"
    }
  ]
}
```

**Account `type` values:**

| type            | Description |
|-----------------|-------------|
| `spot`          | Standard spot trading account |
| `margin`        | Isolated margin account (one per trading pair) |
| `super-margin`  | Cross-margin account (also called `cross-margin`) |
| `otc`           | Over-the-counter account |
| `investment`    | C2C margin lending account |
| `borrow`        | C2C margin borrowing account |
| `point`         | HTX point card account |
| `minepool`      | Mining pool account |
| `etf`           | ETF account |

**`subtype` field:**
- For `margin` accounts: contains the trading pair (e.g. `"btcusdt"`)
- For all others: empty string

**`state` values:**
- `working` — account is active and operational
- `lock` — account is locked

---

### GET /v1/account/accounts/{account-id}/balance — Account Balance

**Auth required**: Yes (Read permission)

**Path parameter:** `account-id` (long)

**Response JSON:**
```json
{
  "status": "ok",
  "data": {
    "id": 100009,
    "type": "spot",
    "state": "working",
    "list": [
      {
        "currency": "btc",
        "type": "trade",
        "balance": "1.500000000000000000"
      },
      {
        "currency": "btc",
        "type": "frozen",
        "balance": "0.250000000000000000"
      },
      {
        "currency": "usdt",
        "type": "trade",
        "balance": "10000.000000000000000000"
      },
      {
        "currency": "usdt",
        "type": "frozen",
        "balance": "500.000000000000000000"
      }
    ]
  }
}
```

**Balance `type` values:**

| type      | Description |
|-----------|-------------|
| `trade`   | Available balance — can be used for orders |
| `frozen`  | Locked in open orders or margin loans |
| `loan`    | Borrowed amount (margin accounts only) |
| `interest` | Accrued interest (margin accounts only) |

All `balance` values are decimal strings to preserve precision.

---

## ISOLATED MARGIN TRADING

Each isolated margin account corresponds to one trading pair (e.g. `btcusdt`).
Account type is `margin`, `subtype` is the pair.

### POST /v1/margin/orders — Borrow Funds (Isolated)

**Auth required**: Yes (Trade permission)

**Request JSON:**
```json
{
  "symbol": "btcusdt",
  "currency": "btc",
  "amount": "0.5"
}
```

| Field    | Type    | Required | Description |
|----------|---------|----------|-------------|
| symbol   | string  | YES      | Trading pair for isolated margin |
| currency | string  | YES      | Asset to borrow |
| amount   | decimal | YES      | Amount to borrow |

**Response JSON:**
```json
{
  "status": "ok",
  "data": 123456
}
```

`data` is the loan order ID.

---

### POST /v1/margin/orders/{order-id}/repay — Repay Loan (Isolated)

**Request JSON:**
```json
{
  "amount": "0.5"
}
```

---

### GET /v1/margin/loan-info — Isolated Margin Loan Info

Returns interest rates and borrowing limits per currency per symbol.

---

### GET /v1/margin/accounts/balance — Isolated Margin Balance

**Query Parameters:**

| Field  | Type   | Required | Description |
|--------|--------|----------|-------------|
| symbol | string | NO       | Filter by trading pair |

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 100010,
      "type": "margin",
      "symbol": "btcusdt",
      "state": "working",
      "risk-rate": "1000",
      "fl-price": "0",
      "list": [
        {
          "currency": "btc",
          "type": "trade",
          "balance": "1.5"
        },
        {
          "currency": "btc",
          "type": "frozen",
          "balance": "0.0"
        },
        {
          "currency": "btc",
          "type": "loan",
          "balance": "0.5"
        },
        {
          "currency": "btc",
          "type": "interest",
          "balance": "0.0001"
        }
      ]
    }
  ]
}
```

**Key fields:**
- `risk-rate`: Margin risk ratio (liquidation triggered below threshold)
- `fl-price`: Estimated liquidation price

---

### GET /v1/margin/loan-orders — Isolated Margin Loan History

---

## CROSS MARGIN TRADING

Single account covers all cross-margin positions. Account type is `super-margin`.

### POST /v1/cross-margin/orders — Borrow Funds (Cross)

**Request JSON:**
```json
{
  "currency": "usdt",
  "amount": "1000"
}
```

### POST /v1/cross-margin/orders/{order-id}/repay — Repay Loan (Cross)

### GET /v1/cross-margin/loan-info — Cross Margin Loan Info

### GET /v1/cross-margin/accounts/balance — Cross Margin Balance

### GET /v1/cross-margin/loan-orders — Cross Margin Loan History

---

## ASSET TRANSFERS

### POST /v1/dw/transfer-in/margin — Spot to Isolated Margin

```json
{
  "symbol": "btcusdt",
  "currency": "btc",
  "amount": "0.5"
}
```

### POST /v1/dw/transfer-out/margin — Isolated Margin to Spot

```json
{
  "symbol": "btcusdt",
  "currency": "btc",
  "amount": "0.5"
}
```

### POST /v1/account/transfer — Universal Transfer

Supports transfers between: spot ↔ isolated-margin ↔ cross-margin ↔ OTC

**Request JSON:**
```json
{
  "from-user": 12345,
  "from-account-type": "spot",
  "from-account": 100009,
  "to-user": 12345,
  "to-account-type": "margin",
  "to-account": 100010,
  "currency": "btc",
  "amount": "0.5"
}
```

### POST /v1/futures/transfer — Spot ↔ Futures

**Request JSON:**
```json
{
  "currency": "btc",
  "amount": "0.5",
  "type": "pro-to-futures"
}
```

| type value         | Direction |
|--------------------|-----------|
| `pro-to-futures`   | Spot → Coin-M Futures |
| `futures-to-pro`   | Coin-M Futures → Spot |

---

## FEE RATES

### GET /v2/reference/transact-fee-rate — Fee Rate Query

**Auth required**: Yes (Read permission)

**Query Parameters:**

| Field    | Type   | Required | Description |
|----------|--------|----------|-------------|
| symbols  | string | YES      | Comma-separated symbols, max 10, e.g. `btcusdt,ethusdt` |

**Response JSON:**
```json
{
  "code": 200,
  "data": [
    {
      "symbol": "btcusdt",
      "makerFeeRate": "-0.00020",
      "takerFeeRate": "0.00200",
      "actualMakerRate": "-0.00020",
      "actualTakerRate": "0.00200"
    }
  ]
}
```

Note: Negative maker fee = rebate. This endpoint uses v2 response format (`code` int, not `status` string).

---

## FUTURES ACCOUNTS — Coin-Margined (DM)

**Base URL**: `https://api.hbdm.com`
**Margin currency**: The underlying coin (e.g. BTC for BTC futures)

### POST /api/v1/contract_account_info — Account Balance

**Request JSON:**
```json
{
  "symbol": "BTC"
}
```

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "symbol": "BTC",
      "margin_balance": "1.500000000000000000",
      "margin_position": "0.100000000000000000",
      "margin_frozen": "0.050000000000000000",
      "margin_available": "1.350000000000000000",
      "margin_static": "1.490000000000000000",
      "profit_real": "0.010000000000000000",
      "profit_unreal": "0.001000000000000000",
      "risk_rate": "15.00",
      "liquidation_price": null,
      "withdraw_available": "1.300000000000000000",
      "lever_rate": 10,
      "adjust_factor": "0.10",
      "margin_asset": "BTC"
    }
  ],
  "ts": 1630000000000
}
```

**Key balance fields:**

| Field               | Description |
|---------------------|-------------|
| `margin_balance`    | Total account equity |
| `margin_position`   | Margin used by open positions |
| `margin_frozen`     | Margin frozen in open orders |
| `margin_available`  | Available to open new positions |
| `margin_static`     | Static margin (no unrealized PnL) |
| `profit_real`       | Realized profit |
| `profit_unreal`     | Unrealized profit |
| `risk_rate`         | Account risk ratio |
| `liquidation_price` | Estimated liquidation price (null if no position) |
| `withdraw_available`| Available for withdrawal |

---

### POST /api/v1/contract_position_info — Position Info

**Request JSON:**
```json
{
  "symbol": "BTC"
}
```

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "symbol": "BTC",
      "contract_code": "BTC-USD",
      "contract_type": "quarter",
      "volume": 10,
      "available": 10,
      "frozen": 0,
      "cost_open": "38000.000000000000",
      "cost_hold": "38000.000000000000",
      "profit_unreal": "10.000000000000000000",
      "profit_rate": "0.0263",
      "profit": "0.001000000000000000",
      "margin_asset": "BTC",
      "position_margin": "0.100000000000000000",
      "lever_rate": 10,
      "direction": "buy",
      "last_price": "38100.000000000000"
    }
  ]
}
```

---

### POST /api/v1/contract_switch_lever_rate — Change Leverage

**Request JSON:**
```json
{
  "symbol": "BTC",
  "lever_rate": 20,
  "contract_code": "BTC-USD"
}
```

| Field         | Type   | Required | Description |
|---------------|--------|----------|-------------|
| symbol        | string | YES      | Coin symbol |
| lever_rate    | int    | YES      | New leverage (1–125) |
| contract_code | string | NO       | Specific contract |

---

### POST /api/v1/contract_available_level_rate — Query Available Leverage

Returns which leverage multipliers are currently available for a contract.

---

### POST /api/v1/contract_account_position_info — Combined Account + Position

Returns account balance and all open positions in a single call. Preferred for efficiency.

---

## FUTURES ACCOUNTS — USDT-Margined Swaps (Linear)

**Base URL**: `https://api.hbdm.com`
**API path prefix**: `/linear-swap-api/v1/`
**Margin currency**: USDT

### Isolated Margin

| Action | Endpoint |
|--------|----------|
| Account info | `POST /linear-swap-api/v1/swap_account_info` |
| Position info | `POST /linear-swap-api/v1/swap_position_info` |
| Account + positions | `POST /linear-swap-api/v1/swap_account_position_info` |
| Switch leverage | `POST /linear-swap-api/v1/swap_switch_lever_rate` |
| Switch position mode | `POST /linear-swap-api/v1/swap_switch_position_mode` |

### Cross Margin

| Action | Endpoint |
|--------|----------|
| Account info | `POST /linear-swap-api/v1/swap_cross_account_info` |
| Position info | `POST /linear-swap-api/v1/swap_cross_position_info` |
| Account + positions | `POST /linear-swap-api/v1/swap_cross_account_position_info` |
| Switch leverage | `POST /linear-swap-api/v1/swap_cross_switch_lever_rate` |
| Switch position mode | `POST /linear-swap-api/v1/swap_cross_switch_position_mode` |

### Position Mode (One-Way vs Hedge)

**POST /linear-swap-api/v1/swap_switch_position_mode**

```json
{
  "margin_account": "USDT",
  "position_mode": "single_side"
}
```

| position_mode  | Description |
|----------------|-------------|
| `single_side`  | One-way mode — long or short only |
| `dual_side`    | Hedge mode — simultaneous long and short |

In one-way mode, use `reduce_only: 1` on orders to close positions.

---

## Sources

- [HTX Spot API Reference — Accounts](https://huobiapi.github.io/docs/spot/v1/en/)
- [HTX Coin-M Futures API Reference](https://huobiapi.github.io/docs/dm/v1/en/)
- [HTX USDT-M Swap API Reference](https://huobiapi.github.io/docs/usdt_swap/v1/en/)
