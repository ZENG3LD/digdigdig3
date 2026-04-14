# Zerodha Kite Connect - Authentication

## Public Endpoints

- **Public endpoints exist**: No
- **Require authentication**: Yes (all endpoints require authentication)
- **Rate limits without auth**: N/A (no public endpoints)

## API Key

### Required For
- **All endpoints**: Yes
- **Paid tier only**: No (Personal API also uses API keys, but with limitations)
- **Rate limit increase**: No (rate limits are the same regardless)
- **Specific endpoints**: All endpoints require authentication

### How to Obtain
- **Sign up**: https://kite.trade/ (requires active Zerodha trading account)
- **Prerequisites**:
  - Active Zerodha trading account
  - 2FA TOTP enabled on the account
- **API key management**: https://developers.kite.trade (Kite Connect Developer Portal)
- **Free tier includes key**: Yes (Personal API keys are free)
- **Process**:
  1. Log in to Kite Connect Developer Portal
  2. Create a new app
  3. Configure redirect URL (can use http://127.0.0.1:8000 for testing)
  4. Note down api_key and api_secret

### API Key Format
- **Header**: `Authorization: token api_key:access_token`
- **NOT Query param**: API key is not passed as query parameter (except in WebSocket URL)
- **NOT Bearer token**: Uses custom "token" authorization scheme
- **Other**: Session token endpoint uses form-encoded api_key parameter

### Multiple Keys
- **Multiple keys allowed**: Yes (can create multiple apps)
- **Rate limits per key**: Yes (rate limits enforced at API key level)
- **Use cases for multiple keys**:
  - Separate development and production apps
  - Different applications for different strategies
  - Isolating rate limits across different services

## OAuth (Custom OAuth-like Flow)

### Overview
Zerodha uses a **custom OAuth-like flow** rather than standard OAuth 2.0.

### Flow Type
- **Supported**: Custom OAuth-like (NOT standard OAuth 2.0)
- **Grant types**: Authorization Code-like flow
- **Scopes**: Not applicable (full access granted)
- **Token endpoint**: POST https://api.kite.trade/session/token
- **Authorization endpoint**: https://kite.zerodha.com/connect/login?v=3&api_key={api_key}

### Authentication Flow (Step-by-Step)

#### Step 1: Navigate to Login Endpoint
```
GET https://kite.zerodha.com/connect/login?v=3&api_key={api_key}
```

**Optional Parameters**:
- `redirect_params`: URL-encoded custom data to receive back in callback

**Action**: User submits credentials on the Kite login page

---

#### Step 2: Receive Request Token
After successful login, the system redirects to the registered callback URL with:
- `request_token` - Query parameter
- `action=login` - Query parameter
- Any custom `redirect_params` provided earlier

**Example Redirect**:
```
http://127.0.0.1:8000/?request_token=abc123xyz&action=login&status=success
```

**Important**: The `request_token` is valid for only a few minutes and is single-use.

---

#### Step 3: Generate Checksum
Calculate SHA-256 hash:
```
message = api_key + request_token + api_secret
checksum = SHA256(message)
```

**Example** (Python):
```python
import hashlib

api_key = "your_api_key"
request_token = "abc123xyz"
api_secret = "your_api_secret"

message = api_key + request_token + api_secret
checksum = hashlib.sha256(message.encode()).hexdigest()
```

---

#### Step 4: Exchange for Access Token
```
POST https://api.kite.trade/session/token
Content-Type: application/x-www-form-urlencoded

api_key={api_key}
&request_token={request_token}
&checksum={checksum}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "user_type": "individual",
    "email": "user@example.com",
    "user_name": "User Name",
    "user_shortname": "User",
    "broker": "ZERODHA",
    "exchanges": ["NSE", "BSE", "NFO", "CDS", "BFO", "MCX"],
    "products": ["CNC", "NRML", "MIS"],
    "order_types": ["MARKET", "LIMIT", "SL", "SL-M"],
    "avatar_url": null,
    "user_id": "XX0000",
    "api_key": "your_api_key",
    "access_token": "generated_access_token",
    "public_token": "generated_public_token",
    "enctoken": "encrypted_token",
    "refresh_token": "",
    "silo": "",
    "login_time": "2026-01-26 10:30:45"
  }
}
```

## Token Details

| Token | Purpose | Lifetime | Usage |
|-------|---------|----------|-------|
| **request_token** | One-time exchange credential | Minutes (single-use) | Exchange for access_token |
| **access_token** | Request authentication | Until 6 AM next day or manual logout | All API requests |
| **public_token** | Public session validation | Session-dependent | Limited validation (rarely used) |
| **refresh_token** | Long-standing read access | Approved platforms only | Not generally available |
| **enctoken** | Encrypted session token | Session-dependent | Internal use |

## Request Signing

All authenticated requests must include the HTTP `Authorization` header:

```
Authorization: token api_key:access_token
```

**Format**: `token {api_key}:{access_token}`

**Example**:
```bash
curl -H "Authorization: token abc123:xyz789" \
     https://api.kite.trade/orders
```

## Session Management

### Logout Endpoint
```
DELETE https://api.kite.trade/session/token?api_key={api_key}&access_token={access_token}
```

**Action**: Invalidates tokens and destroys the API session

**Important**: This does NOT affect Kite web/mobile app sessions (only API sessions)

### Token Expiry
- **access_token** expires daily at approximately 6:00 AM IST
- New login flow required daily
- No automatic refresh mechanism available

### Re-authentication
When a token expires:
1. Redirect user to login URL again
2. Repeat the entire authentication flow
3. Obtain new access_token

## WebSocket Authentication

WebSocket connection requires authentication via query parameters:

```
wss://ws.kite.trade?api_key={api_key}&access_token={access_token}
```

**No post-connection authentication message required** - authentication happens during WebSocket handshake.

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 400 | Missing or bad request parameters | Check all required parameters are present |
| 403 | Invalid API key or session expired | Re-authenticate (TokenException) |
| 429 | Rate limit exceeded | Wait and retry with backoff |

### TokenException (403)
- **Cause**: Session expiry or invalidation
- **Resolution**: Require user re-authentication
- **Common scenarios**:
  - Daily token expiry (6 AM IST)
  - Manual logout
  - Token invalidation

## Security Notes

### Critical Security Requirements

1. **NEVER embed api_secret in client-side code**
   - Must be kept server-side only
   - Exposure allows unauthorized token generation

2. **NEVER expose access_token publicly**
   - Grants full account access
   - Can place orders and access sensitive data

3. **Use HTTPS for all communications**
   - All API endpoints require HTTPS
   - WebSocket uses WSS (secure WebSocket)

4. **Implement secure storage**
   - Store api_secret in environment variables or secure vaults
   - Never commit secrets to version control

### Best Practices

- Use localhost redirect during development
- Implement token refresh before 6 AM expiry
- Handle TokenException gracefully with re-auth flow
- Use separate API keys for development/production
- Monitor API key usage for anomalies
- Rotate API secrets periodically

## Authentication Examples

### Complete Authentication Flow (Python)
```python
import hashlib
import requests

# Step 1: Redirect user to login URL
api_key = "your_api_key"
login_url = f"https://kite.zerodha.com/connect/login?v=3&api_key={api_key}"
print(f"Navigate to: {login_url}")

# Step 2: User logs in and is redirected with request_token
# Extract request_token from callback URL

request_token = "abc123xyz"  # From callback URL
api_secret = "your_api_secret"

# Step 3: Generate checksum
message = api_key + request_token + api_secret
checksum = hashlib.sha256(message.encode()).hexdigest()

# Step 4: Exchange for access_token
response = requests.post(
    "https://api.kite.trade/session/token",
    data={
        "api_key": api_key,
        "request_token": request_token,
        "checksum": checksum
    }
)

data = response.json()
access_token = data["data"]["access_token"]

# Step 5: Use access_token for all API requests
headers = {
    "Authorization": f"token {api_key}:{access_token}"
}

orders = requests.get(
    "https://api.kite.trade/orders",
    headers=headers
)
```

### WebSocket Authentication (JavaScript)
```javascript
const apiKey = "your_api_key";
const accessToken = "your_access_token";

const ws = new WebSocket(`wss://ws.kite.trade?api_key=${apiKey}&access_token=${accessToken}`);

ws.onopen = () => {
    console.log("WebSocket connected");
    // No additional authentication message needed
};
```

## Rate Limits (Related to Authentication)

- Rate limits are enforced at the **API key level**
- All requests from a single API key must not exceed **10 requests per second**
- WebSocket subscriptions limited to **3,000 instruments**
- Max **3 WebSocket connections per API key**

## Automated Authentication

Recent developments (2026) include Model Context Protocol (MCP) servers with completely automated authentication, eliminating manual token copying. However, the standard flow still requires user interaction for login.

## Comparison with Standard OAuth 2.0

| Feature | Kite Connect | Standard OAuth 2.0 |
|---------|--------------|-------------------|
| Authorization endpoint | Custom | Standard |
| Token endpoint | Custom | Standard |
| Scopes | Not supported | Supported |
| Refresh token | Not available | Typically available |
| Token expiry | Fixed (6 AM IST) | Configurable |
| Checksum | SHA-256 manual | Not required |
| Grant types | Custom flow | Multiple standard flows |
