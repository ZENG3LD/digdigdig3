# Paradex DEX Connector

Production-ready connector for Paradex — a perpetuals DEX built on StarkNet (Ethereum L2).

## Status

✅ **READY FOR USE** - Authentication working, all endpoints implemented

## Quick Start

### 1. Get Credentials

Paradex uses StarkNet cryptographic keys, not simple API key/secret pairs.

```
https://app.paradex.trade
→ Connect wallet (MetaMask or StarkNet wallet)
→ The connector derives your StarkNet L2 key from your L1 Ethereum key
→ Obtain a JWT token via POST /v1/auth
```

For testnet (Sepolia): `https://testnet.paradex.trade`

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export PARADEX_STARKNET_ACCOUNT="0x1234...your_starknet_address"
export PARADEX_STARKNET_PRIVATE_KEY="your_stark_private_key"
export PARADEX_TESTNET="true"   # optional, defaults to mainnet
```

**Windows PowerShell:**
```powershell
$env:PARADEX_STARKNET_ACCOUNT="0x1234...your_starknet_address"
$env:PARADEX_STARKNET_PRIVATE_KEY="your_stark_private_key"
$env:PARADEX_TESTNET="true"
```

### 3. Test

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test paradex_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::crypto::dex::paradex::ParadexConnector;
use digdigdig3::core::{Credentials, Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JWT token goes in api_key field; secret_key unused
    let credentials = Credentials::new("your_jwt_token", "");
    let connector = ParadexConnector::new(credentials, false).await?;

    // Get BTC perpetual price
    let symbol = Symbol::new("BTC-USD-PERP", "");
    let price = connector.get_price(symbol.clone(), AccountType::FuturesCross).await?;
    println!("BTC-USD-PERP: ${}", price);

    // Place market order
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        Quantity::from(0.01),
        AccountType::FuturesCross,
    ).await?;
    println!("Order placed: {}", order.id);

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices
- ✅ Order book snapshots (REST + WebSocket)
- ✅ Ticker / markets summary
- ✅ Historical OHLCV klines
- ✅ Best bid/offer (BBO) feed
- ✅ Public trade feed
- ✅ Funding rates

### Trading
- ✅ Market orders
- ✅ Limit orders (GTC, IOC, POST_ONLY)
- ✅ Stop orders (STOP_LIMIT, STOP_MARKET, TAKE_PROFIT_*)
- ✅ Batch order placement and cancellation
- ✅ Order amend (modify)
- ✅ Cancel all orders (with optional market filter)
- ✅ RPI (Retail Price Improvement) order type

### Account
- ✅ Balance queries
- ✅ Account info and history
- ✅ Open positions
- ✅ Fill history
- ✅ Funding payment history

### NOT Supported
- ❌ Spot trading (perpetuals only)
- ❌ Withdrawals / transfers (use Paradex UI)
- ❌ Options (PERP_OPTION types not yet wired)

## L2 Orderbook

Paradex provides two complementary WebSocket channels for orderbook data and a REST snapshot endpoint.

### Channels

| Channel | Throttle | Depth | Notes |
|---------|----------|-------|-------|
| `order_book.{MARKET}` | 50ms or 100ms | 15 levels max | Unified snapshot+delta feed |
| `order_book_deltas.{MARKET}` | None (raw) | Unlimited | Delta-only; seed from REST first |
| `bbo.{MARKET}` | None (event-driven) | BBO only | Fires only on price/size change |

### Snapshot vs Delta

The `order_book` channel uses a unified message format. The `update_type` field distinguishes message types:

- `"s"` — full snapshot: discard local book and rebuild from `inserts`
- `"d"` — incremental delta: apply `inserts`, `updates`, `deletes` to existing book

Subscription example:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "refresh_rate": "50ms",
    "depth": 15
  },
  "id": 1
}
```

### Sequence Numbers

All orderbook messages carry a monotonically increasing `seq_no` per market (shared across REST and WebSocket).

- **Gap detection:** if `received_seq_no != last_seq_no + 1`, the local book is stale
- **Recovery:** await the next snapshot message (`update_type: "s"`) or re-fetch REST `GET /v1/orderbook/{market}`
- **No checksum:** integrity relies on `seq_no` alone; no CRC32 or hash field

### Price Aggregation

The `price_tick` parameter collapses nearby price levels into aggregated buckets. Format uses underscores as decimal separators: `"0_1"` = 0.1, `"1_0"` = 1.0.

- Supported on REST (`?price_tick=0_1`) and `order_book` WS channel
- Not supported on `order_book_deltas` (always raw levels)

### Dual-Book Model (RPI)

Paradex operates two distinct orderbook views:

| Book | REST endpoint | RPI orders |
|------|---------------|------------|
| Standard (API) | `GET /v1/orderbook/{market}` | Excluded |
| Interactive | `GET /v1/orderbook/{market}/interactive` | Included |

RPI (Retail Price Improvement) orders are hidden from algorithmic traders and only match against non-API users. The standard API book is always uncrossed; the interactive book may appear crossed.

The REST response includes both `best_bid_api` / `best_ask_api` and `best_bid_interactive` / `best_ask_interactive` fields.

There is no separate WebSocket channel for the interactive book.

### Connection Notes

- Protocol: JSON-RPC 2.0 over WebSocket
- Server pings every 55 seconds; client must pong within 5 seconds
- Server disconnects if 2,000+ messages accumulate unprocessed
- Max 20 new connections/second, 600 connections/minute per IP

## Authentication

**Type:** JWT via StarkNet STARK signature (EIP-712 inspired, Pedersen hash)

### Flow

1. Sign an auth message using your StarkNet L2 private key to produce an `[r, s]` signature pair
2. `POST /v1/auth` with headers `PARADEX-STARKNET-ACCOUNT`, `PARADEX-STARKNET-SIGNATURE`, `PARADEX-TIMESTAMP`
3. Receive a JWT token (5-minute lifetime; refresh every 3 minutes)
4. Pass `Authorization: Bearer <jwt>` on all private REST endpoints
5. For WebSocket private channels: send a single `authenticate` message; no re-auth needed for the connection's lifetime

### Order Signing

Every order submission requires an additional STARK signature over the order fields (market, side, price, size, timestamp). This is separate from the JWT auth signature.

Signing performance in Rust is approximately 0.2 ms per signature.

### Subkeys

Subkeys are randomly generated keypairs registered to a main account. They have trading permissions but cannot withdraw or transfer funds. Recommended for automated bots.

### Rate Limits

| Endpoint group | Limit |
|----------------|-------|
| Auth (`POST /auth`) | 600 req/min per IP |
| Orders (place/cancel/amend) | 800 req/s or 17,250 req/min per account |
| Private GETs | 600 req/min per account |
| Public endpoints | 1,500 req/min per IP |

## Environments

### Mainnet
- REST: `https://api.prod.paradex.trade/v1`
- WebSocket: `wss://ws.api.prod.paradex.trade/v1`

### Testnet (Sepolia)
- REST: `https://api.testnet.paradex.trade/v1`
- WebSocket: `wss://ws.api.testnet.paradex.trade/v1`

Pass `testnet: true` to `ParadexConnector::new()` to use testnet.

## Files

```
paradex/
├── README.md          # This file
├── mod.rs             # Module exports (ParadexConnector, ParadexAuth, ParadexParser, ParadexWebSocket)
├── auth.rs            # JWT generation and STARK signature handling
├── endpoints.rs       # URL constants, endpoint enum, symbol formatting, kline resolution mapping
├── connector.rs       # ExchangeIdentity, MarketData, Trading, Account, Positions trait impls
├── parser.rs          # JSON parsing for REST and WebSocket responses
├── websocket.rs       # WebSocket connection with broadcast channel
└── research/
    ├── authentication.md    # Auth flow and STARK signing details
    ├── endpoints.md         # Full REST endpoint reference
    ├── websocket.md         # WebSocket channels and protocol
    └── l2_orderbook.md      # L2 orderbook capabilities in depth
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PARADEX_STARKNET_ACCOUNT` | Yes | StarkNet wallet address (0x-prefixed) |
| `PARADEX_STARKNET_PRIVATE_KEY` | Yes | StarkNet L2 private key |
| `PARADEX_TESTNET` | No | Set to `"true"` to use Sepolia testnet |

## Testing

```bash
# Integration tests (requires env vars set)
cargo test --test paradex_integration -- --nocapture

# Specific tests
cargo test --test paradex_integration test_ping -- --nocapture
cargo test --test paradex_integration test_get_price -- --nocapture
cargo test --test paradex_integration test_get_balance -- --nocapture

# Unit tests
cargo test --lib paradex

# Live tests (real orders — use testnet!)
cargo test --test paradex_live -- --nocapture --ignored
```

## Troubleshooting

### 401 Unauthorized

**Cause:** Expired or missing JWT token

**Solution:**
1. JWT tokens expire after 5 minutes — regenerate via `POST /v1/auth`
2. Verify `PARADEX_STARKNET_ACCOUNT` matches the address used to sign
3. Check that `PARADEX-TIMESTAMP` in the auth request is a current Unix timestamp (not milliseconds)

### Order Rejected (400)

**Cause:** Missing or invalid order signature

**Solution:**
- Ensure each order includes `signature` and `signature_timestamp`
- The signature must be computed over the current order fields using your StarkNet private key
- `signature_timestamp` is in milliseconds

### Orderbook Gap Detected

**Cause:** Missed WebSocket message (network blip or slow consumer)

**Solution:**
- For `order_book` channel: wait for the next `update_type: "s"` snapshot — it arrives automatically
- For `order_book_deltas` channel: re-fetch REST `GET /v1/orderbook/{market}` to reseed

### Connection Dropped by Server

**Cause:** Message queue exceeded 2,000 unprocessed messages

**Solution:**
- Drain the WebSocket receive buffer faster (dedicated read task)
- Reduce subscriptions or use `bbo` channel instead of full `order_book` if depth is not needed

## Documentation

- **Official API:** https://docs.paradex.trade/
- **Authentication:** https://docs.paradex.trade/trading/api-authentication
- **WebSocket:** https://docs.paradex.trade/ws/general-information/introduction
- **Code samples:** https://github.com/tradeparadex/code-samples
- **Python SDK:** https://github.com/tradeparadex/paradex-py

## License

Part of the NEMO trading system.
