# CryptoCompare Connector

Crypto data aggregator connector. Now part of CoinDesk/CCData. Market data only — no trading.

## Status

Ready for use — authentication working, REST endpoints implemented, WebSocket pending.

## Quick Start

### 1. Get API Key

```
https://www.cryptocompare.com/
→ Create account
→ https://www.cryptocompare.com/cryptopian/api-keys
→ Create API Key → set "Read All Price Streaming and Polling Endpoints"
→ Copy key (shown once)
```

Free tier available. No credit card required.

### 2. Set Environment Variable

**Linux/macOS/Git Bash:**
```bash
export CRYPTOCOMPARE_API_KEY="your_api_key"
```

**Windows PowerShell:**
```powershell
$env:CRYPTOCOMPARE_API_KEY="your_api_key"
```

### 3. Test

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test cryptocompare_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::data_feeds::cryptocompare::CryptoCompareConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = CryptoCompareConnector::from_env();

    // Get current price (CCCAGG aggregate across all exchanges)
    let btc = Symbol::new("BTC", "USD");
    let price = connector.get_price(btc.clone(), AccountType::Spot).await?;
    println!("BTC/USD: ${}", price);

    // Get OHLCV history (daily bars, full history available)
    let klines = connector.get_klines(btc, "1d", Some(30), AccountType::Spot).await?;
    println!("Got {} daily candles", klines.len());

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices (CCCAGG aggregate + per-exchange)
- ✅ Historical OHLCV — daily (full), hourly (full), minute (7 days free / 30 days Starter / 1 year Pro)
- ✅ Multi-pair price queries (`pricemulti`)
- ✅ Ticker snapshots
- ✅ Exchange metadata (170+ exchanges)
- ✅ Coin list (5,700+ cryptocurrencies)
- ✅ News aggregation
- ✅ Social metrics (Reddit, Twitter, Facebook, GitHub)
- ✅ WebSocket streaming (trades, tickers — channels 0, 2, 5)

### NOT Supported
- ❌ Trading (data provider only)
- ❌ Account management
- ❌ L2 orderbook — free tier
- ❌ Tick/raw trade data download — free tier
- ❌ L3 orderbook data (any tier)
- ❌ Futures/options market data

## L2 Orderbook

L2 orderbook requires a paid plan. Three access surfaces:

| Surface | Tier | Description |
|---------|------|-------------|
| WebSocket Channel 16 (`ORDERBOOK_L2`) | Starter+ (~$80/mo) | Real-time snapshot + delta updates |
| REST `/data/ob/l2/snapshot` | Starter+ | Per-request full snapshot |
| Historical REST (CCData API) | Professional+ (~$200/mo) | Per-minute snapshots since Sept 2020 |
| Data Streamer (enterprise) | Enterprise + IP whitelist | Real-time with CCSEQ sequence numbers |

### WebSocket Channel 16 — Subscription

```json
{ "action": "SubAdd", "subs": ["16~Kraken~BTC~USD"] }
```

**Snapshot message** (`TYPE: "16"`) — full book on first subscribe:
```json
{
  "TYPE": "16", "M": "Kraken", "FSYM": "BTC", "TSYM": "USD",
  "BIDS": [{"P": 45000.0, "Q": 1.5}],
  "ASKS": [{"P": 45001.0, "Q": 1.2}],
  "TS": 1706280000
}
```

**Delta message** (`TYPE: "16~UPDATE"`) — changed levels only, `Q: 0` means level removed:
```json
{
  "TYPE": "16~UPDATE", "M": "Kraken", "FSYM": "BTC", "TSYM": "USD",
  "BID_CHANGES": [{"P": 45000.0, "Q": 2.0}],
  "ASK_CHANGES": [{"P": 45001.0, "Q": 0}],
  "TS": 1706280001
}
```

**Note:** Legacy Channel 16 has no sequence numbers or checksums. For gap recovery, re-subscribe to get a fresh snapshot.

### REST Snapshot

```bash
GET https://min-api.cryptocompare.com/data/ob/l2/snapshot?fsym=BTC&tsym=USD&e=Kraken&api_key=KEY
```

Full book returned. No configurable depth.

## Authentication

**Type:** API key — no HMAC, no OAuth, no signature.

**Preferred method:** Query parameter
```
https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD&api_key=YOUR_KEY
```

**Alternative:** Authorization header
```http
Authorization: Apikey YOUR_KEY
```

Some endpoints work without a key but at very low limits (10-20 req/min). Always use a key.

**Free tier attribution:** Must display "Powered by CryptoCompare" in any application. Paid tiers remove this requirement.

## Tiers and Pricing

| Tier | Price | Rate Limit | Minute History | Orderbook | Attribution |
|------|-------|------------|----------------|-----------|-------------|
| Free | $0 | 50/s, 1k/min, 150k/hr | 7 days | No | Required |
| Starter | ~$80/mo | ~300/min | 30 days | Yes (Ch. 16) | No |
| Professional | ~$200/mo | ~1k/min | 1 year | Yes + REST snapshot | No |
| Enterprise | Custom | Up to 40k/s | Unlimited | Full (Data Streamer + CCSEQ) | No |

Rate limit errors return HTTP 200 with JSON `"Type": 99` — not HTTP 429.

Check usage:
```bash
GET https://min-api.cryptocompare.com/stats/rate/limit?api_key=YOUR_KEY
```

## Files

```
cryptocompare/
├── README.md           # This file
├── mod.rs              # Module exports
├── auth.rs             # API key injection
├── endpoints.rs        # URLs and endpoint enum
├── connector.rs        # Trait implementations
├── parser.rs           # JSON parsing
├── websocket.rs        # WebSocket streaming (channels 0, 2, 5, 16)
└── research/
    ├── api_overview.md
    ├── authentication.md
    ├── endpoints_full.md
    ├── websocket_full.md
    ├── tiers_and_limits.md
    ├── l2_orderbook.md
    ├── data_types.md
    └── coverage.md
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `CRYPTOCOMPARE_API_KEY` | Yes (recommended) | API key from cryptocompare.com/cryptopian/api-keys |

## Testing

```bash
# Integration tests (REST)
cargo test --test cryptocompare_integration -- --nocapture

# Live tests (real API calls, requires key)
cargo test --test cryptocompare_live -- --nocapture --ignored

# Unit tests
cargo test --lib cryptocompare
```

## Base URLs

| API | URL |
|-----|-----|
| Legacy REST | `https://min-api.cryptocompare.com` |
| New CCData REST | `https://data-api.cryptocompare.com` |
| WebSocket | `wss://streamer.cryptocompare.com/v2` |

## Documentation

- API docs: https://min-api.cryptocompare.com/documentation
- CCData developer portal: https://developers.coindesk.com
- API key management: https://www.cryptocompare.com/cryptopian/api-keys
- Sign up: https://www.cryptocompare.com/

## License

Part of the NEMO trading system.
