# Tinkoff Invest API - Authentication

## Public Endpoints

- **Public endpoints exist**: No (all endpoints require authentication)
- **Require authentication**: Yes (all methods)
- **Rate limits without auth**: N/A (cannot access API without token)

**Important**: Unlike many crypto exchanges, Tinkoff Invest API has NO public endpoints. All data access requires a valid authentication token.

## API Key (Token)

### Required For
- **All endpoints**: Yes (100% of API methods)
- **Paid tier only**: No (API is free for all Tinkoff Investments clients)
- **Rate limit increase**: Yes (active traders get higher limits)
- **Specific endpoints**: All methods require token

### How to Obtain

1. **Open Tinkoff Investments account**: https://www.tinkoff.ru/invest/
2. **Navigate to settings**: https://www.tinkoff.ru/invest/settings/
3. **Disable trade confirmation codes** (required for API trading)
4. **Generate token**:
   - Choose environment: Production or Sandbox
   - Choose token type: Readonly, Full-access, or Account-specific
   - Set permissions and expiration (if applicable)
5. **Copy token immediately**: Tokens display only ONCE and cannot be viewed later
6. **Store securely**: Token provides access to trading and portfolio data

**Official token management page**: https://www.tinkoff.ru/invest/settings/

### API Key Format

**gRPC (primary)**:
```
Authorization: Bearer t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```

**WebSocket**:
```
Authorization: Bearer t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```
OR
```
Web-Socket-Protocol: json, t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```

**REST (legacy)**:
```
Authorization: Bearer t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```

**Key characteristics**:
- Prefix: `t.` (indicates Tinkoff token)
- Length: ~50-60 characters
- Character set: Alphanumeric (a-z, A-Z, 0-9)
- Location: Metadata (gRPC) or Header (HTTP/WebSocket)

### Multiple Keys
- **Multiple keys allowed**: Yes (unlimited tokens)
- **Rate limits per key**: Yes (each token has independent limits)
- **Use cases for multiple keys**:
  - Separate tokens for different applications
  - Different permission levels (readonly vs full-access)
  - Account-specific tokens for multi-account management
  - Production vs Sandbox tokens
  - Rotation for security

## Token Types

### 1. Readonly Token

**Purpose**: Read-only access to portfolio and market data

**Permissions**:
- ✅ View portfolio, positions, operations
- ✅ Get market data (candles, order books, trades)
- ✅ List instruments, trading schedules
- ✅ Stream market data and portfolio updates
- ❌ Place orders
- ❌ Cancel orders
- ❌ Modify portfolio

**Use cases**:
- Portfolio monitoring
- Market data analysis
- Trading algorithm backtesting
- Dashboard applications

**Error if used for trading**: 40002 (PERMISSION_DENIED - Insufficient privileges)

### 2. Full-Access Token

**Purpose**: Complete API access including trading

**Permissions**:
- ✅ All readonly permissions
- ✅ Place market/limit orders
- ✅ Cancel orders
- ✅ Place/cancel stop orders
- ✅ Modify existing orders

**Use cases**:
- Algorithmic trading
- Automated order execution
- Full trading bots
- Portfolio rebalancing

**Security note**: Most powerful token type - protect carefully

### 3. Account-Specific Token

**Purpose**: Restrict access to single trading account

**Permissions**:
- Configurable: Readonly or Full-access for ONE account
- All operations limited to specified account_id

**Use cases**:
- Multi-account management with isolation
- Delegated account access
- Per-account trading strategies

**Benefits**:
- Improved security (limits blast radius)
- Account-level permission control

### 4. Sandbox Token

**Purpose**: Testing environment access

**Permissions**:
- ✅ All sandbox methods (OpenSandboxAccount, PostSandboxOrder, etc.)
- ❌ Production methods (will return error)

**Use cases**:
- Strategy testing without real money
- API integration testing
- Educational purposes
- Backtesting with simulated execution

**CRITICAL**: Sandbox token MUST NOT be used with production endpoints. Using sandbox token with production services returns error.

**Endpoint**: `sandbox-invest-public-api.tinkoff.ru:443`

## Token Lifespan & Expiration

### Standard Tokens
- **Lifespan**: 3 months from last use
- **Auto-extension**: Token lifetime resets on each API call
- **Inactive expiration**: Token expires if unused for 3 months

### Granular Tokens (with 2FA)
- **Lifespan**: 90 days (fixed, not extended by usage)
- **2FA requirement**: Enabled by default
- **Security**: Higher security for sensitive operations

### Token Revocation Events

Tokens expire/revoke when:
1. User ceases being Tinkoff client (account closed)
2. All device sessions are terminated
3. Account or card is blocked
4. Token is manually revoked in settings
5. Inactive for 3+ months (standard tokens)
6. 90 days elapsed (granular tokens with 2FA)

### Checking Token Validity
- **Error code**: 40003 (UNAUTHENTICATED - Token missing or inactive)
- **Response**: gRPC status UNAUTHENTICATED
- **Action**: Generate new token at https://www.tinkoff.ru/invest/settings/

## OAuth (if applicable)

**OAuth 2.0**: Not supported

Tinkoff Invest API uses proprietary token-based authentication, NOT OAuth.

## Signature/HMAC (if applicable - rare for data providers)

**HMAC signature**: Not required

Tinkoff Invest API uses simple Bearer token authentication without HMAC signing.

**Why no signature**:
- Token provides sufficient security
- gRPC uses TLS encryption
- Simpler integration for developers
- Token revocation available for compromised credentials

## Authentication Examples

### gRPC with Token (Python)

```python
from tinkoff.invest import Client

TOKEN = "t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890"

with Client(TOKEN) as client:
    accounts = client.users.get_accounts()
    print(accounts)
```

### gRPC with Token (Rust)

```rust
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

let token = "t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890";
let channel = Channel::from_static("https://invest-public-api.tinkoff.ru:443")
    .connect()
    .await?;

let mut request = tonic::Request::new(GetAccountsRequest {});
request.metadata_mut().insert(
    "authorization",
    MetadataValue::from_str(&format!("Bearer {}", token))?
);
```

### WebSocket with Token

```javascript
const WebSocket = require('ws');

const token = "t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890";
const ws = new WebSocket('wss://invest-public-api.tinkoff.ru/ws/', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Web-Socket-Protocol': 'json'
  }
});

ws.on('open', () => {
  console.log('Connected');
  // Subscribe to candles, trades, etc.
});
```

### REST with Token (legacy)

```bash
curl -H "Authorization: Bearer t.aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890" \
  https://invest-public-api.tbank.ru/rest/tinkoff.public.invest.api.contract.v1.UsersService/GetAccounts
```

## Error Codes

### Authentication Errors

| Code | gRPC Status | Description | Resolution |
|------|-------------|-------------|------------|
| 40002 | PERMISSION_DENIED | Insufficient privileges (readonly token used for trading) | Use full-access token |
| 40003 | UNAUTHENTICATED | Token missing or inactive | Generate new token in settings |
| 40004 | PERMISSION_DENIED | Account ineligible for trading (not qualified investor, etc.) | Complete qualification test or check account status |

### Token-Related Errors

| Code | Description | Resolution |
|------|-------------|------------|
| 40003 | Token expired (3 months inactive) | Generate new token |
| 40003 | Token revoked manually | Generate new token |
| 40003 | All sessions terminated | Re-login and generate new token |
| 40003 | Account/card blocked | Resolve account issues with support |
| 40003 | Invalid token format | Check token copied correctly (one-time display) |

## Security Best Practices

### Token Storage
1. **Never commit to git**: Add to .gitignore
2. **Use environment variables**: `export TINKOFF_TOKEN=t.xxx`
3. **Secure storage**: Use secrets manager (AWS Secrets Manager, HashiCorp Vault, etc.)
4. **Encrypt at rest**: If storing tokens in database
5. **Rotate regularly**: Generate new tokens periodically

### Token Usage
1. **Use readonly tokens when possible**: Minimize attack surface
2. **Account-specific tokens**: For multi-account scenarios
3. **Separate tokens per application**: Easier revocation if compromised
4. **Monitor usage**: Check account activity regularly
5. **Revoke unused tokens**: Clean up old tokens in settings

### Network Security
1. **Use TLS/SSL**: Always (gRPC enforces TLS)
2. **Avoid public WiFi**: When using full-access tokens
3. **Firewall rules**: Restrict API access to known IPs (if possible)

### Operational Security
1. **Log token usage**: Track API calls for anomalies
2. **Alert on errors**: Monitor 40002/40003 errors
3. **Implement rate limiting**: Protect against token theft abuse
4. **Test in sandbox first**: Use sandbox tokens for development

## Trading Restrictions

### Order Value Limits
- **Maximum per order**: 6,000,000 RUB
- **Above limit**: Requires additional confirmation (NOT available via API)
- **Error code**: 90003 (Order value too high)
- **Workaround**: Split large orders into smaller chunks

### Qualified Investor Requirements
- **Some instruments**: Require qualified investor status
- **Error code**: 90002 (Only for qualified investors)
- **Resolution**: Complete qualification test in Tinkoff Investments app

### API Trading Restrictions
- **Some instruments**: Forbidden for API trading
- **Error code**: 30052 (Instrument forbidden for trading by API)
- **Check before trading**: Use `GetTradingStatus` method

## Rate Limiting Impact

### Dynamic Rate Limiting
- **Active traders**: Higher limits (more trading fees = more API requests)
- **Low-volume traders**: Standard limits
- **Token-based**: Each token has independent rate limiting

### Rate Limit Errors
- **Code 80001**: Concurrent stream limit exceeded
- **Code 80002**: Request rate exceeded (per minute quota)
- **Resolution**: Implement exponential backoff, increase trading volume for higher limits

## Additional Authentication Info

### Tracking ID
- **Header**: `x-tracking-id` (response metadata)
- **Purpose**: Unique request identifier for technical support
- **Include in support requests**: Helps diagnose authentication issues

### App Name (for SDK developers)
- **Header**: `x-app-name` (optional)
- **Format**: `<github-username>.<repo-name>`
- **Purpose**: Instrumentation statistics
- **Contact**: al.a.volkov@tinkoff.ru for dedicated app registration

### Token Prefix Evolution
- **Classic tokens**: Revoked (no longer valid)
- **Current format**: `t.` prefix
- **Granular tokens**: 90-day lifespan with 2FA

## Sandbox Authentication

### Separate Token Required
- **Sandbox token**: Generated separately from production token
- **Endpoint**: `sandbox-invest-public-api.tinkoff.ru:443`
- **Isolation**: Complete separation from production environment

### Sandbox Account Management
1. Generate sandbox token in settings
2. Call `OpenSandboxAccount` to create test account
3. Use `SandboxPayIn` to add virtual funds
4. Trade using sandbox methods (PostSandboxOrder, etc.)
5. Clean up with `CloseSandboxAccount` when done

### Cross-Environment Protection
- **Sandbox token + production endpoint**: ERROR
- **Production token + sandbox endpoint**: ERROR
- **Prevents accidents**: Cannot accidentally trade real money with sandbox code

## Migration from OpenAPI v1 (Legacy)

If migrating from old REST API:
1. **New token required**: Old API keys not compatible
2. **Generate token**: Use current method (invest/settings)
3. **Update endpoints**: Switch to gRPC or new REST proxy
4. **Update authentication**: Use Bearer token (not API key header)
5. **Deprecation**: Old OpenAPI v1 may be sunset - migrate ASAP

## Summary Checklist

- [x] All endpoints require authentication (no public access)
- [x] Token obtained from https://www.tinkoff.ru/invest/settings/
- [x] Four token types: Readonly, Full-access, Account-specific, Sandbox
- [x] Token format: `Authorization: Bearer t.xxx`
- [x] Token lifespan: 3 months (standard) or 90 days (granular with 2FA)
- [x] Tokens display only once - must copy immediately
- [x] Unlimited tokens allowed per account
- [x] Error 40003 = invalid/expired token
- [x] Error 40002 = insufficient permissions (readonly token for trading)
- [x] Sandbox requires separate token and endpoint
- [x] No OAuth or HMAC - simple Bearer token
- [x] Dynamic rate limiting based on trading activity
- [x] Trading restrictions: 6M RUB max per order via API
