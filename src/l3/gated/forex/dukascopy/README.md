# Dukascopy Forex Data Connector

Historical tick data connector for the Dukascopy public datafeed. Swiss forex broker and JForex platform operator. Data-provider only — no trading, no authentication required.

## Status

✅ **READY FOR USE** - Binary tick download implemented, LZMA decompression working, klines constructed from ticks

## Quick Start

No API keys needed. Data is public.

```rust
use digdigdig3::forex::dukascopy::DukascopyConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = DukascopyConnector::new();

    let symbol = Symbol { base: "EUR".to_string(), quote: "USD".to_string() };

    // Get hourly klines (constructed from tick data)
    let klines = connector.get_klines(symbol, "1h", Some(24), AccountType::Spot).await?;
    println!("Got {} candles", klines.len());

    Ok(())
}
```

## Features

### Market Data
- ✅ Historical tick data (bid/ask, tick-level granularity)
- ✅ Historical OHLCV (klines constructed from ticks)
- ✅ Forex pairs (EUR/USD, GBP/USD, USD/JPY, etc.)
- ✅ Metals (XAU/USD gold, XAG/USD silver)
- ✅ Data from 2003+ for major pairs
- ❌ Real-time data (binary datafeed is historical only)
- ❌ Order book
- ❌ Ticker / 24h stats

### Trading
- ❌ Not supported (data provider only)

### Account
- ❌ Not supported

## Authentication

**Type:** None — public datafeed, no credentials required.

The `DukascopyAuth` struct exists for API consistency but is a no-op. No headers or query parameters are added to requests.

```rust
// No auth setup needed
let connector = DukascopyConnector::new();     // equivalent to from_env()
```

## Data Access Method

Dukascopy does not provide a REST API. Data is downloaded as **LZMA-compressed binary files** (`.bi5` format):

```
https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5
```

- Each file covers exactly one hour of tick data
- Format: 20 bytes per tick (binary, big-endian)
- Month index is **0-based**: January = `00`, December = `11`
- Empty response (0 bytes) = no data for that hour (weekend / holiday / future)
- No authentication, no rate limiting documented

**Symbol format:** Concatenated uppercase, no separator — `EURUSD`, `GBPUSD`, `XAUUSD`.

## Files

```
dukascopy/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # No-op auth (public datafeed)
├── endpoints.rs       # URL building, symbol formatting, point values
├── parser.rs          # LZMA decompression, binary tick parsing
├── connector.rs       # Trait implementations
└── research/          # Research notes
```

## Environment Variables

None required.

## Testing

```bash
# All integration tests (downloads real data from Dukascopy)
cargo test --test dukascopy_integration -- --nocapture

# Unit tests (parser, auth, endpoints — no network)
cargo test --lib dukascopy
```

## Rate Limits

No documented rate limits. The datafeed is a public CDN-style file server. Be respectful — avoid bulk parallel downloads.

## Known Limitations

- No real-time data (JForex SDK required for live feed, not implemented here)
- No WebSocket
- Weekends and market holidays return empty files (normal — not an error)
- Data availability varies by pair and date; older dates may have gaps

## Documentation

- **Datafeed URL structure:** https://github.com/ninety47/dukascopy
- **JForex platform:** https://www.dukascopy.com/swiss/english/forex/jforex/
- **Historical data portal:** https://www.dukascopy.com/trading-tools/widgets/tools/historical_data_feed/

## License

Part of the NEMO trading system.
