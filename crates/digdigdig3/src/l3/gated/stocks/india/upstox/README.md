# Upstox Connector

Production connector for Upstox Indian stock broker and market data provider.

## Status

✅ **READY FOR USE** - OAuth2 authentication working, REST and WebSocket endpoints implemented

## Quick Start

### 1. Get API Credentials (5 minutes)

```
https://developer.upstox.com/
→ Login with your Upstox Demat account
→ Create New App → set Redirect URI
→ Copy API Key and API Secret
→ Complete OAuth flow to get Access Token
```

Upstox requires an active Demat account. API app creation is free; trading requires an Rs 499/month subscription.

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export UPSTOX_API_KEY="your_api_key"
export UPSTOX_API_SECRET="your_api_secret"
export UPSTOX_ACCESS_TOKEN="your_access_token"
```

**Windows PowerShell:**
```powershell
$env:UPSTOX_API_KEY="your_api_key"
$env:UPSTOX_API_SECRET="your_api_secret"
$env:UPSTOX_ACCESS_TOKEN="your_access_token"
```

### 3. Test

```bash
# Rust integration tests
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test upstox_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::india::upstox::UpstoxConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with environment variables
    let connector = UpstoxConnector::from_env(false).await?; // false = standard endpoint

    // Get current price (NSE equity)
    let symbol = Symbol::new("NSE_EQ|INE002A01018", "INR"); // Reliance
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("RELIANCE: Rs.{}", price);

    // Get historical candles (daily from 2000, intraday from 2022)
    let candles = connector.get_klines(symbol.clone(), "1h", Some(100), AccountType::Spot).await?;
    println!("Got {} candles", candles.len());

    // Get account balance
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for b in balances {
        println!("{}: Rs.{}", b.asset, b.total);
    }

    // Place market order
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        1.0, // quantity in shares
        AccountType::Spot,
    ).await?;
    println!("Order placed: {}", order.id);

    Ok(())
}
```

### HFT Mode (Low Latency)

```rust
// Use HFT endpoint for lower latency order routing
let connector = UpstoxConnector::from_env(true).await?; // true = HFT endpoint
```

## Features

### Market Data
- ✅ Real-time LTP quotes
- ✅ OHLC quotes
- ✅ Full market depth (5 levels)
- ✅ Historical OHLCV (daily from 2000, intraday from Jan 2022)
- ✅ Option chain with Greeks and IV
- ✅ Ticker snapshots

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ SL / SL-M orders
- ✅ GTT orders (Good Till Triggered)
- ✅ AMO (After Market Orders)
- ✅ Modify and cancel orders
- ✅ Multi-order APIs (batch place/modify/cancel)
- ✅ HFT endpoint for low-latency order routing

### Account
- ✅ Balance / margin queries
- ✅ Holdings (delivery equity)
- ✅ Positions (intraday + F&O)
- ✅ Order history and trade book
- ✅ Profile and fund statements

### WebSocket
- ✅ Market data feed (Protocol Buffers binary format)
- ✅ Portfolio stream (order + position updates)
- WebSocket market data: `wss://api.upstox.com/v2/feed/market-data-feed/protobuf`
- WebSocket portfolio: `wss://api.upstox.com/v2/feed/portfolio-stream-feed`

### NOT Supported
- ❌ Testnet via this connector (sandbox exists but not wired)
- ❌ Crypto trading (Indian broker only)
- ❌ International exchanges
- ❌ Commodity (MCX) trading through this connector currently

## Authentication

**Type:** OAuth 2.0 — Authorization Code flow

**Flow:**
1. Redirect user to: `https://api.upstox.com/v2/login/authorization/dialog?client_id=<key>&redirect_uri=<uri>&response_type=code`
2. User logs in and authorizes the app
3. Receive `code` via redirect callback
4. Exchange for access token: `POST https://api.upstox.com/v2/login/authorization/token`
5. Use token in all subsequent requests

**Authorization header:**
```http
Authorization: Bearer <access_token>
```

**Token lifetime:**
- Access token expires at 3:30 AM IST next day
- No refresh token mechanism — must re-authenticate daily
- Extended 1-year token available for read-only operations (contact Upstox support)
- API accessible only 5:30 AM to 12:00 AM IST (returns error UDAPI100074 outside window)

**Token renewal:** Re-run the OAuth flow each day or use the extended token for read-only use cases.

## Exchanges & Segments

| Exchange | Segment | Instruments |
|----------|---------|-------------|
| NSE | Equity (`NSE_EQ`) | ~2,000 actively traded stocks |
| BSE | Equity (`BSE_EQ`) | ~5,000 stocks |
| NSE | F&O (`NSE_FO`) | Index and stock futures + options |
| BSE | F&O (`BSE_FO`) | Sensex, Bankex futures + options |
| MCX | Commodity (`MCX_FO`) | Gold, Silver, Crude Oil, Natural Gas |

**Symbol format:** `{EXCHANGE}|{ISIN}` or `{EXCHANGE}|{INSTRUMENT_KEY}` (e.g. `NSE_EQ|INE002A01018`)

## Endpoints

| Base URL | Use |
|----------|-----|
| `https://api.upstox.com/v2` | Standard REST API |
| `https://api-hft.upstox.com/v2` | HFT (low latency) |
| `https://api.upstox.com/v3` | V3 endpoints (expanded features) |

## Rate Limits

| Operation | Per Second | Per Minute | Per 30 Min |
|-----------|-----------|------------|------------|
| Standard APIs | 50 | 500 | 2000 |
| Multi-order APIs | 4 | 40 | 160 |

WebSocket connections: 2 (standard), 5 (Upstox Plus)

## Files

```
upstox/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # OAuth2 token management, from_env loader
├── endpoints.rs       # API URLs (v2, HFT, v3) and endpoint enum
├── connector.rs       # Trait implementations (MarketData, Trading, Account, Positions)
├── parser.rs          # JSON / Protobuf response parsing
└── research/
    ├── api_overview.md
    ├── authentication.md
    ├── endpoints_full.md
    ├── websocket_full.md
    ├── tiers_and_limits.md
    ├── data_types.md
    ├── coverage.md
    └── response_formats.md
```

## Troubleshooting

### 401 Unauthorized

**Cause:** Expired or missing access token

**Solution:**
1. Access tokens expire at 3:30 AM IST daily — re-run OAuth flow
2. Check `UPSTOX_ACCESS_TOKEN` env var is set and not stale
3. Verify `UPSTOX_API_KEY` matches the app that issued the token

### UDAPI100074 — API outside window

**Cause:** Calling API before 5:30 AM or after 12:00 AM IST

**Solution:** Schedule API calls within 5:30 AM - 12:00 AM IST window.

### Market Closed

- **Equity (NSE/BSE):** 9:15 AM - 3:30 PM IST, Mon-Fri
- **F&O (NFO):** 9:15 AM - 3:30 PM IST, Mon-Fri
- **Commodity (MCX):** 9:00 AM - 11:30 PM IST

### Symbol format errors

Upstox uses instrument key format: `NSE_EQ|INE002A01018` (not plain tickers). Look up instrument keys from the Upstox instruments CSV: `https://assets.upstox.com/market-quote/instruments/exchange/complete.csv.gz`

## Documentation

- **Developer Docs:** https://upstox.com/developer/api-documentation/
- **Developer Portal:** https://developer.upstox.com/
- **Instruments list:** https://assets.upstox.com/market-quote/instruments/exchange/complete.csv.gz
- **Official SDKs:** Python, Node.js, Java, PHP, .NET on GitHub

## Testing

```bash
# Integration tests
cargo test --test upstox_integration -- --nocapture

# Specific tests
cargo test --test upstox_integration test_get_price -- --nocapture
cargo test --test upstox_integration test_get_balance -- --nocapture
cargo test --test upstox_integration test_get_klines -- --nocapture

# Unit tests
cargo test --lib upstox
```

## Security

1. **Never commit the access token** - It grants full account access; use `.env` (add to `.gitignore`)
2. **Never commit API secret** - Required to generate new tokens
3. **Rotate API key** - Revoke and regenerate from developer portal if compromised
4. **Restrict redirect URI** - Use a specific URI, not a wildcard, in your app settings

## License

Part of the NEMO trading system.
