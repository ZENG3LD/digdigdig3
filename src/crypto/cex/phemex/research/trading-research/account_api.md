# Phemex Account API — Balances, Wallet, Margin, Transfers, Sub-accounts

Source: https://phemex-docs.github.io/

---

## Account Information

### Query Trading Account and Positions

```
GET /accounts/accountPositions      (COIN-M Perpetual)
GET /g-accounts/accountPositions    (USDM Perpetual)
```

Returns: account balance, available balance, all open positions, leverage per position, margin mode.

| Field      | Type   | Required | Description                                   |
|------------|--------|----------|-----------------------------------------------|
| `currency` | string | Cond.    | Settlement currency filter (COIN-M only; e.g. `BTC`) |

### Query Positions with Unrealized PnL

```
GET /accounts/positions     (COIN-M)
GET /g-accounts/positions   (USDM)
```

Returns positions augmented with real-time unrealized PnL values.

### Query Fee Rate

```
GET /api-data/futures/fee-rate?settleCurrency=<settleCurrency>
```

Public endpoint. Returns `takerFeeRateEr` and `makerFeeRateEr` per trading symbol.

No dedicated "trading volume tier" or fee schedule lookup endpoint is documented.

---

## Balance & Wallet

### Query Spot / Margin Wallet Balances

```
GET /wallets
```

Optional parameter: `currency` (string) — filter by asset.

Returns balances for spot and margin accounts. Does not require a specific currency parameter; omitting it returns all assets.

### Query Unified Account Asset

```
GET /unified/asset
```

Returns detailed asset balances and positions across the unified trading account (if enabled).

### Query Unified Collateral Info

```
GET /unified/collateral
```

Returns collateral information for unified account.

---

## Deposit

### Get Deposit Address

```
GET /deposit-address
```

| Field      | Type   | Required | Description                                 |
|------------|--------|----------|---------------------------------------------|
| `currency` | string | Yes      | Cryptocurrency (e.g. `BTC`, `USDT`)         |
| `chain`    | string | No       | Specific blockchain network (e.g. `ERC20`)  |

### Get Supported Deposit Chains

```
GET /deposit-chains
```

| Field      | Type   | Required | Description                                      |
|------------|--------|----------|--------------------------------------------------|
| `currency` | string | Yes      | Cryptocurrency                                   |

Returns: available networks, confirmation requirements.

### Get Deposit History

```
GET /deposit-history
```

| Field      | Type    | Required | Description                      |
|------------|---------|----------|----------------------------------|
| `currency` | string  | No       | Filter by asset                  |
| `start`    | integer | No       | Start time (milliseconds)        |
| `end`      | integer | No       | End time (milliseconds)          |
| `offset`   | integer | No       | Pagination offset                |
| `limit`    | integer | No       | Max records (up to 200)          |

---

## Withdrawal

### Create Withdrawal Request

```
POST /withdraw
```

| Field      | Type    | Required | Description                          |
|------------|---------|----------|--------------------------------------|
| `currency` | string  | Yes      | Asset to withdraw                    |
| `address`  | string  | Yes      | Destination wallet address           |
| `amount`   | decimal | Yes      | Withdrawal amount                    |
| `chain`    | string  | Yes      | Blockchain network (e.g. `TRC20`)    |

### Cancel Withdrawal Request

```
DELETE /withdraw/cancel
```

| Field        | Type   | Required | Description                      |
|--------------|--------|----------|----------------------------------|
| `withdrawID` | string | Yes      | Withdrawal request identifier    |

### Get Withdrawal History

```
GET /withdraw-history
```

| Field      | Type    | Required | Description               |
|------------|---------|----------|---------------------------|
| `currency` | string  | No       | Filter by asset           |
| `start`    | integer | No       | Start time (ms)           |
| `end`      | integer | No       | End time (ms)             |
| `offset`   | integer | No       | Pagination offset         |
| `limit`    | integer | No       | Max records (up to 200)   |

### Get Supported Withdrawal Chains

```
GET /withdraw-chains
```

| Field      | Type   | Required | Description                               |
|------------|--------|----------|-------------------------------------------|
| `currency` | string | Yes      | Asset                                     |

Returns: available networks, fee information.

---

## Internal Transfers (Spot ↔ Futures ↔ Sub-accounts)

### Transfer Between Spot and Futures

```
POST /assets/transfer
```

| Field      | Type    | Required | Description                                      |
|------------|---------|----------|--------------------------------------------------|
| `currency` | string  | Yes      | Asset to transfer                                |
| `amount`   | decimal | Yes      | Transfer quantity                                |
| `from`     | string  | Yes      | Source account type (e.g. `spot`, `futures`)     |
| `to`       | string  | Yes      | Destination account type                         |

### Transfer Spot Sub-account → Main

```
POST /assets/subUserTransfer
```

| Field      | Type    | Required | Description              |
|------------|---------|----------|--------------------------|
| `currency` | string  | Yes      | Asset                    |
| `amount`   | decimal | Yes      | Amount                   |

### Query Spot Sub/Main Transfer History

```
GET /assets/subUserTransfer/history
```

### Transfer Futures Sub-account → Main

```
POST /futures/subUserTransfer
```

| Field      | Type    | Required | Description              |
|------------|---------|----------|--------------------------|
| `currency` | string  | Yes      | Asset                    |
| `amount`   | decimal | Yes      | Amount                   |

### Query Futures Sub/Main Transfer History

```
GET /futures/subUserTransfer/history
```

### Universal Transfer (Main Account Only)

Transfer between sub-to-main, main-to-sub, or sub-to-sub.

```
POST /assets/universalTransfer
```

| Field      | Type    | Required | Description              |
|------------|---------|----------|--------------------------|
| `fromUID`  | integer | Yes      | Source user ID           |
| `toUID`    | integer | Yes      | Destination user ID      |
| `currency` | string  | Yes      | Asset                    |
| `amount`   | decimal | Yes      | Amount                   |

---

## Margin / Lending

### Borrow

```
POST /margin/borrow
```

Parameters not fully detailed in public docs. Submits a margin borrow request.

### Repay

```
POST /margin/payback
```

Submits a margin repayment.

### Get Borrow History

```
GET /margin/borrow/history
```

### Get Payback History

```
GET /margin/payback/history
```

### Get Interest History

```
GET /margin/interest/history
```

No explicit "max borrowable" or "current interest rate" endpoint documented. Margin account balances are retrieved via the standard `GET /wallets` endpoint.

---

## Unified Trading Account (UTA)

### Query Risk Mode

```
GET /unified/risk-mode
```

### Switch Risk Mode

```
PUT /unified/risk-mode
```

| Field | Type   | Required | Description                                            |
|-------|--------|----------|--------------------------------------------------------|
| mode  | string | Yes      | `CrossAsset`, `SingleAsset`, or `Isolated`             |

### Payback Debts

```
POST /unified/payback
```

### Query Risk Unit

```
GET /unified/risk-unit
```

Returns risk unit info for RiskWallets or single position.

### Query Borrow History (UTA)

```
GET /unified/borrow-history
```

### Query Payback History (UTA)

```
GET /unified/payback-history
```

### Query Interest History (UTA)

```
GET /unified/interest-history
```

### Query Convert History (UTA)

```
GET /unified/convert-history
```

---

## Convert / Swap

### Get Quote

```
GET /convert/quote
```

Parameters include source currency, destination currency, and amount.

### Execute Conversion

```
POST /convert
```

Requires quote ID returned from quote request.

### Get Conversion History

```
GET /convert/history
```

Supports date range and pagination filters.

---

## Sub-accounts

Sub-account management (create, list, per-sub balances) is not documented in the public REST API reference. Transfer operations between sub and main accounts are available via:

- `POST /assets/subUserTransfer`
- `POST /futures/subUserTransfer`
- `POST /assets/universalTransfer`

Detailed sub-account creation and listing endpoints are not documented in the public-facing reference.

---

## API Key Management

No endpoint for querying API key permissions or IP whitelist settings via REST is documented. API key management is performed through the Phemex web interface.

---

Sources:
- https://phemex-docs.github.io/
