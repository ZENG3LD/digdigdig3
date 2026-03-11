# Lighter.xyz Account API Specification

Source: https://apidocs.lighter.xyz
Base URL (Mainnet): `https://mainnet.zklighter.elliot.ai`

---

## Account Structure Overview

Lighter uses a hierarchical account model:
- One **Ethereum wallet** (L1 address) is the root identity
- The wallet registers one **main account** by signing a message on Lighter's smart contracts
- Up to **sub-accounts** can be created under the main account
- Each account (main or sub) has its own: API keys, nonce, balances, positions, orders

---

## Get Account Info

```
GET https://mainnet.zklighter.elliot.ai/api/v1/account
```

**Parameters:**

| Name | Location | Type | Required | Values |
|---|---|---|---|---|
| `by` | query | string | Yes | `"index"` or `"l1_address"` |
| `value` | query | string | Yes | Account index (int) or Ethereum address (0x...) |

**Response — `DetailedAccount` object:**

| Field | Type | Description |
|---|---|---|
| `index` | integer | Account index |
| `l1_address` | string | Ethereum address |
| `account_type` | integer | Account type classification |
| `available_balance` | string | Balance available for trading/withdrawal |
| `collateral` | string | Total collateral posted |
| `total_asset_value` | string | Sum of all asset values |
| `positions` | array | Array of `AccountPosition` objects |
| `assets` | array | Array of `AccountAsset` objects |
| `pool_info` | object | Public pool details (if account is a pool) |
| `shares` | array | Public pool share information |
| `pending_unlocks` | array | Scheduled unlocking funds |

**AccountPosition fields:**
- `market_index` — market identifier
- Position size, average entry price, position value
- Unrealized PnL, realized PnL
- Open order count

**AccountAsset fields:**
- `symbol` — asset symbol
- `balance` — total balance
- `locked_balance` — balance locked in open orders

---

## Get Accounts by L1 Address

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountsByL1Address
```

Returns all sub-accounts associated with a given Ethereum address.

---

## Get Account Limits

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountLimits
```

**Parameters:**

| Name | Location | Type | Required |
|---|---|---|---|
| `account_index` | query | int64 | Yes |
| `auth` | query | string | No |
| `authorization` | header | string | No |

**Response fields:**

| Field | Type | Description |
|---|---|---|
| `code` | int32 | Response status |
| `message` | string | Response message |
| `max_llp_percentage` | int32 | Maximum LLP (Lighter Liquidity Pool) percentage (example: 25) |
| `user_tier` | string | Account tier (e.g., `"std"` = standard, `"prem"` = premium) |
| `can_create_public_pool` | boolean | Whether this account can create a public liquidity pool |
| `max_llp_amount` | string | Maximum LLP amount in decimal |
| `current_maker_fee_tick` | int32 | Active maker fee tier index |
| `current_taker_fee_tick` | int32 | Active taker fee tier index |
| `effective_lit_stakes` | string | Staked LIT including active leases |
| `leased_lit` | string | Total actively leased LIT |

---

## Get Account Metadata

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountMetadata
```

---

## Get PnL

```
GET https://mainnet.zklighter.elliot.ai/api/v1/pnl
```

Returns profit and loss data for the account.

---

## Get Account Active Orders

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountActiveOrders
```

Returns all currently open/active orders for an account on a specific market.
See `trading_api.md` for full field list.

**Parameters:** `account_index` (int64, required), `market_id` (int16, required), `auth`.

---

## Get Account Inactive Orders (Order History)

```
GET https://mainnet.zklighter.elliot.ai/api/v1/accountInactiveOrders
```

Returns filled, canceled, and expired orders.

**Parameters:** `account_index` (int64, required), `market_id` (int16, default 255 = all),
`ask_filter` (int8, default -1), `between_timestamps`, `cursor`, `limit` (int64, required, 1-100).

---

## Get Liquidations

```
GET https://mainnet.zklighter.elliot.ai/api/v1/liquidations
```

Returns liquidation history for the account.

---

## Get Position Funding

```
GET https://mainnet.zklighter.elliot.ai/api/v1/positionFunding
```

Returns funding payments paid/received per position.

---

## Change Account Tier

```
POST https://mainnet.zklighter.elliot.ai/api/v1/changeAccountTier
```

Switches account between Standard and Premium tiers.

**Constraints:**
- Can only be changed once every 24 hours
- No open orders or positions at time of switch

**Request body (application/x-www-form-urlencoded):**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `account_index` | int64 | Yes | Account to change |
| `new_tier` | string | Yes | Target tier (e.g., `"std"` or `"prem"`) |
| `auth` | string | No | Auth token |

**Response:**
```json
{
  "code": 200,
  "message": "success"
}
```

---

## Deposits and Withdrawals

### How Deposits Work (On-Chain Flow)

1. User initiates a deposit by sending USDC (or supported asset) to Lighter's
   **smart contracts on Ethereum L1**.
2. Lighter's smart contracts hold the tokens in escrow.
3. The Lighter L2 (ZK-rollup) credits the user's account balance once the L1
   transaction is included and the block is produced.

This is a native ZK-rollup deposit — no bridge trust assumptions, funds are
secured by Ethereum and ZK proofs.

**Supported chains for bridging:**
- Ethereum (native)
- Arbitrum
- Avalanche
- Base
- (multi-chain bridging supported)

**Primary collateral asset:** USDC

### Query Deposit History

```
GET https://mainnet.zklighter.elliot.ai/api/v1/deposit/history
```

Returns historical deposit transactions for the account.

### Query Latest Deposit Status

```
GET https://mainnet.zklighter.elliot.ai/api/v1/deposit/latest
```

### Query Available Deposit Networks

```
GET https://mainnet.zklighter.elliot.ai/api/v1/deposit/networks
```

Returns supported networks and addresses for depositing.

### Fast Bridge (Cross-Chain Deposits)

```
POST https://mainnet.zklighter.elliot.ai/api/v1/createIntentAddress
GET  https://mainnet.zklighter.elliot.ai/api/v1/fastbridge/info
```

### How Withdrawals Work (On-Chain Flow)

1. User submits a withdrawal transaction on Lighter (via `sendTx` with withdraw tx_type).
2. Once the block containing that transaction is **ZK-settled** on L1, the funds
   become claimable.
3. User submits a transaction on Ethereum L1 to claim the withdrawn funds.

**Types of withdrawals:**

| Type | Description | Auth Required |
|---|---|---|
| Secure (Standard) Withdrawal | Withdraws to the original L1 address only | API key only |
| Fast Withdrawal | Instant withdrawal (likely via LP liquidity) | May require L1 wallet private key |
| Transfer to other address | Send to a different L1 address | Requires Ethereum wallet private key |

**Fast withdrawal:**
```
POST https://mainnet.zklighter.elliot.ai/api/v1/fastwithdraw
GET  https://mainnet.zklighter.elliot.ai/api/v1/fastwithdraw/info
```

**Withdrawal delay info:**
```
GET https://mainnet.zklighter.elliot.ai/api/v1/withdrawalDelay
```

### Query Withdraw History

```
GET https://mainnet.zklighter.elliot.ai/api/v1/withdraw/history
```

### Query Transfer History

```
GET https://mainnet.zklighter.elliot.ai/api/v1/transfer/history
```

---

## Real-Time Account Data (WebSocket)

Connection: `wss://mainnet.zklighter.elliot.ai/stream`

All account channels require an `auth` token in the subscription message.

| Channel | Description |
|---|---|
| `account_all/{ACCOUNT_ID}` | Full account snapshot + updates across all markets |
| `account_market/{MARKET_ID}/{ACCOUNT_ID}` | Account data for one specific market |
| `user_stats/{ACCOUNT_ID}` | Collateral, leverage, buying power in real-time |
| `account_tx/{ACCOUNT_ID}` | All account transaction history |
| `account_orders/{MARKET_INDEX}/{ACCOUNT_ID}` | Live order updates on one market |
| `account_all_orders/{ACCOUNT_ID}` | All open orders across all markets |
| `account_all_trades/{ACCOUNT_ID}` | Trade fill stream |
| `account_all_positions/{ACCOUNT_ID}` | Position snapshots and updates |
| `account_all_assets/{ACCOUNT_ID}` | Spot asset balances |
| `account_spot_avg_entry_prices/{ACCOUNT_ID}` | Cost basis tracking for spot |
| `notification/{ACCOUNT_ID}` | Liquidation and forced deleverage alerts |
| `pool_data/{ACCOUNT_ID}` | Pool activity (if account is a pool) |
| `pool_info/{ACCOUNT_ID}` | Pool metadata and APY |

---

## Exchange Info Endpoints

### System Config
```
GET https://mainnet.zklighter.elliot.ai/api/v1/systemConfig
```

### Info
```
GET https://mainnet.zklighter.elliot.ai/api/v1/info
```

### Exchange Stats
```
GET https://mainnet.zklighter.elliot.ai/api/v1/exchangeStats
```

### Exchange Metrics
```
GET https://mainnet.zklighter.elliot.ai/api/v1/exchangeMetrics
```

### L1 Basic Info (Ethereum contract info)
```
GET https://mainnet.zklighter.elliot.ai/api/v1/layer1BasicInfo
```

---

## Funding Rates

```
GET https://mainnet.zklighter.elliot.ai/api/v1/funding-rates
GET https://mainnet.zklighter.elliot.ai/api/v1/fundings
```

---

## Sources

- https://apidocs.lighter.xyz/reference/account-1
- https://apidocs.lighter.xyz/reference/accountlimits
- https://apidocs.lighter.xyz/reference/accountactiveorders
- https://apidocs.lighter.xyz/reference/accountinactiveorders
- https://apidocs.lighter.xyz/reference/changeaccounttier
- https://apidocs.lighter.xyz/docs/websocket-reference
- https://docs.lighter.xyz/perpetual-futures/account-types
- https://apidocs.lighter.xyz/
