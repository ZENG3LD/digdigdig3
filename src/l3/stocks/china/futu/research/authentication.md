# Futu OpenAPI - Authentication

## Public Endpoints

- Public endpoints exist: **No**
- Require authentication: **Yes** (all endpoints require OpenD authentication)
- Rate limits without auth: N/A (cannot access without authentication)

**Note**: Unlike typical REST APIs, Futu requires OpenD gateway to be authenticated before any API calls work.

## Authentication Architecture

Futu uses a **two-tier authentication system**:

1. **OpenD ↔ Futu Servers**: OpenD authenticates to Futu with account credentials
2. **Client ↔ OpenD**: Client connects to OpenD (local or remote)

```
┌────────┐         ┌────────┐         ┌─────────────┐
│ Client │ ◄─────► │ OpenD  │ ◄─────► │ Futu Server │
│  SDK   │   TCP   │Gateway │   TCP   │             │
└────────┘         └────────┘         └─────────────┘
    │                   │
    └──► No Auth    ────┘──► Full Auth (credentials)
         (local)              + 2FA
```

## OpenD Authentication (Primary)

### How OpenD Authenticates

**Method 1: Interactive Login (GUI)**
1. Launch OpenD application
2. Login dialog appears
3. Enter credentials:
   - Futu ID / moomoo ID
   - Password
   - Two-factor code (if enabled)
4. OpenD connects to Futu servers
5. Session maintained by OpenD

**Method 2: Configuration File (Automated)**
1. Edit OpenD config file (`FutuOpenD.xml` or similar)
2. Add credentials:
   ```xml
   <FutuOpenD>
     <login_account>your_futu_id</login_account>
     <login_pwd>encrypted_password</login_pwd>
     <auto_login>1</auto_login>
   </FutuOpenD>
   ```
3. OpenD auto-login on startup

**Method 3: Command Line (Headless)**
```bash
./FutuOpenD -login_account=your_id -login_pwd=your_pwd
```

### Required Credentials
- **Account**: Futu ID or moomoo ID (email or phone number)
- **Password**: Account password
- **Two-Factor**: OTP code (if 2FA enabled on account)
- **Device Authorization**: May require device authorization on first login

### Account Types
1. **Futu ID**: Standard Futubull account
2. **moomoo ID**: moomoo app account (US/Singapore/Australia)
3. **Universal Account**: Multi-market account (Securities/Futures)

### Compliance Requirements
- **Account Opening**: Must have opened account via Futubull/moomoo app
- **API Questionnaire**: Must complete compliance questionnaire after first OpenD login
- **API Agreement**: Must accept API usage terms
- **Market Permissions**: Requires specific permissions for each market (HK, US, A-shares, etc.)

## Client to OpenD Authentication

### Local Connection (No Authentication)
When OpenD runs on `127.0.0.1`, no client authentication required:

```python
from futu import *

# Connect to local OpenD (no auth needed)
quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)
```

**Security**: Local connections trusted by default

### Remote Connection (RSA Key Authentication)
When OpenD runs on remote server, RSA key required:

```python
# Connect to remote OpenD (RSA key required)
quote_ctx = OpenQuoteContext(
    host='remote_server_ip',
    port=11111,
    is_encrypt=True,
    security_firm=SecurityFirm.FUTUSECURITIES
)
```

**RSA Key Setup**:
1. Generate RSA key pair in OpenD:
   - OpenD → Settings → API → Generate RSA Key
2. OpenD displays public key
3. Client uses public key to encrypt connection
4. OpenD verifies with private key

**Key Format**: RSA 2048-bit (or configured size)

## Trading Authentication (Unlock Trade)

### Required For
- All trading endpoints: Yes
- Order placement: **Yes**
- Order modification/cancellation: **Yes**
- Query orders/positions: No (can query without unlock)
- Query account funds: No (can query without unlock)
- Market data: No (market data doesn't require unlock)

### How to Unlock

**Method 1: SDK Unlock**
```python
from futu import *

trade_ctx = OpenSecTradeContext(host='127.0.0.1', port=11111)

# Unlock with password
ret, err = trade_ctx.unlock_trade(password='trade_password')
if ret == RET_OK:
    print("Trade unlocked successfully")
else:
    print(f"Unlock failed: {err}")
```

**Method 2: Set in OpenD Config**
```xml
<FutuOpenD>
  <trade_unlock_pwd>encrypted_trade_password</trade_unlock_pwd>
  <auto_unlock_trade>1</auto_unlock_trade>
</FutuOpenD>
```

### Trade Password
- **Set Location**: OpenD → Settings → Trade → Set Password
- **Format**: 6-digit PIN or custom password (depending on configuration)
- **Encryption**: Password encrypted in config file
- **Expiry**: Unlock persists until OpenD restart or manual lock
- **Lock**: Can call `unlock_trade(False)` to lock

### Unlock Duration
- **Session-based**: Unlock persists for OpenD session
- **Restart**: Requires re-unlock after OpenD restart
- **Auto-unlock**: Can configure OpenD to auto-unlock (not recommended for production)

## Market Data Authentication (Quote Rights)

### Quote Authority Levels

Market data access controlled by **quote rights** (购买行情卡):

| Market | LV1 (Basic) | LV2 (Advanced) | Paid Card Required |
|--------|-------------|----------------|-------------------|
| **HK Stocks** | Free | Free (mainland China) / Paid (others) | No (LV1), Yes (LV2 for non-mainland) |
| **US Stocks** | Free basic | N/A | No basic, Yes for Nasdaq TotalView |
| **US Options** | Free (>$3K assets + history) | N/A | No (if qualified) |
| **A-Shares** | Free (mainland only) | N/A | No (if mainland) |
| **HK Futures** | Requires futures account | N/A | Yes |
| **US Futures** | Requires futures account | N/A | Yes (CME) |

### Quote Level Details

**LV1 (Level 1) Quote:**
- Basic price, bid, ask, volume
- 10-level order book (HK)
- Recent trades
- Candlestick data
- Free for most markets (with restrictions)

**LV2 (Level 2) Quote (HK only):**
- All LV1 data
- 40-level order book
- Broker queue (broker IDs)
- Enhanced market depth
- Free for mainland China users, paid for others

**Nasdaq TotalView (US):**
- Full depth order book (60 levels)
- Requires paid subscription card

### How to Purchase Quote Cards

1. **Via App**:
   - Open Futubull/moomoo app
   - Market → Market Data Subscription
   - Select desired markets
   - Purchase subscription

2. **Auto-apply to API**:
   - Quote cards purchased in app automatically apply to OpenD
   - No separate API subscription needed

3. **Pricing** (approximate, check app for current):
   - HK LV2: ~$10-20/month
   - US Nasdaq TotalView: ~$30-50/month
   - Futures data: Varies by exchange

### Checking Quote Authority

```python
# Check global state (includes quote authority)
ret, data = quote_ctx.get_global_state()
if ret == RET_OK:
    print(data)  # Shows market state and authority
```

**No direct API** to check specific quote levels - must attempt subscription:

```python
# Try to subscribe (will fail if no authority)
ret, err = quote_ctx.subscribe(['HK.00700'], [SubType.BROKER])
if ret != RET_OK:
    print(f"No LV2 authority: {err}")
```

## API Key (Not Used)

- API Key format: **Not applicable**
- Futu does NOT use API keys like REST APIs
- Authentication is account-based via OpenD

## OAuth (Not Used)

- OAuth 2.0: **Not supported**
- Futu uses proprietary authentication via OpenD

## Signature/HMAC (Not Used)

- HMAC-SHA256: **Not used**
- Request signing: **Not required**
- Futu uses encrypted TCP connection instead

## Authentication Examples

### Example 1: Basic Quote Context (Local OpenD)

```python
from futu import *

# Connect to local OpenD (already authenticated)
quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)

try:
    # Check connection
    ret, data = quote_ctx.get_global_state()
    if ret == RET_OK:
        print("Connected successfully")
        print(data)
    else:
        print(f"Connection failed: {data}")

    # Subscribe to quotes
    ret, err = quote_ctx.subscribe(['US.AAPL'], [SubType.QUOTE])
    if ret == RET_OK:
        print("Subscribed successfully")
    else:
        print(f"Subscription failed: {err}")

finally:
    quote_ctx.close()
```

### Example 2: Trade Context with Unlock (Local OpenD)

```python
from futu import *

# Connect to local OpenD
trade_ctx = OpenSecTradeContext(host='127.0.0.1', port=11111)

try:
    # Unlock trading
    ret, err = trade_ctx.unlock_trade(password='123456')  # Your trade password
    if ret == RET_OK:
        print("Trade unlocked")
    else:
        print(f"Unlock failed: {err}")
        exit(1)

    # Get account list
    ret, data = trade_ctx.get_acc_list()
    if ret == RET_OK:
        print("Accounts:")
        print(data)
    else:
        print(f"Failed to get accounts: {data}")

    # Place order (requires unlock)
    ret, data = trade_ctx.place_order(
        price=150.00,
        qty=100,
        code='US.AAPL',
        trd_side=TrdSide.BUY,
        order_type=OrderType.NORMAL,
        trd_env=TrdEnv.SIMULATE  # Paper trading
    )
    if ret == RET_OK:
        print("Order placed:")
        print(data)
    else:
        print(f"Order failed: {data}")

finally:
    trade_ctx.close()
```

### Example 3: Remote OpenD with Encryption

```python
from futu import *

# Connect to remote OpenD (RSA encryption)
quote_ctx = OpenQuoteContext(
    host='remote.server.com',
    port=11111,
    is_encrypt=True,  # Enable encryption
    security_firm=SecurityFirm.FUTUSECURITIES
)

# RSA key must be configured in OpenD on remote server
# Client SDK handles encryption automatically

try:
    ret, data = quote_ctx.get_global_state()
    if ret == RET_OK:
        print("Connected to remote OpenD")
    else:
        print(f"Connection failed: {data}")

finally:
    quote_ctx.close()
```

### Example 4: Paper Trading (Simulated Account)

```python
from futu import *

# Paper trading uses same authentication, just different TrdEnv

trade_ctx = OpenSecTradeContext(host='127.0.0.1', port=11111)

try:
    # Unlock (same password works for both real and simulated)
    ret, err = trade_ctx.unlock_trade(password='123456')
    if ret != RET_OK:
        print(f"Unlock failed: {err}")
        exit(1)

    # Place simulated order (TrdEnv.SIMULATE)
    ret, data = trade_ctx.place_order(
        price=150.00,
        qty=100,
        code='US.AAPL',
        trd_side=TrdSide.BUY,
        trd_env=TrdEnv.SIMULATE  # Simulated account
    )
    if ret == RET_OK:
        print("Simulated order placed")
    else:
        print(f"Order failed: {data}")

finally:
    trade_ctx.close()
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| RET_OK (0) | Success | - |
| RET_ERROR (-1) | Generic error | Check error message |
| -1, "not login" | OpenD not authenticated | Check OpenD login status, re-login |
| -1, "unlock trade fail" | Wrong trade password | Verify password, check OpenD settings |
| -1, "no trade permission" | Account lacks trading permission | Enable trading in account settings |
| -1, "no authority" | Insufficient quote rights | Purchase required quote card |
| -1, "subscribe failed: no authority" | No quote rights for market | Purchase quote subscription |
| -1, "freq limit" | Rate limit exceeded | Wait and retry (see tiers_and_limits.md) |
| -1, "account not exist" | Account ID not found | Check account list, verify acc_id |
| -1, "unlock expired" | Trade unlock expired | Re-unlock trade |
| -1, "device not authorized" | Device not authorized | Authorize device in Futubull app |

## Security Best Practices

### 1. OpenD Security
- **Local OpenD**: Run on same machine as client for maximum security
- **Remote OpenD**: Always use `is_encrypt=True` for remote connections
- **Firewall**: Restrict OpenD port to trusted IPs only
- **Auto-login**: Disable in production for sensitive accounts

### 2. Trade Password
- **Strong Password**: Use strong trade password (not same as account password)
- **No Hardcode**: Don't hardcode trade password in source code
- **Environment Variable**: Use environment variables or secure config
- **Lock After Use**: Call `unlock_trade(False)` when trading done

### 3. Account Security
- **Two-Factor**: Enable 2FA on Futu account
- **Device Authorization**: Enable device authorization
- **API Questionnaire**: Complete compliance honestly
- **Monitor Activity**: Regularly check API activity in app

### 4. Paper Trading First
- **Test First**: Always test strategies with `TrdEnv.SIMULATE` first
- **Verify Logic**: Ensure order logic correct before live trading
- **Risk Management**: Implement stop-loss and position sizing

### 5. Connection Management
- **Close Contexts**: Always call `close()` when done
- **Exception Handling**: Handle connection errors gracefully
- **Reconnect Logic**: Implement automatic reconnection
- **State Checking**: Periodically check `get_global_state()`

## Troubleshooting

### "not login" Error
**Cause**: OpenD not authenticated to Futu servers
**Solution**:
1. Check OpenD GUI - is it logged in?
2. If not, login manually or configure auto-login
3. Check network connectivity to Futu servers
4. Check OpenD logs for authentication errors

### "unlock trade fail" Error
**Cause**: Wrong trade password or unlock disabled
**Solution**:
1. Verify trade password in OpenD settings
2. Try unlocking manually in OpenD GUI first
3. Check if trade unlock is disabled in settings
4. Ensure account has trading permissions

### "no authority" Error
**Cause**: Insufficient quote rights for requested market/data
**Solution**:
1. Check which market data requires paid subscription
2. Purchase required quote card in Futubull/moomoo app
3. Wait for subscription to activate (usually instant)
4. Restart OpenD if subscription doesn't apply

### "device not authorized" Error
**Cause**: First-time device connection requires authorization
**Solution**:
1. Open Futubull/moomoo app
2. Go to Settings → Account Security → Device Management
3. Approve the new device
4. Retry OpenD login

### Connection Timeout
**Cause**: Cannot connect to OpenD
**Solution**:
1. Verify OpenD is running (check process list)
2. Verify OpenD port (default 11111, check config)
3. Check firewall rules
4. Try `telnet 127.0.0.1 11111` to test TCP connectivity

## Summary

- **Two-Tier Auth**: OpenD authenticates to Futu, client connects to OpenD
- **No API Keys**: Uses account credentials, not API keys
- **Trade Unlock**: Required for order placement
- **Quote Rights**: Market data requires appropriate subscription levels
- **Local vs Remote**: Local = no client auth, Remote = RSA encryption
- **Paper Trading**: Same auth flow, just different `TrdEnv` parameter
