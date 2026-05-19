# Dhan - Authentication

## Public Endpoints

- Public endpoints exist: Yes (limited)
- Public endpoints:
  - `GET /v2/instrument/{exchangeSegment}` - Instrument lists (CSV)
- Require authentication: No (for instrument lists only)
- Rate limits without auth: Unlimited for instrument lists

## API Key

### Required For
- All endpoints: Yes (except instrument lists)
- Paid tier only: No (free for all Dhan users)
- Rate limit increase: No (same limits for all tiers)
- Specific endpoints: All Trading APIs, Data APIs, Portfolio APIs

### How to Obtain
- Sign up: https://dhanhq.co/ or https://dhan.co/
- Open Dhan trading account (required)
- API key management: Web portal at web.dhan.co
- Free tier includes key: Yes (all Dhan users get free API access)
- Process:
  1. Login to Dhan web portal
  2. Navigate to API settings
  3. Generate API Key and Secret (1-year validity)
  4. Configure Static IP addresses (mandatory from Jan 2026 for Order APIs)
  5. Generate daily access token using API key/secret

### API Key Format
- **API Key**: Long alphanumeric string
- **API Secret**: Long alphanumeric string (keep confidential)
- **Access Token**: JWT format (eyJhbGc...)
- **Validity**:
  - API Key: 1 year
  - API Secret: 1 year (same as key)
  - Access Token: 24 hours

### Authentication Method
- **Header**: `access-token: JWT_ACCESS_TOKEN`
- **NOT** Bearer token format
- **NOT** X-API-Key format
- **NOT** Query parameter

### Example Headers
```
Content-Type: application/json
access-token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Multiple Keys
- Multiple keys allowed: No (one API key per account)
- Rate limits per key: Yes (all limits are per API key/account)
- Use cases for multiple keys: N/A (single key per account)

## Access Token Generation

### Method: API Key + Secret Based

**Step 1: Generate Access Token**

**Endpoint**: `POST https://api.dhan.co/v2/access_token`

**Request Headers**:
```
Content-Type: application/json
```

**Request Body**:
```json
{
  "client_id": "1000000123",
  "api_key": "your_api_key_here",
  "api_secret": "your_api_secret_here"
}
```

**Response** (Success):
```json
{
  "status": "success",
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "remarks": "Login Successful"
}
```

**Response** (Failure):
```json
{
  "errorType": "AuthenticationError",
  "errorCode": "AS4001",
  "errorMessage": "Invalid API credentials"
}
```

### Token Validity: 24 Hours
- Access token expires after 24 hours
- Must generate new token daily
- Expiry time is from token generation, not calendar day
- Cannot extend validity
- SEBI compliance requirement (as of 2026)

### Token Renewal

**For Web-Generated Tokens Only**:
If token was generated via Dhan web portal (not API), you can renew:

**Endpoint**: `POST https://api.dhan.co/v2/access_token/renew`

**Request Headers**:
```
Content-Type: application/json
access-token: CURRENT_TOKEN
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "remarks": "Token renewed successfully"
}
```

**Note**: Renewing expires the current token and provides a new one with 24-hour validity.

## OAuth (if applicable)

### OAuth 2.0
- Supported: No
- Grant types: N/A
- Scopes: N/A
- Token endpoint: N/A
- Authorization endpoint: N/A

Dhan uses proprietary API Key + Secret authentication, not OAuth 2.0.

## Signature/HMAC (if applicable)

### Algorithm
- **NOT USED** - Dhan uses simple JWT-based authentication

Unlike most exchange APIs (Binance, etc.), Dhan does NOT require:
- HMAC signing
- Request signature computation
- Timestamp in signature
- Nonce/recvWindow

Authentication is straightforward:
1. Generate access token using API key/secret
2. Include token in `access-token` header
3. Token validates all requests

## Static IP Requirement (2026 Update)

### Mandatory From January 2026

**Required For**:
- All Order APIs (POST, PUT, DELETE `/v2/orders`)
- Super Orders APIs
- Forever Orders APIs
- Any order placement/modification/cancellation

**NOT Required For**:
- Data APIs
- Portfolio/Holdings APIs
- Market data APIs
- Sandbox/Testing environment
- Historical data APIs

### How to Configure
1. Login to Dhan web portal (web.dhan.co)
2. Navigate to API Settings
3. Add Static IP address(es)
4. At least 1 Static IP is mandatory
5. Can add multiple Static IPs

### Validation
- Order API requests checked against whitelisted IPs
- Requests from non-whitelisted IPs are rejected
- Error returned if IP not whitelisted

### Error Response (Non-Whitelisted IP)
```json
{
  "errorType": "SecurityError",
  "errorCode": "IP4001",
  "errorMessage": "Request from non-whitelisted IP address"
}
```

## Authentication Examples

### REST with Access Token

**Generate Token**:
```bash
curl -X POST https://api.dhan.co/v2/access_token \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "1000000123",
    "api_key": "your_api_key",
    "api_secret": "your_api_secret"
  }'
```

**Use Token for API Call**:
```bash
curl -X GET https://api.dhan.co/v2/holdings \
  -H "Content-Type: application/json" \
  -H "access-token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Place Order**:
```bash
curl -X POST https://api.dhan.co/v2/orders \
  -H "Content-Type: application/json" \
  -H "access-token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{
    "dhanClientId": "1000000123",
    "transactionType": "BUY",
    "exchangeSegment": "NSE_EQ",
    "productType": "INTRADAY",
    "orderType": "MARKET",
    "validity": "DAY",
    "securityId": "1333",
    "quantity": 1
  }'
```

### WebSocket with Access Token

**Query Parameter Method**:
```javascript
const ws = new WebSocket('wss://api-feed.dhan.co?token=eyJhbGc...&version=2');
```

**Subscription Message Method**:
```javascript
const ws = new WebSocket('wss://api-feed.dhan.co');

ws.onopen = () => {
  // Subscribe with authentication
  ws.send(JSON.stringify({
    "RequestCode": 15,
    "InstrumentCount": 1,
    "InstrumentList": [{"ExchangeSegment": 1, "SecurityId": "1333"}]
  }));
};
```

**Market Depth WebSocket**:
```javascript
const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
const clientId = "1000000123";
const ws = new WebSocket(
  `wss://depth-api-feed.dhan.co/twentydepth?token=${token}&clientId=${clientId}&authType=2`
);
```

### Python SDK Example
```python
from dhanhq import dhanhq

# Initialize with credentials
dhan = dhanhq(
    client_id="1000000123",
    access_token="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
)

# Access token must be generated externally (via API or web)
# SDK uses the provided token for all requests
```

### Generating Token Programmatically (Python)
```python
import requests

def generate_access_token(client_id, api_key, api_secret):
    url = "https://api.dhan.co/v2/access_token"
    headers = {"Content-Type": "application/json"}
    payload = {
        "client_id": client_id,
        "api_key": api_key,
        "api_secret": api_secret
    }

    response = requests.post(url, json=payload, headers=headers)
    data = response.json()

    if response.status_code == 200:
        return data['access_token']
    else:
        raise Exception(f"Auth failed: {data}")

# Usage
token = generate_access_token("1000000123", "api_key", "api_secret")
```

### Rust Example
```rust
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AuthRequest {
    client_id: String,
    api_key: String,
    api_secret: String,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    status: String,
}

async fn generate_access_token(
    client_id: &str,
    api_key: &str,
    api_secret: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let auth_request = AuthRequest {
        client_id: client_id.to_string(),
        api_key: api_key.to_string(),
        api_secret: api_secret.to_string(),
    };

    let response = client
        .post("https://api.dhan.co/v2/access_token")
        .header("Content-Type", "application/json")
        .json(&auth_request)
        .send()
        .await?;

    let auth_response: AuthResponse = response.json().await?;
    Ok(auth_response.access_token)
}
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized - Invalid token | Generate new access token |
| 403 | Forbidden - IP not whitelisted | Add IP to whitelist in Dhan portal |
| AS4001 | Invalid API credentials | Check API key and secret |
| AS4002 | Token expired | Generate new access token (24h validity) |
| AS4003 | Invalid client ID | Verify Dhan client ID |
| IP4001 | Non-whitelisted IP | Add static IP in API settings |
| AS4004 | Account suspended | Contact Dhan support |

## Security Best Practices

### API Key & Secret Storage
- **Never** commit to Git/version control
- Store in environment variables
- Use secrets management (AWS Secrets Manager, HashiCorp Vault, etc.)
- Rotate keys periodically (annually at minimum)

### Access Token Handling
- Store securely in memory
- Don't log tokens
- Regenerate daily (automated)
- Implement token refresh before expiry
- Clear tokens on application shutdown

### Static IP Security
- Use dedicated servers with static IPs
- Don't share static IPs across multiple services
- Monitor for unauthorized access
- Implement IP rotation strategy if using cloud (Elastic IPs)

### Rate Limit Respect
- Don't share API keys across multiple applications
- Implement request queuing
- Use exponential backoff on errors
- Monitor usage to avoid hitting limits

## Sandbox Environment Authentication

### Sandbox-Specific Notes
- Same authentication method (API key + secret)
- Separate API keys for sandbox vs production
- **Static IP NOT required** for sandbox
- Token validity same (24 hours)
- Generate sandbox keys from sandbox portal

### Sandbox URL
- Sandbox base URL: Same as production (https://api.dhan.co/)
- Differentiated by account type (sandbox account vs live account)
- Sandbox data is simulated (all orders filled at 100)

## Common Authentication Errors

### Error: "Invalid API credentials"
- **Cause**: Wrong API key or secret
- **Fix**: Regenerate keys from Dhan portal

### Error: "Token expired"
- **Cause**: 24-hour token validity exceeded
- **Fix**: Generate new token daily

### Error: "IP not whitelisted"
- **Cause**: Static IP requirement not met (from Jan 2026)
- **Fix**: Add current IP to whitelist

### Error: "Account not enabled for APIs"
- **Cause**: API access not activated for account
- **Fix**: Enable API access in Dhan settings

## Token Management Strategy

### For Production Systems
1. **Auto-generate token daily**: Schedule token generation (e.g., 6 AM IST before market open)
2. **Token caching**: Store in memory/Redis with expiry
3. **Graceful refresh**: Regenerate 1 hour before expiry
4. **Fallback handling**: Retry with new token on auth errors
5. **Monitoring**: Alert on authentication failures

### Example Token Manager (Python)
```python
from datetime import datetime, timedelta
import time

class DhanTokenManager:
    def __init__(self, client_id, api_key, api_secret):
        self.client_id = client_id
        self.api_key = api_key
        self.api_secret = api_secret
        self.token = None
        self.token_expiry = None

    def get_token(self):
        # Check if token exists and is valid
        if self.token and self.token_expiry > datetime.now():
            return self.token

        # Generate new token
        self.token = self._generate_token()
        self.token_expiry = datetime.now() + timedelta(hours=23)  # 23h for safety margin
        return self.token

    def _generate_token(self):
        # Call token generation API
        # ... (implementation from earlier example)
        pass
```

## Rate Limiting for Authentication

- Token generation: 20 requests/second
- Practically: Generate once per day
- No need for aggressive token refresh
- Implement single token generation per session
