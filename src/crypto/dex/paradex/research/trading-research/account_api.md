# Paradex DEX - Account API Specification

Source: https://docs.paradex.trade/
Researched: 2026-03-11

## Base URL

```
https://api.prod.paradex.trade/v1
```

All account endpoints require `Authorization: Bearer {JWT}` header.

---

## Account Information

### Get Account Summary

```
GET https://api.prod.paradex.trade/v1/account
Authorization: Bearer {JWT}
```

**Optional query parameter:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `subaccount_address` | string | Sub-account StarkNet address |

**Response: HTTP 200 — AccountSummaryResponse:**

| Field | Type | Description |
|-------|------|-------------|
| `account` | string | User's StarkNet account identifier |
| `account_value` | string | Total account value including unrealized P&Ls |
| `free_collateral` | string | Account value in excess of initial margin required |
| `initial_margin_requirement` | string | Collateral required to open trades for existing positions |
| `maintenance_margin_requirement` | string | Collateral required to maintain existing positions |
| `margin_cushion` | string | Account value exceeding maintenance margin requirement |
| `total_collateral` | string | User's total deposited collateral |
| `settlement_asset` | string | Settlement asset symbol (e.g. `"USDC"`) |
| `status` | string | `"ACTIVE"` or `"LIQUIDATION"` |
| `seq_no` | integer | Unique sequence number for deduplication |
| `updated_at` | integer | Unix timestamp of last account update (ms) |

---

### Get Account Settings

```
GET https://api.prod.paradex.trade/v1/account/settings
Authorization: Bearer {JWT}
```

No query parameters.

**Response: HTTP 200:**

```json
{
  "trading_value_display": "string"
}
```

---

### Get Account History (Metrics Over Time)

```
GET https://api.prod.paradex.trade/v1/account/history?type={type}
Authorization: Bearer {JWT}
```

**Required query parameter:**

| Parameter | Values | Description |
|-----------|--------|-------------|
| `type` | `pnl`, `value`, `volume`, `fee_savings` | Type of historical metric |

**Response: HTTP 200:**

```json
{
  "data": [1234.56, 1235.00, ...],
  "timestamps": [1700000000000, 1700000060000, ...]
}
```

Returns parallel arrays of numeric datapoints and corresponding Unix timestamps (ms).

---

### Get Margin Configuration

```
GET https://api.prod.paradex.trade/v1/account/margin?market={market}
Authorization: Bearer {JWT}
```

**Required query parameter:** `market` (string, e.g. `"BTC-USD-PERP"`)

**Response: HTTP 200:**

```json
{
  "account": "string",
  "margin_methodology": "cross_margin | portfolio_margin",
  "configs": [
    {
      "market": "string",
      "margin_type": "CROSS | ISOLATED",
      "leverage": 20,
      "isolated_margin_leverage": 20
    }
  ]
}
```

Notes:
- `margin_methodology`: `cross_margin` or `portfolio_margin` — Paradex supports both
- Default leverage is the market maximum; users can lower it
- No confirmed PUT endpoint for changing leverage found in public docs at time of research

---

## Balances

### Get Account Balances

The Python SDK exposes `fetch_balances()` which fetches all coin balances for the authenticated account. The REST endpoint is a private endpoint requiring JWT authorization.

Based on SDK patterns and available documentation, the endpoint follows:

```
GET https://api.prod.paradex.trade/v1/balances
Authorization: Bearer {JWT}
```

**Note**: The specific endpoint URL was not directly confirmed via a docs page fetch during this research (page returned 404). The Python SDK confirms this method exists as `fetch_balances()`.

Paradex is primarily a USDC-margined exchange. Settlement asset is typically `USDC`.

---

## Trade History (Fills)

### Get Fill History

The Python SDK exposes `fetch_fills(params)` for fill history. The REST endpoint is a private paginated endpoint.

Based on available documentation, the endpoint follows the pattern of other paginated account endpoints:

```
GET https://api.prod.paradex.trade/v1/fills
Authorization: Bearer {JWT}
```

**Expected query parameters** (inferred from WebSocket fill channel fields and similar endpoints):

| Parameter | Type | Description |
|-----------|------|-------------|
| `market` | string | Filter by market |
| `start_at` | integer | Start time (unix ms) |
| `end_at` | integer | End time (unix ms) |
| `page_size` | integer | Results per page (default: 100) |
| `cursor` | string | Pagination cursor |

**Fill object fields** (from WebSocket `fills` channel — same data structure):

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Fill ID |
| `account` | string | Account address |
| `market` | string | Market symbol |
| `order_id` | string | Parent order ID |
| `client_id` | string | Client-assigned order ID |
| `side` | string | BUY or SELL |
| `size` | string | Fill quantity |
| `price` | string | Fill price |
| `fee` | string | Fee amount |
| `fee_currency` | string | Fee currency |
| `fill_type` | string | Type of fill |
| `flags` | array | Fill flags (e.g. RPI indicator) |
| `liquidity` | string | `MAKER` or `TAKER` |
| `realized_pnl` | string | Realized P&L from this fill |
| `realized_funding` | string | Realized funding from this fill |
| `remaining_size` | string | Remaining unfilled size after this fill |
| `underlying_price` | string | Underlying asset price at fill time |
| `created_at` | integer | Fill timestamp (ms) |

**Note**: The `/v1/fills` endpoint URL is confirmed via Python SDK (`fetch_fills`) and WebSocket channel docs, but the full REST query parameter list was not directly fetchable from a docs page during this research.

---

## Deposit / Withdrawal Flow (Starknet-based)

### Architecture Overview

Paradex is built on Starknet Layer 2. All deposits and withdrawals are processed through Starknet bridge contracts.

Supported bridges:
- **STARKGATE** — Official StarkWare L1↔L2 bridge
- **LAYERSWAP** — Cross-chain liquidity network
- **RHINOFI** — Cross-chain bridge
- **HYPERLANE** — Interoperability protocol

### Get Transfer History

```
GET https://api.prod.paradex.trade/v1/transfers
Authorization: Bearer {JWT}
```

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `cursor` | string | No | Pagination cursor |
| `start_at` | integer | No | Start time (unix ms) |
| `end_at` | integer | No | End time (unix ms) |
| `page_size` | integer | No | Results per page (default: 100) |
| `status` | string | No | `PENDING`, `AVAILABLE`, `COMPLETED`, `FAILED` |

**Response: HTTP 200:**

```json
{
  "next": "cursor_or_null",
  "prev": "cursor_or_null",
  "results": [
    {
      "id": "string",
      "account": "string",
      "amount": "string",
      "token": "string",
      "kind": "DEPOSIT | WITHDRAWAL | UNWINDING | VAULT_DEPOSIT | VAULT_WITHDRAWAL | AUTO_WITHDRAWAL",
      "direction": "IN | OUT",
      "status": "PENDING | AVAILABLE | COMPLETED | FAILED",
      "bridge": "STARKGATE | LAYERSWAP | RHINOFI | HYPERLANE",
      "external_chain": "string",
      "external_account": "string",
      "external_txn_hash": "string",
      "txn_hash": "string",
      "counterparty": "string",
      "created_at": "integer",
      "last_updated_at": "integer",
      "failure_reason": "string",
      "auto_withdrawal_fee": "string",
      "socialized_loss_factor": "string",
      "vault_address": "string",
      "vault_unwind_completion_percentage": "string"
    }
  ]
}
```

**Transfer kinds:**

| Kind | Description |
|------|-------------|
| `DEPOSIT` | Incoming funds from L1 or another chain |
| `WITHDRAWAL` | Outgoing funds to L1 or another chain |
| `UNWINDING` | Vault position unwinding |
| `VAULT_DEPOSIT` | Deposit into a trading vault |
| `VAULT_WITHDRAWAL` | Withdrawal from a trading vault |
| `AUTO_WITHDRAWAL` | Automated withdrawal |

---

## WebSocket Real-Time Account Channels

WebSocket URL: `wss://ws.api.prod.paradex.trade/v1`

All private channels require JWT auth before subscribing.

### Account Updates Channel

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {"channel": "account"},
  "id": 1
}
```

Pushes real-time updates for: `account_value`, `free_collateral`, `initial_margin_requirement`, `maintenance_margin_requirement`, `margin_cushion`.

### Fills Channel (Real-time)

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {"channel": "fills.{market}"},
  "id": 1
}
```

Replace `{market}` with market symbol (e.g. `fills.BTC-USD-PERP`) or use `fills` for all markets.
Delivers fill objects in real-time as orders execute.

### Positions Channel (Real-time)

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {"channel": "positions"},
  "id": 1
}
```

Delivers position updates in real-time as positions change.

### Funding Payments Channel

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {"channel": "funding_payments"},
  "id": 1
}
```

### Transfers Channel

**Subscribe:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {"channel": "transfers"},
  "id": 1
}
```

---

## Important Notes

- All monetary values are returned as **strings** (not floats) to preserve precision
- Paradex is **USDC-margined** — all collateral and P&L are in USDC
- There is **no direct withdrawal endpoint** in the REST API — withdrawals are initiated through the Starknet bridge contracts or the UI
- The platform supports both **cross-margin** and **portfolio margin** methodologies
- Account status `LIQUIDATION` means the account is actively being liquidated

---

## Sources

- [Get account information](https://docs.paradex.trade/api/prod/account/get)
- [Get account settings](https://docs.paradex.trade/api/prod/account/get-account-settings)
- [Get account history](https://docs.paradex.trade/api/prod/account/get-account-history)
- [Get account margin configuration](https://docs.paradex.trade/api/prod/account/get-account-margin)
- [List account transfers](https://docs.paradex.trade/api/prod/transfers/get)
- [Funding payments history](https://docs.paradex.trade/api/prod/account/get-funding)
- [List open positions](https://docs.paradex.trade/api/prod/account/get-positions)
- [WebSocket fills channel](https://docs.paradex.trade/ws/web-socket-channels/fills/fills)
- [WebSocket account channel](https://docs.paradex.trade/ws/web-socket-channels/account/account)
- [Paradex Python SDK](https://tradeparadex.github.io/paradex-py/)
