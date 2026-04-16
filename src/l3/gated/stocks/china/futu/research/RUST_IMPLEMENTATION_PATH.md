# Futu OpenAPI - Native Rust Implementation Path

**Research Date**: 2026-01-26
**Status**: Phase 1 - Implementation Roadmap
**Approach**: Pure Rust TCP + Protobuf client (no Python dependency)

---

## Executive Summary

**Effort**: 20-25 working days for basic implementation
**Complexity**: High
**Maintenance**: High
**Result**: Pure Rust client communicating with OpenD via TCP + Protocol Buffers

**Still requires OpenD gateway** - cannot bypass, but eliminates Python dependency.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────┐
│  Pure Rust Implementation                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  FutuConnector (v5 trait implementations)           │  │
│  │  - impl MarketData                                   │  │
│  │  - impl Trading                                      │  │
│  └─────────────────────┬────────────────────────────────┘  │
│  ┌─────────────────────▼────────────────────────────────┐  │
│  │  FutuClient (TCP + Protobuf)                         │  │
│  │  - TcpStream (tokio)                                 │  │
│  │  - Message framing                                   │  │
│  │  - Encode/decode (prost)                             │  │
│  │  - Subscription management                           │  │
│  │  - Callback handlers                                 │  │
│  └─────────────────────┬────────────────────────────────┘  │
│  ┌─────────────────────▼────────────────────────────────┐  │
│  │  Proto Layer (generated from .proto files)          │  │
│  │  - Common types                                      │  │
│  │  - Qot_* (market data)                               │  │
│  │  - Trd_* (trading)                                   │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬───────────────────────────────────┘
                         │ TCP + Protobuf
                    ┌────▼─────┐
                    │  OpenD   │  (Still required)
                    └──────────┘
```

---

## Required Rust Crates

### Core Dependencies

```toml
[dependencies]
# Async runtime
tokio = { version = "1.45", features = ["full"] }

# Protocol Buffers
prost = "0.13"
prost-types = "0.13"

# Serialization helpers
bytes = "1.8"
byteorder = "1.5"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Async utilities
futures = "0.3"
async-trait = "0.1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

[build-dependencies]
# Code generation from .proto files
prost-build = "0.13"
```

### Project Structure

```
futu/
├── Cargo.toml
├── build.rs                # Protobuf code generation
├── proto/                  # .proto files (copied from Python SDK)
│   ├── Common.proto
│   ├── Qot_Common.proto
│   ├── Trd_Common.proto
│   ├── Qot_GetStockQuote.proto
│   ├── Trd_PlaceOrder.proto
│   └── ... (100+ files)
├── src/
│   ├── lib.rs             # Main library exports
│   ├── client.rs          # TCP client implementation
│   ├── protocol.rs        # Message framing/protocol
│   ├── messages.rs        # Message type registry
│   ├── subscription.rs    # Subscription manager
│   ├── callbacks.rs       # Push callback system
│   ├── connector.rs       # v5 trait implementations
│   ├── parser.rs          # Proto → Rust type conversions
│   ├── error.rs           # Error types
│   └── proto/             # Generated protobuf code (build output)
│       ├── mod.rs
│       ├── common.rs
│       ├── qot_common.rs
│       ├── trd_common.rs
│       └── ... (generated)
└── examples/
    ├── quote_basic.rs
    └── trading_basic.rs
```

---

## Implementation Phases

### Phase 1: Setup & Code Generation (2 days)

#### Task 1.1: Extract .proto Files

```bash
# 1. Install Futu Python SDK
pip install futu-api

# 2. Find proto files
FUTU_PATH=$(python -c "import futu; print(futu.__path__[0])")
echo $FUTU_PATH/common/pb

# 3. Copy to Rust project
mkdir -p proto
cp $FUTU_PATH/common/pb/*.proto proto/

# 4. Verify files
ls -1 proto/ | wc -l
# Should be 100+ .proto files
```

#### Task 1.2: Configure prost-build

```rust
// build.rs
use std::io::Result;

fn main() -> Result<()> {
    // Compile all .proto files
    let proto_files: Vec<_> = std::fs::read_dir("proto")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "proto" {
                Some(path.to_str()?.to_string())
            } else {
                None
            }
        })
        .collect();

    println!("cargo:rerun-if-changed=proto");

    prost_build::Config::new()
        .out_dir("src/proto")
        .compile_protos(&proto_files, &["proto/"])?;

    Ok(())
}
```

#### Task 1.3: Generate and Organize

```bash
# Build project to generate proto code
cargo build

# Check generated code
ls -lh src/proto/
# Should see many .rs files (100+)
```

**Expected output**: ~50,000-100,000 lines of generated Rust code

### Phase 2: TCP Client & Protocol Layer (3 days)

#### Task 2.1: Message Framing

```rust
// src/protocol.rs
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use prost::Message;
use byteorder::{BigEndian, ByteOrder};

pub struct MessageHeader {
    pub version: u32,
    pub msg_type: u32,
    pub seq_no: u32,
    pub format: u32,  // 0 = Protobuf, 1 = JSON
    pub body_len: u32,
    // ... other fields
}

pub async fn send_message<T: Message>(
    stream: &mut TcpStream,
    msg_type: u32,
    seq_no: u32,
    body: &T,
) -> Result<()> {
    // 1. Encode body
    let mut body_bytes = Vec::new();
    body.encode(&mut body_bytes)?;

    // 2. Construct header
    let header = MessageHeader {
        version: 0,
        msg_type,
        seq_no,
        format: 0,
        body_len: body_bytes.len() as u32,
    };

    // 3. Encode header
    let mut header_bytes = Vec::new();
    header.encode(&mut header_bytes)?;

    // 4. Send: [header_len][header][body]
    let mut len_buf = [0u8; 4];
    BigEndian::write_u32(&mut len_buf, header_bytes.len() as u32);
    stream.write_all(&len_buf).await?;
    stream.write_all(&header_bytes).await?;
    stream.write_all(&body_bytes).await?;
    stream.flush().await?;

    Ok(())
}

pub async fn recv_message(
    stream: &mut TcpStream,
) -> Result<(MessageHeader, Vec<u8>)> {
    // 1. Read header length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let header_len = BigEndian::read_u32(&len_buf);

    // 2. Read header
    let mut header_bytes = vec![0u8; header_len as usize];
    stream.read_exact(&mut header_bytes).await?;
    let header = MessageHeader::decode(&header_bytes[..])?;

    // 3. Read body
    let mut body_bytes = vec![0u8; header.body_len as usize];
    stream.read_exact(&mut body_bytes).await?;

    Ok((header, body_bytes))
}
```

#### Task 2.2: Connection Management

```rust
// src/client.rs
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

pub struct FutuClient {
    stream: Arc<Mutex<TcpStream>>,
    seq_no: Arc<Mutex<u32>>,
    response_channels: Arc<Mutex<HashMap<u32, oneshot::Sender<Response>>>>,
    callback_tx: mpsc::UnboundedSender<PushMessage>,
}

impl FutuClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host, port)).await?;

        let (callback_tx, callback_rx) = mpsc::unbounded_channel();

        let client = Self {
            stream: Arc::new(Mutex::new(stream)),
            seq_no: Arc::new(Mutex::new(1)),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
            callback_tx,
        };

        // Spawn receive loop
        tokio::spawn(client.clone().receive_loop());

        Ok(client)
    }

    async fn next_seq_no(&self) -> u32 {
        let mut seq = self.seq_no.lock().await;
        let current = *seq;
        *seq += 1;
        current
    }

    pub async fn send_request<Req, Res>(
        &self,
        msg_type: u32,
        request: Req,
    ) -> Result<Res>
    where
        Req: Message,
        Res: Message + Default,
    {
        let seq_no = self.next_seq_no().await;

        // Create response channel
        let (tx, rx) = oneshot::channel();
        self.response_channels.lock().await.insert(seq_no, tx);

        // Send request
        let mut stream = self.stream.lock().await;
        send_message(&mut *stream, msg_type, seq_no, &request).await?;
        drop(stream);

        // Wait for response (timeout after 30 seconds)
        let response_bytes = tokio::time::timeout(
            Duration::from_secs(30),
            rx
        ).await??;

        // Decode response
        Ok(Res::decode(&response_bytes[..])?)
    }

    async fn receive_loop(self) -> Result<()> {
        loop {
            let (header, body) = {
                let mut stream = self.stream.lock().await;
                recv_message(&mut *stream).await?
            };

            // Check if this is response or push
            if let Some(tx) = self.response_channels.lock().await.remove(&header.seq_no) {
                // Response to our request
                let _ = tx.send(body);
            } else {
                // Push notification
                let push = PushMessage {
                    msg_type: header.msg_type,
                    body,
                };
                let _ = self.callback_tx.send(push);
            }
        }
    }
}
```

### Phase 3: Market Data Implementation (4 days)

#### Task 3.1: Subscribe/Unsubscribe

```rust
// src/subscription.rs
use crate::proto::qot_sub::*;

const MSG_TYPE_QOT_SUB: u32 = 3001;

impl FutuClient {
    pub async fn subscribe(
        &self,
        symbols: &[&str],
        subtypes: &[SubType],
    ) -> Result<()> {
        let security_list = symbols.iter().map(|s| {
            parse_security(s)
        }).collect();

        let request = Request {
            c2s: Some(C2S {
                security_list,
                sub_type_list: subtypes.iter().map(|st| *st as i32).collect(),
                is_sub_or_unsub: true,
                is_first_push: true,
                is_subscribe_push: true,
                ..Default::default()
            }),
        };

        let response: Response = self.send_request(MSG_TYPE_QOT_SUB, request).await?;

        if response.ret_type != 0 {
            return Err(FutuError::ApiError(
                response.ret_msg.unwrap_or_else(|| "Unknown error".into())
            ));
        }

        Ok(())
    }

    pub async fn unsubscribe(
        &self,
        symbols: &[&str],
        subtypes: &[SubType],
    ) -> Result<()> {
        let security_list = symbols.iter().map(|s| {
            parse_security(s)
        }).collect();

        let request = Request {
            c2s: Some(C2S {
                security_list,
                sub_type_list: subtypes.iter().map(|st| *st as i32).collect(),
                is_sub_or_unsub: false,  // false = unsubscribe
                ..Default::default()
            }),
        };

        let response: Response = self.send_request(MSG_TYPE_QOT_SUB, request).await?;

        if response.ret_type != 0 {
            return Err(FutuError::ApiError(
                response.ret_msg.unwrap_or_else(|| "Unknown error".into())
            ));
        }

        Ok(())
    }
}
```

#### Task 3.2: Get Stock Quote

```rust
// src/market_data.rs
use crate::proto::qot_get_stock_quote::*;

const MSG_TYPE_QOT_GET_STOCK_QUOTE: u32 = 3010;

impl FutuClient {
    pub async fn get_stock_quote(&self, symbols: &[&str]) -> Result<Vec<Quote>> {
        let security_list = symbols.iter().map(|s| {
            parse_security(s)
        }).collect();

        let request = Request {
            c2s: Some(C2S { security_list }),
        };

        let response: Response = self.send_request(
            MSG_TYPE_QOT_GET_STOCK_QUOTE,
            request
        ).await?;

        if response.ret_type != 0 {
            return Err(FutuError::ApiError(
                response.ret_msg.unwrap_or_else(|| "Unknown error".into())
            ));
        }

        let s2c = response.s2c.ok_or(FutuError::MissingField("s2c"))?;

        // Convert protobuf snapshots to Quote structs
        let quotes = s2c.snapshot_list.iter().map(|snapshot| {
            Quote {
                symbol: format!("{}.{}",
                    market_to_string(snapshot.security.market),
                    snapshot.security.code
                ),
                last_price: snapshot.last_price,
                open_price: snapshot.open_price,
                high_price: snapshot.high_price,
                low_price: snapshot.low_price,
                prev_close_price: snapshot.prev_close_price,
                volume: snapshot.volume,
                turnover: snapshot.turnover,
                // ... map all fields
            }
        }).collect();

        Ok(quotes)
    }
}
```

#### Task 3.3: Other Market Data Methods

Implement similarly:
- `get_order_book()` → MSG_TYPE 3012
- `get_ticker()` → MSG_TYPE 3013
- `get_kline()` → MSG_TYPE 3006
- `get_market_state()` → MSG_TYPE 3203

**Pattern is same for all**:
1. Construct Request protobuf
2. Call `send_request()` with message type ID
3. Decode Response protobuf
4. Check ret_type for errors
5. Convert protobuf types to Rust domain types

### Phase 4: Trading Implementation (4 days)

#### Task 4.1: Unlock Trade

```rust
// src/trading.rs
use crate::proto::trd_unlock_trade::*;

const MSG_TYPE_TRD_UNLOCK_TRADE: u32 = 2005;

impl FutuClient {
    pub async fn unlock_trade(
        &self,
        password: &str,
        security_firm: i32,
    ) -> Result<()> {
        let request = Request {
            c2s: Some(C2S {
                unlock: true,
                pwd_md5: md5_hash(password),
                security_firm: Some(security_firm),
            }),
        };

        let response: Response = self.send_request(
            MSG_TYPE_TRD_UNLOCK_TRADE,
            request
        ).await?;

        if response.ret_type != 0 {
            return Err(FutuError::TradingLocked(
                response.ret_msg.unwrap_or_else(|| "Unlock failed".into())
            ));
        }

        Ok(())
    }
}
```

#### Task 4.2: Place Order

```rust
use crate::proto::trd_place_order::*;

const MSG_TYPE_TRD_PLACE_ORDER: u32 = 2202;

impl FutuClient {
    pub async fn place_order(
        &self,
        account: &TradingAccount,
        symbol: &str,
        side: TrdSide,
        order_type: OrderType,
        price: f64,
        quantity: f64,
    ) -> Result<u64> {  // Returns order ID
        let request = Request {
            c2s: Some(C2S {
                packet_id: Some(PacketID {
                    conn_id: get_connection_id(),
                    serial_no: get_next_serial(),
                }),
                trd_acc: Some(account.to_proto()),
                trd_side: side as i32,
                order_type: order_type as i32,
                code: parse_code(symbol),
                qty: quantity,
                price,
                adjust_price: Some(false),
                sec_market: Some(parse_market(symbol)),
                ..Default::default()
            }),
        };

        let response: Response = self.send_request(
            MSG_TYPE_TRD_PLACE_ORDER,
            request
        ).await?;

        if response.ret_type != 0 {
            return Err(FutuError::OrderFailed(
                response.ret_msg.unwrap_or_else(|| "Order placement failed".into())
            ));
        }

        let s2c = response.s2c.ok_or(FutuError::MissingField("s2c"))?;
        Ok(s2c.order_id)
    }
}
```

#### Task 4.3: Other Trading Methods

Implement:
- `cancel_order()` → MSG_TYPE 2205
- `modify_order()` → MSG_TYPE 2205 (with modify flag)
- `get_order_list()` → MSG_TYPE 2201
- `get_position_list()` → MSG_TYPE 2101
- `get_account_list()` → MSG_TYPE 2001

### Phase 5: Callback System (3 days)

#### Task 5.1: Callback Handler Trait

```rust
// src/callbacks.rs
use async_trait::async_trait;

#[async_trait]
pub trait QuoteHandler: Send + Sync {
    async fn on_quote_update(&mut self, quote: Quote);
}

#[async_trait]
pub trait TickerHandler: Send + Sync {
    async fn on_ticker_update(&mut self, ticker: Ticker);
}

#[async_trait]
pub trait OrderHandler: Send + Sync {
    async fn on_order_update(&mut self, order: Order);
}

pub struct CallbackManager {
    quote_handlers: Vec<Box<dyn QuoteHandler>>,
    ticker_handlers: Vec<Box<dyn TickerHandler>>,
    order_handlers: Vec<Box<dyn OrderHandler>>,
}

impl CallbackManager {
    pub fn register_quote_handler<H: QuoteHandler + 'static>(&mut self, handler: H) {
        self.quote_handlers.push(Box::new(handler));
    }

    pub async fn handle_push_message(&mut self, push: PushMessage) -> Result<()> {
        match push.msg_type {
            MSG_TYPE_QOT_UPDATE_BASIC => {
                let update = qot_update_basic::Response::decode(&push.body[..])?;
                for snapshot in update.s2c.unwrap().snapshot_list {
                    let quote = convert_to_quote(snapshot);
                    for handler in &mut self.quote_handlers {
                        handler.on_quote_update(quote.clone()).await;
                    }
                }
            }
            MSG_TYPE_QOT_UPDATE_TICKER => {
                let update = qot_update_ticker::Response::decode(&push.body[..])?;
                // ... handle ticker updates
            }
            MSG_TYPE_TRD_UPDATE_ORDER => {
                let update = trd_update_order::Response::decode(&push.body[..])?;
                // ... handle order updates
            }
            _ => {
                tracing::warn!("Unknown push message type: {}", push.msg_type);
            }
        }
        Ok(())
    }
}
```

#### Task 5.2: Integrate with Client

```rust
impl FutuClient {
    pub fn set_callback_manager(&mut self, manager: CallbackManager) {
        // Spawn callback processor
        tokio::spawn(async move {
            while let Some(push) = callback_rx.recv().await {
                if let Err(e) = manager.handle_push_message(push).await {
                    tracing::error!("Callback error: {}", e);
                }
            }
        });
    }
}
```

### Phase 6: V5 Trait Implementation (2 days)

#### Task 6.1: MarketData Trait

```rust
// src/connector.rs
use crate::traits::{MarketData, Ticker, OrderBook};

pub struct FutuConnector {
    client: Arc<FutuClient>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

#[async_trait]
impl MarketData for FutuConnector {
    async fn fetch_ticker(&self, symbol: &str) -> Result<Ticker> {
        // Ensure subscribed
        self.ensure_subscribed(symbol, SubType::Basic).await?;

        // Small delay for subscription to activate
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get quote
        let quotes = self.client.get_stock_quote(&[symbol]).await?;
        let quote = quotes.first().ok_or(FutuError::NoData)?;

        Ok(Ticker {
            symbol: quote.symbol.clone(),
            last_price: quote.last_price,
            bid: quote.bid_price,
            ask: quote.ask_price,
            volume: quote.volume as f64,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn fetch_order_book(&self, symbol: &str, depth: usize) -> Result<OrderBook> {
        self.ensure_subscribed(symbol, SubType::OrderBook).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let book = self.client.get_order_book(symbol).await?;

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids: book.bid_list.into_iter().take(depth).collect(),
            asks: book.ask_list.into_iter().take(depth).collect(),
            timestamp: chrono::Utc::now(),
        })
    }

    // ... implement other methods
}
```

#### Task 6.2: Trading Trait

```rust
#[async_trait]
impl Trading for FutuConnector {
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        price: f64,
        quantity: f64,
    ) -> Result<String> {
        let account = self.get_default_account().await?;

        let order_id = self.client.place_order(
            &account,
            symbol,
            side.into(),
            OrderType::Normal,
            price,
            quantity,
        ).await?;

        Ok(order_id.to_string())
    }

    async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let order_id_u64 = order_id.parse::<u64>()?;
        self.client.cancel_order(order_id_u64).await
    }

    // ... implement other methods
}
```

### Phase 7: Error Handling & Testing (3 days)

#### Task 7.1: Comprehensive Error Types

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FutuError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Not subscribed to {symbol} with {subtype:?}")]
    NotSubscribed {
        symbol: String,
        subtype: SubType,
    },

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Trading not unlocked")]
    TradingLocked(String),

    #[error("Order failed: {0}")]
    OrderFailed(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Connection closed")]
    ConnectionClosed,
}
```

#### Task 7.2: Unit Tests

```rust
// src/client.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe() {
        let client = FutuClient::connect("127.0.0.1", 11111).await.unwrap();
        client.subscribe(&["US.AAPL"], &[SubType::Basic]).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_quote() {
        let client = FutuClient::connect("127.0.0.1", 11111).await.unwrap();
        client.subscribe(&["US.AAPL"], &[SubType::Basic]).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let quotes = client.get_stock_quote(&["US.AAPL"]).await.unwrap();
        assert_eq!(quotes.len(), 1);
        assert!(quotes[0].last_price > 0.0);
    }
}
```

#### Task 7.3: Integration Tests

```rust
// tests/integration_test.rs
use futu::*;

#[tokio::test]
async fn test_full_flow() {
    // Requires OpenD running locally
    let client = FutuClient::connect("127.0.0.1", 11111).await.unwrap();

    // Subscribe
    client.subscribe(&["US.AAPL", "HK.00700"], &[SubType::Basic]).await.unwrap();

    // Get quotes
    let quotes = client.get_stock_quote(&["US.AAPL", "HK.00700"]).await.unwrap();
    assert_eq!(quotes.len(), 2);

    // Unsubscribe
    tokio::time::sleep(Duration::from_secs(60)).await;  // Wait 1 minute
    client.unsubscribe(&["US.AAPL"], &[SubType::Basic]).await.unwrap();
}
```

---

## Estimated Timeline

| Phase | Tasks | Days | Complexity |
|-------|-------|------|------------|
| **1. Setup** | Extract protos, configure prost-build | 2 | Low |
| **2. TCP Protocol** | Framing, connection management | 3 | Medium |
| **3. Market Data** | Subscribe, quote, orderbook, kline | 4 | Medium |
| **4. Trading** | Unlock, place/cancel orders, positions | 4 | High |
| **5. Callbacks** | Push notification handlers | 3 | High |
| **6. V5 Traits** | Implement MarketData/Trading traits | 2 | Low |
| **7. Testing** | Unit tests, integration tests | 3 | Medium |
| **8. Documentation** | API docs, examples, README | 2 | Low |
| **9. Polish** | Error handling, logging, refactoring | 2 | Low |
| **Total** | | **25 days** | **High** |

**Assumes**: 1 experienced Rust developer, 8 hours/day

---

## Key Challenges

### 1. Protocol Reverse Engineering

**Problem**: Futu protocol not fully documented

**Mitigation**:
- Study Python SDK source code
- Capture and analyze TCP packets (Wireshark)
- Test each endpoint incrementally
- Document findings

### 2. Message Type Registry

**Problem**: Must map 100+ message type IDs to response types

**Solution**:
```rust
// Auto-generate with macro
macro_rules! message_registry {
    ($($msg_type:expr => $response_type:ty),*) => {
        fn decode_response(msg_type: u32, bytes: &[u8]) -> Result<Box<dyn Any>> {
            match msg_type {
                $($msg_type => Ok(Box::new(<$response_type>::decode(bytes)?)),)*
                _ => Err(FutuError::UnknownMessageType(msg_type)),
            }
        }
    };
}

message_registry! {
    3010 => qot_get_stock_quote::Response,
    3012 => qot_get_order_book::Response,
    2202 => trd_place_order::Response
    // ... 100+ more
}
```

### 3. Async Callback System

**Problem**: Push notifications arrive asynchronously

**Solution**: Use tokio channels + callback handlers (see Phase 5)

### 4. Subscription State Management

**Problem**: Must track what's subscribed, when, quota usage

**Solution**:
```rust
struct SubscriptionManager {
    subscriptions: HashMap<(String, SubType), Instant>,  // symbol+subtype -> subscribe time
    quota_used: usize,
    quota_max: usize,
}

impl SubscriptionManager {
    fn can_subscribe(&self) -> bool {
        self.quota_used < self.quota_max
    }

    fn can_unsubscribe(&self, symbol: &str, subtype: SubType) -> bool {
        if let Some(subscribe_time) = self.subscriptions.get(&(symbol.to_string(), subtype)) {
            subscribe_time.elapsed() > Duration::from_secs(60)  // 1 minute wait
        } else {
            false  // Not subscribed
        }
    }
}
```

### 5. Error Handling Complexity

**Problem**: Mix of network errors, protocol errors, API errors

**Solution**: Use `thiserror` for ergonomic error types (see Phase 7)

---

## Performance Considerations

### Zero-Copy Where Possible

```rust
// Instead of copying bytes
let data = response.data.clone();  // ❌ Copies

// Use references
let data = &response.data;  // ✅ Zero-copy
```

### Connection Pooling

```rust
// For multiple concurrent operations
pub struct FutuClientPool {
    clients: Vec<Arc<FutuClient>>,
}

impl FutuClientPool {
    pub fn get_client(&self) -> Arc<FutuClient> {
        // Round-robin or least-loaded
    }
}
```

### Batching Requests

```rust
// Batch multiple symbols in one request
client.get_stock_quote(&["US.AAPL", "US.GOOGL", "US.MSFT"]).await?;

// Instead of multiple requests (slower)
for symbol in symbols {
    client.get_stock_quote(&[symbol]).await?;  // ❌ Slow
}
```

---

## Maintenance Burden

### Ongoing Effort

| Task | Frequency | Effort |
|------|-----------|--------|
| **Protocol updates** | When Futu updates | High |
| **Bug fixes** | As discovered | Medium |
| **New endpoints** | When needed | Medium |
| **Regression testing** | After Futu updates | High |
| **Documentation** | Continuous | Low |

**Annual estimate**: 10-20 days/year for maintenance

---

## Advantages of Pure Rust

✅ **No Python dependency**
✅ **Better performance** (no FFI overhead)
✅ **Smaller binary** (~5-10 MB vs ~50 MB)
✅ **Type safety** (Rust compiler catches bugs)
✅ **Easier deployment** (single binary)

---

## Disadvantages of Pure Rust

❌ **High initial effort** (25 days)
❌ **Maintenance burden** (10-20 days/year)
❌ **Protocol reverse engineering** (undocumented)
❌ **Still requires OpenD** (can't bypass)
❌ **Unofficial implementation** (not supported by Futu)

---

## Recommendation

### When to Choose Native Rust

Choose if:
1. **Pure Rust is mandatory** (no Python allowed)
2. **Performance critical** (ultra-low latency required)
3. **Long-term commitment** (team can maintain)
4. **Expertise available** (experienced Rust developers)

### When to Choose PyO3 Instead

Choose PyO3 if:
1. **Fast implementation** needed (< 1 week)
2. **Low maintenance** preferred
3. **Python dependency** acceptable
4. **Battle-tested** solution wanted

**Most projects should use PyO3 (Option 1)**, not native Rust.

---

## Sources

- [Prost (Rust Protobuf)](https://github.com/tokio-rs/prost)
- [Tokio Async Runtime](https://github.com/tokio-rs/tokio)
- [Futu Python SDK (reference)](https://github.com/FutunnOpen/py-futu-api)
- [Futu Protocol Docs](https://openapi.futunn.com/futu-api-doc/en/ftapi/protocol.html)
