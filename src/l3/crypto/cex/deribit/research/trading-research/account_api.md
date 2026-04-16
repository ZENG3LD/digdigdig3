# Deribit Account API Specification

Source: https://docs.deribit.com/
API Version: v2
Protocol: JSON-RPC 2.0 over WebSocket or HTTP

---

## Account Information

### Get Account Summary (Single Currency)

**Method:** `private/get_account_summary`
**Scope required:** `account:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency symbol: BTC, ETH, STETH, ETHW, USDC, USDT, EURR, SOL, XRP, USYC, PAXG, BNB, USDE |
| `subaccount_id` | integer | No | Subaccount user ID (for querying subaccount data) |
| `extended` | boolean | No | Include extended fields: account ID, username, email, account type, fee_group |

#### Key Response Fields

**Balance & Equity:**

| Field | Type | Description |
|---|---|---|
| `balance` | number | Account balance in the currency |
| `equity` | number | Current equity (balance + unrealized P&L) |
| `available_funds` | number | Funds available for new trades (aggregated across cross-collateral when enabled) |
| `available_withdrawal_funds` | number | Funds available for withdrawal |
| `margin_balance` | number | Margin balance (aggregated with cross-collateral) |

**Margin Information:**

| Field | Type | Description |
|---|---|---|
| `initial_margin` | number | Required margin for all open positions |
| `maintenance_margin` | number | Minimum margin before liquidation |
| `projected_initial_margin` | number | Projected initial margin accounting for pending expirations |
| `projected_maintenance_margin` | number | Projected maintenance margin for upcoming settlements |
| `open_orders_margin` | number | Margin reserved for open orders |

**Fees:**

| Field | Type | Description |
|---|---|---|
| `fee_balance` | number | Balance reserved for fee payments |
| `fee_group` | string | Fee discount tier (requires `extended: true`) |
| `fees` | object | Full fee structure for currency pairs and instrument types (present when fee discounts exist) |
| `spot_reserve` | number | Balance reserved in active spot orders |

---

### Get Account Summaries (All Currencies)

**Method:** `private/get_account_summaries`
**Scope required:** `account:read`

Retrieves summaries for all currencies in a single call.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `subaccount_id` | integer | No | Subaccount ID |
| `extended` | boolean | No | Include additional fields (account ID, username, email, account type) |

Response: array of account summary objects, one per currency.

---

### Get Transaction Log

**Method:** `private/get_transaction_log`
**Scope required:** `account:read`
**Rate limit:** 1 request/second (special limit — more restrictive than general private limits)

Complete transaction history with no time limit.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency filter |
| `start_timestamp` | integer | No | Start of time range (ms) |
| `end_timestamp` | integer | No | End of time range (ms) |
| `count` | integer | No | Number of records |
| `continuation` | string | No | Continuation token for pagination |
| `query` | string | No | Filter by transaction type |

---

### Get Access Log

**Method:** `private/get_access_log`
**Scope required:** `account:read`

API access and authentication events.

---

## Balance and Wallet

### Get Deposits

**Method:** `private/get_deposits`
**Scope required:** `wallet:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency (e.g., BTC, ETH) |
| `count` | integer | No | Number of records |
| `offset` | integer | No | Pagination offset |

---

### Get Current Deposit Address

**Method:** `private/get_current_deposit_address`
**Scope required:** `wallet:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |

Returns existing deposit address for the currency.

---

### Create Deposit Address

**Method:** `private/create_deposit_address`
**Scope required:** `wallet:read_write`

Generates a new on-chain deposit address.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency (e.g., BTC) |

---

### Get Withdrawals

**Method:** `private/get_withdrawals`
**Scope required:** `wallet:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |
| `count` | integer | No | Number of records |
| `offset` | integer | No | Pagination offset |

---

### Submit Withdrawal

**Method:** `private/withdraw`
**Scope required:** `wallet:read_write`
**Note:** Requires Two-Factor Authentication (2FA).

---

### Get Transfers

**Method:** `private/get_transfers`
**Scope required:** `wallet:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |
| `count` | integer | No | Number of records |
| `offset` | integer | No | Pagination offset |

---

### Submit Internal Transfer (Main to Subaccount)

**Method:** `private/submit_transfer_to_subaccount`
**Scope required:** `wallet:read_write`

Transfer funds from main account to a subaccount.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |
| `amount` | number | Yes | Transfer amount |
| `destination` | integer | Yes | Destination subaccount ID |

---

### Submit Transfer Between Subaccounts

**Method:** `private/submit_transfer_between_subaccounts`
**Scope required:** `wallet:read_write`

Transfers between two subaccounts.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |
| `amount` | number | Yes | Amount |
| `source` | integer | Yes | Source subaccount ID |
| `destination` | integer | Yes | Destination subaccount ID |

---

### Submit Transfer to External User

**Method:** `private/submit_transfer_to_user`
**Scope required:** `wallet:read_write`

Transfer to another Deribit user by their username.

---

### Travel Rule Compliance

**Method:** `private/set_clearance_originator`
**Scope required:** `wallet:read_write`

Submit originator information required for travel rule compliance.

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency |
| `transaction_hash` | string | Yes | Transaction hash |
| `originator_name` | string | Yes | Sender's full name |
| `originator_address` | string | No | Sender's address |
| `originator_account_number` | string | No | Sender's account number |

---

## Subaccounts

### Create Subaccount

**Method:** `private/create_subaccount`
**Scope required:** `account:read_write`

No parameters required. Creates a new subaccount with a default name.

Response: `{ subaccount_id, username }` plus initial configuration.

---

### List All Subaccounts

**Method:** `private/get_subaccounts`
**Scope required:** `account:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `with_portfolio` | boolean | No | Include equity, available funds, maintenance margin in response |

Response fields per subaccount:
- `id` — subaccount user ID
- `username` — subaccount login
- `email` — registered email
- `is_password` — whether password-login enabled
- `login_enabled` — login status
- `notification_config` — notification settings
- (with `with_portfolio: true`) — `equity`, `available_funds`, `maintenance_margin`

---

### Get Detailed Subaccount Info

**Method:** `private/get_subaccounts_details`
**Scope required:** `account:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency (e.g., BTC, ETH) |
| `with_open_orders` | boolean | No | Include open orders in response |

Returns: positions, balances, and optionally open orders for all subaccounts.

---

### Subaccount Balance

Use `private/get_account_summary` with `subaccount_id` parameter to query a specific subaccount balance.

---

### Transfer Between Subaccounts

See `private/submit_transfer_between_subaccounts` in the Wallet section above.

---

### Subaccount Configuration

Additional management operations available:
- Rename subaccount
- Assign email
- Toggle login enabled/disabled
- Configure notification settings
- Remove subaccount (requires 2FA for sensitive operations)

---

## API Key Management

### List API Keys

**Method:** `private/list_api_keys`
**Scope required:** `account:read_write`
**Note:** May require additional Two-Factor Authentication (Security Key) depending on account settings.

Returns all API keys associated with the account with their permissions.

---

### Create API Key

**Method:** `private/create_api_key`
**Scope required:** `account:read_write`
**Note:** Requires TFA (Two-Factor Authentication).

Parameters include desired scopes, optional IP whitelist, optional key name.

---

### Remove API Key

**Method:** `private/remove_api_key`
**Scope required:** `account:read_write`
**Note:** Requires TFA.

---

### Reset API Key Secret

**Method:** `private/reset_api_key`
**Scope required:** `account:read_write`
**Note:** Requires TFA.

---

### Enable / Disable API Key

**Method:** `private/enable_api_key` / `private/disable_api_key`
**Scope required:** `account:read_write`

---

### IP Whitelist

- IP whitelisting can be configured per API key during creation or editing
- When IP whitelist is set, requests from non-whitelisted IPs are rejected
- Configured via the web interface or the `private/create_api_key` / key management endpoints
- Applies per-key, not globally

---

## Positions

### Get All Positions

**Method:** `private/get_positions`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `currency` | string | Yes | Currency: BTC, ETH, USDC, etc. |
| `kind` | string | No | Filter by kind: future, option, spot, future_combo, option_combo |
| `subaccount_id` | integer | No | Subaccount ID |

Returns array of position objects. See `trading_api.md` for full field list.

---

### Get Single Position

**Method:** `private/get_position`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `instrument_name` | string | Yes | Instrument identifier |

See `trading_api.md` for full response field list including Greeks, P&L, margin fields.

---

## Settlement History

### Get Settlement History by Instrument

**Method:** `private/get_settlement_history_by_instrument`
**Scope required:** `trade:read`

#### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `instrument_name` | string | Yes | Instrument identifier |
| `type` | string | No | Settlement type filter |
| `count` | integer | No | Number of records |
| `continuation` | string | No | Pagination token |

---

## Moving Positions Between Subaccounts

Available via `private/move_positions` — transfers open positions between subaccounts without closing them. Requires appropriate scope and that both subaccounts are under the same main account.

---

## Sources

- [Deribit API Documentation](https://docs.deribit.com/)
- [private/get_account_summary reference](https://docs.deribit.com/api-reference/account-management/private-get_account_summary)
- [private/get_account_summaries reference](https://docs.deribit.com/api-reference/account-management/private-get_account_summaries)
- [Deribit Deposits API](https://docs.deribit.com/articles/managing-deposits-api)
- [Deribit Subaccounts API](https://docs.deribit.com/articles/managing-subaccounts-api)
- [Deribit Moving Positions API](https://docs.deribit.com/articles/moving-positions-api)
- [deribit_api WalletApi reference (community)](https://github.com/roman427/deribit_api/blob/master/docs/WalletApi.md)
