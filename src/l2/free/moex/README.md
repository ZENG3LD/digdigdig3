# MOEX (Moscow Exchange) ISS Connector

Data connector for Moscow Exchange — Russian stocks, bonds, FX, and derivatives via the ISS (Informational & Statistical Server) REST API.

## Status

**DATA ONLY** - No trading operations. ISS is a read-only market data API.

## Quick Start

### 1. Free Access (No Setup Required)

The ISS API is publicly accessible without credentials. Data is delayed by 15 minutes.

```bash
curl "https://iss.moex.com/iss/engines/stock/markets/shares/securities/SBER/orderbook.json"
```

### 2. Real-Time Access (MOEX Passport)

Register at `https://passport.moex.com` (free). Provides real-time data via Basic auth cookie.

### 3. Set Environment Variables (Optional)

**Linux/macOS/Git Bash:**
```bash
export MOEX_USERNAME="your_moex_passport_email"
export MOEX_PASSWORD="your_moex_passport_password"
export MOEX_ALGOPACK_KEY="your_algopack_api_key"   # optional, paid
```

**Windows PowerShell:**
```powershell
$env:MOEX_USERNAME="your_moex_passport_email"
$env:MOEX_PASSWORD="your_moex_passport_password"
$env:MOEX_ALGOPACK_KEY="your_algopack_api_key"
```

### 4. Test

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test moex_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::russia::moex::{MoexConnector, MoexAuth};
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Public connector — free tier, 15-minute delay, no setup
    let connector = MoexConnector::new_public();

    // Get current price for Sberbank
    let symbol = Symbol::new("SBER", "RUB");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("SBER: {} RUB", price);

    // Get historical candles (1h interval)
    let klines = connector.get_klines(symbol, "60", Some(100), AccountType::Spot).await?;
    println!("Got {} hourly candles", klines.len());

    Ok(())
}
```

```rust
// With MOEX Passport authentication — real-time data
let auth = MoexAuth::new("user@email.com", "password");
let connector = MoexConnector::new(auth);

// Or load from environment variables
let connector = MoexConnector::from_env();
```

## Features

### Market Data
- ✅ Prices (real-time with auth, 15-min delay without)
- ✅ Historical OHLCV candles (1m, 10m, 1h, 1d, 1w, 1M, 1Q; history from 1997+)
- ✅ L2 orderbook snapshots (10x10 equities/bonds/FX; 5x5 futures/options)
- ✅ Recent trades
- ✅ Ticker / last quote
- ✅ Security metadata and specifications
- ✅ Indices (IMOEX, RTSI, sector indices)
- ✅ Dividends and corporate actions
- ✅ Fundamentals (IFRS and Russian accounting standards)
- ✅ Derivatives metadata (FORTS futures and options)

### Markets
- ✅ Equities (Russian stocks, TQBR board)
- ✅ Bonds (OFZ government bonds, TQOB; corporate bonds, TQCB)
- ✅ FX (USD/RUB, EUR/RUB, CNY/RUB — CETS board)
- ✅ Futures (FORTS, RFUD board)
- ✅ Options (FORTS, ROPD board)

### NOT Supported
- ❌ Trading (ISS is data-only — no order placement)
- ❌ Account balances or positions
- ❌ Real-time streaming L2 (no public WebSocket for equities/futures orderbook)
- ❌ Full-depth order book (10x10 max via REST; unlimited depth requires FAST/SIMBA institutional feed)
- ❌ Sub-second polling (practical minimum ~1s; rate limits not published but ~1 req/s per IP)

## L2 Orderbook

### What is Available

MOEX ISS provides orderbook snapshots via REST polling — not streaming.

| Asset Class | Endpoint Engine/Market | Depth | Board |
|-------------|------------------------|-------|-------|
| Equities | `stock/shares` | 10 bid + 10 ask | TQBR |
| Bonds | `stock/bonds` | 10 bid + 10 ask | TQCB |
| FX / Currencies | `currency/selt` | 10 bid + 10 ask | CETS |
| Futures | `futures/forts` | 5 bid + 5 ask | RFUD |
| Options | `futures/options` | 5 bid + 5 ask | ROPD |

### Endpoint Pattern

```
GET /iss/engines/{engine}/markets/{market}/securities/{secid}/orderbook.json
GET /iss/engines/{engine}/markets/{market}/boards/{board}/securities/{secid}/orderbook.json
```

Example:
```bash
# Sberbank orderbook (free, 15-min delayed)
curl "https://iss.moex.com/iss/engines/stock/markets/shares/securities/SBER/orderbook.json"

# Same endpoint via ALGOPACK (real-time, Bearer token)
curl -H "Authorization: Bearer YOUR_KEY" \
  "https://apim.moex.com/iss/engines/stock/markets/shares/boards/tqbr/securities/SBER/orderbook.json"
```

### Response Fields

ISS returns a columnar JSON format. The `orderbook` block contains:

| Field | Type | Description |
|-------|------|-------------|
| `SECID` | string | Ticker (e.g. `"SBER"`) |
| `BOARDID` | string | Board code (e.g. `"TQBR"`) |
| `BUYSELL` | string | `"B"` = Bid, `"S"` = Ask |
| `PRICE` | float64 | Price level |
| `QUANTITY` | int32 | Volume in lots at this level |
| `SEQNUM` | int64 | Data packet sequence number |
| `UPDATETIME` | string | Last update time (`HH:MM:SS`) |
| `DECIMALS` | int32 | Price decimal precision |

Rows are sorted: Bids descending, Asks ascending.

### No Public Streaming

The WebSocket endpoint at `wss://wss-api.moex.com` is limited to OTC Bonds and marketplace products — it does not stream equities or FORTS L2 orderbook data.

The ISS WebSocket (`wss://iss.moex.com/infocx/v3/websocket`) uses STOMP protocol and supports subscriptions but is restricted to the same data available via REST.

For true real-time streaming L2 at institutional quality, MOEX operates FAST/SIMBA binary UDP multicast feeds — these require a co-location or dedicated connectivity agreement with MOEX and are not accessible via public API.

## Authentication Tiers

### Tier 1 — Free Public (No credentials)

- Data delay: **15 minutes**
- Endpoint: `https://iss.moex.com/iss/`
- No registration required
- Suitable for research, backtesting, non-time-sensitive applications

### Tier 2 — Real-Time ISS (MOEX Passport)

- Data delay: **none**
- Requires free MOEX Passport account at `passport.moex.com`
- Authentication: HTTP Basic → session cookie
- Same depth limits as Tier 1 (10x10 / 5x5)
- Suitable for algorithmic strategies with second-scale latency

```
POST https://passport.moex.com/authenticate
Authorization: Basic base64(login:password)
→ Response: Set-Cookie: MicexPassportCookie=...
Include cookie on subsequent ISS requests.
```

### Tier 3 — ALGOPACK (Paid subscription)

- Data delay: **none**, plus advanced derived metrics
- API key from `data.moex.com` after purchasing ALGOPACK subscription
- Endpoint: `https://apim.moex.com/iss/`
- Adds `obstats` endpoint: aggregated orderbook statistics (spread, bid/ask imbalance, liquidity metrics)
- Covers equities (`eq`), futures/options (`fo`), FX (`fx`) segments
- Update cycle: ~10 seconds for obstats

### Tier 4 — FAST/SIMBA (Institutional)

- Full-depth, ultra-low latency binary multicast over UDP
- Equities/FX: SIMBA (SBE encoding) — order-by-order book, no depth limit
- Derivatives: FAST/Spectra — up to 50 levels per side
- Requires co-location or dedicated VPN/leased line contract with MOEX
- Not accessible via public internet

## Rate Limits

Not officially published. Community-reported threshold: ~1 request/second per IP before throttling. Heavy use of all-market bulk endpoints may trigger limits faster. There is no official per-key quota documented.

## Files

```
moex/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # MOEX Passport and ALGOPACK authentication
├── endpoints.rs       # ISS endpoint definitions and URL builders
├── connector.rs       # Trait implementations (MarketData, etc.)
├── parser.rs          # ISS columnar JSON parsing
├── websocket.rs       # STOMP WebSocket client (ISS infocx)
├── tests.rs           # Unit tests
└── research/
    └── l2_orderbook.md   # L2 depth capabilities reference
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `MOEX_USERNAME` | No | MOEX Passport login (email) for real-time data |
| `MOEX_PASSWORD` | No | MOEX Passport password |
| `MOEX_ALGOPACK_KEY` | No | ALGOPACK API key from data.moex.com |

Without any variables set, the connector operates in free public mode (15-minute delayed data).

## Testing

```bash
# Integration tests (free tier, no credentials needed)
cargo test --test moex_integration -- --nocapture

# Specific tests
cargo test --test moex_integration test_ping -- --nocapture
cargo test --test moex_integration test_get_price -- --nocapture
cargo test --test moex_integration test_get_orderbook -- --nocapture

# Real-time tests (requires MOEX_USERNAME + MOEX_PASSWORD)
cargo test --test moex_integration test_realtime -- --nocapture --ignored

# Unit tests
cargo test --lib moex
```

## Documentation

- **ISS API Reference:** https://iss.moex.com/iss/reference/
- **MOEX Programming Interface:** https://www.moex.com/a2920
- **MOEX Interfaces Overview:** https://www.moex.com/a7939 (ISS, FAST, SIMBA, FIX)
- **ALGOPACK Docs:** https://moexalgo.github.io/
- **ALGOPACK Datashop:** https://data.moex.com/products/algopack
- **MOEX Passport:** https://passport.moex.com

## License

Part of the NEMO trading system.
