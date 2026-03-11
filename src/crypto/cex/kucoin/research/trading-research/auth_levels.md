# KuCoin Authentication, Rate Limits, and Testnet

## CRITICAL DISTINCTIONS
1. **Passphrase** is REQUIRED — unique among major exchanges (KuCoin + OKX only)
2. **Passphrase must be ENCRYPTED** for API v2/v3 keys (not sent in plaintext)
3. Spot base URL and Futures base URL are **completely different hostnames**
4. KuCoin uses **resource pool quotas** (not simple requests/second)

---

## 1. API KEY TYPES & PERMISSIONS

### Permission Levels
When creating an API key on KuCoin, you assign one or more permissions:

| Permission | Scope | Required For |
|-----------|-------|-------------|
| `General` | Read-only | Account info, balances, order status, trade history |
| `Spot` | Spot trading | Place/cancel spot orders (HF and classic) |
| `Margin` | Margin trading | Borrow, repay, margin orders |
| `Futures` | Futures trading | Place/cancel futures orders, modify leverage |
| `Earn` | Earn products | Subscribe/redeem KuCoin Earn |
| `Withdrawal` | Asset transfer | Withdrawals, transfers out |
| `FlexTransfers` | Universal transfer | Internal account transfers (required for `/api/v3/accounts/universal-transfer`) |

**Note**: `General` permission is the minimum for all READ operations. Write operations require specific trading permissions.

### API Key Versions
| Version | KC-API-KEY-VERSION | Passphrase Handling |
|---------|-------------------|---------------------|
| v1 | `1` or omit | Passphrase sent **in plaintext** (INSECURE, deprecated) |
| v2 | `2` | Passphrase **HMAC-SHA256 encrypted + base64 encoded** |
| v3 | `3` | Same as v2 encryption |

**Always use v2 or v3 keys.** New keys created on KuCoin default to v2.

### IP Restrictions
- Optional: restrict API key to specific IP addresses
- Recommended for production keys

### Sub-Account Keys
- Sub-accounts have their own API keys
- Master account can grant/revoke sub-account permissions
- Sub-account keys can have different permission sets than master

---

## 2. AUTHENTICATION MECHANISM

### Base URLs
| Environment | Type | Base URL |
|-------------|------|----------|
| Production | Spot & Margin | `https://api.kucoin.com` |
| Production | Futures | `https://api-futures.kucoin.com` |
| Sandbox | Spot | `https://openapi-sandbox.kucoin.com` |
| Sandbox | Futures | `https://api-sandbox-futures.kucoin.com` |

### Required Headers for ALL Private Requests
| Header | Value |
|--------|-------|
| `KC-API-KEY` | Your API key string |
| `KC-API-SIGN` | Base64-encoded HMAC-SHA256 signature |
| `KC-API-TIMESTAMP` | Unix timestamp in **milliseconds** |
| `KC-API-PASSPHRASE` | Encrypted passphrase (for v2/v3 keys) |
| `KC-API-KEY-VERSION` | `2` (for v2 keys) |
| `Content-Type` | `application/json` |

Optional header:
| Header | Value | When |
|--------|-------|------|
| `X-SITE-TYPE` | `australia` | For Australia site users only |

### Signature Generation Algorithm

#### Step 1: Build the prehash string
```
prehash = timestamp + METHOD + endpoint + body
```
Where:
- `timestamp` = same milliseconds value as `KC-API-TIMESTAMP` header
- `METHOD` = uppercase HTTP verb: `GET`, `POST`, `DELETE`
- `endpoint` = path + query string for GET/DELETE: e.g. `/api/v1/accounts?currency=BTC`
- `body` = raw JSON body string for POST requests, or `""` (empty string) for GET/DELETE with no body

**CRITICAL**: For GET/DELETE requests, query parameters go in the URL (not body). Include them in `endpoint`. For POST requests, parameters go in the JSON body — `endpoint` is just the path without query string.

#### Step 2: Sign the prehash string
```
KC-API-SIGN = base64(HMAC_SHA256(api_secret, prehash))
```

#### Step 3: Encrypt the passphrase (v2/v3 keys only)
```
KC-API-PASSPHRASE = base64(HMAC_SHA256(api_secret, passphrase))
```

### Pseudocode Example (Rust-style)
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::encode;

type HmacSha256 = Hmac<Sha256>;

fn sign_request(
    api_secret: &str,
    timestamp_ms: i64,
    method: &str,          // "POST", "GET", "DELETE"
    endpoint: &str,        // "/api/v1/hf/orders" (include query string for GET)
    body: &str,            // JSON body string, or "" for GET/DELETE
) -> String {
    let prehash = format!("{}{}{}{}", timestamp_ms, method, endpoint, body);
    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    let result = mac.finalize().into_bytes();
    encode(result)
}

fn encrypt_passphrase(api_secret: &str, passphrase: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes()).unwrap();
    mac.update(passphrase.as_bytes());
    let result = mac.finalize().into_bytes();
    encode(result)
}
```

### Python Example (Reference)
```python
import hmac
import hashlib
import base64
import time

def get_headers(api_key, api_secret, api_passphrase, method, endpoint, body=''):
    timestamp = str(int(time.time() * 1000))
    prehash = timestamp + method.upper() + endpoint + body

    signature = base64.b64encode(
        hmac.new(
            api_secret.encode('utf-8'),
            prehash.encode('utf-8'),
            hashlib.sha256
        ).digest()
    ).decode('utf-8')

    encrypted_passphrase = base64.b64encode(
        hmac.new(
            api_secret.encode('utf-8'),
            api_passphrase.encode('utf-8'),
            hashlib.sha256
        ).digest()
    ).decode('utf-8')

    return {
        'KC-API-KEY': api_key,
        'KC-API-SIGN': signature,
        'KC-API-TIMESTAMP': timestamp,
        'KC-API-PASSPHRASE': encrypted_passphrase,
        'KC-API-KEY-VERSION': '2',
        'Content-Type': 'application/json'
    }
```

### GET Request Example
```
Endpoint: GET /api/v1/accounts?currency=BTC&type=trade
prehash: "1705123456789GET/api/v1/accounts?currency=BTC&type=trade"
body: ""  (empty for GET)
```

### POST Request Example
```
Endpoint: POST /api/v1/hf/orders
body: '{"clientOid":"abc123","symbol":"BTC-USDT","type":"limit","side":"buy","price":"50000","size":"0.001"}'
prehash: "1705123456789POST/api/v1/hf/orders{\"clientOid\":\"abc123\",...}"
```

---

## 3. RATE LIMITS

KuCoin uses a **weight-based quota system** that resets every 30 seconds (for most pools) or 3 seconds (for Unified Account).

### Resource Pools
Limits are **per UID** (not per IP), tracked separately per pool:

| Pool | Reset Window | VIP 0 Quota | VIP 12 Quota |
|------|-------------|-------------|--------------|
| Unified Account | 3 seconds | 200 | 2000 |
| Spot (includes Margin) | 30 seconds | 4000 | 40000 |
| Futures | 30 seconds | 2000 | 20000 |
| Management | 30 seconds | 2000 | 20000 |
| Earn | 30 seconds | 2000 | 2000 |
| CopyTrading | 30 seconds | 2000 | 2000 |
| Public | 30 seconds | 2000 | 2000 |

### Weight Per Endpoint (Known Values)

**Spot HF**:
| Endpoint | Weight |
|----------|--------|
| POST `/api/v1/hf/orders` (limit) | 2 |
| POST `/api/v1/hf/orders` (market) | 2 |
| POST `/api/v1/hf/orders/alter` (modify) | 1 (updated from 3) |
| DELETE `/api/v1/hf/orders/{orderId}` | 1 |
| GET `/api/v1/hf/orders/{orderId}` | 1 |

**Spot Classic**:
| Endpoint | Weight |
|----------|--------|
| POST `/api/v1/orders` | 2 |
| POST `/api/v1/stop-order` | 1 |

**Futures**:
| Endpoint | Weight |
|----------|--------|
| POST `/api/v1/orders` (limit) | 2 |
| POST `/api/v1/st-orders` (TP/SL) | 2 |
| DELETE `/api/v1/orders/{orderId}` | 1 |

**Transfers**:
| Endpoint | Weight |
|----------|--------|
| POST `/api/v2/accounts/inner-transfer` | 10 |

### Rate Limit Headers in Response
Every API response includes rate limit status:

| Header | Description |
|--------|-------------|
| `gw-ratelimit-limit` | Total quota for this resource pool |
| `gw-ratelimit-remaining` | Remaining quota in current window |
| `gw-ratelimit-reset` | Milliseconds until quota resets |

### Rate Limit Errors
- HTTP status: `429`
- Error code: `429000`
- Action: Wait the duration specified in `gw-ratelimit-reset` header before retrying
- Note: Server overload can also trigger 429 even within limits

### Applying for Higher Limits
Users can apply for higher rate limits through KuCoin support (for VIP clients and institutional traders).

---

## 4. TESTNET / SANDBOX

### Sandbox Environment
| Property | Value |
|----------|-------|
| Spot base URL | `https://openapi-sandbox.kucoin.com` |
| Futures base URL | `https://api-sandbox-futures.kucoin.com` |
| Sandbox docs | `https://sandbox-docs.kucoin.com/?lang=en_US` |
| Account creation | Must create SEPARATE sandbox account |

### Key Sandbox Differences
1. **Separate credentials**: Sandbox API key/secret/passphrase are DIFFERENT from production. Cannot reuse production credentials on sandbox.
2. **Separate account**: Must register at the sandbox website to get test funds.
3. **Same API paths**: Endpoint paths are identical to production — only the base URL changes.
4. **Not real money**: All trading is simulated with test funds.
5. **Potential 504 errors**: Sandbox `/api/v1/orders` has historically returned 504 on some requests (known issue).
6. **Public data**: Sandbox market data may not perfectly mirror production prices.

### Switching Between Environments in Rust
```rust
enum KuCoinEnvironment {
    Production,
    Sandbox,
}

impl KuCoinEnvironment {
    fn spot_base_url(&self) -> &'static str {
        match self {
            Self::Production => "https://api.kucoin.com",
            Self::Sandbox => "https://openapi-sandbox.kucoin.com",
        }
    }

    fn futures_base_url(&self) -> &'static str {
        match self {
            Self::Production => "https://api-futures.kucoin.com",
            Self::Sandbox => "https://api-sandbox-futures.kucoin.com",
        }
    }
}
```

---

## 5. ERROR CODES (KEY TRADING ERRORS)

| Code | Meaning |
|------|---------|
| `200000` | Success |
| `400100` | Parameter error (invalid value) |
| `400200` | Insufficient funds |
| `400300` | Invalid order size |
| `400400` | Price too far from market |
| `400500` | Invalid symbol |
| `400600` | API key not found |
| `400700` | Not enough permissions |
| `401000` | Authentication error |
| `403000` | IP not allowed |
| `404000` | Resource not found |
| `429000` | Rate limit exceeded |
| `500000` | Internal server error |

---

## Sources
- [KuCoin Authentication - docs-new](https://www.kucoin.com/docs-new/authentication)
- [KuCoin Signing a Message (legacy)](https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message)
- [KuCoin Creating a Request (legacy)](https://www.kucoin.com/docs/basic-info/connection-method/authentication/creating-a-request)
- [KuCoin Rate Limit - docs-new](https://www.kucoin.com/docs-new/rate-limit)
- [KuCoin REST API Rate Limits (legacy)](https://www.kucoin.com/docs/basic-info/request-rate-limit/rest-api)
- [KuCoin Sandbox Documentation](https://sandbox-docs.kucoin.com/?lang=en_US)
- [KuCoin API Key Upgrade Guideline](https://www.kucoin.com/support/900006465403)
- [KuCoin Introduction - docs-new](https://www.kucoin.com/docs-new/introduction)
