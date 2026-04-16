# dYdX v4 DEX Connector

Connector for dYdX v4 — a decentralized perpetuals exchange running on its own Cosmos-based appchain. Market data is served by the Indexer API with no authentication. Trading requires on-chain Cosmos transaction signing.

## Status

✅ **MARKET DATA READY** - Indexer API fully implemented, public endpoints working

⚠️ **TRADING IN PROGRESS** - On-chain signing infrastructure implemented (`tx_builder`, `proto`), enabled via `grpc` + `onchain-cosmos` features

## Quick Start

### 1. No Setup Required for Market Data

dYdX v4 Indexer API is fully public. No API keys, no account registration.

```bash
# Test immediately — no credentials needed
cargo test --test dydx_integration -- --nocapture
```

### 2. For Trading (Optional)

Trading requires a dYdX Chain wallet (24-word mnemonic). You can generate one on [dydx.trade](https://dydx.trade) or derive one from any Cosmos-compatible mnemonic.

```bash
export DYDX_MNEMONIC="word1 word2 ... word24"
export DYDX_ADDRESS="dydx1..."
export DYDX_SUBACCOUNT="0"
```

## Usage

### Public Market Data (No Auth)

```rust
use digdigdig3::crypto::dex::dydx::DydxConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // No credentials needed
    let connector = DydxConnector::public(false).await?;

    // Get current perpetual price
    let symbol = Symbol::new("BTC", "USD");
    let price = connector.get_price(symbol.clone(), AccountType::FuturesCross).await?;
    println!("BTC-USD: ${}", price);

    // Get ticker with 24h stats
    let ticker = connector.get_ticker(symbol.clone(), AccountType::FuturesCross).await?;
    println!("24h volume: {}", ticker.volume_24h.unwrap_or(0.0));

    // Get L2 orderbook snapshot
    let book = connector.get_order_book(symbol.clone(), AccountType::FuturesCross).await?;
    println!("Best bid: {}, Best ask: {}", book.bids[0].price, book.asks[0].price);

    // Get historical candles
    let klines = connector.get_klines(symbol, "1h", Some(24), AccountType::FuturesCross).await?;
    println!("Got {} candles", klines.len());

    Ok(())
}
```

### dYdX-Specific Extended Methods

```rust
// List all perpetual markets with metadata
let markets = connector.get_all_markets().await?;

// Single market metadata (tick size, step size, open interest, etc.)
let info = connector.get_market_info("BTC-USD").await?;

// Recent trades
let trades = connector.get_recent_trades("ETH-USD", 50).await?;

// Current block height (useful for order expiry calculation)
let height = connector.get_block_height().await?;
```

## Features

### Market Data
- ✅ Real-time prices via Indexer API
- ✅ Historical OHLCV candles (1MIN, 5MINS, 15MINS, 30MINS, 1HOUR, 4HOURS, 1DAY)
- ✅ L2 orderbook snapshot (up to 100 levels per side)
- ✅ L2 orderbook streaming via WebSocket (`v4_orderbook` channel)
- ✅ Ticker snapshots (24h stats, oracle price, funding rate)
- ✅ All perpetual markets listing
- ✅ Historical funding rates

### Account (Read-Only via Indexer)
- ✅ Subaccount balances and equity
- ✅ Open perpetual positions
- ✅ Order history
- ✅ Fill history
- ✅ Funding payment history
- ✅ Transfer history

### Trading
- ✅ On-chain transaction signing infrastructure (`tx_builder.rs`, `proto.rs`)
- ✅ `MsgPlaceOrder` and `MsgCancelOrder` protobuf encoding
- ✅ Cosmos SDK key derivation (BIP-39/BIP-44, path `m/44'/118'/0'/0/0`)
- ❌ Live order placement not yet tested end-to-end (requires `grpc` feature)
- ❌ Deposits and withdrawals (CCTP/IBC — out of scope)

### NOT Supported
- ❌ Spot trading (dYdX v4 is perpetuals-only)
- ❌ L3 orderbook (requires running a full node)
- ❌ gRPC streaming from full node (Indexer WebSocket used instead)
- ❌ Cross-chain operations (Noble IBC bridge, CCTP)

## L2 Orderbook

### REST Snapshot

```
GET https://indexer.dydx.trade/v4/orderbooks/perpetualMarket/BTC-USD
```

- Up to **100 levels per side** (server-controlled, no client depth parameter)
- Prices sorted: bids descending, asks ascending
- Server applies `uncrossBook: true` — crossed levels removed before response
- String-encoded decimals: `{ "price": "74693", "size": "0.0075" }`

### WebSocket Streaming

```rust
let ws = DydxWebSocket::new(false); // false = mainnet
ws.subscribe_orderbook("BTC-USD").await?;
```

**Protocol:** `v4_orderbook` channel, snapshot-then-delta

| Message type | Meaning |
|---|---|
| `subscribed` | Initial full snapshot (object format `{ price, size }`) |
| `channel_data` | Incremental delta (tuple format `[price, size, offset]`) |

**Delta semantics:**
- `size == "0"` — delete this price level
- `size != "0"` — upsert (add or replace)
- `offset` (third tuple element) — per-level logical timestamp for uncrossing crossed books

**Gap detection:** Track `message_id` (connection-level monotonic counter). On gap, re-subscribe to get fresh snapshot.

**Batching:** Pass `"batched": true` in subscribe message to reduce frame count at slight latency cost.

**Limits:** Up to 32 simultaneous `v4_orderbook` subscriptions per connection.

**No checksum.** Integrity relies entirely on the snapshot + delta sequence.

### Why the Book Can Cross

dYdX v4 has no global centralized book — the canonical state belongs to the current block proposer. Proposers rotate every ~1 second, causing brief crossed-price states between blocks. Use the `offset` field to uncross: compare offsets of the best bid and best ask; discard the level with the lower (older) offset.

## Authentication

dYdX v4 uses **no API keys**. Authentication is wallet-based:

| Operation | Auth Required | Method |
|---|---|---|
| Market data (Indexer REST) | No | Public endpoint |
| Account queries (Indexer REST) | No | Public endpoint, address in URL |
| WebSocket market feeds | No | Public channel |
| Place/cancel orders | Yes | On-chain Cosmos transaction signing |

### On-Chain Signing

Trading operations broadcast signed transactions to a validator node via gRPC. The signing flow:

1. Derive private key from 24-word mnemonic (BIP-39 → BIP-44, coin type 118)
2. Build protobuf message (`MsgPlaceOrder` or `MsgCancelOrder`)
3. Wrap in `TxRaw` with sequence number, account number, chain ID
4. Sign with ECDSA secp256k1
5. Broadcast to validator gRPC node

The connector implements this in `tx_builder.rs` using the `cosmrs` crate, guarded by the `onchain-cosmos` and `grpc` Cargo features.

**Gas:** Trading itself is gas-free (no fee per order). The main account needs a small DYDX balance for any on-chain transactions (transfers, deposits).

## Endpoints

### Indexer (Public)

| Network | REST | WebSocket |
|---|---|---|
| Mainnet | `https://indexer.dydx.trade/v4` | `wss://indexer.dydx.trade/v4/ws` |
| Testnet | `https://indexer.v4testnet.dydx.exchange/v4` | `wss://indexer.v4testnet.dydx.exchange/v4/ws` |

### Validator gRPC (Trading)

Multiple public providers: `grpc://oegs.dydx.trade:443`, `https://dydx-dao-grpc-1.polkachu.com:443`, and others. See `research/endpoints.md` for the full list.

## File Structure

```
dydx/
├── README.md           # This file
├── mod.rs              # Module exports, feature flags
├── auth.rs             # Auth placeholder (wallet key management)
├── connector.rs        # DydxConnector + trait implementations
├── endpoints.rs        # URL constants, endpoint enum, symbol formatting
├── parser.rs           # JSON parsing (Indexer REST responses)
├── proto.rs            # Handwritten prost protobuf types (feature = "grpc")
├── tx_builder.rs       # Cosmos TxRaw builder + signing (feature = "onchain-cosmos")
├── websocket.rs        # WebSocket client (v4_orderbook and other channels)
└── research/
    ├── authentication.md   # Auth model, Cosmos signing, key derivation
    ├── endpoints.md        # All Indexer and Node endpoints
    ├── l2_orderbook.md     # L2 orderbook capabilities, snapshot/delta protocol
    └── websocket.md        # WebSocket channels, message formats
```

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DYDX_MNEMONIC` | Trading only | 24-word BIP-39 mnemonic |
| `DYDX_ADDRESS` | Trading only | `dydx1...` bech32 address |
| `DYDX_SUBACCOUNT` | Trading only | Subaccount number (default `0`) |
| `DYDX_TESTNET` | No | Set to `1` to use testnet endpoints |

No variables needed for market data.

## Rate Limits

- **Indexer REST:** 100 requests / 10 seconds per IP
- **WebSocket:** 32 simultaneous subscriptions per connection per channel type
- **On-chain trading:** 200 short-term orders / block, 2 stateful orders / block

## Testing

```bash
# All integration tests (no credentials needed)
cargo test --test dydx_integration -- --nocapture

# Specific tests
cargo test --test dydx_integration test_get_price -- --nocapture
cargo test --test dydx_integration test_get_order_book -- --nocapture
cargo test --test dydx_integration test_get_markets -- --nocapture

# WebSocket orderbook stream
cargo test --test dydx_integration test_orderbook_stream -- --nocapture

# Live tests (marked #[ignore] by default)
cargo test --test dydx_live -- --nocapture --ignored
```

## Key Concepts

**Perpetuals only.** Every market is a perpetual future, settled in USDC. There are no spot markets.

**Subaccounts.** Each wallet address has up to 128,001 subaccounts. Subaccounts 0–127 are cross-margin (parent); subaccounts 128–128,000 are isolated-margin (child). The orderbook itself is global — subaccounts only affect margin accounting.

**Market types (v5.0.0+).** Markets are either `PERPETUAL_MARKET_TYPE_CROSS` (all pre-v5 markets) or `PERPETUAL_MARKET_TYPE_ISOLATED` (added January 2025). The orderbook API and WebSocket format are identical for both.

**Short-term vs stateful orders.** Short-term orders (`goodTilBlock` within current block + 20) are gossip-based and expire automatically. Stateful orders (`goodTilBlockTime`, max 95 days) go through consensus and can be canceled on-chain.

## Documentation

- **Indexer REST API:** https://docs.dydx.xyz/indexer-client/http
- **Indexer WebSocket:** https://docs.dydx.xyz/indexer-client/websockets
- **Full Node streaming:** https://docs.dydx.exchange/api_integration-full-node-streaming
- **Uncrossing orderbook:** https://docs.dydx.exchange/api_integration-guides/how_to_uncross_orderbook
- **GitHub (v4-chain):** https://github.com/dydxprotocol/v4-chain

## License

Part of the NEMO trading system.
