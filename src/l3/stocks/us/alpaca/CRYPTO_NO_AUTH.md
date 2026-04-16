# Alpaca Crypto Without Authentication

## Overview

The Alpaca connector now supports accessing crypto market data **without API keys**.

All crypto endpoints on `data.alpaca.markets` work without authentication, allowing you to:
- Get real-time crypto prices (BTC/USD, ETH/USD, etc.)
- Fetch crypto ticker data
- Retrieve crypto OHLCV (klines/bars)
- Access crypto orderbook data

## Usage

```rust
use digdigdig3::stocks::us::alpaca::AlpacaConnector;
use digdigdig3::core::types::{Symbol, AccountType};
use digdigdig3::core::traits::MarketData;

// Create crypto-only connector (no API keys required)
let connector = AlpacaConnector::crypto_only();

// Get BTC/USD price
let btc = Symbol::new("BTC", "USD");
let price = connector.get_price(btc, AccountType::Spot).await?;
println!("BTC/USD: ${}", price);
```

## Verification

All crypto endpoints confirmed working without authentication via curl:

```bash
# BTC/USD snapshot
curl "https://data.alpaca.markets/v1beta3/crypto/us/snapshots?symbols=BTC/USD"

# ETH/USD latest trades
curl "https://data.alpaca.markets/v1beta3/crypto/us/latest/trades?symbols=ETH/USD"

# BTC/USD hourly bars
curl "https://data.alpaca.markets/v1beta3/crypto/us/bars?symbols=BTC/USD&timeframe=1Hour&limit=24"

# BTC/USD orderbook
curl "https://data.alpaca.markets/v1beta3/crypto/us/latest/orderbooks?symbols=BTC/USD"
```

All return data successfully without any authentication headers.

## Implementation Details

### Constructor
Added `AlpacaConnector::crypto_only()` which:
- Creates connector with no auth credentials
- Uses live environment (not paper trading)
- Works only for crypto symbols (symbols containing `/`)

### Changes to Methods

1. **`ping()`**
   - Without auth: Uses crypto snapshot endpoint (`/v1beta3/crypto/us/snapshots?symbols=BTC/USD`)
   - With auth: Uses clock endpoint (`/v2/clock`) as before

2. **`get_market_data()`**
   - Skips auth headers when `auth.has_credentials()` returns false
   - Skips feed parameter for crypto endpoints (only needed for stocks)

3. **`get_price()` / `get_ticker()` / `get_klines()`**
   - Detect crypto symbols by presence of `/` in symbol string
   - Route to crypto endpoints (`CryptoSnapshots`, `CryptoBars`) vs stock endpoints
   - Handle crypto response format with `snapshots` wrapper

### Factory Integration

Updated `ConnectorFactory::create_public()`:
```rust
ExchangeId::Alpaca => {
    // Create crypto-only connector (works without API keys)
    let c = AlpacaConnector::crypto_only();
    Ok(Arc::new(AnyConnector::Alpaca(Arc::new(c))))
}
```

## Limitations

### Works Without Auth:
- Crypto market data (BTC/USD, ETH/USD, etc.)
- Crypto snapshots, trades, quotes, bars
- Crypto orderbook

### Requires Auth:
- Stock market data (AAPL, TSLA, etc.)
- Trading operations (buy, sell, cancel)
- Account operations (balance, positions)
- Account-specific endpoints (`/v2/account`, `/v2/orders`, etc.)

## Example

Run the example:
```bash
cd zengeld-terminal/crates/connectors/crates/v5
cargo run --example alpaca_crypto_no_auth
```

This will test all crypto endpoints without requiring any environment variables or API keys.

## Why This Matters

1. **No signup required** - Can fetch crypto data immediately
2. **Testing** - Easy to test connector functionality without credentials
3. **Public data** - Useful for market data aggregation, charting, etc.
4. **Factory pattern** - Works seamlessly with `ConnectorFactory::create_public()`

## Testing

The connector maintains full backward compatibility:
- With API keys: Stock + crypto data + trading (unchanged)
- Without API keys: Crypto data only (new functionality)

All existing tests pass unchanged. Stock functionality is unaffected.
