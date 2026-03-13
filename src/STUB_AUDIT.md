# Stub Audit Report — digdigdig3/src

Generated: 2026-03-13
Scope: All `.rs` files under `digdigdig3/src/`
Files audited: 820
Total findings: 128 grep hits, categorized below

---

## Legend

- **LAZY STUB** — Incomplete implementation that silently returns wrong data; action required
- **LEGITIMATE** — Correct API behavior (empty input guard, exchange doesn't have the endpoint, etc.)
- **MINOR TODO** — Missing enhancement but core behavior is correct; low risk
- **DESIGN STUB** — Known architectural limitation (wrong protocol, requires different SDK, etc.)

---

## Category 1: `get_all_tickers()` returning `Ok(vec![])` — LAZY STUBS

These methods make a real HTTP call, discard the response, and return an empty vec. The exchange has the endpoint; the parser just hasn't been written.

### FINDING 1-A — Bybit

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/bybit/connector.rs:248-255`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
    // ...
    let response = self.get(BybitEndpoint::Ticker, params).await?;
    // TODO: parse all tickers
    let _ = response;
    Ok(vec![])
}
```

Live data is fetched, then silently discarded. Callers receive an empty vec instead of actual ticker data. Bybit's `/v5/market/tickers` endpoint exists and returns full ticker arrays.

---

### FINDING 1-B — KuCoin

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/kucoin/connector.rs:312-320`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
let response = self.get(endpoint, HashMap::new(), account_type).await?;
// TODO: parse all tickers
let _ = response;
Ok(vec![])
```

Same pattern — response fetched, discarded. KuCoin has both `/api/v1/market/allTickers` (spot) and `/api/v1/contracts/active` (futures).

---

### FINDING 1-C — OKX

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/okx/connector.rs:195-203`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
let response = self.get(OkxEndpoint::AllTickers, params).await?;
// TODO: implement parse_all_tickers in parser
let _ = response;
Ok(vec![])
```

OKX `/api/v5/market/tickers` returns full ticker array. Parser not written.

---

## Category 2: WebSocket implementations that are complete stubs

### FINDING 2-A — Alpaca WebSocket

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/us/alpaca/websocket.rs:62-176`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
    // TODO: Implement actual WebSocket connection
    // This is a placeholder implementation
    // ...
    // For now, just mark as connected to allow compilation
    *self.status.write().await = ConnectionStatus::Connected;
    Ok(())  // Returns "connected" but connects to nothing
}

fn event_stream(&self) -> Pin<...> {
    // For now, return empty stream
    Box::pin(futures_util::stream::empty())
}
```

`connect()` falsely reports success without establishing a WebSocket connection. `subscribe()` silently stores the request but never sends it. `event_stream()` returns an empty stream. Any caller that relies on live data will silently receive nothing.

---

### FINDING 2-B — Tiingo WebSocket

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/us/tiingo/websocket.rs:119-173`
**Severity:** MEDIUM
**Type:** DESIGN STUB (returns Error, not false-success)

```rust
async fn connect(&mut self, _account_type: AccountType) -> WebSocketResult<()> {
    Err(WebSocketError::UnsupportedOperation(
        "Tiingo WebSocket support is a stub. Full implementation pending.".to_string()
    ))
}
```

At least it returns an error rather than false success. Tiingo does have a WebSocket API but it is not implemented. This is acceptable as a placeholder.

---

### FINDING 2-C — IB (Interactive Brokers) WebSocket

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/aggregators/ib/websocket.rs:11-48`
**Severity:** MEDIUM
**Type:** DESIGN STUB (documented, returns Error)

All methods return `UnsupportedOperation`. Module comment says "placeholder implementation." IB Client Portal WebSocket is substantially different. Documented correctly.

---

### FINDING 2-D — Whale Alert WebSocket

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/onchain/analytics/whale_alert/websocket.rs:128-170`
**Severity:** MEDIUM
**Type:** LAZY STUB

```rust
// TODO: Send subscription message via WebSocket
// For now, this is a placeholder implementation
Err(WebSocketError::UnsupportedOperation(...))
```

`connect()` also returns UnsupportedOperation even though the URL is built correctly. The subscription methods have TODO comments. Whale Alert does have a WebSocket API.

---

### FINDING 2-E — Bitquery WebSocket

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/onchain/analytics/bitquery/websocket.rs:32-62`
**Severity:** LOW
**Type:** DESIGN STUB (documented stub, no traits claimed)

Struct exists with URL helper only. No `WebSocketConnector` trait implemented. Comment clearly states it's a stub. Acceptable placeholder.

---

### FINDING 2-F — MEXC WebSocket unsubscribe

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/mexc/websocket.rs:454-457`
**Severity:** LOW
**Type:** MINOR TODO

```rust
async fn unsubscribe(&mut self, _request: SubscriptionRequest) -> WebSocketResult<()> {
    // TODO: Implement unsubscribe
    Ok(())
}
```

Silently succeeds without sending unsubscribe message to exchange. The subscription remains active server-side. MEXC unsubscribe is just `{"method": "UNSUBSCRIPTION", "params": [...]}`.

---

## Category 3: Parser TODOs that produce incorrect data

### FINDING 3-A — Hyperliquid: Order type always hardcoded to Limit{price:0.0}

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/hyperliquid/parser.rs:422-423`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
// TODO: Parse order type from "t" field
let order_type = OrderType::Limit { price: 0.0 };
```

Every order returned from `parse_order_data()` reports `OrderType::Limit { price: 0.0 }` regardless of actual type. Market, Stop, and TP/SL orders are all misclassified with wrong price. The `"t"` field in Hyperliquid's API carries the actual type.

---

### FINDING 3-B — Hyperliquid: MarginType always Cross

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/hyperliquid/parser.rs:396`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
margin_type: crate::core::MarginType::Cross, // TODO: Detect from leverage.type
```

Isolated margin positions are misclassified. The `leverage` object has a `type` field that distinguishes `cross` from `isolated`.

---

### FINDING 3-C — Hyperliquid: Symbol name not matched in price parsing

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/hyperliquid/parser.rs:122-124`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
// TODO: Need to match against symbol name from universe
// For now, use index-based matching
return Ok(mid);
```

Returns price from the first asset in the universe array regardless of which symbol was requested.

---

### FINDING 3-D — OKX WebSocket: Order/Balance/Position update parsers unimplemented

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/okx/parser.rs:540-558`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
pub fn parse_ws_order_update(data: &Value) -> ExchangeResult<OrderUpdateEvent> {
    // TODO: Implement proper OrderUpdateEvent parsing
    let _ = data;
    Err(ExchangeError::Parse("WebSocket order updates not yet implemented".to_string()))
}

pub fn parse_ws_balance_update(data: &Value) -> ExchangeResult<BalanceUpdateEvent> {
    // TODO: Implement proper BalanceUpdateEvent parsing
    let _ = data;
    Err(ExchangeError::Parse("WebSocket balance updates not yet implemented".to_string()))
}

pub fn parse_ws_position_update(data: &Value) -> ExchangeResult<PositionUpdateEvent> {
    // TODO: Implement proper PositionUpdateEvent parsing
    let _ = data;
    Err(ExchangeError::Parse("WebSocket position updates not yet implemented".to_string()))
}
```

All three private channel parsers return errors. OKX WebSocket private channels (orders, account, positions) are completely non-functional.

---

### FINDING 3-E — Binance WebSocket: ACCOUNT_UPDATE (futures) discarded

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/binance/websocket.rs:534-537`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
"ACCOUNT_UPDATE" => {
    // TODO: Parse balance and position updates
    Ok(None)
}
```

Binance Futures `ACCOUNT_UPDATE` events (balance changes from trades/funding) are silently discarded.

---

### FINDING 3-F — Upbit: `created_at` always 0

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/upbit/parser.rs:328`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
created_at: 0, // TODO: Parse created_at ISO 8601
```

Order timestamps are always zero. Upbit returns `created_at` in ISO 8601 format.

---

### FINDING 3-G — Upbit: `average_price` always None (TODO branch)

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/upbit/parser.rs:306-312`
**Severity:** LOW
**Type:** MINOR TODO

```rust
let average_price = if filled_quantity > 0.0 {
    // TODO: Need trades_count and total value to calculate
    None
} else {
    None
};
```

Average fill price never populated even when data is available.

---

### FINDING 3-H — KRX Parser: Response format unknown/guessed

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/korea/krx/parser.rs:80-100`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
// TODO: Update this when actual API response format is known
// New format (Open API) - TODO: Update when actual format is known
// Assuming similar structure but possibly different wrapper or field names
let array = if let Some(data) = response.get("data")... {
    data
} else if let Some(items) = response.get("items")... {
    items
} else if let Some(block) = response.get("OutBlock_1")... {
    block
```

The parser guesses among multiple possible formats. The actual KRX Open API response format is not validated. Kline parsing may silently return wrong data or an empty vec.

---

### FINDING 3-I — MOEX: `parse_timestamp()` returns current time instead of actual timestamp

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/russia/moex/parser.rs:94-99`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
fn parse_timestamp(_datetime_str: &str) -> Option<i64> {
    // For simplicity, parse as simple...
    // TODO: Implement proper datetime parsing
    Some(chrono::Utc::now().timestamp_millis())  // returns NOW for every candle
}
```

Every bar parsed from MOEX gets the current wall-clock time as its timestamp. Historical kline data is completely unusable — all candles have the same (wrong) timestamp.

---

### FINDING 3-J — MOEX: `parse_orderbook()` is a placeholder

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/russia/moex/parser.rs:262-269`
**Severity:** MEDIUM
**Type:** LAZY STUB

```rust
// MOEX orderbook structure may vary
// This is a placeholder implementation
```

The comment says "placeholder" but the code does attempt to parse BUYSELL/PRICE/QUANTITY columns. Likely functional if MOEX returns that structure.

---

### FINDING 3-K — Dhan WebSocket: Quote packet parsing incomplete

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/india/dhan/websocket.rs:129-135`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
// TODO: Implement full quote packet parsing
// See research/websocket_full.md for complete structure
let mut result = HashMap::new();
result.insert("packet_size".to_string(), data.len() as f64);
Ok(result)
```

The binary quote packet parser only returns the packet size, not actual OHLCV or tick data. Any caller using Dhan WebSocket receives useless data.

---

## Category 4: Lighter.xyz — Transaction signing incomplete

### FINDING 4-A — Lighter: Auth token not actually signed

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/dex/lighter/auth.rs:98-113`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
/// This is a placeholder implementation. The actual token should be signed
/// using the API key private key in a production implementation.
pub fn generate_auth_token(&self, expiry_seconds: u64) -> ExchangeResult<String> {
    Ok(format!("{}:{}:{}:{}", expiry_time, account_index, api_key_index, random_hex))
}
```

Token format is constructed without ECDSA signing. The actual Lighter API requires the token to be cryptographically signed with the private key. Unsigned tokens will be rejected by the exchange.

---

### FINDING 4-B — Lighter: Transaction signing returns error

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/dex/lighter/auth.rs:138-149`
**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/dex/lighter/connector.rs:186`
**Severity:** HIGH
**Type:** DESIGN STUB

```rust
/// Sign transaction (placeholder for Phase 3)
pub fn sign_transaction(...) -> ExchangeResult<String> {
    Err(ExchangeError::Auth("Transaction signing not yet implemented (Phase 3)".to_string()))
}
// In connector:
// TODO Phase 3: Implement transaction signing
let headers = HashMap::new();
```

Trading operations on Lighter will fail — POST requests skip authentication entirely.

---

## Category 5: OANDA Streaming — Entire feature unimplemented

### FINDING 5-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/forex/oanda/streaming.rs:74-88`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
pub async fn connect(&mut self) -> ExchangeResult<()> {
    // TODO: Implement HTTP streaming connection
    Err(ExchangeError::UnsupportedOperation("HTTP streaming not yet implemented..."))
}
pub async fn next_message(&mut self) -> ExchangeResult<StreamMessage> {
    // TODO: Read next line from HTTP stream and parse JSON
    Err(ExchangeError::UnsupportedOperation("..."))
}
```

Both `PricingStream` and `TransactionStream` structs exist but all methods return errors. OANDA's streaming endpoints (`/v3/accounts/{id}/pricing/stream`, `/v3/accounts/{id}/transactions/stream`) are primary use-case features.

---

## Category 6: Futu — Acknowledged full stub

### FINDING 6-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/china/futu/connector.rs`
**Severity:** LOW
**Type:** DESIGN STUB (documented and deliberate)

The entire Futu connector is an intentional stub. Futu uses TCP + Protocol Buffers instead of HTTP REST. The connector returns `UnsupportedOperation` for all methods with detailed instructions on how to properly integrate. This is correctly documented in `mod.rs`, `connector.rs`, `parser.rs`, and `README`.

---

## Category 7: Uniswap WebSocket — Hardcoded pool address

### FINDING 7-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/swap/uniswap/websocket.rs:446-448`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
// For now, we'll use a placeholder pool address
let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"; // WETH/USDC 0.05%
```

Regardless of which symbol the caller subscribes to, the connector subscribes to the WETH/USDC pool. Every subscription silently maps to the same single pool.

---

## Category 8: Angel One — Hardcoded placeholder IP/MAC headers

### FINDING 8-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/india/angel_one/auth.rs:124-128`
**Severity:** HIGH
**Type:** LAZY STUB

```rust
// Required IP/MAC headers - using placeholder values
// In production, these should be actual client IP and MAC address
headers.insert("X-ClientLocalIP".to_string(), "192.168.1.1".to_string());
headers.insert("X-ClientPublicIP".to_string(), "0.0.0.0".to_string());
headers.insert("X-MACAddress".to_string(), "00:00:00:00:00:00".to_string());
```

Angel One API requires valid IP and MAC headers for compliance. Using `0.0.0.0` and `00:00:00:00:00:00` will likely be rejected or flagged by the exchange.

---

## Category 9: Connector Manager — Account and Trading traits not delegated

### FINDING 9-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/connector_manager/aggregator.rs:490-509`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
// TODO: Uncomment when Account trait is delegated in connector.rs
// TODO: Uncomment when Trading trait is delegated in connector.rs
```

The aggregator cannot call `get_balance()` or any trading methods across exchanges. The feature is blocked by a missing delegation in `connector.rs`.

---

## Category 10: Deribit WebSocket — BalanceUpdate hardcoded to BTC

### FINDING 10-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/deribit/websocket.rs:238`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
StreamType::BalanceUpdate => "user.portfolio.BTC".to_string(), // TODO: support multiple currencies
```

Balance updates only subscribe to BTC portfolio. ETH and other currency portfolios are never subscribed.

---

## Category 11: JQuants — Date range not implemented

### FINDING 11-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/japan/jquants/connector.rs:199`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
// TODO: Add date range support based on limit
// For now, fetch all available data
```

`get_klines()` ignores `start_time` / `end_time` and fetches all available data. On heavily traded symbols this may return thousands of rows needlessly. JQuants API supports `from` / `to` parameters.

---

## Category 12: KRX — Market always defaulted to KOSPI

### FINDING 12-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/korea/krx/connector.rs:300-301`
**Severity:** MEDIUM
**Type:** MINOR TODO

```rust
// TODO: Add market detection logic based on symbol
let market = MarketId::Kospi;
```

KOSDAQ and KONEX symbols are incorrectly queried against the KOSPI endpoint.

---

## Category 13: GMX — Token decimals hardcoded

### FINDING 13-A

**File:** `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/dex/gmx/parser.rs:28-45`
**Severity:** LOW
**Type:** MINOR TODO

```rust
/// TODO: Fetch from /tokens endpoint and cache
fn get_token_decimals(symbol: &str) -> u32 {
    match symbol.to_uppercase().as_str() {
        "BTC" | "WBTC" | "WBTC.b" => 8,
        // ...
        _ => 18, // Default to 18
    }
}
```

New GMX tokens not in the hardcoded list default to 18 decimals, which could produce incorrect price calculations.

---

## Category 14: Benign / Legitimate `Ok(vec![])` returns

The following `Ok(vec![])` returns are legitimate behavior, NOT stubs:

| File | Line | Reason |
|------|------|--------|
| `binance/connector.rs:1584` | BatchOrders | Empty input guard (`orders.is_empty()`) |
| `bingx/connector.rs:1063,1197` | BatchOrders | Empty input guard |
| `bitfinex/connector.rs:924,1050` | BatchOrders | Empty input guard |
| `bitfinex/connector.rs:1148` | `get_transfer_history` | Documented: Bitfinex has no transfer history endpoint |
| `bitget/connector.rs:1659,1816` | BatchOrders | Empty input guard |
| `bitstamp/connector.rs:308` | `cancel_all_orders` | Exchange returns success/failure only, not ID list |
| `bitstamp/connector.rs:767` | `get_funds_history` (deposits) | Bitstamp has no deposit history endpoint |
| `bybit/connector.rs:1679,1743` | BatchOrders | Empty input guard |
| `crypto_com/connector.rs:1182,1304` | BatchOrders | Empty input guard |
| `gateio/connector.rs:1571` | BatchOrders | Empty input guard |
| `htx/connector.rs:1049` | `get_positions` | Spot has no positions |
| `hyperliquid/connector.rs:1614` | `get_transfer_history` | Documented: endpoint doesn't exist |
| `hyperliquid/parser.rs:568` | `parse_order_history` | Fallback for unrecognized response format |
| `kraken/connector.rs:1094` | BatchOrders | Empty input guard (spot has no batch) |
| `okx/connector.rs:1237,1293` | BatchOrders | Empty input guard |
| `phemex/connector.rs:1298` | `get_transfer_history` | Documented: not a standard endpoint |
| `dukascopy/connector.rs:76,109` | Tick file downloads | Correct: weekends/holidays have no data |
| `finnhub/parser.rs:203,228` | WS message parse | Correct: ping and unknown messages return empty |
| `census/parser.rs:112,197` | Dataset parse | Correct: empty response guard |
| `bitstamp/connector.rs:295` | `get_all_tickers` | Bitstamp has no bulk ticker endpoint (documented) |

---

## Category 15: No-auth stubs (legitimate by design)

These files use the word "stub" but represent services that require no authentication — their "stub auth" is correct behavior:

- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/conflict/ucdp/auth.rs` — UCDP requires no auth
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/cyber/ripe_ncc/auth.rs` — RIPE NCC requires no auth
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/economic/ecb/auth.rs` — ECB requires no auth
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/governance/eu_parliament/auth.rs` — EU Parliament requires no auth
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/maritime/imf_portwatch/auth.rs` — IMF PortWatch requires no auth
- `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/intelligence_feeds/space/launch_library/auth.rs` — Launch Library 2 has no mandatory auth

---

## Summary Table — Action Required

| # | File | Severity | Issue | Action |
|---|------|----------|-------|--------|
| 1-A | `crypto/cex/bybit/connector.rs:248` | HIGH | `get_all_tickers()` discards response | Implement `BybitParser::parse_tickers()` |
| 1-B | `crypto/cex/kucoin/connector.rs:312` | HIGH | `get_all_tickers()` discards response | Implement `KuCoinParser::parse_all_tickers()` |
| 1-C | `crypto/cex/okx/connector.rs:195` | HIGH | `get_all_tickers()` discards response | Implement `OkxParser::parse_all_tickers()` |
| 2-A | `stocks/us/alpaca/websocket.rs:62` | HIGH | WS `connect()` fakes success, no real connection | Implement tokio-tungstenite connect + auth |
| 2-D | `onchain/analytics/whale_alert/websocket.rs` | MEDIUM | WS entirely unimplemented | Implement WS connection with API key |
| 2-F | `crypto/cex/mexc/websocket.rs:454` | LOW | `unsubscribe()` is no-op | Send UNSUBSCRIPTION message |
| 3-A | `crypto/cex/hyperliquid/parser.rs:422` | HIGH | Every order type hardcoded to `Limit{price:0.0}` | Parse `"t"` field from response |
| 3-B | `crypto/cex/hyperliquid/parser.rs:396` | MEDIUM | `MarginType` always Cross | Parse `leverage.type` field |
| 3-D | `crypto/cex/okx/parser.rs:540-558` | HIGH | WS order/balance/position parsers return errors | Implement all three parsers |
| 3-E | `crypto/cex/binance/websocket.rs:534` | MEDIUM | `ACCOUNT_UPDATE` events discarded | Parse balance and position updates |
| 3-F | `crypto/cex/upbit/parser.rs:328` | MEDIUM | `created_at` always 0 | Parse ISO 8601 timestamp |
| 3-H | `stocks/korea/krx/parser.rs:80` | HIGH | Response format unknown/guessed | Verify actual KRX Open API format |
| 3-I | `stocks/russia/moex/parser.rs:94` | HIGH | All candle timestamps = current time | Implement datetime parsing with chrono |
| 3-K | `stocks/india/dhan/websocket.rs:129` | HIGH | Quote packet returns only size, not data | Implement binary packet parser per research doc |
| 4-A | `crypto/dex/lighter/auth.rs:98` | HIGH | Auth token not ECDSA-signed | Implement ECDSA signing (secp256k1) |
| 4-B | `crypto/dex/lighter/connector.rs:186` | HIGH | POST requests have no auth headers | Complete Phase 3 signing |
| 5-A | `forex/oanda/streaming.rs:74` | HIGH | Both streaming classes entirely unimplemented | Implement reqwest streaming with `bytes_stream()` |
| 7-A | `crypto/swap/uniswap/websocket.rs:446` | HIGH | Hardcoded WETH/USDC pool for all subscriptions | Map symbol → pool address lookup |
| 8-A | `stocks/india/angel_one/auth.rs:124` | HIGH | Hardcoded `0.0.0.0` / `00:00:..` IP/MAC | Use real client IP via `local_ip_address` crate or config |
| 9-A | `connector_manager/aggregator.rs:490` | MEDIUM | Account/Trading not delegated in aggregator | Implement trait delegation in `connector.rs` |
| 10-A | `crypto/cex/deribit/websocket.rs:238` | MEDIUM | Balance WS hardcoded to BTC only | Support configurable currency list |
| 11-A | `stocks/japan/jquants/connector.rs:199` | MEDIUM | Date range ignored, fetches all data | Pass `from`/`to` params to JQuants API |
| 12-A | `stocks/korea/krx/connector.rs:300` | MEDIUM | KOSDAQ/KONEX queried as KOSPI | Detect market from symbol prefix |
| 13-A | `crypto/dex/gmx/parser.rs:28` | LOW | Token decimals hardcoded, missing new tokens | Fetch from `/tokens` endpoint |

**Total HIGH severity actionable stubs: 12**
**Total MEDIUM severity actionable stubs: 9**
**Total LOW severity actionable stubs: 3**
