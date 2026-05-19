# Fyers Connector - Setup & Usage Guide

Complete implementation of Fyers Securities API v3 for Indian markets.

## Features

- **FREE API Access** - No subscription fees
- **F&O Specialization** - Strong Futures & Options support
- **Multi-Exchange** - NSE, BSE, MCX, NCDEX
- **High Rate Limits** - 100,000 requests/day, 10/sec, 200/min
- **Fast Execution** - Orders execute under 50ms
- **WebSocket Support** - Data, Order, and TBT streams

## Prerequisites

1. Active Fyers trading account
2. Demat account
3. External 2FA TOTP enabled

## Authentication Setup

### Step 1: Create API App

1. Go to https://myapi.fyers.in/dashboard/
2. Click "Create App"
3. Fill in:
   - App Name: `My Trading App`
   - Redirect URI: `https://yourapp.com/callback`
4. Save and note down:
   - `APP_ID` (Client ID): e.g., `ABC123XYZ-100`
   - `APP_SECRET` (Secret Key): e.g., `ABCDEFGH1234567890`

### Step 2: Enable 2FA TOTP

1. Go to https://myaccount.fyers.in/
2. Navigate to "Security" → "External 2FA TOTP"
3. Enable and scan QR code with Google/Microsoft Authenticator
4. **Save the TOTP secret key** for automation

### Step 3: Generate Access Token

#### Option A: Manual (First Time)

```bash
# Set credentials
export FYERS_APP_ID="ABC123XYZ-100"
export FYERS_APP_SECRET="ABCDEFGH1234567890"

# Run the auth helper (create this script)
cargo run --example fyers_auth
```

The script will:
1. Print authorization URL
2. You navigate to it in browser
3. Login with username, password, TOTP code
4. Copy `auth_code` from redirect URL
5. Script exchanges it for `access_token`

#### Option B: Automated (Selenium/Playwright)

Use community tools:
- https://github.com/tkanhe/fyers-api-access-token-v3

### Step 4: Set Environment Variables

```bash
export FYERS_APP_ID="ABC123XYZ-100"
export FYERS_APP_SECRET="ABCDEFGH1234567890"
export FYERS_ACCESS_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
```

Or create `.env` file:
```env
FYERS_APP_ID=ABC123XYZ-100
FYERS_APP_SECRET=ABCDEFGH1234567890
FYERS_ACCESS_TOKEN=eyJ0eXAiOiJKV1QiLCJhbGc...
```

## Usage Examples

### Basic Setup

```rust
use digdigdig3::stocks::india::fyers::{FyersConnector, FyersAuth};
use digdigdig3::{AccountType, Symbol, OrderSide};

// Create connector from environment variables
let connector = FyersConnector::from_env()?;

// Or with explicit credentials
let auth = FyersAuth::with_token("APP_ID", "APP_SECRET", "ACCESS_TOKEN");
let connector = FyersConnector::new(auth)?;
```

### Market Data

```rust
use digdigdig3::MarketData;

let symbol = Symbol::new("SBIN", "NSE");

// Get current price
let price = connector.get_price(symbol.clone(), AccountType::Spot).await?;
println!("SBIN price: {}", price);

// Get ticker (OHLCV + 24h stats)
let ticker = connector.get_ticker(symbol.clone(), AccountType::Spot).await?;
println!("Open: {}, High: {}, Low: {}, Volume: {}",
    ticker.open, ticker.high, ticker.low, ticker.volume);

// Get orderbook (Level 2)
let orderbook = connector.get_orderbook(symbol.clone(), Some(5), AccountType::Spot).await?;
println!("Best bid: {:?}", orderbook.bids.first());
println!("Best ask: {:?}", orderbook.asks.first());

// Get historical klines
let klines = connector.get_klines(symbol, "5m", Some(100), AccountType::Spot).await?;
for kline in klines.iter().take(5) {
    println!("OHLC: {} {} {} {}", kline.open, kline.high, kline.low, kline.close);
}
```

### Trading

```rust
use digdigdig3::Trading;

let symbol = Symbol::new("SBIN", "NSE");

// Place market order
let order = connector.market_order(
    symbol.clone(),
    OrderSide::Buy,
    100.0, // quantity
    AccountType::Spot
).await?;
println!("Order placed: {}", order.id);

// Place limit order
let order = connector.limit_order(
    symbol.clone(),
    OrderSide::Buy,
    100.0, // quantity
    550.0, // limit price
    AccountType::Spot
).await?;

// Cancel order
let canceled = connector.cancel_order(
    symbol.clone(),
    &order.id,
    AccountType::Spot
).await?;

// Get open orders
let orders = connector.get_open_orders(Some(symbol), AccountType::Spot).await?;
for order in orders {
    println!("{}: {} {} @ {:?}", order.id, order.side, order.symbol, order.price);
}
```

### Account & Positions

```rust
use digdigdig3::{Account, Positions};

// Get balance
let balances = connector.get_balance(None, AccountType::Spot).await?;
for balance in balances {
    println!("{}: Total={}, Free={}, Locked={}",
        balance.asset, balance.total, balance.free, balance.locked);
}

// Get account info
let account_info = connector.get_account_info(AccountType::Spot).await?;
println!("User ID: {}", account_info.user_id);

// Get positions
let positions = connector.get_positions(None, AccountType::Spot).await?;
for pos in positions {
    println!("{} {} @ {} (P&L: {})",
        pos.side, pos.symbol, pos.entry_price, pos.unrealized_pnl);
}
```

### Extended Methods

```rust
// Get holdings (delivery portfolio)
let holdings = connector.get_holdings().await?;

// Get trade book
let trades = connector.get_tradebook().await?;

// Convert position
connector.convert_position(
    "NSE:SBIN-EQ",
    1, // position side
    100.0, // quantity
    "INTRADAY", // from product type
    "CNC" // to product type
).await?;

// Modify order
connector.modify_order(
    &order_id,
    Some(1), // order type (LIMIT)
    Some(551.0), // new limit price
    Some(100) // new quantity
).await?;
```

## Symbol Format

Fyers uses: `EXCHANGE:SYMBOL-SERIES`

### Examples

```rust
// Equity (NSE)
Symbol::new("SBIN", "NSE")        // → NSE:SBIN-EQ

// Equity (BSE)
Symbol::new("SENSEX", "BSE")      // → BSE:SENSEX-EQ

// Futures
Symbol::new("NIFTY24JANFUT", "NSE") // → NSE:NIFTY24JANFUT

// Options
Symbol::new("NIFTY2411921500CE", "NSE") // → NSE:NIFTY2411921500CE

// Commodity
Symbol::new("GOLDM24JANFUT", "MCX") // → MCX:GOLDM24JANFUT
```

## Order Types & Product Types

### Order Types

```rust
// type: 1 = LIMIT
// type: 2 = MARKET
// type: 3 = STOP (stop-loss market)
// type: 4 = STOPLIMIT (stop-loss limit)
```

### Product Types

```rust
"INTRADAY" // Intraday square-off
"CNC"      // Cash and Carry (delivery)
"MARGIN"   // Margin (derivatives only)
"CO"       // Cover Order
"BO"       // Bracket Order
```

### Validity

```rust
"DAY" // Valid till end of day
"IOC" // Immediate or Cancel
```

## Testing

### Run All Tests

```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo test --lib stocks::india::fyers::tests -- --nocapture
```

### Run Specific Test

```bash
cargo test --lib stocks::india::fyers::tests::test_get_price -- --nocapture
```

### Run Trading Tests (Real Orders!)

```bash
# These are ignored by default - run explicitly
cargo test --lib stocks::india::fyers::tests::test_limit_order_and_cancel -- --ignored --nocapture
```

## Rate Limits

- **10 requests/second**
- **200 requests/minute**
- **100,000 requests/day**

Response headers:
```
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 195
X-RateLimit-Reset: 1640000000
```

## Error Handling

```rust
use digdigdig3::ExchangeError;

match connector.get_price(symbol, AccountType::Spot).await {
    Ok(price) => println!("Price: {}", price),
    Err(ExchangeError::Auth(msg)) => {
        // Token expired - re-authenticate
        eprintln!("Auth error: {}", msg);
    }
    Err(ExchangeError::RateLimit) => {
        // Rate limit hit - wait and retry
        eprintln!("Rate limit exceeded");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Common Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401/-1600 | Authentication failed | Re-authenticate (token expired) |
| 429 | Rate limit exceeded | Wait and retry |
| -100 | Invalid parameters | Check request format |
| -351 | Symbol limit exceeded | Reduce symbol count (WebSocket) |

## Token Expiry

Access tokens expire after trading day (~24 hours).

**No refresh token mechanism** - must re-authenticate daily.

### Auto Re-authentication

```rust
use std::time::Duration;

async fn with_retry<F, T>(mut f: F) -> Result<T, ExchangeError>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, ExchangeError>>>>,
{
    match f().await {
        Ok(result) => Ok(result),
        Err(ExchangeError::Auth(_)) => {
            // Re-authenticate
            // let new_token = get_new_access_token().await?;
            // connector.auth.set_access_token(new_token);
            // Retry
            f().await
        }
        Err(e) => Err(e),
    }
}
```

## WebSocket Support

(Coming soon - Phase 6)

Three WebSocket types:
1. **Data WebSocket** - Price updates, orderbook, trades
2. **Order WebSocket** - Order/trade/position updates
3. **TBT WebSocket** - Tick-by-tick binary feed (Protobuf)

## Market Coverage

### Exchanges

- **NSE** - National Stock Exchange (equity, F&O, currency)
- **BSE** - Bombay Stock Exchange (equity)
- **MCX** - Multi Commodity Exchange (commodities)
- **NCDEX** - National Commodity & Derivatives Exchange

### Segments

- **CM** - Capital Market (equity)
- **FO** - Futures & Options (derivatives)
- **CD** - Currency Derivatives
- **COMM** - Commodities

## Notes

1. **FREE API** - No monthly subscription for basic access
2. **Trading account required** - Must have active Fyers account
3. **2FA TOTP mandatory** - Required for API access
4. **Daily re-auth needed** - Tokens expire daily
5. **F&O specialization** - Excellent for derivatives trading
6. **No testnet** - Only production environment available
7. **Fast execution** - Orders execute under 50ms
8. **High limits** - 100k requests/day (10x increase in v3)

## Resources

- **API Dashboard**: https://myapi.fyers.in/dashboard/
- **Documentation**: https://myapi.fyers.in/docsv3
- **Community Forum**: https://fyers.in/community/fyers-api-rha0riqv/
- **Status Page**: https://status.fyers.in/
- **GitHub Samples**: https://github.com/FyersDev/fyers-api-sample-code

## Support

- Email: support@fyers.in
- Community: https://fyers.in/community/
- Support Portal: https://support.fyers.in/

## License

This connector implementation is part of the NEMO trading system. See main project license.
