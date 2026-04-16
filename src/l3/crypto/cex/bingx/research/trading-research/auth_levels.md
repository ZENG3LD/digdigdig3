# BingX API Authentication & Permission Levels

---

## Authentication Method

### Algorithm

**HMAC-SHA256** — All private requests must be signed using HMAC with SHA-256.

No RSA, Ed25519, or HMAC-SHA384 variants. Only HMAC-SHA256 is documented.

### Header for API Key

```
X-BX-APIKEY: <your_api_key>
```

This header is included in every private REST request. The API key itself is NOT part of the signature calculation — only the secret key is used to sign.

### Signature Construction

**Step 1:** Assemble all request parameters into a query string (alphabetical order is NOT required — any order works):

```
param1=value1&param2=value2&timestamp=<ms_timestamp>
```

For POST requests with a request body, include those body parameters in the same string.

**Step 2:** Generate HMAC-SHA256 signature:

```bash
echo -n "quoteOrderQty=20&side=BUY&symbol=ETHUSDT&timestamp=1649404670162&type=MARKET" | \
  openssl dgst -sha256 -hmac "<secretKey>" -hex
```

Or equivalently in pseudocode:
```
signature = HMAC_SHA256(secret_key, query_string)
```

**Step 3:** Append signature to the request:

```
GET https://open-api.bingx.com/openApi/spot/v1/trade/query?symbol=ETHUSDT&timestamp=1649404670162&type=MARKET&signature=<computed_hex_signature>
```

The `signature` is appended as a query string parameter even for POST requests.

### Required Parameters in Every Private Request

| Parameter | Location | Description |
|-----------|----------|-------------|
| `X-BX-APIKEY` | HTTP Header | The API key value |
| `timestamp` | Query / Body | Unix timestamp in milliseconds at time of request |
| `signature` | Query string | HMAC-SHA256 hex digest |
| `recvWindow` | Query / Body | Optional. Max valid time window in ms. Default: 5000ms. Max: 60000ms |

### Timestamp Validity Window

Requests with a `timestamp` older than 5000 milliseconds (5 seconds) at time of server receipt are rejected with error `100001` (Signature authentication failed).

This window can be extended using the `recvWindow` parameter (up to 60 seconds), but a tighter window is recommended for security.

### Request Rejection Error

| Code | Description |
|------|-------------|
| `100001` | Signature authentication failed — bad key, wrong secret, or expired timestamp |

---

## Permission Levels

### Available Permission Scopes

BingX API keys support the following permission types (confirmed from Cornix, Cryptohopper, Gunbot setup guides and BingX official support):

| Permission | Name | Description |
|------------|------|-------------|
| Read | **Read** | Query account data, balances, positions, order history. Read-only. No trading or fund movement. |
| Spot Trading | **Spot Trading** (or "Spot Account") | Place, cancel, and manage orders in the Spot market |
| Perpetual Futures Trading | **Perpetual Futures Trading** | Place, cancel, and manage orders in the USDT-M perpetual futures market |
| Universal Transfer | **Universal Transfer** | Transfer assets between internal BingX accounts (Spot ↔ Futures ↔ Fund) |
| Sub-Account Management | **Sub-Account Management** | Create and manage sub-accounts, their API keys, and inter-sub-account transfers |
| Withdrawal | **Withdrawal** | Submit withdrawal requests via API |

Notes:
- Default permission when creating a new key: **Read only**
- Permissions can be individually toggled per API key
- Standard trading bots typically need: Read + Spot Trading (and/or Perpetual Futures Trading)
- Withdrawal permission is optional and strongly advised to keep disabled unless explicitly needed

### Coin-M / Inverse Futures Permission

NOT DOCUMENTED as a distinct permission level. Likely covered under "Perpetual Futures Trading" or it may require a separate toggle — NOT CONFIRMED in accessible sources.

### Standard Futures / Copy Trading Permissions

NOT DOCUMENTED in accessible sources. Standard Futures API is NOT publicly available as an open API. Copy Trading API exists but its permission model is not described in accessible sources.

---

## Rate Limits

### General Rate Limit Architecture

BingX uses a rolling-window rate limit system. The CCXT implementation documents a `rateLimit` of 100ms between requests (i.e., up to 10 requests/second at the base level), with a rolling window size of 2000 weight units.

### Market / Public Endpoint Limits

| Scope | Limit |
|-------|-------|
| Market data endpoints (shared) | **100 requests / 10 seconds per IP** |

### Account / Private Endpoint Limits

| Scope | Limit |
|-------|-------|
| Account interfaces per UID | Individual per-endpoint limits apply |
| Account interfaces total per IP | **1,000 requests / 10 seconds** (as of 2024-04-25) |

Historical progression of the IP-level account limit (from upgrade announcements):
- Pre-2024: lower base limit
- 2024-04-15: increased to 150 req/10s
- 2024-04-18: increased to 300 req/10s
- 2024-04-22: increased to 600 req/10s
- 2024-04-25: increased to **1,000 req/10s** (current)

### Order Placement Specific Limit (Futures)

| Endpoint | Limit |
|----------|-------|
| `POST /openApi/swap/v2/trade/order` | **10 requests/second** (upgraded from 5/sec as of 2025-10-16) |

### Endpoint Weight Values (from CCXT implementation)

Individual endpoints consume different amounts of rate limit weight:

| Endpoint | Weight |
|----------|--------|
| `GET /openApi/spot/v1/account/balance` | 1 |
| `GET /openApi/spot/v1/trade/myTrades` | 2 |
| `POST /openApi/spot/v1/trade/batchOrders` | 5 |
| `GET /openApi/swap/v3/user/balance` | 2 |
| `POST /openApi/swap/v2/trade/order` | 2 |
| `POST /openApi/swap/v2/trade/batchOrders` | 2 |
| `DELETE /openApi/swap/v2/trade/batchOrders` | 2 |
| `POST /openApi/swap/v2/trade/leverage` | 5 |
| `POST /openApi/swap/v2/trade/marginType` | 5 |
| `POST /openApi/swap/v2/trade/positionMargin` | 5 |
| `POST /openApi/swap/v1/positionSide/dual` | 5 |

### Rate Limit Error Code

| Code | Meaning |
|------|---------|
| `100410` | Frequency limit error — exceeded rate limit |

### WebSocket Rate Limits

WebSocket frequency limits are enforced separately. Error code `100410` applies to WebSocket frequency violations as well.

---

## IP Restrictions

### IP Whitelisting

BingX supports binding API keys to specific IP addresses. When enabled:
- The API key will **only work from the whitelisted IPs**
- Attempts from non-whitelisted IPs will be rejected

### IP Whitelist Effect on Expiration

API key expiration rules by configuration:

| Configuration | Expiration Behavior |
|---------------|---------------------|
| Key bound to IP address | **No expiration** |
| Key with read-only permission | **No expiration** |
| Key with trading/transfer/subaccount permissions AND no IP binding | **Auto-deleted after 14 days of inactivity** |
| Key with withdrawal permission AND no IP binding | **Not allowed** — withdrawal keys MUST have IP binding |

### Withdrawal Keys Require IP Binding

API keys with the Withdrawal permission cannot be used without IP address binding. BingX enforces this as a security requirement.

### Maximum API Keys

- Up to **20 active API keys** per main account
- Up to **20 active API keys** per sub-account

---

## Public vs Private Endpoints

### Public Endpoints (No Auth Required)

All market data endpoints under these paths require no authentication:

- `GET /openApi/spot/v1/market/*` — Spot market data
- `GET /openApi/spot/v2/market/*` — Spot market data v2
- `GET /openApi/swap/v2/quote/*` — Futures market data
- `GET /openApi/cswap/v1/market/*` — Coin-M futures market data

### Private Endpoints (Auth Required)

All trading, account, position, and wallet endpoints require:
1. `X-BX-APIKEY` header
2. `timestamp` parameter
3. `signature` parameter

---

## Sandbox / Test Environment

BingX provides a test/sandbox environment using VST (virtual settlement token):

**Sandbox Base URL:** `https://open-api-vst.bingx.com`

The sandbox uses the same API structure. VST tokens can be used for testing without real funds.

---

## Error Codes Reference

| Code | Description |
|------|-------------|
| `100001` | Signature authentication failed |
| `100202` | Insufficient balance |
| `100400` | Invalid parameter |
| `100440` | Order price deviates greatly from market price |
| `100410` | Rate limit / frequency limit exceeded |
| `100500` | Internal server error |
| `100503` | Server busy |

HTTP status codes: 4XX for client-side errors, 5XX for server-side errors.

---

## Sources

- [BingX Standard Contract REST API.md (Auth Example)](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [BingX Official API Docs](https://bingx-api.github.io/docs/)
- [CCXT BingX Implementation — Rate Limits & Sign Method](https://raw.githubusercontent.com/ccxt/ccxt/master/python/ccxt/bingx.py)
- [BingX Rate Limit Upgrade Announcement 2025-10-16](https://bingx.com/en/support/articles/31103871611289)
- [Latenode Community — HMAC-SHA256 Signature Generation](https://community.latenode.com/t/what-is-the-method-to-generate-a-bingx-api-signature-using-node-js/513)
- [Gunbot — BingX API Key Creation Guide](https://www.gunbot.com/support/guides/exchange-configuration/creating-api-keys/bingx-api-key-creation/)
- [Cornix — BingX API Permissions](https://help.cornix.io/en/articles/11470179-connect-api-keys-bingx-spot-futures)
- [Cryptohopper — BingX API Key Setup](https://support.cryptohopper.com/en/articles/9388204-how-to-connect-to-bingx-with-api-keys)
- [BingX API Key Security Blog Post](https://bingx.com/en/accounts/api)
