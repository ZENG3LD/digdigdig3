# Deribit Authentication

Complete authentication specification for Deribit API.

## Overview

Deribit uses **OAuth 2.0 style authentication** for all private API requests. You must obtain an access token and refresh token using API key credentials before calling private endpoints.

## API Key Creation

1. Create API keys via the web interface: Account > API tab
2. Set appropriate scopes/permissions for the key
3. Store `client_id` (API Key) and `client_secret` (API Secret) securely

**Important**: Test and production environments require separate accounts and API keys.

## Authentication Endpoint

**Method**: `public/auth`
**Scope**: Public
**Description**: Authenticates and returns access token and refresh token

### Grant Types

Deribit supports three grant types:

#### 1. Client Credentials (Simple)
Direct authentication using API key and secret.

**Parameters**:
- `grant_type`: `"client_credentials"`
- `client_id`: string - Your API Key
- `client_secret`: string - Your API Secret
- `scope`: string (optional) - Requested scope

**Example Request**:
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

#### 2. Client Signature (Secure)
Uses cryptographic signature instead of sending the secret directly.

**How it works**:
1. Generate a timestamp (milliseconds since epoch)
2. Generate a random nonce (string)
3. Create signature string: `{timestamp}\n{nonce}\n{data}` (data is optional)
4. Compute HMAC-SHA256 signature using your Client Secret as the key
5. Encode signature as hexadecimal string

**Parameters**:
- `grant_type`: `"client_signature"`
- `client_id`: string - Your API Key
- `timestamp`: integer - Current timestamp in milliseconds
- `nonce`: string - Random unique string
- `signature`: string - HMAC-SHA256 signature (hex encoded)
- `data`: string (optional) - Additional data to include in signature

**Signature Generation**:
```
message = "{timestamp}\n{nonce}\n{data}"
signature = HMAC_SHA256(client_secret, message)
signature_hex = hex_encode(signature)
```

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "client_signature",
    "client_id": "YOUR_CLIENT_ID",
    "timestamp": 1609459200000,
    "nonce": "abcd1234",
    "signature": "a1b2c3d4e5f6...",
    "data": ""
  }
}
```

#### 3. Refresh Token
Extends session without re-sending credentials.

**Parameters**:
- `grant_type`: `"refresh_token"`
- `refresh_token`: string - Previously received refresh token

**Example Request**:
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

## Authentication Response

Successful authentication returns:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "token_type": "bearer",
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "expires_in": 900,
    "scope": "trade:read trade:write"
  }
}
```

**Response Fields**:
- `access_token`: string - JWT token for authorizing API calls
- `token_type`: string - Always `"bearer"`
- `refresh_token`: string - Token for refreshing access
- `expires_in`: integer - Token lifetime in seconds (typically 900 = 15 minutes)
- `scope`: string - Granted scopes/permissions

---

## Using Access Tokens

### HTTP Requests

Add the token to the `Authorization` header:

```http
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc...
```

**Example HTTP Request**:
```http
POST /api/v2/private/buy HTTP/1.1
Host: www.deribit.com
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc...
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 5275,
  "method": "private/buy",
  "params": {
    "instrument_name": "BTC-PERPETUAL",
    "amount": 100
  }
}
```

### WebSocket Requests

Include `access_token` in the request params or authenticate the connection first:

**Option 1: Authenticate Connection**
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

After successful auth, all subsequent messages on that WebSocket connection are authenticated.

**Option 2: Per-Request Token** (if not authenticated at connection level)
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "private/buy",
  "params": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "instrument_name": "BTC-PERPETUAL",
    "amount": 100
  }
}
```

---

## Token Lifecycle

### Token Expiration
- Access tokens expire after `expires_in` seconds (typically 15 minutes)
- Refresh tokens have longer lifetime
- Monitor token expiration and refresh proactively

### Token Refresh Flow

1. Before access token expires, call `public/auth` with `grant_type=refresh_token`
2. Receive new access token and new refresh token
3. Replace old tokens with new ones
4. Continue using new access token

**Best Practice**: Refresh tokens when ~80% of `expires_in` has elapsed (e.g., after 12 minutes for 15-minute tokens).

### Token Scope

Tokens can have different scopes:

#### Connection Scope (Default)
- Token valid only for specific connection
- When connection closes, token becomes invalid
- Suitable for single-session applications

#### Session Scope
- Token persists across connections
- Use `session:name` parameter in auth request
- Suitable for applications that reconnect frequently

**Example with Session Scope**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "client_credentials",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET",
    "scope": "session:my_trading_bot trade:read trade:write"
  }
}
```

---

## API Key Scopes/Permissions

When creating API keys, you must specify which operations the key can perform:

### Available Scopes

- **`trade:read`**: Read trading data (orders, positions)
- **`trade:write`**: Place and cancel orders
- **`wallet:read`**: Read wallet information
- **`wallet:write`**: Create withdrawals, transfers
- **`wallet:read_write`**: Combined wallet read and write
- **`account:read`**: Read account information
- **`account:write`**: Modify account settings
- **`block_trade`**: Block trading operations

**Example**: For a trading bot, request scopes: `trade:read trade:write account:read`

---

## Security Best Practices

### 1. Never Hardcode Credentials
- Store API keys in environment variables or secure vaults
- Never commit credentials to version control
- Use different keys for development and production

### 2. Use Client Signature for Production
- Avoids sending client secret over the network
- More secure than client credentials
- Requires proper timestamp synchronization

### 3. Implement Token Refresh
- Refresh tokens before expiration
- Handle refresh failures gracefully
- Re-authenticate if refresh fails

### 4. Secure Token Storage
- Store tokens in memory only (not on disk for most applications)
- Clear tokens on application shutdown
- Use secure storage if persistence is required

### 5. Handle Authentication Errors

**Common Error Codes**:
- **13004**: `invalid_credentials` - Wrong client ID or secret
- **13668**: `security_key_authorization_error` - 2FA/Security key issue
  - `data.reason: "tfa_code_not_matched"` - Invalid 2FA code
  - `data.reason: "used_tfa_code"` - 2FA code already used
- **13001**: `authorization_required` - Missing or invalid token
- **13002**: `insufficient_scope` - Token lacks required permissions

**Error Handling**:
```rust
match error.code {
    13004 => {
        // Invalid credentials - check API key and secret
        // Do NOT retry automatically (security risk)
    },
    13001 | 13002 => {
        // Token expired or insufficient scope
        // Attempt to refresh token or re-authenticate
    },
    13668 => {
        // 2FA required or failed
        // Prompt user for 2FA code or check security settings
    },
    _ => {
        // Other errors
    }
}
```

---

## Two-Factor Authentication (2FA)

If 2FA is enabled on the account:

1. API requests may require 2FA confirmation
2. Include `tfa` parameter in requests:
   ```json
   {
     "method": "private/withdraw",
     "params": {
       "currency": "BTC",
       "address": "...",
       "amount": 0.5,
       "tfa": "123456"
     }
   }
   ```
3. Invalid or used 2FA codes return error 13668

---

## Implementation Checklist

For V5 connector implementation:

- [ ] Implement `public/auth` with client credentials grant
- [ ] Implement client signature grant (recommended for production)
- [ ] Implement refresh token flow
- [ ] Store access token and refresh token
- [ ] Add `Authorization: Bearer {token}` header to all private requests
- [ ] Monitor token expiration and refresh proactively
- [ ] Handle authentication errors (13004, 13001, 13002, 13668)
- [ ] Support connection-scope and session-scope tokens
- [ ] Implement secure credential storage (environment variables)
- [ ] Add retry logic for transient authentication failures
- [ ] Log authentication events (without logging secrets)

---

## Example Implementation Flow (Rust)

```rust
// 1. Initial authentication
let auth_response = authenticate(client_id, client_secret).await?;
let access_token = auth_response.access_token;
let refresh_token = auth_response.refresh_token;
let expires_at = Instant::now() + Duration::from_secs(auth_response.expires_in);

// 2. Use access token for requests
let response = make_authenticated_request(
    "private/get_account_summary",
    params,
    &access_token
).await?;

// 3. Refresh before expiration
if Instant::now() > expires_at - Duration::from_secs(180) { // 3 min buffer
    let refresh_response = refresh_access_token(&refresh_token).await?;
    access_token = refresh_response.access_token;
    refresh_token = refresh_response.refresh_token;
    expires_at = Instant::now() + Duration::from_secs(refresh_response.expires_in);
}
```

---

## References

- Deribit API Documentation: https://docs.deribit.com/
- OAuth 2.0 Specification: https://oauth.net/2/
- API Authentication Guide: https://support.deribit.com/hc/en-us/articles/29748629634205-API-Authentication-Guide
