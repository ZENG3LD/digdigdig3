# Angel One SmartAPI - Authentication

## Public Endpoints

- **Public endpoints exist**: Yes (very limited)
- **Require authentication**: No (only instrument master file download)
- **Rate limits without auth**: Not applicable (only one public endpoint)

### Public Endpoints
1. **Instrument Master File**: https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json
   - No authentication required
   - Publicly accessible JSON file
   - Contains all symbol tokens and metadata

**All other endpoints require authentication.**

## API Key

### Required For
- **All endpoints**: Yes (except instrument master download)
- **Paid tier only**: No (free for all Angel One clients)
- **Rate limit increase**: No (same limits for all users)
- **Specific endpoints**: All trading, market data, portfolio, and user endpoints

### How to Obtain

#### Step 1: Open Angel One Account
- Visit: https://www.angelone.in
- Complete account opening process
- Verify KYC and activate trading account

#### Step 2: Register for SmartAPI
- Login to Angel One account
- Navigate to SmartAPI portal: https://smartapi.angelone.in
- Register for SmartAPI access
- Complete registration process

#### Step 3: Generate API Key
- Access SmartAPI dashboard after registration
- Generate new API key
- Save API key securely (cannot be retrieved later)

#### Step 4: Enable TOTP (2FA)
- Generate QR code for TOTP (Time-based One-Time Password)
- Scan QR code with authenticator app (Google Authenticator, Authy, etc.)
- Save secret key for TOTP generation

### API Key Format
API keys are alphanumeric strings (exact format not publicly specified).

**Usage**: Passed during client initialization
```python
from SmartApi import SmartConnect

smartApi = SmartConnect(api_key="YOUR_API_KEY")
```

### Multiple Keys
- **Multiple keys allowed**: Yes (can generate multiple API keys)
- **Rate limits per key**: Yes (limits apply per API key)
- **Use cases for multiple keys**:
  - Separate keys for different applications
  - Development vs production keys
  - Different strategies or bots

## Authentication Flow

### Primary Authentication: Client Code + PIN + TOTP

Angel One SmartAPI uses a **three-factor authentication** system:

1. **Client Code** (Angel One trading account ID)
2. **Client PIN** (Angel One account PIN)
3. **TOTP** (Time-based One-Time Password)

### Complete Authentication Process

#### Step 1: Initialize Client
```python
from SmartApi import SmartConnect

api_key = "YOUR_API_KEY"
smartApi = SmartConnect(api_key)
```

#### Step 2: Generate Session
```python
import pyotp

# TOTP generation
totp_secret = "YOUR_TOTP_SECRET"  # From QR code during API key creation
totp = pyotp.TOTP(totp_secret).now()

# Login
data = smartApi.generateSession(
    clientCode="YOUR_CLIENT_CODE",
    password="YOUR_PIN",
    totp=totp
)
```

#### Step 3: Extract Tokens
```python
# Response contains tokens
auth_token = data['data']['jwtToken']  # JWT token for REST API
refresh_token = data['data']['refreshToken']  # For token renewal
feed_token = smartApi.getfeedToken()  # For WebSocket authentication
```

### Authentication Credentials

| Credential | Type | Purpose | Source |
|------------|------|---------|--------|
| API Key | String | Client initialization | SmartAPI dashboard |
| Client Code | String | Account identification | Angel One account |
| Client PIN | String | Account password | Angel One account |
| TOTP Secret | String | 2FA code generation | QR code during API setup |
| JWT Token | String | REST API authorization | generateSession response |
| Refresh Token | String | JWT renewal | generateSession response |
| Feed Token | String | WebSocket authorization | getfeedToken() call |

### Session Response Format
```json
{
  "status": true,
  "message": "User Logged In Successfully",
  "errorcode": "",
  "data": {
    "jwtToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "feedToken": "1234567890"
  }
}
```

## Token Types and Usage

### 1. JWT Token (Auth Token)
- **Purpose**: REST API authentication
- **Usage**: Automatically included in REST API requests by SDK
- **Validity**: Until midnight (market close)
- **Renewal**: Use refresh token
- **Storage**: Store securely, reuse for session duration

### 2. Refresh Token
- **Purpose**: Renew JWT token without re-authentication
- **Usage**: Call `renewAccessToken(refreshToken)` or `generateToken(refreshToken)`
- **Validity**: Longer than JWT token (exact duration not specified)
- **One-time use**: No (can be reused)

### 3. Feed Token
- **Purpose**: WebSocket V2 authentication
- **Usage**: Required for WebSocket connections
- **Validity**: Same as session (until midnight)
- **Retrieval**: `smartApi.getfeedToken()` after successful login
- **Separate from JWT**: Different token specifically for WebSocket

## Token Renewal

### Renewing JWT Token
```python
# Method 1: Using refresh token
new_token_data = smartApi.generateToken(refresh_token)

# Method 2: Using SDK method
new_token_data = smartApi.renewAccessToken(refresh_token)
```

### When to Renew
- Before JWT token expires (before midnight)
- On receiving HTTP 403 with TokenException
- Proactively renew every few hours for long-running applications

### Token Renewal Response
```json
{
  "status": true,
  "message": "Token Generated Successfully",
  "errorcode": "",
  "data": {
    "jwtToken": "new_jwt_token_here",
    "refreshToken": "new_refresh_token_here"
  }
}
```

## Session Management

### Session Validity
- **Duration**: From login until 12 midnight (market close)
- **Auto-renewal**: No (must renew manually or re-login next day)
- **Multiple sessions**: Yes (can have multiple concurrent sessions with different API keys)

### Session Termination

#### Manual Logout
```python
logout_response = smartApi.terminateSession(clientId="YOUR_CLIENT_CODE")
```

#### Automatic Termination
- Sessions automatically expire at midnight
- Invalid/expired tokens return HTTP 403 with TokenException

### Best Practices
1. **Login once daily**: At market open or before trading
2. **Store tokens securely**: Save JWT and refresh tokens in secure storage
3. **Monitor expiry**: Track session validity
4. **Handle errors**: Catch TokenException and re-authenticate
5. **Logout on exit**: Call terminateSession when done

## User Profile

### Get Profile
```python
profile = smartApi.getProfile(refresh_token)
```

### Profile Response
```json
{
  "status": true,
  "message": "SUCCESS",
  "data": {
    "clientcode": "A12345",
    "name": "John Doe",
    "email": "john@example.com",
    "mobileno": "9876543210",
    "exchanges": ["NSE", "BSE", "NFO", "MCX"],
    "products": ["CNC", "NRML", "MIS"],
    "lastlogintime": "2026-01-26 09:15:00",
    "broker": "ANGELONE"
  }
}
```

**Useful Information**:
- Available exchanges for the account
- Enabled product types
- Account details

## OAuth

**OAuth 2.0**: Not supported

Angel One SmartAPI does not use OAuth 2.0. Authentication is via Client Code + PIN + TOTP only.

## Signature/HMAC

**HMAC Signature**: Not required

Unlike many crypto exchanges, Angel One SmartAPI does **not require** request signing with HMAC.

Authentication is purely token-based:
- JWT token for REST API (included automatically by SDK)
- Feed token for WebSocket

**No signature construction needed.**

## Error Codes

### Authentication-Related Errors

| HTTP Code | Error Code | Description | Resolution |
|-----------|------------|-------------|------------|
| 401 | AG8001 | Invalid API Key | Verify API key is correct |
| 401 | - | Invalid Client Code/PIN | Check credentials |
| 401 | - | Invalid TOTP | Verify TOTP secret and time sync |
| 403 | TokenException | JWT token expired/invalid | Renew token or re-login |
| 403 | - | Session expired | Re-authenticate with credentials |
| 400 | - | Missing parameters | Include all required auth parameters |

### Error Response Format
```json
{
  "status": false,
  "message": "Invalid credentials",
  "errorcode": "AG8001",
  "data": null
}
```

## Authentication Examples

### Complete Authentication Flow (Python)
```python
from SmartApi import SmartConnect
import pyotp

# Configuration
API_KEY = "your_api_key"
CLIENT_CODE = "A12345"
CLIENT_PIN = "your_pin"
TOTP_SECRET = "your_totp_secret"

# Initialize
smartApi = SmartConnect(api_key=API_KEY)

# Generate TOTP
totp = pyotp.TOTP(TOTP_SECRET).now()

# Login
try:
    session_data = smartApi.generateSession(
        clientCode=CLIENT_CODE,
        password=CLIENT_PIN,
        totp=totp
    )

    # Extract tokens
    jwt_token = session_data['data']['jwtToken']
    refresh_token = session_data['data']['refreshToken']

    # Get feed token for WebSocket
    feed_token = smartApi.getfeedToken()

    print("Authentication successful!")
    print(f"JWT Token: {jwt_token[:50]}...")
    print(f"Feed Token: {feed_token}")

except Exception as e:
    print(f"Authentication failed: {e}")
```

### Token Renewal Example
```python
# Renew JWT token before expiry
try:
    new_tokens = smartApi.generateToken(refresh_token)
    jwt_token = new_tokens['data']['jwtToken']
    refresh_token = new_tokens['data']['refreshToken']
    print("Token renewed successfully!")
except Exception as e:
    print(f"Token renewal failed: {e}")
    # Re-authenticate if renewal fails
```

### Logout Example
```python
# Logout when done
try:
    logout_response = smartApi.terminateSession(clientId=CLIENT_CODE)
    print("Logged out successfully!")
except Exception as e:
    print(f"Logout failed: {e}")
```

### REST API Request with Authentication
```bash
# cURL example (manual REST call)
curl -X GET "https://apiconnect.angelone.in/rest/secure/angelbroking/user/v1/getProfile" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "X-ClientLocalIP: YOUR_CLIENT_IP" \
  -H "X-ClientPublicIP: YOUR_PUBLIC_IP" \
  -H "X-MACAddress: YOUR_MAC_ADDRESS" \
  -H "X-PrivateKey: YOUR_API_KEY"
```

**Note**: SDK handles all headers automatically. Manual header construction rarely needed.

### WebSocket Authentication Example
```python
from SmartApi.smartWebSocketV2 import SmartWebSocketV2

# After REST login
AUTH_TOKEN = jwt_token
FEED_TOKEN = feed_token

# Initialize WebSocket with tokens
sws = SmartWebSocketV2(AUTH_TOKEN, API_KEY, CLIENT_CODE, FEED_TOKEN)

# Define callbacks
def on_open(wsapp):
    print("WebSocket connected!")

def on_error(wsapp, error):
    print(f"WebSocket error: {error}")

sws.on_open = on_open
sws.on_error = on_error

# Connect (authentication happens automatically)
sws.connect()
```

## Security Best Practices

1. **Never hardcode credentials**: Use environment variables or secure configuration
2. **Protect API keys**: Store securely, never commit to version control
3. **Secure TOTP secret**: Backup securely, treat like a password
4. **Token storage**: Store JWT and refresh tokens in secure, encrypted storage
5. **HTTPS only**: All API calls use HTTPS (enforced by API)
6. **Logout after use**: Call terminateSession when done trading
7. **Monitor sessions**: Track active sessions and logout unused ones
8. **Time synchronization**: Ensure system time is accurate for TOTP generation
9. **API key rotation**: Periodically generate new API keys
10. **Access logs**: Monitor API access logs in SmartAPI dashboard

## Additional Headers (SDK Managed)

While SDK handles these automatically, REST API requests include:

| Header | Description | Example |
|--------|-------------|---------|
| Authorization | JWT token | Bearer eyJhbGci... |
| Content-Type | Request format | application/json |
| X-ClientLocalIP | Client local IP | 192.168.1.100 |
| X-ClientPublicIP | Client public IP | 203.0.113.1 |
| X-MACAddress | Client MAC | 00:1B:44:11:3A:B7 |
| X-PrivateKey | API key | YOUR_API_KEY |

**SDK automatically includes these headers in all authenticated requests.**

## Compliance & Regulations

- **SEBI Guidelines**: All API usage subject to SEBI (Securities and Exchange Board of India) regulations
- **KYC Required**: Must complete KYC to activate trading account
- **Terms Acceptance**: Must accept SmartAPI terms during registration
- **Usage Monitoring**: Angel One monitors API usage for compliance
- **Rate Limits**: Enforced to prevent abuse and ensure fair usage

## Summary

| Feature | Angel One SmartAPI |
|---------|-------------------|
| **Authentication Method** | Client Code + PIN + TOTP (3-factor) |
| **API Key Required** | Yes (for all endpoints except instrument master) |
| **Token Types** | JWT (REST), Refresh (renewal), Feed (WebSocket) |
| **Session Validity** | Until midnight (market close) |
| **Token Renewal** | Yes (via refresh token) |
| **OAuth Support** | No |
| **HMAC Signature** | No |
| **2FA Required** | Yes (TOTP mandatory) |
| **Free Tier** | Yes (all Angel One clients) |
| **SDK Availability** | Python, Java, Go, NodeJS, R, C#, PHP |
