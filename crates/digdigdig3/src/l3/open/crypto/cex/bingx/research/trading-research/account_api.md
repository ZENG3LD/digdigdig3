# BingX Account API — Complete Reference

**Base URL:** `https://open-api.bingx.com`

---

## Account Information

### Get Account UID

```
GET /openApi/account/v1/uid
```

Returns the current account's UID. Authentication required.

### Get API Key Permissions

```
GET /openApi/v1/account/apiPermissions
```

Returns the permission flags associated with the current API key.

Source: CCXT bingx.py private endpoint dictionary; confirmed added via GitHub issue #24607.

### Get API Key Restrictions

```
GET /openApi/v1/account/apiRestrictions
```

Returns restrictions (IP whitelist, permissions, etc.) for the current key.

Source: bingx_py client.py module.

### Get Trading Commission Rate (Spot)

```
GET /openApi/spot/v1/user/commissionRate
```

Returns maker and taker fee rates for spot trading.

### Get Trading Commission Rate (Futures)

```
GET /openApi/swap/v2/user/commissionRate
```

Parameters: `recvWindow` (optional), `timestamp`, `signature`.

Returns the current commission rate (maker/taker) for the authenticated account.

### Get Income / Profit-Loss Fund Flow (Futures)

```
GET /openApi/swap/v2/user/income
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | NO | Filter by trading pair |
| `incomeType` | string | NO | Filter by income type (e.g., FUNDING_FEE, COMMISSION, REALIZED_PNL) |
| `startTime` | long | NO | Start time in milliseconds |
| `endTime` | long | NO | End time in milliseconds |
| `limit` | int | NO | Number of records |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

### Fee Schedule

NOT AVAILABLE as a dedicated API endpoint. BingX publishes fee tiers on their website:
- Spot: Maker ~0.1%, Taker ~0.1% (standard). Volume-based tier discounts exist.
- Futures: Maker ~0.02%, Taker ~0.05% (standard). Tier discounts available.

Use the `commissionRate` endpoints above to query the actual rate for the authenticated account.

---

## Balance & Wallet

### Account Structure

BingX has separate account types:
- **Spot Account** — for spot trading assets
- **Fund Account** — general wallet/fund holding area
- **Perpetual Futures Account** — for USDT-M futures margin
- **Coin-M Futures Account** — for inverse futures margin
- **Standard Futures Account** — for standard contracts (no public API)

**Important note:** As of May 2025, BingX completed an account system upgrade separating spot trading funds into a dedicated Spot Account. The V1 spot balance endpoint (`/openApi/spot/v1/account/balance`) now returns only Spot Account assets, NOT total assets across all accounts.

### Get Spot Account Balance

```
GET /openApi/spot/v1/account/balance
```

Parameters: `recvWindow` (optional), `timestamp`, `signature`.

Returns all asset balances in the Spot Account.

### Get All Account Balances (Unified View)

```
GET /openApi/account/v1/allAccountBalance
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `accountType` | string | NO | Filter by account type |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Returns consolidated asset overview across all account types.

Source: bingx_py spot account client — `get_asset_overview()`.

### Get Perpetual Futures Balance

```
GET /openApi/swap/v3/user/balance
```

Parameters: `recvWindow` (optional), `timestamp`, `signature`.

Note: v3 is the current balance endpoint for swap. v2 may also exist but v3 is confirmed via bingx_py.

**Response fields:**
```json
{
  "asset": "USDT",
  "balance": "122607.35137903",
  "crossWalletBalance": "23.72469206",
  "crossUnPnl": "0.00000000",
  "availableBalance": "23.72469206",
  "maxWithdrawAmount": "23.72469206",
  "marginAvailable": true,
  "updateTime": 1617939110373
}
```

### Get Coin-M Futures Balance

```
GET /openApi/cswap/v1/user/balance
```

Source: CCXT bingx.py cswap private endpoints.

### Get Deposit Address

```
GET /openApi/wallets/v1/capital/deposit/address
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `coin` | string | YES | Coin name (e.g., `BTC`, `USDT`) |
| `network` | string | NO | Network/chain (e.g., `TRC20`, `ERC20`) |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Source: CCXT bingx.py wallets v1 private endpoints; bingx-php wallet service.

### Get Deposit History

NOT DOCUMENTED as a separate explicit endpoint in accessible sources. The `capital/config/getall` endpoint returns coin configurations. Check official docs for a dedicated deposit history path.

### Withdraw

```
POST /openApi/wallets/v1/capital/withdraw/apply
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `coin` | string | YES | Coin to withdraw |
| `network` | string | YES | Withdrawal network |
| `address` | string | YES | Destination wallet address |
| `amount` | decimal | YES | Withdrawal amount |
| `addressTag` | string | NO | Memo/tag (required for some coins like XRP) |
| `clientId` | string | NO | Client-assigned withdrawal ID |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

**Security note:** API keys with withdrawal permissions MUST be linked to an IP address whitelist.

### Get Withdrawal History

NOT DOCUMENTED in accessible sources as a distinct endpoint path. NOT AVAILABLE confirmed status: UNCERTAIN — may exist but not found.

### Get Coin Configuration (for supported coins, networks, fees)

```
GET /openApi/wallets/v1/capital/config/getall
```

Returns all supported coins with their deposit/withdrawal network info and fees.

Source: CCXT bingx.py wallets v1 private endpoints.

---

## Internal Asset Transfer

### Transfer Between Accounts

```
POST /openApi/api/v3/post/asset/transfer
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | YES | Transfer type enum (e.g., FUND_SFUTURES for Fund→Futures) |
| `asset` | string | YES | Asset/coin to transfer |
| `amount` | decimal | YES | Amount to transfer |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Also documented as:
```
POST /openApi/api/asset/v1/transfer
```

Used to move funds into the Spot Account from Fund Account.

### Get Transfer Records

```
GET /openApi/api/v3/asset/transfer
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | NO | Transfer type filter |
| `tranId` | string | NO | Transfer ID |
| `startTime` | long | NO | Start time in milliseconds |
| `endTime` | long | NO | End time in milliseconds |
| `current` | int | NO | Page number |
| `size` | int | NO | Page size |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

### Internal Transfer (Main Account — Wallet to Wallet)

```
POST /openApi/wallets/v1/capital/innerTransfer/apply
```

Execute internal wallet transfers. Parameters include `coin`, `transferClientId`, direction, and amount.

Source: bingx_py client.py module.

### Get Internal Transfer Records

```
GET /openApi/wallets/v1/capital/innerTransfer/records
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `coin` | string | NO | Filter by coin |
| `transferClientId` | string | NO | Client-assigned transfer ID |
| `startTime` | long | NO | Start time in milliseconds |
| `endTime` | long | NO | End time in milliseconds |
| `offset` | int | NO | Page offset |
| `limit` | int | NO | Number of records |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Source: bingx_py spot account client — `get_main_account_internal_transfer_records()`.

---

## Margin / Lending

BingX does not offer a traditional margin lending/borrowing API in the accessible documentation. Margin in the context of futures is handled through the position margin endpoints (add/reduce margin — see `trading_api.md`).

- **Borrow/Repay for spot margin:** NOT DOCUMENTED in accessible sources
- **Get borrow history:** NOT DOCUMENTED
- **Get interest rate:** NOT DOCUMENTED
- **Get max borrowable:** NOT DOCUMENTED

Note: BingX offers "Standard Futures" which may have separate margin mechanics, but the Standard Futures API is NOT publicly documented as open API (confirmed: no public Standard Futures open API as of available sources).

---

## Sub-Accounts

Each main account can have up to 20 active API keys per account.

### Create Sub-Account

```
POST /openApi/subAccount/v1/account/create
```

Source: compendium.finance Pendax SDK documentation (`createSubaccount()`).

### Get Sub-Account List

```
GET /openApi/subAccount/v1/account/list
```

Supports pagination. Source: `getSubAccountList()` in Pendax SDK.

### Freeze/Unfreeze Sub-Account

```
POST /openApi/subAccount/v1/account/freeze
```

Source: `setSubaccountFrozen()` in Pendax SDK.

### Get Account UID (for sub-account context)

```
GET /openApi/account/v1/uid
```

### Get Sub-Account Spot Assets

```
GET /openApi/subAccount/v1/assets/spot
```

Source: `getSubaccountSpotAssets()`.

### Get Sub-Account Asset Overview (Batch)

```
POST /openApi/subAccount/v1/assets/overview/batch
```

Check multiple sub-accounts simultaneously. Source: `getSubaccountAssetOverviewBatchInquiry()`.

### Sub-Account Internal Transfer

```
POST /openApi/account/transfer/v1/subAccount/transferAsset
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `fromId` | string | YES | Source account UID |
| `toId` | string | YES | Target account UID |
| `asset` | string | YES | Coin/asset |
| `amount` | decimal | YES | Transfer amount |

Source: GitHub issue #24607 confirmed endpoints; Pendax SDK `subaccountInternalTransfer()`.

### Get Sub-Account Transfer History

```
GET /openApi/account/transfer/v1/subAccount/asset/transferHistory
```

Source: GitHub issue #24607.

### Get Transferable Coins Between Parent-Child

```
POST /openApi/account/transfer/v1/subAccount/transferAsset/supportCoins
```

Source: GitHub issue #24607.

### Sub-Account Deposit Address

```
POST /openApi/subAccount/v1/deposit/address/create
```

Source: `createSubaccountDepositAddress()`.

```
GET /openApi/subAccount/v1/deposit/address
```

Source: `getSubaccountDepositAddress()`.

### Sub-Account Deposit Records

```
GET /openApi/subAccount/v1/deposit/records
```

Source: `getSubaccountDepositRecords()`.

### Sub-Account Transfer Records (Master View)

```
GET /openApi/subAccount/v1/transfer/history
```

Source: `getSubaccsferHistory()` — master account audit log.

### Authorize Sub-Account Internal Transfers

```
POST /openApi/subAccount/v1/transfer/authorize
```

Source: `authorizeSubaccountInternalTransfers()`.

---

## Sub-Account API Key Management

### Create Sub-Account API Key

```
POST /openApi/subAccount/v1/apiKey/create
```

Source: Pendax SDK `createSubaccountApikey()`; confirmed endpoint path in search results.

### Get Sub-Account API Key

```
GET /openApi/subAccount/v1/apiKey/query
```

Source: `getSubaccountApikey()`.

### Reset/Update Sub-Account API Key

```
POST /openApi/subAccount/v1/apiKey/reset
```

Modify permissions and labels. Source: `resetSubaccountApikey()`.

### Delete Sub-Account API Key

```
DELETE /openApi/subAccount/v1/apiKey/delete
```

Revoke sub-account API credentials. Source: `deleteSubaccountApikey()`.

---

## API Key Constraints

- Maximum 20 active API keys per main account (same limit applies to sub-accounts)
- API keys with withdrawal permissions MUST be linked to an IP address
- API keys NOT linked to an IP address AND with trading/transfer/subaccount permissions expire after **14 days of inactivity**
- API keys that ARE linked to an IP address, OR have read-only permissions: **no expiration**

---

## Sources

- [BingX Official API Docs](https://bingx-api.github.io/docs/)
- [CCXT BingX Implementation (bingx.py)](https://raw.githubusercontent.com/ccxt/ccxt/master/python/ccxt/bingx.py)
- [bingx_py Spot Account Client](https://bingx-py.readthedocs.io/en/latest/_modules/bingx_py/client/spot/account.html)
- [bingx_py Swap Account Client](https://bingx-py.readthedocs.io/en/latest/_modules/bingx_py/client/swap/account.html)
- [bingx_py Main Client](https://bingx-py.readthedocs.io/en/latest/_modules/bingx_py/client.html)
- [bingx-php SDK — Wallet & Account Services](https://github.com/tigusigalpa/bingx-php)
- [BingX Sub-Account API Endpoints (GitHub Issue #24607)](https://github.com/ccxt/ccxt/issues/24607)
- [Compendium Finance — Pendax SDK BingX Sub-Account](https://docs.compendium.finance/pendax/using-pendax-sdk/bingx-functions/sub-account-managenent)
- [BingX Standard Contract REST API.md](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [BingX Spot Account Upgrade Announcement](https://bingx.com/en/support/articles/13672873913359)
