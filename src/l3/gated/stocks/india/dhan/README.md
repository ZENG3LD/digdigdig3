# Dhan Indian Stock Broker Connector

Production-ready connector for Dhan (DhanHQ) — Indian stock broker supporting NSE, BSE, and MCX.

## Status

READY FOR USE - Authentication working, all endpoints implemented

## Quick Start

### 1. Get API Credentials

```
https://dhan.co → Login
→ Profile → DhanHQ Trading APIs
→ Generate API Key + Secret (1-year validity)
→ Note your Client ID
```

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export DHAN_CLIENT_ID="your_client_id"
export DHAN_API_KEY="your_api_key"
export DHAN_API_SECRET="your_api_secret"
```

**Windows PowerShell:**
```powershell
$env:DHAN_CLIENT_ID="your_client_id"
$env:DHAN_API_KEY="your_api_key"
$env:DHAN_API_SECRET="your_api_secret"
```

### 3. Test

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test dhan_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::india::dhan::DhanConnector;
use digdigdig3::core::{Credentials, Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = Credentials::from_env();
    let connector = DhanConnector::new(credentials, false).await?;

    // Get current price (Security ID based)
    let symbol = Symbol::new("1333", "INR"); // RELIANCE on NSE_EQ
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("RELIANCE: ₹{}", price);

    // Get account balance
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for balance in balances {
        println!("{}: ₹{}", balance.asset, balance.total);
    }

    // Place market order (static IP required for order APIs)
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        Quantity::from(1.0),
        AccountType::Spot,
    ).await?;
    println!("Order placed: {}", order.id);

    // Dhan-specific: holdings and positions
    let holdings = connector.get_holdings().await?;
    let positions = connector.get_positions_detail().await?;

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices (LTP)
- ✅ OHLCV snapshots
- ✅ L2 orderbook — 5-level REST snapshot (all exchanges)
- ✅ L2 orderbook — 5-level WebSocket streaming (all exchanges)
- ✅ L2 orderbook — 20-level WebSocket streaming (NSE only, paid)
- ✅ L2 orderbook — 200-level WebSocket streaming (NSE only, paid)
- ✅ Historical intraday and EOD data
- ✅ Ticker / quote snapshots (VWAP, OI, circuit limits)

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ Order status tracking
- ✅ Order amendment
- ✅ Order cancellation
- ✅ F&O (futures and options) trading

### Account
- ✅ Balance queries
- ✅ Holdings (long-term delivery positions)
- ✅ Intraday positions
- ✅ F&O positions
- ✅ Trade history / order history

### NOT Supported
- ❌ Crypto trading (stocks/derivatives broker only)
- ❌ BSE for 20-level or 200-level depth (NSE only for deep L2)
- ❌ MCX for 20-level or 200-level depth (NSE only for deep L2)
- ❌ Testnet via separate URL (sandbox uses production URL with sandbox token)

## L2 Orderbook

Dhan provides three depth tiers, each with different access methods, exchange coverage, and instrument limits.

| Tier | Levels | Delivery | Max Instruments | Exchanges |
|------|--------|----------|-----------------|-----------|
| Standard | 5 | REST snapshot + WebSocket | 1,000/req (REST), 5,000/conn (WS) | NSE, BSE, MCX |
| Deep | 20 | WebSocket only | 50 per connection | NSE EQ + FNO only |
| Full | 200 | WebSocket only | 1 per connection | NSE EQ + FNO only |

### Depth Tier Requirements

- **5-level**: Included free with Dhan account (market data subscription not required)
- **20-level and 200-level**: Require Data API subscription — INR 499/month + taxes

### WebSocket Endpoints

```
5-level:   wss://api-feed.dhan.co
20-level:  wss://depth-api-feed.dhan.co/twentydepth
200-level: wss://full-depth-api.dhan.co/twohundreddepth
```

All WebSocket responses are **binary, Little Endian**. Subscription requests are JSON.

### Binary Packet Format

**5-level full packet** (163 bytes): 8-byte header + 55-byte quote fields + 100-byte depth (5 levels × 20 bytes each)

**20-level and 200-level packets** (332 / 3,212 bytes per side):
- 12-byte header: message length, response code (41 = bid, 51 = ask), exchange segment, security ID, row count
- Per level (16 bytes): `f64` price, `u32` quantity, `u32` order count

Depth is delivered as **full snapshots per update** (not incremental deltas). Bid and ask arrive as separate packets.

### REST Snapshot (5-level)

```bash
curl -X POST "https://api.dhan.co/v2/marketfeed/quote" \
  -H "access-token: <ACCESS_TOKEN>" \
  -H "client-id: <CLIENT_ID>" \
  -H "Content-Type: application/json" \
  -d '{"NSE_EQ": [1333, 11536], "BSE_EQ": [500325]}'
```

Rate limit: 1 request/second. Up to 1,000 instruments per request.

### Instrument IDs

Dhan uses numeric Security IDs, not ticker symbols. Examples:

| Symbol | SecurityId | Exchange |
|--------|------------|----------|
| RELIANCE | 1333 | NSE_EQ |
| INFY | 1594 | NSE_EQ |
| TCS | 11536 | NSE_EQ |

The full instrument master (SecurityId ↔ symbol mapping) is available for download from the Dhan data portal.

## Authentication

**Type:** JWT token (no HMAC request signing)

**Flow:**
1. Exchange `client_id` + `api_key` + `api_secret` for a daily access token via `POST /access-token`
2. Token is valid for 24 hours — must be refreshed daily
3. Pass token as `access-token` header on all REST calls, or as `token` query param on WebSocket connections

**Headers (REST):**
```http
access-token: <ACCESS_TOKEN>
client-id: <CLIENT_ID>
Content-Type: application/json
Accept: application/json
```

**WebSocket auth (query params):**
```
?version=2&token=<ACCESS_TOKEN>&clientId=<CLIENT_ID>&authType=2
```

**Static IP requirement:** Mandatory for all Order APIs (place, modify, cancel). Market data read-only access does not require a static IP.

**Sandbox mode:** Dhan sandbox uses the same API URLs as production. Sandbox vs. live is determined solely by the access token type — not by a different base URL.

## Rate Limits

| API Category | Limit |
|-------------|-------|
| General (non-trading) | 20 req/sec |
| Order APIs | 25 req/sec |
| Data APIs | 10 req/sec |
| Quote APIs (`/marketfeed/*`) | 1 req/sec |
| Orders per day | 5,000 max |
| WebSocket connections | 5 per user |
| Instruments per WS connection (live feed) | 5,000 |
| Instruments per subscribe message (live feed) | 100 |
| Instruments per 20-depth WS connection | 50 |
| Instruments per 200-depth WS connection | 1 |

## Files

```
dhan/
├── README.md         # This file
├── mod.rs            # Module exports
├── auth.rs           # JWT token generation and refresh
├── endpoints.rs      # URLs, endpoint enum, exchange segment mapping
├── connector.rs      # Trait implementations (MarketData, Trading, Account, Positions)
├── parser.rs         # JSON and binary response parsing
├── websocket.rs      # WebSocket connection (binary LE packet decoding)
└── research/
    ├── l2_orderbook.md       # Depth tiers, binary formats, WS channels
    ├── endpoints_full.md     # Full REST endpoint reference
    ├── authentication.md     # Auth flow and token management
    ├── websocket_full.md     # WebSocket protocol details
    ├── api_overview.md       # API overview and capabilities
    ├── tiers_and_limits.md   # Subscription tiers and rate limits
    ├── data_types.md         # Data types and segment codes
    └── coverage.md           # Exchange and instrument coverage
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DHAN_CLIENT_ID` | Your Dhan client identifier |
| `DHAN_API_KEY` | API key from Dhan profile |
| `DHAN_API_SECRET` | API secret from Dhan profile |

## Testing

```bash
# All integration tests
cargo test --test dhan_integration -- --nocapture

# Specific tests
cargo test --test dhan_integration test_ping -- --nocapture
cargo test --test dhan_integration test_get_price -- --nocapture
cargo test --test dhan_integration test_orderbook_5level -- --nocapture
cargo test --test dhan_integration test_get_balance -- --nocapture

# Unit tests only
cargo test --lib dhan

# Live tests (require real credentials + Data API subscription for L2)
cargo test --test dhan_live -- --nocapture --ignored
```

## Troubleshooting

### 401 Unauthorized

**Cause:** Expired or invalid access token

**Solution:**
1. Access tokens expire daily — re-run token generation
2. Verify `DHAN_CLIENT_ID`, `DHAN_API_KEY`, `DHAN_API_SECRET` are set correctly
3. Check token was generated for the correct environment (sandbox vs. production)

### Order API returns 403

**Cause:** Request originating from a non-whitelisted IP

**Solution:**
- Trading (order placement/modification/cancellation) requires a static IP
- Whitelist your IP from the Dhan API settings page
- Market data read calls are not affected by this restriction

### 20-level / 200-level depth returns nothing

**Cause:** Data API subscription not active, or non-NSE instrument used

**Solution:**
- Subscribe to Data API (INR 499/month) from the Dhan platform
- Confirm instrument is on NSE_EQ or NSE_FNO — BSE and MCX are not supported for deep L2
- 200-level: only 1 instrument per WebSocket connection; do not send multi-instrument subscribe

### WebSocket binary parse errors

**Cause:** Incorrect endianness or wrong offset for packet tier

**Solution:**
- All fields are Little Endian — use `i32::from_le_bytes`, `f64::from_le_bytes`, etc.
- 5-level full packet (response code 8): depth starts at offset 62, not 63
- 20/200-level: header is 12 bytes (not 8), depth levels are 16 bytes each

## Documentation

- **Official API:** https://dhanhq.co/docs/v2/
- **Live Market Feed:** https://dhanhq.co/docs/v2/live-market-feed/
- **Full Market Depth:** https://dhanhq.co/docs/v2/full-market-depth/
- **Market Quote:** https://dhanhq.co/docs/v2/market-quote/
- **Data API Subscription:** https://dhan.co/support/platforms/dhanhq-api/how-does-the-dhanhq-data-api-subscription-work/
- **Sign up:** https://dhan.co/

## License

Part of the NEMO trading system.
