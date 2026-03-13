# Transport Abstraction Architecture Research
# digdigdig3 — Adding gRPC and GraphQL Transports

**Date:** 2026-03-14
**Scope:** Architecture options for adding gRPC and GraphQL alongside the existing `HttpClient` and `WebSocketConnector`

---

## 1. Current Architecture Baseline

### What Exists Today

```
core/
├── http/
│   └── client.rs          # HttpClient — wraps reqwest
│                          # get/post/put/delete/patch + retry + rate-limit + metrics
├── websocket/
│   └── base_websocket.rs  # BaseWebSocket<C: WebSocketConfig + IdentityConfig>
│                          # auto-reconnect, subscription recovery, ping/pong
└── traits/
    └── mod.rs             # WebSocketConnector trait — connect/disconnect/subscribe/event_stream
```

**Key characteristics of `HttpClient`:**
- Single `reqwest::Client` inside
- Retry logic with exponential backoff (max 3 attempts by default)
- 429 / 5xx retry, 4xx no-retry
- Three `Arc<AtomicU64>` metrics counters (requests_total, errors_total, last_latency_ms)
- Returns `serde_json::Value` — deliberately untyped at the transport level
- Each connector creates its own `HttpClient` instance; no shared state

**Key characteristics of `BaseWebSocket<C>`:**
- Parameterized over `C: WebSocketConfig + IdentityConfig`
- All exchange-specific logic (subscribe/unsubscribe messages, ping format, message classification) lives in `C`
- Auto-reconnect loop in a spawned task
- Subscription recovery after reconnect
- Exposes `event_stream() -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>>>>`

### Current Transport Usage by Connector Count

From inventory files across the codebase:

| Transport | Connector count | Examples |
|-----------|----------------|---------|
| REST (HttpClient) only | ~130 | Binance, KuCoin, Bybit, OKX, all stocks/forex |
| REST + WebSocket | ~20 | Binance futures, Bybit, Kraken, Gemini |
| gRPC (NOT YET IMPLEMENTED) | 3–4 | dYdX v4 Node, Tinkoff native, Futu |
| GraphQL (NOT YET IMPLEMENTED) | 2–3 | Uniswap v3 subgraph (The Graph), Bitquery |
| TCP Protobuf (NOT YET IMPLEMENTED) | 1 | Futu OpenAPI |
| REST proxy for gRPC | 2 | Tinkoff (via `invest-public-api.tbank.ru/rest`), dYdX (Indexer covers reads) |

---

## 2. Connectors That Require Non-HTTP Transports

### 2.1 gRPC Required

**dYdX v4 — Node API (Cosmos SDK)**
- Architecture: Dual-tier. Indexer API = REST/WebSocket (read-only). Node API = gRPC (write, trading).
- gRPC endpoints: `grpc://oegs.dydx.trade:443` and community validators
- Methods requiring gRPC: `MsgPlaceOrder`, `MsgCancelOrder`, `MsgBatchCancel`, transfers
- Current workaround: only Indexer API implemented; trading methods return `UnsupportedOperation`
- `.proto` source: dydx-chain protobuf definitions (Cosmos SDK `cosmos.tx.v1beta1`)

**Tinkoff Invest (Russia)**
- Architecture: Native protocol is gRPC (`invest-public-api.tbank.ru:443`)
- Has REST proxy: `https://invest-public-api.tbank.ru/rest` (gRPC-JSON transcoding)
- Current implementation: uses REST proxy via `reqwest::Client` directly (NOT `HttpClient`)
- All requests are POST with JSON body (gRPC-HTTP/JSON transcoding pattern)
- The connector bypasses `HttpClient` entirely — uses raw `reqwest::Client`

**Futu OpenAPI (China stocks)**
- Architecture: TCP + Protocol Buffers v3 — NOT HTTP at all
- No REST proxy available
- Current status: STUB — all methods return `UnsupportedOperation`
- Note in code: "Run OpenD gateway; implement Protobuf client; or use Python SDK via PyO3/FFI"

### 2.2 GraphQL Required

**Uniswap v3 — The Graph subgraph**
- Architecture: Three backends — Ethereum RPC (slot0 for live price), The Graph GraphQL (historical), Uniswap Trading API REST (POST for swaps)
- GraphQL endpoint: `https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3`
- Query type: `POST /graphql` with JSON body `{"query": "...", "variables": {...}}`
- Subscription type: `graphql-transport-ws` over WebSocket (for real-time)

**Bitquery**
- Architecture: GraphQL-first — all endpoints are GraphQL queries
- HTTP endpoint: `POST https://graphql.bitquery.io/` with Bearer token
- WebSocket: `wss://streaming.bitquery.io/graphql` using `graphql-transport-ws` subprotocol
- All endpoints require authentication (every query needs API key)
- Note in audit: "Wire existing GraphQL subscription queries to `WebSocketConnector` trait"

**GMX v2 — The Graph (optional)**
- Historical positions, trades, liquidations available via The Graph GraphQL
- Not currently in connector; marked "not in connector" in research

---

## 3. Option Analysis

### Option A: Transport Enum

```rust
enum Transport {
    Rest(HttpClient),
    Grpc(GrpcClient),         // wraps tonic::Channel
    GraphQL(GraphQlClient),   // wraps reqwest + query builder
    TcpProtobuf(TcpClient),   // for Futu
}

struct SomeConnector {
    transport: Transport,
    // ...
}
```

**How it works:** Every connector call matches on the transport variant to dispatch.

**Pros:**
- Single field in connector struct
- Transport is runtime-switchable (if an exchange adds a REST fallback)
- Easy to enumerate "what transport does this connector use"

**Cons:**
- Every internal call needs a `match transport { ... }` arm — even REST-only connectors pay this cost in code verbosity
- gRPC and REST have fundamentally different call shapes. REST returns `Value`; gRPC returns typed Protobuf structs. The enum arms cannot share return types without boxing.
- You cannot put a connector that needs BOTH REST and gRPC (dYdX: REST for reads, gRPC for writes) in a single `Transport` variant without either nesting or a `Vec<Transport>`
- `GrpcClient` would need to be `Arc<Channel>` (tonic's `Channel` is `Clone + Send`), but `HttpClient` is not `Clone` due to `Arc<AtomicU64>` — inconsistency
- Feature flag `#[cfg(feature = "grpc")]` on one variant causes the whole enum to require the flag, breaking REST-only connectors in no-feature builds

**Verdict: Not recommended.** The enum flattens a 2D problem (transport type × read/write side) into 1D. It works for connectors with a single transport but breaks immediately for dYdX.

---

### Option B: Layered Clients (Independent Structs)

```rust
// Existing — unchanged
pub struct HttpClient { /* reqwest + retry */ }

// New — wraps tonic::Channel
#[cfg(feature = "grpc")]
pub struct GrpcClient {
    channel: tonic::transport::Channel,
    timeout: Duration,
    debug: bool,
}

// New — thin wrapper over HttpClient
pub struct GraphQlClient {
    http: HttpClient,  // reuses retry, rate-limit, metrics
    endpoint: String,
}

// dYdX connector holds both:
pub struct DydxConnector {
    http: HttpClient,          // Indexer REST reads
    grpc: Option<GrpcClient>,  // Node API writes (None if no creds)
    // ...
}

// Bitquery connector:
pub struct BitqueryConnector {
    graphql: GraphQlClient,
    ws: Option<BaseWebSocket<BitqueryWsConfig>>,
    // ...
}
```

**How `GrpcClient` works internally:**
- Wraps `tonic::transport::Channel`
- Exposes typed methods per exchange (no unified interface — each connector generates its own service client from `.proto`)
- Build script (`build.rs`) calls `tonic_build::compile_protos(...)` behind `#[cfg(feature = "grpc")]`
- Each gRPC connector module has its own generated `*_client.rs`

**How `GraphQlClient` works internally:**
- Delegates all HTTP to `HttpClient` — gets retry, backoff, 429 handling for free
- Adds `query(query: &str, variables: Value) -> ExchangeResult<Value>` method
- POST body: `{"query": "...", "variables": {...}}`
- Returns parsed JSON `data` field, maps `errors` array to `ExchangeError::Api`

**Pros:**
- Zero overhead for the ~130 REST-only connectors — they never touch `GrpcClient` or `GraphQlClient`
- `GraphQlClient` is trivial — it IS `HttpClient` with a different body format. ~50 lines total.
- `GrpcClient` can be fully `#[cfg(feature = "grpc")]` without touching anything else
- Each connector holds exactly the clients it needs — no phantom fields
- Matches how the codebase is already structured (Tinkoff bypasses `HttpClient`, dYdX uses `HttpClient` for REST reads)
- `tonic::transport::Channel` is `Clone + Send + Sync` — fits `Arc<Channel>` pattern easily

**Cons:**
- No unified "call any transport with one method" interface — but this was never a goal
- Each gRPC connector must generate its own protobuf bindings (build.rs complexity)
- Connector that needs both REST and gRPC has two fields — minor verbosity

**Verdict: Recommended.** Matches existing architecture patterns. Minimal disruption. Zero cost for majority.

---

### Option C: Abstract Transport Trait

```rust
#[async_trait]
pub trait TransportLayer: Send + Sync {
    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        headers: HashMap<String, String>,
    ) -> ExchangeResult<Value>;
}

impl TransportLayer for HttpClient { ... }
impl TransportLayer for GrpcClient { ... }   // forced to JSON-encode Protobuf responses
impl TransportLayer for GraphQlClient { ... }
```

**The fundamental problem with Option C:**
gRPC is NOT a request/response-over-path protocol. A gRPC call is identified by a `ServiceClient::method_name(request: proto::Request) -> Response<proto::Reply>`. There is no "path" in the REST sense — paths are defined in `.proto` as service/method pairs and baked into generated code. You cannot call a gRPC endpoint generically via `request(method, "/some/path", body)`.

Additional mismatches:
- gRPC streaming (server-streaming, bidirectional) has no analog in the `request()` signature
- Protobuf response types lose static type safety when forced into `Value`
- The signature leaks HTTP concepts (`Method`, `path`) into a generic transport trait

**Where it almost works:** GraphQL is literally REST POST, so `GraphQlClient` could implement `TransportLayer`. But this offers no benefit vs. just having `GraphQlClient` call `HttpClient.post()` directly.

**Verdict: Not recommended.** The impedance mismatch between gRPC's generated typed API and a generic `request()` signature cannot be bridged without sacrificing type safety or performance.

---

## 4. Recommendation: Option B with Feature Gating

### Architecture Decision

**Use Option B: Layered independent clients.**

Rationale:
1. 130+ of ~137 connectors are REST-only. They should not pay any cost — in compile time, binary size, or code complexity — for transports they do not use.
2. GraphQL is structurally identical to REST POST. It should be a thin `GraphQlClient` wrapper over `HttpClient`, sharing all retry/rate-limit/metrics infrastructure.
3. gRPC (tonic) requires code generation from `.proto` files. This is per-connector work, not framework work. It belongs in connector modules, not in `core/`.
4. The codebase already demonstrates this pattern: Tinkoff uses raw `reqwest::Client` in its connector module, completely bypassing `HttpClient`. Option B formalizes this pattern with better abstractions.

### Proposed Module Layout

```
core/
├── http/
│   ├── client.rs          # HttpClient — unchanged
│   └── mod.rs
├── websocket/
│   ├── base_websocket.rs  # BaseWebSocket — unchanged
│   └── mod.rs
├── graphql/               # NEW — thin wrapper
│   ├── client.rs          # GraphQlClient
│   └── mod.rs
├── grpc/                  # NEW — behind feature flag
│   ├── client.rs          # GrpcClient (wraps tonic::Channel)
│   └── mod.rs
└── mod.rs                 # re-export all, grpc behind cfg
```

### `GraphQlClient` Implementation Sketch

```rust
// core/graphql/client.rs

pub struct GraphQlClient {
    http: HttpClient,
    endpoint: String,
    /// Optional auth header (e.g. "Bearer {api_key}")
    auth_header: Option<(String, String)>,
}

impl GraphQlClient {
    pub fn new(endpoint: &str, timeout_ms: u64) -> ExchangeResult<Self> {
        Ok(Self {
            http: HttpClient::new(timeout_ms)?,
            endpoint: endpoint.to_string(),
            auth_header: None,
        })
    }

    pub fn with_auth(mut self, header_name: &str, header_value: &str) -> Self {
        self.auth_header = Some((header_name.to_string(), header_value.to_string()));
        self
    }

    /// Execute a GraphQL query or mutation
    pub async fn query(
        &self,
        query: &str,
        variables: Value,
    ) -> ExchangeResult<Value> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let mut headers = HashMap::new();
        if let Some((k, v)) = &self.auth_header {
            headers.insert(k.clone(), v.clone());
        }

        let response = self.http.post(&self.endpoint, &body, &headers).await?;

        // GraphQL errors live in response["errors"] even on HTTP 200
        if let Some(errors) = response.get("errors") {
            if let Some(arr) = errors.as_array() {
                if !arr.is_empty() {
                    let msg = arr[0]
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("GraphQL error")
                        .to_string();
                    return Err(ExchangeError::Api { code: -1, message: msg });
                }
            }
        }

        // Return the "data" field
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ExchangeError::ParseError("GraphQL response missing 'data'".into()))
    }

    /// Forward stats from inner HttpClient
    pub fn stats(&self) -> (u64, u64, u64) {
        self.http.stats()
    }
}
```

**Note:** GraphQL subscriptions over WebSocket use the `graphql-transport-ws` subprotocol. These should be implemented as a separate `WebSocketConfig` implementation in the connector module (e.g. `BitqueryWsConfig`), reusing `BaseWebSocket<BitqueryWsConfig>` — no new transport client needed.

### `GrpcClient` Implementation Sketch

```rust
// core/grpc/client.rs
// Only compiled when feature "grpc" is enabled

#[cfg(feature = "grpc")]
pub struct GrpcClient {
    /// tonic Channel is Clone + Send + Sync — safe to share via Arc
    channel: tonic::transport::Channel,
    timeout: Duration,
    debug: bool,
    /// Metrics (same pattern as HttpClient)
    pub requests_total: Arc<AtomicU64>,
    pub errors_total: Arc<AtomicU64>,
    pub last_latency_ms: Arc<AtomicU64>,
}

#[cfg(feature = "grpc")]
impl GrpcClient {
    pub async fn connect(endpoint: &str, timeout_ms: u64) -> ExchangeResult<Self> {
        let channel = tonic::transport::Channel::from_shared(endpoint.to_string())
            .map_err(|e| ExchangeError::Network(format!("Invalid gRPC endpoint: {}", e)))?
            .timeout(Duration::from_millis(timeout_ms))
            .connect()
            .await
            .map_err(|e| ExchangeError::Network(format!("gRPC connect failed: {}", e)))?;

        let debug = std::env::var("DEBUG_GRPC").is_ok();

        Ok(Self {
            channel,
            timeout: Duration::from_millis(timeout_ms),
            debug,
            requests_total: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
            last_latency_ms: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Get the inner channel (connectors use this to construct tonic service clients)
    ///
    /// # Example (dYdX connector)
    /// ```ignore
    /// let channel = self.grpc.channel();
    /// let mut order_client = OrderServiceClient::new(channel);
    /// order_client.place_order(request).await?;
    /// ```
    pub fn channel(&self) -> tonic::transport::Channel {
        self.channel.clone()   // Channel is cheap to clone (shared Arc internally)
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.requests_total.load(Ordering::Relaxed),
            self.errors_total.load(Ordering::Relaxed),
            self.last_latency_ms.load(Ordering::Relaxed),
        )
    }
}
```

**Design note:** `GrpcClient` does NOT expose a generic `call()` method. It is purely a managed connection handle. Each connector constructs its own tonic-generated service client from `grpc.channel()`. This is intentional — tonic's generated code is the "typed API layer" for gRPC, just as parser.rs is the typed layer for REST JSON.

---

## 5. Feature Gating Impact Analysis

### Cargo.toml Setup

```toml
[features]
default = []
grpc = ["tonic", "prost"]

[dependencies]
# Always present
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio-tungstenite = "0.24"
serde_json = "1"

# Optional — only with "grpc" feature
tonic = { version = "0.12", optional = true }
prost = { version = "0.13", optional = true }

[build-dependencies]
# Only needed when building gRPC connectors
tonic-build = { version = "0.12", optional = true }
```

### Conditional Compilation Pattern

```rust
// core/mod.rs
pub mod http;
pub mod websocket;
pub mod graphql;   // Always available

#[cfg(feature = "grpc")]
pub mod grpc;

// Re-exports
pub use http::HttpClient;
pub use graphql::GraphQlClient;

#[cfg(feature = "grpc")]
pub use grpc::GrpcClient;
```

### Connector Registry / Factory Impact

The connector factory (if one exists) typically instantiates connectors by name/ID. gRPC connectors are conditionally compiled:

```rust
// In connector factory / registry
pub fn create_connector(id: ExchangeId) -> Box<dyn CoreConnector> {
    match id {
        ExchangeId::Binance => Box::new(BinanceConnector::new()),
        ExchangeId::Uniswap => Box::new(UniswapConnector::new()),

        // gRPC connectors only available with feature flag
        #[cfg(feature = "grpc")]
        ExchangeId::DydxNode => Box::new(DydxNodeConnector::new()),

        #[cfg(feature = "grpc")]
        ExchangeId::Tinkoff => Box::new(TinkoffNativeConnector::new()),

        // Default: return UnsupportedOperation stub or error
        #[cfg(not(feature = "grpc"))]
        ExchangeId::DydxNode => panic!("Compile with --features grpc for dYdX Node API"),
    }
}
```

**Key point:** The `ExchangeId::DydxTrading` variant still exists in the enum regardless of feature flags. Only the connector implementation is conditionally compiled. This means the registry can always enumerate all exchange IDs, but some connectors may not be available in a given build.

**Alternative cleaner approach:** Keep dYdX as a single connector but make the `Trading` trait methods check at runtime:

```rust
impl Trading for DydxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        #[cfg(feature = "grpc")]
        {
            self.grpc_place_order(req).await
        }
        #[cfg(not(feature = "grpc"))]
        {
            Err(ExchangeError::UnsupportedOperation(
                "dYdX trading requires gRPC. Compile with --features grpc".into()
            ))
        }
    }
}
```

This is cleaner — the connector always exists, but methods gracefully degrade without the feature.

---

## 6. Specific Connector Implementation Plans

### 6.1 dYdX v4 Node (gRPC Trading)

**What to add:**
```
crypto/dex/dydx/
├── connector.rs       # Existing (Indexer REST)
├── node_client.rs     # NEW: gRPC client for Node API (cfg = "grpc")
└── proto/             # NEW: .proto files (or import from dydx-chain repo)
    ├── order.proto
    └── clob.proto
```

**How trading would work:**
1. `DydxConnector` gets `Option<GrpcClient>` field
2. `place_order()` constructs `MsgPlaceOrder` protobuf message
3. Signs with Cosmos secp256k1 key (separate auth module)
4. Broadcasts via `BroadcastTxService::broadcast_tx()`
5. Polls Indexer REST for order status (already implemented)

**Key detail:** dYdX gRPC uses HTTPS (TLS) on port 443, not the usual cleartext `grpc://` — `GrpcClient::connect()` must use `tonic::transport::Channel::from_shared().tls_config(...)`.

### 6.2 Tinkoff Native gRPC (Russia)

**Current state:** Uses REST proxy via raw `reqwest::Client` — works but bypasses `HttpClient` infrastructure.
**Two options:**
- Option B1: Migrate to `HttpClient` (correct architectural fit — REST proxy IS HTTP)
- Option B2: Add native gRPC path (better performance, required for streaming quotes)

For now, **B1 is the correct move** — refactor `TinkoffConnector` to use `HttpClient` instead of raw `reqwest::Client`. The REST proxy handles all current needs.

Native gRPC (B2) would be needed only for real-time streaming (market data subscriptions), which would use `BaseWebSocket` pattern anyway.

### 6.3 Uniswap v3 GraphQL (The Graph)

**What to add:**
```
crypto/dex/uniswap/
├── connector.rs          # Existing (uses HttpClient for REST + Ethereum RPC)
└── graphql_queries.rs    # NEW: static query strings for The Graph
```

**Implementation:**
```rust
pub struct UniswapConnector {
    http: HttpClient,              // Ethereum RPC + Uniswap Trading API
    graphql: GraphQlClient,        // The Graph subgraph queries
    ws: Option<BaseWebSocket<...>>,
}
```

`graphql_queries.rs` holds `const` query strings — no codegen needed (no schema types required at compile time since we work with `Value`).

### 6.4 Bitquery (GraphQL-first)

**New connector:**
```
intelligence_feeds/bitquery/
├── connector.rs         # BitqueryConnector
├── graphql_queries.rs   # All query strings as consts
└── websocket.rs         # BitqueryWsConfig for graphql-transport-ws
```

**WebSocket protocol note:** `graphql-transport-ws` requires a specific handshake:
1. WebSocket connection with `Sec-WebSocket-Protocol: graphql-transport-ws`
2. Send `connection_init` message with auth payload
3. Send `subscribe` with `id`, `type: "subscribe"`, and `payload.query`
4. Receive `next` messages with data

This maps naturally to `BaseWebSocket<BitqueryWsConfig>` — `WebSocketConfig::create_subscribe_message()` returns the `subscribe` JSON frame; `classify_message()` maps `next`/`complete`/`error` types.

The custom subprotocol header (`Sec-WebSocket-Protocol`) requires a small addition to `BaseWebSocket`:

```rust
// In BaseWebSocket connect_async call:
let request = tungstenite::handshake::client::Request::builder()
    .uri(&ws_url)
    .header("Sec-WebSocket-Protocol", "graphql-transport-ws")
    .body(())
    .unwrap();
connect_async(request).await
```

This could be added as an optional `WebSocketConfig` method:
```rust
pub trait WebSocketConfig {
    // ... existing methods ...

    /// Optional: extra headers for the WebSocket handshake
    /// Default: empty (standard WebSocket)
    fn handshake_headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
```

---

## 7. Minimal Change Path (What to Actually Build First)

Given the current state, the recommended sequence is:

### Step 1: `GraphQlClient` — No Feature Flag (1–2 days)

GraphQL is just HTTP POST with a specific body format. No new dependencies. No build.rs.
`GraphQlClient` wraps `HttpClient` — ~80 lines of code.

**Unblocks:** Uniswap The Graph queries, Bitquery HTTP queries.

### Step 2: Refactor Tinkoff to Use `HttpClient` (Half a day)

Replace raw `reqwest::Client` in `TinkoffConnector` with `HttpClient`.
No new transport needed — REST proxy already works.

**Unblocks:** Tinkoff gets retry, backoff, metrics for free.

### Step 3: `GrpcClient` Behind `feature = "grpc"` (2–3 days)

Create `GrpcClient` as a connection manager. Wire into `DydxConnector` as `Option<GrpcClient>`.

**Unblocks:** dYdX trading operations.

### Step 4: WebSocket Subprotocol Support in `BaseWebSocket` (1 day)

Add `handshake_headers()` to `WebSocketConfig` trait. Update `BaseWebSocket::connect()` to pass headers.

**Unblocks:** Bitquery real-time GraphQL subscriptions.

### Step 5: Futu TCP Protobuf (Future, Low Priority)

Futu uses raw TCP + Protobuf, not HTTP or gRPC. This requires a separate `TcpClient` (wraps `tokio::net::TcpStream`) with a custom framing protocol. Lowest priority — keep as STUB with clear error message.

---

## 8. Comparison Summary

| Criterion | Option A (enum) | Option B (layered) | Option C (trait) |
|-----------|----------------|--------------------|------------------|
| REST-only connector overhead | Match arm per call | Zero | Zero |
| gRPC impedance fit | Poor | Good | Poor (no typed API) |
| GraphQL fit | Medium | Excellent (wraps HttpClient) | Medium |
| Multi-transport connector (dYdX) | Awkward (Vec or nesting) | Natural (two fields) | Impossible for gRPC |
| Feature flag isolation | Breaks whole enum | Clean per-module | Breaks whole trait |
| Codebase disruption | High (all connectors) | Minimal | Medium |
| Implementation complexity | Medium | Low | High |
| Recommended | No | **YES** | No |

---

## 9. Sources

- [Bridging Worlds: How we Unified gRPC and REST APIs in Rust — Hyperswitch](https://github.com/juspay/hyperswitch/wiki/Bridging-Worlds:-How-we-Unified-gRPC-and-REST-APIs-in-Rust)
- [Tonic — native gRPC client/server for Rust (hyperium/tonic)](https://github.com/hyperium/tonic)
- [dYdX v4 Infrastructure Endpoints](https://docs.dydx.xyz/interaction/endpoints)
- [graphql-client — Typed GraphQL requests in Rust](https://github.com/graphql-rust/graphql-client)
- [reqwest-graphql — GraphQL over reqwest](https://docs.rs/reqwest-graphql/latest/reqwest_graphql/)
- [Cargo Features — conditional compilation](https://doc.rust-lang.org/cargo/reference/features.html)
- [Combining Axum, Hyper, Tonic, Tower for hybrid apps](https://academy.fpblock.com/blog/axum-hyper-tonic-tower-part3/)
- [Rust and gRPC: A complete guide — LogRocket](https://blog.logrocket.com/rust-and-grpc-a-complete-guide/)
- [Tower Service abstraction layer (Axum + Tonic)](https://leapcell.io/blog/unpacking-the-tower-abstraction-layer-in-axum-and-tonic)
