# MOEX - Authentication

## Public Endpoints

- **Public endpoints exist**: Yes
- **Require authentication**: No (for delayed data)
- **Rate limits without auth**: Unknown (not publicly documented, likely conservative)
- **Data delay without auth**: 15 minutes for market data

## Authentication Overview

MOEX ISS provides two authentication paradigms:

1. **ISS REST API**: Mostly public with optional authentication for real-time data
2. **WebAPI/Trading**: OAuth 2.0 with digital signatures (for trading, not covered in this research)

This document focuses on ISS authentication for market data access.

## API Key

### Required For
- **All endpoints**: No (public access available)
- **Paid tier only**: No API key system for ISS
- **Rate limit increase**: Not applicable (no API key system)
- **Specific endpoints**: Real-time data and orderbook require subscription authentication
- **WebSocket real-time**: Yes (credentials required)

### How to Obtain
- **Sign up**: https://passport.moex.com/en/registration
- **Create MOEX Passport account**: Free registration
- **API key management**: Not applicable (uses username/password, not API keys)
- **Free tier includes key**: No API key system; credentials are username/password
- **For paid subscription**: Contact client support manager to enable real-time data access

### Authentication Format (ISS)

**ISS REST API does not use API keys**. Authentication methods:

#### For REST Endpoints (rarely needed)
Most ISS endpoints are public. For restricted endpoints:

```bash
# HTTP Basic Authentication (if required)
curl -u "username:password" https://iss.moex.com/iss/endpoint.json
```

**Header format**:
```
Authorization: Basic base64(username:password)
```

#### For WebSocket (STOMP)
Authentication via STOMP CONNECT frame:

```
CONNECT
accept-version:1.2
host:iss.moex.com
login:your_username
passcode:your_password
heart-beat:10000,10000

^@
```

### Multiple Keys
- **Multiple credentials allowed**: Yes (create multiple Passport accounts)
- **Rate limits per credential**: Unknown (not documented)
- **Use cases**: Separate accounts for different applications or environments

## OAuth (WebAPI - Trading Only)

**Note**: This section describes OAuth for MOEX WebAPI (trading), not ISS (market data).

### OAuth 2.0
- **Supported**: Yes (for WebAPI trading interface, NOT ISS)
- **Grant types**: Client Credentials
- **Scopes**: Defined by subscription and permissions
- **Token endpoint**: https://sso.moex.com/auth/realms/SSO/protocol/openid-connect/token
- **Authorization endpoint**: https://passport.moex.com/

### Flow (WebAPI Trading)

**Prerequisites**:
- `client_id` (application identifier) issued by MOEX personal manager
- `client_secret` (security key) issued by MOEX
- MOEX Passport Token
- Digital signature capability (GOST or RSA)

**Steps**:

1. **Obtain MOEX Passport Token**
```bash
GET https://passport.moex.com/en/registration
# Use Basic authentication with username:password
```

Response:
```
passport_token_here
```

2. **Create Digital Signature**

Using cryptographic software (Validata API for GOST, standard tools for RSA):
```bash
# Create detached signature of Passport Token
signature = sign(passport_token, private_key)
```

3. **Request Access Token**
```bash
POST https://sso.moex.com/auth/realms/SSO/protocol/openid-connect/token

Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials&
client_id=your_client_id&
client_secret=your_client_secret&
passport_token=your_passport_token&
signature=your_signature
```

4. **Response**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

5. **Use Access Token**
```bash
curl -H "Authorization: Bearer your_access_token" \
  https://api.moex.com/webapi/endpoint
```

**Important**: OAuth is for WebAPI (trading), not ISS (market data). ISS uses simpler username/password for real-time subscriptions.

## Signature/HMAC (WebAPI Only)

**ISS does NOT require signatures**. Signatures are only for WebAPI (trading).

### Algorithm (WebAPI)
- HMAC-SHA256 or RSA digital signatures
- GOST (Russian cryptographic standard) also supported

### Components (WebAPI)
- Passport Token from MOEX authentication service
- Private key (RSA or GOST)
- Detached signature format

### Signature Construction (WebAPI)
```
message = passport_token
signature = sign(message, private_key)
# Signature algorithm: RSA-SHA256, GOST R 34.10-2012, or GOST R 34.10-2001
```

### Headers (WebAPI)
```
Authorization: Bearer access_token
Content-Type: application/json
```

**Note**: ISS market data API does not use this signature mechanism.

## Authentication Examples

### ISS REST API (Public Access - No Auth)
```bash
# Get current market data (delayed 15 min, no auth required)
curl https://iss.moex.com/iss/engines/stock/markets/shares/securities.json

# Get historical candles (no auth required)
curl "https://iss.moex.com/iss/engines/stock/markets/shares/boards/TQBR/securities/SBER/candles.json?from=2026-01-20&interval=60"

# Get security info (no auth required)
curl https://iss.moex.com/iss/securities/SBER.json
```

### ISS REST API (With Authentication - For Real-time)
```bash
# If endpoint requires authentication (rare for REST, common for WebSocket)
curl -u "username:password" https://iss.moex.com/iss/endpoint.json

# Or with Authorization header
curl -H "Authorization: Basic $(echo -n 'username:password' | base64)" \
  https://iss.moex.com/iss/endpoint.json
```

### WebSocket with STOMP Authentication
```javascript
// JavaScript example with @stomp/stompjs
const client = new Client({
  brokerURL: 'wss://iss.moex.com/infocx/v3/websocket',
  connectHeaders: {
    login: 'your_username',
    passcode: 'your_password'
  },
  heartbeatIncoming: 10000,
  heartbeatOutgoing: 10000,
  onConnect: (frame) => {
    console.log('Connected:', frame);
    // Subscribe to topics
    client.subscribe('/topic/market.stock.shares.SBER', (message) => {
      console.log('Market data:', JSON.parse(message.body));
    });
  },
  onStompError: (frame) => {
    console.error('STOMP error:', frame);
  }
});

client.activate();
```

```python
# Python example with stomp.py
import stomp

class MyListener(stomp.ConnectionListener):
    def on_message(self, frame):
        print('Received:', frame.body)

    def on_connected(self, frame):
        print('Connected')
        # Subscribe to market data
        conn.subscribe('/topic/market.stock.shares.SBER', id='sub-1')

conn = stomp.Connection([('iss.moex.com', 443)], use_ssl=True)
conn.set_listener('', MyListener())
conn.connect('your_username', 'your_password', wait=True, headers={'heart-beat': '10000,10000'})

# Keep connection alive
while True:
    time.sleep(1)
```

## Error Codes

### HTTP Status Codes (REST API)
| Code | Description | Resolution |
|------|-------------|------------|
| 200 | Success | Data returned successfully |
| 400 | Bad Request | Check request parameters |
| 401 | Unauthorized | Invalid or missing credentials |
| 403 | Forbidden | Insufficient permissions (upgrade subscription) |
| 404 | Not Found | Invalid endpoint or security not found |
| 429 | Too Many Requests | Rate limit exceeded (wait or contact support) |
| 500 | Internal Server Error | Temporary server issue, retry later |
| 503 | Service Unavailable | Maintenance or overload, retry later |

### STOMP Error Frames (WebSocket)
```
ERROR
message:Authentication failed
content-type:text/plain

Invalid username or password
^@
```

Common STOMP errors:
- **Authentication failed**: Invalid credentials
- **Subscription limit exceeded**: Too many subscriptions per connection
- **Invalid destination**: Topic/channel does not exist
- **Connection timeout**: Heart-beat not received, reconnect required

## Subscription Access Levels

### Free Tier (No Subscription)
- **Access**: Delayed data (15 minutes)
- **Authentication**: Not required (optional for user tracking)
- **REST API**: Full access to all endpoints (delayed data)
- **WebSocket**: Delayed data streams
- **Orderbook**: Not available
- **Historical data**: Yes (all history available)
- **Rate limits**: Unknown (likely conservative)

### Paid Tier (Real-time Subscription)
- **Access**: Real-time data (no delay)
- **Authentication**: Required (username/password)
- **REST API**: Real-time data access
- **WebSocket**: Real-time streams
- **Orderbook**: 10x10 for equities/bonds/FX, 5x5 for derivatives
- **Historical data**: Yes (full access)
- **Rate limits**: Higher (specific limits not documented)
- **Cost**: Contact MOEX sales for pricing

### Institutional/Distributor Tiers
- **Access**: Real-time + redistribution rights
- **Authentication**: OAuth 2.0 (WebAPI) or credentials (ISS)
- **Additional features**:
  - Full Order Book product
  - Bulk data downloads
  - Archive access
  - Dedicated support
- **Cost**: Custom pricing, contact MOEX

## Obtaining Subscription

### Steps to Get Real-time Access

1. **Create MOEX Passport Account**
   - Go to: https://passport.moex.com/en/registration
   - Fill registration form
   - Verify email

2. **Contact MOEX Sales**
   - Email: Contact form on https://www.moex.com/s1147
   - Phone: +7 (495) 733-9507
   - Request: Real-time data subscription for ISS

3. **Provide Information**
   - Organization details (if corporate)
   - Use case (trading, analytics, research)
   - Data requirements (which markets, engines)
   - Redistribution intentions (if any)

4. **Sign Agreement**
   - MOEX will provide subscription agreement
   - Review terms and pricing
   - Sign and return

5. **Account Activation**
   - MOEX activates real-time access on your Passport account
   - Credentials remain same (username/password)
   - Real-time data now accessible via REST and WebSocket

6. **Testing**
   - Verify real-time access via WebSocket
   - Confirm no 15-minute delay
   - Test orderbook access (if subscribed)

## Data Usage Restrictions

**Critical**: Even with subscription, data usage is restricted:

- **Personal use**: Allowed for trading and analysis
- **Redistribution**: Prohibited without separate distributor license
- **Commercial services**: Require distributor agreement
- **Display**: Public display may require additional licensing
- **Third-party**: Sharing data with third parties prohibited

**Distributor License Required For**:
- Reselling market data
- Building data products/services
- Public data displays (websites, apps)
- Providing data to clients/users

**Distributor Pricing**: Separate tier, contact MOEX sales

## API Key Management (Not Applicable)

ISS does not use API keys. Instead:

- **Credentials**: Username and password from MOEX Passport
- **Management**: Via MOEX Passport account settings
- **Password reset**: Through Passport portal
- **Account security**: Enable 2FA if available
- **Multiple credentials**: Create multiple Passport accounts

## Rate Limiting (Undocumented)

**Note**: MOEX does not publicly document rate limits for ISS API.

**Estimated based on typical practices**:
- **Free tier**: Likely 60-300 requests per minute
- **Paid tier**: Likely higher, possibly 600-1200 requests per minute
- **WebSocket**: Message rate likely limited but not specified
- **Enforcement**: HTTP 429 error if exceeded

**Best Practices**:
- Implement exponential backoff on errors
- Cache frequently accessed reference data
- Use WebSocket for real-time streams (more efficient than polling)
- Batch requests when possible
- Monitor response headers for rate limit info (if provided)

**If rate limited**:
- Wait before retrying
- Contact MOEX support to request higher limits
- Optimize request patterns
- Consider upgrading subscription

## Security Best Practices

1. **Credentials Protection**
   - Never hardcode passwords in source code
   - Use environment variables or secure vaults
   - Rotate passwords periodically
   - Don't share credentials across applications

2. **Transport Security**
   - Always use HTTPS/WSS (TLS/SSL)
   - Verify SSL certificates
   - Don't disable certificate validation

3. **Error Handling**
   - Don't log passwords in error messages
   - Handle authentication failures gracefully
   - Implement automatic reconnection for WebSocket

4. **Access Control**
   - Limit credential access to authorized personnel
   - Use separate accounts for dev/staging/prod
   - Monitor account activity

5. **Compliance**
   - Adhere to MOEX Terms of Service
   - Respect data usage restrictions
   - Don't redistribute data without license

## Support & Documentation

- **Technical Support**: help@moex.com, +7 (495) 733-9507
- **Sales/Subscription**: https://www.moex.com/s1147
- **Passport Account**: https://passport.moex.com/
- **API Reference**: https://iss.moex.com/iss/reference/
- **Terms of Service**: https://www.moex.com/ (check legal section)

## Summary

- **ISS REST API**: Mostly public, no API keys, username/password for real-time
- **WebSocket**: STOMP authentication with username/password
- **Free tier**: Delayed data (15 min), no auth required
- **Paid tier**: Real-time data, requires subscription and authentication
- **OAuth**: Only for WebAPI (trading), not ISS (market data)
- **Rate limits**: Not documented, implement conservative retry logic
- **Data restrictions**: No redistribution without license
