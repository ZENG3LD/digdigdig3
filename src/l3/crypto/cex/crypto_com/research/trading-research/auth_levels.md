# Crypto.com Exchange API v1 — Authentication and Rate Limits Research

Source: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
Research Date: 2026-03-11

---

## Authentication Method

### Algorithm: HMAC-SHA256

Crypto.com Exchange API v1 uses **HMAC-SHA256** for request authentication. The Secret Key is NEVER sent in plain text.

### Signature Construction

#### Step 1: Sort Parameters Alphabetically

If the request has a `params` object, sort all parameter keys in ascending alphabetical order.

#### Step 2: Build Parameter String

Concatenate each key with its value, with NO spaces or delimiters:
```
key1value1key2value2key3value3
```

Example: for params `{ "instrument_name": "BTC_USDT", "side": "BUY", "type": "LIMIT" }` sorted and concatenated:
```
instrument_nameBTC_USDTsideBUYtypeLIMIT
```

#### Step 3: Build the Signature Payload

Concatenate in this exact order with NO delimiters:
```
{method}{id}{api_key}{parameter_string}{nonce}
```

Example:
```
private/create-order11234MY_API_KEYinstrument_nameBTC_USDTsideBUYtypeLIMIT1610905028000
```

#### Step 4: Hash and Encode

Apply HMAC-SHA256 using the API Secret as the cryptographic key, then encode the output as a **lowercase hex string**.

```
sig = hex( HMAC-SHA256( api_secret, payload ) )
```

### Fields in Request Body

| Field | Location | Required | Description |
|-------|----------|----------|-------------|
| `api_key` | JSON body `params` or top-level | YES | Your API Key |
| `sig` | JSON body top-level | YES | Computed HMAC-SHA256 hex signature |
| `nonce` | JSON body top-level | YES | Current timestamp in milliseconds since Unix epoch |
| `id` | JSON body top-level | YES | Request ID (any integer, used for matching responses) |
| `method` | JSON body top-level | YES | Endpoint method string, e.g. `private/create-order` |

**There are NO HTTP header-based auth fields** — all authentication is in the JSON body.

### Example Authenticated REST Request

```json
{
  "id": 11,
  "method": "private/create-order",
  "api_key": "YOUR_API_KEY",
  "params": {
    "instrument_name": "BTC_USDT",
    "side": "BUY",
    "type": "MARKET",
    "notional": "100"
  },
  "nonce": 1610905028000,
  "sig": "abc123...hex_encoded_hmac_sha256"
}
```

### Nonce Validation

- `nonce` must be the current time in milliseconds since Unix epoch
- Server rejects nonces that differ from server time by more than **60 seconds**
- Error code returned on invalid nonce: **40102**

### WebSocket Authentication

For the User API WebSocket (`wss://stream.crypto.com/exchange/v1/user`), authentication is done once per session:

```json
{
  "id": 1,
  "method": "public/auth",
  "api_key": "YOUR_API_KEY",
  "nonce": 1610905028000,
  "sig": "COMPUTED_SIGNATURE"
}
```

After `public/auth` is sent and confirmed, subsequent messages in the same WebSocket session do NOT need to include `api_key` or `sig`.

**Important:** Wait at least **1 second** after establishing the WebSocket connection before sending the auth request, to avoid rate-limit errors (limits are pro-rated from the connection open time).

---

## Permission Levels

### API Key Permissions

Permissions are set via the Exchange website UI at **User Center → API**.

| Permission | Default | Description |
|------------|---------|-------------|
| **Read (Can Read)** | YES — default | View balances, positions, orders, market data |
| **Trade** | Optional | Place, amend, cancel orders, close positions |
| **Withdraw** | Optional | Initiate withdrawals |

**Notes:**
- Default when creating a new API key is Read-only.
- Permissions are additive — you add Trade or Withdraw on top of Read.
- There is NO API endpoint to query your own API key's permission level at runtime.
- Permission management is exclusively through the Exchange website UI.

### Per-Key Permissions

YES — each API key can have different permissions. It is possible to create separate keys for:
- A read-only monitoring key
- A trading-only key (no withdraw)
- A full-access key

---

## Rate Limits

### REST API Rate Limits (per API key)

All limits are enforced per individual API key.

| Endpoint(s) | Limit |
|-------------|-------|
| `private/create-order` | 15 req / 100ms |
| `private/cancel-order` | 15 req / 100ms |
| `private/cancel-all-orders` | 15 req / 100ms |
| `private/get-order-detail` | 30 req / 100ms |
| `private/get-order-history` | 1 req / second |
| `private/get-trades` | 1 req / second |
| All other authenticated private calls | 3 req / 100ms |

### REST API Rate Limits (per IP address — Public Endpoints)

| Endpoint(s) | Limit |
|-------------|-------|
| `public/get-book` | 100 req / second |
| `public/get-ticker` | 100 req / second |
| `public/get-trades` | 100 req / second |
| `public/get-valuations` | 100 req / second |
| `public/get-candlestick` | 100 req / second |
| `public/get-insurance` | 100 req / second |
| Public staking endpoints | 50 req / second |
| Private staking endpoints | 50 req / second |

### WebSocket Rate Limits

| Connection | Limit |
|------------|-------|
| User API (`/user`) | 150 req / second |
| Market Data (`/market`) | 100 req / second |

**WebSocket Timing Note:** Rate limits are pro-rated based on the calendar second the WebSocket connection was opened. Always wait 1 second after connecting before sending requests.

### Rate Limit Differences: Public vs Private

- **Public endpoints:** Limited per IP address; higher throughput (100 req/s for market data).
- **Private endpoints:** Limited per API key; stricter limits for order operations (15/100ms ≈ 150/second for create/cancel).
- **No difference in rate limits based on permission level** — a read-only key and a trading key have the same rate limits for the same endpoint type.

### Order-Specific Rate Limits

Order creation and cancellation: **15 requests per 100ms per API key**

This translates to approximately 150 orders per second theoretical maximum per key, but in practice the 100ms window means bursting is limited.

---

## IP Restrictions

### IP Whitelisting

**Available:** YES — IP whitelisting is supported per API key.

- Configured at API key creation time via the Exchange website UI.
- If specified, the API key can ONLY be used from whitelisted IP addresses.
- Up to how many IPs can be whitelisted: NOT DOCUMENTED (exact limit not in retrieved docs).
- IP whitelist cannot be modified after key creation — you must create a new key to change the whitelist.
- If no IPs are specified, the key works from any IP address.

**Quote from documentation:**
> "If specified, the API can only be used from the whitelisted IP addresses."

---

## Error Codes Related to Auth

| Code | Meaning |
|------|---------|
| `40102` | Invalid nonce (differs from server time by more than 60 seconds) |
| Other 4xxxx codes | Various auth/permission errors (complete list NOT FULLY DOCUMENTED in retrieved content) |

---

## Security Best Practices (from official docs)

1. NEVER include the Secret Key in plain text in requests.
2. The server independently calculates the HMAC signature and compares — if hashes match, request is authenticated.
3. Use IP whitelisting for production trading keys.
4. Create separate API keys with minimal required permissions (principle of least privilege).

---

## Summary for V5 Trait Implementation

| Concern | Value |
|---------|-------|
| Signing algorithm | HMAC-SHA256 |
| Signature encoding | Lowercase hex string |
| Auth transmission | JSON body fields (`api_key`, `sig`, `nonce`) |
| HTTP headers for auth | NONE — all in body |
| Nonce format | Unix epoch milliseconds |
| Nonce window | ±60 seconds |
| WebSocket auth | One-time `public/auth` message per session |
| Permission scopes | Read, Trade, Withdraw (set via UI) |
| IP restriction | Optional whitelist at key creation |
| Order rate limit | 15/100ms per key |
| Query rate limit | 3–30/100ms depending on endpoint |

---

## Sources

- [Crypto.com Exchange API v1 Official Docs](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Crypto.com Exchange Institutional API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index-insto-8556ea5c-4dbb-44d4-beb0-20a4d31f63a7.html)
- [Crypto.com API Help Center](https://help.crypto.com/en/articles/3511424-api)
