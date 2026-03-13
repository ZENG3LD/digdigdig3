# GraphQL Transport Integration Research
# digdigdig3 — Uniswap, Bitquery, GMX

**Date:** 2026-03-14
**Scope:** GraphQL crate selection, WebSocket subscription protocols, The Graph specifics,
Bitquery API, GMX subgraph endpoints, and a minimal architecture recommendation for digdigdig3.

---

## Table of Contents

1. [GraphQL Rust Crates Comparison](#1-graphql-rust-crates-comparison)
2. [GraphQL over WebSocket Protocols](#2-graphql-over-websocket-protocols)
3. [The Graph — Uniswap and GMX Subgraphs](#3-the-graph--uniswap-and-gmx-subgraphs)
4. [Bitquery API](#4-bitquery-api)
5. [Architecture Recommendation for digdigdig3](#5-architecture-recommendation-for-digdigdig3)
6. [Cargo.toml Additions](#6-cargotoml-additions)
7. [Sources](#7-sources)

---

## 1. GraphQL Rust Crates Comparison

### 1.1 Overview of Candidates

There are three main options for handling GraphQL in Rust, plus the raw-reqwest approach.

| Crate | Version (2026-01) | Approach | Schema Required? | Compile Overhead |
|-------|-------------------|----------|-----------------|-----------------|
| `graphql-client` | 0.16.0 | Query-first codegen (proc-macro) | Yes (.graphql + schema.json) | High (proc-macro per query) |
| `cynic` | 3.x | Type-first (structs → GQL) | Yes (at compile time) | Medium (derive macros) |
| `gql_client` | 0.9.x | Runtime-only, no codegen | No | Minimal |
| Raw `reqwest` + `serde_json` | N/A | POST with JSON body manually | No | Zero |

### 1.2 graphql-client

**How it works:** You write a `.graphql` query file and point a `#[derive(GraphQLQuery)]` proc-macro
at it plus a schema introspection JSON. At compile time it generates typed Rust structs for both
variables and responses. The `Response<T>` wrapper is deserialized via `serde`.

**Pros:**
- Full type safety — compile error if query does not match schema.
- Supports fragments, unions, enums, custom scalars, deprecation checks.
- Version 0.16.0 (January 2026) — actively maintained.
- Works with `graphql-ws-client` for subscriptions (feature flag `graphql-client`).

**Cons:**
- Requires schema files checked into the repo (`.graphql` + `schema.json`).
- Each `#[derive(GraphQLQuery)]` invocation has proc-macro cost at build time.
- When schemas vary across providers (Uniswap vs Bitquery vs GMX), you need separate schema
  files for each, plus separate query files. Build graph gets complex.
- Codegen approach means changes to remote schemas require re-introspection and re-running codegen.

### 1.3 cynic

**How it works:** Inverted approach — you write Rust structs first and the library derives the
GraphQL query document from them. Uses `querygen` tool to bootstrap structs from an existing GQL
query string. Supports dynamic (runtime) query building while still type-checking against schema.

**Pros:**
- "Bring your own types" — better control over struct layout.
- Supports sharing structs between multiple queries.
- Runtime dynamic query construction (unusual for a typed GQL client).
- Also integrates with `graphql-ws-client`.

**Cons:**
- Still requires schema access at compile time for full type checking.
- Less mature ecosystem; subscriptions described as "fairly alpha quality" in docs.
- Inverted mental model is unfamiliar — more learning curve.

### 1.4 gql_client

**How it works:** Lightweight runtime-only crate. Takes a query string + variables as a `HashMap`
and returns `serde_json::Value` or a custom deserializable type. No codegen, no schema files.

**Pros:**
- Zero compile-time overhead — no proc-macros.
- No schema files needed.
- Good for prototyping or providers where schema is unavailable or changes often.

**Cons:**
- Last updated: 2022-ish. Maintenance status unclear/stale.
- No type safety — schema divergence discovered at runtime only.
- No built-in WebSocket/subscription support.

### 1.5 Raw reqwest + serde_json (Recommended for Queries)

**How it works:** GraphQL over HTTP is simply a POST with `Content-Type: application/json` and a
body of `{"query": "...", "variables": {...}}`. This is plain JSON — no special protocol.

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct GqlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

#[derive(Deserialize)]
struct GqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

// Usage:
let body = GqlRequest {
    query: "{ pools(first: 10) { id token0 { symbol } token1 { symbol } } }",
    variables: None,
};
let resp: GqlResponse<PoolsData> = client
    .post("https://gateway.thegraph.com/api/KEY/subgraphs/id/SUBGRAPH_ID")
    .json(&body)
    .send()
    .await?
    .json()
    .await?;
```

**Pros:**
- Zero additional dependencies (reqwest already in digdigdig3).
- Zero compile-time overhead.
- No schema files to maintain.
- Perfectly consistent with the existing `HttpClient` design in `core/http/client.rs`
  (which already returns `serde_json::Value`).
- Works for ALL three providers (Uniswap, Bitquery HTTP queries, GMX).

**Cons:**
- No compile-time schema validation — query errors discovered at runtime.
- Query strings must be kept in Rust string literals or separate `.graphql` files read at runtime.

### 1.6 Decision Matrix for digdigdig3

digdigdig3 uses the **raw `serde_json::Value` at the transport layer by design** (see
`RESEARCH_TRANSPORT_ARCHITECTURE.md`). The `HttpClient` already returns untyped `Value`. Adding
`graphql-client` codegen on top would be architectural friction — it generates typed structs but
the transport layer throws them away. The schema-varies-per-provider problem also makes codegen
cumbersome: three separate schema files, three separate query directories, complex build.rs.

**Recommendation: Raw reqwest POST for queries. For subscriptions: `graphql-ws-client` + `tokio-tungstenite`.**

---

## 2. GraphQL over WebSocket Protocols

### 2.1 Two Competing Protocols

There are two incompatible WebSocket subprotocols used for GraphQL subscriptions:

| Protocol | Subprotocol Header | Status | Notes |
|----------|--------------------|--------|-------|
| `subscriptions-transport-ws` | `graphql-ws` | **Deprecated** | Apollo's original; unmaintained since ~2018 |
| `graphql-transport-ws` | `graphql-transport-ws` | **Active standard** | Modern replacement by enisdenjo/graphql-ws |

The naming is confusingly inverted: the deprecated library is called `subscriptions-transport-ws`
but uses the `graphql-ws` subprotocol header. The modern library is called `graphql-ws` but
uses the `graphql-transport-ws` subprotocol header.

### 2.2 Protocol Message Flow (graphql-transport-ws)

```
Client → Server: ConnectionInit { payload: { Authorization: "Bearer ..." } }
Server → Client: ConnectionAck {}
Client → Server: Subscribe { id: "1", payload: { query: "subscription { ... }" } }
Server → Client: Next { id: "1", payload: { data: {...} } }
Server → Client: Next { id: "1", payload: { data: {...} } }
...
Client → Server: Complete { id: "1" }
Server → Client: Complete { id: "1" }

-- Keep-alive --
Server → Client: Ping {}
Client → Server: Pong {}
```

### 2.3 Protocol Message Flow (legacy subscriptions-transport-ws / graphql-ws subprotocol)

```
Client → Server: connection_init
Server → Client: connection_ack
Server → Client: ka  (keep-alive, sent every N seconds)
Client → Server: start { id: "1", payload: { query: "subscription { ... }" } }
Server → Client: data { id: "1", payload: { data: {...} } }
...
Client → Server: stop { id: "1" }
```

### 2.4 Which Protocol Does Bitquery Use?

Bitquery supports **both** protocols. Their documentation confirms:

- Modern: `graphql-transport-ws` — server sends `pong` for keep-alive
- Legacy: `graphql-ws` — server sends `ka` (keep-alive) messages

The WebSocket endpoint is the same for both:
```
wss://streaming.bitquery.io/graphql
```

The Sec-WebSocket-Protocol header selects which subprotocol to use:
```
Sec-WebSocket-Protocol: graphql-transport-ws   # modern
Sec-WebSocket-Protocol: graphql-ws             # legacy
```

Bitquery's official Rust example uses `async-tungstenite` + `graphql-ws-client` which defaults
to `graphql-transport-ws`. This is the recommended path.

**Important Bitquery-specific note:** Cannot send WebSocket `close` messages. The only way to end
a subscription is to drop the WebSocket connection entirely.

### 2.5 graphql-ws-client Crate

**Repository:** https://github.com/obmarg/graphql-ws-client (migrated to Codeberg, GitHub is read-only)
**Version:** 0.12.0 (January 2026)
**Protocol:** Implements `graphql-transport-ws` (the modern standard)

**Feature flags:**
```toml
graphql-ws-client = { version = "0.12.0", features = [
    "client-graphql-client",    # enables graphql-client integration
    # OR
    "client-cynic",             # enables cynic integration
    "tungstenite-0-26",         # tokio-tungstenite version matching
] }
```

WebSocket backends supported:
- `async-tungstenite`
- `tokio-tungstenite` (versions 0.23–0.28 via feature flags)
- `ws-stream-wasm`

**Key API:**
```rust
use graphql_ws_client::Client;
use tokio_tungstenite::connect_async;

let (ws_stream, _) = connect_async(request).await?;
let (sink, stream) = ws_stream.split();

let mut gql_client = Client::build(graphql_ws_client::tungstenite::Connection::new(sink, stream))
    .subscribe(subscription_operation)
    .await?;

while let Some(response) = gql_client.next().await {
    // process response
}
```

### 2.6 Can We Use Raw tokio-tungstenite Without graphql-ws-client?

Yes, entirely feasible. The protocol is simple enough to implement manually. The subscription
message format is just JSON over WebSocket frames:

```rust
// Send ConnectionInit with auth
ws_sink.send(Message::Text(serde_json::to_string(&json!({
    "type": "connection_init",
    "payload": { "Authorization": format!("Bearer {}", token) }
}))?)).await?;

// Send Subscribe
ws_sink.send(Message::Text(serde_json::to_string(&json!({
    "type": "subscribe",
    "id": "1",
    "payload": { "query": "subscription { ... }", "variables": {} }
}))?)).await?;

// Read loop
while let Some(msg) = ws_stream.next().await {
    let text = msg?.into_text()?;
    let val: serde_json::Value = serde_json::from_str(&text)?;
    match val["type"].as_str() {
        Some("next") => { /* val["payload"]["data"] */ }
        Some("complete") => break,
        Some("ping") => { ws_sink.send(pong_msg).await?; }
        Some("connection_ack") => { /* proceed to subscribe */ }
        _ => {}
    }
}
```

**Trade-offs of manual implementation:**
- No extra crate dependency (digdigdig3 already has `tokio-tungstenite` via existing WebSocket
  infrastructure in `core/websocket/base_websocket.rs`).
- More boilerplate but transparent control — fits digdigdig3's pattern of minimal external deps.
- Consistent with existing `BaseWebSocket` architecture.

**Recommendation:** Prefer manual raw protocol for Bitquery subscriptions using existing
`BaseWebSocket` infrastructure. Only add `graphql-ws-client` if subscription complexity grows
(multiplexed subs, reconnect with sub recovery, etc.).

---

## 3. The Graph — Uniswap and GMX Subgraphs

### 3.1 Architecture Overview

The Graph is a decentralized indexing protocol for blockchain data. Subgraphs are GraphQL APIs
that index on-chain events into queryable entities. Hosted service has been deprecated; all
production queries now go through the decentralized network.

**Query URL format:**
```
https://gateway.thegraph.com/api/<API_KEY>/subgraphs/id/<SUBGRAPH_ID>
```

**Authentication:** API key embedded in URL path (not a header). Obtain from:
https://thegraph.com/studio/apikeys/

**Billing:**
- Free tier: 100,000 queries/month (Subgraph Studio testing endpoint)
- Production network: ~$2 per 100,000 queries / ~$0.00002 per query
- Payment: GRT token or credit card
- No explicit per-second rate limit documented, but Studio endpoint is rate-limited for testing

### 3.2 Uniswap V3 Subgraph

**Subgraph IDs:**
- V2 Mainnet: `A3Np3RQbaBA6oKJgiwDJeo5T3zrYfGHPWFYayMwtNDum`
- V3 Mainnet: `5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV`

**Endpoints:**
```
# V2
https://gateway.thegraph.com/api/<API_KEY>/subgraphs/id/A3Np3RQbaBA6oKJgiwDJeo5T3zrYfGHPWFYayMwtNDum

# V3
https://gateway.thegraph.com/api/<API_KEY>/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV
```

**Available entities (V3):**

| Entity | Description |
|--------|-------------|
| `pool` | Pool state — tick, sqrtPrice, liquidity, feeTier, token0, token1 |
| `pools` | All pools, filterable, paginated |
| `swap` | Single swap — sender, recipient, amount0, amount1, timestamp |
| `swaps` | Swap history per pool, orderable by timestamp |
| `token` | Token metadata — symbol, name, decimals, volumeUSD, poolCount |
| `tokenDayDatas` | Token daily aggregated data |
| `poolDayDatas` | Pool daily OHLC-equivalent — date, token0Price, token1Price, volumeToken0, volumeToken1, liquidity |
| `poolHourDatas` | Pool hourly data |
| `bundle` | ETH price |

**Pagination pattern:**
```graphql
# Max 1000 entities per query. Use skip for pages.
query pools($skip: Int!) {
  pools(
    first: 1000,
    skip: $skip,
    orderDirection: asc
  ) {
    id
    sqrtPrice
    token0 { id symbol }
    token1 { id symbol }
    feeTier
  }
}
```

**Pool swap history query:**
```graphql
{
  swaps(
    orderBy: timestamp,
    orderDirection: desc,
    where: { pool: "0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf" }
  ) {
    timestamp
    sender
    recipient
    amount0
    amount1
    transaction { id blockNumber gasUsed gasPrice }
    token0 { symbol }
    token1 { symbol }
  }
}
```

**Pool daily OHLC-equivalent query:**
```graphql
{
  poolDayDatas(
    first: 100,
    orderBy: date,
    orderDirection: asc,
    where: {
      pool: "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8",
      date_gt: 1633642435
    }
  ) {
    date
    liquidity
    sqrtPrice
    token0Price
    token1Price
    volumeToken0
    volumeToken1
  }
}
```

**Pagination implementation in Rust:**
```rust
let mut skip = 0usize;
let mut all_pools = vec![];
loop {
    let query = format!(r#"
        query {{
          pools(first: 1000, skip: {skip}, orderDirection: asc) {{
            id token0 {{ symbol }} token1 {{ symbol }} feeTier
          }}
        }}
    "#);
    let resp = http_client.post(endpoint, &query_body).await?;
    let batch: Vec<Pool> = serde_json::from_value(resp["data"]["pools"].clone())?;
    if batch.is_empty() { break; }
    skip += batch.len();
    all_pools.extend(batch);
}
```

### 3.3 GMX Subgraphs

GMX has multiple subgraphs split by chain and data type. The main repository is
`github.com/gmx-io/gmx-subgraph` with subdirectories per deployment.

**Known subgraph IDs from The Graph Explorer:**

| Subgraph | Chain | ID |
|---------|-------|----|
| GMX Avalanche (V1) | Avalanche | `6pXgnXcL6mkXBjKX7NyHN7tCudv2JGFnXZ8wf8WbjPXv` |
| GMX V2 (Synthetics) | Arbitrum One | `F8JuJQQuDYoXkM3ngneRnrL9RA7sT5DjL6kBZE1nJZc3` |

**Endpoint pattern:**
```
https://gateway.thegraph.com/api/<API_KEY>/subgraphs/id/<SUBGRAPH_ID>
```

**For Arbitrum gateway specifically:**
```
https://gateway-arbitrum.network.thegraph.com/api/<API_KEY>/subgraphs/id/<SUBGRAPH_ID>
```

**Alternative — Subsquid endpoint (GMX Synthetics on Arbitrum):**
```
https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql
```
No API key required for Subsquid, but it is a third-party indexer (not The Graph).

**Key GMX V2 events indexed:**
- `IncreasePosition` — position opened/increased
- `DecreasePosition` — position partially closed
- `ClosePosition` — position fully closed
- `LiquidatePosition` — liquidation event
- `UpdatePosition` — tracks all position state changes

**Example GMX positions query (V2 Arbitrum):**
```graphql
{
  positionIncreases(
    first: 100,
    orderBy: timestamp,
    orderDirection: desc,
    where: { account: "0xYOUR_ADDRESS" }
  ) {
    account
    market
    collateralToken
    sizeDeltaUsd
    collateralDeltaAmount
    executionPrice
    timestamp
    transaction { id }
  }
}
```

**GMX V1 subgraph repo subdirectories:**
- `gmx-arbitrum-raw` — raw events (position increases, decreases, swaps)
- `gmx-arbitrum-stats` — aggregated stats (fees, volume, OI by day)
- `gmx-arbitrum-prices` — price feed data
- `gmx-avalanche-*` — same structure for Avalanche

**Note:** The V1 Arbitrum subgraph ID is not directly visible in The Graph explorer search results
as of 2026-03 (it may have been migrated or become stale). The official GMX stats frontend at
`stats.gmx.io` queries these subgraphs. For V1 Arbitrum, the safest approach is to:
1. Check `github.com/gmx-io/gmx-subgraph` README for current deployment IDs.
2. Or use Bitquery's GMX-specific queries as an alternative data source.

---

## 4. Bitquery API

### 4.1 Overview

Bitquery is a blockchain analytics platform providing a unified GraphQL API for 40+ blockchains.
V2 ("Streaming API") uses OAuth2 authentication and supports both HTTP queries and WebSocket
subscriptions through the same endpoint.

**Key difference from V1:** V2 uses OAuth2 Bearer tokens (format: `ory_at_...`) instead of V1's
simple API key header.

### 4.2 Endpoints

| Transport | Endpoint |
|-----------|----------|
| HTTP GraphQL queries | `https://streaming.bitquery.io/graphql` |
| WebSocket subscriptions | `wss://streaming.bitquery.io/graphql` |
| OAuth2 token generation | `https://oauth2.bitquery.io/oauth2/token` |

### 4.3 Authentication — OAuth2 Token Flow

**Step 1: Create application credentials**
1. Go to https://account.bitquery.io/user/api_v2/applications
2. Create application, set token expiration.
3. Retrieve `client_id` and `client_secret`.

**Step 2: Acquire access token (POST)**
```
POST https://oauth2.bitquery.io/oauth2/token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=YOUR_CLIENT_ID
&client_secret=YOUR_CLIENT_SECRET
&scope=api
```

**Response:**
```json
{
  "access_token": "ory_at_sKK8sSq8...",
  "expires_in": 2627999,
  "scope": "api",
  "token_type": "bearer"
}
```

Token lifetime: variable (up to ~30 days for application credentials). Without an application,
the default bearer token rotates every 12 hours.

**Step 3a: Use token for HTTP queries**
```
POST https://streaming.bitquery.io/graphql
Authorization: Bearer ory_at_sKK8sSq8...
Content-Type: application/json

{ "query": "{ EVM { DEXTradeByTokens { ... } } }" }
```

**Step 3b: Use token for WebSocket subscriptions**
```
wss://streaming.bitquery.io/graphql?token=ory_at_sKK8sSq8...
```
Or as URL parameter (both methods supported). Headers required:
```
Sec-WebSocket-Protocol: graphql-transport-ws
Content-Type: application/json
```

### 4.4 HTTP Query Example — DEX Trades (V2 EVM API)

```graphql
{
  EVM(network: eth) {
    DEXTradeByTokens(
      orderBy: { descendingByField: "Block_Number" }
      limit: { count: 100 }
      where: {
        Trade: {
          Currency: { SmartContract: { is: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" } }
        }
      }
    ) {
      Block { Number }
      Trade {
        Amount
        AmountInUSD
        Buyer
        Seller
        Price
        PriceInUSD
        Currency { Symbol SmartContract }
        Side { Currency { Symbol } Amount }
        Dex { SmartContract ProtocolName ProtocolFamily }
      }
    }
  }
}
```

### 4.5 WebSocket Subscription Example — Solana DEX Trades

```graphql
subscription {
  Solana {
    DEXTradeByTokens(
      where: {
        Trade: {
          Dex: { ProgramAddress: { is: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" } }
        }
      }
    ) {
      Block { Time Slot }
      Trade {
        Amount
        Price
        Currency { Symbol MintAddress }
        Side { Amount Currency { Symbol } }
        Dex { ProgramAddress ProtocolName }
      }
    }
  }
}
```

### 4.6 Rust Implementation (from Bitquery Official Docs)

Official Cargo.toml dependencies for the Bitquery Rust WebSocket example:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
graphql-client = "0.10.0"
async-tungstenite = "*"
graphql-ws-client = "*"
eyre = "*"
dotenv = "*"
```

The official example uses `graphql-client` for query definition and `graphql-ws-client` for the
WebSocket transport. However, as noted in Section 1, for digdigdig3 the raw approach is preferred.

**Bitquery GitHub repo for Rust streaming example:**
https://github.com/bitquery/graphql-streaming-example-rs/

### 4.7 Keep-Alive and Reconnect Requirements

Bitquery WebSocket connections require reconnect logic. If no data or keep-alive message is
received for ~10 seconds, the connection should be treated as dead and reconnected. The keep-alive
signal:
- `graphql-transport-ws` protocol: server sends `Ping`, client must reply with `Pong`
- `graphql-ws` (legacy) protocol: server sends `ka` message (no reply needed)

Bitquery WebSocket **cannot send `close` messages** — dropping the connection is the only way
to end a subscription.

### 4.8 No Official Rust SDK

Bitquery has no official Rust SDK. The approaches are:
1. Raw `reqwest` POST for HTTP queries.
2. Raw `tokio-tungstenite` with manual protocol for WebSocket subscriptions.
3. `graphql-ws-client` + `async-tungstenite` (as shown in official docs example).

---

## 5. Architecture Recommendation for digdigdig3

### 5.1 Summary of Transport Requirements

| Provider | Query Type | Transport Needed |
|----------|-----------|-----------------|
| Uniswap (The Graph) | Historical DEX data | HTTP POST (JSON body) |
| GMX (The Graph) | Historical positions/trades | HTTP POST (JSON body) |
| Bitquery (queries) | Blockchain analytics | HTTP POST (JSON body) |
| Bitquery (streaming) | Real-time trade feed | WebSocket + graphql-transport-ws |

**Key insight:** GraphQL queries (not subscriptions) = plain HTTP POST. No new transport crate
needed. The existing `HttpClient` in `core/http/client.rs` handles this perfectly.

### 5.2 Recommended Module Structure

```
onchain/
├── graphql/
│   ├── mod.rs          # GqlClient wrapper over HttpClient
│   ├── types.rs        # GqlRequest, GqlResponse<T>, GqlError
│   └── pagination.rs   # Paginator<T> — handles skip/first loop
├── uniswap/
│   ├── mod.rs
│   ├── queries.rs      # Query string constants (V2 + V3)
│   ├── parser.rs       # Typed structs for Pool, Swap, PoolDayData
│   └── connector.rs    # UniswapConnector implementing data traits
├── gmx/
│   ├── mod.rs
│   ├── queries.rs      # Query string constants
│   ├── parser.rs       # Typed structs for Position, Trade, Liquidation
│   └── connector.rs
└── bitquery/
    ├── mod.rs
    ├── auth.rs         # OAuth2 token acquisition + refresh
    ├── queries.rs      # Query string constants
    ├── parser.rs
    ├── connector.rs    # HTTP query connector
    └── subscription.rs # WebSocket streaming connector
```

### 5.3 GqlClient Wrapper

A thin wrapper over `HttpClient` that handles GraphQL-specific concerns:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
pub struct GqlRequest<'a> {
    pub query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
}

#[derive(Deserialize)]
pub struct GqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize, Debug)]
pub struct GqlError {
    pub message: String,
    pub locations: Option<Vec<GqlLocation>>,
}

#[derive(Deserialize, Debug)]
pub struct GqlLocation {
    pub line: u32,
    pub column: u32,
}

pub struct GqlClient {
    inner: HttpClient,    // existing digdigdig3 HttpClient
    endpoint: String,
}

impl GqlClient {
    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, ExchangeError> {
        let body = GqlRequest { query, variables };
        let resp: GqlResponse<T> = self.inner
            .post_json(&self.endpoint, &body)
            .await?;
        if let Some(errors) = resp.errors {
            return Err(ExchangeError::ApiError(errors[0].message.clone()));
        }
        resp.data.ok_or(ExchangeError::EmptyResponse)
    }
}
```

### 5.4 Pagination Helper

```rust
pub struct Paginator<T> {
    client: GqlClient,
    query_template: String,
    batch_size: usize,
}

impl<T: for<'de> Deserialize<'de>> Paginator<T> {
    pub async fn collect_all(&self, entity_key: &str) -> Result<Vec<T>, ExchangeError> {
        let mut results = vec![];
        let mut skip = 0usize;
        loop {
            let vars = serde_json::json!({
                "first": self.batch_size,
                "skip": skip
            });
            let page: Value = self.client.query(&self.query_template, Some(vars)).await?;
            let batch: Vec<T> = serde_json::from_value(page[entity_key].clone())?;
            if batch.is_empty() { break; }
            skip += batch.len();
            results.extend(batch);
            if batch.len() < self.batch_size { break; }
        }
        Ok(results)
    }
}
```

### 5.5 Bitquery Subscription via Existing BaseWebSocket

The existing `BaseWebSocket` in `core/websocket/base_websocket.rs` handles auto-reconnect and
ping/pong already. The Bitquery subscription can be implemented by extending it to speak
`graphql-transport-ws`:

```rust
// In bitquery/subscription.rs

const GRAPHQL_TRANSPORT_WS: &str = "graphql-transport-ws";

struct BitqueryWsConfig {
    token: String,
    subscription_query: String,
}

// On connect, send connection_init with auth payload
async fn on_connect(sink: &mut WsSink, config: &BitqueryWsConfig) -> Result<()> {
    let init = serde_json::json!({
        "type": "connection_init",
        "payload": { "Authorization": format!("Bearer {}", config.token) }
    });
    sink.send(Message::Text(init.to_string())).await?;
    Ok(())
}

// After connection_ack, send subscribe
async fn on_ack(sink: &mut WsSink, sub_id: &str, query: &str) -> Result<()> {
    let sub = serde_json::json!({
        "type": "subscribe",
        "id": sub_id,
        "payload": { "query": query }
    });
    sink.send(Message::Text(sub.to_string())).await?;
    Ok(())
}
```

### 5.6 Decision: graphql-ws-client vs Manual Protocol

**Use `graphql-ws-client` if:**
- Multiplexing multiple subscriptions over a single connection is needed.
- Subscription recovery after reconnect is required (re-sends Subscribe messages on reconnect).
- Team prefers a crate API over raw JSON assembly.

**Use manual protocol (raw tokio-tungstenite) if:**
- Minimizing dependencies is a priority.
- Only one subscription per connection is needed (Bitquery typical usage).
- Consistent with existing `BaseWebSocket` code style.
- Fine-grained control over keep-alive and reconnect behavior matters.

**For digdigdig3:** Start with manual protocol. It is 30-50 lines of code and fits the existing
architecture. Upgrade to `graphql-ws-client` if complexity grows.

### 5.7 No New Crates Needed for Query-Only Providers

For Uniswap and GMX (The Graph), which are query-only (no subscriptions):

```toml
# No new dependencies required in Cargo.toml
# These are already present in digdigdig3:
# - reqwest (HTTP client)
# - serde / serde_json (JSON serialization)
# - tokio (async runtime)
```

For Bitquery WebSocket subscriptions, if using manual approach:
```toml
# Also already present:
# - tokio-tungstenite (via existing BaseWebSocket)
# - futures (StreamExt)
```

Only if using `graphql-ws-client`:
```toml
graphql-ws-client = { version = "0.12.0", features = ["tungstenite-0-26"] }
```

---

## 6. Cargo.toml Additions

### 6.1 Minimal (queries only, no new deps)

```toml
# No additions needed — all covered by existing reqwest + serde_json + tokio
```

### 6.2 With graphql-ws-client for Subscriptions

```toml
[dependencies]
graphql-ws-client = { version = "0.12.0", optional = true, features = ["tungstenite-0-26"] }

[features]
bitquery-streaming = ["graphql-ws-client"]
```

### 6.3 Feature Gate Recommendation

```toml
[features]
default = []
uniswap = []         # HTTP queries via raw reqwest — no new deps
gmx = []             # HTTP queries via raw reqwest — no new deps
bitquery = []        # HTTP queries via raw reqwest — no new deps
bitquery-streaming = ["dep:graphql-ws-client"]  # WebSocket subscriptions
```

---

## 7. Sources

- [Uniswap Subgraph Overview](https://docs.uniswap.org/api/subgraph/overview)
- [Uniswap V3 Query Examples](https://docs.uniswap.org/api/subgraph/guides/v3-examples)
- [The Graph: Querying from an Application](https://thegraph.com/docs/en/subgraphs/querying/from-an-application/)
- [The Graph Billing Documentation](https://thegraph.com/docs/en/subgraphs/billing/)
- [The Graph Studio Pricing](https://thegraph.com/studio-pricing/)
- [GMX Subgraph Repository](https://github.com/gmx-io/gmx-subgraph)
- [GMX V2 on The Graph Explorer (Arbitrum)](https://thegraph.com/explorer/subgraphs/F8JuJQQuDYoXkM3ngneRnrL9RA7sT5DjL6kBZE1nJZc3?view=About&chain=arbitrum-one)
- [GMX Avalanche on The Graph Explorer](https://thegraph.com/explorer/subgraphs/6pXgnXcL6mkXBjKX7NyHN7tCudv2JGFnXZ8wf8WbjPXv?view=Query&chain=arbitrum-one)
- [GMX REST API Docs](https://docs.gmx.io/docs/api/rest/)
- [GMX Synthetics Subsquid Playground](https://gmx.squids.live/gmx-synthetics-arbitrum/graphql)
- [Bitquery WebSocket Docs](https://docs.bitquery.io/docs/subscriptions/websockets/)
- [Bitquery Rust WebSocket Example](https://docs.bitquery.io/docs/subscriptions/example-rust/)
- [Bitquery OAuth Token Generation](https://docs.bitquery.io/docs/authorisation/how-to-generate/)
- [Bitquery WebSocket Auth](https://docs.bitquery.io/docs/authorisation/websocket/)
- [Bitquery GitHub Rust Streaming Example](https://github.com/bitquery/graphql-streaming-example-rs/)
- [graphql-client crate (GitHub)](https://github.com/graphql-rust/graphql-client)
- [graphql-client on crates.io](https://crates.io/crates/graphql_client)
- [cynic GraphQL client](https://cynic-rs.dev/)
- [cynic on GitHub](https://github.com/obmarg/cynic)
- [graphql-ws-client crate](https://crates.io/crates/graphql-ws-client)
- [graphql-ws-client docs.rs](https://docs.rs/graphql-ws-client/latest/graphql_ws_client/)
- [graphql-ws-client GitHub](https://github.com/obmarg/graphql-ws-client)
- [graphql-ws Protocol (enisdenjo)](https://github.com/enisdenjo/graphql-ws)
- [subscriptions-transport-ws (deprecated)](https://github.com/apollographql/subscriptions-transport-ws)
- [GraphQL over WebSockets — The Guild](https://the-guild.dev/graphql/hive/blog/graphql-over-websockets)
- [Easy GraphQL with Rust (raw reqwest)](https://arthurkhlghatyan.github.io/graphql-in-rust/)
