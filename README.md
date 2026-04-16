# digdigdig3

Multi-exchange connector library — unified async Rust API for crypto exchanges, stock brokers, forex providers, DEX connectors, and prediction markets.

**Version:** 0.1.20
**Edition:** Rust 2021
**License:** MIT OR Apache-2.0
**Repository:** https://github.com/ZENG3LD/digdigdig3

> **Note:** On-chain monitoring (Solana, Bitcoin, TON, Sui, Aptos chain watchers, `OnChainEvent` types, `EventProducer` trait, and all transaction builders) was extracted to the [dig2chain](https://github.com/ZENG3LD/dig2chain) workspace. digdigdig3 retains only the on-chain providers needed by DEX connectors for query and signing (EVM via alloy, Cosmos via cosmrs, StarkNet via starknet-crypto).

---

## Overview

digdigdig3 is organized around three levels describing data depth, and two access tiers within L3 describing registration requirements.

| Level | Description | Auth Required | Count |
|-------|-------------|--------------|-------|
| L1 | Price / OHLCV data feeds — klines, ticker, trades. No orderbook. | No or API key | 7 |
| L2 | L1 + orderbook data (depth snapshots, delta streams). No execution. | No or API key | 3 |
| L3 open | Full stack (L1 + L2 + execution). Market data publicly accessible. | No for data; API key for trading | 22 |
| L3 gated | Full stack. All data requires account/API key. KYC may be required. | Yes | 11 |

**Total connectors: 43**

**Development focus:** L3/open — these are the connectors actively used and tested by the primary consumer (mylittlechart). L3/gated, L1, and L2 connectors are structurally complete but untested against real APIs.

---

## Module Hierarchy

```
digdigdig3/src/
├── l1/                              # Data feeds (OHLCV, ticker — no orderbook)
│   ├── free/   yahoo, krx, finnhub
│   └── paid/   alphavantage, tiingo, twelvedata, jquants
│
├── l2/                              # L1 + orderbook data (no execution)
│   ├── free/   moex
│   └── paid/   polygon, cryptocompare
│
├── l3/                              # Full stack (L1 + L2 + execution)
│   ├── open/                        # Market data requires no registration
│   │   ├── crypto/cex/  18 CEX
│   │   ├── crypto/dex/  dydx, lighter, paradex
│   │   └── prediction/  polymarket
│   └── gated/                       # Account/KYC required for all data
│       ├── stocks/us/    alpaca
│       ├── stocks/india/ zerodha, upstox, angel_one, dhan, fyers
│       ├── stocks/china/ futu
│       ├── stocks/russia/ tinkoff
│       ├── forex/        oanda, dukascopy
│       └── multi/        ib
│
├── core/                            # Traits, types, utils, HTTP, WebSocket
└── connector_manager/               # ConnectorHandle, ConnectorRegistry, ConnectorPool
```

---

## Adding as a Dependency

```toml
[dependencies]
digdigdig3 = { path = "../digdigdig3" }
# or
digdigdig3 = { git = "https://github.com/ZENG3LD/digdigdig3", tag = "v0.1.20" }
```

With specific feature flags:

```toml
[dependencies]
digdigdig3 = { version = "0.1.20", features = ["grpc", "k256-signing"] }
```

---

## L1: Data Feeds (no orderbook)

L1 connectors provide OHLCV klines, ticker, and trades. No orderbook depth. No execution.

All L1 connectors are structurally complete and compile. None have been validated against real APIs.

### L1 free

| Provider | Klines | Ticker | WebSocket | Auth |
|----------|--------|--------|-----------|------|
| Yahoo Finance | Yes | Yes | Yes | No auth |
| KRX (Korea) | Yes | Yes | No | API key |
| Finnhub | Yes | Yes | Yes | API key |

### L1 paid

| Provider | Klines | Ticker | WebSocket | Auth | Region |
|----------|--------|--------|-----------|------|--------|
| Alpha Vantage | Yes | No | No | API key | US / Forex |
| Tiingo | Yes | Yes | Yes | Bearer token | US |
| TwelveData | Yes | Yes | Yes | API key | US |
| J-Quants | Yes | Yes | No | Refresh/ID token | Japan |

All L1 connectors return `UnsupportedOperation` for trading and account methods. They are data providers only.

---

## L2: Orderbook Data (no execution)

L2 connectors add full orderbook depth (snapshots and delta streams) on top of L1 capabilities. No trading or account operations.

All L2 connectors are structurally complete and compile. None have been validated against real APIs.

### L2 free

| Provider | Klines | Orderbook | WebSocket | Auth |
|----------|--------|-----------|-----------|------|
| MOEX (Russia) | Yes | Yes | Yes | No auth (ISS) |

### L2 paid

| Provider | Klines | Orderbook | WebSocket | Auth |
|----------|--------|-----------|-----------|------|
| Polygon | Yes | Yes | Yes | API key |
| CryptoCompare | Yes | Yes | Yes | API key |

---

## L3 Open: Full Stack (no registration needed for market data)

L3 open connectors provide complete market data publicly, and support execution once API keys are configured. Public data (orderbook, trades, klines, ticker) does not require credentials.

**These are the connectors actively used and tested by mylittlechart.**

### L3 open — Crypto CEX (18 connectors)

**Legend:**
- **Tested** — data flowed through the live bridge in production
- **Untested** — code exists and compiles, never validated on real data
- **N/A** — method not implemented (returns `UnsupportedOperation`)

| Exchange | REST klines | REST ticker | REST orderbook | REST trades | WS trades | WS ticker | WS orderbook | WS klines |
|----------|-------------|-------------|----------------|-------------|-----------|-----------|--------------|-----------|
| Binance | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bybit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| OKX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| KuCoin | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Kraken | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Coinbase | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Gate.io | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Gemini | Tested | Tested (poll/15s) | Untested | N/A | Tested | Tested | Untested | Untested |
| MEXC | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| HTX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bitget | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| BingX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Crypto.com | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Upbit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bitfinex | Untested | Untested | Untested | N/A | Untested | Untested | Untested | Untested |
| Bitstamp | Untested | Untested | Untested | N/A | Untested | Untested | Untested | Untested |
| Deribit | Untested | Untested | Untested | N/A | Untested | Untested | Untested | Untested |
| HyperLiquid | Untested | Untested | Untested | N/A | Untested | Untested | Untested | Untested |

Notes:
- Gemini supplements WebSocket ticker with a REST poll fallback (15s interval).
- `get_recent_trades` (REST trades column) is implemented only for Lighter (DEX). All CEX connectors return `UnsupportedOperation` and are marked N/A.
- WS orderbook and WS klines channels exist in code for most connectors but were never subscribed to in a live session.

### Disabled CEX

| Exchange | Issue | Status |
|----------|-------|--------|
| Vertex | Exchange shut down 2025-08-14, acquired by Ink Foundation | Permanently disabled. Code retained as reference. |

### L3 open — Crypto DEX (3 connectors)

| Connector | Chain | Feature Flag | Execution |
|-----------|-------|-------------|-----------|
| dYdX v4 | Cosmos | `onchain-cosmos` + `grpc` | Full trading trait via Cosmos gRPC |
| Lighter.xyz | Custom | — | Full trading trait, native ECgFp5+Poseidon2+Schnorr signing (zero third-party crates) |
| Paradex | StarkNet | `onchain-starknet` | Full trading trait, StarkNet ECDSA signing |

Notes:
- Paradex WebSocket removed from live watchlist — per-symbol attribution is unreliable (exchange uses a global channel).
- Solana interaction (previously Jupiter/Raydium) is handled via dig2chain. These connectors were removed from digdigdig3.

### L3 open — Prediction Markets (1 connector)

| Connector | Type | Execution |
|-----------|------|-----------|
| Polymarket | Binary options on events | Full trading trait — place/cancel/resolve |

---

## L3 Gated: Full Stack (account/KYC required)

All data — including public market data — requires API keys or account credentials. Structurally complete, untested against real APIs.

### L3 gated — Stocks US (1 connector)

| Broker | Klines | Trading | WebSocket | Auth Method |
|--------|--------|---------|-----------|-------------|
| Alpaca | Yes | Yes | Yes | API key header |

### L3 gated — Stocks India (5 connectors)

| Broker | Klines | Trading | WebSocket | Auth Method |
|--------|--------|---------|-----------|-------------|
| Zerodha (Kite) | No* | Yes | No | OAuth2 token |
| Upstox | Yes | Yes | No | OAuth2 token |
| Angel One | Yes | Yes | No | JWT + TOTP |
| Dhan | Yes | Yes | Yes | Access token |
| Fyers | Yes | Yes | No | JWT OAuth2 |

*Zerodha `get_klines` requires a separate Historical API subscription — returns `UnsupportedOperation` without it.

### L3 gated — Stocks China (1 connector)

| Broker | Klines | Trading | Auth Method |
|--------|--------|---------|-------------|
| Futu OpenD | Yes | Partial | OpenD proto (requires local daemon) |

Futu requires the OpenD binary running locally. All methods return `UnsupportedOperation` until OpenD is connected.

### L3 gated — Stocks Russia (1 connector)

| Broker | Klines | Trading | Auth Method |
|--------|--------|---------|-------------|
| Tinkoff Invest | Yes | Yes | Bearer token + gRPC |

### L3 gated — Forex (2 connectors)

| Provider | Klines | Streaming | Trading | Auth |
|----------|--------|-----------|---------|------|
| OANDA | Yes | Yes (server-sent) | Yes | Bearer token |
| Dukascopy | Yes | No | No | No auth (public data) |

Note: Dukascopy is placed in gated/ because the connection model requires an API account context — it is not a fully open feed like L3 open connectors.

### L3 gated — Multi-asset (1 connector)

| Broker | Klines | Trading | WebSocket | Auth |
|--------|--------|---------|-----------|------|
| Interactive Brokers (IB) | Yes | Yes | Yes | OAuth2 / Bearer |

---

## Execution Coverage

### CEX — Optional Trading Traits

| Connector | CancelAll | AmendOrder | BatchOrders | AccountTransfers | CustodialFunds | SubAccounts |
|-----------|-----------|------------|-------------|-----------------|----------------|-------------|
| Binance | Yes | Yes | Yes | Yes | Yes | Yes |
| Bybit | Yes | Yes | Yes | Yes | Yes | Yes |
| OKX | Yes | Yes | Yes | Yes | Yes | Yes |
| KuCoin | Yes | Yes | Yes | Yes | Yes | Yes |
| Gate.io | Yes | Yes | Yes | Yes | Yes | Yes |
| Bitget | Yes | Yes | Yes | Yes | Yes | Yes |
| BingX | Yes | Yes | Yes | Yes | Yes | Yes |
| HTX | Yes | No | Yes | Yes | Yes | Yes |
| Crypto.com | Yes | Yes | Yes | No | Yes | Yes |
| MEXC | Yes | No | Yes | Yes | Yes | Yes |
| Kraken | Yes | Yes | No | No | Yes | Yes |
| Coinbase | Yes | No | No | No | Yes | No |
| Gemini | Yes | No | No | No | Yes | No |
| Upbit | Yes | Yes | No | No | Yes | No |

### DEX Execution

| Connector | Execution Notes |
|-----------|----------------|
| dYdX v4 | Order placement via Cosmos gRPC — full trading trait |
| Lighter | Full trading trait — native ECgFp5+Poseidon2+Schnorr signing |
| Paradex | Full trading trait, StarkNet signing via `onchain-starknet` feature |

### Stock Broker Execution

| Broker | place_order | cancel_order | amend_order | batch_orders |
|--------|-------------|--------------|-------------|--------------|
| Alpaca | Yes | Yes | Yes | No |
| Zerodha | Yes | Yes | Yes | No |
| Upstox | Yes | Yes | Yes | No |
| Angel One | Yes | Yes | Yes | No |
| Dhan | Yes | Yes | Yes | No |
| Fyers | Yes | Yes | No | Yes |
| Tinkoff | Yes | Yes | Yes | No |
| Futu | Partial | No | No | No |

### Optional Trading Traits (defined, not yet implemented in any connector)

These traits exist in `src/core/traits/operations.rs` with default `UnsupportedOperation` implementations. They are ready for connector-level override:

- `MarginTrading` — margin borrow/repay, margin account info
- `EarnStaking` — earn products, subscribe/redeem
- `ConvertSwap` — convert quotes, dust conversion
- `CopyTrading` — follow/unfollow traders, copy positions
- `LiquidityProvider` — LP position management
- `VaultManager` — vault deposit/withdraw
- `StakingDelegation` — delegate/undelegate, claim rewards
- `BlockTradeOtc` — OTC block trade creation and execution
- `MarketMakerProtection` — MMP config, mass quoting
- `TriggerOrders` — conditional order placement
- `PredictionMarket` — prediction market order flow

---

## L2 Orderbook Capabilities

Each exchange declares its L2/orderbook capabilities via `orderbook_capabilities(account_type)`:

```rust
use digdigdig3::core::traits::WebSocketConnector;
use digdigdig3::core::types::AccountType;

let ws = BinanceWebSocket::new(None, None);
let caps = ws.orderbook_capabilities(AccountType::Spot);

println!("WS depths: {:?}", caps.ws_depths);           // [5, 10, 20]
println!("REST max: {:?}", caps.rest_max_depth);        // Some(5000)
println!("Snapshot: {}", caps.supports_snapshot);        // true
println!("Delta: {}", caps.supports_delta);              // true
println!("Checksum: {:?}", caps.checksum);               // None
println!("Channels: {}", caps.ws_channels.len());        // 8

// Pick best channel for your needs
if let Some(ch) = caps.best_channel(Some(20), true) {
    println!("Use channel: {} (delta={}, depth={:?})", ch.name, !ch.is_snapshot, ch.depth);
}
```

### Capabilities per exchange

| Exchange | WS Depths | Checksum | Aggregation | Sequence |
|----------|-----------|----------|-------------|----------|
| Binance | 5/10/20 | No | No | U/u (futures: +pu) |
| Bybit | 1/50/200/1000 | No | No | u field |
| OKX | 1/5/50/400 | CRC32 top-25 | No | seqId/prevSeqId |
| Kraken | 10/25/100/500/1000 | CRC32 top-10 | No | No (spot) |
| KuCoin | 5/50 (snapshot) | No | No | sequence |
| Coinbase | full book | No | No | sequence |
| GateIO | 5/10/20/50/100 | No | No | U/u |
| Bitfinex | 1/25/100/250 | CRC32 top-25 | P0-P4 | optional |
| Bitget | 1/5/15/full | CRC32 top-25 | No | seqId |
| HTX | 5/10/20/150/400 | No | step0-step5 | seqNum |
| MEXC | 5/10/20 | No | No | version |
| BingX | 5/10/20 | No | No | U/u |
| Bitstamp | 100 (snap)/full (delta) | No | No | microtimestamp |
| Gemini | 5/10/20 (Fast API) | No | No | socket_sequence |
| CryptoCom | 10/50/150 | CRC32 | No | u field |
| Upbit | 1/5/15/30 | No | KRW only | No |
| Deribit | 1/10/20 | No | group param | change_id |
| HyperLiquid | 20 (fixed) | No | nSigFigs | No |
| dYdX v4 | full book | No | No | sequence |
| Lighter | full book | No | No | sequence |
| Paradex | full book | No | No | sequence |
| Polymarket | N/A (binary market) | N/A | N/A | N/A |

---

## Architecture

### Core Trait Hierarchy

```
ExchangeIdentity (name, id, account types)
    |--- MarketData (price, orderbook, klines, ticker, ping)
    |--- Trading (place_order, cancel_order, get_order, open_orders, order_history)
    |--- Account (balance, account_info, fees)
    |--- Positions (positions, funding_rate, open_interest, mark_price, closed_pnl)

CoreConnector = ExchangeIdentity + MarketData + Trading + Account + Positions
    (blanket impl — use as generic bound for code working with any exchange)

Optional execution traits (each requires Trading or Account supertrait):
    CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts

Optional advanced traits (default UnsupportedOperation, override per-connector):
    MarginTrading, EarnStaking, ConvertSwap, CopyTrading, LiquidityProvider,
    VaultManager, StakingDelegation, BlockTradeOtc, MarketMakerProtection,
    TriggerOrders, PredictionMarket

WebSocket:
    WebSocketConnector (connect, subscribe, event_stream, active_subscriptions)
    WebSocketExt (convenience blanket: subscribe_ticker, subscribe_klines, etc.)
```

### Precision Guard (f64 to Decimal at Execution Boundary)

All DataFeed paths use `f64` for maximum performance (indicators, UI, research). At the Trading trait boundary, prices and quantities are converted to exchange-safe strings via `rust_decimal`:

```
DataFeed:  get_klines() -> Vec<Kline{f64}>  -> indicators/UI (fast, no overhead)
Execution: place_order(price: f64, qty: f64)
               |
           PrecisionCache.price(&symbol, price)  -> "67543.25"
           PrecisionCache.qty(&symbol, qty)      -> "0.12345"
               |
           Exchange API receives exact string
```

- `safe_price(f64, tick)` — converts via `Decimal::from_str(price.to_string())`, rounds to nearest tick
- `safe_qty(f64, step)` — same conversion, floors to step_size (never exceeds available quantity)
- `PrecisionCache` — per-symbol HashMap loaded from `get_exchange_info()`, stores tick/step per symbol
- Fallback: raw `f64::to_string()` if symbol not in cache

### Connector Manager

The connector manager (`src/connector_manager/`) provides a runtime pool for managing multiple connectors simultaneously:

| Component | Purpose |
|-----------|---------|
| `ConnectorHandle` | Unified dynamic dispatch wrapper for any connector |
| `ConnectorRegistry` | Maps `ExchangeId` to connector instances |
| `ConnectorFactory` | Builds connectors from `ConnectorConfig` |
| `ConnectorConfig` | Per-exchange config structs |
| `ConnectorPool` | DashMap-based concurrent pool |
| `ConnectorAggregator` | Multi-exchange data fan-out |

### Module Structure (per connector)

Every connector follows this layout:

```
src/{level}/{tier}/{category}/{name}/
    mod.rs          -- pub re-exports
    endpoints.rs    -- URL constants, endpoint enum, symbol formatting
    auth.rs         -- signing implementation (HMAC, JWT, EIP-712, etc.)
    parser.rs       -- JSON response parsing, type mapping
    connector.rs    -- trait implementations
    websocket.rs    -- WebSocket connector implementation
```

Reference implementation: `src/l3/open/crypto/cex/kucoin/` — most complete example with all optional traits.

---

## On-Chain Transport for DEX Connectors

Chain providers used internally by DEX connectors for transaction signing and query submission. These are not standalone chain monitors — for full blockchain event streaming, use [dig2chain](https://github.com/ZENG3LD/dig2chain).

| Provider | Chain | Feature Flag | Used By |
|----------|-------|-------------|---------|
| `EvmProvider` | Ethereum/EVM | `onchain-evm` (default) | Uniswap, GMX (legacy) |
| `CosmosProvider` | Cosmos | `onchain-cosmos` | dYdX |
| `StarkNetProvider` | StarkNet | `onchain-starknet` | Paradex |

None of these have been tested end-to-end with real transaction submission.

Note: Solana chain interaction is implemented in dig2chain. No `solana-sdk` dependency in digdigdig3.

---

## Transport Methods

| Transport | Feature Flag | Used By |
|-----------|-------------|---------|
| REST (reqwest) | default | All connectors |
| WebSocket | `websocket` | CEX + DEX + major stock brokers |
| gRPC (tonic) | `grpc` | dYdX (Cosmos gRPC), Tinkoff Invest |
| On-chain EVM RPC | `onchain-evm` | EvmProvider |
| On-chain Cosmos gRPC | `onchain-cosmos` | dYdX |
| On-chain StarkNet | `onchain-starknet` | Paradex |

HTTP client: `src/core/http/client.rs` — async reqwest wrapper with auth injection, retry logic, and rate limiting (`SimpleRateLimiter` token bucket and `WeightRateLimiter`).

WebSocket base: `src/core/websocket/base_websocket.rs` — handles reconnect, ping/pong, and subscription replay on reconnect.

---

## Auth Methods

| Method | Connectors |
|--------|-----------|
| HMAC-SHA256 | Binance, Bybit, OKX, KuCoin, Gate.io, Bitget, BingX, MEXC, HTX, Crypto.com |
| HMAC-SHA384 | Gemini |
| HMAC-SHA512 | Kraken |
| HMAC + passphrase | OKX, KuCoin, Bitget (additional passphrase layer on top of HMAC) |
| JWT ES256 (EC P-256) | Coinbase, Upbit, Tinkoff, Dhan, J-Quants |
| JWT + TOTP | Angel One |
| OAuth2 / Bearer token | Upstox, Fyers, Zerodha, OANDA, IB |
| EIP-712 (Ethereum typed data) | EvmProvider (Uniswap/GMX) |
| Cosmos SDK wallet | dYdX |
| StarkNet ECDSA (STARK key) | Paradex |
| Native ECgFp5+Schnorr | Lighter |
| API key in header | Polygon, Finnhub, Tiingo, TwelveData, Alpha Vantage, CryptoCompare |
| No auth | MOEX ISS, Yahoo Finance, Dukascopy |

---

## Feature Flags

| Feature | Dependencies Enabled | Notes |
|---------|---------------------|-------|
| `default` | `onchain-evm` | EVM provider included by default |
| `onchain-evm` | `alloy` (provider-ws, rpc-types) | EVM chain provider |
| `onchain-ethereum` | `onchain-evm` | Backward-compat alias |
| `onchain-cosmos` | `cosmrs` (bip32) | Cosmos provider for dYdX |
| `onchain-starknet` | `starknet-crypto` | StarkNet provider for Paradex |
| `starknet` | `onchain-starknet` | Legacy alias |
| `websocket` | — | WebSocket enablement flag |
| `grpc` | `tonic` (tls, tls-native-roots), `prost` | gRPC transport |
| `k256-signing` | `k256` (ecdsa-core, ecdsa) | k256 ECDSA signing |

Removed features (extracted to dig2chain): `onchain-solana`, `onchain-bitcoin`, `onchain-sui`, `onchain-ton`, `onchain-aptos`.

---

## Known Issues and Disabled Connectors

| Connector | Issue | Status |
|-----------|-------|--------|
| Vertex | Exchange shut down 2025-08-14, acquired by Ink Foundation | Permanently disabled. Code retained as reference. |
| GMX | No real-time WebSocket API — websocket.rs polls REST internally | Removed from live watchlist. REST data works. |
| Paradex | Per-symbol WebSocket attribution unreliable (exchange uses a global channel) | WebSocket removed from live watchlist. REST works. |
| Futu | Requires OpenD local daemon | All methods return `UnsupportedOperation` until OpenD binary is running. |

---

## Test Coverage

| Connector | Test Count | Type |
|-----------|-----------|------|
| Fyers | 31 tests | Unit tests in src/ |
| MOEX | 24 tests | Unit tests in src/ |
| Tinkoff | 28 tests | Unit tests in src/ |
| KRX | 32 tests | Unit tests in src/ |
| Futu | 6 tests | Unit tests in src/ |
| Alpha Vantage | 10 tests | Unit tests in src/ |
| Vertex | 20 tests | Disabled (shut down) |

All 18 active CEX connectors have zero test files. This is the primary quality gap.

---

*Audit date: 2026-04-16. Version 0.1.20.*
