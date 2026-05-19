# OANDA v20 Connector

Forex broker connector for OANDA v20 REST API. Supports trading and market data for 120+ forex pairs, metals, commodities, and indices.

## Status

✅ **READY FOR USE** - Authentication working, trading and market data implemented

## Quick Start

### 1. Get API Token

```
https://www.oanda.com/account/tpa/personal_token
→ Log in to your account
→ Generate a new API token
→ Copy the token
```

For a practice (demo) account: https://fxtrade.oanda.com/your_account/fxtrade/register/gate

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export OANDA_API_TOKEN="your_bearer_token"
export OANDA_ACCOUNT_ID="your_account_id"  # e.g. 001-001-1234567-001
```

**Windows PowerShell:**
```powershell
$env:OANDA_API_TOKEN="your_bearer_token"
$env:OANDA_ACCOUNT_ID="your_account_id"
```

### 3. Test

```bash
cargo test --test oanda_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::forex::oanda::OandaConnector;
use digdigdig3::core::{Credentials, Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create practice connector
    let credentials = Credentials::new("your_bearer_token", "");
    let connector = OandaConnector::new(credentials, true).await?;

    // Get EUR/USD price
    let symbol = Symbol::new("EUR", "USD");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("EUR/USD: {}", price);

    // Get historical candles
    let candles = connector.get_klines(symbol.clone(), "1h", Some(48), AccountType::Spot).await?;
    println!("Got {} candles", candles.len());

    // Place market order (10,000 units)
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        10000.0,
        AccountType::Spot
    ).await?;
    println!("Order placed: {}", order.id);

    Ok(())
}
```

### Live Account

```rust
// Switch to live account (real money)
let connector = OandaConnector::new(credentials, false).await?;
```

## Features

### Market Data
- ✅ Real-time pricing (bid/ask/mid)
- ✅ Historical OHLCV candles (5s to monthly)
- ✅ Order book snapshots
- ✅ Position book snapshots
- ✅ 120+ forex pairs (EUR_USD, GBP_JPY, etc.)
- ✅ Metals (XAU_USD gold, XAG_USD silver)
- ✅ Commodities and indices
- ✅ HTTP streaming for real-time prices

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ Stop orders
- ✅ Take-profit and stop-loss on orders
- ✅ Order modification
- ✅ Order cancellation
- ✅ Position close (full or partial)

### Account
- ✅ Account balance and NAV
- ✅ Open positions
- ✅ Open trades
- ✅ Transaction history
- ✅ P&L tracking
- ✅ Practice and live accounts

### Streaming
- ✅ Pricing stream (HTTP chunked transfer)
- ✅ Transaction stream

### NOT Supported
- ❌ WebSocket (OANDA uses HTTP streaming instead)
- ❌ Crypto trading (forex broker only)
- ❌ Stock trading
- ❌ Futures

## Authentication

**Type:** Bearer Token (no HMAC signing)

**Headers:**
```http
Authorization: Bearer your_token_here
Content-Type: application/json
```

**No signature calculation required.**

Token is passed in `api_key` field of `Credentials`.

## Environments

### Practice (Demo)
- REST: `https://api-fxpractice.oanda.com`
- Stream: `https://stream-fxpractice.oanda.com`
- Access: Global, free demo account
- Balance: Virtual funds

### Live
- REST: `https://api-fxtrade.oanda.com`
- Stream: `https://stream-fxtrade.oanda.com`
- Access: Funded live account required
- Balance: Real money

**Connector defaults to practice mode** for safety. Pass `false` to `OandaConnector::new` for live.

## Symbol Format

OANDA uses underscore-separated pairs:

| Universal | OANDA |
|-----------|-------|
| EUR/USD | EUR_USD |
| GBP/JPY | GBP_JPY |
| XAU/USD | XAU_USD |

The connector handles conversion automatically.

## Files

```
oanda/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # Bearer token authentication
├── endpoints.rs       # API endpoint enum and URL constants
├── connector.rs       # Trait implementations
├── parser.rs          # JSON response parsing
├── streaming.rs       # HTTP pricing and transaction streams
└── research/          # API research notes
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `OANDA_API_TOKEN` | Yes | Bearer token from OANDA account settings |
| `OANDA_ACCOUNT_ID` | Yes | Account ID (format: `001-001-XXXXXXX-001`) |

## Testing

```bash
# Integration tests (requires env vars)
cargo test --test oanda_integration -- --nocapture

# Specific tests
cargo test --test oanda_integration test_get_price -- --nocapture
cargo test --test oanda_integration test_get_balance -- --nocapture

# Unit tests
cargo test --lib oanda
```

## Rate Limits

- REST API: 100 requests/second per token
- Streaming: 20 concurrent streaming connections per account

## Documentation

- **Official API:** https://developer.oanda.com/rest-live-v20/introduction/
- **Sign up (practice):** https://fxtrade.oanda.com/your_account/fxtrade/register/gate
- **Token generation:** https://www.oanda.com/account/tpa/personal_token
- **Account IDs:** Available in OANDA fxTrade platform under account details

## License

Part of the NEMO trading system.
