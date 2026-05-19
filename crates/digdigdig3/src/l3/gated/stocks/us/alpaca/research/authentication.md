# Alpaca - Authentication

## Public Endpoints

- Public endpoints exist: **Partial** (some crypto historical data doesn't require auth)
- Require authentication: **Yes** (most endpoints require API key)
- Rate limits without auth: **Not applicable** (almost all require auth)

**Exception:** Historical crypto data does NOT require authentication (but still has rate limits)

## API Key

### Required For

- All endpoints: **Yes** (except historical crypto)
- Paid tier only: **No** (free tier has API keys)
- Rate limit increase: **Yes** (paid tier gets unlimited API calls)
- Specific endpoints: **All Trading API, Market Data API (except crypto historical)**

### How to Obtain

**Paper-Only Account (Free, Global):**
1. Sign up: https://app.alpaca.markets/signup
2. Email signup only (no verification needed)
3. Paper trading API keys generated immediately
4. Access: Dashboard → API Keys

**Live Trading Account (US Residents):**
1. Sign up: https://app.alpaca.markets/signup
2. Complete KYC (Know Your Customer) verification
3. Fund account (optional, can get keys before funding)
4. Access: Dashboard → API Keys
5. Separate keys for paper and live trading

**Market Data Subscription:**
- Free tier: Included with any account (paper or live)
- Paid tier (Algo Trader Plus $99/mo): Upgrade in dashboard
- Same API keys work for both trading and market data

### API Key Format

**Two components:**
- **API Key ID** (public): Alphanumeric string, ~20 characters
- **API Secret Key** (private): Alphanumeric string, ~40 characters

**Example format:**
```
APCA-API-KEY-ID: PKXYZ123ABC456DEF789
APCA-API-SECRET-KEY: abcdef1234567890abcdef1234567890abcdef12
```

### Header Format (REST API)

**Trading API:**
```http
APCA-API-KEY-ID: your_key_id
APCA-API-SECRET-KEY: your_secret_key
```

**Broker API (Basic Auth):**
```http
Authorization: Basic base64encode(key_id:secret_key)
```

**Example cURL:**
```bash
curl -H "APCA-API-KEY-ID: PKXYZ123ABC456DEF789" \
     -H "APCA-API-SECRET-KEY: abcdef1234567890..." \
     https://api.alpaca.markets/v2/account
```

### Query Parameter Format

**NOT supported** - Alpaca does NOT accept API keys as query parameters.
Only header authentication is supported.

### Bearer Token Format

**OAuth2 only** - When using OAuth2, use Bearer token:
```http
Authorization: Bearer your_access_token
```

### Multiple Keys

- Multiple keys allowed: **Yes** (can generate multiple key pairs)
- Rate limits per key: **Yes** (each key has independent rate limit)
- Use cases for multiple keys:
  - Separate keys for different strategies/bots
  - Separate paper/live keys (automatically provided)
  - Testing vs production environments
  - Different applications using same account

**Note:** Paper and Live accounts have **separate API keys** - never mix them!

## OAuth (if applicable)

### OAuth 2.0

- Supported: **Yes** (Connect API)
- Grant types:
  - **Authorization Code** - For third-party apps accessing user accounts
  - **Client Credentials** - For server-to-server communication
  - **Private Key JWT** - For enhanced security
- Scopes: **Not detailed in documentation** (likely read/trade/account scopes)
- Token endpoint: https://authx.alpaca.markets/v1/oauth2/token
- Authorization endpoint: **Not explicitly documented**

### OAuth Flow (Client Credentials Example)

1. **Request Token:**
```http
POST https://authx.alpaca.markets/v1/oauth2/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials&
client_id=your_client_id&
client_secret=your_client_secret
```

2. **Receive Token:**
```json
{
  "access_token": "your_access_token",
  "token_type": "Bearer",
  "expires_in": 900
}
```

3. **Use Token:**
```http
Authorization: Bearer your_access_token
```

**Token lifetime:** 900 seconds (15 minutes) - must refresh before expiration

### Private Key JWT Flow

For enhanced security, use private key JWT assertion:

1. **Generate JWT** with your private key
2. **Send assertion:**
```http
POST https://authx.alpaca.markets/v1/oauth2/token
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&
assertion=your_signed_jwt
```

3. **Receive Token** (same as above)

## Signature/HMAC (if applicable - rare for data providers)

**NOT USED** - Alpaca does NOT require HMAC signatures.

Authentication is simple:
- **REST API:** API Key ID + Secret in headers
- **WebSocket:** API Key ID + Secret in auth message
- **OAuth:** Bearer token in Authorization header

**No signature generation required** - unlike many crypto exchanges, Alpaca uses simple API key authentication without request signing.

## Authentication Examples

### REST with API Key (Python)

```python
import requests

headers = {
    "APCA-API-KEY-ID": "PKXYZ123ABC456DEF789",
    "APCA-API-SECRET-KEY": "abcdef1234567890abcdef1234567890abcdef12"
}

response = requests.get(
    "https://api.alpaca.markets/v2/account",
    headers=headers
)
```

### REST with API Key (Rust)

```rust
use reqwest::header::{HeaderMap, HeaderValue};

let mut headers = HeaderMap::new();
headers.insert(
    "APCA-API-KEY-ID",
    HeaderValue::from_static("PKXYZ123ABC456DEF789")
);
headers.insert(
    "APCA-API-SECRET-KEY",
    HeaderValue::from_static("abcdef1234567890...")
);

let client = reqwest::Client::new();
let response = client
    .get("https://api.alpaca.markets/v2/account")
    .headers(headers)
    .send()
    .await?;
```

### REST with OAuth Token

```bash
curl -H "Authorization: Bearer your_access_token" \
     https://api.alpaca.markets/v2/account
```

### WebSocket with API Key (Market Data)

**Method 1: Headers (if WebSocket client supports)**
```javascript
const ws = new WebSocket('wss://stream.data.alpaca.markets/v2/iex', {
  headers: {
    'APCA-API-KEY-ID': 'your_key_id',
    'APCA-API-SECRET-KEY': 'your_secret_key'
  }
});
```

**Method 2: Auth Message (Most Common)**
```javascript
const ws = new WebSocket('wss://stream.data.alpaca.markets/v2/iex');

ws.on('open', () => {
  ws.send(JSON.stringify({
    action: 'auth',
    key: 'your_key_id',
    secret: 'your_secret_key'
  }));
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);
  if (msg[0].T === 'success') {
    console.log('Authenticated!');
    // Now subscribe to channels
    ws.send(JSON.stringify({
      action: 'subscribe',
      trades: ['AAPL'],
      quotes: ['AAPL']
    }));
  }
});
```

### WebSocket with OAuth Token

```javascript
ws.send(JSON.stringify({
  action: 'auth',
  key: 'oauth',
  secret: 'your_oauth_access_token'
}));
```

### WebSocket with Basic Auth (Broker API)

```javascript
const auth = Buffer.from(`${key_id}:${secret_key}`).toString('base64');

// If client supports headers:
const ws = new WebSocket('wss://...', {
  headers: {
    'Authorization': `Basic ${auth}`
  }
});

// OR via auth message:
ws.send(JSON.stringify({
  action: 'auth',
  key: key_id,
  secret: secret_key
}));
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Not authenticated / Unauthorized | Check API key ID is correct, ensure auth message sent |
| 402 | Authentication failed | Verify API secret key is correct, check for typos |
| 403 | Forbidden | Insufficient subscription tier, or trying to access paid endpoint with free tier; Check account status not blocked |
| 405 | Symbol limit exceeded | Free tier limited to 30 symbols on WebSocket, reduce subscriptions |
| 406 | Connection limit exceeded | Most tiers allow only 1 WebSocket connection, close existing connection |
| 409 | Insufficient subscription | Upgrade to paid tier (Algo Trader Plus) for real-time options data |
| 422 | Unprocessable Entity | Invalid API key format, malformed auth message |
| 429 | Rate limit exceeded | Wait for rate limit reset (60 seconds for free tier) |

## Authentication Security Best Practices

1. **Never expose secrets:**
   - Don't commit API keys to git
   - Use environment variables
   - Rotate keys if compromised

2. **Use separate keys:**
   - Paper trading: Test with paper keys first
   - Live trading: Use live keys only in production
   - Different bots: Generate separate keys for isolation

3. **Monitor usage:**
   - Check Dashboard for API key activity
   - Revoke unused keys
   - Set up alerts for unusual activity

4. **OAuth for third-party apps:**
   - Don't share API keys with third parties
   - Use OAuth2 Connect API for integrations
   - Tokens expire in 15 minutes (more secure)

5. **Rate limit awareness:**
   - Free tier: 200 req/min per key
   - Paid tier: Unlimited (but fair use applies)
   - WebSocket: 1 connection per key (most tiers)

6. **Paper vs Live:**
   - **NEVER** use live keys in testing
   - Paper and live have separate base URLs
   - Double-check URL before connecting

## API Key Management

### Dashboard Access
- Login: https://app.alpaca.markets/
- Navigate: Account → API Keys
- Actions: View, Regenerate, Delete keys

### Key Permissions
- **Trading keys:** Full access to trading, account, positions, orders
- **Market data:** Same keys access market data (tier determines SIP vs IEX)
- **No read-only keys:** All keys have full permissions (use OAuth for granular access)

### Regenerating Keys
1. Go to Dashboard → API Keys
2. Click "Regenerate" on specific key
3. **Warning:** Old key immediately invalidated
4. Update all applications with new secret

### Revoking Keys
1. Go to Dashboard → API Keys
2. Click "Delete" on specific key
3. **Warning:** All applications using this key will fail authentication
4. Generate new key if needed

## Environment-Specific Authentication

### Paper Trading (Testing)
- Base URL: https://paper-api.alpaca.markets
- WebSocket: wss://paper-api.alpaca.markets/stream
- Market Data: wss://stream.data.alpaca.markets/v2/iex (or sip if paid)
- Use: **Paper API keys** from dashboard

### Live Trading (Production)
- Base URL: https://api.alpaca.markets
- WebSocket: wss://api.alpaca.markets/stream
- Market Data: wss://stream.data.alpaca.markets/v2/sip (paid) or iex (free)
- Use: **Live API keys** from dashboard
- **Warning:** Real money at risk!

### Sandbox (Development)
- Base URL: https://data.sandbox.alpaca.markets
- WebSocket: wss://stream.data.sandbox.alpaca.markets/v2/{feed}
- Use: Same API keys as production
- Purpose: Test market data integration without affecting trading

## Authentication Timeout

### REST API
- No timeout - each request authenticated independently
- Keep-alive connections supported

### WebSocket
- **Market Data:** Must authenticate within **10 seconds** after connection
- **Trading Updates:** Must authenticate before subscribing
- **Failure:** Connection closed if timeout exceeded

### OAuth Token Expiration
- Access token lifetime: **900 seconds (15 minutes)**
- Must refresh before expiration
- No refresh token documented (request new token)

## Cross-API Authentication

**Same keys work across:**
- Trading API (paper and live have separate keys)
- Market Data API (same keys, tier determines data feed)
- WebSocket streams (market data and trading updates)
- Broker API (if using broker services)

**Different keys needed for:**
- Paper vs Live environments (separate key pairs)
- OAuth applications (separate client credentials)
