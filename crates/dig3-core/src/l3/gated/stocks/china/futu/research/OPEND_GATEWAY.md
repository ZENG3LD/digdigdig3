# Futu OpenAPI - OpenD Gateway Deep Dive

**Research Date**: 2026-01-26
**Status**: Phase 1 - OpenD Analysis
**Focus**: Installation, configuration, operation, constraints

---

## Executive Summary

**OpenD is mandatory middleware** for Futu API access. Cannot bypass it - all client connections must go through OpenD gateway.

**Key Facts**:
- Acts as TCP proxy between client and Futu servers
- Handles authentication with user credentials
- Manages quote subscriptions and quotas
- Runs locally (127.0.0.1) or on cloud server
- Available for Windows, macOS, Linux (CentOS, Ubuntu)
- Free download, no additional cost

---

## What Is OpenD?

### Purpose

OpenD (Open Daemon) is a **gateway program** that:

1. **Authenticates** to Futu servers with your account credentials
2. **Exposes TCP interface** (default port 11111) for client connections
3. **Translates** between client protocol and Futu server protocol
4. **Manages** subscription quotas and rate limits
5. **Buffers and pushes** real-time market data
6. **Maintains** persistent connection to Futu servers

### Why OpenD Exists

**Architectural reasons**:
- **Abstraction**: Hides Futu server protocol complexity
- **Multi-language support**: TCP interface works with any language
- **Security**: Client doesn't need to know server protocol
- **Performance**: Local gateway reduces latency
- **State management**: Centralizes subscription/quota tracking
- **Authentication**: Separates credential management from client code

### What OpenD Is NOT

❌ **Not optional**: Cannot access Futu API without OpenD
❌ **Not a REST API**: Exposes TCP, not HTTP
❌ **Not open source**: Proprietary Futu software
❌ **Not protocol documentation**: Doesn't document server protocol
❌ **Not a broker**: Just a gateway, not trading infrastructure

---

## Installation

### Download Sources

**Official Website**:
- https://www.futuhk.com/en/support/topic1_464 (Futubull)
- https://www.moomoo.com/support (moomoo)

**Platforms**:
| Platform | Binary | Size | Notes |
|----------|--------|------|-------|
| **Windows** | FutuOpenD.exe | ~50MB | GUI + CLI |
| **macOS** | FutuOpenD.app | ~60MB | GUI + CLI |
| **Linux CentOS** | FutuOpenD | ~45MB | CLI only |
| **Linux Ubuntu** | FutuOpenD | ~45MB | CLI only |

### Installation Steps

#### Windows
```powershell
# 1. Download FutuOpenD_x.x.x_Win.exe
# 2. Run installer
FutuOpenD_x.x.x_Win.exe

# 3. Install to default location
# C:\Program Files\Futu\FutuOpenD\

# 4. Launch via Start Menu or
"C:\Program Files\Futu\FutuOpenD\FutuOpenD.exe"
```

#### macOS
```bash
# 1. Download FutuOpenD_x.x.x_Mac.dmg
# 2. Mount DMG
open FutuOpenD_x.x.x_Mac.dmg

# 3. Drag FutuOpenD.app to Applications
cp -r /Volumes/FutuOpenD/FutuOpenD.app /Applications/

# 4. Launch
open /Applications/FutuOpenD.app
```

#### Linux (Ubuntu/CentOS)
```bash
# 1. Download FutuOpenD_x.x.x_Ubuntu.tar.gz
wget https://download.futunn.com/FutuOpenD_x.x.x_Ubuntu.tar.gz

# 2. Extract
tar -xzf FutuOpenD_x.x.x_Ubuntu.tar.gz
cd FutuOpenD_x.x.x_Ubuntu

# 3. Make executable
chmod +x FutuOpenD

# 4. Run (GUI if X server available)
./FutuOpenD

# Or headless mode
./FutuOpenD -cmd -login_account=your_id -login_pwd=your_pwd
```

### System Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 7+, macOS 10.13+, Ubuntu 18.04+, CentOS 7+ |
| **RAM** | 256 MB minimum, 512 MB recommended |
| **Disk** | 200 MB free space |
| **Network** | Internet connection (HTTPS to Futu servers) |
| **Ports** | Outbound: 443 (HTTPS), Inbound: 11111 (configurable) |

---

## Configuration

### Configuration File

**Location**:
- Windows: `%APPDATA%\Futu\FutuOpenD\FutuOpenD.xml`
- macOS: `~/Library/Application Support/Futu/FutuOpenD/FutuOpenD.xml`
- Linux: `~/.Futu/FutuOpenD/FutuOpenD.xml`

### Example Configuration

```xml
<?xml version="1.0" encoding="UTF-8"?>
<FutuOpenD>
  <!-- Authentication -->
  <login_account>your_email@example.com</login_account>
  <login_pwd>encrypted_password_hash</login_pwd>
  <login_pwd_md5></login_pwd_md5>
  <auto_login>0</auto_login>  <!-- 0=manual, 1=auto -->

  <!-- Network Settings -->
  <api_ip>127.0.0.1</api_ip>  <!-- Listen IP (0.0.0.0 for remote) -->
  <api_port>11111</api_port>  <!-- Listen port -->

  <!-- Trading -->
  <trade_unlock_pwd>encrypted_trade_password</trade_unlock_pwd>
  <trade_unlock_pwd_md5></trade_unlock_pwd_md5>
  <auto_unlock_trade>0</auto_unlock_trade>  <!-- 0=manual, 1=auto -->

  <!-- Encryption (for remote connections) -->
  <rsa_private_key></rsa_private_key>
  <rsa_private_key_file></rsa_private_key_file>

  <!-- Logging -->
  <log_level>info</log_level>  <!-- debug, info, warning, error -->
  <log_dir>./Log</log_dir>

  <!-- API Settings -->
  <push_proto_type>0</push_proto_type>  <!-- 0=Protobuf, 1=JSON -->
  <push_language>en</push_language>     <!-- en, zh-cn, zh-hk -->

  <!-- Security -->
  <enable_websocket>0</enable_websocket>
  <tls_enable>0</tls_enable>
</FutuOpenD>
```

### Key Configuration Options

#### Authentication
```xml
<login_account>email@example.com</login_account>
<!-- Can be: Email, Phone number, Futu ID -->

<auto_login>1</auto_login>
<!-- 0: Prompt for password on startup -->
<!-- 1: Auto-login using saved credentials -->
```

**Security Note**: Passwords are encrypted in config file, but auto-login means credentials stored locally. Use strong filesystem permissions.

#### Network
```xml
<api_ip>127.0.0.1</api_ip>
<!-- 127.0.0.1: Local connections only (secure) -->
<!-- 0.0.0.0: Accept remote connections (requires RSA key) -->

<api_port>11111</api_port>
<!-- Default: 11111 -->
<!-- Change if port conflict exists -->
```

#### Trading
```xml
<trade_unlock_pwd>encrypted_password</trade_unlock_pwd>
<auto_unlock_trade>0</auto_unlock_trade>
<!-- 0: Must call unlock_trade() in code -->
<!-- 1: Auto-unlock on startup (DANGEROUS for production) -->
```

**Recommendation**: Never use `auto_unlock_trade=1` in production. Always require explicit unlock in code.

#### Protocol Format
```xml
<push_proto_type>0</push_proto_type>
<!-- 0: Protocol Buffers (default, faster) -->
<!-- 1: JSON (slower, easier to debug) -->
```

---

## Operation Modes

### 1. Visualization Mode (GUI)

**When to use**: Development, testing, manual monitoring

**Features**:
- Graphical login interface
- Real-time connection status
- Quota usage display
- Active subscription list
- Log viewer
- Manual unlock controls

**Launch**:
```bash
# Windows
FutuOpenD.exe

# macOS
open FutuOpenD.app

# Linux (with X server)
./FutuOpenD
```

**UI Elements**:
- **Login panel**: Enter credentials, 2FA code
- **Status bar**: Shows connection state (Connected/Disconnected)
- **Quota panel**: Real-time/historical quota usage
- **Subscriptions**: List of active subscriptions
- **Connections**: Client connections (IP, port, timestamp)
- **Logs**: Real-time log display

### 2. Command Line Mode (Headless)

**When to use**: Production servers, cloud deployment, automation

**Features**:
- No GUI required
- Auto-login support
- Background daemon mode
- Log file output

**Launch**:
```bash
# Basic headless mode
./FutuOpenD -cmd

# With auto-login
./FutuOpenD -cmd \
  -login_account=your_email@example.com \
  -login_pwd=your_password

# Custom port
./FutuOpenD -cmd -api_port=12000

# Custom log level
./FutuOpenD -cmd -log_level=debug
```

**Command Line Options**:
| Option | Description | Example |
|--------|-------------|---------|
| `-cmd` | Headless mode | `-cmd` |
| `-login_account` | Account (email/phone/ID) | `-login_account=user@example.com` |
| `-login_pwd` | Account password | `-login_pwd=password123` |
| `-api_ip` | Listen IP | `-api_ip=0.0.0.0` |
| `-api_port` | Listen port | `-api_port=11111` |
| `-log_level` | Log verbosity | `-log_level=debug` |
| `-log_dir` | Log directory | `-log_dir=/var/log/opend` |

### 3. Daemon Mode (Linux)

**Systemd service**:

```ini
# /etc/systemd/system/opend.service
[Unit]
Description=Futu OpenD Gateway
After=network.target

[Service]
Type=simple
User=trading
Group=trading
WorkingDirectory=/opt/opend
ExecStart=/opt/opend/FutuOpenD -cmd -login_account=user@example.com -login_pwd=password123
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable opend
sudo systemctl start opend

# Check status
sudo systemctl status opend

# View logs
sudo journalctl -u opend -f
```

---

## Authentication

### Account Types

OpenD authenticates with:
- **Futubull account** (Hong Kong)
- **moomoo account** (US, Singapore, Australia, etc.)
- **Universal account** (Securities + Futures)

### Login Flow

```
┌──────────────┐                              ┌──────────────┐
│    OpenD     │                              │ Futu Servers │
│  (Gateway)   │                              │   (Cloud)    │
└──────┬───────┘                              └──────┬───────┘
       │                                             │
       │  1. Connect (HTTPS)                        │
       ├────────────────────────────────────────────>│
       │                                             │
       │  2. Send credentials (encrypted)           │
       ├────────────────────────────────────────────>│
       │     - Account ID                            │
       │     - Password (hashed)                     │
       │     - Device ID                             │
       │                                             │
       │  3. Verify credentials                      │
       │<────────────────────────────────────────────┤
       │                                             │
       │  4. Request 2FA code (if enabled)          │
       │<────────────────────────────────────────────┤
       │                                             │
       │  5. User enters 2FA code in OpenD          │
       │                                             │
       │  6. Send 2FA code                          │
       ├────────────────────────────────────────────>│
       │                                             │
       │  7. Authentication successful              │
       │<────────────────────────────────────────────┤
       │     - Session token                         │
       │     - Quote authority (LV1/LV2)            │
       │     - Subscription quotas                   │
       │                                             │
       │  8. Start TCP service (port 11111)         │
       │                                             │
       │  9. Maintain heartbeat                     │
       ├<───────────────────────────────────────────>│
       │     (every 60 seconds)                      │
       │                                             │
```

### Two-Factor Authentication

If 2FA is enabled on account:

**Interactive mode** (GUI):
1. OpenD displays 2FA prompt
2. User enters code from authenticator app
3. OpenD sends code to server

**Headless mode** (CLI):
❌ **Cannot use 2FA with `-login_pwd` flag**
- Must use GUI for first login
- Can configure auto-login after initial login
- Or disable 2FA (not recommended)

**Workaround for headless**:
1. Login once via GUI
2. Enable auto-login
3. OpenD saves session token
4. Future launches use saved session (no 2FA prompt)

### Device Authorization

First-time device login requires authorization:

1. Login attempt from new device/IP
2. Futu sends notification to mobile app
3. User approves device in app
4. Login proceeds

**Impacts cloud deployment**: First OpenD launch on cloud server requires mobile app approval.

---

## Connection Management

### Client Connection

**From client code**:
```python
# Python SDK
from futu import *

quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)
# Connection established on first API call
ret, data = quote_ctx.get_global_state()
```

```rust
// Rust (native)
use tokio::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:11111").await?;
// Must implement protocol handshake
```

### Multiple Clients

**OpenD supports multiple simultaneous client connections**:
- Same OpenD can serve multiple scripts/bots
- Shared subscription quota pool
- Shared rate limits
- Each connection has separate state

**Example**:
```
OpenD (127.0.0.1:11111)
  ├─> Client 1 (Python bot)  [Subscribed: US.AAPL]
  ├─> Client 2 (Rust bot)    [Subscribed: HK.00700]
  └─> Client 3 (Backtester)  [No subscriptions]

Total quota used: 2 / 100
```

### Connection Limits

**No explicit limit** on number of client connections, but:
- Quota pool shared across all clients
- Rate limits apply globally
- Too many connections may degrade performance

**Recommendation**: 5-10 concurrent client connections maximum.

### Remote Connections

#### Local Connection (No Encryption)
```xml
<api_ip>127.0.0.1</api_ip>
```
- Only accept connections from localhost
- No encryption (trusted local machine)
- Lowest latency

#### Remote Connection (RSA Encryption)
```xml
<api_ip>0.0.0.0</api_ip>  <!-- Accept all IPs -->
<rsa_private_key>-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA...
-----END RSA PRIVATE KEY-----</rsa_private_key>
```

**Client must use RSA public key**:
```python
# Python SDK
quote_ctx = OpenQuoteContext(
    host='remote.server.com',
    port=11111,
    is_encrypt=True,  # Enable RSA encryption
    security_firm=SecurityFirm.FUTUSECURITIES
)
```

**RSA Key Generation**:
1. Launch OpenD GUI
2. Settings → API → Generate RSA Key Pair
3. OpenD displays public key
4. Save public key for client use
5. Private key stored in config file

---

## Quota Management

### Real-Time Subscription Quota

OpenD tracks quota usage:

**Quota Tiers**:
| Tier | Quota | Based On |
|------|-------|----------|
| Basic | 100 | New account |
| Standard | 300 | Assets >10K HKD or trading volume |
| High Volume | 1,000 | Assets >100K HKD or high volume |
| Premium | 2,000 | Assets >500K HKD or very high volume |

**How OpenD Manages**:
```
Client: subscribe(['US.AAPL'], [SubType.QUOTE, SubType.TICKER])
OpenD:  Check quota: 2 used + 2 new = 4 total (4/100 used)
OpenD:  Forward to Futu servers
OpenD:  Update quota tracking: 4/100

Client: subscribe(['HK.00700'], [SubType.QUOTE])
OpenD:  Check quota: 4 used + 1 new = 5 total (5/100 used)
OpenD:  Forward to Futu servers

Client: unsubscribe(['US.AAPL'], [SubType.QUOTE])
OpenD:  Check unsubscribe wait time (must be >1 minute since subscribe)
OpenD:  If >1 minute: Forward to server, update quota: 4/100
OpenD:  If <1 minute: Reject with "must wait 1 minute" error
```

**Viewing Quota in OpenD**:
- GUI: Quota panel shows real-time usage
- CLI: Check logs or query via API

### Historical K-line Quota

OpenD also tracks historical data quota:
- Each security's first historical request in 30 days = 1 quota
- Subsequent requests within 30 days = 0 quota (cached/free)
- OpenD maintains 30-day tracking

---

## Health Monitoring

### Connection Status

OpenD maintains connection state:

| State | Description | Action |
|-------|-------------|--------|
| **Not Logged In** | OpenD not authenticated | Login required |
| **Logging In** | Authentication in progress | Wait |
| **Logged In** | Connected to Futu servers | Normal operation |
| **Disconnected** | Connection lost | Auto-reconnect attempt |
| **Reconnecting** | Attempting reconnection | Wait |

### Heartbeat

OpenD sends keepalive to Futu servers:
- Interval: Every 60 seconds
- If no response: Mark as disconnected
- Auto-reconnect: Attempt every 10 seconds
- Max retries: Unlimited (keeps trying)

**Client heartbeat**:
- Client can implement own keepalive to OpenD
- Not mandatory (TCP socket stays alive)
- Recommended: Send `KeepAlive` message every 60s

### Monitoring from Client

```python
# Check OpenD status
ret, data = quote_ctx.get_global_state()

if ret == RET_OK:
    print(f"Market: {data['market_state']}")
    print(f"Server time: {data['server_time']}")
    print(f"Login status: OK")
else:
    print(f"OpenD error: {data}")
```

### Log Files

**Location**:
- Windows: `C:\Users\<user>\AppData\Roaming\Futu\FutuOpenD\Log\`
- macOS: `~/Library/Logs/Futu/FutuOpenD/`
- Linux: `~/.Futu/FutuOpenD/Log/` or custom `-log_dir`

**Log files**:
```
FutuOpenD.log         # Main log
FutuOpenD_error.log   # Error log
FutuOpenD_2026-01-26.log  # Daily rotation
```

**Log levels**:
- `DEBUG`: Detailed protocol messages
- `INFO`: Normal operations
- `WARNING`: Non-fatal issues
- `ERROR`: Fatal errors

**Example logs**:
```
[2026-01-26 10:30:15.123] [INFO] Login successful
[2026-01-26 10:30:16.456] [INFO] TCP service started on 127.0.0.1:11111
[2026-01-26 10:30:20.789] [INFO] Client connected from 127.0.0.1:54321
[2026-01-26 10:30:25.012] [INFO] Subscribe: US.AAPL [QUOTE] (quota: 1/100)
[2026-01-26 10:31:00.345] [INFO] Heartbeat sent
[2026-01-26 10:32:00.678] [INFO] Heartbeat sent
```

---

## Performance Characteristics

### Latency

**Client → OpenD → Futu**:
- Local OpenD: 0.5-2ms overhead
- Remote OpenD: +10-50ms (network latency)
- Futu servers: 1-5ms processing

**Total latency**:
- Best case (local OpenD): ~2-7ms
- Cloud OpenD: ~15-60ms
- Advertised order execution: "as fast as 0.0014s" (to exchange)

### Throughput

**Rate limits apply**:
- Standard requests: 60 per 30 seconds
- Trading requests: 15 per 30 seconds per account
- OpenD enforces these limits

**Real-time updates**: No limit (server-push)

### Resource Usage

**OpenD resource consumption**:
- **CPU**: Low (1-2% idle, 5-10% under load)
- **RAM**: 200-400 MB typical
- **Network**: Low bandwidth (real-time data ~10-50 KB/s per symbol)
- **Disk**: Log files (~10-100 MB/day depending on log level)

**Scaling**: Single OpenD can easily handle 5-10 active clients.

---

## Limitations & Constraints

### 1. Cannot Bypass OpenD

**No direct server access**:
- Futu server protocol is proprietary
- Servers only accept connections from OpenD
- Cannot implement pure client without OpenD

**Impact**: Users must install and run OpenD. Cannot distribute "client-only" library.

### 2. User Credentials Required

**Account dependency**:
- Must have Futubull/moomoo account
- Must provide credentials to OpenD
- Cannot use API keys (no impersonal access)

**Impact**: Each user needs their own account and OpenD instance.

### 3. Local Filesystem Access

**Configuration and logs**:
- OpenD reads/writes config file (`FutuOpenD.xml`)
- Creates log files
- Requires filesystem permissions

**Impact**: Serverless or containerized deployments need volume mounts.

### 4. Process Management

**Separate process**:
- Not a library (separate executable)
- Must ensure OpenD is running before client
- Need process monitoring in production

**Impact**: Deployment complexity, health checks, restart logic.

### 5. Port Availability

**Default port 11111**:
- Might conflict with other services
- Need to configure custom port if conflict
- Firewall rules if remote access

### 6. Platform-Specific

**Binary availability**:
- Windows, macOS, Linux (CentOS/Ubuntu)
- No ARM binaries (Raspberry Pi, Apple Silicon M1/M2 might work via Rosetta)
- Cannot compile from source (proprietary)

### 7. 2FA Challenges

**Headless deployment**:
- 2FA requires mobile app approval
- First login must be interactive
- Auto-login workaround (less secure)

---

## Deployment Patterns

### Pattern 1: Development (Local)

```
┌─────────────────────────────────────┐
│  Developer Machine                  │
│                                     │
│  ┌───────────┐      ┌───────────┐  │
│  │  OpenD    │◄────►│  Bot/Code │  │
│  │(GUI Mode) │      │ (Dev/Test)│  │
│  └─────┬─────┘      └───────────┘  │
│        │ 127.0.0.1:11111            │
└────────┼─────────────────────────────┘
         │ Internet
         ▼
   ┌──────────┐
   │Futu Cloud│
   └──────────┘
```

**Characteristics**:
- OpenD runs in GUI mode
- Bot connects to localhost
- Easy debugging (can see OpenD UI)

### Pattern 2: Production (Cloud Server)

```
┌─────────────────────────────────────┐
│  Cloud Server (AWS/GCP/etc.)        │
│                                     │
│  ┌───────────┐      ┌───────────┐  │
│  │  OpenD    │◄────►│Production │  │
│  │(Headless) │      │    Bot    │  │
│  └─────┬─────┘      └───────────┘  │
│        │ 127.0.0.1:11111            │
└────────┼─────────────────────────────┘
         │ Internet
         ▼
   ┌──────────┐
   │Futu Cloud│
   └──────────┘
```

**Setup**:
```bash
# 1. Install OpenD on server
# 2. Configure auto-login
# 3. Run as systemd service
sudo systemctl start opend

# 4. Run bot
./trading_bot
```

### Pattern 3: Docker Container

```yaml
# docker-compose.yml
version: '3.8'
services:
  opend:
    image: custom/futu-opend:latest  # Must build custom image
    volumes:
      - ./config:/root/.Futu/FutuOpenD
      - ./logs:/var/log/opend
    environment:
      - LOGIN_ACCOUNT=${FUTU_ACCOUNT}
      - LOGIN_PASSWORD=${FUTU_PASSWORD}
    ports:
      - "11111:11111"
    restart: always

  trading_bot:
    image: my_trading_bot:latest
    depends_on:
      - opend
    environment:
      - OPEND_HOST=opend
      - OPEND_PORT=11111
```

**Challenges**:
- No official Docker image
- Must build custom Dockerfile
- Volume mounts for config/logs
- Health checks needed

### Pattern 4: Multiple Bots, Single OpenD

```
┌─────────────────────────────────────┐
│  Server                             │
│                                     │
│       ┌───────────┐                 │
│       │   OpenD   │                 │
│       └─────┬─────┘                 │
│             │ 127.0.0.1:11111       │
│    ┌────────┼────────┐              │
│    │        │        │              │
│  ┌─▼──┐  ┌─▼──┐  ┌─▼──┐            │
│  │Bot1│  │Bot2│  │Bot3│            │
│  └────┘  └────┘  └────┘            │
│                                     │
└─────────────────────────────────────┘
```

**Advantages**:
- Single authentication
- Shared quota pool
- Lower resource usage

**Disadvantages**:
- Quota contention
- Single point of failure

---

## Troubleshooting

### OpenD Won't Start

**Symptoms**: OpenD.exe crashes immediately or won't launch

**Causes**:
1. **Port 11111 already in use**
   ```bash
   # Check port
   netstat -ano | findstr :11111  # Windows
   lsof -i :11111                 # Linux/macOS
   ```
   **Solution**: Change port in config or kill process using port

2. **Corrupted config file**
   ```bash
   # Backup and delete config
   mv FutuOpenD.xml FutuOpenD.xml.bak
   # OpenD will create new default config on launch
   ```

3. **Missing dependencies (Linux)**
   ```bash
   # Install dependencies
   sudo apt-get install libssl1.1 libxcb1  # Ubuntu
   ```

### Cannot Login

**Symptoms**: "Login failed" or "Invalid credentials"

**Causes**:
1. **Wrong credentials**: Verify email/password in mobile app first
2. **2FA not entered**: Enter 2FA code if enabled
3. **Device not authorized**: Approve device in mobile app
4. **Account suspended**: Check account status
5. **Network issues**: Check firewall, proxy settings

### Client Cannot Connect

**Symptoms**: "Connection refused" or timeout

**Checks**:
```bash
# 1. Is OpenD running?
ps aux | grep FutuOpenD  # Linux/macOS
tasklist | findstr FutuOpenD  # Windows

# 2. Is port listening?
netstat -ano | findstr :11111  # Windows
lsof -i :11111                 # Linux/macOS

# 3. Can connect via telnet?
telnet 127.0.0.1 11111
# Should connect (Ctrl+] then quit to exit)

# 4. Check firewall
sudo ufw status  # Linux
```

### Subscription Quota Exceeded

**Symptoms**: "No quota" or "Quota limit exceeded"

**Solution**:
```python
# Check current subscriptions
ret, data = quote_ctx.query_subscription()
print(data)

# Unsubscribe unused
quote_ctx.unsubscribe(['OLD.SYMBOL'], [SubType.QUOTE])

# Wait 1 minute
time.sleep(60)

# Subscribe new
quote_ctx.subscribe(['NEW.SYMBOL'], [SubType.QUOTE])
```

---

## Security Best Practices

### 1. Filesystem Permissions

**Config file contains credentials**:
```bash
# Restrict access (Linux/macOS)
chmod 600 ~/.Futu/FutuOpenD/FutuOpenD.xml

# Only owner can read/write
ls -l ~/.Futu/FutuOpenD/FutuOpenD.xml
# -rw------- 1 user user ... FutuOpenD.xml
```

### 2. Network Security

**Local deployment** (preferred):
```xml
<api_ip>127.0.0.1</api_ip>  <!-- Only localhost -->
```

**Remote deployment** (if needed):
```xml
<api_ip>0.0.0.0</api_ip>     <!-- Accept remote -->
<rsa_private_key>...</rsa_private_key>  <!-- Require encryption -->
```

**Firewall rules**:
```bash
# Allow only specific IPs
sudo ufw allow from 10.0.0.5 to any port 11111

# Or use SSH tunnel
ssh -L 11111:localhost:11111 user@remote-server
# Connect to local 11111, tunnels to remote OpenD
```

### 3. Credential Management

**Avoid hardcoding**:
```bash
# Use environment variables
export FUTU_ACCOUNT="user@example.com"
export FUTU_PASSWORD="secure_password"

./FutuOpenD -cmd -login_account=$FUTU_ACCOUNT -login_pwd=$FUTU_PASSWORD
```

**Use secrets manager** (production):
```python
# Load from AWS Secrets Manager, not hardcoded
import boto3

secrets = boto3.client('secretsmanager')
creds = secrets.get_secret_value(SecretId='futu/credentials')
# Use creds to configure OpenD
```

### 4. Auto-Unlock Security

**Never auto-unlock in production**:
```xml
<!-- DON'T DO THIS in production -->
<auto_unlock_trade>1</auto_unlock_trade>

<!-- DO THIS instead -->
<auto_unlock_trade>0</auto_unlock_trade>
```

**Require explicit unlock in code**:
```python
# Bot must explicitly unlock before trading
ret, err = trade_ctx.unlock_trade(password=get_trade_password())
if ret != RET_OK:
    raise Exception(f"Unlock failed: {err}")

# Now can place orders
```

---

## Summary

**OpenD is the mandatory gateway** for Futu API access:

✅ **Advantages**:
- Abstracts protocol complexity
- Multi-language support (TCP interface)
- Manages authentication and state
- Handles reconnection automatically
- Free (no additional cost)

❌ **Challenges**:
- Separate process to manage
- User credentials required
- Cannot bypass (no direct server access)
- Platform-specific binaries
- 2FA complicates headless deployment

**For v5 Integration**:
- OpenD must run before connector works
- Document OpenD installation in setup guide
- Provide docker-compose example
- Consider health check endpoints

---

## Sources

- [OpenD Overview - Futu API Docs](https://openapi.futunn.com/futu-api-doc/en/opend/opend-intro.html)
- [OpenD Download Page](https://www.futuhk.com/en/support/topic1_464)
- [Futu OpenAPI Introduction](https://openapi.futunn.com/futu-api-doc/en/intro/intro.html)
