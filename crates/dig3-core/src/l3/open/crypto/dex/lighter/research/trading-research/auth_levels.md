# Lighter.xyz Authentication and Rate Limits

Source: https://apidocs.lighter.xyz
Base URL (Mainnet): `https://mainnet.zklighter.elliot.ai`

---

## Authentication Architecture

Lighter is a ZK-rollup DEX — not a CEX. Authentication is wallet-based and
cryptographic. There are no username/password logins. All trading actions are
**signed transactions** submitted to the rollup.

### Authentication Flow Overview

```
1. User has Ethereum wallet (L1 private key)
2. User registers with Lighter: signs a message with Ethereum wallet
   → This associates the L1 address with a Lighter account_index
3. User generates API key(s): ed25519 key pair, associated with the account
   → API key registration requires signing a ChangePubKey transaction on L1
4. Auth tokens are generated from the API private key
5. Auth tokens are included in API requests (header or query param)
6. Trading transactions are signed with API private key before submission
```

---

## API Keys

### Key Structure

- Each API key has: a **public key**, a **private key**, and its own **nonce**
- Key index range: 0 to 254 (255 = query all)
- Reserved indices: 0 (desktop web), 1 (mobile PWA), 2 (mobile app)
- User-created keys: indices 3-254 (up to **252 custom keys** per account)
- Each account (main + each sub-account) has its own separate key set
- Up to **256 API keys** total per account

### Key Creation

Keys can be generated programmatically without the L1 private key. However,
**registering** (associating) a key with Lighter requires either:
- Using the Python or Go SDK (which calls `ChangePubKey` on the smart contract)
- Calling the Lighter smart contract's `ChangePubKey` function directly with the L1 wallet

### Key Management Endpoints

```
GET  /api/v1/apikeys          — List all API keys for an account
GET  /api/v1/nextNonce        — Get the next nonce for an API key
POST /api/v1/tokens/create    — Create a new auth token
POST /api/v1/tokens/revoke    — Revoke an auth token
GET  /api/v1/tokens           — List active tokens
```

---

## Auth Token Formats

### Standard Auth Token

Used for both read and write (trading) operations.

**Format:**
```
{expiry_unix}:{account_index}:{api_key_index}:{random_hex}
```

**Example:**
```
1741200000:42:3:a1b2c3d4e5f6...
```

- Maximum expiry: **8 hours** from creation
- Generated via SDK: `create_auth_token_with_expiry()`
- Or via endpoint: `POST /api/v1/tokens/create`

### Read-Only Auth Token

For query-only access (no trading or withdrawal capability).

**Format:**
```
ro:{account_index}:{single|all}:{expiry_unix}:{random_hex}
```

**Example:**
```
ro:42:single:1772736000:a1b2c3d4e5f6...
```

- Maximum expiry: **10 years** (minimum: 1 day)
- `single` = access to one account; `all` = access to all sub-accounts
- Cannot sign transactions or process withdrawals

---

## How Auth is Passed in Requests

Two equivalent methods (use either):

**Option 1 — Query parameter:**
```
GET /api/v1/accountActiveOrders?account_index=42&market_id=0&auth=<token>
```

**Option 2 — HTTP Header:**
```
GET /api/v1/accountActiveOrders?account_index=42&market_id=0
Authorization: <token>
```

Most account-level read endpoints list `auth`/`authorization` as **optional**
(public account data can be read without auth). Private data channels and
trading actions require auth.

---

## Transaction Signing (for Write Operations)

All order placement, cancellation, and modification operations are **signed L2
transactions**, not plain HTTP requests.

### Signing Process

1. Initialize `SignerClient` with your API private key and account index
2. Call a sign method (e.g., `sign_create_order`) — this produces a signed payload
3. Submit via `POST /api/v1/sendTx` or WebSocket `jsonapi/sendtx`

**Python SDK signer initialization:**
```python
client = lighter.SignerClient(
    url="https://mainnet.zklighter.elliot.ai",
    api_private_keys={API_KEY_INDEX: API_PRIVATE_KEY},
    account_index=ACCOUNT_INDEX
)
```

### Nonce Requirements

- Each API key has its own nonce, incremented by 1 with each transaction
- The SDK manages nonces automatically
- Manual nonce management is supported for complex systems (e.g., multiple keys
  for different order types to maximize throughput)
- Retrieve current nonce: `GET /api/v1/nextNonce`

---

## Account Tiers

Two tiers affect rate limits, fees, and latency:

### Standard Account (Default)

| Attribute | Value |
|---|---|
| Maker fee | 0% |
| Taker fee | 0% |
| Taker order latency | 300 ms |
| Maker/cancel latency | 200 ms |
| sendTx/sendTxBatch per minute | Up to standard REST rate limit |

### Premium Account (Opt-in, requires staked LIT tokens)

Latency for maker and cancel orders is **0ms** on Premium.

| Staked LIT | TX/min | Fee Discount | Maker Fee | Taker Fee | Taker Latency |
|---|---|---|---|---|---|
| 0 | 4,000 | — | 0.0040% | 0.0280% | 200 ms |
| 1,000 | 5,000 | 2.5% | 0.0039% | 0.0273% | 195 ms |
| 3,000 | 6,000 | 5% | 0.0038% | 0.0266% | 190 ms |
| 10,000 | 7,000 | 10% | 0.0036% | 0.0252% | 180 ms |
| 30,000 | 8,000 | 15% | 0.0034% | 0.0238% | 170 ms |
| 100,000 | 12,000 | 20% | 0.0032% | 0.0224% | 160 ms |
| 300,000 | 24,000 | 25% | 0.0030% | 0.0210% | 150 ms |
| 500,000 | 40,000 | 30% | 0.0028% | 0.0196% | 140 ms |

---

## Rate Limits

### REST API — Main Endpoints

Rate limits apply simultaneously at both **IP address level** and **L1 address level**.

| Account Tier | Weight Budget | Window |
|---|---|---|
| **Standard** | 60 weighted requests | Rolling 60 seconds |
| **Premium** | 24,000 weighted requests | Rolling 60 seconds |

**Endpoint Weights:**

| Weight | Endpoints |
|---|---|
| 3 | `sendTx`, `sendTxBatch`, `nextNonce` |
| 50 | `publicPools`, `txFromL1TxHash` |
| 100 | `accountInactiveOrders`, `deposit/latest` |
| 150 | `apikeys` |
| 300 | All other endpoints (default) |
| 500 | `transferFeeInfo` |
| 600 | `trades`, `recentTrades` |
| 3,000 | `changeAccountTier`, `tokens`, `tokens/revoke`, `setAccountMetadata`, `notification/ack`, `createIntentAddress`, `fastwithdraw`, all `referral/*` endpoints |
| 23,000 | `tokens/create` |

Note: `sendTx` and `sendTxBatch` weight is 3 each — they share the same rate limit
pool as the REST API when transactions are sent via WebSocket.

**On exceeding limits:** HTTP 429 Too Many Requests.

Cooldown periods:
- Firewall-level: 60 seconds
- API server-level: calculated based on endpoint weight

### REST API — Explorer Endpoints

Base URL: `https://explorer.elliot.ai`

Both Standard and Premium: **90 weighted requests per rolling minute**.

| Weight | Endpoints |
|---|---|
| 1 | All explorer endpoints (default) |
| 2 | `accounts/*` |
| 3 | `search` |

Special case — `logs` endpoint:
- 100 requests per 60 seconds
- 300 requests per 5 minutes
- 500 requests per 10 minutes

### WebSocket Limits (Per IP Address)

| Parameter | Limit |
|---|---|
| Max connections per IP | 100 |
| Max subscriptions per connection | 100 |
| Max total subscriptions per IP | 1,000 |
| Max new connections per minute | 80 |
| Max messages sent by client per minute | 200 |
| Max inflight (unacknowledged) messages | 50 |
| Max unique accounts per IP | 10 |
| Auto-disconnect after idle | 24 hours |

### Transaction-Type Specific Rate Limits (Per L1 Address)

| Transaction Type | Limit |
|---|---|
| Default (all tx types) | 40 requests/minute |
| `L2UpdateLeverage` | 40 requests/minute |
| `L2Transfer` | 120 requests/minute |
| `L2ChangePubKey` | 300 requests/minute |
| `L2Withdraw` | 2 requests/minute |
| `L2CreateSubAccount` | 2 requests/minute |
| `L2CreatePublicPool` | 2 requests/minute |
| `L2MintShares` | 1 request per 15 seconds |
| `L2UnstakeAssets` | 1 request per 15 seconds |

---

## Volume Quota Program (Premium Accounts Only)

Volume Quota is separate from rate limits — it controls long-term throughput for
order-flow transaction types.

**Which transaction types consume quota:**

| Transaction Type | Quota Cost |
|---|---|
| `L2CreateOrder` (tx_type=14) | 1 unit |
| `L2ModifyOrder` (tx_type=17) | 1 unit |
| `L2CreateGroupedOrders` (tx_type=28) | 1 unit (regardless of group size) |
| `L2CancelAllOrders` (tx_type=5) | 0 (free) |
| Individual cancels | 0 (free) |

**Quota accumulation:**
- Starting quota: 1,000 units
- Earn rate: 1 unit per $5 of trading volume
- Maximum accumulated: 5,000,000 units (never expires)
- Complimentary: 1 free `sendTx` every 15 seconds (no quota consumed)
- Shared across all sub-accounts under the same L1 address
- Each transaction in a `sendTxBatch` consumes 1 quota unit

---

## Colocation

AWS Tokyo is the recommended region for lowest latency colocation with the
Lighter matching engine.

---

## Public vs Private Endpoints Summary

| Access Level | Examples | Auth Required |
|---|---|---|
| **Fully public** | `orderBooks`, `recentTrades`, `candles`, `exchangeStats`, `funding-rates`, `systemConfig`, `info` | No |
| **Account data (readable with index)** | `account?by=index`, `accountActiveOrders` (with account_index) | Technically optional; auth improves limits |
| **Private account data** | WebSocket `account_all_orders`, `account_all_positions`, `account_tx`, `notification` | Yes — auth token required |
| **Write / trading** | `sendTx`, `sendTxBatch`, `changeAccountTier`, `fastwithdraw` | Yes — signed transaction required |
| **Key management** | `tokens/create`, `apikeys` | Yes |

---

## SDK Support

Official SDKs that handle signing and token generation:

| Language | Repository |
|---|---|
| Python | https://github.com/elliottech/lighter-python |
| Go | https://github.com/elliottech/lighter-go |
| TypeScript | `@oraichain/lighter-ts-sdk` on npm |
| Rust | `lighter-rs` on crates.io |

For Rust V5 connector implementation, the `lighter-rs` crate exists on crates.io.
However, for the V5 connector pattern, implementing signing from scratch using
the documented token format and transaction structure is more appropriate than
depending on the SDK crate.

---

## Sources

- https://apidocs.lighter.xyz/docs/api-keys
- https://apidocs.lighter.xyz/docs/rate-limits
- https://apidocs.lighter.xyz/docs/account-types
- https://apidocs.lighter.xyz/docs/get-started
- https://apidocs.lighter.xyz/docs/get-started-for-programmers-1
- https://apidocs.lighter.xyz/docs/volume-quota-program
- https://docs.lighter.xyz/perpetual-futures/sub-accounts-and-api-keys
- https://docs.lighter.xyz/perpetual-futures/account-types
- https://crates.io/crates/lighter-rs
