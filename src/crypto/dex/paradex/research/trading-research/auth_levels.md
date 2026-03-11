# Paradex DEX - Authentication & Rate Limits

Source: https://docs.paradex.trade/
Researched: 2026-03-11

---

## Authentication Architecture Overview

Paradex uses a **two-layer authentication system** unique to Starknet-based DEXes:

1. **Onboarding** — Register your account once using Ethereum + Starknet signatures
2. **JWT Token** — Short-lived tokens (5 min default) used for all private REST calls
3. **Order Signing** — Every order must be cryptographically signed with STARK private key

There are **no plain API keys**. Authentication is cryptographic at every level.

---

## Key Concepts

### Ethereum Account (L1)
Your Ethereum wallet address used for initial onboarding. Used to prove ownership of the L2 Paradex account.

### Starknet Account / Paradex Private Key (L2)
A STARK-curve compatible private key derived from a signature by your Ethereum account. This is the primary trading key for Paradex.

**Derivation:** Sign a specific message with your Ethereum private key → derive the Starknet (L2) private key from that signature.

### Starknet Public Key / Account Address
Computed from the L2 private key via Stark curve point multiplication. This is your on-chain identity.

### Subkeys
Optional scoped-down keypairs with restricted permissions:
- **Can**: Create orders, modify orders, cancel orders, access private data
- **Cannot**: Initiate withdrawals, transfers, or manage sensitive account settings
- Registered to a main account; the main account address is still passed in API calls
- Recommended for bot/API usage to limit exposure of the main key

---

## Step 1: Onboard (One-Time Registration)

### Endpoint

```
POST https://api.prod.paradex.trade/v1/onboarding
```

**Required headers:**

| Header | Type | Description |
|--------|------|-------------|
| `PARADEX-ETHEREUM-ACCOUNT` | string | Ethereum account address (L1) |
| `PARADEX-STARKNET-ACCOUNT` | string | Starknet/Paradex account address (L2) |
| `PARADEX-STARKNET-SIGNATURE` | string | StarkNet signature proving ownership of L2 account |

**Request body (optional fields):**

| Field | Type | Description |
|-------|------|-------------|
| `public_key` | string | User's L2 public key (required for onboarding) |
| `referral_code` | string | Referral code |
| `marketing_code` | string | Campaign marketing code |
| `utm` | object | UTM tracking `{campaign, source, type}` |

**Response:** HTTP 200 — empty object `{}`
- Operation is **idempotent**: calling again for an already-registered account is safe
- Server verifies the caller owns the Starknet address via signature verification

---

## Step 2: Get JWT Token

JWTs expire after **5 minutes** by default (maximum 1 week with `PARADEX-SIGNATURE-EXPIRATION`). There is no extension mechanism — you must re-authenticate regularly.

### Endpoint

```
POST https://api.prod.paradex.trade/v1/auth
```

**Required headers:**

| Header | Type | Description |
|--------|------|-------------|
| `PARADEX-STARKNET-ACCOUNT` | string | StarkNet account address |
| `PARADEX-STARKNET-SIGNATURE` | string | Cryptographic signature over the auth message |
| `PARADEX-TIMESTAMP` | string | Unix timestamp (seconds) when signature was created |

**Optional headers:**

| Header | Type | Description |
|--------|------|-------------|
| `PARADEX-SIGNATURE-EXPIRATION` | string | Custom expiration timestamp (default: +30 min, max: +1 week) |
| `PARADEX-AUTHORIZE-ISOLATED-MARKETS` | string | Boolean to authorize access to isolated margin markets |

**Request body:** None required — all data transmitted via headers.

**Response: HTTP 200:**
```json
{
  "jwt_token": "eyJhbGc..."
}
```

---

## Step 3: Authenticate Requests

Use the JWT in all private endpoint requests:

```
Authorization: Bearer {jwt_token}
```

---

## Step 4: Sign Orders

Every order (single, batch, or algo) requires a **STARK cryptographic signature** in the request body. This is separate from the JWT — you need both.

### Signing Algorithm

Paradex uses **EIP-712-inspired message signing adapted for StarkNet**:

```
signed_data = Enc[PREFIX_MESSAGE, domain_separator, account, hash_struct(message)]
```

Where:
- `PREFIX_MESSAGE = "StarkNet Message"` (literal string)
- `domain_separator = hash(StarkNetDomain{name, chainId, version})`
- `account` = StarkNet account address
- `hash_struct(message)` = EIP-712 typed data hashing using StarkNet Pedersen hash (not keccak256)

### STARK Curve Parameters

- **Curve**: STARK-friendly elliptic curve (not secp256k1)
- Private key must be a valid STARK curve scalar
- Signature output: two-element array `[r, s]` of field elements
- Submitted as: `"signature": "[r,s]"` (string format in JSON body)
- `signature_timestamp`: Unix milliseconds at the time of signing

### Order-Specific Fields Signed

The order struct being signed includes: `market`, `side`, `type`, `size`, `price`, `instruction`, `flags`, `stp`, `trigger_price`, and `signature_timestamp`.

### Reference Implementations

Official code samples available in Go, Java, and Python:
- https://github.com/tradeparadex/code-samples
- C++ signing library: https://github.com/tradeparadex/starknet-signing-cpp

---

## Authentication for Subkeys

If using a subkey instead of the main private key:

1. The `PARADEX-STARKNET-SIGNATURE` in the auth request is generated by the **subkey private key**
2. The `PARADEX-STARKNET-ACCOUNT` must still be the **main account address** (not the subkey address)
3. Orders are signed with the subkey private key
4. All API calls behave identically — the server resolves the subkey-to-account mapping

---

## Public vs Private Endpoints

### Public Endpoints (No Auth Required)

- `GET /v1/markets` — List all markets
- `GET /v1/markets/{market}` — Market details
- `GET /v1/orderbook/{market}` — Order book
- `GET /v1/trades/{market}` — Recent trades
- `GET /v1/bbo/{market}` — Best bid/offer
- `GET /v1/klines` — Candlestick data
- `GET /v1/funding/data` — Funding rate history
- `GET /v1/system/state` — Exchange operational status
- `POST /v1/onboarding` — Account registration (no JWT required)
- `POST /v1/auth` — Get JWT (no JWT required, uses STARK signature)

### Private Endpoints (JWT Required)

All account, order, position, fill, and transfer endpoints require `Authorization: Bearer {JWT}`.

---

## Rate Limits

### Public Endpoints

| Endpoint | Limit |
|----------|-------|
| Default (all public) | **1500 requests/minute** per IP |
| `POST /v1/onboarding` | **600 requests/minute** per IP |
| `POST /v1/auth` | **600 requests/minute** per IP |

### Private Endpoints

| Endpoint Group | Limit |
|----------------|-------|
| `POST`, `DELETE`, `PUT` on `/v1/orders` | **800 req/sec** OR **17,250 req/min** per account |
| All `GET` endpoints (private) | **120 req/sec** OR **600 req/min** per account |
| All private (IP cap) | **1500 req/min** per IP (shared across all accounts from same IP) |

**Key notes:**
- Order write operations (`POST /orders`, `DELETE /orders`) have dramatically higher limits than reads — designed for high-frequency trading
- The IP-level cap of 1500 req/min applies **across all accounts** from the same IP address
- A single batch order request (`POST /v1/orders/batch`) counts as **1 rate limit unit** regardless of how many orders (1–10) it contains — 50x efficiency vs individual requests
- No weight system is used (every request is equal weight within its category)

### Open Orders Limit

There is an **"Open Orders per Account"** limit (exact number not specified in the public documentation pages retrieved). The trading docs reference a "Maximum Open Orders" constraint that varies per market.

---

## JWT Lifecycle Best Practices

From API Best Practices documentation:

1. Maintain a **background refresh loop** — generate a new JWT every ~4 minutes (before the 5-min expiry)
2. Never use an expired JWT — requests will fail with 401
3. Store the JWT in memory only; do not persist to disk
4. On startup: onboard (idempotent) → get JWT → begin trading

---

## Error Codes (Authentication-Related)

| Code | Meaning |
|------|---------|
| `UNAUTHORIZED_ERROR` | Missing or invalid JWT |
| `SIGNATURE_INVALID` | STARK signature verification failed |
| `SIGNATURE_EXPIRED` | Signature timestamp too old |
| `NONCE_ERROR` | Invalid nonce on auth request |

---

## WebSocket Authentication

WebSocket private channels require authentication before subscribing:

**WebSocket URL:**
```
wss://ws.api.prod.paradex.trade/v1
```

**Auth message (sent after connecting, before subscribing to private channels):**
```json
{
  "jsonrpc": "2.0",
  "method": "auth",
  "params": {
    "bearer": "{jwt_token}"
  },
  "id": 1
}
```

**Keepalive:** Server sends a ping every **55 seconds**; client must respond within **5 seconds** or connection is dropped. Most WebSocket libraries handle this automatically.

---

## Summary: What You Need to Trade

| Step | What | How |
|------|------|-----|
| 1 | Ethereum private key | Your L1 wallet |
| 2 | Paradex L2 private key | Derived by signing a message with L1 key using STARK curve derivation |
| 3 | Register account | `POST /v1/onboarding` with STARK signature (once) |
| 4 | Get JWT | `POST /v1/auth` with STARK signature (every 5 min) |
| 5 | Sign each order | STARK signature over order struct → `signature` field in order body |
| 6 | Submit order | `POST /v1/orders` with JWT header + signed order body |

---

## Sources

- [API Authentication (trading guide)](https://docs.paradex.trade/trading/api-authentication)
- [Get JWT endpoint](https://docs.paradex.trade/api/prod/auth/get-jwt)
- [Onboarding endpoint](https://docs.paradex.trade/api/prod/authentication/onboarding)
- [Rate limits](https://docs.paradex.trade/api/general-information/rate-limits/api)
- [API URLs](https://docs.paradex.trade/api/general-information/api-urls)
- [WebSocket introduction](https://docs.paradex.trade/ws/general-information/introduction)
- [starknet-signing-cpp](https://github.com/tradeparadex/starknet-signing-cpp)
- [Code samples](https://github.com/tradeparadex/code-samples)
- [Paradex Python SDK](https://tradeparadex.github.io/paradex-py/)
