# gRPC Transport Integration Research — digdigdig3

**Date:** 2026-03-14
**Scope:** tonic + prost, dYdX v4 Cosmos gRPC, Tinkoff gRPC, Solana Yellowstone/Jito gRPC, Futu TCP+Protobuf, architecture guidance

---

## 1. tonic + prost: Core gRPC Stack

### Current Versions

| Crate | Latest Version | Notes |
|-------|---------------|-------|
| `tonic` | **0.14.5** | Client + server gRPC |
| `tonic-build` | **0.14.2** | Build-time code gen from .proto |
| `prost` | **0.13.x** | Protobuf serialization |
| `prost-types` | **0.13.x** | Well-known protobuf types |

**Version pairing rule:** tonic 0.12 pairs with prost 0.13; tonic 0.14 is the current stable release.

### Dependency Tree

tonic pulls in a significant HTTP/2 stack. With default features enabled:

```
tonic 0.14.5
├── hyper ^1                  (HTTP/2 client, optional — transport feature)
├── h2 ^0.4                   (HTTP/2 framing layer, optional — transport feature)
├── tower ^0.5                (middleware stack, optional — transport feature)
├── tokio ^1                  (async runtime, optional — transport feature)
├── tokio-rustls ^0.26.1      (TLS, optional — tls-ring / tls-aws-lc)
├── axum ^0.8                 (router, optional — router feature)
├── prost (via codegen)
├── bytes ^1.0
├── http ^1.1.0
├── http-body ^1
├── http-body-util ^0.1
├── pin-project ^1
├── tower-layer ^0.3
├── tower-service ^0.3
└── tracing ^0.1
```

**Compression (all optional):**
- `flate2 ^1.0` — gzip + deflate
- `zstd ^0.13.0` — zstd

**TLS backends (mutually exclusive, all optional):**
- `tokio-rustls ^0.26.1` — via `tls-ring` or `tls-aws-lc`
- `rustls-native-certs ^0.8` — via `tls-native-roots`
- `webpki-roots ^1` — via `tls-webpki-roots`

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `transport` | YES | Full HTTP/2 with hyper + tower + tokio |
| `router` | YES | Axum-based service routing |
| `codegen` | YES | Build tooling exports |
| `server` | NO | Server-only transport |
| `channel` | NO | Client-only transport |
| `gzip` | NO | gzip compression |
| `deflate` | NO | deflate compression |
| `zstd` | NO | zstd compression |
| `tls-ring` | NO | TLS via ring backend |
| `tls-aws-lc` | NO | TLS via aws-lc backend |
| `tls-native-roots` | NO | System root certificates |
| `tls-webpki-roots` | NO | Mozilla root certificates |
| `tls-connect-info` | NO | Expose TLS connection info |

### Binary Size and Compile Time Impact

**Binary size:** Adding tonic with transport + TLS adds approximately **3–5 MB** to a release binary (primarily from hyper, h2, rustls). In `--release` mode with `strip = true`, this is reduced but the transitive dependency tree is large (30+ new crates for first-time builds).

**Compile time:** tonic's full dependency tree (hyper + h2 + tower + rustls) adds roughly **45–90 seconds** of incremental compile time on a cold build of a workspace that does not already use these crates. Subsequent incremental builds are fast. The `tonic-build` code generation step is essentially instant for small .proto files.

**Key optimization:** If only the client transport is needed (no gRPC server), use `default-features = false` and only enable `transport` + `channel`. This avoids pulling in `axum`:

```toml
tonic = { version = "0.14", default-features = false, features = ["transport", "channel", "tls-native-roots"] }
```

### Feature-Gating in Workspace

Recommended pattern for optional gRPC support in a connector crate:

```toml
# Cargo.toml (connector crate)
[features]
grpc = ["dep:tonic", "dep:prost", "dep:prost-types"]

[dependencies]
tonic = { version = "0.14", optional = true, default-features = false, features = ["transport", "channel", "tls-native-roots"] }
prost = { version = "0.13", optional = true }
prost-types = { version = "0.13", optional = true }

[build-dependencies]
tonic-build = { version = "0.14", optional = true }
```

In Rust code, gate gRPC modules with `#[cfg(feature = "grpc")]`. The generated protobuf stubs from `tonic-build` should be placed in a `src/proto/` subdirectory and included conditionally.

---

## 2. dYdX v4: Cosmos gRPC

### Overview

dYdX v4 is a sovereign Cosmos SDK + CometBFT chain (`dydx-mainnet-1`). Order placement is **not** available via the REST indexer — it requires:
1. Construct a Cosmos SDK transaction containing `MsgPlaceOrder`
2. Sign with secp256k1 private key
3. Broadcast via gRPC to a validator node

**Read operations** (candles, orderbook, positions) use the REST indexer API or gRPC streaming. **Write operations** (place/cancel orders) use gRPC only.

### Proto File Locations

Main repository: `https://github.com/dydxprotocol/v4-chain`

Key proto paths:
```
proto/dydxprotocol/clob/tx.proto      # MsgPlaceOrder, MsgCancelOrder
proto/dydxprotocol/clob/order.proto   # Order message type
proto/dydxprotocol/subaccounts/       # SubaccountId
proto/dydxprotocol/clob/query.proto   # StreamOrderbookUpdates
```

Generate with: `make proto-gen && make proto-export-deps` → output in `.proto-export-deps/`

### MsgPlaceOrder Fields

```proto
// proto/dydxprotocol/clob/tx.proto
message MsgPlaceOrder {
  Order order = 1;
}

message Order {
  OrderId order_id = 1;        // Identifies the order uniquely
  Side side = 2;               // BUY or SELL
  uint64 quantums = 3;         // Size in base quantums (size / stepBaseQuantum)
  uint64 subticks = 4;         // Price in subticks (price * subticksPerTick)
  oneof good_til_oneof {
    uint32 good_til_block = 5;          // SHORT_TERM: expires after block height
    fixed32 good_til_block_time = 6;    // LONG_TERM/CONDITIONAL: UTC epoch seconds
  }
  Order.TimeInForce time_in_force = 7;
  uint32 reduce_only = 8;      // 1 = reduce-only
  uint32 client_metadata = 9;
  Order.ConditionType condition_type = 10;
  uint64 conditional_order_trigger_subticks = 11;
}

message OrderId {
  SubaccountId subaccount_id = 1;   // { owner: bech32_address, number: 0 }
  uint32 client_id = 2;             // Client-assigned identifier (random u32)
  uint32 order_flags = 3;           // 0=SHORT_TERM, 64=LONG_TERM, 32=CONDITIONAL
  uint32 clob_pair_id = 4;          // Market ID (e.g. 0 = BTC-USD, 1 = ETH-USD)
}
```

**SHORT_TERM constraint:** `currentBlockHeight < goodTilBlock <= currentBlockHeight + 20`

### MsgCancelOrder Fields

```proto
message MsgCancelOrder {
  OrderId order_id = 1;
  oneof good_til_oneof {
    uint32 good_til_block = 2;
    fixed32 good_til_block_time = 3;
  }
}
```

### gRPC Endpoints

| Network | Endpoint | Port | TLS |
|---------|----------|------|-----|
| Mainnet | `dydx-ops-grpc.kingnodes.com` | 443 | YES |
| Mainnet | `dydx-grpc.publicnode.com` | 443 | YES |
| Mainnet | `dydx-grpc.polkachu.com` | 23890 | NO |
| Testnet | Various community endpoints | 443 | YES |

Default gRPC port on a self-hosted node: **9090** (no TLS), **9091** (TLS optional)
gRPC streaming port: **9090** (same), WebSocket alt: **9092**

**Chain ID:** `dydx-mainnet-1`
**Fee denom:** `ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5` (USDC)

### Authentication and Signing

dYdX uses standard Cosmos SDK transaction signing:

1. **Key type:** secp256k1
2. **Sign mode:** `SIGN_MODE_DIRECT`
3. **Account data required:** account number + sequence number (fetched via gRPC `auth.Query/Account`)
4. **Transaction flow:**
   - Build `TxBody` with `[MsgPlaceOrder]`
   - Build `AuthInfo` with signer public key + fee + sequence
   - Create `SignDoc` = `{body_bytes, auth_info_bytes, chain_id, account_number, sequence}`
   - Sign `SignDoc` bytes with secp256k1 private key → `signature`
   - Build `TxRaw` = `{body_bytes, auth_info_bytes, [signature]}`
   - Broadcast via `cosmos.tx.v1beta1.Service/BroadcastTx`

### Existing Rust Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `dydx` on crates.io | Official Rust client (Nethermind) | Active, v0.2.0 |
| `cosmrs` | Cosmos SDK wallet + tx signing | Active, v0.19.x |
| `cosmos-sdk-proto` | Generated proto types | Active |
| `ibc-proto` | IBC proto types (cosmos-rust) | Active |
| `cosmos-grpc-client` | Wrapper using cosmrs + tonic | Community |

**Recommended dependency combo for custom implementation:**
```toml
cosmrs = { version = "0.19", features = ["rpc"] }
tonic = { version = "0.14", features = ["transport", "tls-native-roots"] }
prost = "0.13"
```

The `dydx` crate abstracts all of the above and is the fastest path to integration if the API surface matches the connector's needs. It uses `tonic` internally for gRPC.

### gRPC Streaming (Read-Only)

The `dydxprotocol.clob.Query/StreamOrderbookUpdates` service provides real-time orderbook streaming:

```proto
service Query {
  rpc StreamOrderbookUpdates(StreamOrderbookUpdatesRequest)
      returns (stream StreamOrderbookUpdatesResponse);
}
```

The stream delivers `StreamUpdate` variants:
- `StreamOrderbookUpdate` — order place/remove/update events, with snapshot flag
- `StreamOrderbookFill` — matched fills with `ClobMatch`
- `StreamTakerOrder` — taker orders entering matching loop
- `StreamSubaccountUpdate` — subaccount position changes

---

## 3. Tinkoff Invest gRPC

### Overview

Tinkoff (T-Bank) exposes a native gRPC API for all trading operations. There is no REST-only path for order placement; gRPC is the primary protocol. A REST/Swagger proxy exists as a compatibility layer but gRPC is the canonical interface.

### Connection Details

| Parameter | Value |
|-----------|-------|
| Production endpoint | `invest-public-api.tinkoff.ru:443` |
| Sandbox endpoint | `sandbox-invest-public-api.tinkoff.ru:443` |
| Protocol | gRPC over TLS (HTTP/2) |
| Auth method | `Authorization: Bearer <token>` metadata header |

**Note:** The domain was migrated from `invest-public-api.tinkoff.ru` to `invest-public-api.tbank.ru`. Both endpoints appear to work but `tbank.ru` is the current canonical name.

### Authentication

Bearer token is injected as gRPC metadata on every request via a tonic interceptor:

```rust
use tonic::service::interceptor;
use tonic::metadata::MetadataValue;

let token: MetadataValue<_> = format!("Bearer {}", api_token).parse()?;
let channel = tonic::transport::Channel::from_static(
    "https://invest-public-api.tbank.ru:443"
).connect().await?;

// Attach interceptor to every call
let channel = interceptor(channel, move |mut req: tonic::Request<()>| {
    req.metadata_mut().insert("authorization", token.clone());
    Ok(req)
});
```

**Optional headers:**
- `x-tracking-id` — unique UUID for each request (support tracing)
- `x-app-name` — `<github_user>.<repo_name>` format (analytics)

### Proto Files

Repository: `https://github.com/Tinkoff/investAPI` (legacy), `https://github.com/RussianInvestments/investAPI`

Proto files location: `src/docs/contracts/`

| File | Service / Purpose |
|------|------------------|
| `common.proto` | Shared types (MoneyValue, Quotation, etc.) |
| `instruments.proto` | InstrumentsService — lookup by FIGI, ticker, ISIN |
| `marketdata.proto` | MarketDataService — candles, orderbook, last prices; MarketDataStreamService |
| `orders.proto` | OrdersService — place, cancel, get orders; OrdersStreamService |
| `operations.proto` | OperationsService — portfolio, positions, transaction history |
| `stoporders.proto` | StopOrdersService — place/cancel stop-loss/take-profit |
| `sandbox.proto` | SandboxService — mirrors Orders/Operations for sandbox |
| `users.proto` | UsersService — account info, tariff, margin status |

Key streaming service: `MarketDataStreamService/MarketDataStream` (bidirectional streaming) and `OrdersStreamService/TradesStream` (server streaming for trade events).

### Existing Rust SDKs

| Crate | Tonic Version | Status | Notes |
|-------|-------------|--------|-------|
| `tinkoff-invest-api` (ovr) | 0.8.x | Outdated proto v2 | Not recommended |
| `investments-tinkoff` | 0.12 | More recent | Generated stubs |
| `invest-api-rust-sdk` | Unknown | Community | Claims interceptor support per-service |
| `tinkoff_invest` | Various | Several forks | Check freshness |

**Recommendation:** Given frequent proto file updates, prefer generating stubs directly from the official `.proto` files using `tonic-build` rather than relying on community crates that may lag behind the official API version.

Build pattern:
```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)  // client only
        .compile(
            &["proto/instruments.proto", "proto/orders.proto",
              "proto/marketdata.proto", "proto/operations.proto",
              "proto/stoporders.proto", "proto/users.proto",
              "proto/sandbox.proto", "proto/common.proto"],
            &["proto/"],
        )?;
    Ok(())
}
```

### Comparison: gRPC vs REST Proxy

The Tinkoff REST proxy (`api.tinkoff.ru/rest`) is a gRPC-gateway wrapper. For digdigdig3:
- **REST proxy:** Simpler to start, compatible with existing `reqwest`-based infrastructure, but adds serialization overhead and is secondary-class
- **Native gRPC:** Required for streaming (candles, orderbook, trades), lower latency, canonical interface
- **Verdict:** Use native gRPC for Tinkoff; the streaming capability alone justifies the dependency cost

---

## 4. Solana gRPC: Yellowstone / Jito

### Overview

Solana does not have a native gRPC API at the validator level. gRPC access is provided by two complementary plugin systems:

1. **Yellowstone gRPC (Dragon's Mouth)** — Triton One's Geyser plugin; used for account/transaction/slot streaming from a standard RPC node
2. **Jito MEV Protos** — Jito Labs' block engine gRPC for MEV-related features (bundle submission, shredstream)

These are **entirely separate systems** serving different purposes.

### Yellowstone gRPC

**Repository:** `https://github.com/rpcpool/yellowstone-grpc`
**Proto file:** `yellowstone-grpc-proto/proto/geyser.proto`

**Rust crates:**
```toml
yellowstone-grpc-client = "12.1.0"   # Released 2026-02-24
yellowstone-grpc-proto  = "12.1.0"
```

**Key dependency: tonic ^0.14.0** (matches the current stable tonic)

**Connection pattern:**
```rust
use yellowstone_grpc_client::GeyserGrpcClient;
use tonic::transport::ClientTlsConfig;

let client = GeyserGrpcClient::build_from_shared(endpoint)
    .x_token(token)?
    .tls_config(ClientTlsConfig::new().with_native_roots())?
    .connect()
    .await?;
```

**Subscription API (SubscribeRequest message):**
```
SubscribeRequest {
  accounts: HashMap<String, SubscribeRequestFilterAccounts>
  slots: HashMap<String, SubscribeRequestFilterSlots>
  transactions: HashMap<String, SubscribeRequestFilterTransactions>
  blocks: HashMap<String, SubscribeRequestFilterBlocks>
  blocks_meta: HashMap<String, SubscribeRequestFilterBlocksMeta>
  commitment: Option<CommitmentLevel>   // PROCESSED | CONFIRMED | FINALIZED
  accounts_data_slice: Vec<SubscribeRequestAccountsDataSlice>
  ping: Option<SubscribeRequestPing>   // keepalive
}
```

**Authentication:** Bearer token via `x-token` metadata header (not `Authorization`).

**Hosting options for Raydium price feeds:**
- QuickNode: $49/month for 1 stream
- Chainstack: marketplace pricing
- Helius: included in some plans
- Self-hosted: requires running a Solana validator with the Yellowstone plugin

**Raydium use case:** Subscribe to Raydium AMM account updates to detect price changes in real-time. Filter by the Raydium program ID (`675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`) and specific pool accounts.

### Jito gRPC

**Repository:** `https://github.com/jito-labs/mev-protos`
**Crate:** `jito-geyser-protos` (generated stubs, older approach), `jito-sdk-rust` (newer)

**Jito is relevant for digdigdig3 only if:**
- MEV bundle submission is needed (not a standard trading connector use case)
- Ultra-low-latency shredstream data is needed (shreds arrive before full block confirmation)

**For Raydium price feeds specifically:** Yellowstone is the right choice. Jito ShredStream is for sub-slot latency on specific high-frequency strategies and adds significant infrastructure complexity (requires proxied access to Jito's block engine).

**Conclusion:** For the digdigdig3 Raydium connector, use **Yellowstone only**. Jito protos are not required.

### Yellowstone vs REST Polling for Raydium

| Method | Latency | Cost | Complexity |
|--------|---------|------|------------|
| REST polling (RPC `getAccountInfo`) | 200–500ms per poll | Tied to RPC plan | Low (existing reqwest) |
| Yellowstone gRPC subscription | 50–150ms (push, per block) | $49+/mo or self-host | Medium (tonic dependency) |
| Jito ShredStream | 5–50ms (sub-slot) | High (Jito infrastructure) | High |

For non-HFT use cases, REST polling may be acceptable. For a proper real-time price connector, Yellowstone is the right tradeoff.

---

## 5. Futu: TCP + Protobuf (Not gRPC)

### Protocol Reality

Futu OpenD does **not** use gRPC. It is a proprietary TCP protocol with a custom binary frame format over a local or LAN connection. The OpenD process runs locally and acts as a gateway to Futu's servers.

### Wire Protocol Specification

**Header format (48 bytes, little-endian):**
```c
struct APIProtoHeader {
    u8  szHeaderFlag[2];  // Always "FT" (0x46, 0x54)
    u32 nProtoID;         // Protocol command identifier
    u8  nProtoFmtType;    // 0 = Protobuf, 1 = JSON
    u8  nProtoVer;        // Protocol version (currently 0)
    u32 nSerialNo;        // Incrementing request ID (used to match responses)
    u32 nBodyLen;         // Payload length in bytes
    u8  arrBodySHA1[20];  // SHA1 hash of body for integrity
    u8  arrReserved[8];   // Extension / padding
};
```

**Request message wrapper:**
```proto
message Request { required C2S c2s = 1; }
```

**Response message wrapper:**
```proto
message Response {
    required int32 retType = 1;   // 0 = OK, negative = error
    optional string retMsg = 2;   // Error description
    optional int32 errCode = 3;   // Error code
    optional S2C s2c = 4;         // Payload (service-specific)
}
```

### Connection Flow

1. Open TCP connection to `127.0.0.1:11111` (default OpenD port, configurable)
2. Send `InitConnect` request (ProtoID: 1001) — RSA-1024 encrypted with OpenD's public key
3. Receive `InitConnect` response containing AES symmetric key
4. All subsequent packets are AES-ECB encrypted with 16-byte alignment (padding byte = padding length)
5. Send heartbeat every N seconds (ProtoID: 1004) where N is specified in InitConnect response

### Protocol vs gRPC Comparison

| Aspect | Futu TCP+Protobuf | Standard gRPC |
|--------|------------------|---------------|
| Framing | Custom 48-byte header | HTTP/2 frames |
| Multiplexing | Serial number correlation | HTTP/2 stream ID |
| TLS | Optional AES-ECB (not TLS) | Standard TLS |
| Push | Subscription push | Server streaming |
| Port | 11111 (local) | 443 (remote) |

### Implementation Recommendation

**Do NOT use tonic for Futu.** The Futu protocol is not HTTP/2. Implement as:
- `tokio::net::TcpStream` for connection
- `prost` (without tonic) for protobuf encode/decode
- Custom framing layer for the 48-byte header + AES-ECB encryption
- Request/response correlation via `nSerialNo` map

**Protobuf-only dependency (no gRPC overhead):**
```toml
[features]
futu = ["dep:prost", "dep:prost-types", "dep:aes"]

[dependencies]
prost = { version = "0.13", optional = true }
aes = { version = "0.8", optional = true }  # AES-ECB for encryption
```

The Futu proto files are distributed with the OpenD download and available from the Python SDK at `https://github.com/FutunnOpen/py-futu-api/tree/master/futu/common/pb`.

---

## 6. Architecture: Mixed REST + gRPC Connectors

### The Problem

The digdigdig3 connector library currently uses `reqwest` for all HTTP-based connectors. Adding gRPC creates a second transport layer that should not be required for all connectors.

### Recommended Feature Flag Structure

```toml
# connectors-v5 top-level Cargo.toml (workspace member)
[features]
default = []

# Individual transport features
grpc = ["dep:tonic", "dep:prost", "dep:prost-types"]
grpc-tls = ["grpc", "tonic/tls-native-roots"]

# Per-connector feature flags
dydx-grpc = ["grpc-tls"]
tinkoff-grpc = ["grpc-tls"]
solana-yellowstone = ["dep:yellowstone-grpc-client", "dep:yellowstone-grpc-proto"]
futu = ["dep:prost"]  # custom TCP, no tonic

[dependencies]
# REST (always present)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# gRPC (optional)
tonic = { version = "0.14", optional = true, default-features = false, features = ["transport", "channel"] }
prost = { version = "0.13", optional = true }
prost-types = { version = "0.13", optional = true }

# Yellowstone (separate from raw tonic — it brings its own client)
yellowstone-grpc-client = { version = "12.1", optional = true }
yellowstone-grpc-proto = { version = "12.1", optional = true }
```

### dYdX Hybrid Architecture (REST reads + gRPC writes)

dYdX v4 has a clean split:
- **Indexer REST API** (`indexer.dydx.trade`) — candles, trades, positions, funding, account info (read-only, no auth)
- **Indexer WebSocket** (`wss://indexer.dydx.trade/v4/ws`) — real-time market data streaming
- **Validator gRPC** (`dydx-ops-grpc.kingnodes.com:443`) — order placement, cancellation, transaction broadcast

The connector module can be split into two sub-connectors sharing common types:

```
dydx/
├── mod.rs              # Public API, combines both
├── indexer.rs          # REST client (reqwest), always compiled
├── validator.rs        # gRPC client (tonic), #[cfg(feature = "dydx-grpc")]
├── types.rs            # Shared types
└── proto/              # Generated protobuf stubs
    └── mod.rs          # include!(concat!(env!("OUT_DIR"), "/dydxprotocol.clob.rs"))
```

This allows the connector registry to register dYdX even without gRPC (read-only mode), and enable order placement by activating the feature flag.

### Connector Registry Impact

The connector registry (`connector_manager/`) needs to handle connectors with partial capabilities based on active features. Pattern:

```rust
// connector.rs
impl OrderPlacement for DydxConnector {
    fn place_order(&self, order: &Order) -> BoxFuture<Result<OrderId>> {
        #[cfg(feature = "dydx-grpc")]
        { self.validator.place_order(order) }

        #[cfg(not(feature = "dydx-grpc"))]
        { Box::pin(async { Err(ConnectorError::UnsupportedOperation("dYdX order placement requires grpc feature".into())) }) }
    }
}
```

This is cleaner than a runtime check — capability is declared at compile time, zero cost when not used.

### Build Script Organization

For connectors that require `tonic-build` code generation, add a `build.rs` per connector (or a shared build script in the crate root):

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dydx-grpc")]
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/dydx/proto")
        .compile(
            &["proto/dydxprotocol/clob/tx.proto",
              "proto/dydxprotocol/clob/order.proto"],
            &["proto/"],
        )?;

    #[cfg(feature = "tinkoff-grpc")]
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/tinkoff/proto")
        .compile(
            &["proto/orders.proto", "proto/marketdata.proto",
              "proto/instruments.proto", "proto/operations.proto",
              "proto/common.proto"],
            &["proto/"],
        )?;

    Ok(())
}
```

**Alternative (avoids committing generated code):** Use `include!` macro with `OUT_DIR` path, which is the standard tonic pattern. The generated `.rs` files live in the build output directory, not in `src/`.

### Summary: Which Connectors Need What

| Connector | REST | WebSocket | gRPC | Custom TCP | Feature Flag |
|-----------|------|-----------|------|------------|-------------|
| dYdX v4 reads | YES (indexer) | YES (indexer WS) | NO | NO | (default) |
| dYdX v4 writes | NO | NO | YES (validator) | NO | `dydx-grpc` |
| Tinkoff | minimal (legacy) | NO | YES (all ops) | NO | `tinkoff-grpc` |
| Raydium (prices) | YES (fallback) | NO | YES (Yellowstone) | NO | `solana-yellowstone` |
| Futu | NO | NO | NO | YES (OpenD) | `futu` |

---

## 7. Key Decisions Summary

1. **tonic 0.14 + prost 0.13** — use this version pair; it is current stable as of early 2026
2. **Client-only features** — always use `default-features = false` and opt into only `transport` + `channel` + TLS to avoid pulling in axum
3. **dYdX Rust SDK** — the official `dydx` crate (Nethermind) is the fastest path; for custom integration use `cosmrs` + `tonic` + generated proto stubs
4. **Tinkoff** — generate stubs from official proto files using `tonic-build`; do not rely on community crates which lag behind API versions; use tonic interceptor for bearer token injection
5. **Raydium** — use `yellowstone-grpc-client 12.1.0` (already uses tonic 0.14, consistent versions); Jito is not needed for price data
6. **Futu** — NOT gRPC; implement custom TCP framer with `prost` for serialization only, no tonic
7. **Feature gating** — gate all gRPC code behind features; REST-only builds should have zero gRPC dependency

---

## Sources

- [tonic crates.io](https://crates.io/crates/tonic)
- [tonic docs.rs (latest)](https://docs.rs/tonic/latest/tonic/)
- [tonic GitHub (hyperium/tonic)](https://github.com/hyperium/tonic)
- [dYdX v4-chain GitHub](https://github.com/dydxprotocol/v4-chain)
- [dYdX v4-clients GitHub](https://github.com/dydxprotocol/v4-clients)
- [dYdX Quick Start (Rust)](https://docs.dydx.xyz/interaction/client/quick-start-rs)
- [dYdX Validator Client docs](https://docs.dydx.exchange/api_integration-clients/validator_client)
- [dYdX Full Node gRPC Streaming](https://docs.dydx.exchange/api_integration-full-node-streaming)
- [dYdX Endpoints](https://docs.dydx.xyz/interaction/endpoints)
- [Tinkoff investAPI gRPC docs](https://tinkoff.github.io/investAPI/grpc/)
- [Tinkoff investAPI GitHub (legacy)](https://github.com/Tinkoff/investAPI)
- [RussianInvestments investAPI GitHub](https://github.com/RussianInvestments/invest-js)
- [tinkoff-invest-api crate](https://docs.rs/tinkoff-invest-api/latest/tinkoff_invest_api/)
- [yellowstone-grpc GitHub (rpcpool)](https://github.com/rpcpool/yellowstone-grpc)
- [yellowstone-grpc-client docs.rs](https://docs.rs/crate/yellowstone-grpc-client/latest)
- [QuickNode Yellowstone Rust Guide](https://www.quicknode.com/docs/solana/yellowstone-grpc/overview/rust)
- [Futu API Protocol Introduction](https://openapi.futunn.com/futu-api-doc/en/ftapi/protocol.html)
- [cosmrs crates.io](https://crates.io/crates/cosmrs)
- [cosmrs tx module docs](https://docs.rs/cosmrs/latest/cosmrs/tx/index.html)
- [cosmos-rust GitHub](https://github.com/cosmos/cosmos-rust)
- [ibc-proto-rs GitHub](https://github.com/cosmos/ibc-proto-rs)
- [Jito mev-protos GitHub](https://github.com/jito-labs/mev-protos)
