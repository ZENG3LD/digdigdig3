# OKX Authentication, Rate Limits & Testnet — V5 Reference

---

## 1. API KEY TYPES

### Permission Levels

OKX API keys support three independent permission flags. Multiple permissions can be combined on a single key.

| Permission | Value in `perm` field | What it allows |
|-----------|----------------------|----------------|
| **Read** | `read_only` | View account info, balances, order history, positions, bills |
| **Trade** | `trade` | Place/cancel/amend orders, set leverage, transfers, margin settings |
| **Withdraw** | `withdraw` | Initiate withdrawals to external addresses |

Example `perm` field in account config response: `"perm": "read_only,withdraw,trade"`

### Passphrase (OKX-Unique Requirement)

OKX requires a **passphrase** at API key creation time. This is a critical difference from most exchanges:

- Set once during key creation in the OKX web/app UI
- Stored as a salted hash — **cannot be recovered if lost**
- If lost, must delete the key and create a new one
- Sent on every request as the `OK-ACCESS-PASSPHRASE` header
- Cannot be changed after creation

### IP Binding

- Optional but strongly recommended
- Up to **20 IP addresses** per key
- Supports IPv4, IPv6, and CIDR network segments
- Keys with `trade` or `withdraw` permissions **without IP binding** expire after **14 days of inactivity**
- Inactivity = no API calls requiring authentication
- Demo trading keys never expire regardless of IP binding

### Sub-Account Keys

- Each sub-account has its own independent API keys
- Sub-account keys cannot access the master account
- Master account can manage sub-account keys via:
  - `POST /api/v5/users/subaccount/apikey` — create
  - `GET /api/v5/users/subaccount/apikey` — list
  - `POST /api/v5/users/subaccount/modify-apikey` — update
  - `POST /api/v5/users/subaccount/delete-apikey` — delete
- Each sub-account has its own rate limit counters (sub-account level limits)

---

## 2. AUTHENTICATION MECHANISM

### Required Headers (All Private REST Requests)

| Header | Format | Example |
|--------|--------|---------|
| `OK-ACCESS-KEY` | Raw API key string | `"your-api-key-here"` |
| `OK-ACCESS-SIGN` | Base64(HMAC-SHA256(prehash, secret)) | `"Bh6a8JfkGC..."` |
| `OK-ACCESS-TIMESTAMP` | ISO 8601 UTC timestamp | `"2024-01-15T10:30:45.123Z"` |
| `OK-ACCESS-PASSPHRASE` | Your passphrase | `"mypassphrase"` |

**Content-Type** must be `application/json` for POST requests.

### Signature Construction

The prehash string = concatenation of:
```
timestamp + method + requestPath + body
```

Rules:
- `timestamp`: ISO 8601 format with milliseconds (e.g. `2024-01-15T10:30:45.123Z`)
- `method`: UPPERCASE HTTP method (`GET`, `POST`, `DELETE`)
- `requestPath`: Full path **including query string** for GET requests (e.g. `/api/v5/account/balance?ccy=BTC`)
- `body`: Raw JSON string for POST requests; **empty string** `""` for GET requests

### Signing Algorithm

```
prehash = timestamp + "POST" + "/api/v5/trade/order" + '{"instId":"BTC-USDT-SWAP",...}'
signature = Base64(HMAC-SHA256(prehash, secret_key))
```

### Request Expiry

- Requests are **rejected if timestamp is more than 30 seconds** before server time
- Server returns error `50102` ("Timestamp request expired") if outside the window
- Always use UTC time; sync with server time via `GET /api/v5/public/time`

### Example: Python Signature

```python
import hmac, hashlib, base64
from datetime import datetime, timezone

def sign(timestamp: str, method: str, path: str, body: str, secret: str) -> str:
    prehash = timestamp + method.upper() + path + (body or "")
    mac = hmac.new(secret.encode(), prehash.encode(), hashlib.sha256)
    return base64.b64encode(mac.digest()).decode()

def get_timestamp() -> str:
    return datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.') + \
           f"{datetime.now(timezone.utc).microsecond // 1000:03d}Z"
```

### Example: Rust Signature

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

fn sign(timestamp: &str, method: &str, path: &str, body: &str, secret: &str) -> String {
    let prehash = format!("{}{}{}{}", timestamp, method.to_uppercase(), path, body);
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}
```

---

## 3. RATE LIMITS

### Rate Limit Architecture

OKX uses **two parallel rate limit systems** that both apply simultaneously:

1. **Per-Endpoint Limits**: Each endpoint has its own requests-per-window limit
2. **Sub-Account Limit**: Max 1000 order (place + amend) requests/2s across ALL instruments

### Rate Limit Rules

| Rule Type | How Measured | Who it Applies To |
|-----------|-------------|-------------------|
| `User ID` | Counted per UID | Account-wide |
| `Instrument ID` | Counted per instId per UID | Per trading pair |
| `Instrument Family` | Counted per instrument family | Options |
| `IP` | Counted per IP | Public endpoints only |

### Trading Endpoints Rate Limits

| Endpoint | Limit | Rule |
|----------|-------|------|
| `POST /api/v5/trade/order` | Per Instrument ID | Instrument ID + User ID |
| `POST /api/v5/trade/batch-orders` | Per Instrument ID (shared with single) | Instrument ID + User ID |
| `POST /api/v5/trade/cancel-order` | Per Instrument ID | Instrument ID + User ID |
| `POST /api/v5/trade/cancel-batch-orders` | Per Instrument ID | Instrument ID + User ID |
| `POST /api/v5/trade/amend-order` | Per Instrument ID | Instrument ID + User ID |
| `POST /api/v5/trade/amend-batch-orders` | Per Instrument ID | Instrument ID + User ID |
| `POST /api/v5/trade/close-position` | 20 req/2s | User ID |
| `POST /api/v5/trade/order-algo` | 20 req/2s | User ID |
| `POST /api/v5/trade/cancel-algos` | 20 req/2s | User ID |
| `POST /api/v5/trade/amend-algos` | 20 req/2s | User ID |
| `GET /api/v5/trade/order` | 60 req/2s | User ID |
| `GET /api/v5/trade/orders-pending` | 60 req/2s | User ID |
| `GET /api/v5/trade/orders-history` | 40 req/2s | User ID |
| `GET /api/v5/trade/orders-history-archive` | 20 req/2s | User ID |
| `GET /api/v5/trade/fills` | 60 req/2s | User ID |
| `GET /api/v5/trade/fills-history` | 10 req/2s | User ID |
| `GET /api/v5/trade/orders-algo-pending` | 20 req/2s | User ID |
| `GET /api/v5/trade/orders-algo-history` | 20 req/2s | User ID |

**Important notes on trading limits:**
- Place, cancel, and amend limits are **independent from each other**
- REST and WebSocket order operations share the same rate limit counters
- Batch endpoints: each order in the batch counts individually
- Sub-account global cap: **1000 new+amend orders/2s** across all instruments combined
- ELP (Enhanced Liquidity Provider) taker access: 50 orders/2s per instrument

### Account Endpoints Rate Limits

| Endpoint | Limit | Rule |
|----------|-------|------|
| `GET /api/v5/account/balance` | 10 req/2s | User ID |
| `GET /api/v5/account/config` | 20 req/2s | User ID |
| `GET /api/v5/account/positions` | 10 req/2s | User ID |
| `GET /api/v5/account/positions-history` | 1 req/10s | User ID |
| `GET /api/v5/account/leverage-info` | 20 req/2s | User ID |
| `POST /api/v5/account/set-leverage` | 40 req/2s | User ID + Inst Family |
| `POST /api/v5/account/set-position-mode` | 5 req/2s | User ID |
| `GET /api/v5/account/max-size` | 20 req/2s | User ID |
| `GET /api/v5/account/max-avail-size` | 20 req/2s | User ID |
| `POST /api/v5/account/position-margin` | 20 req/2s | User ID |
| `GET /api/v5/account/max-loan` | 20 req/2s | User ID |
| `GET /api/v5/account/trade-fee` | 5 req/2s | User ID |
| `GET /api/v5/account/bills` | 5 req/s | User ID |
| `GET /api/v5/account/bills-archive` | 5 req/2s | User ID |
| `GET /api/v5/account/interest-accrued` | 5 req/2s | User ID |
| `GET /api/v5/account/interest-rate` | 5 req/2s | User ID |
| `GET /api/v5/account/risk-state` | 10 req/2s | User ID |
| `GET /api/v5/account/instruments` | 20 req/2s | User ID + Inst Type |

### Asset Endpoints Rate Limits

| Endpoint | Limit | Rule |
|----------|-------|------|
| `POST /api/v5/asset/transfer` | 1 req/s | User ID |
| `GET /api/v5/asset/transfer-state` | 10 req/s | User ID |
| `GET /api/v5/asset/deposit-address` | 6 req/s | User ID |
| `GET /api/v5/asset/deposit-history` | 6 req/s | User ID |
| `POST /api/v5/asset/withdrawal` | 6 req/s | User ID |
| `GET /api/v5/asset/withdrawal-history` | 6 req/s | User ID |

### Public Endpoints Rate Limits (No Auth)

| Endpoint | Limit | Rule |
|----------|-------|------|
| `GET /api/v5/public/instruments` | 20 req/2s | IP |
| `GET /api/v5/market/tickers` | 20 req/2s | IP |
| `GET /api/v5/market/ticker` | 20 req/2s | IP |
| `GET /api/v5/market/books` | 20 req/2s | IP |
| `GET /api/v5/market/candles` | 40 req/2s | IP |
| `GET /api/v5/market/history-candles` | 20 req/2s | IP |
| `GET /api/v5/market/trades` | 100 req/2s | IP |
| `GET /api/v5/public/time` | 10 req/2s | IP |

### WebSocket Rate Limits

| Action | Limit |
|--------|-------|
| WebSocket connection establishment | 3 req/s per IP |
| Subscribe/unsubscribe/login operations | 480 times/hour |
| Order placement via WebSocket | Shared with REST per-instrument limit |
| Private channel subscriptions | Max 240 per connection |

### Rate Limit Error Codes

| Code | Message | Meaning |
|------|---------|---------|
| `50011` | Rate limit reached | Per-endpoint limit exceeded |
| `50061` | Sub-account rate limit exceeded | 1000 orders/2s sub-account cap exceeded |

### Rate Limit Headers

OKX does **not** return rate limit headers in REST responses. Use the documented limits directly.

---

## 4. TESTNET / DEMO TRADING

### Access Method

1. Login to OKX web/app
2. Go to Trade → Demo Trading
3. Go to Personal Center → Demo Trading API
4. Create a Demo Trading API Key
5. Use the same credentials format as production

Demo keys are separate from production keys — they are created specifically in the demo trading UI.

### Endpoints

**REST (same host as production):**
```
https://www.okx.com
```

**WebSocket Public (demo):**
```
wss://wspap.okx.com:8443/ws/v5/public
```

**WebSocket Private (demo):**
```
wss://wspap.okx.com:8443/ws/v5/private
```

**WebSocket Business (demo):**
```
wss://wspap.okx.com:8443/ws/v5/business
```

**Production WebSocket (for comparison):**
```
wss://ws.okx.com:8443/ws/v5/public
wss://ws.okx.com:8443/ws/v5/private
wss://ws.okx.com:8443/ws/v5/business
```

### Required Header for Demo REST Requests

Every demo REST request must include:
```
x-simulated-trading: 1
```

This header is what routes the request to the demo environment. Without it, requests hit production.

### Demo Trading Differences

| Feature | Production | Demo |
|---------|-----------|------|
| API key expiry | Expires after 14d inactivity (trade/withdraw without IP) | **Never expires** |
| Deposits | Real funds | Simulated |
| Withdrawals | Real | **Not supported** |
| Funding transfers | Real | **Not supported** |
| Purchase/redemption | Real | **Not supported** |
| Trading | Real | Simulated |
| Market data | Real | Real |
| Order book | Real | Real |
| Rate limits | Production limits | Production limits |
| WebSocket host | `ws.okx.com` | `wspap.okx.com` |

### Demo Limitations

- Cannot test fund withdrawal flows
- Cannot test deposit flows
- Cannot test purchase or redemption products
- WebSocket uses different hostname
- Some account-level features may behave differently

---

## 5. V5 API DESIGN NOTES FOR RUST TRAIT IMPLEMENTATION

### Critical Design Differences vs Other Exchanges

1. **Separate algo order system**: Regular orders and TP/SL/trigger orders use completely different endpoints. V5 traits need two distinct subsystems:
   - `TradeApi` → `/api/v5/trade/*`
   - `AlgoApi` → `/api/v5/trade/order-algo`, `/api/v5/trade/cancel-algos`, etc.

2. **tdMode is mandatory on every order**: Unlike Binance where you pick a market type upfront, OKX requires `tdMode` (cash/isolated/cross) on every order. This must be tracked per-order in your request builder.

3. **instType is inferred, not sent**: For most trade endpoints, `instType` is not a request field — it is determined by the `instId` format:
   - `BTC-USDT` → SPOT or MARGIN (depends on tdMode)
   - `BTC-USDT-SWAP` → SWAP
   - `BTC-USD-241227` → FUTURES
   - `BTC-USD-241227-50000-C` → OPTION

4. **Unified account balance**: One `GET /api/v5/account/balance` call returns everything. No need to call separate spot/margin/futures balance endpoints.

5. **posSide is mode-dependent**:
   - In `long_short_mode`: must specify `long` or `short` on every derivative order
   - In `net_mode`: omit `posSide` or use `net`
   - Check account config first, then implement accordingly

6. **Close position is a first-class operation**: `POST /api/v5/trade/close-position` is the proper way to close derivatives, not placing a reverse order manually.

7. **sCode vs code**: Top-level `code` is for HTTP-level success. Per-item `sCode` in batch responses can be non-zero even when top-level `code` is `"0"`. Always check both.

8. **Amendment confirmation**: `POST /api/v5/trade/amend-order` response confirms the request was **received**, not that it was processed. Subscribe to the orders WebSocket channel to get the actual amendment result.

9. **All numeric values are strings**: `px`, `sz`, `lever`, `fee`, `upl` etc. are all `String` type in JSON. Parse carefully.

---

## Sources

- [OKX API v5 Official Docs — Overview](https://www.okx.com/docs-v5/en/)
- [OKX API Key Documentation](https://www.okx.com/docs-v5/en/#overview-api-key)
- [OKX Demo Trading Guide](https://www.okx.com/docs-v5/en/#overview-demo-trading)
- [OKX Sub-Account Rate Limit Announcement](https://www.okx.com/help/fill-ratio-sub-account-rate-limit)
- [OKX Rate Limit Overview](https://www.okx.com/docs-v5/en/#overview-rate-limit)
- [OKX API v5 Complete Guide](https://www.okx.com/en-us/learn/complete-guide-to-okex-api-v5-upgrade)
