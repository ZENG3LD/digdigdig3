# Polymarket Prediction Market Connector

Connector for Polymarket — a CLOB-based prediction market on Polygon. Markets trade probabilities (0.0–1.0 in USDC) for real-world event outcomes. Identified by `condition_id` (blockchain) and `token_id` (ERC-1155 YES/NO tokens).

## Status

✅ **READY FOR USE** - Public market data, order book, klines, WebSocket implemented

## Quick Start

No API keys needed for read access.

```rust
use digdigdig3::prediction::polymarket::PolymarketConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = PolymarketConnector::public();

    // List active markets
    let markets = connector.get_exchange_info(AccountType::Spot).await?;
    println!("Active markets: {}", markets.len());

    // Get YES probability for a market (condition_id as symbol.base)
    let symbol = Symbol::new("0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1", "USDC");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("YES probability: {:.1}%", price * 100.0);

    // Price history (klines)
    let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot, None).await?;
    println!("Got {} candles", klines.len());

    Ok(())
}
```

### WebSocket

```rust
use digdigdig3::prediction::polymarket::{ClobWebSocket, WsEvent};

let token_ids = vec!["71321045679...".to_string()]; // ERC-1155 token_id
let mut ws = ClobWebSocket::new(token_ids, false);
ws.connect().await?;

while let Ok(Some(event)) = ws.recv().await {
    match event {
        WsEvent::LastTradePrice(trade) => println!("Trade: {}", trade.price),
        WsEvent::BookSnapshot(book)   => println!("Snapshot: {} bids", book.bids.len()),
        WsEvent::PriceChange(changes) => println!("Delta: {} changes", changes.price_changes.len()),
        _ => {}
    }
}
```

## Features

### Market Data
- ✅ Active markets list (`get_exchange_info`)
- ✅ YES token probability price (`get_price`)
- ✅ Order book snapshot (`get_order_book`) — full depth, no limit
- ✅ Price history / klines (`get_klines`) — intervals: `1m`, `1h`, `6h`, `1d`, `1w`, `all`
- ✅ Ticker with midpoint (`get_ticker`)
- ✅ WebSocket real-time order book (snapshot + incremental deltas)
- ✅ WebSocket trade events (`last_trade_price`)
- ✅ WebSocket tick size change events
- ✅ Bulk order book (`POST /books`)
- ❌ Futures / funding rates (not applicable)

### Trading
- ❌ Not implemented (requires EIP-712 wallet + CLOB L2 auth)

### Account
- ❌ Not implemented

## Authentication

Polymarket uses a three-layer auth model:

| Layer | Method | Used for |
|-------|--------|----------|
| L0 | None | All read endpoints, WebSocket market channel |
| L1 | EIP-712 wallet signature | Deriving CLOB API credentials |
| L2 | HMAC-SHA256 | Order placement/cancellation, user WS channel |

**This connector implements L0 (public) and L2 header signing.** L1 wallet-based credential derivation is not yet implemented.

**L2 headers (authenticated requests):**
```http
POLY_ADDRESS:    0xYourWalletAddress
POLY_API_KEY:    uuid-format-api-key
POLY_SIGNATURE:  base64url(HMAC-SHA256(base64_secret, timestamp+method+path+[body]))
POLY_TIMESTAMP:  unix_ms
POLY_PASSPHRASE: your-passphrase
```

All market data read endpoints work **without any credentials**.

## Key Concepts

Polymarket uses a three-level identifier hierarchy:

| Level | Field | Description | Example |
|-------|-------|-------------|---------|
| Event | `slug` | Groups related markets | `"trump-2024"` |
| Market | `condition_id` | Blockchain condition ID (0x + 64 hex) | `"0x5f65...8f1"` |
| Token | `token_id` | ERC-1155 YES or NO token (long decimal string) | `"71321045..."` |

- All orderbook/price API calls require **`token_id`**, not `condition_id`
- Prices are probabilities: `0.65` = 65% chance the event resolves YES
- Each market has exactly two tokens: YES + NO, where `price_YES + price_NO ≈ 1.0`
- Minimum order: 1 USDC; tick size: `0.01` (may reduce to `0.001` near extremes)

**Getting token IDs:**
```
GET https://clob.polymarket.com/markets/<condition_id>
→ Response: tokens[{token_id, outcome: "Yes"}, {token_id, outcome: "No"}]
```

## L2 Orderbook

### REST

| Endpoint | Description | Rate limit |
|----------|-------------|------------|
| `GET /book?token_id=...` | Full order book snapshot | 1,500 req/10s |
| `POST /books` | Bulk snapshots (array of token IDs) | 500 req/10s |
| `GET /midpoint?token_id=...` | Mid price | 1,500 req/10s |
| `GET /price?token_id=...&side=BUY\|SELL` | Best bid or ask | 1,500 req/10s |
| `GET /spread?token_id=...` | Bid-ask spread | — |
| `GET /last-trade-price?token_id=...` | Last trade price | — |
| `GET /tick-size?token_id=...` | Minimum price increment | — |

Book response includes a `hash` (MD5-like) for integrity verification. Full depth is always returned — no `depth` parameter.

**Known issue (2025):** `GET /book` can return stale ghost data (`bid=0.01`, `ask=0.99`) for inactive markets. Use `GET /price` for reliable best-bid/ask.

### WebSocket

```
wss://ws-subscriptions-clob.polymarket.com/ws/market
```

Subscribe (send after connect):
```json
{"type": "market", "assets_ids": ["TOKEN_ID"], "initial_dump": true, "level": 2}
```

Send `"PING"` every 10s; server responds `"PONG"`.

**Event types:**

| Event | Trigger | Action |
|-------|---------|--------|
| `book` | On subscribe; after each trade | Replace entire local book |
| `price_change` | Order add / cancel / fill | Apply incremental delta: set `book[side][price] = size`; if `size == "0"` remove level |
| `last_trade_price` | Trade execution | Record trade; book snapshot follows |
| `tick_size_change` | Probability near extreme | Update minimum tick size |
| `best_bid_ask` | Top-of-book only (requires `custom_feature_enabled: true`) | Lightweight BBO update |

**Delta maintenance:**
```
on price_change:
    for change in changes:
        if change.size == "0":
            remove book[change.side][change.price]
        else:
            book[change.side][change.price] = change.size
```

`size` is always the **new aggregate total** at that level, never a diff. No sequence numbers — on reconnect, re-subscribe with `initial_dump: true` and wait for a `book` snapshot.

**Parser note:** The codebase uses `changes` as the array field name in `price_change` events; official docs use `price_changes`. Verify against live data.

**Price quirk:** Prices may omit leading zero (`".48"` instead of `"0.48"`). The codebase normalizes this in `normalize_price_in_place()`.

## APIs

| API | Base URL | Purpose |
|-----|----------|---------|
| CLOB | `https://clob.polymarket.com` | Order books, prices, markets |
| Gamma | `https://gamma-api.polymarket.com` | Events, enhanced market metadata |
| Data | `https://data-api.polymarket.com` | User positions, trades |
| WS | `wss://ws-subscriptions-clob.polymarket.com/ws/market` | Real-time market data |

## Files

```
polymarket/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # HMAC-SHA256 L2 signing
├── endpoints.rs       # API endpoint enum, base URLs, interval mapping
├── parser.rs          # JSON parsing, domain type conversions
├── connector.rs       # Trait implementations (ExchangeIdentity, MarketData)
├── websocket.rs       # WebSocket client, event types, reconnect logic
└── research/
    └── l2_orderbook.md    # L2 orderbook capabilities research
```

## Environment Variables

Required only for authenticated (trading) operations — not needed for market data:

```bash
export POLYMARKET_API_KEY="your-uuid-api-key"
export POLYMARKET_API_SECRET="your-base64-secret"
export POLYMARKET_PASSPHRASE="your-passphrase"
export POLYMARKET_WALLET_ADDRESS="0xYourPolygonWallet"
```

## Testing

```bash
# Integration tests (public endpoints, real Polymarket data)
cargo test --test polymarket_integration -- --nocapture

# Unit tests (parsing, auth, endpoints — no network)
cargo test --lib polymarket

# WebSocket test (requires network)
cargo test --test polymarket_integration test_websocket -- --nocapture --ignored
```

## Rate Limits

| Endpoint group | Limit |
|---------------|-------|
| `/book`, `/price`, `/midpoint` | 1,500 req / 10s |
| `/books` (bulk POST) | 500 req / 10s |
| General CLOB read | 9,000 req / 10s |
| `POST /order` | 3,500 req/10s burst; 36,000 req/10 min sustained |

Limits use sliding windows; requests are throttled rather than hard-rejected.

## Documentation

- **API reference:** https://docs.polymarket.com/api-reference/introduction.md
- **Rust CLOB client:** https://github.com/Polymarket/rs-clob-client
- **Community guide:** https://pm.wiki/learn/polymarket-api

## License

Part of the NEMO trading system.
