# Bitquery - Authentication

## Public Endpoints

- **Public endpoints exist**: No
- **All endpoints require authentication**: Yes (API token required for all queries)
- **Rate limits without auth**: Not applicable - auth always required
- **Anonymous access**: Not available

---

## API Key / OAuth Token

### Required For
- **All endpoints**: Yes - Every GraphQL query/subscription requires authentication
- **Free tier**: Yes - API key required even on free Developer plan
- **Paid tier**: Yes - Different tier, same auth mechanism
- **WebSocket**: Yes - Token required for subscriptions

### How to Obtain

#### 1. Sign Up
- **URL**: https://account.bitquery.io/auth/signup
- **Requirements**:
  - Email address
  - Password
  - Name and company name
  - CAPTCHA verification
  - Email confirmation (verification link sent)

#### 2. Access Token Generation
After signup, generate OAuth token:

**Method 1: Via GraphQL IDE**
- Go to https://ide.bitquery.io
- Log in with credentials
- Token auto-generated for IDE use
- Copy from IDE settings/account section

**Method 2: Via Account Dashboard**
- Go to https://account.bitquery.io
- Navigate to API Keys section
- Generate new OAuth token
- Copy token (format: `ory_at_...`)

**Method 3: OAuth Client Credentials (Advanced)**
- Create OAuth application in account settings
- Use client ID + secret
- Request access token via OAuth 2.0 flow
- Token expires and requires refresh

### API Key Format

#### REST-like GraphQL (POST)
**Header Method (Recommended)**:
```http
POST https://streaming.bitquery.io/graphql
Content-Type: application/json
Authorization: Bearer ory_at_YOUR_ACCESS_TOKEN

{
  "query": "{ EVM(network: eth) { Blocks { ... } } }"
}
```

**Alternative: URL Parameter**:
```http
POST https://streaming.bitquery.io/graphql?token=ory_at_YOUR_ACCESS_TOKEN
Content-Type: application/json

{
  "query": "{ ... }"
}
```

#### WebSocket (Subscriptions)
**URL Parameter (Recommended)**:
```
wss://streaming.bitquery.io/graphql?token=ory_at_YOUR_ACCESS_TOKEN
```

**Alternative: Header** (if client supports):
```javascript
// Node.js with ws library
const ws = new WebSocket('wss://streaming.bitquery.io/graphql', {
  headers: {
    'Authorization': 'Bearer ory_at_YOUR_ACCESS_TOKEN'
  }
});
```

### Token Format
- **Prefix**: `ory_at_`
- **Type**: OAuth 2.0 Access Token
- **Example**: `ory_at_abcdef1234567890xyz...`
- **Encoding**: Base64-encoded string (typically ~100+ characters)

### Multiple Keys
- **Multiple tokens allowed**: Yes
- **Per application**: Can create separate OAuth applications
- **Use cases**:
  - Different applications/services
  - Development vs production
  - Rate limit isolation (each key has own limits)
- **Management**: Via account dashboard

### Token Expiration
- **Access tokens expire**: Yes (OAuth 2.0 standard)
- **Typical lifetime**: Not documented (likely 1-24 hours)
- **Refresh required**: Yes, via OAuth refresh token flow
- **IDE auto-refreshes**: Yes (IDE handles refresh automatically)

---

## OAuth (if applicable)

### OAuth 2.0
- **Supported**: Yes (primary auth method)
- **Grant types**:
  - **Client Credentials**: For server-to-server (recommended)
  - **Authorization Code**: For user-facing applications (if needed)
- **Scopes**: Not explicitly documented (likely full access per tier)
- **Token endpoint**: Part of account.bitquery.io OAuth flow
- **Authorization endpoint**: https://account.bitquery.io/auth/...

### OAuth Flow (Client Credentials)

#### 1. Create OAuth Application
- Go to https://account.bitquery.io
- Navigate to OAuth/API settings
- Create new application
- Get `client_id` and `client_secret`

#### 2. Request Access Token
```bash
curl -X POST https://account.bitquery.io/oauth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "client_credentials",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET"
  }'
```

**Response**:
```json
{
  "access_token": "ory_at_abcdef1234567890...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

#### 3. Use Access Token
```bash
curl -X POST https://streaming.bitquery.io/graphql \
  -H "Authorization: Bearer ory_at_abcdef1234567890..." \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ EVM(network: eth, dataset: archive) { Blocks(limit: {count: 1}) { Block { Number } } } }"
  }'
```

#### 4. Refresh Token (when expired)
Re-request using client credentials (access token expires, refresh by re-authenticating)

### OAuth Best Practices
1. **Store credentials securely**: Never commit client secrets
2. **Rotate tokens**: Regenerate if compromised
3. **Use environment variables**: For client_id/client_secret
4. **Cache access tokens**: Until expiration
5. **Handle 401 errors**: Re-authenticate automatically

---

## Signature/HMAC (if applicable)

**NOT USED** - Bitquery uses OAuth tokens only, no HMAC signing required.

Unlike crypto exchanges (Binance, Bybit), Bitquery doesn't require request signing.

---

## Authentication Examples

### REST-like GraphQL with API Key

#### cURL
```bash
curl -X POST https://streaming.bitquery.io/graphql \
  -H "Authorization: Bearer ory_at_YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ EVM(network: eth, dataset: archive) { Blocks(limit: {count: 10}) { Block { Number Time } } } }"
  }'
```

#### Python (requests)
```python
import requests

url = 'https://streaming.bitquery.io/graphql'
headers = {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer ory_at_YOUR_TOKEN'
}
query = """
{
  EVM(network: eth, dataset: archive) {
    Blocks(limit: {count: 10}) {
      Block {
        Number
        Time
      }
    }
  }
}
"""
response = requests.post(url, headers=headers, json={'query': query})
print(response.json())
```

#### JavaScript (fetch)
```javascript
const url = 'https://streaming.bitquery.io/graphql';
const headers = {
  'Content-Type': 'application/json',
  'Authorization': 'Bearer ory_at_YOUR_TOKEN'
};
const query = `
{
  EVM(network: eth, dataset: archive) {
    Blocks(limit: {count: 10}) {
      Block {
        Number
        Time
      }
    }
  }
}
`;

fetch(url, {
  method: 'POST',
  headers: headers,
  body: JSON.stringify({ query })
})
  .then(res => res.json())
  .then(data => console.log(data));
```

#### Rust (reqwest)
```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = "https://streaming.bitquery.io/graphql";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str("Bearer ory_at_YOUR_TOKEN")?
    );

    let query = r#"
    {
      EVM(network: eth, dataset: archive) {
        Blocks(limit: {count: 10}) {
          Block {
            Number
            Time
          }
        }
      }
    }
    "#;

    let body = json!({ "query": query });

    let response = client
        .post(url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let data = response.text().await?;
    println!("{}", data);

    Ok(())
}
```

---

### WebSocket with API Key

#### JavaScript (graphql-ws)
```javascript
import { createClient } from 'graphql-ws';

const client = createClient({
  url: 'wss://streaming.bitquery.io/graphql?token=ory_at_YOUR_TOKEN',
});

const subscription = `
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks {
      Block {
        Number
        Time
      }
    }
  }
}
`;

client.subscribe(
  { query: subscription },
  {
    next: (data) => console.log('Block:', data),
    error: (error) => console.error('Error:', error),
    complete: () => console.log('Subscription complete'),
  }
);
```

#### Python (gql)
```python
from gql import Client, gql
from gql.transport.websockets import WebsocketsTransport

token = 'ory_at_YOUR_TOKEN'
transport = WebsocketsTransport(
    url=f'wss://streaming.bitquery.io/graphql?token={token}',
    subprotocols=['graphql-transport-ws']
)

client = Client(transport=transport, fetch_schema_from_transport=False)

subscription = gql('''
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks {
      Block {
        Number
        Time
      }
    }
  }
}
''')

for result in client.subscribe(subscription):
    print(f"Block: {result}")
```

---

## Error Codes

### HTTP Error Codes

| Code | Description | Cause | Resolution |
|------|-------------|-------|------------|
| 400 | Bad Request | Invalid query syntax | Check GraphQL query format |
| 401 | Unauthorized | Missing or invalid token | Verify token, regenerate if needed |
| 403 | Forbidden | Insufficient permissions or quota exceeded | Check tier limits, upgrade plan |
| 429 | Too Many Requests | Rate limit exceeded | Slow down requests, wait for reset |
| 500 | Internal Server Error | Server-side issue | Retry after delay, contact support |
| 424 | Failed Dependency | Temporary system issue | Retry in a few minutes |

### GraphQL Error Responses

#### Authentication Error (401)
```json
{
  "errors": [
    {
      "message": "Unauthorized: Invalid or missing access token"
    }
  ]
}
```

#### Rate Limit Error (429)
```json
{
  "errors": [
    {
      "message": "Too Many Sessions: 429",
      "extensions": {
        "code": "RATE_LIMIT_EXCEEDED"
      }
    }
  ]
}
```

#### Quota Exceeded (403)
```json
{
  "errors": [
    {
      "message": "Points quota exceeded. Please upgrade your plan.",
      "extensions": {
        "code": "QUOTA_EXCEEDED",
        "remaining_points": 0
      }
    }
  ]
}
```

### WebSocket Error Codes

| Code | Description | Cause | Resolution |
|------|-------------|-------|------------|
| 1000 | Normal Closure | Clean disconnect | No action needed |
| 1001 | Going Away | Server restart/maintenance | Reconnect |
| 1002 | Protocol Error | Invalid WebSocket protocol | Check protocol (graphql-transport-ws) |
| 1003 | Unsupported Data | Invalid message format | Check JSON format |
| 1006 | Abnormal Closure | Network issue or auth failure | Check token, reconnect |
| 1009 | Message Too Big | Payload exceeds limit (1MB default) | Reduce query size, increase client max_size |

### WebSocket Error Message
```json
{
  "type": "error",
  "payload": {
    "message": "Authentication failed: Invalid token"
  }
}
```

---

## Authentication Security

### Best Practices
1. **Never commit tokens**: Use environment variables
2. **Rotate regularly**: Regenerate tokens periodically
3. **Use HTTPS/WSS**: Always encrypted connections
4. **Limit token scope**: Create separate tokens per application
5. **Monitor usage**: Check account dashboard for unusual activity
6. **Revoke compromised tokens**: Immediately regenerate if leaked

### Token Storage
```bash
# Environment variable (recommended)
export BITQUERY_TOKEN="ory_at_YOUR_TOKEN"

# In code
token = os.getenv('BITQUERY_TOKEN')
```

### Error Handling
```python
import requests

def query_bitquery(query, token):
    url = 'https://streaming.bitquery.io/graphql'
    headers = {
        'Content-Type': 'application/json',
        'Authorization': f'Bearer {token}'
    }

    response = requests.post(url, headers=headers, json={'query': query})

    if response.status_code == 401:
        raise AuthenticationError("Invalid token - please regenerate")
    elif response.status_code == 403:
        raise QuotaExceededError("Points quota exceeded - upgrade plan")
    elif response.status_code == 429:
        raise RateLimitError("Rate limit exceeded - slow down")

    return response.json()
```

---

## Rate Limits (Authentication-based)

Rate limits vary by tier (see `tiers_and_limits.md` for details):

### Free Tier (Developer Plan)
- **10 requests/minute**
- **2 simultaneous WebSocket streams**
- **10,000 points/month**

### Commercial Plan
- **Custom rate limits** (no throttling)
- **Unlimited WebSocket streams**
- **Custom points allocation**

### Monitoring
- **Dashboard**: https://account.bitquery.io
- **Real-time tracking**: Points consumption visible during queries
- **Usage history**: Monthly statistics in account

---

## Additional Notes

1. **No API key rotation API**: Must manually regenerate in dashboard
2. **Token in URL**: Safe for HTTPS/WSS (encrypted)
3. **IDE uses same auth**: GraphQL IDE auto-authenticates with session
4. **No IP whitelisting**: Authentication is token-based only
5. **Multi-user access**: Enterprise plans can have multiple users with separate tokens
