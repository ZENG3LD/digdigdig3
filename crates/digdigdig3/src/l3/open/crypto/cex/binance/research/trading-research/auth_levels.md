# Binance API Authentication and Rate Limits

Sources: Binance Spot and Futures REST API documentation (developers.binance.com)

---

## 1. API KEY PERMISSIONS

API keys are created in the Binance web UI under Account > API Management. Each key has a set of toggleable permission flags.

### Permission Flags

| Permission Flag | API Field | What It Enables |
|----------------|-----------|-----------------|
| Read access | `enableReading` | Market data, account info, order history, balance queries |
| Spot & Margin trading | `enableSpotAndMarginTrading` | Place/cancel orders on Spot, Cross Margin, Isolated Margin |
| Futures trading | `enableFutures` | Access to USDM/COIN-M Futures API (`/fapi/`, `/dapi/`) |
| Margin (legacy toggle) | `enableMargin` | Cross-margin trading (older permission system; subsumed by Spot & Margin) |
| Options trading | `enableVanillaOptions` | European options API (`/eapi/`) |
| Portfolio Margin | `enablePortfolioMarginTrading` | Portfolio Margin API (`/papi/`) |
| Withdrawals | `enableWithdrawals` | `POST /sapi/v1/capital/withdraw/apply` — **requires IP whitelist** |
| Internal transfers | `enableInternalTransfer` | Master ↔ Sub-account instant transfers |
| Universal transfer | `permitsUniversalTransfer` | `POST /sapi/v1/asset/transfer` between all account types |

---

### Permission Level Summary

| Access Level | Permissions Needed | Use Case |
|-------------|-------------------|----------|
| Read-only | `enableReading` | Monitoring, analytics, portfolio tracking |
| Spot trading | `enableReading` + `enableSpotAndMarginTrading` | Spot bot |
| Spot + Margin | `enableReading` + `enableSpotAndMarginTrading` + `enableMargin` | Margin bot |
| Futures bot | `enableReading` + `enableFutures` | USDM/COIN-M trading |
| Full trading | All of the above | Multi-product trading system |
| Withdraw | All + `enableWithdrawals` + IP whitelist | Fund management |

---

### IP Whitelist

- **Optional** for all permissions EXCEPT withdrawals.
- **Mandatory** for `enableWithdrawals` — cannot enable withdrawals without IP whitelist.
- Configurable in Binance API management UI.
- Up to N IP addresses can be whitelisted per key.
- If IP whitelist is set and `enableSpotAndMarginTrading` is active, that trading permission does NOT expire (keys without IP whitelist expire after a period of inactivity).
- `ipRestrict` field in `GET /sapi/v1/account/apiRestrictions` shows if IP restriction is active.

---

### Sub-Account API Keys

- Sub-accounts have their own API keys managed through `POST /sapi/v1/sub-account/virtualSubAccount`.
- Sub-account API keys can have the same permission flags as main account keys.
- `enableInternalTransfer` on a main account key allows transfers between master and sub-accounts.
- Sub-account futures position risk: `GET /sapi/v1/sub-account/futures/positionRisk`.
- Sub-accounts cannot enable withdrawals directly.

---

### Query Your API Key Permissions

```
GET /sapi/v1/account/apiRestrictions
```

Weight: 1. Requires `X-MBX-APIKEY` header and signed timestamp.

Response:
```json
{
  "ipRestrict": false,
  "createTime": 1698645219000,
  "enableReading": true,
  "enableSpotAndMarginTrading": true,
  "enableWithdrawals": false,
  "enableInternalTransfer": false,
  "permitsUniversalTransfer": true,
  "enableVanillaOptions": false,
  "enableFutures": true,
  "enableMargin": true,
  "enablePortfolioMarginTrading": false
}
```

---

## 2. RATE LIMITS

### Authentication Header

All authenticated requests require:
```
X-MBX-APIKEY: <your_api_key>
```

Signature goes in the query string or request body as `signature=<hex_or_base64>`.

---

### Spot REST API Rate Limits

Rate limits are **IP-based**, not API-key-based.

| Limit Type | Value | Interval | Header Tracking |
|-----------|-------|----------|-----------------|
| `REQUEST_WEIGHT` | 6,000 | Per minute (1M) | `X-MBX-USED-WEIGHT-1M` |
| `RAW_REQUESTS` | 61,000 | Per 5 minutes (5M) | `X-MBX-USED-WEIGHT-5M` |
| `ORDERS` | 50 | Per 10 seconds (10S) | `X-MBX-ORDER-COUNT-10S` |
| `ORDERS` | 160,000 | Per day (1D) | `X-MBX-ORDER-COUNT-1D` |
| WebSocket connections | 300 | Per 5 minutes | N/A |
| Market data WS connections | 100 | Concurrent | N/A (raised Jan 16 2025) |

**Important:** The order limits (`ORDERS`) are **account-based**, not IP-based. They count unfilled orders placed, not requests. Filled orders do not count against the limit.

**Response headers** on every request:
- `X-MBX-USED-WEIGHT-1M`: Current IP request weight usage per minute.
- `X-MBX-ORDER-COUNT-10S`: Orders placed in last 10 seconds (account-level).
- `X-MBX-ORDER-COUNT-1D`: Orders placed in last day (account-level).

**HTTP Status Codes for Rate Limiting:**
- `429 Too Many Requests`: Rate limit exceeded. Response includes `Retry-After` header.
- `418 I'm a Teapot`: IP has been auto-banned for repeated `429` violations. Ban duration scales: 2 min → 5 min → 30 min → 3 days.

**Endpoint Weight Examples (Spot):**

| Endpoint | Weight |
|----------|--------|
| `GET /api/v3/ping` | 1 |
| `GET /api/v3/depth` | 1–250 (based on limit param) |
| `POST /api/v3/order` | 1 |
| `DELETE /api/v3/order` | 1 |
| `GET /api/v3/order` | 4 |
| `GET /api/v3/openOrders` (1 symbol) | 6 |
| `GET /api/v3/openOrders` (all) | 80 |
| `GET /api/v3/allOrders` | 20 |
| `GET /api/v3/account` | 20 |
| `GET /api/v3/myTrades` | 20 (no orderId) / 5 (with orderId) |
| `GET /api/v3/rateLimit/order` | 40 |
| `GET /api/v3/account/commission` | 20 |
| `POST /api/v3/order/cancelReplace` | 1 |
| `PUT /api/v3/order/amend/keepPriority` | 4 |
| `POST /api/v3/orderList/oco` | 1 |
| `POST /api/v3/sor/order` | 1 |
| `POST /api/v3/sor/order/test` (no commissions) | 1 |
| `POST /api/v3/sor/order/test` (with commissions) | 20 |

---

### Futures (USDM) Rate Limits

**IP Rate Limit:**

| Limit Type | Value | Interval | Header Tracking |
|-----------|-------|----------|-----------------|
| `REQUEST_WEIGHT` | 2,400 | Per minute | `x-mbx-used-weight-1m` |

**Order Rate Limits (per account):**

| Limit | Value | Interval | Header |
|-------|-------|----------|--------|
| Order count | 300 | Per 10 seconds | `X-MBX-ORDER-COUNT-10S` |
| Order count | 1,200 | Per minute | `X-MBX-ORDER-COUNT-1M` |

**HTTP Status Codes:** Same as Spot — `429` and `418` with auto-ban escalation.

**Base URL:** `https://fapi.binance.com`

**Futures Endpoint Weight Examples:**

| Endpoint | IP Weight | Order Count (10s) | Order Count (1m) |
|----------|-----------|-------------------|------------------|
| `POST /fapi/v1/order` | 0 | 1 | 1 |
| `PUT /fapi/v1/order` | 0 | 1 | 1 |
| `DELETE /fapi/v1/order` | 1 | — | — |
| `DELETE /fapi/v1/allOpenOrders` | 1 | — | — |
| `POST /fapi/v1/batchOrders` | 5 | 5 | 1 |
| `PUT /fapi/v1/batchOrders` | 5 | 5 | 1 |
| `DELETE /fapi/v1/batchOrders` | 1 | — | — |
| `GET /fapi/v1/order` | 1 | — | — |
| `GET /fapi/v1/openOrders` (1 symbol) | 1 | — | — |
| `GET /fapi/v1/openOrders` (all) | 40 | — | — |
| `GET /fapi/v1/allOrders` | 5 | — | — |
| `GET /fapi/v2/positionRisk` | 5 | — | — |
| `GET /fapi/v2/account` | 5 | — | — |
| `GET /fapi/v2/balance` | 5 | — | — |
| `POST /fapi/v1/leverage` | 1 | — | — |
| `POST /fapi/v1/marginType` | 1 | — | — |
| `POST /fapi/v1/positionMargin` | 1 | — | — |
| `GET /fapi/v1/leverageBracket` | 1 | — | — |
| `GET /fapi/v1/commissionRate` | 20 | — | — |
| `GET /fapi/v1/income` | 30 | — | — |
| `POST /fapi/v1/positionSide/dual` | 1 | — | — |
| `GET /fapi/v1/positionSide/dual` | 30 | — | — |

**Futures-specific query: current rate limit usage:**
`GET /fapi/v1/rateLimit/order` — Weight: 1.

---

### Margin API Rate Limits

Uses the same Spot IP-based `REQUEST_WEIGHT` system.

**Notable high-weight endpoints:**
- `POST /sapi/v1/margin/borrow-repay`: 1500 (UID-based, not IP-based).
- `GET /sapi/v1/margin/maxBorrowable`: 50.
- `GET /sapi/v1/margin/maxTransferable`: 50.

---

### Wallet/Transfer Rate Limits

- `POST /sapi/v1/asset/transfer`: Weight 900 (UID-based).
- `GET /sapi/v1/capital/deposit/address`: Weight 10.
- `GET /sapi/v1/capital/deposit/hisrec`: Weight 1.
- `GET /sapi/v1/capital/withdraw/history`: Weight 1.

---

### VIP Tier Rate Limits

The official documentation **does not publish VIP-tier-specific rate limit numbers** in the public API docs. VIP tiers primarily affect:
- Commission rates (maker/taker fees).
- Access to certain broker/institutional features.
- Some higher-tier accounts may have negotiated rate limits off-documentation.

The `feeTier` field in `GET /fapi/v2/account` (Futures) reflects the user's VIP level (0 = regular, higher = VIP).

---

## 3. TESTNET

### Spot Testnet

| Property | Value |
|----------|-------|
| REST base URL | `https://testnet.binance.vision` |
| WebSocket API | `wss://ws-api.testnet.binance.vision/ws-api/v3` |
| WebSocket Streams | `wss://stream.testnet.binance.vision/ws` |
| WebSocket Streams (combined) | `wss://stream.testnet.binance.vision/stream` |
| Old deprecated URL | `https://testnet.binance.vision/api` (no longer valid for auth) |

**How to get testnet API keys:**
1. Go to `https://testnet.binance.vision/`.
2. Click "Log In with GitHub".
3. Authorize the binance-exchange app.
4. Click "Generate HMAC_SHA256 Key" to create credentials.
5. Note: only HMAC keys are available on testnet (no RSA/Ed25519).

**Differences from Production:**
- Simulated market data (NOT real market prices).
- Data is reset approximately every 2 months.
- Test assets are distributed automatically to all accounts.
- Special Unicode test symbols included (e.g. `这是测试币456`).
- Not suitable for backtesting strategies (market conditions differ significantly).
- Optional commission simulation available on testnet only.
- FIX API available (max 10 concurrent connections per account).

---

### Futures Testnet

| Property | Value |
|----------|-------|
| REST base URL | `https://demo-fapi.binance.com` |
| WebSocket | `wss://fstream.binancefuture.com` |
| Old URL (no longer primary) | `https://testnet.binancefuture.com` |

**How to get Futures testnet API keys:**
1. Go to `https://testnet.binancefuture.com`.
2. Log in or register.
3. Scroll down to "API Key" section.
4. Copy displayed API Key and Secret Key.
5. Secret Key shown only once — save it immediately.

**Differences from Production (Futures):**
- Simulated data — NOT real market conditions.
- Limited symbol availability vs. production.
- No real position risk (no actual money at stake).
- Rate limits may differ.
- Testnet may lag behind production in API feature releases.

---

### Demo Trading (Separate from Classic Testnet)

Binance introduced "Demo Trading" which differs from classic Testnet:

| Feature | Testnet | Demo Trading |
|---------|---------|-------------|
| Market data | Simulated | Real-time (actual market) |
| Best for | API integration testing | Strategy testing |
| Spot REST | `testnet.binance.vision` | `demo-api.binance.com` |
| Spot WS | `ws-api.testnet.binance.vision` | `demo-ws-api.binance.com` |

---

## 4. AUTHENTICATION — TECHNICAL DETAILS

### Security Type Levels

| Security Type | Who Uses It | API Key Required | Signature Required |
|--------------|-------------|-----------------|-------------------|
| `NONE` | Public endpoints (market data) | No | No |
| `MARKET_DATA` | Some streaming endpoints | Yes | No |
| `USER_STREAM` | User data stream management | Yes | No |
| `USER_DATA` | Private account queries | Yes | Yes |
| `TRADE` | Order placement / cancellation | Yes | Yes |

### Signature Algorithms (All Supported)

**1. HMAC SHA256 (most common)**

```
signature = hex(HMAC_SHA256(secret_key, query_string + request_body))
```
- Output: hex string.
- Case-insensitive.
- Example for verifying: `HMAC(secretKey, "symbol=BTCUSDT&side=SELL&type=LIMIT&timeInForce=GTC&quantity=1&price=0.2&timestamp=1668481559918")`.

**2. RSA (PKCS#8 only)**

```
signature = base64(RSASSA-PKCS1-v1_5-SHA256(private_key, query_string + request_body))
```
- Must be percent-encoded after base64 encoding.
- Case-sensitive.
- Upload public key to Binance; exchange provides API key.
- PKCS#8 format required.

**3. Ed25519 (Recommended for best performance/security)**

```
signature = base64(Ed25519_sign(private_key, query_string + request_body))
```
- Case-sensitive.
- Fastest signature verification.
- Recommended by Binance for new integrations.

### Request Construction

1. Build parameter string: `param1=value1&param2=value2&...`
2. For GET: put in query string. For POST/PUT/DELETE: can be in query string OR request body (application/x-www-form-urlencoded).
3. Mixed placement allowed: some params in query string, some in body — Binance concatenates them.
4. **As of 2026-01-15:** percent-encode payloads before computing signature. Requests not following this are rejected with `-1022 INVALID_SIGNATURE`.
5. Append `&timestamp=<unix_ms>` (required for all SIGNED endpoints).
6. Optionally append `&recvWindow=<ms>` (default 5000, max 60000).
7. Compute signature over the full string.
8. Append `&signature=<computed_value>` to the query string.
9. Add `X-MBX-APIKEY: <api_key>` header.

### Timestamp Validation

- `timestamp` must be within `recvWindow` ms of server time (default ±5000 ms).
- If timestamp is more than 1 second in the FUTURE, request is rejected.
- Recommendation: use `GET /api/v3/time` to sync with server time.
- Can use microseconds (µs) via `X-MBX-TIME-UNIT: MICROSECOND` header or `timestamp` param in µs.

---

## Sources

- [Binance Spot Request Security](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/request-security)
- [Binance Spot Rate Limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits)
- [Binance Spot WebSocket Rate Limits](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/rate-limits)
- [Binance Futures General Info](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info)
- [Binance Futures Query Rate Limit](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Query-Rate-Limit)
- [Binance API Key Permissions](https://developers.binance.com/docs/wallet/account/api-key-permission)
- [Binance Spot Testnet Changelog](https://developers.binance.com/docs/binance-spot-api-docs/testnet)
- [Binance Testnet Environments Forum](https://dev.binance.vision/t/binance-testnet-environments/99)
- [Binance How to Test on Testnet](https://www.binance.com/en/support/faq/how-to-test-my-functions-on-binance-testnet-ab78f9a1b8824cf0a106b4229c76496d)
- [HMAC Signature Guide](https://www.binance.com/en/academy/articles/hmac-signature-what-it-is-and-how-to-use-it-for-binance-api-security)
