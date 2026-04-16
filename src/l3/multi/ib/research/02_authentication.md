# Interactive Brokers Client Portal Web API - Authentication

## Authentication Methods Overview

Interactive Brokers Client Portal Web API supports multiple authentication methods to accommodate different use cases:

1. **Client Portal Gateway** (Java-based, for individual accounts)
2. **OAuth 2.0** (for enterprise clients, modern standard)
3. **OAuth 1.0a Extended** (legacy support, still documented)
4. **SSO (Single Sign-On)** (for institutional clients)

## Client Portal Gateway Authentication

### Overview

The Client Portal Gateway is a **Java-based local proxy application** that runs on the developer's machine and handles authentication between the client application and IBKR's backend servers. This is the **primary authentication method for individual accounts**.

### Architecture

```
[Your Application] <--> [Client Portal Gateway (localhost:5000)] <--> [IBKR Backend]
                         (Handles authentication)
```

### Gateway Setup

#### Download and Installation
- Gateway available from IBKR website
- Java runtime required (JRE 8 or higher)
- Supported platforms: Windows, macOS, Linux

#### Default Configuration
- **Default Port:** 5000
- **Protocol:** HTTPS (self-signed certificate)
- **WebSocket Port:** Same as HTTP port (5000)
- **Configuration File:** `conf.yaml` in gateway root directory

#### Configuration File (conf.yaml)

```yaml
listenPort: 5000              # Can be changed to any available port
listenSsl: true               # SSL enabled by default
sslCert: root/cacert.pem      # Path to SSL certificate
sslPwd: ""                    # SSL certificate password (if any)
ips:
  allow:
    - 127.0.0.1               # Only localhost by default
  deny: []
```

#### Changing the Port

If port 5000 is occupied by another process:

1. Open `conf.yaml` in the gateway root directory
2. Modify `listenPort: 5000` to desired port (e.g., `listenPort: 5001`)
3. Restart the gateway
4. Update your API client to use the new port

### Authentication Flow

#### Step 1: Launch Gateway

```bash
# Linux/macOS
cd /path/to/clientportal.gw
bin/run.sh root/conf.yaml

# Windows
cd C:\path\to\clientportal.gw
bin\run.bat root\conf.yaml
```

#### Step 2: Browser Authentication

1. Navigate to `https://localhost:5000` in web browser
2. Accept self-signed certificate warning (expected for localhost)
3. Enter IBKR username and password
4. Complete Two-Factor Authentication (2FA) - **MANDATORY**
   - IBKR Mobile app notification
   - Security code device
   - SMS code (if configured)

#### Step 3: Session Established

Once authenticated, the gateway maintains the brokerage session, and your application can make API requests to `https://localhost:5000/v1/api/...`

### Session Management with Gateway

#### Check Authentication Status

```http
GET https://localhost:5000/v1/api/iserver/auth/status
```

**Response:**
```json
{
  "authenticated": true,
  "competing": false,
  "connected": true,
  "message": "",
  "MAC": "AA:BB:CC:DD:EE:FF",
  "serverInfo": {
    "serverName": "JifN19089",
    "serverVersion": "Build 10.22.0b, Jan 4, 2024 4:53:08 PM"
  }
}
```

**Fields:**
- `authenticated` - true if brokerage session is active
- `competing` - true if another session is competing (login conflict)
- `connected` - true if connected to backend
- `message` - Error message if authentication failed

#### Initialize Brokerage Session

If `authenticated: false`, initialize the session:

```http
POST https://localhost:5000/v1/api/iserver/auth/ssodh/init
```

**Response:**
```json
{
  "compete": false,
  "connected": true
}
```

#### Keep Session Alive (Tickle)

Sessions timeout after ~6 minutes of inactivity. Use the tickle endpoint:

```http
GET https://localhost:5000/v1/api/tickle
```

**Rate Limit:** 1 request per second

**Response:**
```json
{
  "session": "123abc456def",
  "ssoExpires": 600000,
  "collission": false,
  "userId": 12345678,
  "hmds": {
    "error": "no bridge"
  },
  "iserver": {
    "tickle": true,
    "authStatus": {
      "authenticated": true,
      "competing": false,
      "connected": true
    }
  }
}
```

**Fields:**
- `ssoExpires` - Milliseconds until session expires
- `collission` - Competing session detected
- `iserver.authStatus.authenticated` - Brokerage session status

#### Logout

```http
POST https://localhost:5000/v1/api/logout
```

**Response:**
```json
{
  "confirmed": true
}
```

### Session Constraints

#### Timeouts
- **Idle Timeout:** ~6 minutes without requests
- **Maximum Session Duration:** 24 hours
- **Automatic Reset:** Midnight in Eastern Time (New York), CET (Zug), or HKT (Hong Kong)

#### Limitations
- **Single Concurrent Session:** Only one active session per username at any time
- **Same Machine Requirement:** Browser authentication must occur on same machine as gateway
- **No Automation:** Individual accounts cannot automate initial authentication (browser login required)

### SSL Certificate Handling

#### Self-Signed Certificate Warning

The gateway uses a self-signed certificate by default, causing browser warnings:

```
Warning: Your connection is not private
NET::ERR_CERT_AUTHORITY_INVALID
```

**This is expected and safe for localhost connections.**

#### Accepting Certificate in Code

**Python Example:**
```python
import requests
import urllib3

# Disable SSL warnings (only for localhost development)
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

response = requests.get(
    'https://localhost:5000/v1/api/iserver/auth/status',
    verify=False  # Disable SSL verification for self-signed cert
)
```

**Note:** Only disable SSL verification for localhost gateway. Always verify SSL for production OAuth endpoints.

#### Custom SSL Certificate

To use a valid SSL certificate:

1. Obtain SSL certificate and private key
2. Place certificate in gateway `root/` directory
3. Update `conf.yaml`:

```yaml
sslCert: root/your-cert.pem
sslPwd: "your-cert-password"
```

4. Restart gateway

## OAuth 2.0 Authentication

### Overview

OAuth 2.0 is the **recommended authentication method for enterprise clients** and institutional integrations. IBKR uses **Private Key JWT** (RFC 7521, RFC 7523) for client authentication, which is more secure than traditional client secret authentication.

### OAuth 2.0 Flow Type

**Flow:** Client Credentials with Private Key JWT
**Standard:** RFC 6749 (OAuth 2.0), RFC 7521 (JWT Client Authentication)

### Prerequisites

1. **Enterprise Account:** OAuth 2.0 primarily for institutional/enterprise clients
2. **API Registration:** Register application in IBKR Self Service Portal
3. **RSA Key Pair:** Generate public/private key pair (2048-bit or higher)
4. **Public Key Upload:** Upload public key to IBKR during registration

### OAuth 2.0 Endpoints

**Authorization Server:** `https://api.ibkr.com/v1/api/oauth/`

**Token Endpoint:** `POST https://api.ibkr.com/v1/api/oauth/token`

### Client Registration

#### 1. Generate RSA Key Pair

```bash
# Generate private key (2048-bit)
openssl genrsa -out private_key.pem 2048

# Extract public key
openssl rsa -in private_key.pem -pubout -out public_key.pem
```

#### 2. Register Application

- Login to IBKR Self Service Portal
- Navigate to API Management section
- Register new OAuth application
- Upload public key (`public_key.pem`)
- Receive `client_id` (consumer key)

### Private Key JWT Generation

#### JWT Header

```json
{
  "alg": "RS256",
  "typ": "JWT"
}
```

#### JWT Payload (Claims)

```json
{
  "iss": "YOUR_CLIENT_ID",       // Issuer (your client_id)
  "sub": "YOUR_CLIENT_ID",       // Subject (your client_id)
  "aud": "https://api.ibkr.com/v1/api/oauth/token",  // Audience
  "exp": 1706282400,             // Expiration (Unix timestamp, max 5 minutes)
  "iat": 1706282100,             // Issued At (Unix timestamp)
  "jti": "unique-jwt-id-12345"   // JWT ID (unique per request)
}
```

#### JWT Signature

Sign the JWT using your **private key** with RS256 (RSA + SHA256):

```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  private_key
)
```

### Access Token Request

```http
POST https://api.ibkr.com/v1/api/oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=YOUR_CLIENT_ID
&client_assertion_type=urn:ietf:params:oauth:client-assertion-type:jwt-bearer
&client_assertion=YOUR_SIGNED_JWT_TOKEN
```

**Parameters:**
- `grant_type` - Always `client_credentials`
- `client_id` - Your registered client ID
- `client_assertion_type` - Always `urn:ietf:params:oauth:client-assertion-type:jwt-bearer`
- `client_assertion` - Your signed JWT token

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsIn...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read write"
}
```

### Using Access Token

Include the access token in the `Authorization` header for all API requests:

```http
GET https://api.ibkr.com/v1/api/portfolio/accounts
Authorization: Bearer eyJhbGciOiJSUzI1NiIsIn...
```

### Token Refresh

OAuth 2.0 access tokens expire after the time specified in `expires_in` (typically 3600 seconds / 1 hour). Request a new token before expiration using the same flow.

### OAuth 2.0 Python Example

```python
import time
import jwt
import requests
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.backends import default_backend

# Load private key
with open('private_key.pem', 'rb') as key_file:
    private_key = serialization.load_pem_private_key(
        key_file.read(),
        password=None,
        backend=default_backend()
    )

# JWT claims
client_id = "YOUR_CLIENT_ID"
now = int(time.time())
claims = {
    "iss": client_id,
    "sub": client_id,
    "aud": "https://api.ibkr.com/v1/api/oauth/token",
    "exp": now + 300,  # Expires in 5 minutes
    "iat": now,
    "jti": f"jwt-{now}"
}

# Sign JWT
client_assertion = jwt.encode(claims, private_key, algorithm='RS256')

# Request access token
token_response = requests.post(
    'https://api.ibkr.com/v1/api/oauth/token',
    data={
        'grant_type': 'client_credentials',
        'client_id': client_id,
        'client_assertion_type': 'urn:ietf:params:oauth:client-assertion-type:jwt-bearer',
        'client_assertion': client_assertion
    }
)

token_data = token_response.json()
access_token = token_data['access_token']

# Use access token
headers = {'Authorization': f'Bearer {access_token}'}
response = requests.get('https://api.ibkr.com/v1/api/portfolio/accounts', headers=headers)
print(response.json())
```

## OAuth 1.0a Extended Authentication

### Overview

OAuth 1.0a Extended is a **legacy authentication method** still supported by IBKR. It uses HMAC-SHA256 signatures for request authentication.

### OAuth 1.0a Endpoints

**Request Token:** `POST https://api.ibkr.com/v1/api/oauth/request_token`
**Access Token:** `POST https://api.ibkr.com/v1/api/oauth/access_token`
**Live Requests:** Include OAuth signature in `Authorization` header

### Consumer Registration

1. Login to IBKR Self Service Portal
2. Navigate to OAuth consumer registration
3. Register new consumer
4. Receive:
   - `oauth_consumer_key` (9-character string)
   - `consumer_secret` (for signature generation)

### OAuth 1.0a Parameters

**Required in Authorization Header:**

```
OAuth oauth_consumer_key="YOUR_KEY",
      oauth_token="ACCESS_TOKEN",
      oauth_signature_method="HMAC-SHA256",
      oauth_timestamp="1706282400",
      oauth_nonce="unique-nonce-12345",
      oauth_signature="BASE64_SIGNATURE"
```

**Parameter Descriptions:**

- `oauth_consumer_key` - Your 9-character consumer key
- `oauth_token` - Access token (obtained via `/access_token` endpoint or portal)
- `oauth_signature_method` - Always `HMAC-SHA256` (only supported method)
- `oauth_timestamp` - Unix timestamp (must be >= previous request timestamp)
- `oauth_nonce` - Unique random string for replay protection
- `oauth_signature` - HMAC-SHA256 signature of request

### Signature Generation

#### Signature Base String

```
HTTP_METHOD&
URL_ENCODED(BASE_URL)&
URL_ENCODED(SORTED_PARAMETERS)
```

**Example:**
```
GET&
https%3A%2F%2Fapi.ibkr.com%2Fv1%2Fapi%2Fportfolio%2Faccounts&
oauth_consumer_key%3DABC123456%26oauth_nonce%3Drandom123%26oauth_signature_method%3DHMAC-SHA256%26oauth_timestamp%3D1706282400%26oauth_token%3Dtoken123
```

#### Signing Key

```
URL_ENCODED(consumer_secret)&URL_ENCODED(token_secret)
```

If no token secret: `consumer_secret&`

#### Compute Signature

```python
import hmac
import hashlib
import base64

signature = base64.b64encode(
    hmac.new(
        signing_key.encode('utf-8'),
        signature_base_string.encode('utf-8'),
        hashlib.sha256
    ).digest()
).decode('utf-8')
```

### OAuth 1.0a Request Example

```http
GET https://api.ibkr.com/v1/api/portfolio/accounts
Authorization: OAuth oauth_consumer_key="ABC123456",
                     oauth_token="xyz789token",
                     oauth_signature_method="HMAC-SHA256",
                     oauth_timestamp="1706282400",
                     oauth_nonce="random123",
                     oauth_signature="cZE6FhHKxwQg6y7V+JjBWDG8YwA="
```

### OAuth 1.0a Python Example

```python
import hmac
import hashlib
import base64
import time
import random
import string
from urllib.parse import quote

# Configuration
consumer_key = "YOUR_CONSUMER_KEY"
consumer_secret = "YOUR_CONSUMER_SECRET"
access_token = "YOUR_ACCESS_TOKEN"
token_secret = "YOUR_TOKEN_SECRET"  # May be empty

# Request details
method = "GET"
url = "https://api.ibkr.com/v1/api/portfolio/accounts"

# OAuth parameters
oauth_params = {
    "oauth_consumer_key": consumer_key,
    "oauth_token": access_token,
    "oauth_signature_method": "HMAC-SHA256",
    "oauth_timestamp": str(int(time.time())),
    "oauth_nonce": ''.join(random.choices(string.ascii_letters + string.digits, k=32))
}

# Create signature base string
param_string = "&".join([f"{k}={quote(v, safe='')}" for k, v in sorted(oauth_params.items())])
signature_base = f"{method}&{quote(url, safe='')}&{quote(param_string, safe='')}"

# Create signing key
signing_key = f"{quote(consumer_secret, safe='')}&{quote(token_secret, safe='')}"

# Generate signature
signature = base64.b64encode(
    hmac.new(signing_key.encode(), signature_base.encode(), hashlib.sha256).digest()
).decode()

oauth_params["oauth_signature"] = signature

# Build Authorization header
auth_header = "OAuth " + ", ".join([f'{k}="{v}"' for k, v in oauth_params.items()])

# Make request
import requests
response = requests.get(url, headers={"Authorization": auth_header})
print(response.json())
```

## SSO (Single Sign-On) Authentication

### Overview

SSO authentication is available for **institutional clients** with custom identity provider integrations. This is typically used in enterprise environments with centralized authentication systems.

### SSO Endpoints

**Validate SSO Session:** `GET https://api.ibkr.com/v1/api/sso/validate`

**Rate Limit:** 1 request per minute

### SSO Flow

1. User authenticates via institutional identity provider
2. Identity provider issues SSO token
3. Application validates SSO token with IBKR: `GET /sso/validate`
4. IBKR returns session validation response
5. Application proceeds with authenticated requests

### SSO Requirements

- Enterprise agreement with IBKR
- Custom integration setup
- Identity provider configuration
- SSO token exchange mechanism

## Authentication Comparison

| Feature | Client Portal Gateway | OAuth 2.0 | OAuth 1.0a | SSO |
|---------|----------------------|-----------|------------|-----|
| **Target Users** | Individual accounts | Enterprise clients | Legacy systems | Institutional |
| **Setup Complexity** | Low | Medium | High | High |
| **Automation** | No (manual login) | Yes | Yes | Yes |
| **Session Management** | Gateway handles | Token-based | Token-based | IdP-based |
| **Security** | 2FA + Browser | Private Key JWT | HMAC-SHA256 | IdP-dependent |
| **Rate Limit** | 10 req/s | 50 req/s | 50 req/s | Varies |
| **Use Case** | Personal trading | Automated systems | Legacy apps | Enterprise SSO |

## Best Practices

### Security
1. **Never hardcode credentials** in source code
2. **Use environment variables** for sensitive data
3. **Store private keys securely** (encrypted file system, key management service)
4. **Rotate credentials regularly** (especially OAuth tokens)
5. **Monitor for authentication failures** and alert on anomalies

### Session Management
1. **Implement tickle mechanism** to keep sessions alive
2. **Handle session timeouts gracefully** with automatic re-authentication
3. **Check auth status** before critical operations
4. **Monitor competing sessions** and handle conflicts
5. **Respect rate limits** on authentication endpoints

### Error Handling
1. **Implement exponential backoff** for auth failures
2. **Log authentication events** (successful and failed)
3. **Handle HTTP 401 Unauthorized** with re-authentication flow
4. **Handle HTTP 429 Too Many Requests** with backoff
5. **Provide clear error messages** to users

### Development vs Production
1. **Development:** Use Client Portal Gateway with localhost
2. **Production (Individual):** Use Client Portal Gateway with monitoring
3. **Production (Enterprise):** Use OAuth 2.0 with token refresh logic
4. **Testing:** Separate test accounts from production accounts

## Common Authentication Issues

### Issue: "Not authenticated" error despite login

**Cause:** Brokerage session not initialized
**Solution:** Call `POST /iserver/auth/ssodh/init` after login

### Issue: Session timeout after a few minutes

**Cause:** No tickle requests sent
**Solution:** Implement periodic tickle calls (every 30-60 seconds)

### Issue: "Competing session" detected

**Cause:** Same username logged in elsewhere
**Solution:** Logout other session or wait for timeout

### Issue: Gateway certificate errors in production

**Cause:** Self-signed certificate not trusted
**Solution:** Use custom valid SSL certificate or implement proper cert handling

### Issue: OAuth token expired

**Cause:** Token lifetime exceeded
**Solution:** Request new token before expiration using refresh logic

### Issue: HMAC signature mismatch (OAuth 1.0a)

**Cause:** Incorrect parameter sorting or encoding
**Solution:** Verify signature base string construction and parameter sorting

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**Authentication Methods:** Gateway, OAuth 2.0, OAuth 1.0a, SSO
