# Crypto.com Exchange API v1 — Account API Research

Source: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
Research Date: 2026-03-11

---

## REST Base URL

```
Production: https://api.crypto.com/exchange/v1/{method}
UAT Sandbox: https://uat-api.3ona.co/exchange/v1/{method}
```

All private endpoints require HMAC-SHA256 authentication. See `auth_levels.md` for signing details.

---

## Account Information

### Get Account Info: `private/get-accounts`

Returns master account and all sub-account details.

**Rate Limit:** 3 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page_size` | int | NO | Results per page |
| `page` | int | NO | 0-based page number |

Response fields per account:
- `uuid` — account UUID
- `master_account_uuid` — parent account UUID (null for master)
- `margin_access` — margin trading access level
- `derivatives_access` — derivatives trading access level
- `kyc` — KYC verification level
- `created_at` — account creation timestamp

### Get Account Settings: `private/get-account-settings`

Returns account-level configuration parameters.

### Change Account Settings: `private/change-account-settings`

Modify account-level configuration.

**Specific parameters for both:** NOT FULLY DOCUMENTED in retrieved content.

### Get Fee Rate: `private/get-fee-rate`

Returns the user's maker/taker fee rates.

**Rate Limit:** 3 requests per 100ms

No required parameters (returns for the authenticated account).

Response fields: maker rate, taker rate (exact field names NOT DOCUMENTED in retrieved content).

### Get Instrument Fee Rate: `private/get-instrument-fee-rate`

Returns fee rates for a specific instrument.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | YES | e.g. `BTC_USDT` |

### Get Trading Volume (for fee tier): NOT DOCUMENTED AS STANDALONE ENDPOINT

Fee tier is determined by 30-day trading volume. This volume information may be included in `private/get-fee-rate` response, but a standalone "get trading volume" endpoint is NOT DOCUMENTED.

---

## Balance and Wallet

### Get Balances: `private/user-balance`

**Account Structure:** Unified balance model — spot, margin, and derivatives are under a single account. There is no separate "futures wallet" vs "spot wallet" distinction — positions and margin requirements are reflected within the same balance response.

**Rate Limit:** 3 requests per 100ms

No required parameters.

Response fields:
- `total_available_balance` — balance available to open new orders (Margin Balance minus Initial Margin)
- `total_margin_balance` — total margin balance
- `total_initial_margin` — total initial margin held for open orders
- `total_maintenance_margin` — maintenance margin for open positions
- `total_position_cost` — cost basis of open positions
- Per-asset breakdown:
  - `instrument_name`
  - `quantity`
  - `market_value`
  - `collateral_eligible` — whether asset counts as collateral
  - `haircut` — haircut percentage applied to collateral value

Isolated position fields added 2026-01-08:
- `isolation_id`
- `isolation_type`
- Per-isolated-position margin/leverage details

### Get Sub-account Balances: `private/get-subaccount-balances`

Returns balances for all sub-accounts under the master account.

Response fields per sub-account:
- `account_uuid`
- `instrument_name`
- `total_available_balance`
- `total_margin_balance`
- `total_initial_margin`
- `total_maintenance_margin`
- Isolated position margin data

### Get Deposit Address: `private/get-deposit-address`

**Rate Limit:** 3 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | YES | e.g. `BTC`, `ETH` |
| `network` | string | NO | Blockchain network if multiple supported |

### Get Deposit History: `private/get-deposit-history`

**Rate Limit:** 3 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | NO | Filter by currency |
| `start_ts` | long | NO | Start timestamp (ms) |
| `end_ts` | long | NO | End timestamp (ms) |
| `page_size` | int | NO | Records per page |
| `page` | int | NO | 0-based page |
| `status` | string | NO | Filter by status |

### Create Withdrawal: `private/create-withdrawal`

**Rate Limit:** 3 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | YES | Currency to withdraw |
| `amount` | string | YES | Amount (validated against max withdrawal in user-balance) |
| `address` | string | YES | Withdrawal destination address |
| `address_tag` | string | NO | Memo/tag for currencies that require it |
| `network_id` | string | NO | Blockchain network |
| `client_wid` | string | NO | Client withdrawal ID for deduplication |

### Get Withdrawal History: `private/get-withdrawal-history`

**Rate Limit:** 3 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `currency` | string | NO | Filter by currency |
| `start_ts` | long | NO | Start timestamp (ms) |
| `end_ts` | long | NO | End timestamp (ms) |
| `page_size` | int | NO | Records per page |
| `page` | int | NO | 0-based page |
| `status` | string | NO | Filter by status |

### Get Currency Networks: `public/get-currency-networks`

Returns available blockchain networks for each currency. Useful for determining valid `network_id` values before depositing or withdrawing.

### Internal Transfer Between Accounts: `private/create-subaccount-transfer`

Transfers funds between master account and sub-accounts (or between sub-accounts).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `from` | string | YES | Source account UUID (master or sub) |
| `to` | string | YES | Destination account UUID (master or sub) |
| `currency` | string | YES | Currency to transfer |
| `amount` | string | YES | Amount to transfer |

**Note:** There is no separate "spot-to-futures" or "spot-to-margin" internal transfer endpoint in the traditional sense. The platform uses a unified balance model with sub-account transfers as the primary mechanism for moving funds between accounts.

---

## Margin and Lending

**Note:** The Exchange API v1 supports margin trading (borrowing to trade on leverage). The following margin operations are documented:

### Margin Orders via `private/create-order`

Margin orders are placed by setting `spot_margin: "MARGIN"` in the order parameters. This enables borrowing against existing collateral.

### Isolated Margin

Isolated margin positions are managed via:
- `isolation_id` parameter on order creation
- `private/create-isolated-margin-transfer` — add/remove margin from isolated positions
- `private/change-isolated-margin-leverage` — adjust isolated position leverage

### Borrow / Repay / Interest Rates

Explicit `private/borrow`, `private/repay`, and `private/get-interest-rate` endpoints: NOT DOCUMENTED as standalone endpoints in Exchange API v1.

**Note:** Margin borrowing appears to be handled automatically when margin orders are placed (auto-borrow model), rather than through explicit borrow/repay endpoints. Explicit margin lending endpoints may exist in older API versions but are NOT DOCUMENTED in Exchange API v1 retrieved documentation.

### Get Max Borrowable: NOT DOCUMENTED

A standalone endpoint for querying maximum borrowable amount is NOT DOCUMENTED in Exchange API v1. This may be inferred from `private/user-balance` response fields.

---

## Sub-accounts

### Create Sub-account

NOT AVAILABLE via API. Sub-account creation is done through the Exchange website UI, not the API.

### List Sub-accounts: `private/get-accounts`

Returns both master account and all sub-accounts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page_size` | int | NO | Results per page |
| `page` | int | NO | 0-based |

### Transfer Between Sub-accounts: `private/create-subaccount-transfer`

See "Internal Transfer" section above. Supports master-to-sub, sub-to-master, and sub-to-sub transfers.

### Sub-account Balances: `private/get-subaccount-balances`

See "Get Sub-account Balances" section above.

---

## API Key Management

### Get API Key Permissions

NOT AVAILABLE as an API endpoint. API key permissions are managed exclusively through the Exchange website UI (User Center → API).

### IP Whitelist Info

NOT AVAILABLE as an API endpoint. IP whitelisting is configured through the Exchange website UI at key creation time.

### Available Permissions (set via UI)

| Permission | Description |
|------------|-------------|
| Read (Can Read) | Default. View balances, orders, positions. |
| Trade | Place, amend, cancel orders. |
| Withdraw | Initiate withdrawals. |

**Note:** Specific permission scope names as returned in API responses are NOT DOCUMENTED. Permissions are managed via UI only; the API does not expose a "get-api-key-permissions" endpoint.

---

## Balance History

### `private/get-balance-history`

Tracks balance changes over time.

**Specific parameters:** NOT FULLY DOCUMENTED in retrieved content.

---

## Sources

- [Crypto.com Exchange API v1 Official Docs](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Crypto.com Exchange Institutional API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index-insto-8556ea5c-4dbb-44d4-beb0-20a4d31f63a7.html)
- [Crypto.com Margin Trading User Guide](https://help.crypto.com/en/articles/6510664-margin-trading-user-guide)
- [Crypto.com API Help Center](https://help.crypto.com/en/articles/3511424-api)
