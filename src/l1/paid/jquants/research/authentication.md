# JQuants - Authentication

## Public Endpoints

- Public endpoints exist: No
- Require authentication: Yes (all data endpoints require auth)
- Rate limits without auth: N/A (cannot access without auth)

## API Key

### Required For
- All endpoints: Yes (except initial token acquisition)
- Paid tier only: No (Free tier also requires account and API key)
- Rate limit increase: Yes (varies by plan: Free 5/min → Premium 500/min)
- Specific endpoints: All data endpoints require authentication

### How to Obtain

1. **Sign up**: https://jpx-jquants.com/en
2. **Steps**:
   - Register email address on landing page
   - Receive verification email
   - Click confirmation link
   - Complete email verification
   - Sign in to user portal
   - Subscribe to a plan (Free, Light, Standard, or Premium)
   - Access API credentials from dashboard
3. **API key management**: User portal dashboard at https://jpx-jquants.com/en
4. **Free tier includes key**: Yes (requires email verification and plan selection)

### Authentication Flow (Two-Step Process)

JQuants uses a two-token authentication system:

1. **Refresh Token** (long-lived, 1 week validity)
2. **ID Token** (short-lived, 24 hours validity)

### Multiple Keys
- Multiple keys allowed: Not documented (likely single key per account)
- Rate limits per key: Yes (per account/API key)
- Use cases for multiple keys: Not applicable

## Authentication Method: Two-Token System

### Overview

```
Email/Password → Refresh Token (7 days) → ID Token (24 hours) → API Access
```

### Step 1: Obtain Refresh Token

**Method 1: Dashboard (Recommended)**
- Log into https://jpx-jquants.com/en
- Navigate to API settings/dashboard
- Copy refresh token directly

**Method 2: API Endpoint**

**Endpoint:** `POST https://api.jquants.com/v1/token/auth_user`

**Headers:** None required

**Body (JSON):**
```json
{
  "mailaddress": "your_email@example.com",
  "password": "your_password"
}
```

**Response (200 OK):**
```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses:**
- 400: `"'mailaddress' or 'password' is incorrect."`
- 403: Authentication forbidden
- 500: `"Unexpected error. Please try again later."`

**Example (curl):**
```bash
curl -X POST https://api.jquants.com/v1/token/auth_user \
  -H "Content-Type: application/json" \
  -d '{"mailaddress":"user@example.com","password":"pass123"}'
```

**Example (Python):**
```python
import requests, json

data = {
    "mailaddress": "user@example.com",
    "password": "your_password"
}

response = requests.post(
    "https://api.jquants.com/v1/token/auth_user",
    data=json.dumps(data)
)

refresh_token = response.json()["refreshToken"]
```

### Step 2: Obtain ID Token

**Endpoint:** `POST https://api.jquants.com/v1/token/auth_refresh`

**Query Parameters:**
- `refreshtoken`: The refresh token from Step 1

**Headers:** None required

**Response (200 OK):**
```json
{
  "idToken": "eyJraWQiOiJhYmNkZWYxMjM0NTY3ODkwIiwiYWxnIjoiUlMyNTYifQ..."
}
```

**Error Responses:**
- 400: `"'refreshtoken' is required."`
- 403: Invalid or expired refresh token
- 500: `"Unexpected error. Please try again later."`

**Example (curl):**
```bash
curl -X POST "https://api.jquants.com/v1/token/auth_refresh?refreshtoken=YOUR_REFRESH_TOKEN"
```

**Example (Python):**
```python
import requests

refresh_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

response = requests.post(
    f"https://api.jquants.com/v1/token/auth_refresh?refreshtoken={refresh_token}"
)

id_token = response.json()["idToken"]
```

### Step 3: Use ID Token for API Calls

All data endpoints require the ID token in the `Authorization` header.

**Header Format:**
```
Authorization: Bearer {idToken}
```

**Example API Call (curl):**
```bash
curl -H "Authorization: Bearer YOUR_ID_TOKEN" \
  "https://api.jquants.com/v1/prices/daily_quotes?code=27800"
```

**Example API Call (Python):**
```python
import requests

id_token = "eyJraWQiOiJhYmNkZWYxMjM0NTY3ODkwIi..."

headers = {
    "Authorization": f"Bearer {id_token}"
}

response = requests.get(
    "https://api.jquants.com/v1/prices/daily_quotes",
    headers=headers,
    params={"code": "27800"}
)

data = response.json()
```

## Token Management

### Refresh Token
- **Validity**: 7 days (1 week)
- **Renewal**: Must get new refresh token from dashboard or re-authenticate after expiry
- **Storage**: Store securely (treats like password)
- **Rotation**: Manual (retrieve new one before expiry)

### ID Token
- **Validity**: 24 hours
- **Renewal**: Automatic (request new one using refresh token)
- **Storage**: Can be cached for up to 24 hours
- **Rotation**: Recommended to refresh daily or on 401 errors

### Token Refresh Strategy

**Best Practice:**
```python
class JQuantsAuth:
    def __init__(self, refresh_token):
        self.refresh_token = refresh_token
        self.id_token = None
        self.id_token_expiry = None

    def get_id_token(self):
        # Check if current token is still valid
        if self.id_token and self.id_token_expiry > datetime.now():
            return self.id_token

        # Get new ID token
        response = requests.post(
            f"https://api.jquants.com/v1/token/auth_refresh",
            params={"refreshtoken": self.refresh_token}
        )

        self.id_token = response.json()["idToken"]
        # Set expiry to 23 hours from now (1 hour safety margin)
        self.id_token_expiry = datetime.now() + timedelta(hours=23)

        return self.id_token

    def request(self, url, params=None):
        headers = {"Authorization": f"Bearer {self.get_id_token()}"}
        response = requests.get(url, headers=headers, params=params)

        # Handle token expiry
        if response.status_code == 401:
            # Force token refresh
            self.id_token = None
            headers = {"Authorization": f"Bearer {self.get_id_token()}"}
            response = requests.get(url, headers=headers, params=params)

        return response
```

## V2 Authentication Changes (December 2025)

The V2 API introduced **API key-based authentication**, simplifying the process:

### V2 Method (Simpler)
- **Single API key**: No more two-token flow
- **Usage**: Include API key in requests (exact method TBD in V2 docs)
- **Migration**: V1 users encouraged to migrate to V2

**Note**: As of January 2026, V2 authentication details not fully documented in English docs. Likely uses:
```
X-API-Key: your_api_key_here
```
or
```
Authorization: Bearer your_api_key_here
```

Check official V2 documentation for exact header format.

## OAuth (if applicable)

### OAuth 2.0
- Supported: No
- Grant types: N/A
- Scopes: N/A
- Token endpoint: N/A (uses custom token system)
- Authorization endpoint: N/A

JQuants does NOT use standard OAuth 2.0. It uses a proprietary two-token system (refresh token + ID token).

## Signature/HMAC (if applicable - rare for data providers)

### Algorithm
- HMAC-SHA256: No
- HMAC-SHA512: No
- Signature required: No

JQuants uses **Bearer token authentication only**. No request signing required.

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 400 | Bad Request | Check email/password or parameters |
| 401 | Unauthorized | Invalid or expired ID token; refresh it |
| 403 | Forbidden | Account issue or plan tier insufficient |
| 429 | Rate Limit Exceeded | Wait before retry; upgrade plan if needed |
| 500 | Internal Server Error | Retry after delay; contact support if persists |

## Security Best Practices

1. **Never commit tokens**: Store in environment variables or secret management
2. **Rotate refresh tokens**: Get new one from dashboard before 7-day expiry
3. **Handle 401 gracefully**: Auto-refresh ID token on auth errors
4. **Rate limit awareness**: Respect tier limits to avoid 429 errors
5. **HTTPS only**: All requests must use HTTPS (enforced by API)

## Example: Complete Auth Flow (Rust)

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Deserialize)]
struct RefreshTokenResponse {
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

#[derive(Deserialize)]
struct IdTokenResponse {
    #[serde(rename = "idToken")]
    id_token: String,
}

struct JQuantsAuth {
    client: Client,
    refresh_token: String,
    id_token: Option<String>,
    id_token_expiry: Option<SystemTime>,
}

impl JQuantsAuth {
    async fn get_refresh_token(email: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let body = serde_json::json!({
            "mailaddress": email,
            "password": password
        });

        let resp: RefreshTokenResponse = client
            .post("https://api.jquants.com/v1/token/auth_user")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.refresh_token)
    }

    async fn get_id_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // Check if current token is valid
        if let (Some(token), Some(expiry)) = (&self.id_token, self.id_token_expiry) {
            if SystemTime::now() < expiry {
                return Ok(token.clone());
            }
        }

        // Get new ID token
        let url = format!(
            "https://api.jquants.com/v1/token/auth_refresh?refreshtoken={}",
            self.refresh_token
        );

        let resp: IdTokenResponse = self.client
            .post(&url)
            .send()
            .await?
            .json()
            .await?;

        // Cache token with 23-hour expiry
        self.id_token = Some(resp.id_token.clone());
        self.id_token_expiry = Some(SystemTime::now() + Duration::from_secs(23 * 3600));

        Ok(resp.id_token)
    }

    async fn request(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let token = self.get_id_token().await?;

        let resp = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        // Handle 401 by refreshing token
        if resp.status() == 401 {
            self.id_token = None; // Force refresh
            let token = self.get_id_token().await?;

            let resp = self.client
                .get(url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            return Ok(resp.text().await?);
        }

        Ok(resp.text().await?)
    }
}
```

## Rate Limit Headers

JQuants does NOT currently provide rate limit headers like:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

You must track rate limits client-side based on your plan tier.

## Recommended Implementation

For V5 connector:
1. Store refresh token in config/env
2. Implement auto-refresh for ID token (23-hour cache)
3. Handle 401 errors with automatic token refresh
4. Implement client-side rate limiting
5. Consider V2 API migration once fully documented
