# Fyers - Authentication

## Public Endpoints

- **Public endpoints exist:** Yes (limited)
- **Require authentication:** Market status, symbol master (some endpoints)
- **Rate limits without auth:** Same as authenticated (10/sec, 200/min, 100k/day)

**Public Endpoints:**
- GET `/data/market-status` - Market status
- GET `/data/symbol-master` - Symbol master CSV (may require auth)

**Note:** Most useful endpoints require authentication.

---

## API Key & App Registration

### Required For
- **All endpoints:** Most endpoints require authentication
- **Paid tier only:** No (API is free)
- **Rate limit increase:** No (same limits for all users)
- **Specific endpoints:** Trading, account data, market data quotes/depth/history

### How to Obtain

**Prerequisites:**
1. Active Fyers trading account
2. Demat account
3. Enable External 2FA TOTP

**Steps:**
1. **Sign up:** https://fyers.in/ (create trading account)
2. **Enable 2FA TOTP:**
   - Go to My Profile → https://myaccount.fyers.in/
   - Enable External 2FA TOTP
   - Scan QR code with Google/Microsoft Authenticator
   - Copy and save TOTP KEY for API automation
3. **Create API App:**
   - API Dashboard: https://myapi.fyers.in/dashboard/
   - Click "Create App"
   - Fill in app details (name, redirect URI)
   - Get `APP_ID` (Client ID) and `APP_SECRET` (Secret Key)

**API Key Management:**
- Dashboard: https://myapi.fyers.in/dashboard/
- Can create multiple apps
- Each app has separate credentials

### Free Tier Includes Key
- **Yes** - API access is completely free for Fyers account holders

---

## API Credentials Format

### App ID (Client ID)
- Format: Alphanumeric string
- Example: `ABC123XYZ-100`
- Used in: OAuth flow, API requests

### App Secret (Secret Key)
- Format: Alphanumeric string
- Example: `ABCDEFGH1234567890`
- Used in: Token generation (hashing)
- **Keep secret!** Never expose in client-side code

### Access Token
- Format: JWT (JSON Web Token)
- Example: `eyJ0eXAiOiJKV1QiLCJhbGc...` (long string)
- Full format for API calls: `APPID:ACCESS_TOKEN`
- Expiry: Limited lifetime (re-authenticate when expired)
- Used in: All authenticated API requests, WebSocket connections

---

## Authentication Method

### OAuth 2.0 Flow

Fyers uses a **custom OAuth 2.0-like flow** (not standard OAuth).

**Grant Type:** Authorization Code

**Scopes:** Not applicable (all permissions granted by default)

**Endpoints:**
- Authorization: User login via browser (redirect flow)
- Token: POST `/api/v3/validate-authcode` or `/api/v3/token`

---

## Authentication Flow

### Step 1: Generate Authorization URL

**Purpose:** Create login URL for user to authenticate in browser.

**Method:** Generate URL using SessionModel (SDK) or construct manually.

**Parameters:**
- `client_id` - Your APP_ID
- `redirect_uri` - Your registered redirect URI
- `response_type` - "code" (always)
- `state` - Random string for session management (security)

**Python SDK Example:**
```python
from fyers_apiv3 import fyersModel

client_id = "ABC123XYZ-100"
secret_key = "ABCDEFGH1234567890"
redirect_uri = "https://yourapp.com/callback"

session = fyersModel.SessionModel(
    client_id=client_id,
    secret_key=secret_key,
    redirect_uri=redirect_uri,
    response_type="code",
    state="sample_state"
)

# Generate authorization URL
auth_url = session.generate_authcode()
print("Login URL:", auth_url)
```

**Generated URL Example:**
```
https://api.fyers.in/api/v3/generate-authcode?client_id=ABC123XYZ-100&redirect_uri=https://yourapp.com/callback&response_type=code&state=sample_state
```

---

### Step 2: User Login & Authorization

**Process:**
1. User opens authorization URL in browser
2. User logs in to Fyers account
   - Username/Client ID
   - Password
   - TOTP (2FA code from authenticator app)
3. User authorizes your app
4. Fyers redirects to `redirect_uri` with `auth_code`

**Redirect URL Example:**
```
https://yourapp.com/callback?auth_code=eyJ0eXAiOiJKV1...&state=sample_state
```

**Extract auth_code from redirect:**
```python
# Parse redirect URL to get auth_code
auth_code = "eyJ0eXAiOiJKV1..."
```

---

### Step 3: Exchange Auth Code for Access Token

**Purpose:** Convert authorization code to access token.

**Method:** POST to token endpoint with appIdHash.

**Required Components:**
1. `auth_code` - From redirect (Step 2)
2. `appIdHash` - SHA-256 hash of `api_id + app_secret`

**Calculate appIdHash:**
```python
import hashlib

app_id = "ABC123XYZ-100"
app_secret = "ABCDEFGH1234567890"

# Concatenate and hash
app_id_hash = hashlib.sha256((app_id + ":" + app_secret).encode()).hexdigest()
print("appIdHash:", app_id_hash)
```

**Token Request:**
```python
session.set_token(auth_code)
response = session.generate_token()

print("Access Token:", response['access_token'])
```

**Manual Request (HTTP):**
```bash
curl -X POST https://api.fyers.in/api/v3/validate-authcode \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "authorization_code",
    "appIdHash": "abc123...",
    "code": "eyJ0eXAiOiJKV1..."
  }'
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

---

### Step 4: Use Access Token

**Format for API calls:**
```
APPID:ACCESS_TOKEN
```

**Example:**
```python
import requests

app_id = "ABC123XYZ-100"
access_token = "eyJ0eXAiOiJKV1QiLCJhbGc..."
full_token = f"{app_id}:{access_token}"

# Use in headers
headers = {
    "Authorization": full_token
}

# Get profile
response = requests.get(
    "https://api.fyers.in/api/v3/profile",
    headers=headers
)

print(response.json())
```

**Using SDK:**
```python
fyers = fyersModel.FyersModel(
    client_id=app_id,
    token=access_token
)

profile = fyers.get_profile()
print(profile)
```

---

## Signature/HMAC

### Algorithm
**NOT REQUIRED** - Fyers uses SHA-256 hash, **not HMAC**.

### SHA-256 Hash Usage

**Where Used:** Token generation (appIdHash)

**Algorithm:** SHA-256 (not HMAC-SHA256)

**Components:**
- API ID (APP_ID)
- App Secret

**Construction:**
```python
import hashlib

message = app_id + ":" + app_secret
app_id_hash = hashlib.sha256(message.encode()).hexdigest()
```

**Example:**
```
APP_ID: ABC123XYZ-100
APP_SECRET: ABCDEFGH1234567890

Message: ABC123XYZ-100:ABCDEFGH1234567890
SHA-256 Hash: a1b2c3d4e5f6... (64 character hex string)
```

**Note:** This is **only used during token generation**, not for signing every request.

---

## Request Authentication

### REST API

**Header Format:**
```
Authorization: APPID:ACCESS_TOKEN
```

**Example Request:**
```bash
curl -X GET https://api.fyers.in/api/v3/profile \
  -H "Authorization: ABC123XYZ-100:eyJ0eXAiOiJKV1QiLCJhbGc..."
```

### WebSocket

**Access Token in Connection:**
```javascript
const accessToken = "ABC123XYZ-100:eyJ0eXAiOiJKV1QiLCJhbGc...";

const ws = new WebSocket('wss://api-t1.fyers.in/socket/v3/dataSock');

// Token passed during handshake or initial message
```

---

## Token Expiry & Refresh

### Token Lifetime
- **Expiry:** Access tokens expire after a certain period (not publicly documented)
- **Typical:** Valid for trading day / 24 hours (varies)
- **No Refresh Token:** Fyers does not provide refresh tokens

### When Token Expires
- API returns **401 Unauthorized** error
- Error code: `-1600`
- Message: "Could not authenticate the user"

### Re-authentication Required
1. User must log in again (OAuth flow)
2. Generate new auth_code
3. Exchange for new access_token

### Automation Challenges
- Manual login required (browser interaction)
- 2FA TOTP needed (can be automated with TOTP key)
- Solutions:
  - Store TOTP key and generate codes programmatically
  - Use Selenium/Playwright for automated login
  - Community scripts available (e.g., fyers-api-access-token-v3)

---

## Multiple API Keys

### Multiple Keys Allowed
- **Yes** - Can create multiple apps in dashboard
- Each app has separate `APP_ID` and `APP_SECRET`

### Rate Limits Per Key
- **Per account** - Rate limits apply to the trading account, not per app
- Creating multiple apps does **not** increase rate limits

### Use Cases for Multiple Keys
1. Different applications (web, mobile, backend)
2. Separate production/development environments
3. Different strategies/algorithms
4. Security isolation (revoke one without affecting others)

---

## Authentication Examples

### REST API - Get Profile

```bash
curl -X GET https://api.fyers.in/api/v3/profile \
  -H "Authorization: ABC123XYZ-100:eyJ0eXAiOiJKV1QiLCJhbGc..."
```

### REST API - Place Order

```bash
curl -X POST https://api.fyers.in/api/v3/orders \
  -H "Authorization: ABC123XYZ-100:eyJ0eXAiOiJKV1QiLCJhbGc..." \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "NSE:SBIN-EQ",
    "qty": 100,
    "type": 2,
    "side": 1,
    "productType": "INTRADAY",
    "limitPrice": 0,
    "stopPrice": 0,
    "validity": "DAY"
  }'
```

### WebSocket - Data Connection

```python
from fyers_apiv3.FyersWebsocket import data_ws

access_token = "ABC123XYZ-100:eyJ0eXAiOiJKV1QiLCJhbGc..."

data_socket = data_ws.FyersDataSocket(
    access_token=access_token,
    log_path="",
    write_to_file=False,
    reconnect=True,
    on_message=lambda msg: print(msg)
)

data_socket.connect()
data_socket.subscribe(["NSE:SBIN-EQ"], mode="full")
```

### WebSocket - Order Connection

```python
from fyers_apiv3.FyersWebsocket import order_ws

order_socket = order_ws.FyersOrderSocket(
    access_token=access_token,
    on_orders=lambda orders: print("Orders:", orders),
    on_trades=lambda trades: print("Trades:", trades)
)

order_socket.connect()
```

---

## Error Codes

### Authentication Errors

| Code | HTTP | Description | Cause | Resolution |
|------|------|-------------|-------|------------|
| -1600 | 401 | Could not authenticate user | Invalid/expired token | Re-authenticate |
| 401 | 401 | Unauthorized | Missing/invalid credentials | Check access token |
| 403 | 403 | Forbidden | Insufficient permissions | Check account status |
| -100 | 400 | Invalid parameters | Malformed request | Verify request format |

### Token Generation Errors

| Code | Description | Resolution |
|------|-------------|------------|
| - | Invalid auth_code | Auth code expired, get new one |
| - | Invalid appIdHash | Check SHA-256 hash calculation |
| - | Invalid redirect_uri | Must match registered URI |

---

## Security Best Practices

1. **Never expose APP_SECRET** in client-side code
2. **Store credentials securely** (environment variables, secrets manager)
3. **Use HTTPS** for all API calls (enforced by Fyers)
4. **Protect access tokens** (short-lived, regenerate frequently)
5. **Enable 2FA TOTP** on your Fyers account
6. **Monitor API usage** via dashboard
7. **Revoke compromised apps** immediately in dashboard
8. **Use separate apps** for different environments
9. **Log out inactive sessions**
10. **Implement token expiry handling** in your code

---

## TOTP Automation

### For Automated Trading

**Challenge:** OAuth flow requires manual browser login with TOTP.

**Solution:** Automate TOTP code generation.

**Steps:**
1. When enabling 2FA TOTP, save the TOTP secret key
2. Use TOTP library to generate codes programmatically
3. Automate browser login with Selenium/Playwright + TOTP

**Python TOTP Example:**
```python
import pyotp

totp_secret = "YOUR_TOTP_SECRET_KEY"
totp = pyotp.TOTP(totp_secret)

# Generate current TOTP code
current_code = totp.now()
print("TOTP Code:", current_code)
```

**Community Tools:**
- https://github.com/tkanhe/fyers-api-access-token-v3
- Automated access token generation scripts

---

## Authentication Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Generate Authorization URL                              │
│    - client_id, redirect_uri, response_type, state         │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. User Login in Browser                                   │
│    - Username, Password, TOTP (2FA)                        │
│    - Authorize app                                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Redirect with auth_code                                 │
│    - https://yourapp.com/callback?auth_code=...            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Exchange auth_code for access_token                     │
│    - POST /api/v3/validate-authcode                        │
│    - Body: { grant_type, appIdHash, code }                 │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Receive access_token                                    │
│    - Response: { access_token: "..." }                     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Use access_token for API/WebSocket                     │
│    - Authorization: APPID:ACCESS_TOKEN                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Notes

1. **OAuth flow requires browser** - Not fully automated without scripting
2. **2FA TOTP is mandatory** for API access
3. **Access tokens expire** - Implement re-authentication logic
4. **No refresh tokens** - Must repeat OAuth flow
5. **appIdHash uses SHA-256**, not HMAC
6. **Rate limits are per account**, not per app
7. **WebSocket requires same access token** as REST API
8. **Token format: APPID:ACCESS_TOKEN** for all requests
9. **Monitor token expiry** and handle 401 errors gracefully
10. **Use official SDKs** for simplified authentication flow
