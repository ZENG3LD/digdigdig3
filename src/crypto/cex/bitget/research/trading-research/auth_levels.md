# Bitget V2 Authentication, Permissions, and Rate Limits

---

## Authentication Overview

Bitget V2 uses **HMAC-SHA256** signing with **Base64 encoding**, identical in structure to OKX and KuCoin. A **passphrase** is mandatory — it is set when creating the API key and cannot be changed (must recreate the key to get a new passphrase).

---

## Required Request Headers

All authenticated endpoints require these four headers:

| Header | Value | Description |
|--------|-------|-------------|
| `ACCESS-KEY` | `<your_api_key>` | API key string |
| `ACCESS-SIGN` | `<base64_hmac_sha256>` | Computed signature (see below) |
| `ACCESS-TIMESTAMP` | `<unix_ms>` | Current timestamp in milliseconds since epoch |
| `ACCESS-PASSPHRASE` | `<your_passphrase>` | Passphrase set at API key creation |
| `Content-Type` | `application/json` | Required for POST requests |
| `locale` | `en-US` | Optional; language preference |

---

## Signature Algorithm

### Step 1: Build the pre-sign string

```
prehash = timestamp + METHOD + requestPath [+ "?" + queryString] [+ body]
```

Rules:
- `timestamp` — same value as `ACCESS-TIMESTAMP` header (milliseconds string)
- `METHOD` — HTTP method in uppercase: `GET` or `POST`
- `requestPath` — the path portion only, e.g. `/api/v2/spot/trade/place-order`
- `queryString` — URL-encoded query string (for GET requests), **without** the leading `?`; omit entirely if no query params
- `body` — raw JSON body string (for POST); omit if body is empty

**GET with query params:**
```
1695808690167GET/api/v2/spot/account/assets?coin=USDT
```

**POST with body:**
```
1695808690167POST/api/v2/spot/trade/place-order{"symbol":"BTCUSDT","side":"buy","orderType":"limit","force":"gtc","price":"30000","size":"0.001"}
```

**GET without query params:**
```
1695808690167GET/api/v2/spot/account/assets
```

### Step 2: Sign with HMAC-SHA256

```
signature_bytes = HMAC-SHA256(secret_key, prehash_string)
ACCESS-SIGN = Base64Encode(signature_bytes)
```

Both `secret_key` and `prehash_string` are treated as UTF-8 strings.

### Step 3: Set headers

```
ACCESS-KEY:        your_api_key
ACCESS-SIGN:       <base64 result>
ACCESS-TIMESTAMP:  1695808690167
ACCESS-PASSPHRASE: your_passphrase
```

---

## Rust Implementation Reference

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

fn sign_bitget(
    secret_key: &str,
    timestamp: &str,   // ms since epoch as string
    method: &str,      // "GET" or "POST"
    request_path: &str,// e.g. "/api/v2/spot/trade/place-order"
    query_string: &str,// "" if none
    body: &str,        // "" if none (GET)
) -> String {
    let prehash = if query_string.is_empty() {
        format!("{}{}{}{}", timestamp, method.to_uppercase(), request_path, body)
    } else {
        format!("{}{}{}?{}{}", timestamp, method.to_uppercase(), request_path, query_string, body)
    };

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(prehash.as_bytes());
    let result = mac.finalize();
    general_purpose::STANDARD.encode(result.into_bytes())
}
```

---

## API Key Permissions

Bitget supports granular permission scopes per API key:

| Permission | Description |
|------------|-------------|
| `readonly` | Read-only access to account data, balances, orders, positions |
| `trade` | Place, cancel, modify orders; set leverage/margin mode |
| `transfer` | Transfer assets between accounts |
| `withdraw` | Withdraw funds from Bitget |

Best practice: grant only the minimum permissions required for your use case. For trading bots: `readonly` + `trade` + `transfer` (no `withdraw`).

### Max API Keys per Account

- Up to **10 API keys** per master account
- Each key can have independent permissions
- Sub-accounts each have their own API key quota

### IP Whitelist

Each API key can be restricted to specific IP addresses. Strongly recommended for production trading bots. Up to a configurable list of static IPs per key. Requests from non-whitelisted IPs will be rejected.

### Passphrase Notes

- Set at creation time — **cannot be changed**
- Must recreate the API key to change the passphrase
- Same requirement as OKX and KuCoin V2 APIs
- Stored and sent in plain text in the `ACCESS-PASSPHRASE` header (use HTTPS always)

---

## Rate Limits

Bitget enforces rate limits **per UID** (user account) per endpoint. Exceeding limits returns HTTP `429`.

### Spot Trade Rate Limits

| Endpoint | Limit |
|----------|-------|
| `POST /api/v2/spot/trade/place-order` | 10 req/sec/UID |
| `POST /api/v2/spot/trade/cancel-order` | 10 req/sec/UID |
| `POST /api/v2/spot/trade/batch-orders` | 5 req/sec/UID |
| `POST /api/v2/spot/trade/batch-cancel-order` | 5 req/sec/UID |
| `GET /api/v2/spot/trade/unfilled-orders` | 10 req/sec/UID |
| `GET /api/v2/spot/trade/history-orders` | 10 req/sec/UID |
| `GET /api/v2/spot/trade/fills` | 10 req/sec/UID |

### Futures Trade Rate Limits

| Endpoint | Limit |
|----------|-------|
| `POST /api/v2/mix/order/place-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/cancel-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/modify-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/batch-place-order` | 5 req/sec/UID |
| `POST /api/v2/mix/order/batch-cancel-orders` | 5 req/sec/UID |
| `GET /api/v2/mix/order/orders-pending` | 10 req/sec/UID |
| `GET /api/v2/mix/order/orders-history` | 10 req/sec/UID |
| `GET /api/v2/mix/order/fills` | 10 req/sec/UID |

### Plan Order Rate Limits

| Endpoint | Limit |
|----------|-------|
| `POST /api/v2/mix/order/place-plan-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/place-tpsl-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/place-pos-tpsl-order` | 10 req/sec/UID |
| `POST /api/v2/mix/order/cancel-plan-order` | 10 req/sec/UID |

### Account Rate Limits

| Endpoint | Limit |
|----------|-------|
| `GET /api/v2/spot/account/assets` | 10 req/sec/UID |
| `GET /api/v2/mix/account/accounts` | 10 req/sec/UID |
| `GET /api/v2/mix/position/all-position` | 10 req/sec/UID |
| `POST /api/v2/mix/account/set-leverage` | 10 req/sec/UID |
| `POST /api/v2/mix/account/set-margin-mode` | 10 req/sec/UID |
| `POST /api/v2/spot/wallet/transfer` | 10 req/sec/UID |

### Rate Limit Notes

- Rate limits are **per endpoint per UID** (not shared across endpoints)
- IP-level limits may also apply in extreme cases
- HTTP `429` is returned when limits are exceeded
- Each endpoint's rate limit is documented independently on the Bitget API docs page

---

## Demo / Simulation Trading

Bitget provides a **demo trading** environment using virtual funds. No separate domain — same `api.bitget.com` base URL.

### Method 1: Paper Trading Header (Recommended)

Add the `paptrading` header to any request:

```
paptrading: 1
```

Use normal production `productType` values (`USDT-FUTURES`, etc.) and normal symbols (`BTCUSDT`).

Works with both REST and WebSocket. All order management and account endpoints work identically.

### Method 2: Demo Symbol Pairs

Trade using special demo symbols (no header required):

| Demo Symbol | Equivalent |
|-------------|------------|
| `SBTCUSDT` | BTC/USDT demo |
| `SUSDT-FUTURES` | USDT-M futures demo productType |

### WebSocket Demo URL

```
wss://wspap.bitget.com/v2/ws/public
wss://wspap.bitget.com/v2/ws/private
```

(Normal production WebSocket: `wss://ws.bitget.com/v2/ws/public`)

### Demo Account Setup

- Requires KYC (same account as live trading)
- Virtual funds are available in the demo environment
- Can be accessed via API immediately after enabling demo trading on the website

---

## Response Envelope

All API responses share this envelope:

```json
{
  "code":        "00000",
  "msg":         "success",
  "requestTime": 1695808690167,
  "data":        {}
}
```

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | `"00000"` = success; any other = error |
| `msg` | string | Human-readable message |
| `requestTime` | number | Server timestamp (ms) when request was received |
| `data` | any | Response payload (object or array) |

### Common Error Codes

| Code | Description |
|------|-------------|
| `00000` | Success |
| `40001` | ACCESS-KEY is empty |
| `40002` | ACCESS-SIGN is empty |
| `40003` | ACCESS-TIMESTAMP is empty |
| `40004` | Invalid ACCESS-TIMESTAMP (>5 seconds from server) |
| `40005` | Invalid ACCESS-KEY |
| `40006` | Invalid ACCESS-SIGN |
| `40007` | ACCESS-PASSPHRASE is empty or wrong |
| `40010` | Request too frequent |
| `40011` | IP not in whitelist |
| `43012` | Insufficient balance |
| `45110` | Order does not exist |

---

## Timestamp Tolerance

The server accepts requests where the `ACCESS-TIMESTAMP` is within **±5 seconds** of server time. Requests outside this window are rejected with error `40004`.

Synchronize your clock using the server time endpoint:

**GET** `/api/v2/public/time`

```json
{
  "code": "00000",
  "data": {
    "serverTime": "1695808690167"
  }
}
```

---

## Sources

- [Signature Documentation](https://www.bitget.com/api-doc/common/signature)
- [HMAC Sample Code](https://www.bitget.com/api-doc/common/signature-samaple/hmac)
- [Quick Start Guide](https://www.bitget.com/api-doc/common/quick-start)
- [REST API Demo Trading](https://www.bitget.com/api-doc/common/demotrading/restapi)
- [WebSocket Demo Trading](https://www.bitget.com/api-doc/common/demotrading/websocket)
- [FAQ](https://www.bitget.com/api-doc/common/faq)
- [Bitget API Rate Limits Overview](https://www.bitget.com/wiki/bitget-api-rate-limits)
