# Yahoo Finance Data Feed Connector

Free, unofficial data aggregator with massive multi-asset coverage. No API key needed.

> **WARNING: UNOFFICIAL API.** Yahoo Finance shut down its official API in 2017. All
> endpoints used here are reverse-engineered from Yahoo's own website. They can break
> without notice. Personal use only per Yahoo's Terms of Service.

## Status

✅ **WORKING** - Migrated to `/v8/finance/chart` endpoint (January 2026 fix)

The original `/v7/finance/quote` endpoint returned 401 as of January 2026. The connector
now uses the chart endpoint, which is what Yahoo's own website uses and is unlikely to
disappear.

## Quick Start

No setup required. No API key. No account.

```rust
use digdigdig3::data_feeds::yahoo::YahooFinanceConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // No authentication needed for most endpoints
    let connector = YahooFinanceConnector::new();

    // US stock price
    let aapl = Symbol::new("AAPL", "USD");
    let ticker = connector.get_ticker(aapl, AccountType::Spot).await?;
    println!("AAPL: ${}", ticker.last_price);

    // Crypto (use Yahoo format: "BTC-USD")
    let btc = Symbol::new("BTC", "USD");
    let klines = connector.get_klines(btc, "1d", Some(30), AccountType::Spot, None).await?;
    println!("Got {} daily candles", klines.len());

    // Yahoo-specific extended methods
    let options = connector.get_options_chain("AAPL").await?;
    let profile  = connector.get_asset_profile("AAPL").await?;
    let earnings = connector.get_earnings("AAPL").await?;

    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices (15-20 second exchange delay)
- ✅ Historical OHLCV bars (1m, 5m, 15m, 1h, 1d, 1wk, 1mo)
- ✅ Pre-market and after-hours data
- ✅ Ticker snapshots (day high/low/volume/change%)
- ✅ Options chains (all strikes and expirations)
- ✅ Fundamental data (income statements, balance sheets, cash flows, earnings)
- ✅ Analyst ratings, institutional ownership, insider transactions
- ✅ Symbol search and screener
- ✅ Market summary (major indices)
- ✅ ESG scores, fund profiles, trending symbols

### NOT Supported
- ❌ L2 orderbook (Yahoo Finance has never provided bid/ask depth — see below)
- ❌ Trading (data aggregator only, no order placement)
- ❌ Account management (no accounts exist)
- ❌ WebSocket live stream (exists at `wss://streamer.finance.yahoo.com/` but not implemented — REST polling only)
- ❌ True real-time tick data (minimum 15-20 second delay)

### L2 Orderbook

Yahoo Finance provides **no orderbook data whatsoever** — no bid price, no ask price,
no depth-of-market. This is a fundamental limitation of the provider, not a missing
implementation. Yahoo is a retail-oriented data aggregator, not a trading venue.

All orderbook trait methods return `ExchangeError::UnsupportedOperation`.

For L2 data use exchange connectors: Binance (`@depth` WebSocket), KuCoin
(`/market/orderbook/level2`), or a paid provider like Polygon.io (US equities).

## Coverage

Yahoo Finance aggregates data across asset classes using its own symbol format:

| Asset Class | Symbol Format | Examples |
|-------------|---------------|---------|
| US Stocks | Standard ticker | `AAPL`, `MSFT`, `GOOGL` |
| International Stocks | TICKER.EXCHANGE | `SAP.DE`, `0700.HK` |
| Cryptocurrencies | BASE-QUOTE | `BTC-USD`, `ETH-USD` |
| Forex | PAIR=X | `EURUSD=X`, `GBPUSD=X` |
| Commodities | SYMBOL=F | `GC=F` (Gold), `CL=F` (Oil) |
| Indices | ^SYMBOL | `^GSPC` (S&P 500), `^DJI` (Dow) |
| ETFs | Standard ticker | `SPY`, `QQQ`, `IWM` |

## Historical Data Limits

Intraday history is restricted by interval:

| Interval | Max History |
|----------|-------------|
| 1m | Last 7 days |
| 2m, 5m, 15m, 30m, 60m, 90m | Last 60 days |
| 1h | Last 730 days |
| 1d, 1wk, 1mo | Full history (decades) |

## Authentication

Most endpoints require no authentication. The connector includes a cookie+crumb mechanism
for the historical CSV download endpoint (`/v7/finance/download/{symbol}`) which Yahoo
has required auth for since early 2026.

```rust
// Standard usage — no auth
let connector = YahooFinanceConnector::new();

// With cookie/crumb for historical CSV download
let mut connector = YahooFinanceConnector::new();
connector.auth_mut().set_cookie("your_yahoo_cookie");
connector.obtain_crumb().await?;  // Fetches crumb from Yahoo's /v1/test/getcrumb
```

The chart endpoint (`/v8/finance/chart`) used for all standard OHLCV queries does not
require authentication.

## Rate Limits

Yahoo provides no official rate limit documentation. Community-observed limits:

| Metric | Observed | Confidence |
|--------|----------|-----------|
| Requests/hour | ~2000 per IP | High |
| Requests/second | ~5-10 burst | Medium |
| Response on limit exceeded | HTTP 429, plain text | - |

There are no rate limit headers in responses. The only signal of rate limiting is a 429
status code with body `Too Many Requests`.

Recommendations:
- Keep request rate below 2 req/second
- Implement exponential backoff on 429 (start at 30s, double each retry)
- Use the multi-symbol quote endpoint where available to reduce request count
- Cache responses — fundamentals can be cached for hours

## Stability & Risk

**Risk level: MEDIUM.**

Yahoo has progressively disabled endpoints over the years. The `/v7/finance/quote`
endpoint stopped working in January 2026. The current implementation uses
`/v8/finance/chart`, which powers Yahoo Finance's own website and is therefore
unlikely to disappear, but there are no guarantees.

Known-working endpoints as of January 2026:

| Endpoint | Status |
|----------|--------|
| `/v8/finance/chart/{symbol}` | Working (primary) |
| `/v1/finance/search` | Working |
| `/v6/finance/quote/marketSummary` | Working |
| `/v10/finance/quoteSummary/{symbol}` | Requires cookie/crumb |
| `/v7/finance/quote` | Broken (401 since Jan 2026) |

Monitor the [yfinance](https://github.com/ranaroussi/yfinance) and
[yahoo-finance2](https://github.com/gadicc/yahoo-finance2) GitHub issue trackers for
early warning of endpoint changes.

## Files

```
yahoo/
├── README.md        # This file
├── mod.rs           # Module exports
├── auth.rs          # Cookie + crumb authentication
├── endpoints.rs     # URL construction, endpoint enum, symbol formatting
├── connector.rs     # Trait implementations (MarketData, etc.)
├── parser.rs        # JSON parsing for chart/quoteSummary/options responses
├── websocket.rs     # WebSocket stub (not implemented)
└── research/
    ├── api_overview.md          # Provider overview and licensing
    ├── API_REALITY_2026.md      # 2026 status, endpoint migration, alternatives
    ├── endpoints_full.md        # Complete endpoint reference with parameters
    ├── tiers_and_limits.md      # Rate limits, RapidAPI proxy options
    └── l2_orderbook.md          # Why L2 is unavailable
```

## Testing

```bash
cd zengeld-terminal/crates/connectors/crates/v5

# Integration tests (hit real Yahoo Finance API)
cargo test --test yahoo_integration -- --nocapture

# Live tests (ignored by default, require network)
cargo test --test yahoo_live -- --nocapture --ignored

# Unit tests only
cargo test --lib yahoo
```

## Alternatives

If Yahoo Finance becomes too unreliable:

| Provider | Cost | Real-time | Official | L2 | Notes |
|----------|------|-----------|----------|----|-------|
| Finnhub | Free / $30-90/mo | Yes | Yes | Partial (paid) | Best free official alternative |
| Twelve Data | Free / $30-80/mo | Yes | Yes | No | Good stock coverage |
| Alpha Vantage | $50+/mo | 15-min delay | Yes | No | Only 25 req/day free |
| Polygon.io | $29+/mo | Yes | Yes | Yes (stocks) | Best for US equities L2 |
| RapidAPI Yahoo proxy | $10-200/mo | 15-20s delay | Proxy | No | Same Yahoo data, paid |

## Legal

Yahoo Finance data is **personal use only** per Yahoo's Terms of Service. Commercial use
and data redistribution are prohibited. See:
https://legal.yahoo.com/us/en/yahoo/terms/otos/index.html
