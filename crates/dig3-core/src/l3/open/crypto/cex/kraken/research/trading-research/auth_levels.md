# Kraken Auth Levels, Rate Limits, and Testnet

## Critical Architecture Differences

| Aspect | Spot | Futures |
|--------|------|---------|
| Base URL | `https://api.kraken.com/0` | `https://futures.kraken.com/derivatives/api/v3` |
| Auth method | HMAC-SHA512 (body + path) | HMAC-SHA512 (body + nonce + path) |
| Nonce location | POST body | Header (`Nonce`) |
| Signature header | `API-Sign` | `Authent` |
| Key header | `API-Key` | `APIKey` |
| Request format | Always POST | GET for queries, POST for mutations |

---

## 1. API KEY TYPES

### Spot API Key Permissions (Granular)

Kraken Spot uses **granular per-action permissions** — each API key can be configured with exactly the permissions it needs. This is a unique and important security feature compared to most exchanges.

#### Funds Permissions
| Permission | API Key Setting | Required For |
|-----------|-----------------|-------------|
| Query funds | `Funds permissions — Query` | `Balance`, `TradeBalance`, `ExtendedBalance` |
| Deposit | `Funds permissions — Deposit` | `DepositMethods`, `DepositAddresses`, `DepositStatus` |
| Withdraw | `Funds permissions — Withdraw` | `Withdraw`, `WithdrawInfo`, `WithdrawStatus`, `WalletTransfer` |

#### Orders & Trades Permissions
| Permission | API Key Setting | Required For |
|-----------|-----------------|-------------|
| Query open orders & trades | `Orders and trades — Query open orders & trades` | `OpenOrders`, `QueryOrders`, `OpenPositions`, `TradeBalance` |
| Query closed orders & trades | `Orders and trades — Query closed orders & trades` | `ClosedOrders`, `QueryTrades`, `TradeVolume` |
| Create & modify orders | `Orders and trades — Create & modify orders` | `AddOrder`, `AddOrderBatch`, `EditOrder`, `CancelAllOrdersAfter` |
| Cancel & close orders | `Orders and trades — Cancel & close orders` | `CancelOrder`, `CancelAll`, `CancelAllOrdersAfter` |

#### Data Permissions
| Permission | API Key Setting | Required For |
|-----------|-----------------|-------------|
| Query ledger entries | `Data — Query ledger entries` | `Ledgers`, `QueryLedgers` |
| Export data | `Data — Export data` | `AddExport`, `RetrieveExport`, `RemoveExport` |
| Access WebSocket API | `Data — Access WebSocket API` | `GetWebSocketsToken` |

#### Additional Security Features
- **IP Whitelisting** — restrict API key usage to specific IP addresses
- **2FA** — optional OTP requirement per API key (pass via `otp` field in POST body)
- **Expiry** — optional expiration date for API key

### Spot Key Best Practices
For a trading-only bot:
- Enable: `Query open orders & trades`, `Create & modify orders`, `Cancel & close orders`
- Disable: `Withdraw`, `Deposit` (reduces damage if key is compromised)

---

### Futures API Key Permissions

Futures keys have simpler, coarser permission tiers compared to Spot. The Futures API key is managed separately from the Spot API key (different key pair).

Futures keys can be configured with:
- Read access (balances, positions, order history)
- Trading access (place/cancel orders)
- Full access

Futures API keys do **not** have the same granular permission system as Spot.

---

## 2. AUTHENTICATION MECHANISMS

### Spot REST Authentication

**Algorithm:** HMAC-SHA512

**Required Headers:**
```
API-Key: <public_api_key>
API-Sign: <computed_signature>
Content-Type: application/x-www-form-urlencoded   (or application/json)
```

**Required in Body:**
```
nonce=<uint64>
otp=<TOTP_code>   (optional, only if 2FA enabled)
```

**Signature Generation (5 steps):**

```
1. message = nonce + POST_data_string
   (where POST_data_string is the URL-encoded body, including nonce)

2. sha256_hash = SHA256(message)

3. path = "/0/private/<EndpointName>"
   (e.g. "/0/private/AddOrder")

4. hmac_input = path_bytes + sha256_hash

5. API-Sign = Base64(HMAC-SHA512(Base64Decode(private_key), hmac_input))
```

**Rust pseudocode:**
```rust
let message = format!("{}{}", nonce, post_body);
let sha256 = sha256::digest(message.as_bytes());
let path = "/0/private/AddOrder";
let hmac_input = [path.as_bytes(), &sha256].concat();
let private_key = base64::decode(secret)?;
let signature = hmac_sha512(&private_key, &hmac_input);
let api_sign = base64::encode(signature);
```

**Nonce Rules:**
- Must be a monotonically increasing unsigned 64-bit integer
- Recommend: millisecond-resolution Unix timestamp
- Too many invalid nonces → temporary ban (`EAPI:Invalid nonce`)
- Each API key maintains its own nonce counter

---

### Futures REST Authentication

**Algorithm:** HMAC-SHA512 (different input construction from Spot)

**Required Headers:**
```
APIKey: <public_api_key>
Authent: <computed_signature>
Nonce: <optional_nonce_string>
```

**Signature Generation (5 steps):**

```
1. message = postData + nonce + endpointPath
   (where endpointPath does NOT include the base URL)
   (e.g. endpointPath = "/derivatives/api/v3/sendorder")

2. sha256_hash = SHA256(message)

3. private_key = Base64Decode(api_secret)

4. hmac_result = HMAC-SHA512(private_key, sha256_hash)

5. Authent = Base64(hmac_result)
```

**2024 Update:** The auth was updated February 20, 2024 to require hashing the full, URL-encoded URI component as it appears in the request (particularly relevant for `batchorder` which passes JSON in query params).

**Nonce Format:**
- Optional string header
- Good nonce: millisecond system time as string
- System tolerates brief out-of-order nonces
- Generate new API keys to reset nonce counter if issues arise

---

### WebSocket Authentication (Spot)

For Spot WebSocket private feeds:
1. Call `POST /0/private/GetWebSocketsToken` (REST) to get a token
2. Use token in WebSocket subscription messages
3. Token is valid for 15 minutes

```json
{
  "method": "subscribe",
  "params": {
    "channel": "executions",
    "token": "<ws_token>"
  }
}
```

**Required Permission:** `Data — Access WebSocket API`

---

## 3. RATE LIMITS

### Spot REST — General API Rate Limit (Counter System)

Each API key has an independent **call counter**:

| Tier | Max Counter | Decay Rate |
|------|-------------|------------|
| Starter | 15 | -0.33/second |
| Intermediate | 20 | -0.5/second |
| Pro | 20 | -1/second |

**Per-call cost:**
- Most endpoints: +1
- Ledger/trade history: +2
- `AddOrder`, `CancelOrder`: use the **Trading Engine Rate Limit** (separate system, see below)

**Error responses:**
- `EAPI:Rate limit exceeded` — REST counter exceeded max
- `EService:Throttled: <Unix timestamp>` — excessive concurrent requests (retry after timestamp)

---

### Spot Trading Engine Rate Limit (Matching Engine)

This is a **separate, per-pair counter** that tracks order activity. More nuanced than the general rate limit.

**Per-pair counter thresholds:**

| Tier | Max Counter | Decay Rate (per second) |
|------|-------------|------------------------|
| Starter | 60 | -1.00/s |
| Intermediate | 125 | -2.34/s |
| Pro | 180 | -3.75/s |

**Open order limits per pair:**

| Tier | Max Open Orders |
|------|-----------------|
| Starter | 60 |
| Intermediate | 80 |
| Pro | 225 |

**Transaction cost structure:**

| Action | Fixed Cost | Decay Cost (time-based penalty) |
|--------|-----------|--------------------------------|
| Add Order | +1 | None |
| Amend Order | +1 | +3 if order rested < 5s; +2 if < 10s; +1 if < 15s |
| Edit Order | +1 | +6 if rested < 5s; +4 if < 10s; +2 if < 45s; +1 if < 90s |
| Cancel Order | 0 | +8 if rested < 5s; +6 if < 10s; +5 if < 15s; ... +1 if < 300s |
| Batch Add | +(n/2) rounded up | None |
| Batch Cancel | 0 | +(8*n) to +(1*n) depending on resting time |

**Key insight:** The counter cost for **cancel** includes a time-decay penalty based on how long the order has been resting. Cancelling an order immediately after placing it (< 5s) incurs a +8 penalty. This discourages high-frequency cancel/replace strategies.

**Error responses:**
- `EOrder:Rate limit exceeded` — trading engine counter exceeded
- `EOrder:Orders limit exceeded` — open order count exceeded

**Monitoring:**
- Counter values are available in the WebSocket `openOrders` feed (v1) or `executions` feed (v2)
- Kraken provides a rate counter calculator in support documentation

---

### Futures Rate Limits

Futures uses its own independent rate limiting system. Specific counter values are not publicly documented in the same detail as Spot, but general limits apply:

- Separate rate limit per endpoint category
- Higher-tier accounts (verified) have higher limits
- No combined "counter" system — standard HTTP 429 response when limit hit
- Demo/testnet environment has relaxed limits

---

## 4. TESTNET / SANDBOX

### Spot Testnet

**No public sandbox for Spot REST.** Spot REST does not have an officially documented public testnet.

Alternative approaches for Spot testing:
- Use `validate=true` parameter in `AddOrder` to test order parameters without submission
- Trade tiny amounts on live with very small position sizes
- Kraken may provide UAT environments to institutional clients (contact support)

### Futures Sandbox (Demo Environment)

**Full sandbox available for Futures.**

| Environment | URL |
|-------------|-----|
| Futures Production | `https://futures.kraken.com/derivatives/api/v3` |
| Futures Demo/Testnet | `https://demo-futures.kraken.com/derivatives/api/v3` |

**Demo environment details:**
- Requires separate API key credentials (different from production)
- Create demo account at `demo-futures.kraken.com`
- Funds in demo account are fictional/paper
- Full API functionality matches production
- Same WebSocket endpoint: `wss://demo-futures.kraken.com/ws/v1`
- When using SDK: set `DEMO=True` or equivalent parameter

**Demo WebSocket:**
```
wss://demo-futures.kraken.com/ws/v1
```

---

## 5. ERROR CODES

### Spot Error Format

Errors returned in the `error` array as strings:

```json
{
  "error": ["EAPI:Rate limit exceeded"],
  "result": {}
}
```

#### Common Spot Error Codes

| Error | Description |
|-------|-------------|
| `EAPI:Invalid key` | API key not found or invalid |
| `EAPI:Invalid signature` | Signature verification failed |
| `EAPI:Invalid nonce` | Nonce is not monotonically increasing |
| `EAPI:Rate limit exceeded` | General rate counter exceeded |
| `EAPI:Feature disabled` | Feature not available |
| `EAPI:Permission denied` | API key lacks required permission |
| `EService:Unavailable` | Exchange temporarily unavailable |
| `EService:Busy` | Exchange busy — retry later |
| `EService:Throttled: <ts>` | Too many concurrent requests |
| `EOrder:Rate limit exceeded` | Trading engine counter exceeded |
| `EOrder:Orders limit exceeded` | Open order count exceeded |
| `EOrder:Invalid order` | Order parameters invalid |
| `EOrder:Cannot open position` | Margin/leverage error |
| `EOrder:Insufficient funds` | Not enough balance |
| `EOrder:Order minimum not met` | Order too small |
| `EFunding:Unknown withdraw key` | Invalid withdrawal key |
| `EFunding:Insufficient funds` | Insufficient balance for withdrawal |

### Futures Error Format

Futures errors appear in the `result` field:

```json
{
  "result": "error",
  "error": "apiKeyRequired"
}
```

Or for order-level errors, in `sendStatus.status`:

```json
{
  "result": "success",
  "sendStatus": {
    "status": "insufficientAvailableFunds"
  }
}
```

#### Common Futures Status Values

| Status | Description |
|--------|-------------|
| `placed` | Order successfully placed |
| `insufficientAvailableFunds` | Not enough margin/funds |
| `invalidOrderType` | Unknown orderType value |
| `tooManySmallOrders` | Order size too small |
| `maxPositionViolation` | Would exceed position limits |
| `marketSuspended` | Market is halted |
| `cancelled` | Order was cancelled |
| `apiKeyRequired` | No API key provided |
| `authenticationError` | Auth signature invalid |
| `notFound` | Order ID not found |

---

## 6. V5 TRAIT DESIGN IMPLICATIONS

### Key Observations for V5 Connector

1. **Dual system:** Must implement two separate connector structs — `KrakenSpotConnector` and `KrakenFuturesConnector` — they share no auth or endpoint logic.

2. **EditOrder on Spot:** Supported natively via `POST /0/private/EditOrder`, but Kraken recommends the newer WebSocket v2 `AmendOrder`. REST EditOrder does NOT preserve queue position.

3. **EditOrder on Futures:** Supported via `POST /derivatives/api/v3/editorder` by `orderId` or `cliOrdId`.

4. **No set_leverage for Spot:** Leverage is per-order. Trait must accommodate this — `set_leverage()` is a no-op or returns `UnsupportedOperation` for Spot.

5. **Futures leverage:** `PUT /derivatives/api/v3/leveragepreferences` — per-symbol setting, separate from order placement.

6. **Dead Man's Switch:** `CancelAllOrdersAfter` is unique to Kraken. Expose as a dedicated trait method or utility.

7. **Batch orders:** Both Spot (15 orders) and Futures support batching. Good for V5 batch trait.

8. **Conditional close (OTO):** Spot-only feature via `close[]` params. Maps to a combined "place with TP/SL" operation.

9. **Nonce:** Must be globally monotonically increasing per API key. Use `AtomicU64` initialized from current time in ms.

10. **Permissions in errors:** If API key lacks a permission, error is `EAPI:Permission denied` — map to `ExchangeError::AuthError`.

---

## Sources

- [Spot REST Authentication | Kraken API Center](https://docs.kraken.com/api/docs/guides/spot-rest-auth/)
- [Futures REST Authentication | Kraken API Center](https://docs.kraken.com/api/docs/guides/futures-rest/)
- [Spot REST Rate Limits | Kraken API Center](https://docs.kraken.com/api/docs/guides/spot-rest-ratelimits/)
- [Spot Trading Rate Limits | Kraken API Center](https://docs.kraken.com/api/docs/guides/spot-ratelimits/)
- [API Testing Environment (Futures Demo) | Kraken Support](https://support.kraken.com/articles/360024809011-api-testing-environment-derivatives)
- [How to Create a Spot API Key | Kraken Support](https://support.kraken.com/articles/360000919966-how-to-create-an-api-key)
- [Advanced API FAQ | Kraken Support](https://support.kraken.com/articles/advanced-api-faq)
- [What Are the API Rate Limits? | Kraken Support](https://support.kraken.com/articles/206548367-what-are-the-api-rate-limits-)
- [Kraken APIs Global Intro | Kraken API Center](https://docs.kraken.com/api/docs/guides/global-intro/)
- [Spot Errors | Kraken API Center](https://docs.kraken.com/api/docs/guides/spot-errors/)
