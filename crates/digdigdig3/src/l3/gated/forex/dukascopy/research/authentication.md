# Dukascopy - Authentication

## Overview

Dukascopy's authentication varies significantly by access method:
- **Binary Downloads**: No authentication required
- **JForex SDK**: Username/password with SDK session
- **FIX API**: FIX Logon message with credentials
- **Third-Party REST/WS**: Configuration file

---

## Public Endpoints

### Historical Tick Data (Binary Downloads)

- **Public endpoints exist**: Yes
- **Require authentication**: No
- **Rate limits without auth**: Yes (undocumented threshold)
- **Access**: Direct HTTP GET

**Endpoints**:
```
https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5
```

**Example**:
```bash
curl -O https://datafeed.dukascopy.com/datafeed/EURUSD/2024/01/15/14h_ticks.bi5
```

**Notes**:
- No API key needed
- Rate limiting applied (connection throttling after bulk downloads)
- Free for personal and educational use
- Commercial redistribution requires agreement

---

## JForex SDK Authentication

### Required For
- All SDK functionality: Yes
- Live trading: Yes (live account)
- Historical data: Yes (demo or live account)
- Real-time data: Yes (demo or live account)
- Rate limit: No explicit limit (fair use)

### How to Obtain

**Demo Account**:
1. Sign up: https://www.dukascopy.com/swiss/english/forex/demo/
2. Receive username and password via email
3. Extended validity demo available (no expiration)
4. Free historical data access

**Live Account**:
1. Open trading account: https://www.dukascopy.com/swiss/english/forex/open-account/
2. Minimum deposit varies by account type
3. Full API access included

### Authentication Process

**Connection via IClient**:
```java
import com.dukascopy.api.*;
import com.dukascopy.api.system.*;

IClient client = ClientFactory.getDefaultInstance();

// Connect and login
client.connect("jnlp://www.dukascopy.com/client/demo.jnlp",
               "username",
               "password");

// Wait for connection
while (client.getState() != IClient.State.CONNECTED) {
    Thread.sleep(100);
}

// Start strategy
client.startStrategy(new YourStrategy());
```

**Historical Data Mode (ITesterClient)**:
```java
ITesterClient client = ClientFactory.getDefaultInstance();

client.setSubscribedInstruments(Set.of(Instrument.EURUSD));
client.setDataInterval(DataLoadingMethod.ALL_TICKS,
                       startTime,
                       endTime);

// No login needed for historical testing
client.startStrategy(new YourStrategy());
```

### Authentication Credentials

**Format**:
- Username: Demo account username (e.g., DEMO123456)
- Password: Account password
- JNLP URL:
  - Demo: `jnlp://www.dukascopy.com/client/demo.jnlp`
  - Live: `jnlp://www.dukascopy.com/client/live.jnlp`

### Session Management

**Session Duration**:
- No explicit timeout for SDK sessions
- Sessions maintained while connection active
- Reconnection handled automatically

**Multiple Sessions**:
- Multiple connections allowed: No (one session per account)
- Multiple strategies: Yes (within same session)

---

## FIX API Authentication

### Required For
- Real-time market data: Yes
- Order management: Yes
- All FIX operations: Yes
- Minimum deposit: USD 100,000

### How to Obtain

1. Open live account with minimum USD 100,000
2. Contact Dukascopy to enable FIX API access
3. Register IP addresses for connection
4. Receive connection details (host, port, credentials)

### FIX Logon Process

**Logon Message (MsgType=A)**:
```
8=FIX.4.4|9=XXX|35=A|49=CLIENT_ID|56=DUKASCOPY|
34=1|52=YYYYMMDD-HH:MM:SS|98=0|108=30|
141=Y|553=USERNAME|554=PASSWORD|10=XXX|
```

**Field Descriptions**:

| Tag | Field Name | Required | Description | Example |
|-----|------------|----------|-------------|---------|
| 8 | BeginString | Yes | FIX version | FIX.4.4 |
| 35 | MsgType | Yes | Message type | A (Logon) |
| 49 | SenderCompID | Yes | Client ID | CLIENT123 |
| 56 | TargetCompID | Yes | Dukascopy ID | DUKASCOPY |
| 34 | MsgSeqNum | Yes | Message sequence | 1 |
| 52 | SendingTime | Yes | Timestamp | YYYYMMDD-HH:MM:SS |
| 98 | EncryptMethod | Yes | Encryption | 0 (None, SSL used) |
| 108 | HeartBtInt | Yes | Heartbeat interval | 30 (seconds) |
| 141 | ResetSeqNumFlag | No | Reset sequence | Y/N |
| 553 | Username | Yes | Account username | YOUR_USERNAME |
| 554 | Password | Yes | Account password | YOUR_PASSWORD |

**Logon Response (Success)**:
```
8=FIX.4.4|9=XXX|35=A|49=DUKASCOPY|56=CLIENT_ID|
34=1|52=YYYYMMDD-HH:MM:SS|98=0|108=30|10=XXX|
```

**Logon Rejection (MsgType=3)**:
```
8=FIX.4.4|9=XXX|35=3|49=DUKASCOPY|56=CLIENT_ID|
45=1|58=Invalid credentials|10=XXX|
```

### Connection Details

**Protocol**: FIX 4.4 over SSL
**Ports**:
- Trading Gateway: 10443 (SSL)
- Data Feed: 9443 (SSL)

**SSL/TLS**:
- SSL encryption: Required
- Certificate: No special certificates needed
- Tunneling: SSL tunneling supported

**IP Restrictions**:
- IP Registration: Required
- Whitelisting: Must register IPs with Dukascopy
- Connection attempts: Max 5 per minute per server per IP

### Session Management

**Heartbeat**:
- Default interval: 30 seconds
- Configurable: Yes (via tag 108 in Logon)
- Timeout: Connection closed if no heartbeat response

**Session Restoration**:
- Max restoration time: 2 hours
- Sequence number reset: Supported (tag 141=Y)
- Message replay: Supported via ResendRequest

**Logout**:
```
8=FIX.4.4|9=XXX|35=5|49=CLIENT_ID|56=DUKASCOPY|
34=XXX|52=YYYYMMDD-HH:MM:SS|10=XXX|
```

---

## Third-Party REST/WebSocket Authentication

**Source**: https://github.com/ismailfer/dukascopy-api-websocket
**Method**: Application configuration file
**Status**: Unofficial

### Configuration File

**File**: `application.properties`

```properties
# Dukascopy credentials
dukascopy.username=YOUR_DEMO_USERNAME
dukascopy.password=YOUR_PASSWORD

# Account type
dukascopy.demo=true  # true for demo, false for live

# Optional settings
dukascopy.jnlpUrl=jnlp://www.dukascopy.com/client/demo.jnlp
```

### Authentication Flow

1. **Server Startup**:
   - Spring Boot reads application.properties
   - Establishes JForex SDK session
   - Validates credentials

2. **Client Connection**:
   - No per-request authentication
   - WebSocket/REST inherit server's JForex session

3. **Failure Handling**:
   - Invalid credentials: Server fails to start
   - Session lost: Server reconnects automatically

### No Request-Level Auth

**REST Endpoints**:
```bash
# No API key headers needed
curl http://localhost:7080/api/v1/history?instID=EURUSD&timeFrame=1MIN&from=1234567890000
```

**WebSocket**:
```javascript
// No auth messages needed
const ws = new WebSocket('ws://localhost:7081/ticker?topOfBook=true');
```

---

## OAuth

**OAuth 2.0**: Not supported
**OAuth 1.0**: Not supported

Dukascopy does not use OAuth for any authentication.

---

## Signature/HMAC

**HMAC-SHA256**: Not required
**Request Signing**: Not used

Dukascopy authentication uses:
- Username/password (JForex SDK, FIX API)
- No request signing
- No API keys
- No HMAC signatures

**Exception**: FIX API uses FIX-standard message authentication, but not HMAC.

---

## API Keys

**API Keys**: Not used by Dukascopy

Authentication methods:
- Account credentials (username/password)
- FIX Logon messages
- SDK session management

**Note**: No separate API keys for REST/WebSocket in official APIs.

---

## Error Codes

### JForex SDK Errors

| Error | Description | Resolution |
|-------|-------------|------------|
| CaptchaException | CAPTCHA challenge required | Complete CAPTCHA in browser |
| AuthenticationException | Invalid credentials | Verify username/password |
| OfflineException | Cannot connect to server | Check network, try later |
| AccountNotSubscribedException | Instrument not subscribed | Subscribe to instrument |

### FIX API Errors

| MsgType | Description | Tag 58 (Text) | Resolution |
|---------|-------------|---------------|------------|
| 3 (Reject) | Invalid message | "Invalid credentials" | Check username/password |
| 3 (Reject) | Sequence error | "MsgSeqNum too low" | Reset sequence or resend |
| 5 (Logout) | Session terminated | "Session expired" | Reconnect with Logon |
| 5 (Logout) | IP not registered | "Unauthorized IP" | Register IP with Dukascopy |

### HTTP Errors (Binary Downloads)

| Code | Description | Resolution |
|------|-------------|------------|
| 404 | File not found | Check symbol, date, hour (data may not exist) |
| 429 | Rate limit exceeded | Wait before retrying |
| 503 | Service unavailable | Server maintenance, retry later |

---

## Security Best Practices

### Credential Storage

**Never hardcode credentials**:
```java
// BAD
client.connect("jnlp://...", "username", "hardcodedpassword");

// GOOD
String username = System.getenv("DUKASCOPY_USERNAME");
String password = System.getenv("DUKASCOPY_PASSWORD");
client.connect("jnlp://...", username, password);
```

### Environment Variables

```bash
export DUKASCOPY_USERNAME="your_username"
export DUKASCOPY_PASSWORD="your_password"
export DUKASCOPY_DEMO="true"
```

### Configuration Files

```properties
# Use external config
dukascopy.username=${DUKASCOPY_USERNAME}
dukascopy.password=${DUKASCOPY_PASSWORD}
```

### FIX API Security

- Register only necessary IPs
- Use dedicated FIX API credentials (separate from web login)
- Monitor connection attempts
- Enable session restoration with sequence number validation
- Use SSL/TLS for all connections

---

## License & Commercial Use

### Personal/Non-Commercial Use
- **License**: Non-exclusive, non-transferable, worldwide
- **Cost**: Free (with demo account)
- **Restrictions**: Personal use only
- **Data**: Historical and real-time access

### Commercial Use
- **Requirement**: Signed supplementary agreement
- **Process**: Contact Dukascopy sales
- **Terms**: Negotiated based on use case
- **Data Redistribution**: Requires separate licensing

**Terms of Use**: https://www.dukascopy.com/swiss/english/home/terms-of-use/

---

## Summary Table

| Access Method | Auth Type | Credentials | Free? | Min Deposit | IP Registration |
|---------------|-----------|-------------|-------|-------------|-----------------|
| Binary Downloads | None | None | Yes | N/A | No |
| JForex SDK (Demo) | Username/Password | Demo account | Yes | N/A | No |
| JForex SDK (Live) | Username/Password | Live account | No | Varies | No |
| FIX API | FIX Logon | Live account | No | $100,000 | Yes |
| Third-Party REST/WS | Config file | Demo/Live account | Yes | N/A | No |

---

## Getting Started Checklist

### For Historical Data (Free)
- [ ] No signup needed for binary downloads
- [ ] OR create demo account for JForex SDK
- [ ] Download JForex SDK from Maven
- [ ] Test connection with demo credentials

### For Real-Time Data (Free)
- [ ] Create demo account
- [ ] Download JForex SDK
- [ ] Implement IFeedListener
- [ ] Subscribe to instruments

### For Trading (Paid)
- [ ] Open live account
- [ ] Fund account (minimum varies)
- [ ] Request API access if needed
- [ ] Use same credentials as web platform

### For FIX API (Professional)
- [ ] Open live account with $100,000+
- [ ] Contact Dukascopy for FIX API access
- [ ] Register connection IPs
- [ ] Receive FIX connection details
- [ ] Implement FIX 4.4 client
