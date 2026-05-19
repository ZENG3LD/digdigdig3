# Zerodha Kite Connect Connector

Connector for Zerodha Indian stock broker via the Kite Connect API.

## Status

✅ **READY FOR USE** - Authentication implemented, all endpoints available

## Quick Start

### 1. Get API Keys

```
https://developers.kite.trade/
→ Create account and log in
→ Create an app to get api_key and api_secret
→ Note: Kite Connect API costs ₹500/month (full access)
```

### 2. Get an Access Token

Zerodha uses a daily OAuth-like flow:

```
1. Direct user to: https://kite.zerodha.com/connect/login?v=3&api_key={api_key}
2. After login, receive request_token via redirect callback
3. Compute checksum: SHA256(api_key + request_token + api_secret)
4. POST /session/token to exchange for access_token
5. Store access_token — it expires daily at 6:00 AM IST
```

### 3. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export ZERODHA_API_KEY="your_api_key"
export ZERODHA_API_SECRET="your_api_secret"
export ZERODHA_ACCESS_TOKEN="your_access_token"
```

**Windows PowerShell:**
```powershell
$env:ZERODHA_API_KEY="your_api_key"
$env:ZERODHA_API_SECRET="your_api_secret"
$env:ZERODHA_ACCESS_TOKEN="your_access_token"
```

### 4. Test

```bash
cargo test --test zerodha_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::india::zerodha::ZerodhaConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = ZerodhaConnector::from_env();

    // Get last traded price
    let symbol = Symbol::new("RELIANCE", "INR");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("RELIANCE: ₹{}", price);

    // Get account margins
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for balance in balances {
        println!("{}: ₹{}", balance.asset, balance.total);
    }

    // Place a market order
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        Quantity::from(1.0),
        AccountType::Spot,
    ).await?;
    println!("Order: {}", order.id);

    Ok(())
}
```

## Features

### Market Data
- ✅ Last traded price (LTP)
- ✅ OHLC quotes
- ✅ Full market depth (5-level order book)
- ✅ Historical OHLCV candles (minute to daily)
- ✅ Instrument master list (NSE, BSE, MCX, etc.)

### Trading
- ✅ Market, limit, SL, SL-M orders
- ✅ Regular, CNC, MIS, NRML products
- ✅ AMO (After Market Orders)
- ✅ GTT (Good Till Triggered) orders
- ✅ Iceberg orders
- ✅ Order modification and cancellation

### Account
- ✅ Margins (equity + commodity)
- ✅ Holdings (long-term positions)
- ✅ Day positions
- ✅ Trade history
- ✅ User profile

### NOT Supported
- ❌ Mutual funds API (separate Coin API)
- ❌ IPO subscriptions
- ❌ WebSocket in this build (feature-gated)

## Authentication

**Type:** Custom OAuth-like flow with SHA-256 checksum

**Authorization header format:**
```http
Authorization: token {api_key}:{access_token}
```

Note: Uses `token` scheme, NOT `Bearer`.

**Token lifetime:** Expires daily at 6:00 AM IST. No refresh token — must re-authenticate each day.

**Checksum computation:**
```rust
SHA256(api_key + request_token + api_secret)
```

## Supported Exchanges

| Exchange | Description |
|----------|-------------|
| NSE | National Stock Exchange — equities |
| BSE | Bombay Stock Exchange — equities |
| NFO | NSE Futures & Options |
| BFO | BSE Futures & Options |
| MCX | Multi Commodity Exchange |
| CDS | NSE Currency Derivatives |
| BCD | BSE Currency Derivatives |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ZERODHA_API_KEY` | Yes | App API key from developers.kite.trade |
| `ZERODHA_API_SECRET` | Yes | App API secret |
| `ZERODHA_ACCESS_TOKEN` | Yes | Daily access token (re-generate each morning) |

## Files

```
zerodha/
├── README.md           # This file
├── IMPLEMENTATION_BRIEF.md  # Implementation notes
├── mod.rs              # Module exports
├── auth.rs             # SHA-256 checksum auth, token management
├── endpoints.rs        # API endpoint enum, URL construction
├── connector.rs        # Trait implementations
├── parser.rs           # JSON response parsing
└── research/           # Kite Connect API research notes
```

## Testing

```bash
# Integration tests (requires ZERODHA_* env vars)
cargo test --test zerodha_integration -- --nocapture

# Specific tests
cargo test --test zerodha_integration test_ping -- --nocapture
cargo test --test zerodha_integration test_get_price -- --nocapture
cargo test --test zerodha_integration test_get_balance -- --nocapture

# Unit tests
cargo test --lib zerodha
```

## Rate Limits

- REST API: 10 requests/second (documented limit)
- Historical data: 3 requests/second
- WebSocket: 1 connection, up to 3000 instrument tokens

## API Costs

| Plan | Price | Features |
|------|-------|----------|
| Personal API (free tier) | ₹0 | Basic quotes, no WebSocket, no historical |
| Kite Connect | ₹500/month | Full access — WebSocket, historical, trading |

## Documentation

- **Official API:** https://kite.trade/docs/connect/v3/
- **Developer portal:** https://developers.kite.trade/
- **API reference:** https://kite.trade/docs/connect/v3/restapi/
