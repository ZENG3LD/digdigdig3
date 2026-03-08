# Futu OpenAPI - Architecture Analysis & V5 Incompatibility

**Research Date**: 2026-01-26
**Status**: Phase 1 - Architecture Incompatibility Analysis
**Conclusion**: **FUNDAMENTALLY INCOMPATIBLE** with v5 REST-based connector pattern

---

## Executive Summary

Futu OpenAPI uses a **custom TCP protocol with Protocol Buffers**, NOT HTTP REST or WebSocket. This architectural choice makes it fundamentally incompatible with the v5 connector pattern, which assumes HTTP-based request/response communication.

**Key Incompatibility**: V5 connectors are designed around:
- HTTP requests (GET/POST)
- REST endpoints with URL paths
- JSON request/response bodies
- Stateless request/response pattern

Futu requires:
- Persistent TCP socket connection
- Protocol Buffer binary serialization
- Stateful session management
- Callback-based push notifications
- Local/remote gateway (OpenD) as mandatory middleware

---

## Current Futu Architecture

### Three-Tier System

```
┌─────────────────┐      TCP + Protobuf       ┌─────────────┐      TCP + Protobuf       ┌──────────────────┐
│  Client App     │ ◄──────────────────────► │   OpenD     │ ◄──────────────────────► │  Futu Servers    │
│  (Your Code)    │      127.0.0.1:11111      │   Gateway   │      (Proprietary)        │  (Cloud)         │
└─────────────────┘                           └─────────────┘                           └──────────────────┘
       │                                             │                                            │
       │                                             │                                            │
    Uses SDK                                  Runs locally                               Exchange connections
  (Python/Java/C#)                           or cloud server                             Market data feeds
                                            Authentication proxy                          Order routing
```

### Architecture Components

#### 1. Client Application Layer
- **Your trading bot/strategy code**
- Uses official SDK (Python, Java, C#, C++, JavaScript) OR
- Implements raw TCP + Protobuf protocol yourself
- Maintains persistent connection to OpenD
- Registers callback handlers for push data

#### 2. OpenD Gateway (Mandatory Middleware)
- **Location**: Local machine (127.0.0.1) or cloud server (remote IP)
- **Function**: Protocol translator and authentication proxy
- **Protocol**: Custom TCP with Protocol Buffers
- **Port**: Default 11111 (configurable)
- **Authentication**:
  - OpenD authenticates to Futu servers with user credentials
  - Client connects to OpenD (local = no auth, remote = RSA key)
- **State Management**:
  - Maintains session with Futu servers
  - Handles reconnection logic
  - Manages quote subscriptions
  - Buffers and pushes real-time data
- **Must be running**: Cannot access Futu API without OpenD

#### 3. Futu Server Layer
- **Location**: Futu's cloud infrastructure
- **Proprietary protocol**: Not publicly documented
- **Functions**:
  - Exchange connectivity (HKEX, NYSE, NASDAQ, etc.)
  - Market data aggregation
  - Order routing to exchanges
  - Account management
  - Real-time data streaming

### Communication Flow

#### Market Data Request Flow
```
1. Client: subscribe(['US.AAPL'], [SubType.QUOTE])
   └─> Serializes to Protobuf message
   └─> Sends via TCP socket to OpenD (127.0.0.1:11111)

2. OpenD: Receives subscription request
   └─> Validates quota (100-2000 depending on tier)
   └─> Forwards to Futu servers via proprietary protocol
   └─> Caches subscription state

3. Futu Server: Processes subscription
   └─> Checks quote authority (LV1/LV2)
   └─> Establishes exchange feed connection
   └─> Begins streaming data to OpenD

4. OpenD: Receives real-time updates
   └─> Buffers data
   └─> Pushes to client via callback

5. Client: Callback handler invoked
   └─> on_recv_rsp() called with Protobuf message
   └─> SDK deserializes to DataFrame/object
   └─> Your code processes update
```

#### Trading Order Flow
```
1. Client: unlock_trade(password='123456')
   └─> TCP message to OpenD
   └─> OpenD validates with Futu server
   └─> Returns success/failure

2. Client: place_order(code='US.AAPL', price=150, qty=100)
   └─> Serializes order params to Protobuf
   └─> TCP send to OpenD

3. OpenD: Forwards to Futu server
   └─> Futu server routes to exchange
   └─> Order acknowledgment returned
   └─> OpenD pushes order status updates to client

4. Client: on_recv_rsp() callback receives order updates
   └─> OrderStatus: Submitted → Working → Filled
   └─> Real-time updates, not polling
```

---

## V5 Connector Pattern (What Futu Doesn't Match)

### V5 Architecture Assumptions

V5 connectors follow this pattern (see KuCoin reference):

```rust
// V5 assumes HTTP REST
pub struct KuCoinConnector {
    http_client: reqwest::Client,  // ❌ Futu: No HTTP
    base_url: String,                // ❌ Futu: No REST URLs
    // ...
}

impl MarketData for KuCoinConnector {
    async fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        // V5 pattern: HTTP GET request
        let url = format!("{}/api/v1/market/orderbook/level1", self.base_url);
        let response = self.http_client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await?;

        // Parse JSON response
        let json: Value = response.json().await?;
        // ...
    }
}
```

### V5 Module Structure (Standard Pattern)

```
exchanges/{exchange}/
├── mod.rs          # Exports
├── endpoints.rs    # ❌ REST URLs - Futu has none
├── auth.rs         # ❌ HMAC signing - Futu uses session auth
├── parser.rs       # ❌ JSON parsing - Futu uses Protobuf
├── connector.rs    # Trait implementations
└── websocket.rs    # ❌ WebSocket (wss://) - Futu uses TCP
```

### Why V5 Assumptions Fail for Futu

| V5 Component | Purpose | Futu Equivalent | Compatible? |
|--------------|---------|-----------------|-------------|
| `endpoints.rs` | Define REST URL paths | No URLs, TCP protocol | ❌ No |
| `auth.rs` | HMAC-SHA256 request signing | Session-based auth via OpenD | ❌ No |
| `parser.rs` | Parse JSON responses | Parse Protocol Buffers | ⚠️ Different format |
| `websocket.rs` | WebSocket streams (wss://) | TCP socket (raw) | ❌ No |
| `reqwest::Client` | HTTP client | TCP TcpStream | ❌ No |
| Stateless requests | Each request independent | Stateful session required | ❌ No |

---

## Why Futu Uses TCP + Protobuf (Not REST)

### Performance Optimization

**Latency Critical**: Futu advertises "order execution as fast as 0.0014s"
- **Persistent TCP**: No connection overhead (REST = new connection per request)
- **Binary Protobuf**: Smaller payloads than JSON (faster serialization)
- **Push-based**: Server pushes updates immediately (no polling latency)
- **No HTTP overhead**: No HTTP headers, cookies, etc.

### Real-Time Market Data

**High-Frequency Streaming**: Stocks can have hundreds of updates per second
- **Callback architecture**: Server pushes updates immediately
- **Subscription model**: Subscribe once, receive updates continuously
- **No polling**: REST would require polling every 100ms (inefficient)

### Multi-Market Coverage

**8 markets in one API**: HK, US, CN, SG, JP, AU, MY, CA
- **Single persistent connection**: Handles all markets
- **Efficient multiplexing**: Multiple subscriptions on one socket
- **Broker-grade infrastructure**: Designed for professional trading

### SDK Abstraction

**Language-agnostic protocol**: TCP + Protobuf works everywhere
- **Native SDKs**: Python, Java, C#, C++, JavaScript
- **Protocol Buffers**: Generate code for any language
- **Consistent interface**: Same protocol across all SDKs

---

## Detailed Protocol Analysis

### Protocol Buffer Format

From official documentation references:

```protobuf
// Example: KeepAlive protocol (heartbeat)
syntax = 'proto2';
package KeepAlive;

option java_package = "com.futu.openapi.pb";
option go_package = "github.com/futuopen/ftapi4go/pb/keepalive";

message C2S {
  optional int64 time = 1;  // Client timestamp
}

message S2C {
  optional int64 time = 1;  // Server timestamp
}

message Request {
  required C2S c2s = 1;
}

message Response {
  required S2C s2c = 1;
}
```

**Common Proto Files** (from documentation):
- `Common.proto` - Shared enumerations and types
- `Qot_Common.proto` - Quote/market data common types
- `Trd_Common.proto` - Trading common types
- Each endpoint has its own .proto file (e.g., `Qot_GetStockQuote.proto`)

### Message Format

Every message has this structure:

```
┌────────────────────────────────────────────────────┐
│ Header (Protocol Buffer)                           │
│  - Message type ID (identifies which .proto)       │
│  - Sequence number (for request/response matching) │
│  - Protocol version                                │
│  - Format (0 = Protobuf, 1 = JSON)                 │
│  - Encryption flag                                 │
│  - Reserved fields                                 │
├────────────────────────────────────────────────────┤
│ Body (Protocol Buffer or JSON)                     │
│  - Request/Response data structure                 │
│  - Depends on message type                         │
└────────────────────────────────────────────────────┘
```

### TCP Communication Pattern

**Connection Lifecycle**:
```python
# Persistent connection (not HTTP request/response)
quote_ctx = OpenQuoteContext(host='127.0.0.1', port=11111)

# Connection established on first API call
ret, data = quote_ctx.get_global_state()

# Subscribe to real-time data
quote_ctx.subscribe(['US.AAPL'], [SubType.QUOTE])

# Register callback for push updates
class QuoteHandler(StockQuoteHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        # Protobuf message pushed from server
        ret_code, data = super().on_recv_rsp(rsp_pb)
        print(data)  # Real-time quote update

quote_ctx.set_handler(QuoteHandler())
quote_ctx.start()  # Start async receiving loop

# Connection stays open until explicitly closed
quote_ctx.close()
```

**Key Difference from REST**:
- REST: Open connection → Send request → Receive response → Close connection
- Futu: Open connection → Keep alive → Send/receive many messages → Close when done

---

## OpenD Gateway Deep Dive

### Why OpenD is Required

**OpenD is NOT optional**. Cannot access Futu API without it.

**Functions OpenD provides**:
1. **Authentication Proxy**: OpenD authenticates with user's Futu credentials
   - Stores account credentials (encrypted)
   - Maintains login session with Futu servers
   - Handles 2FA if enabled

2. **Protocol Translation**: Simplifies client implementation
   - Accepts client connections on TCP port (default 11111)
   - Translates to Futu's proprietary server protocol
   - Handles encryption/decryption

3. **State Management**: Manages complex stateful operations
   - Tracks subscription quotas (100-2000 per tier)
   - Buffers real-time market data
   - Reconnects to Futu servers automatically

4. **Multi-Client Support**: One OpenD can serve multiple clients
   - Multiple scripts can connect to same OpenD
   - Shared subscription quota
   - Centralized authentication

### OpenD Installation & Configuration

**Download**:
- Windows: FutuOpenD.exe
- macOS: FutuOpenD.app
- Linux: FutuOpenD binary (CentOS, Ubuntu)
- Source: https://www.futuhk.com/en/support/topic1_464

**Configuration File** (`FutuOpenD.xml`):
```xml
<FutuOpenD>
  <!-- Authentication -->
  <login_account>your_futu_id</login_account>
  <login_pwd>encrypted_password</login_pwd>
  <auto_login>1</auto_login>

  <!-- Network -->
  <api_ip>127.0.0.1</api_ip>
  <api_port>11111</api_port>

  <!-- Trading -->
  <trade_unlock_pwd>encrypted_trade_password</trade_unlock_pwd>
  <auto_unlock_trade>1</auto_unlock_trade>

  <!-- RSA encryption (for remote connections) -->
  <rsa_private_key>...</rsa_private_key>
</FutuOpenD>
```

**Command Line Launch**:
```bash
# GUI mode
./FutuOpenD

# Headless mode
./FutuOpenD -login_account=your_id -login_pwd=your_pwd
```

### OpenD Deployment Challenges

**For V5 Integration**:
1. **User Dependency**: Every user must install and run OpenD
   - Not a pure library solution
   - Requires separate process management
   - Adds deployment complexity

2. **Local vs Cloud**:
   - **Local**: Best latency, but requires user's machine
   - **Cloud**: 24/7 operation, but adds network latency and security concerns

3. **Configuration Management**:
   - Users must configure credentials
   - Cannot embed in library
   - Credentials security (encrypted but stored locally)

4. **Process Management**:
   - Must ensure OpenD is running before starting bot
   - Handle OpenD crashes/restarts
   - Monitor OpenD connection health

---

## Architectural Incompatibility Details

### 1. No REST Endpoints

**V5 Assumption**: Every exchange has REST URLs
```rust
// V5 pattern (KuCoin example)
pub const BASE_URL: &str = "https://api.kucoin.com";
pub const TICKER_ENDPOINT: &str = "/api/v1/market/orderbook/level1";

// Futu equivalent: NONE
// Cannot do: GET https://api.futu.com/v1/quote?symbol=US.AAPL
// Must do: TCP socket send Protobuf subscribe message to OpenD
```

**Impact**:
- Cannot use `reqwest` for HTTP requests
- Cannot construct REST URLs
- `endpoints.rs` module concept doesn't apply

### 2. No JSON Parsing

**V5 Assumption**: Responses are JSON
```rust
// V5 pattern
#[derive(Deserialize)]
struct TickerResponse {
    data: TickerData,
}
let json: TickerResponse = response.json().await?;
```

**Futu Reality**: Responses are Protocol Buffers
```rust
// Would need
use prost::Message;

#[derive(Message)]
struct QuoteResponse {
    #[prost(message, required, tag = "1")]
    s2c: QuoteS2C,
}

let response = QuoteResponse::decode(bytes)?;
```

**Impact**:
- Need `prost` crate for Protobuf
- Need `.proto` files to generate Rust structs
- Different parsing patterns throughout codebase

### 3. No HMAC Signing

**V5 Assumption**: REST APIs use HMAC-SHA256 signing
```rust
// V5 pattern (auth.rs)
pub fn sign_request(secret: &str, params: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(params.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

**Futu Reality**: Session-based authentication via OpenD
- No per-request signing
- Authentication happens once (OpenD login)
- Client connections use session, not keys

**Impact**:
- `auth.rs` module concept doesn't apply
- No API key/secret pairs
- Different security model

### 4. Stateful, Not Stateless

**V5 Assumption**: Each request is independent
```rust
// V5 pattern - can call any time
let ticker1 = connector.fetch_ticker("BTC-USDT").await?;
let ticker2 = connector.fetch_ticker("ETH-USDT").await?;
// No state between requests
```

**Futu Reality**: Must maintain session and subscription state
```rust
// Futu equivalent - must subscribe first
quote_ctx.subscribe(["US.AAPL"], [SubType.QUOTE]).await?;
// Now subscribed - state maintained

let ticker = quote_ctx.get_stock_quote(["US.AAPL"]).await?;
// This works because we subscribed

// Cannot get quote for unsubscribed symbol
let ticker2 = quote_ctx.get_stock_quote(["US.GOOGL"]).await?;
// ERROR: Not subscribed
```

**Impact**:
- Need state management in connector
- Subscription lifecycle tracking
- Quota management (100-2000 limit)

### 5. Push-Based, Not Pull-Based

**V5 Assumption**: Client requests data (pull)
```rust
// V5 pattern - poll for updates
loop {
    let ticker = connector.fetch_ticker("BTC-USDT").await?;
    process(ticker);
    sleep(1000).await;  // Poll every second
}
```

**Futu Reality**: Server pushes data (callback)
```rust
// Futu equivalent - register callback
struct MyHandler;
impl QuoteHandler for MyHandler {
    fn on_recv_rsp(&mut self, quote: Quote) {
        // Called automatically when quote updates
        process(quote);
    }
}

quote_ctx.set_handler(MyHandler);
quote_ctx.start();  // Start receiving push updates
// No loop needed - callbacks invoked automatically
```

**Impact**:
- Need async callback system
- Different from sync request/response
- Must handle callback lifecycle

### 6. WebSocket vs TCP Socket

**V5 Assumption**: WebSocket for real-time data
```rust
// V5 pattern (websocket.rs)
use tokio_tungstenite::connect_async;

let (ws_stream, _) = connect_async("wss://api.kucoin.com/endpoint").await?;
// Standard WebSocket protocol
```

**Futu Reality**: Raw TCP socket with custom protocol
```rust
// Futu equivalent
use tokio::net::TcpStream;

let stream = TcpStream::connect("127.0.0.1:11111").await?;
// Custom protocol, not WebSocket
// Must implement message framing, heartbeat, etc.
```

**Impact**:
- Cannot use `tungstenite` WebSocket library
- Must implement raw TCP communication
- Custom framing protocol

---

## What Would Be Needed to Make It Work

### Option A: Abandon V5 Pattern Completely

**Create separate module outside v5**:
```
zengeld-terminal/crates/connectors/crates/
├── v4/          # Old REST connectors
├── v5/          # New REST connectors (Binance, KuCoin, etc.)
└── futu/        # ❌ Separate - doesn't fit v5 pattern
    ├── proto/   # Protocol Buffer definitions
    ├── opend/   # OpenD client implementation
    ├── quote/   # Market data context
    ├── trade/   # Trading context
    └── connector.rs  # Bridge to v5 traits (lossy conversion)
```

**Pros**:
- Clean separation of concerns
- Don't force Futu into REST mold
- Can fully utilize Futu's features

**Cons**:
- Not part of unified v5 architecture
- Duplicates some functionality
- Separate maintenance burden

### Option B: Abstract V5 Traits Further

**Make traits protocol-agnostic**:
```rust
// Current v5 trait (HTTP-centric)
pub trait MarketData {
    async fn fetch_ticker(&self, symbol: &str) -> Result<Ticker>;
    //      ^^^^^ Implies request/response
}

// More abstract trait (could work for Futu)
pub trait MarketData {
    // Subscription-based instead of fetch-based
    async fn subscribe_quotes(&mut self, symbols: &[&str]) -> Result<()>;
    async fn get_cached_quote(&self, symbol: &str) -> Result<Option<Ticker>>;
    fn register_quote_handler(&mut self, handler: Box<dyn QuoteHandler>);
}
```

**Pros**:
- Could accommodate both REST and TCP patterns
- Unified trait interface

**Cons**:
- Makes traits more complex
- Breaks existing v5 connectors
- Leaky abstraction (some features don't map)

### Option C: HTTP Bridge/Adapter

**Run local REST server that talks to OpenD**:
```
Your Code → HTTP REST → Bridge Server → TCP Protobuf → OpenD → Futu
            (v5 pattern)  (Rust/Python)  (Custom)
```

**Bridge implementation**:
```rust
// bridge.rs - REST to TCP adapter
#[get("/quote")]
async fn get_quote(symbol: String) -> Json<Ticker> {
    // 1. Connect to OpenD via TCP
    // 2. Send Protobuf subscribe message
    // 3. Wait for callback
    // 4. Convert to JSON
    // 5. Return as REST response
}
```

**Pros**:
- Fits v5 pattern perfectly
- REST abstraction hides complexity

**Cons**:
- Extra latency (HTTP overhead + TCP overhead)
- Another process to manage
- Loses push-based advantages
- Duplicate of OpenD functionality

### Option D: PyO3 Wrapper (Easiest Implementation)

**Use official Python SDK via Rust**:
```rust
use pyo3::prelude::*;

pub struct FutuConnector {
    py_quote_ctx: PyObject,  // Python OpenQuoteContext
}

impl MarketData for FutuConnector {
    async fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        Python::with_gil(|py| {
            let result = self.py_quote_ctx
                .call_method1(py, "get_stock_quote", (vec![symbol],))?;
            // Convert Python DataFrame to Rust Ticker
        })
    }
}
```

**Pros**:
- Leverages battle-tested official SDK
- Minimal protocol implementation
- Access to all Futu features

**Cons**:
- Python runtime dependency
- FFI overhead
- Less "pure Rust" solution

### Option E: Native Rust TCP + Protobuf Client

**Implement from scratch in Rust**:
```rust
// futu/client.rs
use tokio::net::TcpStream;
use prost::Message;

pub struct FutuClient {
    stream: TcpStream,
    proto_definitions: ProtoRegistry,  // Generated from .proto files
}

impl FutuClient {
    async fn subscribe(&mut self, symbols: &[&str]) -> Result<()> {
        // 1. Construct Protobuf subscribe message
        let msg = SubscribeRequest {
            code_list: symbols.iter().map(|s| s.to_string()).collect(),
            subtype_list: vec![SubType::Quote as i32],
        };

        // 2. Encode to bytes
        let mut buf = Vec::new();
        msg.encode(&mut buf)?;

        // 3. Send via TCP with framing
        self.send_message(MSG_TYPE_SUBSCRIBE, &buf).await?;

        // 4. Wait for acknowledgment
        let response = self.recv_message().await?;
        Ok(())
    }
}
```

**Pros**:
- Pure Rust solution
- Full control over implementation
- No external dependencies (Python/OpenD abstraction)

**Cons**:
- High complexity (must reverse-engineer protocol)
- Maintenance burden (Futu updates = you update)
- Still requires OpenD gateway (cannot bypass)
- Protocol is not fully documented

---

## Key Architectural Differences Summary

| Aspect | V5 REST Pattern | Futu TCP Pattern | Compatible? |
|--------|----------------|------------------|-------------|
| **Protocol** | HTTP/HTTPS | Custom TCP | ❌ No |
| **Data Format** | JSON | Protocol Buffers | ❌ No |
| **Authentication** | API Key + HMAC signing | Session via OpenD | ❌ No |
| **Connection Model** | Stateless (new connection per request) | Stateful (persistent connection) | ❌ No |
| **Real-Time Data** | WebSocket (wss://) | TCP callbacks | ❌ No |
| **Request Pattern** | Pull (client requests) | Push (server sends) | ❌ No |
| **Middleware** | None (direct to exchange) | OpenD gateway (mandatory) | ❌ No |
| **URL Endpoints** | REST paths (/api/v1/...) | Message type IDs | ❌ No |
| **Rate Limiting** | HTTP 429 status codes | Error messages in Protobuf | ⚠️ Different |
| **Error Handling** | HTTP status codes | Protocol error codes | ⚠️ Different |
| **Client Library** | reqwest (HTTP) | TcpStream + prost (Protobuf) | ❌ No |

**Verdict**: **0/11 architectural assumptions match**

---

## Fundamental Design Mismatch

### V5 Is Built For REST

The v5 connector architecture makes fundamental assumptions:
1. **HTTP is the transport** → Futu uses TCP
2. **JSON is the format** → Futu uses Protobuf
3. **REST URLs are endpoints** → Futu uses message type IDs
4. **HMAC signing authenticates requests** → Futu uses session auth
5. **Request/response is the pattern** → Futu uses push/callback

### Futu Is Built For Performance

Futu's design priorities:
1. **Latency**: <1ms order execution (TCP faster than HTTP)
2. **Real-time**: Push updates immediately (no polling)
3. **Efficiency**: Binary Protobuf smaller than JSON
4. **Stateful**: Subscriptions persist, not per-request overhead
5. **Broker-grade**: Handles institutional trading volumes

### The Impedance Mismatch

Trying to fit Futu into v5 is like:
- Using a WebSocket library to implement TCP
- Using HTTP request/response to implement streaming
- Using JSON parser to parse binary data
- Using stateless REST to implement stateful sessions

**It's architecturally mismatched at the foundation level.**

---

## Recommendations

### 1. Do NOT Force Into V5

Futu is fundamentally different. Attempting to shoehorn it into v5 will result in:
- Hacky workarounds
- Performance degradation
- Maintenance nightmares
- Loss of Futu's advantages (real-time push, low latency)

### 2. Create Separate Module

Best approach:
```
crates/connectors/crates/
├── v5/          # REST-based connectors
└── futu/        # Separate Futu-specific implementation
    └── (TCP + Protobuf architecture)
```

Still implement MarketData/Trading traits for compatibility, but don't follow v5's internal structure.

### 3. Use PyO3 Wrapper (Pragmatic)

For fastest implementation:
- Use official Python SDK via PyO3
- Provides Rust interface to Futu
- Avoids protocol implementation complexity
- Python dependency acceptable for specialized connector

### 4. Native Rust Client (Future)

If pure Rust is requirement:
- Implement TCP + Protobuf client from scratch
- Still requires OpenD gateway
- High effort, but possible
- Consider long-term maintenance cost

### 5. Document Limitations

If implementing, clearly document:
- Requires OpenD installation
- Different architecture from other connectors
- OpenD must be running before connector works
- User must configure OpenD credentials

---

## Conclusion

**Futu OpenAPI is architecturally incompatible with v5's REST-based pattern.**

This is not a minor difference (like different auth method). This is a fundamental protocol difference:
- V5 assumes HTTP REST
- Futu requires TCP + Protobuf + OpenD gateway

**Recommendation**: Implement as separate module outside v5, using either:
1. **PyO3 wrapper** (easiest, battle-tested)
2. **Native Rust TCP client** (pure Rust, high effort)
3. **Skip Futu** (focus on REST-based brokers)

Do NOT attempt to force Futu into v5's REST-based structure. It will compromise both Futu's performance advantages and v5's clean architecture.

---

## References

- Futu OpenAPI Documentation: https://openapi.futunn.com/futu-api-doc/en/intro/intro.html
- OpenD Gateway Overview: https://openapi.futunn.com/futu-api-doc/en/opend/opend-intro.html
- Python SDK (architecture reference): https://github.com/FutunnOpen/py-futu-api
- Protocol Buffers Rust (prost): https://github.com/tokio-rs/prost
- PyO3 (Rust-Python bridge): https://github.com/PyO3/pyo3
