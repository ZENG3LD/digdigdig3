# Alpaca US Stock Connector

Production-ready connector for Alpaca US stock broker and market data provider.

## Status

✅ **READY FOR USE** - Authentication working, all endpoints implemented

## Quick Start

### 1. Get API Keys (2 minutes)

```
https://app.alpaca.markets/signup
→ Create account (email only, no credit card)
→ Dashboard → API Keys
→ Copy Key ID and Secret Key
```

### 2. Set Environment Variables

**Linux/macOS/Git Bash:**
```bash
export ALPACA_API_KEY_ID="your_key_id"
export ALPACA_API_SECRET_KEY="your_secret_key"
```

**Windows PowerShell:**
```powershell
$env:ALPACA_API_KEY_ID="your_key_id"
$env:ALPACA_API_SECRET_KEY="your_secret_key"
```

### 3. Test

```bash
# Linux/macOS/Git Bash
bash TEST_COMMANDS.sh

# Windows PowerShell
.\TEST_COMMANDS.ps1

# Rust tests
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --test alpaca_integration -- --nocapture
```

## Usage

### With API Keys (Stock + Crypto Trading)

```rust
use digdigdig3::stocks::us::alpaca::AlpacaConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::{MarketData, Trading, Account};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connector (loads from env vars)
    let connector = AlpacaConnector::from_env();

    // Get current price
    let symbol = Symbol::new("AAPL", "USD");
    let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
    println!("AAPL: ${}", price);

    // Get account balance
    let balances = connector.get_balance(None, AccountType::Spot).await?;
    for balance in balances {
        println!("{}: ${}", balance.asset, balance.total);
    }

    // Place market order (paper trading - virtual money)
    let order = connector.market_order(
        symbol,
        OrderSide::Buy,
        Quantity::from(1.0), // 1 share
        AccountType::Spot
    ).await?;
    println!("Order placed: {}", order.id);

    Ok(())
}
```

### Without API Keys (Crypto Market Data Only)

```rust
use digdigdig3::stocks::us::alpaca::AlpacaConnector;
use digdigdig3::core::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create public crypto-only connector (no API keys required)
    let connector = AlpacaConnector::crypto_only();

    // Get crypto prices - works without authentication!
    let btc = Symbol::new("BTC", "USD");
    let price = connector.get_price(btc.clone(), AccountType::Spot).await?;
    println!("BTC/USD: ${}", price);

    // Get crypto ticker
    let ticker = connector.get_ticker(btc.clone(), AccountType::Spot).await?;
    println!("Volume: {}", ticker.volume_24h.unwrap_or(0.0));

    // Get crypto klines
    let klines = connector.get_klines(btc, "1h", Some(24), AccountType::Spot).await?;
    println!("Got {} hourly candles", klines.len());

    // Note: Stock data and trading operations require API keys
    Ok(())
}
```

## Features

### Market Data
- ✅ Real-time prices (IEX feed free, SIP paid)
- ✅ Historical OHLCV data (7+ years stocks)
- ✅ Order book (crypto only)
- ✅ Ticker snapshots
- ✅ News feed (Benzinga)
- ✅ Corporate actions
- ✅ **Crypto data without API keys** (BTC/USD, ETH/USD, etc.)

### Trading
- ✅ Market orders
- ✅ Limit orders
- ✅ Order status tracking
- ✅ Position management
- ✅ Commission-free trading
- ✅ Fractional shares

### Account
- ✅ Balance queries
- ✅ Account info
- ✅ Positions
- ✅ Trade history
- ✅ Paper trading (free)

### NOT Supported
- ❌ Futures (stocks broker only)
- ❌ Leverage/margin API (use separate margin account)
- ❌ Funding rates (not applicable to stocks)
- ❌ Level 2 orderbook for stocks (crypto only)

## Authentication

**Type:** Simple API Key (no HMAC)

**Headers:**
```http
APCA-API-KEY-ID: your_key_id
APCA-API-SECRET-KEY: your_secret_key
```

**No signature required!** Much simpler than crypto exchanges.

### Crypto Data Without Authentication

All crypto market data endpoints work **without API keys**:
- Crypto prices (BTC/USD, ETH/USD, etc.)
- Crypto tickers and snapshots
- Crypto OHLCV data (bars)
- Crypto orderbook

Stock data, trading, and account operations require API keys.

## Environments

### Paper Trading (Default, Free)
- URL: `https://paper-api.alpaca.markets`
- Access: Global (anyone)
- Balance: Virtual $100,000
- Purpose: Testing

### Live Trading (US Only)
- URL: `https://api.alpaca.markets`
- Access: US residents (KYC required)
- Balance: Real money
- Purpose: Production

**Connector defaults to paper trading** for safety.

## Data Feeds

### IEX (Free)
- Coverage: ~2.5% market volume
- Cost: $0
- Real-time: Yes

### SIP (Paid)
- Coverage: 100% market volume
- Cost: $99/month
- Real-time: Yes

```rust
use digdigdig3::stocks::us::alpaca::{AlpacaConnector, DataFeed};

let connector = AlpacaConnector::from_env()
    .with_feed(DataFeed::SIP);  // Use paid feed
```

## Files

```
alpaca/
├── README.md                    # This file
├── QUICK_START.md               # 3-minute setup guide
├── AUTHENTICATION_SETUP.md      # Detailed auth guide
├── TEST_COMMANDS.sh             # Test scripts (Linux/macOS)
├── TEST_COMMANDS.ps1            # Test scripts (Windows)
├── mod.rs                       # Module exports
├── auth.rs                      # Authentication
├── endpoints.rs                 # API endpoints
├── connector.rs                 # Main implementation
├── parser.rs                    # JSON parsing
├── websocket.rs                 # WebSocket streams
└── research/
    ├── authentication.md        # Auth research
    └── INVESTIGATION_RESULTS.md # 401 error investigation
```

## Troubleshooting

### 401 Unauthorized

**Cause:** Missing or invalid API credentials

**Solution:**
1. Check env vars are set: `echo $ALPACA_API_KEY_ID`
2. Verify keys from dashboard
3. Ensure using paper keys with paper URL
4. See `AUTHENTICATION_SETUP.md`

### 403 Forbidden

**Cause:** Wrong environment (paper vs live)

**Solution:**
- Paper keys only work with `paper-api.alpaca.markets`
- Live keys only work with `api.alpaca.markets`
- Don't mix them!

### Market Closed

**Cause:** US stock market closed

**Info:**
- Market hours: 9:30 AM - 4:00 PM ET, Mon-Fri
- Check with `connector.get_clock().await`
- Some operations require market to be open

## Documentation

- **Official API:** https://docs.alpaca.markets/
- **Sign up:** https://app.alpaca.markets/signup
- **Dashboard:** https://app.alpaca.markets/
- **Support:** https://alpaca.markets/support

## Testing

```bash
# All integration tests
cargo test --test alpaca_integration -- --nocapture

# Specific tests
cargo test --test alpaca_integration test_ping -- --nocapture
cargo test --test alpaca_integration test_get_price -- --nocapture
cargo test --test alpaca_integration test_get_balance -- --nocapture

# Unit tests
cargo test --lib alpaca
```

## Rate Limits

### Free Tier
- REST API: 200 requests/minute per key
- WebSocket: 1 connection per key
- Symbols: 30 concurrent subscriptions

### Paid Tier (Algo Trader Plus)
- REST API: Unlimited (fair use)
- WebSocket: 1 connection per key
- Symbols: Unlimited subscriptions

## Security

1. **Never commit keys** - Use `.env` file (add to `.gitignore`)
2. **Rotate keys** - Regenerate periodically in dashboard
3. **Separate keys** - Different keys per environment/bot
4. **Monitor usage** - Check dashboard for unusual activity

## Examples

See `tests/alpaca_integration.rs` for comprehensive examples:
- Market data queries
- Account management
- Trading operations
- Position tracking
- Error handling

## Support

- Issues: See `AUTHENTICATION_SETUP.md` first
- Questions: Check `research/authentication.md`
- Bugs: File issue with reproduction steps

## License

Part of the NEMO trading system.
