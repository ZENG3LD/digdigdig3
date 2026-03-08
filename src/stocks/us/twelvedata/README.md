# Twelvedata Connector

Multi-asset data provider connector for stocks, forex, crypto, ETFs, commodities, and indices.

## ⚠ Important: DATA PROVIDER ONLY

Twelvedata is a **data provider**, not a trading exchange. It provides:
- ✅ Market data (price, quotes, historical OHLCV)
- ✅ 100+ technical indicators
- ✅ Fundamental data (stocks only, Grow+ tier)
- ✅ WebSocket streaming (Pro+ tier)
- ❌ **NO trading/order execution**
- ❌ **NO account/balance information**
- ❌ **NO position management**

Trading-related methods will return `ExchangeError::UnsupportedOperation`.

## Features

### Multi-Asset Support
- **Stocks**: 60,000+ symbols (90+ exchanges globally)
- **Forex**: 200+ pairs (majors, minors, exotics)
- **Crypto**: Thousands of pairs (180+ exchanges)
- **ETFs**: 5,000+ (US + international)
- **Commodities**: 50+ (metals, energy, agriculture)
- **Indices**: 100+ global indices

### Technical Indicators
- 100+ built-in indicators (RSI, MACD, Bollinger Bands, SMA, EMA, etc.)
- Customizable parameters
- Historical indicator values

### Data Quality
- Real-time data (Pro+ plans)
- Historical data back to 1980s-1990s
- Extended hours data (US pre/post-market, Pro+ plans)
- Fundamentals (income statements, balance sheets, earnings, Grow+ plans)

## Quick Start

```rust
use digdigdig3::stocks::us::twelvedata::TwelvedataConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, ExchangeIdentity};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connector with API key
    let connector = TwelvedataConnector::new("your-api-key-here");

    // Or use environment variable (TWELVEDATA_API_KEY)
    let connector = TwelvedataConnector::from_env();

    // Or use demo key for testing (very limited)
    let connector = TwelvedataConnector::demo();

    // Get current price
    let symbol = Symbol::new("AAPL", "USD");
    let price = connector.get_price(symbol, AccountType::Spot).await?;
    println!("AAPL price: ${}", price);

    // Get full quote
    let ticker = connector.get_ticker(symbol.clone(), AccountType::Spot).await?;
    println!("Ticker: {:#?}", ticker);

    // Get historical klines
    let klines = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot).await?;
    println!("Got {} klines", klines.len());

    Ok(())
}
```

## Environment Setup

Set your API key as an environment variable:

```bash
export TWELVEDATA_API_KEY="your-api-key-here"
```

## Getting an API Key

### Free Tier (Basic Plan)
1. Sign up at [https://twelvedata.com/pricing](https://twelvedata.com/pricing)
2. Free tier includes:
   - 8 API calls per minute
   - 800 API calls per day
   - Basic endpoints (price, quote, time_series)
   - No WebSocket access

### Demo Key
For initial testing without signup:
```rust
let connector = TwelvedataConnector::demo();
```

**WARNING**: Demo key has severe rate limits and limited functionality.

### Paid Tiers
- **Grow** ($29+/mo): 55-377 calls/min, fundamentals, historical data
- **Pro** ($99+/mo): 610-1,597 calls/min, WebSocket, extended hours
- **Ultra** ($329+/mo): 2,584+ calls/min, advanced features
- **Enterprise**: Custom pricing

## Supported Methods

### Core Traits (Implemented)

```rust
// ExchangeIdentity
connector.exchange_id() -> ExchangeId
connector.exchange_name() -> &str
connector.exchange_type() -> ExchangeType::DataProvider
connector.is_testnet() -> bool

// MarketData
connector.get_price(symbol, account_type) -> f64
connector.get_ticker(symbol, account_type) -> Ticker
connector.get_klines(symbol, interval, limit, account_type) -> Vec<Kline>
connector.ping() -> ()

// Trading, Account, Positions
// ALL methods return ExchangeError::UnsupportedOperation
```

### Extended Methods (Provider-Specific)

```rust
// Symbol search
connector.symbol_search("AAPL") -> Value

// Reference data
connector.get_stocks() -> Value
connector.get_forex_pairs() -> Value
connector.get_cryptocurrencies() -> Value

// Market info
connector.market_state("NASDAQ") -> Value

// Technical indicators
connector.rsi(&symbol, "1day", 14) -> Value
connector.macd(&symbol, "1day") -> Value
```

## Symbol Formats

### Stocks
```rust
Symbol::new("AAPL", "USD")  // Apple stock
```

### Forex
```rust
Symbol::new("EUR", "USD")  // EUR/USD pair
```

### Crypto
```rust
Symbol::new("BTC", "USD")   // Bitcoin
Symbol::new("ETH", "USDT")  // Ethereum
```

## Intervals

Supported intervals for klines:
- Minutes: `1m`, `5m`, `15m`, `30m`, `45m`
- Hours: `1h`, `2h`, `4h`
- Days: `1d`
- Weeks: `1w`
- Months: `1M`

## Rate Limiting

### Free Tier Limits
- **8 requests per minute**
- **800 requests per day**
- No burst allowance

### Handling Rate Limits

The connector automatically handles rate limit errors:

```rust
match connector.get_price(symbol, AccountType::Spot).await {
    Err(ExchangeError::RateLimitExceeded { retry_after, message }) => {
        println!("Rate limit hit: {}", message);
        // Implement exponential backoff
    }
    Ok(price) => println!("Price: {}", price),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Error Handling

### Common Errors

```rust
ExchangeError::Api { code, message }           // API returned error
ExchangeError::RateLimitExceeded { .. }        // Rate limit hit
ExchangeError::Auth(..)                        // Invalid API key
ExchangeError::PermissionDenied(..)            // Requires higher tier
ExchangeError::UnsupportedOperation(..)        // Trading/account methods
ExchangeError::Parse(..)                       // JSON parsing failed
ExchangeError::Network(..)                     // Network/HTTP error
```

## Testing

### Run Integration Tests

```bash
# Set API key
export TWELVEDATA_API_KEY="your-key"

# Run all tests
cargo test --package digdigdig3 --test twelvedata_integration -- --nocapture

# Run specific test
cargo test --package digdigdig3 --test twelvedata_integration test_get_price -- --nocapture

# Run with demo key (limited)
cargo test --package digdigdig3 --test twelvedata_integration test_demo_connection -- --nocapture
```

## Important Notes

### 1. String Numerics
Time series values are returned as **strings** to preserve precision:
```json
{
  "open": "149.50000",
  "close": "150.25000"
}
```
The parser handles conversion to `f64`.

### 2. Null Values
Many fields may be `null` when data is unavailable. The parser handles this defensively:
```json
{
  "volume": null,
  "fifty_two_week": { "high": null }
}
```

### 3. Credit System
Different endpoints cost different credits:
- Basic data: 1 credit per symbol
- Technical indicators: 1 credit
- Fundamentals: 10-50 credits
- Market movers: 100 credits per request

### 4. WebSocket (Pro+ Only)
WebSocket streaming is only available on Pro+ plans:
```rust
use digdigdig3::stocks::us::twelvedata::TwelvedataWebSocket;

let ws = TwelvedataWebSocket::new("your-api-key");
// Connection, subscription, heartbeat management required
```

## Files Structure

```
twelvedata/
├── mod.rs           - Module exports and documentation
├── endpoints.rs     - API endpoints and symbol formatting
├── auth.rs          - API key authentication (simple header-based)
├── parser.rs        - JSON response parsing
├── connector.rs     - Main connector + trait implementations
├── websocket.rs     - WebSocket connector (Pro+ tier)
├── research/        - API research documentation (8 files, 3682 lines)
└── README.md        - This file
```

## Resources

- **Official Docs**: https://twelvedata.com/docs
- **Pricing**: https://twelvedata.com/pricing
- **Support**: https://support.twelvedata.com/
- **API Playground**: https://twelvedata.com/docs#request-parameters

## License

Part of the NEMO trading system. See project root for license information.
