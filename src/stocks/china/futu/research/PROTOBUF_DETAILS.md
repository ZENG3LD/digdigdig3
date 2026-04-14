# Futu OpenAPI - Protocol Buffers Implementation Details

**Research Date**: 2026-01-26
**Status**: Phase 1 - Protocol Analysis
**Focus**: Protocol Buffer format, message structure, Rust implementation

---

## Executive Summary

Futu OpenAPI uses **Protocol Buffers (proto2 syntax)** for binary serialization over TCP. The protocol consists of:
- **100+ .proto files** for different API endpoints
- **Custom message framing** with header + body structure
- **Three common proto files**: `Common.proto`, `Qot_Common.proto`, `Trd_Common.proto`
- **Rust implementation** requires `prost` crate for code generation

---

## Protocol Buffer Basics

### What Are Protocol Buffers?

Protocol Buffers (Protobuf) is a language-neutral, platform-neutral serialization format developed by Google.

**Key Features**:
- **Binary format**: Smaller than JSON (typically 30-50% size reduction)
- **Strongly typed**: Type safety at compile time
- **Language agnostic**: Generate code for many languages
- **Backward/forward compatible**: Can add fields without breaking old clients
- **Fast serialization**: ~5-10x faster than JSON

**Comparison**:
```
JSON (78 bytes):
{"symbol":"US.AAPL","price":150.25,"volume":1000000,"timestamp":"2026-01-26 10:30:00"}

Protobuf (estimated 35-40 bytes):
Binary representation of same data
```

### Proto2 vs Proto3

Futu uses **proto2 syntax**:

| Feature | proto2 | proto3 |
|---------|--------|--------|
| **Default values** | Explicit (required/optional) | Always optional |
| **Required fields** | ✅ Yes | ❌ No |
| **Extensions** | ✅ Yes | ❌ No |
| **Enum prefix** | ✅ Yes | ❌ No (scoped) |
| **Unknown fields** | Preserved | Preserved (since 3.5) |

**Why proto2**: Older protocol, maintains backward compatibility with legacy systems.

---

## Futu Protocol Structure

### Message Format

Every message sent between client and OpenD has this structure:

```
┌─────────────────────────────────────────────────────────────┐
│ Header Length (4 bytes, big-endian uint32)                  │
├─────────────────────────────────────────────────────────────┤
│ Header (Protocol Buffer - see C2S/S2C structure)            │
│  - Protocol version (uint32)                                 │
│  - Message type ID (uint32) - identifies endpoint           │
│  - Sequence number (uint32) - for request/response matching │
│  - Format type (uint32) - 0 = Protobuf, 1 = JSON            │
│  - Encryption flag (bool)                                    │
│  - SHA1 checksum (bytes)                                     │
│  - Reserved fields                                           │
├─────────────────────────────────────────────────────────────┤
│ Body (Protocol Buffer - endpoint-specific)                  │
│  - Request (C2S - Client to Server) OR                      │
│  - Response (S2C - Server to Client)                        │
└─────────────────────────────────────────────────────────────┘
```

### Message Type IDs

Each API endpoint has a unique message type ID:

| Message Type | ID | Purpose |
|--------------|-----|---------|
| `InitConnect` | 1001 | Initial connection handshake |
| `KeepAlive` | 1004 | Heartbeat/ping |
| `GetGlobalState` | 1002 | Get server state |
| `Qot_Sub` | 3001 | Subscribe to quotes |
| `Qot_GetStockQuote` | 3010 | Get current quote |
| `Qot_GetOrderBook` | 3012 | Get order book |
| `Qot_GetTicker` | 3013 | Get trade ticks |
| `Qot_GetKL` | 3006 | Get candlestick data |
| `Trd_PlaceOrder` | 2202 | Place order |
| `Trd_CancelOrder` | 2205 | Cancel order |
| `Trd_GetAccList` | 2001 | Get account list |
| `Trd_UnlockTrade` | 2005 | Unlock trading |

*(100+ more message types - full list in protocol documentation)*

---

## Common Proto Files

### 1. Common.proto

**Purpose**: Shared enumerations and basic types used across all protocols.

```protobuf
syntax = "proto2";
package Common;

option java_package = "com.futu.openapi.pb";
option go_package = "github.com/futuopen/ftapi4go/pb/common";

// Return codes
enum RetType {
  RetType_Succeed = 0;
  RetType_Failed = -1;
  RetType_TimeOut = -100;
  RetType_DisConnect = -200;
  // ... more error codes
}

// Security market
enum QotMarket {
  QotMarket_Unknown = 0;
  QotMarket_HK_Security = 1;      // Hong Kong
  QotMarket_US_Security = 2;      // US
  QotMarket_CNSH_Security = 3;    // China Shanghai
  QotMarket_CNSZ_Security = 4;    // China Shenzhen
  QotMarket_SG_Security = 5;      // Singapore
  QotMarket_JP_Security = 6;      // Japan
  // ...
}

// Security type
enum SecurityType {
  SecurityType_Unknown = 0;
  SecurityType_Stock = 3;
  SecurityType_ETF = 4;
  SecurityType_Warrant = 5;
  SecurityType_Future = 6;
  SecurityType_Option = 7;
  // ...
}

// Security identifier
message Security {
  required int32 market = 1;  // QotMarket enum
  required string code = 2;   // e.g., "AAPL", "00700"
}

// Price precision
message PriceInfo {
  required double price = 1;
  optional int32 precision = 2;  // Decimal places
}
```

### 2. Qot_Common.proto

**Purpose**: Quote (market data) common types.

```protobuf
syntax = "proto2";
package Qot_Common;

import "Common.proto";

option java_package = "com.futu.openapi.pb";

// Subscription types
enum SubType {
  SubType_None = 0;
  SubType_Basic = 1;         // Basic quote
  SubType_OrderBook = 2;     // Order book depth
  SubType_Ticker = 4;        // Trade tick-by-tick
  SubType_RT = 5;            // Real-time timeline
  SubType_KL_Day = 6;        // Daily candlestick
  SubType_KL_1Min = 7;       // 1-minute candlestick
  SubType_KL_5Min = 8;       // 5-minute candlestick
  SubType_KL_15Min = 9;      // 15-minute candlestick
  SubType_KL_30Min = 10;     // 30-minute candlestick
  SubType_KL_60Min = 11;     // 60-minute candlestick
  SubType_KL_Week = 12;      // Weekly candlestick
  SubType_KL_Month = 13;     // Monthly candlestick
  SubType_Broker = 14;       // Broker queue (HK only)
  // ...
}

// Quote snapshot
message SecuritySnapshot {
  required Common.Security security = 1;
  required double last_price = 2;
  required double open_price = 3;
  required double high_price = 4;
  required double low_price = 5;
  required double prev_close_price = 6;
  required int64 volume = 7;
  required double turnover = 8;
  optional double turnover_rate = 9;
  optional double amplitude = 10;
  optional bool suspension = 11;
  optional string update_time = 12;
  optional double price_spread = 13;
  // ... many more fields (40+ total)
}

// Order book entry
message OrderBookItem {
  required double price = 1;
  required int64 volume = 2;
  required int32 order_count = 3;
}

// Order book
message OrderBook {
  required Common.Security security = 1;
  repeated OrderBookItem bid_list = 2;   // Bid side (buy orders)
  repeated OrderBookItem ask_list = 3;   // Ask side (sell orders)
}

// Trade tick
message Ticker {
  required Common.Security security = 1;
  required string time = 2;              // "HH:mm:ss"
  required int64 sequence = 3;           // Unique tick ID
  required int32 dir = 4;                // 1=Buy, -1=Sell, 0=Unknown
  required double price = 5;
  required int64 volume = 6;
  required double turnover = 7;
}
```

### 3. Trd_Common.proto

**Purpose**: Trading common types.

```protobuf
syntax = "proto2";
package Trd_Common;

import "Common.proto";

option java_package = "com.futu.openapi.pb";

// Trading environment
enum TrdEnv {
  TrdEnv_Real = 0;      // Live trading
  TrdEnv_Simulate = 1;  // Paper trading
}

// Trading market
enum TrdMarket {
  TrdMarket_HK = 1;     // Hong Kong
  TrdMarket_US = 2;     // US
  TrdMarket_CN = 3;     // China A-shares
  TrdMarket_HKCC = 4;   // HK-China Connect
  TrdMarket_Futures = 5; // Futures
  // ...
}

// Order side
enum TrdSide {
  TrdSide_Buy = 1;
  TrdSide_Sell = 2;
  TrdSide_SellShort = 3;
  TrdSide_BuyBack = 4;
}

// Order type
enum OrderType {
  OrderType_Normal = 0;            // Limit order
  OrderType_Market = 1;            // Market order
  OrderType_AbsoluteLimit = 5;     // Limit order (must be best price)
  OrderType_Auction = 6;           // Auction order
  OrderType_AuctionLimit = 7;      // Auction limit order
  OrderType_Special = 8;           // Special order
  OrderType_SpecialLimit = 9;      // Special limit order
  OrderType_Stop = 10;             // Stop loss order
  OrderType_StopLimit = 11;        // Stop limit order
  OrderType_MarketIfTouched = 12;  // MIT order
  OrderType_LimitIfTouched = 13;   // LIT order
  OrderType_TrailingStop = 14;     // Trailing stop
  OrderType_TrailingStopLimit = 15;// Trailing stop limit
}

// Order status
enum OrderStatus {
  OrderStatus_Unsubmitted = 0;     // Not yet submitted
  OrderStatus_Submitted = 1;       // Submitted, waiting
  OrderStatus_Filled_Part = 2;     // Partially filled
  OrderStatus_Filled_All = 3;      // Fully filled
  OrderStatus_Cancelled_Part = 4;  // Partially cancelled
  OrderStatus_Cancelled_All = 5;   // Fully cancelled
  OrderStatus_Failed = 6;          // Failed
  OrderStatus_Disabled = 7;        // Disabled (withdrawn)
  OrderStatus_Deleted = 8;         // Deleted
  OrderStatus_WaitingSubmit = 21;  // Waiting to submit
  OrderStatus_Submitting = 22;     // Submitting
}

// Account info
message TrdAcc {
  required int32 trd_env = 1;     // TrdEnv
  required uint64 acc_id = 2;     // Account ID
  required int32 trd_market = 3;  // TrdMarket
}

// Order info
message Order {
  required int32 trd_side = 1;          // TrdSide
  required int32 order_type = 2;        // OrderType
  required int32 order_status = 3;      // OrderStatus
  required uint64 order_id = 4;         // Unique order ID
  required string order_id_ex = 5;      // Exchange order ID
  required Common.Security security = 6;
  required string name = 7;             // Security name
  required double qty = 8;              // Order quantity
  required double price = 9;            // Order price
  required string create_time = 10;     // "YYYY-MM-DD HH:mm:ss"
  required string update_time = 11;
  optional double filled_qty = 12;      // Filled quantity
  optional double filled_avg_price = 13;// Average fill price
  optional string last_err_msg = 14;    // Error message
  // ... more fields
}
```

---

## Example Endpoint Proto File

### Qot_GetStockQuote.proto

**Purpose**: Get current quote for securities.

```protobuf
syntax = "proto2";
package Qot_GetStockQuote;

import "Common.proto";
import "Qot_Common.proto";

option java_package = "com.futu.openapi.pb";

// Request (Client to Server)
message C2S {
  repeated Common.Security security_list = 1;  // Securities to query
}

message Request {
  required C2S c2s = 1;
}

// Response (Server to Client)
message S2C {
  repeated Qot_Common.SecuritySnapshot snapshot_list = 1;
}

message Response {
  required int32 ret_type = 1;    // RetType: 0 = success, -1 = error
  optional string ret_msg = 2;    // Error message if failed
  optional int32 err_code = 3;    // Error code
  optional S2C s2c = 4;           // Response data if successful
}
```

### Trd_PlaceOrder.proto

**Purpose**: Place trading order.

```protobuf
syntax = "proto2";
package Trd_PlaceOrder;

import "Common.proto";
import "Trd_Common.proto";

option java_package = "com.futu.openapi.pb";

// Request
message C2S {
  required Common.PacketID packet_id = 1;
  required Trd_Common.TrdAcc trd_acc = 2;
  required int32 trd_side = 3;         // TrdSide enum
  required int32 order_type = 4;       // OrderType enum
  required string code = 5;            // Security code
  required double qty = 6;             // Order quantity
  required double price = 7;           // Limit price (0 for market)
  optional bool adjust_price = 8;      // Auto-adjust price
  optional double adjust_side_and_limit = 9;
  optional int32 sec_market = 10;      // Market
  optional string remark = 11;         // Order remark
  optional int32 time_in_force = 12;   // GTC, GTD, DAY
  optional bool fill_outside_rth = 13; // US extended hours
  // ... more optional fields
}

message Request {
  required C2S c2s = 1;
}

// Response
message S2C {
  required uint64 order_id = 1;        // Assigned order ID
}

message Response {
  required int32 ret_type = 1;
  optional string ret_msg = 2;
  optional int32 err_code = 3;
  optional S2C s2c = 4;
}
```

---

## Rust Implementation with Prost

### Prost Crate

**Prost** is the standard Protocol Buffers implementation for Rust.

**Key Features**:
- Generates idiomatic Rust code
- Uses `serde` for serialization
- Async-friendly (works with `tokio`)
- No unsafe code
- Type-safe generated structs

**Alternatives**:
- `rust-protobuf` (older, less idiomatic)
- `protobuf-codegen` (Google's official, less Rust-like)

### Setup (Cargo.toml)

```toml
[dependencies]
prost = "0.13"
prost-types = "0.13"
tokio = { version = "1", features = ["full"] }

[build-dependencies]
prost-build = "0.13"
```

### Code Generation (build.rs)

```rust
// build.rs - runs at compile time
fn main() {
    // Compile .proto files to Rust code
    prost_build::Config::new()
        .out_dir("src/proto")  // Output directory
        .compile_protos(
            &[
                "proto/Common.proto",
                "proto/Qot_Common.proto",
                "proto/Trd_Common.proto",
                "proto/Qot_GetStockQuote.proto",
                "proto/Trd_PlaceOrder.proto",
                // ... include all 100+ proto files
            ],
            &["proto/"],  // Include directory
        )
        .unwrap();
}
```

### Generated Rust Code Example

From `Qot_GetStockQuote.proto`:

```rust
// Generated by prost-build
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2S {
    #[prost(message, repeated, tag = "1")]
    pub security_list: ::prost::alloc::vec::Vec<super::common::Security>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(message, required, tag = "1")]
    pub c2s: ::core::option::Option<C2S>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2C {
    #[prost(message, repeated, tag = "1")]
    pub snapshot_list: ::prost::alloc::vec::Vec<super::qot_common::SecuritySnapshot>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(int32, required, tag = "1")]
    pub ret_type: i32,

    #[prost(string, optional, tag = "2")]
    pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,

    #[prost(int32, optional, tag = "3")]
    pub err_code: ::core::option::Option<i32>,

    #[prost(message, optional, tag = "4")]
    pub s2c: ::core::option::Option<S2C>,
}
```

### Encoding/Decoding Example

```rust
use prost::Message;
use crate::proto::qot_get_stock_quote::*;
use crate::proto::common::*;

// Encode (Rust struct -> bytes)
fn encode_request(symbols: &[&str]) -> Result<Vec<u8>, prost::EncodeError> {
    let security_list = symbols.iter().map(|s| {
        Security {
            market: parse_market(s) as i32,
            code: parse_code(s),
        }
    }).collect();

    let request = Request {
        c2s: Some(C2S { security_list }),
    };

    let mut buf = Vec::new();
    request.encode(&mut buf)?;
    Ok(buf)
}

// Decode (bytes -> Rust struct)
fn decode_response(bytes: &[u8]) -> Result<Response, prost::DecodeError> {
    Response::decode(bytes)
}
```

---

## Message Flow Example

### Subscribe to Quote

**1. Client constructs request**:
```rust
use crate::proto::qot_sub::*;

let request = Request {
    c2s: Some(C2S {
        security_list: vec![
            Security {
                market: QotMarket::QotMarketUsSecurity as i32,
                code: "AAPL".to_string(),
            }
        ],
        sub_type_list: vec![SubType::SubTypeBasic as i32],
        is_sub_or_unsub: true,  // true = subscribe, false = unsubscribe
        is_first_push: true,
        is_subscribe_push: true,
        ..Default::default()
    }),
};

let mut body_bytes = Vec::new();
request.encode(&mut body_bytes)?;
```

**2. Client constructs header**:
```rust
const MSG_TYPE_QOT_SUB: u32 = 3001;

let header = MessageHeader {
    version: 0,
    msg_type: MSG_TYPE_QOT_SUB,
    seq_no: get_next_seq_no(),
    format: 0,  // Protobuf
    encrypted: false,
    sha1: calculate_sha1(&body_bytes),
    reserved: Vec::new(),
};

let mut header_bytes = Vec::new();
header.encode(&mut header_bytes)?;
```

**3. Client sends via TCP**:
```rust
// Send: [header_len][header][body]
stream.write_u32(header_bytes.len() as u32).await?;
stream.write_all(&header_bytes).await?;
stream.write_all(&body_bytes).await?;
```

**4. Server (OpenD) responds**:
```rust
// Read header length
let header_len = stream.read_u32().await?;

// Read header
let mut header_bytes = vec![0u8; header_len as usize];
stream.read_exact(&mut header_bytes).await?;
let header = MessageHeader::decode(&header_bytes[..])?;

// Read body
let body_len = get_body_len_from_header(&header);
let mut body_bytes = vec![0u8; body_len];
stream.read_exact(&mut body_bytes).await?;

// Decode response
let response = qot_sub::Response::decode(&body_bytes[..])?;

// Check result
if response.ret_type == RetType::RetTypeSucceed as i32 {
    println!("Subscribed successfully!");
} else {
    eprintln!("Subscription failed: {}", response.ret_msg.unwrap());
}
```

**5. Server pushes updates (asynchronous)**:
```rust
// Later, when quote updates...
// OpenD pushes update via same connection

let (msg_type, body_bytes) = recv_message(&mut stream).await?;

match msg_type {
    MSG_TYPE_QOT_UPDATE_BASIC => {
        let update = qot_update_basic::Response::decode(&body_bytes)?;
        for snapshot in update.s2c.unwrap().snapshot_list {
            println!("Quote update: {} @ {}",
                snapshot.security.code,
                snapshot.last_price
            );
        }
    }
    _ => {
        warn!("Unknown message type: {}", msg_type);
    }
}
```

---

## Obtaining .proto Files

### From Python SDK

Futu's official Python SDK includes proto files:

```bash
# Install Python SDK
pip install futu-api

# Find proto files
python -c "import futu; print(futu.__path__)"
# Output: ['/path/to/site-packages/futu']

# Proto files located at:
# /path/to/site-packages/futu/common/pb/*.proto
```

**Example structure**:
```
futu/common/pb/
├── Common.proto
├── Qot_Common.proto
├── Trd_Common.proto
├── InitConnect.proto
├── KeepAlive.proto
├── GetGlobalState.proto
├── Qot_Sub.proto
├── Qot_GetStockQuote.proto
├── Qot_GetOrderBook.proto
├── Qot_GetTicker.proto
├── Trd_GetAccList.proto
├── Trd_UnlockTrade.proto
├── Trd_PlaceOrder.proto
├── Trd_CancelOrder.proto
├── ... (100+ files)
```

### Copying for Rust Project

```bash
# 1. Install futu-api
pip install futu-api

# 2. Find installation path
FUTU_PATH=$(python -c "import futu; print(futu.__path__[0])")

# 3. Copy proto files to Rust project
mkdir -p proto
cp $FUTU_PATH/common/pb/*.proto proto/

# 4. Generate Rust code at compile time (via build.rs)
cargo build
```

---

## Handling Required Fields (proto2)

### Required vs Optional

Proto2 has `required`, `optional`, and `repeated` field rules:

```protobuf
message Example {
  required string id = 1;      // Must be present
  optional string name = 2;    // Can be omitted
  repeated int32 values = 3;   // List (0 or more)
}
```

### In Rust (Prost)

Prost handles required fields as `Option<T>`:

```rust
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Example {
    #[prost(string, required, tag = "1")]
    pub id: ::core::option::Option<String>,  // Required -> Option

    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<String>,  // Optional -> Option

    #[prost(int32, repeated, tag = "3")]
    pub values: ::prost::alloc::vec::Vec<i32>,  // Repeated -> Vec
}
```

**Why `Option` for required?**
- Proto2 "required" is encoding-level concept
- Rust safety: Can't guarantee field is present without checking
- Must verify before use:

```rust
let example = decode_example(bytes)?;

// Must unwrap or handle missing required field
let id = example.id.ok_or(Error::MissingRequiredField)?;
println!("ID: {}", id);
```

---

## Message Framing Protocol

### Custom Framing (Not Standard Protobuf)

Standard Protobuf doesn't include message framing. Futu uses custom framing:

```
┌─────────────────────────────────────────────────────────────┐
│ Header Length (4 bytes, big-endian uint32)                  │
├─────────────────────────────────────────────────────────────┤
│ Header (Protobuf-encoded MessageHeader)                     │
├─────────────────────────────────────────────────────────────┤
│ Body (Protobuf-encoded Request/Response)                    │
└─────────────────────────────────────────────────────────────┘
```

### Implementation in Rust

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

async fn send_framed_message(
    stream: &mut tokio::net::TcpStream,
    header: &MessageHeader,
    body: &impl prost::Message,
) -> Result<()> {
    // Encode header
    let mut header_bytes = Vec::new();
    header.encode(&mut header_bytes)?;

    // Encode body
    let mut body_bytes = Vec::new();
    body.encode(&mut body_bytes)?;

    // Write header length (big-endian)
    let header_len = header_bytes.len() as u32;
    stream.write_u32(header_len).await?;

    // Write header
    stream.write_all(&header_bytes).await?;

    // Write body
    stream.write_all(&body_bytes).await?;

    stream.flush().await?;
    Ok(())
}

async fn recv_framed_message(
    stream: &mut tokio::net::TcpStream,
) -> Result<(MessageHeader, Vec<u8>)> {
    // Read header length
    let header_len = stream.read_u32().await?;

    // Read header
    let mut header_bytes = vec![0u8; header_len as usize];
    stream.read_exact(&mut header_bytes).await?;
    let header = MessageHeader::decode(&header_bytes[..])?;

    // Determine body length from header
    let body_len = header.body_len as usize;

    // Read body
    let mut body_bytes = vec![0u8; body_len];
    stream.read_exact(&mut body_bytes).await?;

    Ok((header, body_bytes))
}
```

---

## Advantages of Protobuf for Futu

### 1. Performance

**Smaller payloads**:
```
Quote update (JSON):  ~500 bytes
Quote update (Proto): ~200 bytes  (60% smaller)
```

**Faster serialization**:
- Protobuf: ~5-10x faster than JSON
- Critical for high-frequency updates (stocks update many times per second)

### 2. Type Safety

**Compile-time checks**:
```rust
let request = Request {
    c2s: Some(C2S {
        security_list: vec![...],
        sub_type_list: vec![...],
        is_sub_or_unsub: true,
    }),
};

// Compiler ensures all required fields are present
// Compiler checks types (can't pass string where int expected)
```

### 3. Backward Compatibility

**Can add fields without breaking clients**:
```protobuf
// Version 1
message Quote {
  required string symbol = 1;
  required double price = 2;
}

// Version 2 (added volume field)
message Quote {
  required string symbol = 1;
  required double price = 2;
  optional int64 volume = 3;  // Old clients still work
}
```

### 4. Multi-Language Support

**Same .proto files generate code for all languages**:
- Python: futu-api uses same proto files
- Java: Futu Java SDK uses same proto files
- C++: Futu C++ SDK uses same proto files
- Rust: Can use prost to generate from same proto files

**Consistency**: All SDKs speak identical protocol.

---

## Challenges for Rust Implementation

### 1. Large Number of Files

**100+ .proto files** = 100+ modules in Rust:
- Long compile times (prost code generation)
- Large generated code size (~50,000+ lines)
- Complex module structure

**Mitigation**:
- Only generate proto files for needed endpoints
- Use conditional compilation (`#[cfg(feature = "trading")]`)

### 2. Proto2 Required Fields

**prost uses `Option<T>` for required fields**:
- Must unwrap or handle missing fields
- Not idiomatic Rust (prefer non-Option for truly required)

**Mitigation**:
```rust
// Helper macro to unwrap required fields
macro_rules! required {
    ($field:expr, $name:expr) => {
        $field.ok_or_else(|| Error::MissingRequiredField($name.into()))?
    };
}

// Usage
let symbol = required!(response.s2c.symbol, "symbol");
```

### 3. Protocol Documentation

**Incomplete protocol documentation**:
- Official docs focus on SDK usage, not protocol
- Some message types undocumented
- Must reverse-engineer from Python SDK

**Mitigation**:
- Read Python SDK source code
- Test against OpenD and observe behavior
- Implement subset of features first

### 4. Message Type Registry

**Need to map message type IDs to Response types**:

```rust
// Must implement message type registry
fn decode_response(msg_type: u32, body: &[u8]) -> Result<Box<dyn Any>> {
    match msg_type {
        3010 => Ok(Box::new(qot_get_stock_quote::Response::decode(body)?)),
        3012 => Ok(Box::new(qot_get_order_book::Response::decode(body)?)),
        2202 => Ok(Box::new(trd_place_order::Response::decode(body)?)),
        // ... 100+ cases
        _ => Err(Error::UnknownMessageType(msg_type)),
    }
}
```

**Complex to maintain**: Every new endpoint = new case.

---

## Recommendations

### For PyO3 Approach (Recommended)

**Skip Protobuf implementation entirely**:
- Use Python SDK's proto handling
- Only implement Rust ↔ Python FFI
- Let Python SDK do encoding/decoding

**Pros**: No proto work needed
**Cons**: Python dependency

### For Native Rust Approach

**If implementing from scratch**:

1. **Start small**: Implement 5-10 most important endpoints first
   - Qot_GetStockQuote
   - Qot_Sub
   - Trd_PlaceOrder
   - Trd_CancelOrder
   - Trd_GetAccList

2. **Extract proto files**: Copy from Python SDK installation

3. **Generate Rust code**: Use prost-build in build.rs

4. **Implement framing**: Custom header + body protocol

5. **Test incrementally**: Each endpoint separately

6. **Expand gradually**: Add more endpoints as needed

**Estimated effort**: 2-4 weeks for basic implementation.

---

## Sources

- [Futu Protocol Documentation](https://openapi.futunn.com/futu-api-doc/en/ftapi/protocol.html)
- [Prost (Rust Protocol Buffers)](https://github.com/tokio-rs/prost)
- [Protocol Buffers Documentation](https://protobuf.dev/overview/)
- [Futu Python SDK (proto reference)](https://github.com/FutunnOpen/py-futu-api)
