# Gemini Exchange — Account API Specification

Source: https://docs.gemini.com/rest/account-administration, https://docs.gemini.com/rest/fund-management
Retrieved: 2026-03-11

---

## Account Information

### Get Account Detail

**Method:** POST
**Path:** `/v1/account`
**Auth Required:** Yes — any role (Master or Account-level key)

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/account"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `account.accountName` | string | Display name of account |
| `account.shortName` | string | Short identifier for account |
| `account.type` | string | `"exchange"` or `"custody"` |
| `account.created` | string | Account creation timestamp |
| `users[].name` | string | User name |
| `users[].lastSignIn` | string | Last login timestamp |
| `users[].status` | string | User status |
| `users[].countryCode` | string | Country code |
| `users[].isVerified` | boolean | Whether user is KYC verified |
| `memo_reference_code` | string | Wire transfer memo |
| `virtual_account_number` | string | Virtual account number (if applicable) |

---

### Get Fee Schedule (Maker/Taker Rates)

**Method:** POST
**Path:** `/v1/notionalvolume`
**Auth Required:** Yes — Trader or Auditor role

Returns the current fee tier based on 30-day notional volume.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `date` | string | Date of calculation |
| `last_updated_ms` | integer | Last update timestamp |
| `web_maker_fee_bps` | integer | Web UI maker fee in basis points |
| `web_taker_fee_bps` | integer | Web UI taker fee in basis points |
| `web_auction_fee_bps` | integer | Web UI auction fee in basis points |
| `api_maker_fee_bps` | integer | REST/FIX API maker fee in basis points |
| `api_taker_fee_bps` | integer | REST/FIX API taker fee in basis points |
| `api_auction_fee_bps` | integer | REST/FIX API auction fee in basis points |
| `fix_maker_fee_bps` | integer | FIX protocol maker fee in basis points |
| `fix_taker_fee_bps` | integer | FIX protocol taker fee in basis points |
| `fix_auction_fee_bps` | integer | FIX protocol auction fee in basis points |
| `block_maker_fee_bps` | integer | Block trade maker fee in basis points |
| `block_taker_fee_bps` | integer | Block trade taker fee in basis points |
| `notional_30d_volume` | float | 30-day USD notional volume |
| `notional_1d_volume[].date` | string | Day date |
| `notional_1d_volume[].notional_volume` | float | That day's notional volume |

---

### List Fee Promos (Public)

**Method:** GET
**Path:** `/v1/feepromos`
**Auth Required:** No

Returns symbols with active fee promotions.

---

### Get Trading Volume

**Method:** POST
**Path:** `/v1/tradevolume`
**Auth Required:** Yes — Trader or Auditor role

Returns per-symbol trading volume stats: `total_volume_base`, maker/taker ratios, buy/sell counts, notional amounts per symbol per date.

---

## Balance and Wallet

### Get Available Balances

**Method:** POST
**Path:** `/v1/balances`
**Auth Required:** Yes — Trader, Auditor, or FundManager role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/balances"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `account` | string | Required for Master keys | Sub-account name |
| `showPendingBalances` | boolean | No | Include pending amounts |

#### Response Fields (array of currency objects)

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `"exchange"` |
| `currency` | string | Currency ticker (e.g. `"BTC"`, `"USD"`) |
| `amount` | string | Total balance |
| `available` | string | Available to trade |
| `availableForWithdrawal` | string | Available to withdraw |
| `pendingWithdrawal` | string | Amount pending withdrawal |
| `pendingDeposit` | string | Amount pending deposit |

---

### Get Notional Balances

**Method:** POST
**Path:** `/v1/notionalbalances/{currency}`
**Auth Required:** Yes — Trader, Auditor, or FundManager role

Returns balances with notional USD (or other quote currency) values.

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/notionalbalances/usd"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields (array)

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Asset ticker |
| `amount` | string | Total balance |
| `amountNotional` | string | Notional value in quote currency |
| `available` | string | Available to trade |
| `availableNotional` | string | Available notional value |
| `availableForWithdrawal` | string | Available to withdraw |
| `availableForWithdrawalNotional` | string | Withdrawal amount as notional |

---

### Spot / Futures / Margin Balance Separation

**NOT SEPARATELY DOCUMENTED.** Gemini is primarily a spot exchange. Perpetual derivatives exist but unified balance architecture details are NOT documented in available reference. The `/v1/balances` endpoint covers the exchange account balance; derivative margin is accessed via `/v1/margin/account/summary`.

---

### Get Deposit Address

**Method:** POST
**Path:** `/v1/addresses/{network}`
**Auth Required:** Yes — Trader, Auditor, or FundManager role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/addresses/bitcoin"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `timestamp` | integer | No | Filter addresses created after this time |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields (array)

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Deposit address |
| `timestamp` | string | Creation timestamp |
| `label` | string | Address label |
| `memo` | string | Memo/tag (for applicable networks) |
| `network` | string | Blockchain network name |

---

### Create New Deposit Address

**Method:** POST
**Path:** `/v1/deposit/{network}/newAddress`
**Auth Required:** Yes — FundManager role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/deposit/bitcoin/newAddress"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `label` | string | No | Address label |
| `legacy` | boolean | No | Request legacy address format |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `network` | string | Blockchain network |
| `address` | string | New deposit address |
| `label` | string | Address label |
| `memo` | string | Memo/tag |
| `timestamp` | string | Creation timestamp |

---

### Get Deposit/Transfer History

**Method:** POST
**Path:** `/v1/transfers`
**Auth Required:** Yes — Trader, Auditor, or FundManager role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/transfers"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `currency` | string | No | Filter by currency |
| `timestamp` | integer | No | Only show transfers after this timestamp |
| `limit_transfers` | integer | No | Maximum: 50 |
| `account` | string | No | Sub-account name (Master key only) |
| `show_completed_deposit_advances` | boolean | No | Include completed advance deposits |

#### Response Fields (array)

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `"Deposit"` or `"Withdrawal"` |
| `status` | string | Transfer status |
| `timestampms` | integer | Unix timestamp (milliseconds) |
| `eid` | integer | Transfer event ID |
| `currency` | string | Currency ticker |
| `amount` | string | Transfer amount |
| `txHash` | string | On-chain transaction hash (if available) |

---

### Withdraw Crypto Funds

**Method:** POST
**Path:** `/v1/withdraw/{currency}`
**Auth Required:** Yes — FundManager role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/withdraw/btc"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `address` | string | Yes | Destination blockchain address |
| `amount` | string | Yes | Amount to withdraw |
| `client_transfer_id` | string | No | Idempotency key for withdrawal |
| `memo` | string | No | Memo/tag for applicable networks |
| `account` | string | No | Sub-account name (Master key only) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `address` | string | Destination address |
| `amount` | string | Amount withdrawn |
| `fee` | string | Network fee |
| `withdrawalId` | string | Gemini withdrawal ID |
| `message` | string | Confirmation message |

---

### Get Gas Fee Estimation

**Method:** POST
**Path:** `/v1/withdraw/{currencyCodeLowerCase}/feeEstimate`
**Auth Required:** Yes

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/withdraw/eth/feeEstimate"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `address` | string | Yes | Destination address |
| `amount` | string | Yes | Amount to withdraw |
| `account` | string | Yes | Account identifier |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `currency` | string | Currency |
| `fee` | string | Estimated gas fee |
| `isOverride` | boolean | Whether fee is an override |
| `monthlyLimit` | string | Monthly withdrawal limit |
| `monthlyRemaining` | string | Remaining monthly allowance |

---

### Withdrawal History

Withdrawal history is available via `/v1/transfers` — filter by `type = "Withdrawal"`. No separate withdrawal history endpoint.

---

### Internal Transfer Between Accounts

**Method:** POST
**Path:** `/v1/account/transfer/{currency}`
**Auth Required:** Yes — FundManager role (Master key required)

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | e.g. `"/v1/account/transfer/btc"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `sourceAccount` | string | Yes | Source sub-account short name |
| `targetAccount` | string | Yes | Destination sub-account short name |
| `amount` | string | Yes | Amount to transfer |
| `clientTransferId` | string | No | Idempotency key |
| `withdrawalId` | string | No | Existing withdrawal ID to move |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `fromAccount` | string | Source account |
| `toAccount` | string | Destination account |
| `amount` | string | Transferred amount |
| `fee` | string | Transfer fee (usually 0) |
| `currency` | string | Asset transferred |
| `withdrawalId` | string | Withdrawal ID |
| `uuid` | string | Transfer UUID |
| `message` | string | Confirmation message |
| `txHash` | string | On-chain hash (if applicable) |

**Note:** Spot ↔ Futures ↔ Margin transfers as separate sub-types are NOT explicitly documented. Transfer is between named sub-accounts.

---

### Address Whitelisting (Approved Addresses)

#### List Approved Addresses

**Method:** POST
**Path:** `/v1/approvedAddresses/account/{network}`
**Auth Required:** Yes

**Response:** Array of approved addresses, each with `network`, `scope`, `label`, `status`, `createdAt`, `address`.

#### Create New Approved Address

**Method:** POST
**Path:** `/v1/approvedAddresses/{network}/request`
**Auth Required:** Yes

**Parameters:** `address` (required), `label` (required), `memo`, `account`
**Note:** Subject to 7-day approval hold period.

#### Remove Approved Address

**Method:** POST
**Path:** NOT fully documented (listed in API navigation)
**Auth Required:** Yes

---

### Staking

#### List Staking Balances

**Method:** POST
**Path:** NOT fully documented (Staking section exists)
**Auth Required:** Yes

#### Stake Crypto Funds

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes

#### Unstake Crypto Funds

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes

#### List Staking Rewards

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes

---

## Margin / Lending

### Get Margin Interest Rates

**Method:** POST
**Path:** `/v1/margin/interest/rates` (inferred)
**Auth Required:** Yes

**Response fields:** NOT FULLY DOCUMENTED in available reference.

### Borrow

NOT AVAILABLE as a standalone endpoint. Margin borrowing occurs implicitly when placing an order with `margin_order: true`.

### Repay

NOT AVAILABLE as a standalone endpoint.

### Get Borrow History

NOT AVAILABLE — not documented.

### Get Max Borrowable

NOT AVAILABLE — not documented.

---

## Sub-accounts

### Overview

Gemini supports a Master Account + Sub-accounts model. Master API keys with Administrator role can manage sub-accounts. Sub-accounts can have their own API keys with independent role assignments.

### Create New Account (Sub-account)

**Method:** POST
**Path:** `/v1/account/create`
**Auth Required:** Yes — Master key with Administrator role

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/account/create"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `name` | string | Yes | Unique display name for new account |
| `type` | string | No | `"exchange"` or `"custody"` (default: `"exchange"`) |

**Response:** Returns new account's `account` (short name) and `type`.

---

### List Sub-accounts

**Method:** POST
**Path:** `/v1/account/list`
**Auth Required:** Yes — Master key

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/account/list"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `limit_accounts` | integer | No | Maximum: 500 (default: 500) |
| `timestamp` | integer | No | Filter by creation date |

#### Response Fields (array)

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Account display name |
| `account` | string | Account short name |
| `type` | string | `"exchange"` or `"custody"` |
| `counterparty_id` | string | Counterparty identifier |
| `created` | string | Creation timestamp |
| `status` | string | Account status |

---

### Transfer Between Sub-accounts

See "Internal Transfer Between Accounts" section above (`/v1/account/transfer/{currency}`).

### Sub-account Balances

Use `/v1/balances` with Master key + `account` parameter specifying the sub-account.

### Rename Account

**Method:** POST
**Path:** `/v1/account/rename`
**Auth Required:** Yes — Master or Account-level key with Administrator role

**Parameters:** `newName` (display name), `newAccount` (short name)

---

## API Key Management

### Get API Key Permissions/Roles

**Method:** POST
**Path:** `/v1/roles`
**Auth Required:** Yes — any role

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `isAuditor` | boolean | Key has Auditor role (read-only, mutually exclusive with other roles) |
| `isFundManager` | boolean | Key has FundManager role |
| `isTrader` | boolean | Key has Trader role |
| `isAccountAdmin` | boolean | Key has Administrator role (Master keys only) |
| `counterparty_id` | string | Counterparty ID (Master keys only) |

**Note:** `isAuditor` cannot be `true` at the same time as any other role. `isFundManager` and `isTrader` can both be `true`.

---

### IP Whitelist

- API keys can be restricted to a list of **Trusted IPs** at creation time.
- If Trusted IPs are configured, requests from non-whitelisted IPs are rejected.
- Keys can alternatively be set as **Unrestricted** (no IP filtering).
- Gemini announced enforcement of IP affirmation requirement for Trading API keys effective **June 30, 2025** — keys must either have IPs allowlisted or be explicitly set as Unrestricted.
- IP configuration is done in the Gemini web UI at key creation time; NOT manageable via API.

---

## Transaction History

### Get Transaction History

**Method:** POST
**Path:** `/v1/transactions`
**Auth Required:** Yes

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/transactions"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `timestamp_nanos` | integer | No | Start time in nanoseconds |
| `limit` | integer | No | Default: 100, Maximum: 300 |
| `continuation_token` | string | No | Pagination token from previous response |

**Response:** `results[]` array containing trade and transfer objects with account, amount, price, timestampms, symbol, fee details.

---

## Payment Methods / Fiat Banking

### List Payment Methods

**Method:** POST
**Path:** `/v1/payments/methods`
**Auth Required:** Yes

**Response:** `balances[]` array and `banks[]` array (linked bank accounts).

### Add Bank (USD)

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes

### Add Bank (CAD)

**Method:** POST
**Path:** NOT fully documented
**Auth Required:** Yes

---

## Custody Fee Transfers

### List Custody Fee Transfers

**Method:** POST
**Path:** `/v1/custodyaccountfees`
**Auth Required:** Yes

#### Request Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request` | string | Yes | `"/v1/custodyaccountfees"` |
| `nonce` | integer | Yes | Monotonically increasing |
| `timestamp` | integer | No | Filter start time |
| `limit_transfers` | integer | No | Limit count |
| `account` | string | No | Sub-account name |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `txTime` | string | Transaction time |
| `feeAmount` | string | Fee amount |
| `feeCurrency` | string | Fee currency |
| `eid` | integer | Event ID |
| `eventType` | string | Type of custody fee event |

---

## Sources

- [Fund Management — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/fund-management)
- [Account Administration — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/account-administration)
- [Roles — Gemini Crypto Exchange](https://docs.gemini.com/roles)
- [Orders — REST API — Gemini Crypto Exchange](https://docs.gemini.com/rest/orders)
- [How to secure your API Keys with Trusted IPs — Gemini Support](https://support.gemini.com/hc/en-us/articles/37826759865115-How-to-secure-your-API-Keys-with-Trusted-IPs)
