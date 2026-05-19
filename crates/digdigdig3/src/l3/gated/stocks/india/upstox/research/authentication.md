# Upstox - Authentication

## Public Endpoints

- **Public endpoints exist:** Yes (limited)
- **Require authentication:** No (for public historical data)
- **Rate limits without auth:** 50/s, 500/min, 2000/30min (same as authenticated)

### Public Endpoints:
- GET /v2/historical-candle/{instrument_key}/{interval}/{to_date}
- GET /v2/historical-candle/{instrument_key}/{interval}/{to_date}/{from_date}
- Instrument JSON downloads (https://assets.upstox.com/market-quote/instruments/exchange/*.json.gz)

**Note:** Most endpoints require authentication, including market quotes, trading, and portfolio operations.

---

## API Key

### Required For
- **All endpoints:** No (historical data is public)
- **Paid tier only:** Trading operations require paid subscription
- **Rate limit increase:** No (same limits for all)
- **Specific endpoints:** All market quotes, trading, portfolio, account endpoints

### How to Obtain
1. **Sign up:** https://upstox.com/open-demat-account/
2. **Open Demat account:** Required for trading operations
3. **Create API app:** https://account.upstox.com/developer/apps
4. **Get API credentials:** API Key and API Secret provided
5. **Free tier includes key:** Yes (API creation is free, usage requires subscription)

### API Key Format
- **API Key:** String (e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)
- **API Secret:** String (confidential, never share)
- **Usage:** OAuth 2.0 flow (client_id and client_secret)

### Multiple Keys
- **Multiple keys allowed:** Yes (create multiple apps)
- **Rate limits per key:** Yes (per-API, per-user basis)
- **Use cases for multiple keys:**
  - Different applications
  - Development vs Production
  - Client segregation (for businesses)

---

## OAuth 2.0

### Overview
Upstox implements **standard OAuth 2.0 for customer authentication and login**. All login operations are handled exclusively by upstox.com for security compliance.

### OAuth 2.0 Support
- **Supported:** Yes
- **Grant types:** Authorization Code (primary)
- **Scopes:** Not explicitly defined (access determined by user account type)
- **Token endpoint:** https://api.upstox.com/v2/login/authorization/token
- **Authorization endpoint:** https://api.upstox.com/v2/login/authorization/dialog

---

## Authorization Flow

### Step 1: Redirect to Authorization Dialog

**Endpoint:** `GET https://api.upstox.com/v2/login/authorization/dialog`

**Required Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| client_id | string | Yes | Your API key from app registration |
| redirect_uri | string | Yes | Callback URL (must match registered URL) |
| response_type | string | Yes | Must always be `code` |
| state | string | No | Optional for request/callback continuity |

**Example URL:**
```
https://api.upstox.com/v2/login/authorization/dialog?client_id=YOUR_API_KEY&redirect_uri=https://yourapp.com/callback&response_type=code&state=random_state_string
```

### Step 2: User Authentication
- User redirected to Upstox login page
- User enters credentials
- TOTP (Time-based One-Time Password) for 2FA if enabled
- User approves access to your application

### Step 3: Authorization Code Redirect
- User redirected to your `redirect_uri`
- Authorization code included in query parameter
- State parameter returned (if provided)

**Example callback:**
```
https://yourapp.com/callback?code=AUTH_CODE_HERE&state=random_state_string
```

### Step 4: Exchange Code for Token

**Endpoint:** `POST https://api.upstox.com/v2/login/authorization/token`

**Method:** POST

**Content-Type:** application/x-www-form-urlencoded

**Required Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| code | string | Yes | Authorization code from redirect (single-use only) |
| client_id | string | Yes | Your API key |
| client_secret | string | Yes | Your API secret (confidential) |
| redirect_uri | string | Yes | Same as in authorization request |
| grant_type | string | Yes | Must always be `authorization_code` |

**Example Request:**
```bash
curl -X POST https://api.upstox.com/v2/login/authorization/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "code=AUTH_CODE&client_id=YOUR_API_KEY&client_secret=YOUR_API_SECRET&redirect_uri=https://yourapp.com/callback&grant_type=authorization_code"
```

**Response:**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

---

## Token Types

### Standard Access Token

- **Type:** Bearer token (JWT)
- **Validity:** Until 3:30 AM IST next day (regardless of generation time)
- **Usage:** All API endpoints
- **Format:** `Authorization: Bearer {access_token}`
- **Refresh:** Not supported (must re-authenticate daily)

**Example:**
- Token generated at 8:00 PM Tuesday → Expires 3:30 AM Wednesday
- Token generated at 9:00 AM Tuesday → Expires 3:30 AM Wednesday

### Extended Token

- **Type:** Long-term access token
- **Validity:** One year from generation date
- **Usage:** Limited to five read-only endpoints:
  1. Get Positions
  2. Get Holdings
  3. Get Order Details
  4. Get Order History
  5. Get Order Book
- **Use case:** Multi-client applications, portfolio tracking
- **Availability:** Upon request to Upstox

**Note:** Extended tokens do not support trading operations (place/modify/cancel orders).

---

## Alternative Authentication Methods

### 1. Semi-Automated Authentication
- Schedule authentication requests via mobile app
- User receives mobile notification for approval
- Reduces manual login burden
- Still requires user interaction

### 2. Manual Token Generation
- Navigate to: https://account.upstox.com/developer/apps
- Click on your app
- Generate token directly from dashboard
- Copy token for use in your application
- Valid until 3:30 AM next day

---

## Signature/HMAC

**NOT REQUIRED** for Upstox API.

Upstox uses OAuth 2.0 Bearer tokens for authentication. No HMAC signing or signature generation is needed.

---

## Authentication Examples

### Full OAuth Flow (Python)
```python
import requests

# Step 1: Redirect user to authorization URL
client_id = "YOUR_API_KEY"
redirect_uri = "https://yourapp.com/callback"
auth_url = f"https://api.upstox.com/v2/login/authorization/dialog?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code"

print(f"Visit this URL to authorize: {auth_url}")

# Step 2: User authorizes and you receive code at redirect_uri
auth_code = input("Enter the authorization code: ")

# Step 3: Exchange code for token
token_url = "https://api.upstox.com/v2/login/authorization/token"
data = {
    "code": auth_code,
    "client_id": client_id,
    "client_secret": "YOUR_API_SECRET",
    "redirect_uri": redirect_uri,
    "grant_type": "authorization_code"
}

response = requests.post(token_url, data=data)
token_data = response.json()
access_token = token_data["access_token"]

print(f"Access Token: {access_token}")

# Step 4: Use token for API calls
headers = {
    "Authorization": f"Bearer {access_token}",
    "Accept": "application/json"
}

# Example: Get user profile
profile = requests.get("https://api.upstox.com/v2/user/profile", headers=headers)
print(profile.json())
```

### Using Token for API Calls
```bash
# REST API Example
curl -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
     -H "Accept: application/json" \
     https://api.upstox.com/v2/market-quote/ltp?instrument_key=NSE_EQ|INE669E01016
```

### WebSocket Authentication
```javascript
const axios = require('axios');
const WebSocket = require('ws');

// Step 1: Get authorized WebSocket URL
async function getWebSocketUrl(accessToken) {
  const response = await axios.get(
    'https://api.upstox.com/v2/feed/market-data-feed/authorize',
    {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
        'Accept': '*/*'
      }
    }
  );
  return response.data.data.authorizedRedirectUri;
}

// Step 2: Connect to WebSocket
async function connectWebSocket(accessToken) {
  const wsUrl = await getWebSocketUrl(accessToken);

  const ws = new WebSocket(wsUrl, {
    followRedirects: true
  });

  ws.on('open', () => {
    console.log('WebSocket connected');

    // Subscribe to instruments
    const subscribeMsg = {
      guid: 'unique-id',
      method: 'sub',
      data: {
        mode: 'full',
        instrumentKeys: ['NSE_EQ|INE669E01016']
      }
    };

    // Convert to binary Protocol Buffers format before sending
    // (actual implementation requires protobuf library)
    ws.send(JSON.stringify(subscribeMsg));
  });

  ws.on('message', (data) => {
    // Data is in binary Protocol Buffers format
    console.log('Received:', data);
  });
}

connectWebSocket('YOUR_ACCESS_TOKEN');
```

---

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized | Invalid or expired access token - re-authenticate |
| 403 | Forbidden | Insufficient permissions or subscription required |
| 429 | Rate limit exceeded | Wait and retry (see Retry-After header) |
| UDAPI100074 | API accessible 5:30 AM - 12:00 AM IST only | Check time and retry during market hours |
| UDAPI100049 | Access restricted | Use Uplink Business API instead |

---

## Security Best Practices

1. **Never expose API Secret:** Keep confidential, server-side only
2. **Use HTTPS:** Always use secure connections
3. **Validate state parameter:** Prevent CSRF attacks in OAuth flow
4. **Store tokens securely:** Use secure storage (encrypted database, secrets manager)
5. **Token expiry handling:** Implement re-authentication before 3:30 AM IST
6. **Single-use codes:** Authorization codes can only be used once
7. **Redirect URI validation:** Must match exactly with registered URI
8. **Don't share access tokens:** Per-user tokens, not transferable

---

## Token Management

### Token Expiry
- **Standard token expires:** 3:30 AM IST next day
- **Extended token expires:** One year from generation
- **No refresh token:** Must re-authenticate via OAuth flow

### Daily Re-authentication Strategy
```python
from datetime import datetime, time
import pytz

def is_token_expired():
    ist = pytz.timezone('Asia/Kolkata')
    now = datetime.now(ist)
    expiry_time = time(3, 30)  # 3:30 AM

    # If current time is past 3:30 AM, token from previous day is expired
    return now.time() > expiry_time

def get_fresh_token():
    if is_token_expired():
        # Re-authenticate via OAuth flow
        return perform_oauth_flow()
    else:
        # Use cached token
        return get_cached_token()
```

### Token Storage
- Store in environment variables or secure vault
- Never commit tokens to version control
- Implement encryption at rest
- Use per-user token storage for multi-user apps

---

## API Availability Window

**Important:** Upstox APIs are accessible only during specific hours:
- **Available:** 5:30 AM to 12:00 AM IST (midnight)
- **Unavailable:** 12:00 AM to 5:30 AM IST
- **Error during unavailable hours:** UDAPI100074

Plan re-authentication and token refresh accordingly.
