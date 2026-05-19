# Bybit V5 Authentication, Rate Limits & Testnet

Sources:
- https://bybit-exchange.github.io/docs/v5/guide
- https://bybit-exchange.github.io/docs/v5/rate-limit
- https://bybit-exchange.github.io/docs/v5/user/apikey-info

---

## 1. API KEY TYPES

### Generation Methods

Bybit supports two API key generation methods:

| Method | Encryption | Key Generation | Secret Storage |
|--------|-----------|----------------|----------------|
| **System-generated** | HMAC-SHA256 | Bybit generates both API key + secret | Bybit holds key pair |
| **Self-generated (RSA)** | RSA-SHA256 | User generates RSA keypair locally | User retains private key; only public key submitted to Bybit |

RSA keys are more secure: Bybit never holds the private key.

---

### Permission Scopes

API keys have a `readOnly` flag and granular permissions:

**`readOnly`:**
- `0` = Read + Write (full trading access)
- `1` = Read only (no order placement/cancellation)

**Permission Categories:**

| Permission | Operations Enabled |
|-----------|-------------------|
| `ContractTrade` | Futures/derivatives: place orders, manage positions |
| `Spot` | Spot trading: place orders |
| `Wallet` | AccountTransfer, SubMemberTransfer, Withdraw |
| `Options` | USDC options trading |
| `Derivatives` | Derivatives trading (broader scope) |
| `Exchange` | Convert (swap) history access |
| `Earn` | Access to Earn products |
| `FiatP2P` | P2P trading (master accounts only) |
| `FiatBybitPay` | Bybit Pay transactions |
| `FiatConvertBroker` | Fiat conversion brokerage |
| `BlockTrade` | OTC block trading |
| `Affiliate` | Affiliate program access |

For a typical trading bot, minimum required permissions:
- `ContractTrade` — futures order management
- `Spot` — spot order management
- `Wallet` — for transfers between account types (optional)
- `readOnly: 0` — needed for placing orders

---

### IP Restriction

- Optional but **strongly recommended** for security
- API keys can be bound to a whitelist of IP addresses
- Up to 10 IP addresses per key
- If IP restriction is enabled and request comes from unlisted IP → request rejected with `10004` error code
- Keys without IP restriction are still valid but are considered lower security

---

### Sub-Account API Keys

- Sub-accounts can have their own API keys
- Sub-account keys have same permission structure as master account keys
- `FiatP2P` permission is only available on master account keys
- Master keys can perform sub-account transfers; sub-account keys cannot transfer to other sub-accounts without explicit permission

---

### Key Expiry

API keys can be set with an expiry date (`expiredAt`). The `deadlineDay` field in key info response shows days until expiry. Keys with `deadlineDay: -1` never expire.

---

## 2. AUTHENTICATION

### Required HTTP Headers for Authenticated Requests

| Header | Value |
|--------|-------|
| `X-BAPI-API-KEY` | Your API key |
| `X-BAPI-TIMESTAMP` | Current UTC timestamp in milliseconds |
| `X-BAPI-SIGN` | HMAC-SHA256 or RSA-SHA256 signature |
| `X-BAPI-RECV-WINDOW` | Request validity window in ms (default: `5000`) |

Optional: `X-Referer` or `Referer` header (required only for broker users).

---

### String-to-Sign Construction

**GET requests:**
```
{timestamp}{api_key}{recv_window}{query_string}
```

**POST requests:**
```
{timestamp}{api_key}{recv_window}{json_body_string}
```

Example for GET `/v5/order/realtime?category=linear&symbol=BTCUSDT&limit=10`:
```
1699999999999YOUR_API_KEY5000category=linear&symbol=BTCUSDT&limit=10
```

---

### Signing

**HMAC (system-generated keys):**
```
signature = HMAC_SHA256(secret_key, string_to_sign)
           → encode as lowercase hex string
```

**RSA (self-generated keys):**
```
signature = RSA_SHA256(private_key, string_to_sign)
           → encode as base64 string
```

---

### Timestamp Validation Rule

The server validates:
```
server_time - recv_window <= timestamp < server_time + 1000
```

- If `recv_window = 5000` ms, timestamp must be within 5 seconds of server time
- Maximum allowed `recv_window`: 60,000 ms (60 seconds)
- Use `/v5/market/time` to sync server time if needed

---

## 3. RATE LIMITS

### IP-Level Rate Limits

| Limit Type | Threshold | Penalty |
|-----------|-----------|---------|
| HTTP requests per IP | 600 requests per 5-second window | 10-minute IP ban |
| WebSocket connections | Max 500 new connections per 5 minutes | — |
| WebSocket total | 1,000 total concurrent connections per IP for market data | — |

---

### Per-UID Rate Limits (Per Second)

Rate limits are tracked **per UID** (not per API key). All keys under the same account share the same bucket.

Rate limit headers in HTTP response:
- `X-Bapi-Limit` — maximum requests allowed in current window
- `X-Bapi-Limit-Status` — remaining requests in window
- `X-Bapi-Limit-Reset-Timestamp` — window reset time (ms)

---

### Trade Endpoint Rate Limits

| Endpoint | Method | linear | inverse | option | spot |
|----------|--------|--------|---------|--------|------|
| `/v5/order/create` | POST | 20/s | 10/s | 20/s | 20/s |
| `/v5/order/amend` | POST | 10/s | 10/s | 10/s | 10/s |
| `/v5/order/cancel` | POST | 20/s | 10/s | 20/s | 20/s |
| `/v5/order/cancel-all` | POST | 20/s | 1/s | 20/s | 20/s |
| `/v5/order/create-batch` | POST | 20/s | 10/s | 20/s | 20/s |
| `/v5/order/amend-batch` | POST | 20/s | 10/s | 20/s | 20/s |
| `/v5/order/cancel-batch` | POST | 20/s | 10/s | 20/s | 20/s |
| `/v5/order/realtime` | GET | 50/s | 50/s | 50/s | 50/s |
| `/v5/order/history` | GET | 50/s | — | — | — |
| `/v5/execution/list` | GET | 50/s | 50/s | 50/s | 50/s |

NOTE: These are baseline (non-VIP) limits. Higher VIP tiers receive higher limits.

**Batch operation cost:** `consumed = number_of_requests_per_second × orders_per_batch`. A single batch call with 10 orders counts as 10 against the rate limit, not 1.

---

### Position Endpoint Rate Limits

| Endpoint | Method | Rate Limit |
|----------|--------|------------|
| `/v5/position/list` | GET | 50/s |
| `/v5/position/set-leverage` | POST | 10/s |
| `/v5/position/trading-stop` | POST | 10/s |
| `/v5/position/switch-isolated` | POST | 10/s |
| `/v5/position/switch-mode` | POST | 10/s |
| `/v5/position/add-margin` | POST | 10/s |
| `/v5/position/close-pnl` | GET | 50/s |

---

### Account Endpoint Rate Limits

| Endpoint | Method | Rate Limit |
|----------|--------|------------|
| `/v5/account/wallet-balance` | GET | 50/s |
| `/v5/account/info` | GET | 10/s |
| `/v5/account/fee-rate` | GET | 5-10/s (varies by category) |
| `/v5/account/transaction-log` | GET | 30/s |
| `/v5/user/query-api` | GET | 10/s |

---

### Asset/Transfer Endpoint Rate Limits

| Endpoint | Method | Rate Limit |
|----------|--------|------------|
| `/v5/asset/transfer/inter-transfer` | POST | 20 req/min |
| `/v5/asset/transfer/query-inter-transfer-list` | GET | 60 req/min |
| `/v5/asset/withdraw/create` | POST | 5/s; max 1 per 10s per coin/chain |
| `/v5/asset/withdraw/query-record` | GET | 300 req/min |
| `/v5/asset/deposit/query-record` | GET | 300 req/min |

---

### Rate Limit Error Response

When rate limit is exceeded:
```json
{
  "retCode": 10006,
  "retMsg": "Too many visits!",
  "result": null,
  "retExtInfo": {},
  "time": 1699999999999
}
```

Common error codes:
- `10004` — Invalid sign (authentication failure)
- `10006` — Rate limit exceeded
- `10016` — IP banned (exceeded IP-level limit)
- `10018` — API key expired

---

## 4. TESTNET

### Testnet Base URLs

| Type | URL |
|------|-----|
| REST API | `https://api-testnet.bybit.com` |
| WebSocket Public | `wss://stream-testnet.bybit.com/v5/public/{category}` |
| WebSocket Private | `wss://stream-testnet.bybit.com/v5/private` |

### Production Base URLs

| Type | URL | Notes |
|------|-----|-------|
| REST API (primary) | `https://api.bybit.com` | Global |
| REST API (alternative) | `https://api.bytick.com` | Alternative CDN |
| REST API (Netherlands) | `https://api.bybit.nl` | Region-specific |
| REST API (Turkey) | `https://api.bybit-tr.com` | Region-specific |
| WebSocket Public | `wss://stream.bybit.com/v5/public/{category}` | |
| WebSocket Private | `wss://stream.bybit.com/v5/private` | |

### Testnet Differences

- Separate API keys required (testnet keys do NOT work on mainnet and vice versa)
- Testnet uses simulated funds (no real money)
- `api-testnet.bybit.com` endpoints
- Authentication mechanism identical to production
- Some features may lag production by a version
- Lower rate limits may apply on testnet

### Demo Trading

Bybit also offers **Demo Trading** (different from testnet) accessible at `https://bybit.com` with a demo account. The API endpoint for demo trading is `https://api-demo.bybit.com`. Demo trading simulates real market conditions with virtual funds while using the same authentication and API structure as production.

---

## 5. AUTHENTICATION CODE REFERENCE (Rust Pattern)

```rust
// HMAC authentication for Bybit V5
fn sign_request(api_key: &str, api_secret: &str, timestamp: u64, recv_window: u64, payload: &str) -> String {
    let string_to_sign = format!("{}{}{}{}", timestamp, api_key, recv_window, payload);
    // HMAC-SHA256 → lowercase hex
    hmac_sha256(api_secret.as_bytes(), string_to_sign.as_bytes())
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// Required headers
let headers = [
    ("X-BAPI-API-KEY", api_key),
    ("X-BAPI-TIMESTAMP", &timestamp.to_string()),
    ("X-BAPI-SIGN", &signature),
    ("X-BAPI-RECV-WINDOW", &recv_window.to_string()),
];
```

For GET requests: `payload = query_string` (e.g. `"category=linear&symbol=BTCUSDT"`)
For POST requests: `payload = json_body_string` (e.g. `"{\"category\":\"linear\",\"symbol\":\"BTCUSDT\"}"`)
