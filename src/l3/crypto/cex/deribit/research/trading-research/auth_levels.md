# Deribit Authentication and Rate Limits Specification

Source: https://docs.deribit.com/articles/authentication
API Version: v2

---

## Authentication Method

Deribit uses **JSON-RPC 2.0** over WebSocket or HTTP REST. All private methods require authentication. Authentication is OAuth2-style: acquire an access token via `public/auth`, then include it in subsequent requests.

### Base Endpoints

| Environment | WebSocket | HTTP |
|---|---|---|
| Production | `wss://www.deribit.com/ws/api/v2` | `https://www.deribit.com/api/v2/{method}` |
| Testnet | `wss://test.deribit.com/ws/api/v2` | `https://test.deribit.com/api/v2/{method}` |

Production and testnet have **separate, independent** account registrations and rate-limit pools.

---

## Grant Types

All authentication uses `public/auth` endpoint.

### 1. Client Credentials

Standard OAuth 2.0 — transmits the secret directly.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "client_credentials",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET"
  }
}
```

Best for: server-to-server applications where secret transmission is acceptable.

---

### 2. Client Signature (HMAC or Asymmetric)

Cryptographic authentication — secret is never transmitted. This is the more secure method.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "client_signature",
    "client_id": "YOUR_CLIENT_ID",
    "timestamp": 1576074319000,
    "nonce": "1iqt2wls",
    "data": "",
    "signature": "56590594f97921b09b18f166befe0d1319b198bbcdad7ca73382de2f88fe9aa1"
  }
}
```

---

### 3. Refresh Token

Renew access token without re-supplying credentials.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "refresh_token",
    "refresh_token": "YOUR_REFRESH_TOKEN"
  }
}
```

---

### 4. Basic Auth (HTTP only)

HTTP requests can use standard HTTP Basic Auth:
```
Authorization: Basic BASE64(ClientId:ClientSecret)
```

---

## Signing Algorithm

### HMAC-SHA256 (Symmetric Keys)

Used with standard API keys (client_id + client_secret).

#### WebSocket Signature

```
StringToSign = Timestamp + "\n" + Nonce + "\n" + Data
Signature    = LOWERCASE_HEX( HMAC-SHA256( ClientSecret, StringToSign ) )
```

- `Timestamp` — current time in milliseconds since Unix epoch
- `Nonce` — unique per-request random string (8 characters recommended)
- `Data` — optional additional data; if omitted, treat as empty string; string still ends with `\n` after Nonce
- Key — Client Secret encoded as UTF-8
- Output — lowercase hexadecimal string

**Example:**
```
ClientId    = AMANDA
ClientSecret = AMANDASECRECT
Timestamp   = 1576074319000
Nonce       = 1iqt2wls
Data        = ""

StringToSign = "1576074319000\n1iqt2wls\n"
Signature    = "56590594f97921b09b18f166befe0d1319b198bbcdad7ca73382de2f88fe9aa1"
```

#### HTTP REST Signature

For REST requests, `Data` includes the HTTP method, URI, and body:

```
RequestData  = UPPERCASE(HTTP_METHOD) + "\n" + URI + "\n" + RequestBody + "\n"
StringToSign = Timestamp + "\n" + Nonce + "\n" + RequestData
Signature    = LOWERCASE_HEX( HMAC-SHA256( ClientSecret, StringToSign ) )
```

Example URI: `/api/v2/private/buy`

---

### Asymmetric Keys: Ed25519 (Recommended)

Modern elliptic curve cryptography. Private key never leaves client.

- 256-bit keys
- Faster than RSA
- No padding or hashing layer needed
- Signatures encoded as **URL-safe Base64** with `=` padding stripped

**Signature message:** `timestamp\nnonce\ndata` (same structure as HMAC)

---

### Asymmetric Keys: RSA

Traditional asymmetric encryption for legacy compatibility.

- Minimum 2048-bit key size required
- Uses **PKCS1v15 padding** with **SHA-256 hashing**
- More complex implementation than Ed25519
- Signatures encoded as **URL-safe Base64** with `=` padding stripped

---

## Token Placement in Requests

### WebSocket

Include `access_token` in the JSON-RPC params field for each private request:

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "private/buy",
  "params": {
    "access_token": "TOKEN_HERE",
    "instrument_name": "BTC-PERPETUAL",
    "amount": 10,
    "type": "limit",
    "price": 50000
  }
}
```

Or authenticate once per WebSocket session via `public/auth` — the token is then associated with the connection.

### HTTP REST (Bearer Token)

```
Authorization: Bearer <access_token>
```

### HTTP REST (Signature-based, no token)

```
Authorization: deri-hmac-sha256 id=ClientId,ts=Timestamp,nonce=Nonce,sig=Signature
```

---

## Token Lifecycle

| Property | Value |
|---|---|
| `access_token` | JWT token for API calls |
| `expires_in` | 31,536,000 seconds (1 year, as shown in documentation examples) |
| `refresh_token` | Used to get new access tokens without re-authentication |
| `scope` | Space-separated list of granted scopes (e.g., `"account:read trade:read"`) |
| `token_type` | `"bearer"` |

Timestamp validity window: **60 seconds** from the timestamp in the signature.

---

## Advanced Session Management

### Fork Token

**Endpoint:** `public/fork_token`

Creates a derived session token for multi-session use. Allows concurrent independent sessions from one authentication.

### Exchange Token

**Endpoint:** `public/exchange_token`

Switches session context to a subaccount. Used for trading on behalf of subaccounts with a main account token.

---

## Permission Scopes

Scopes are assigned to API keys and define the maximum permission level of tokens created with that key. Requested scope cannot exceed the key's default scope.

### Complete Scope List

| Scope | Access Level | Description |
|---|---|---|
| `account:read` | Read | View account info, summaries, transaction history, access log |
| `account:read_write` | Read + Write | Create/modify/delete API keys, manage account settings (requires TFA for key operations) |
| `trade:read` | Read | View open orders, order history, trade history, positions |
| `trade:read_write` | Read + Write | Place orders, cancel orders, edit orders, close positions |
| `wallet:read` | Read | View balances, deposit addresses, transfer history, withdrawal history |
| `wallet:read_write` | Read + Write | Create deposit addresses, submit withdrawals (requires 2FA), submit transfers |
| `block_trade:read` | Read | View block trade history |
| `block_trade:read_write` | Read + Write | Execute block trades, verify block trade proposals |
| `block_rfq:read` | Read | Listen to and retrieve open Block RFQs |
| `block_rfq:read_write` | Read + Write | Create, respond to, and manage Block RFQs |
| `custody:read` | Read | Third-party custodian access (enabled by client) |

### Special Scopes

| Scope | Description |
|---|---|
| `connection` | Session connection scope |
| `session:name` | Named session identifier |
| `mainaccount` | Scope for main account operations when using subaccount token |

### Scope Inheritance Rules

- Requested scope **cannot exceed** the API key's default scope
- Example: key with `account:read` default → requesting `account:read_write` returns token with `account:read` only
- Creating, editing, and removing API keys requires `account:read_write` scope
- The resulting token scope is the **intersection** of the key's default scope and the requested scope

---

## API Key Types

| Type | Description |
|---|---|
| HMAC (symmetric) | Standard client_id + client_secret pair |
| Ed25519 (asymmetric) | Public-private key pair; secret never transmitted |
| RSA (asymmetric) | RSA 2048-bit+; for legacy infrastructure |

For asymmetric keys: public key is registered with Deribit; private key stays on client.
Authentication uses `grant_type: client_signature` for both HMAC and asymmetric.

---

## Rate Limits

### Credit System

Deribit uses a **credit-based rate limiting** system:

| Parameter | Value |
|---|---|
| Refill rate | **10 credits per millisecond** (= 10,000 credits per second) |
| Pool maximum | Varies by account tier |
| Depletion behavior | Request fails with error code `10028` (too_many_requests) |
| Session termination | Excessive rate limit errors may result in connection termination |

Each API method consumes a different number of credits. More resource-intensive methods (e.g., order book queries, matching engine operations) cost more credits than lightweight reads.

### Endpoint Categories

| Category | Examples | Credit Cost |
|---|---|---|
| Matching engine | private/buy, private/sell, private/cancel, private/edit | Higher cost |
| Non-matching reads | private/get_order_state, private/get_positions, public/ticker | Lower cost |
| Market data subscriptions | WebSocket subscriptions | Preferred — reduces REST credit consumption |
| Transaction log | private/get_transaction_log | Special: max 1 request/second |

### Public vs Private Limits

- **Public** endpoints (market data): accessible without authentication; IP-level rate limiting applies
- **Private** endpoints: require authentication; per-subaccount limits
- Authenticated connections have higher rate limits than unauthenticated
- Rate limits are **per subaccount** (subaccounts have their own independent limit pools)
- Production and testnet pools are **separate and do not share** limits

### Order-to-Volume Ratio (OTV) Policy

- OTV is monitored per product group
- Ratios exceeding **10,000 BTC** or **1,000 ETH** order-to-volume are considered high
- High OTV may result in warnings or restrictions from Deribit compliance
- Applies separately to futures and options product groups

### Connection Limits

| Limit | Value |
|---|---|
| Max WebSocket connections per IP | 32 |
| Max sessions per API key | 16 |
| HTTP connection expiry | 15 minutes (idle) |

### Error Codes for Rate Limiting

| Code | Meaning |
|---|---|
| `10028` | `too_many_requests` — credits exhausted; request rejected |

**Note:** Excessive error responses can result in IP banning. Monitor error rates actively.

---

## IP Restrictions

- IP whitelisting is **per API key**, not global
- When configured, requests from non-whitelisted IPs are rejected
- Configurable via web UI or `private/create_api_key` / key management endpoints
- No documented maximum number of whitelisted IPs

---

## Best Practices

1. **Prefer WebSocket over HTTP** — real-time subscriptions, cancel-on-disconnect, lower overhead
2. **Use subscriptions for market data** — avoids REST credit consumption for polling
3. **Monitor error rate** — excessive errors (including rate limit errors) can trigger IP banning
4. **Use `cancel_on_disconnect`** — WebSocket feature that auto-cancels orders on connection loss
5. **Refresh tokens** before expiry using `grant_type: refresh_token`
6. **Use Ed25519 asymmetric keys** for maximum security (private key never transmitted)
7. **Separate subaccounts** for separate strategies — each has independent rate limit pools

---

## Sources

- [Deribit API Authentication Guide](https://docs.deribit.com/articles/authentication)
- [Deribit Asymmetric API Keys](https://docs.deribit.com/articles/asymmetric-api-keys)
- [Deribit API Quickstart Guide](https://docs.deribit.com/articles/deribit-quickstart)
- [Deribit JSON-RPC Overview](https://docs.deribit.com/articles/json-rpc-overview)
- [Deribit Access Scopes](https://docs.deribit.com/articles/access-scope)
- [Deribit Creating API Keys](https://docs.deribit.com/articles/creating-api-key)
- [API Authentication Guide - Support](https://support.deribit.com/hc/en-us/articles/29748629634205-API-Authentication-Guide)
- [Rate Limits - Support](https://support.deribit.com/hc/en-us/articles/25944617523357-Rate-Limits)
- [Deribit Rate Limits KB](https://www.deribit.com/kb/deribit-rate-limits)
