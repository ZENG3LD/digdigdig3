# Lighter DEX Connector

Connector for Lighter — a ZK-rollup CLOB DEX on Arbitrum with on-chain orderbook for perpetuals and spot.

## Status

Phase 1 (current): Public market data — authentication working, all public endpoints implemented.
Phase 2 (planned): Account data with auth tokens.
Phase 3 (planned): Trading with EIP-712 / ECDSA transaction signing.

## Quick Start

### 1. Get API Keys (for trading only)

Market data is fully public — no keys needed.

For trading access:
```
https://app.lighter.xyz
→ Connect wallet (Arbitrum)
→ Register account (creates L2 account index)
→ Settings → API Keys → Create key (index 3-254)
→ Download or copy private key (shown once)
```

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export LIGHTER_API_PRIVATE_KEY="0x..."
export LIGHTER_ACCOUNT_INDEX="1"
export LIGHTER_API_KEY_INDEX="3"
```

**Windows PowerShell:**
```powershell
$env:LIGHTER_API_PRIVATE_KEY="0x..."
$env:LIGHTER_ACCOUNT_INDEX="1"
$env:LIGHTER_API_KEY_INDEX="3"
```

### 3. Test

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test lighter_integration -- --nocapture
cargo test --test lighter_live -- --nocapture --ignored
```

## Usage

### Public Market Data (no keys required)

```rust
use connectors_v5::exchanges::lighter::LighterConnector;
use connectors_v5::core::{Symbol, AccountType};
use connectors_v5::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = LighterConnector::public(false).await?;

    // ETH perp price
    let symbol = Symbol::new("ETH", "USDC");
    let price = connector.get_price(&symbol, AccountType::FuturesCross).await?;
    println!("ETH: {}", price);

    // OHLCV candles
    let klines = connector.get_klines(&symbol, "1h", Some(100), AccountType::FuturesCross, None).await?;
    println!("Got {} candles", klines.len());

    // Recent trades
    let trades = connector.get_recent_trades("ETH", AccountType::FuturesCross, Some(50)).await?;
    println!("Got {} trades", trades.len());

    Ok(())
}
```

### With Credentials (account + trading)

```rust
use connectors_v5::exchanges::lighter::LighterConnector;
use connectors_v5::core::{Credentials, AccountType, Symbol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creds = Credentials::from_env()?; // reads LIGHTER_* env vars
    let connector = LighterConnector::new(Some(creds), false).await?;

    // Account balance (Phase 2)
    // let balances = connector.get_balance(None, AccountType::FuturesCross).await?;

    // Place limit order (Phase 3)
    // let order = connector.place_order(&OrderRequest { ... }).await?;

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices (`get_price`)
- ✅ OHLCV candles, resolutions: 1m / 5m / 15m / 1h / 4h / 1d
- ✅ L2 orderbook via WebSocket (50 ms batched snapshots + deltas)
- ✅ L2 orderbook REST snapshot (up to 250 resting orders)
- ✅ Ticker (bid/ask/last/24h stats)
- ✅ Recent trades
- ✅ Funding rates
- ✅ Market metadata and precision info
- ✅ Both perpetual (market_id 0-2047) and spot (market_id 2048+) markets
- ✅ **All market data without API keys**

### Trading
- ⏳ Limit and market orders (Phase 3)
- ⏳ Order cancellation and modification (Phase 3)
- ⏳ Batch transactions, up to 50 per request (Phase 3)
- ⏳ Nonce management per API key (Phase 3)

### Account
- ⏳ Balance queries (Phase 2)
- ⏳ Open positions (Phase 2)
- ⏳ Order history (Phase 2)
- ⏳ PnL, transaction history (Phase 2)

### NOT Supported
- ❌ Deposit / withdrawal (on-chain only, out of scope)
- ❌ Cross-margin configuration (set via frontend)
- ❌ Leverage slider (not a REST operation on Lighter)

## L2 Orderbook

### WebSocket (primary)

Channel: `order_book/{market_id}`, e.g. `order_book/0` for ETH perp.

- First message on subscribe: full snapshot of all price levels.
- Subsequent messages: incremental deltas — only changed levels.
- Update rate: 50 ms fixed batches (max 20 updates/second per market).
- No configurable depth, no server-side aggregation, no checksum field.
- Sequence is tracked via `nonce` / `begin_nonce` from the matching engine.

**Gap detection:** verify `update.order_book.begin_nonce == prev_nonce` on every message. On mismatch, re-subscribe to get a fresh snapshot. Do not use `offset` for this — it is API-server-scoped and resets on reconnect.

**Delta semantics:** a level with `size == "0"` means remove it from the local book; `size > 0` means upsert.

**Keepalive:** send at least one frame (ping or message) every 2 minutes or the server closes the connection.

```json
// Subscribe
{"type": "subscribe", "channel": "order_book/0"}

// Snapshot (on first subscribe)
{
  "channel": "order_book:0",
  "type": "update/order_book",
  "order_book": {
    "asks": [{"price": "3025.00", "size": "2.0000"}],
    "bids": [{"price": "3024.00", "size": "1.0000"}],
    "nonce": 88200,
    "begin_nonce": 0
  }
}

// Delta (~50ms later)
{
  "channel": "order_book:0",
  "type": "update/order_book",
  "order_book": {
    "asks": [{"price": "3025.00", "size": "0.5000"}],
    "bids": [{"price": "3024.00", "size": "0.0000"}],
    "nonce": 88205,
    "begin_nonce": 88200
  }
}
```

### REST Snapshot

```
GET /api/v1/orderBookOrders?market_id=0&limit=250
```

Returns individual resting orders (not aggregated levels). Useful for initial book seed or reconnect.
Max 250 orders, weight 300.

## Authentication

### Public endpoints — no authentication required

All market data (prices, OHLCV, orderbook, trades, funding rates) is fully public.

### Account / WebSocket auth tokens

Auth tokens are signed strings for read access to account channels:

```
Standard:  {expiry_unix}:{account_index}:{api_key_index}:{random_hex}
Read-only: ro:{account_index}:{single|all}:{expiry_unix}:{random_hex}
```

Standard tokens expire in max 8 hours. Read-only tokens have 1 day minimum, 10 year maximum expiry.

### Trading — EIP-712 / ECDSA transaction signing

Write operations are not simple HTTP requests — they are signed L2 transactions submitted via `POST /api/v1/sendTx`:

1. Fetch next nonce: `GET /api/v1/nextNonce?account_index=X&api_key_index=Y`
2. Construct transaction fields (market_id, base_amount, price, side, order_type, nonce, ...)
3. Sign with ECDSA using the API key private key (secp256k1 curve)
4. Submit `{tx_type, tx_info, signature}` payload

Nonce must increment by 1 per API key per transaction. Out-of-order nonces are rejected.

API key indexes 0-2 are reserved (desktop/mobile apps). Use indexes 3-254 for programmatic access.

### Environments

| Environment | REST base URL | WS URL |
|-------------|---------------|--------|
| Mainnet | `https://mainnet.zklighter.elliot.ai` | `wss://mainnet.zklighter.elliot.ai/stream` |
| Testnet | `https://testnet.zklighter.elliot.ai` | `wss://testnet.zklighter.elliot.ai/stream` |

## Markets

| Type | market_id range | Examples |
|------|----------------|---------|
| Perpetuals | 0 – 2047 | 0 = ETH, 1 = BTC, 2 = SOL |
| Spot | 2048+ | 2048 = ETH/USDC |

Market_id 255 is a sentinel meaning "all markets" in REST filter params — not a real market.

Funding rates for perp markets are computed from live orderbook impact prices every minute.

## File Structure

```
lighter/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # Auth token generation and ECDSA signing
├── endpoints.rs       # URLs, endpoint enum, symbol formatting
├── connector.rs       # LighterConnector + trait implementations
├── parser.rs          # JSON response parsing
├── websocket.rs       # WebSocket connection and orderbook management
└── research/
    ├── authentication.md    # Auth system deep dive
    ├── endpoints.md         # Full REST endpoint reference
    ├── l2_orderbook.md      # L2 orderbook capabilities
    └── websocket.md         # WebSocket channel reference
```

## Rate Limits

### WebSocket (per IP)

| Limit | Value |
|-------|-------|
| Max connections | 100 |
| Subscriptions per connection | 100 |
| Total subscriptions | 1,000 |
| Client messages per minute | 200 (excl. sendTx) |

### REST (per account tier)

| Tier | Limit |
|------|-------|
| Builder | 240,000 weighted req/min |
| Premium | 24,000 weighted req/min |
| Standard | 60 req/min |

Key endpoint weights: `orderBookOrders` = 300, `orderBooks` = 300, `candlesticks` = 300, `sendTx` = 6.

## Testing

```bash
# Integration tests (public endpoints, no keys required)
cargo test --test lighter_integration -- --nocapture

# Specific tests
cargo test --test lighter_integration test_get_price -- --nocapture
cargo test --test lighter_integration test_orderbook -- --nocapture
cargo test --test lighter_integration test_klines -- --nocapture

# Live tests (requires LIGHTER_* env vars, marked #[ignore])
cargo test --test lighter_live -- --nocapture --ignored
```

## Troubleshooting

### Orderbook gaps (begin_nonce mismatch)

Re-subscribe to the channel. The first message after subscribe is always a fresh full snapshot.
Do not try to patch the local book from a gapped update.

### Slow reader disconnect

Lighter aggressively disconnects WebSocket clients that cannot consume 50 ms updates fast enough.
Ensure the message handler does not block the receive loop.

### Nonce rejected on order placement

Query `/api/v1/nextNonce` before every transaction. Do not cache nonces across process restarts.
Failed transactions consume nonces — always re-query after any error.

### Invalid signature

Check that the private key corresponds to the registered public key for the given `api_key_index`.
Ensure transaction serialization order and integer encoding match the Lighter spec exactly.

## Documentation

- **API Reference:** https://apidocs.lighter.xyz
- **WebSocket Reference:** https://apidocs.lighter.xyz/docs/websocket-reference
- **Get Started:** https://apidocs.lighter.xyz/docs/get-started-for-programmers-1
- **Python SDK:** https://github.com/elliottech/lighter-python
- **Go SDK:** https://github.com/elliottech/lighter-go
- **Explorer:** https://explorer.elliot.ai

## License

Part of the NEMO trading system.
