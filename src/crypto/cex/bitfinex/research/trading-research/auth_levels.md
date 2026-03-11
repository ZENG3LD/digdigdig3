# Bitfinex API — Authentication, Permissions, Rate Limits, Testnet

Source: https://docs.bitfinex.com/reference

---

## 1. API KEY PERMISSIONS

Bitfinex API keys have two primary permission dimensions:

| Permission Level | Scope | Allows |
|---|---|---|
| Read | Account data | View wallets, orders, positions, history, margin info |
| Write | Trading actions | Submit/update/cancel orders, transfers, withdrawals, deposits |

Keys are generated in the Bitfinex web UI under Account Settings → API Keys. Each key can be scoped independently for read and write.

**Available permission granularity (from Key Permissions endpoint):**

The `POST /v2/auth/r/permissions` endpoint returns the API key's current permission set as a JSON object. Permissions cover:

| Permission | Read | Write |
|---|---|---|
| Account | view account info | — |
| History | view order/trade history | — |
| Orders | view active orders | submit/update/cancel orders |
| Positions | view positions | close/claim positions |
| Funding | view funding | manage funding offers |
| Wallets | view balances | request deposits |
| Withdraw | — | initiate withdrawals |
| Alerts | view price alerts | manage price alerts |

**Multiple concurrent connections**: Each API key has its own nonce counter. For multiple simultaneous WebSocket connections or parallel REST clients, generate **separate API keys** for each. Sharing a key across concurrent connections causes nonce conflicts and rejected requests.

---

## 2. AUTHENTICATION MECHANISM

### Method: HMAC-SHA384

All authenticated REST requests require three headers:

| Header | Value |
|---|---|
| `bfx-apikey` | Your API key string |
| `bfx-nonce` | Strictly increasing integer (typically UNIX timestamp in ms) |
| `bfx-signature` | HMAC-SHA384 hex digest of the signature payload |

### Signature Construction

The signature payload string is:

```
/api/v2/{path}{nonce}{body}
```

Where:
- `{path}` = endpoint path, e.g. `/v2/auth/r/wallets`
- `{nonce}` = same nonce value sent in header
- `{body}` = raw JSON request body string (or empty string if no body)

**Example payload for `/v2/auth/r/wallets` with nonce `1609459200000`:**
```
/api/v2/auth/r/wallets1609459200000
```

**Signing:**
```
signature = HMAC_SHA384(payload, api_secret).hex_digest()
```

### Nonce Rules

- Must be **strictly increasing** per API key
- No maximum time window — only ordering matters
- Common strategy: UNIX timestamp in milliseconds (`Date.now()`)
- Maximum value: `9007199254740991` (JavaScript MAX_SAFE_INTEGER)
- If a nonce arrives lower than the last seen, the request is rejected with nonce error

### Request Format

```
POST https://api.bitfinex.com/v2/auth/r/wallets
Content-Type: application/json
bfx-apikey: YOUR_API_KEY
bfx-nonce: 1609459200000
bfx-signature: abc123...hex...

{}
```

Authenticated endpoints always use **POST** method (even read-only ones like wallet/position retrieval).

---

## 3. RATE LIMITS

### REST API

| Scope | Limit | Behavior on Exceed |
|---|---|---|
| General | 10–90 req/min (varies per endpoint) | IP blocked for 60 seconds |
| Order submit/update/cancel | 90 req/min | IP block 60s |
| Order history | 90 req/min | IP block 60s |
| Wallet/position reads | 90 req/min | IP block 60s |

Error response when rate limited:
```json
{"error": "ERR_RATE_LIMIT"}
```

The exact per-endpoint limits are not fully published. The documented value of **90 req/min** applies to order management and authenticated read endpoints. Some endpoints may have lower limits (as low as **10 req/min**).

### WebSocket

| Connection Type | Rate Limit |
|---|---|
| `wss://api.bitfinex.com` (authenticated) | 5 connections per 15 seconds |
| `wss://api-pub.bitfinex.com/` (public) | 20 connections per minute |
| Channels per connection | 25 maximum |

**WebSocket rate limit penalties:**
- `api.bitfinex.com` exceeded: rate limited for **15 seconds**
- `api-pub.bitfinex.com` exceeded: rate limited for **60 seconds**

---

## 4. TESTNET / PAPER TRADING

Bitfinex does **not** offer a dedicated public testnet API environment (unlike Binance Testnet or Bybit Testnet). Instead they provide:

### Paper Trading via Sub-Accounts

- Create a **paper trading sub-account** via the Bitfinex web UI
- Paper trading sub-accounts use simulated balances
- API access works the same way — same endpoints, same authentication
- Paper trading uses the **live API domain** `https://api.bitfinex.com`
- Supported paper trading pairs:
  - `tTESTBTCF0:TESTUSDTF0` (derivatives paper trading)
  - `tTESTBTCTESTUSD` and `tTESTBTCTESTUSDT` for spot paper trading

**Paper trading setup:**
1. Create a sub-account in Bitfinex settings
2. Enable "Paper Trading" mode on the sub-account
3. Generate API keys for the paper trading sub-account
4. Use standard API endpoints — the sub-account context handles paper vs real

**Limitation**: Paper trading only supports a subset of instruments. Full production trading is only available on live accounts.

---

## 5. BASE URLS

| Environment | REST Base URL | WebSocket URL |
|---|---|---|
| Production | `https://api.bitfinex.com` | `wss://api.bitfinex.com/ws/2` |
| Public data | `https://api-pub.bitfinex.com` | `wss://api-pub.bitfinex.com/ws/2` |

All authenticated endpoints use `https://api.bitfinex.com/v2/auth/...`.

Public market data endpoints (tickers, candles, order books) are available at `https://api-pub.bitfinex.com/v2/...` and do not require authentication.

---

## 6. CRITICAL IMPLEMENTATION NOTES FOR V5 TRAITS

### Array Response Format (Not JSON Objects)

Bitfinex uses positional arrays instead of named JSON objects for all responses. This is fundamentally different from most exchanges.

**Wrong assumption:**
```json
{"id": 123, "symbol": "tBTCUSD", "amount": 0.1}
```

**Actual Bitfinex response:**
```json
[123, null, null, "tBTCUSD", 1609459200000, 1609459200000, 0.1, 0.1, "EXCHANGE LIMIT", ...]
```

Parsing requires index-based access. Null placeholders at certain indices must be skipped. New fields may be appended at the end of arrays in future API versions without a version bump.

### Exchange vs Margin Order Types

Routing depends entirely on the order type string prefix:
- `EXCHANGE LIMIT` → exchange wallet (spot)
- `LIMIT` → margin wallet (requires margin funding)

For a standard spot trading implementation, always use `EXCHANGE *` variants.

### Symbol Format

- Spot pairs: `t` prefix + uppercase, e.g. `tBTCUSD`, `tETHUSD`, `tBTCEUR`
- Derivatives: `t` + base + `F0:` + quote + `F0`, e.g. `tBTCF0:USTF0`
- Funding currencies: `f` prefix, e.g. `fUSD`, `fBTC`

### All Authenticated Endpoints Use POST

Even endpoints that only retrieve data (wallets, positions, orders) use `POST` method. This is required for the HMAC signing to include the request body in the signature.

---

## Sources

- https://docs.bitfinex.com/docs/requirements-and-limitations
- https://docs.bitfinex.com/docs/rest-auth
- https://docs.bitfinex.com/reference/rest-auth-key-permissions
- https://support.bitfinex.com/hc/en-us/articles/900001525006-Paper-Trading-at-Bitfinex-test-learn-and-simulate-trading-strategies
- https://docs.bitfinex.com/docs/introduction
