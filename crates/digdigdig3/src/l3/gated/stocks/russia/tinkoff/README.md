# Tinkoff Invest Connector

Russian broker connector for MOEX (Moscow Exchange) via Tinkoff Invest API.

## Status

✅ **READY FOR USE** - Authentication working, market data and trading implemented

## Quick Start

### 1. Get API Token

```
https://www.tinkoff.ru/invest/settings/
→ "Получить токен"
→ Choose token type: Readonly or Full-access
→ Copy the token (starts with "t.")
```

### 2. Set Environment Variable

**Linux/macOS/Git Bash:**
```bash
export TINKOFF_TOKEN="t.your_token_here"
```

**Windows PowerShell:**
```powershell
$env:TINKOFF_TOKEN="t.your_token_here"
```

### 3. Test

```bash
cargo test --test tinkoff_integration -- --nocapture
```

## Usage

```rust
use digdigdig3::stocks::russia::tinkoff::TinkoffConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connector from TINKOFF_TOKEN env var
    let connector = TinkoffConnector::from_env();

    // Or with explicit token
    let connector = TinkoffConnector::new("t.your_token", false);

    // Get last price (requires token)
    let symbol = Symbol::new("SBER", "RUB");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("SBER: {} RUB", price);

    // Get historical candles
    let candles = connector.get_klines(symbol, "1h", Some(100), AccountType::Spot).await?;
    println!("Got {} candles", candles.len());

    // Get account balance (requires full-access token)
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for b in balances {
        println!("{}: {}", b.asset, b.total);
    }

    Ok(())
}
```

### Sandbox Mode

```rust
// Use sandbox environment (no real money)
let connector = TinkoffConnector::new("t.your_sandbox_token", true);
```

## Features

### Market Data
- ✅ Real-time last prices
- ✅ Historical OHLCV candles (5s to 1 month intervals, up to 10 years)
- ✅ Order book (L2, depth 1-50 levels)
- ✅ Trading status per instrument
- ✅ Session closing prices
- ✅ Anonymous trades (last hour)
- ❌ No public data without token (all endpoints require auth)

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ Stop orders
- ✅ Order cancellation
- ✅ Order status tracking
- ✅ Russian stocks (~1,900 MOEX shares)
- ✅ Bonds (~655 Russian bonds, with coupon data)
- ✅ ETFs (~105 ETFs)
- ✅ Futures (~284 contracts)
- ✅ Options (with underlying asset tracking)
- ✅ Currency pairs (~21 pairs)

### Account
- ✅ Balance queries
- ✅ Portfolio (positions and P&L)
- ✅ Account info
- ✅ Multiple account types (standard, IIS, sandbox)
- ✅ Margin account info

### NOT Supported
- ❌ WebSocket streaming (planned, not yet implemented)
- ❌ gRPC (optional feature flag, REST proxy used by default)
- ❌ Orders above 6,000,000 RUB via API
- ❌ Some instruments requiring qualified investor status

## Authentication

**Type:** Bearer Token (no HMAC signing)

**Header:**
```http
Authorization: Bearer t.your_token_here
```

**No signature calculation required.**

### Token Types

| Type | Access |
|------|--------|
| Readonly | Market data + portfolio view |
| Full-access | All operations including trading |
| Account-specific | Restricted to one trading account |
| Sandbox | Testing environment, no real money |

Generate tokens at: https://www.tinkoff.ru/invest/settings/

## Environments

### Production
- REST: `https://invest-public-api.tbank.ru/rest`
- WebSocket: `wss://invest-public-api.tinkoff.ru/ws/`

### Sandbox
- REST: `https://sandbox-invest-public-api.tinkoff.ru/rest`
- WebSocket: `wss://sandbox-invest-public-api.tinkoff.ru/ws/`

**Use sandbox for testing** — no real money, separate token required.

## API Notes

- Primary protocol is gRPC; this connector uses the REST proxy (JSON over HTTPS)
- Prices use `Quotation` type: `{units: int64, nano: int32}` (9 decimal places)
- Money uses `MoneyValue`: `{currency, units, nano}`
- Timestamps in ISO 8601 UTC
- Symbol identification uses Tinkoff FIGI (not ticker directly)

## Files

```
tinkoff/
├── README.md          # This file
├── mod.rs             # Module exports
├── auth.rs            # Bearer token authentication
├── endpoints.rs       # API endpoint enum and URLs
├── connector.rs       # Trait implementations
├── parser.rs          # JSON response parsing
├── proto.rs           # gRPC protobuf types (feature = "grpc")
├── tests.rs           # Unit tests
└── research/          # API research notes
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TINKOFF_TOKEN` | Yes | Bearer token (starts with `t.`) |

## Testing

```bash
# Integration tests (requires TINKOFF_TOKEN)
cargo test --test tinkoff_integration -- --nocapture

# Unit tests
cargo test --lib tinkoff

# Sandbox tests (set token to sandbox token first)
TINKOFF_TOKEN=t.sandbox_token cargo test --test tinkoff_integration -- --nocapture
```

## Rate Limits

- No hard published limits for the REST proxy
- Recommended: avoid bursts, use reasonable polling intervals
- Order placement has a maximum of 6,000,000 RUB per order via API

## Documentation

- **Official API:** https://tinkoff.github.io/investAPI/
- **Proto contracts:** https://github.com/Tinkoff/investAPI
- **Settings (token generation):** https://www.tinkoff.ru/invest/settings/

## License

Part of the NEMO trading system.
