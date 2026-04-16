# Upbit Account API

Source: https://global-docs.upbit.com/reference
Exchange type: **Spot only** (no sub-accounts, no margin accounts, no futures wallets)
Base URL pattern: `https://{region}-api.upbit.com/v1`
Regions: `sg` (Singapore), `id` (Indonesia), `th` (Thailand)

---

## Account Information

### Get Account Balances

**Method:** `GET`
**Path:** `/accounts`
**Full URL:** `https://{region}-api.upbit.com/v1/accounts`
**Auth:** Bearer JWT (requires `View Account` permission)
**Rate limit group:** `default` — 30 req/sec

No query parameters required. Returns all asset balances held in the account.

#### Response (HTTP 200)

Array of account balance objects:

```json
[
  {
    "currency": "BTC",
    "balance": "0.0051",
    "locked": "0.0",
    "avg_buy_price": "29500.0",
    "avg_buy_price_modified": false,
    "unit_currency": "SGD"
  },
  {
    "currency": "SGD",
    "balance": "150.00",
    "locked": "30.03",
    "avg_buy_price": "0",
    "avg_buy_price_modified": false,
    "unit_currency": "SGD"
  }
]
```

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Asset code (e.g., `BTC`, `SGD`, `ETH`) |
| `balance` | string | Available (free) balance |
| `locked` | string | Balance locked in pending orders or withdrawals |
| `avg_buy_price` | string | Average purchase price of this asset |
| `avg_buy_price_modified` | boolean | Whether avg buy price was manually adjusted |
| `unit_currency` | string | Quote currency for the avg_buy_price field |

Note: `balance` (free) + `locked` = total holding for that asset.

### Get Fee Schedule

NOT DIRECTLY AVAILABLE as a standalone endpoint. Fee information is returned per-market via:
- `GET /orders/info` — returns `bid_fee` and `ask_fee` (maker/taker fee rates for the queried market)
- Individual order responses include `reserved_fee`, `remaining_fee`, `paid_fee` fields

Upbit uses a tiered fee system based on 30-day trading volume, but the fee tier and schedule are not directly queryable via the API. Fee rates are embedded in order responses.

### Get Trading Volume (for Fee Tier)

NOT AVAILABLE as a standalone endpoint. NOT DOCUMENTED in official API reference.

---

## Balance and Wallet

### Accounts Endpoint Coverage

Upbit is **spot-only** with a single unified spot wallet. There are no separate futures wallet, margin wallet, or options wallet endpoints. All balances (fiat and crypto) are in one account, returned by `GET /accounts`.

**Spot:** Covered by `GET /accounts`
**Futures:** NOT AVAILABLE
**Margin:** NOT AVAILABLE
**Unified wallet:** Single account, no sub-account system

---

## Deposit Addresses

### Get Deposit Address

**Method:** `GET`
**Path:** `/deposits/coin_address`
**Full URL:** `https://{region}-api.upbit.com/v1/deposits/coin_address`
**Auth:** Bearer JWT (requires `View Deposits` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Description |
|-----------|------|-------------|
| `currency` | string | Asset code to get deposit address for |
| `net_type` | string | Network/protocol type (e.g., `BTC`, `ETH`, `TRC20`) |

#### Response

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Asset code |
| `net_type` | string | Network type |
| `deposit_address` | string | The deposit wallet address |
| `secondary_address` | string | Optional memo/tag (for MEMO-based chains like XRP, MEMO-based assets) |

### Generate Deposit Address

**Method:** `POST`
**Path:** `/deposits/generate_coin_address`
**Full URL:** `https://{region}-api.upbit.com/v1/deposits/generate_coin_address`
**Auth:** Bearer JWT (requires `View Deposits` permission)

Requests generation of a new deposit address for a currency. Some currencies require prior address generation before `GET /deposits/coin_address` returns a result.

### List All Deposit Addresses

**Method:** `GET`
**Path:** `/deposits/coin_addresses`
**Full URL:** `https://{region}-api.upbit.com/v1/deposits/coin_addresses`
**Auth:** Bearer JWT (requires `View Deposits` permission)

Returns all deposit addresses for all currencies in the account.

---

## Deposit History

### Get Single Deposit

**Method:** `GET`
**Path:** `/deposit`
**Full URL:** `https://{region}-api.upbit.com/v1/deposit`
**Auth:** Bearer JWT (requires `View Deposits` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid` | string | Deposit UUID |
| `txid` | string | Blockchain transaction ID |
| `currency` | string | Asset code |

### List All Deposits

**Method:** `GET`
**Path:** `/deposits`
**Full URL:** `https://{region}-api.upbit.com/v1/deposits`
**Auth:** Bearer JWT (requires `View Deposits` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `currency` | string | Filter by asset code |
| `state` | string | Filter by deposit state |
| `uuids[]` | array | Filter by deposit UUIDs |
| `txids[]` | array | Filter by transaction IDs |
| `limit` | integer | Results per page (max 100) |
| `page` | integer | Page number |
| `order_by` | string | Sort direction: `asc` or `desc` |

---

## Withdrawals

### Get Available Withdrawal Information

**Method:** `GET`
**Path:** `/withdraws/chance`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraws/chance`
**Auth:** Bearer JWT (requires `View Withdrawals` permission)

Returns withdrawal limits, fees, and current account state for a given currency.

| Parameter | Type | Description |
|-----------|------|-------------|
| `currency` | string | Asset code |
| `net_type` | string | Network type |

Response includes:
- `member_level` — account verification tier
- `currency` — asset details including minimum/maximum withdrawal amounts
- `account` — current available balance for this currency
- `withdraw_limit` — daily and total withdrawal limits

### List Approved Withdrawal Addresses

**Method:** `GET`
**Path:** `/withdraws/coin_addresses`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraws/coin_addresses`
**Auth:** Bearer JWT (requires `View Withdrawals` permission)

Returns all whitelisted withdrawal addresses. Upbit requires pre-registration of withdrawal addresses before funds can be sent.

Response fields per address:

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Asset code |
| `net_type` | string | Network type |
| `withdrawal_address` | string | Registered wallet address |
| `secondary_address` | string | Memo/tag if applicable |

### Withdraw Digital Asset

**Method:** `POST`
**Path:** `/withdraws/coin`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraws/coin`
**Auth:** Bearer JWT (requires `Withdraw` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | Yes | Asset code |
| `net_type` | string | Yes | Network type |
| `amount` | string | Yes | Amount to withdraw |
| `address` | string | Yes | Target wallet address (must be whitelisted) |
| `secondary_address` | string | Conditional | Memo/tag for applicable chains |
| `transaction_type` | string | No | `default` or `internal` (exchange-internal transfer) |

### Get Single Withdrawal

**Method:** `GET`
**Path:** `/withdraw`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraw`
**Auth:** Bearer JWT (requires `View Withdrawals` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid` | string | Withdrawal UUID |
| `txid` | string | Blockchain transaction ID |
| `currency` | string | Asset code |

### List Withdrawals

**Method:** `GET`
**Path:** `/withdraws`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraws`
**Auth:** Bearer JWT (requires `View Withdrawals` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `currency` | string | Filter by asset code |
| `state` | string | Filter by withdrawal state |
| `uuids[]` | array | Filter by withdrawal UUIDs |
| `txids[]` | array | Filter by transaction IDs |
| `limit` | integer | Results per page (max 100) |
| `page` | integer | Page number |
| `order_by` | string | Sort: `asc` or `desc` |

Withdrawal states: `submitting`, `submitted`, `almost_accepted`, `rejected`, `accepted`, `processing`, `done`, `cancelled`

### Cancel Withdrawal

**Method:** `DELETE`
**Path:** `/withdraw`
**Full URL:** `https://{region}-api.upbit.com/v1/withdraw`
**Auth:** Bearer JWT (requires `Withdraw` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid` | string | Withdrawal UUID to cancel |

Only `submitting` or `submitted` state withdrawals can be cancelled.

---

## Internal Transfer Between Accounts

NOT AVAILABLE — Upbit does not offer multiple account types (no spot/futures/margin split), so internal transfers between account types do not exist.

---

## Margin and Lending

NOT APPLICABLE — Upbit does not support margin trading, borrowing, lending, or interest. The following are all NOT AVAILABLE:

- Borrow
- Repay
- Get borrow history
- Get interest rate
- Get max borrowable

---

## Sub-Accounts

NOT AVAILABLE — Upbit does not support sub-account creation, management, or inter-account transfers via the API.

---

## Travel Rule (VASP Verification)

Upbit implements the FATF Travel Rule for crypto withdrawals above certain thresholds. API endpoints:

### List VASPs

**Method:** `GET`
**Path:** `/travel-rule/vasps`
**Full URL:** `https://{region}-api.upbit.com/v1/travel-rule/vasps`
**Auth:** Bearer JWT

Returns list of supported Virtual Asset Service Providers for travel rule compliance.

### Verify by Deposit UUID

**Method:** `POST`
**Path:** `/travel-rule/verify-uuid`
**Full URL:** `https://{region}-api.upbit.com/v1/travel-rule/verify-uuid`

### Verify by Transaction ID

**Method:** `POST`
**Path:** `/travel-rule/verify-txid`
**Full URL:** `https://{region}-api.upbit.com/v1/travel-rule/verify-txid`

---

## Service Status

### Get Deposit/Withdrawal Service Status

**Method:** `GET`
**Path:** `/status/wallet`
**Full URL:** `https://{region}-api.upbit.com/v1/status/wallet`
**Auth:** Bearer JWT

Returns operational status for each currency's deposit and withdrawal services.

| Response Field | Description |
|----------------|-------------|
| `currency` | Asset code |
| `wallet_state` | `working`, `withdraw_only`, `deposit_only`, `paused`, `unsupported` |
| `block_state` | Blockchain synchronization status |
| `block_height` | Current synced block height |
| `block_updated_at` | Last blockchain sync time |

---

## API Key Management

### List API Keys

**Method:** `GET`
**Path:** `/api_keys`
**Full URL:** `https://{region}-api.upbit.com/v1/api_keys`
**Auth:** Bearer JWT

Returns list of API keys associated with the account.

Response fields:

| Field | Type | Description |
|-------|------|-------------|
| `access_key` | string | The access key identifier |
| `expire_at` | string | Key expiration timestamp |

Note: Permission details per key are NOT returned in this response per available documentation. Permissions are set at key creation time and are visible in the Upbit web dashboard.

IP whitelist configuration: Supported at key creation time via the Upbit website. NOT configurable via API. NOT DOCUMENTED whether it can be queried via `GET /api_keys`.

---

## Sources

- [Upbit Global Developer Center — Reference](https://global-docs.upbit.com/reference)
- [Overall Account Inquiry](https://global-docs.upbit.com/reference/overall-account-inquiry)
- [Authentication](https://global-docs.upbit.com/reference/auth)
- [Rate Limits](https://global-docs.upbit.com/reference/rate-limits)
