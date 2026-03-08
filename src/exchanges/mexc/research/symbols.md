# MEXC Symbol Format and Naming Conventions

## Symbol Format Differences

MEXC uses different symbol formats depending on the market type:

### Spot Trading
**Format**: Concatenated without separator
- Base asset + Quote asset (no separator)
- All uppercase letters
- **Examples**:
  - `BTCUSDT` (Bitcoin/USDT)
  - `ETHUSDT` (Ethereum/USDT)
  - `MXUSDT` (MX Token/USDT)
  - `BNBBTC` (Binance Coin/Bitcoin)

### Futures Trading (Perpetual Contracts)
**Format**: Underscore separator
- Base asset + `_` + Quote asset
- All uppercase letters
- **Examples**:
  - `BTC_USDT` (Bitcoin/USDT perpetual)
  - `ETH_USDT` (Ethereum/USDT perpetual)
  - `DOGE_USDT` (Dogecoin/USDT perpetual)

---

## Symbol Structure

### Components

A symbol consists of two parts:

1. **Base Asset**: The asset being traded (e.g., BTC, ETH)
2. **Quote Asset**: The asset used for pricing (e.g., USDT, BTC)

### Naming Pattern

```
Spot:    [BASE][QUOTE]
Futures: [BASE]_[QUOTE]
```

---

## Getting Available Symbols

### Default Symbols Endpoint

```http
GET https://api.mexc.com/api/v3/defaultSymbols
```

**Response**: Array of default trading pair symbols
```json
[
  "BTCUSDT",
  "ETHUSDT",
  "MXUSDT",
  "BNBUSDT"
]
```

### Exchange Info Endpoint

```http
GET https://api.mexc.com/api/v3/exchangeInfo
```

**Response**: Comprehensive symbol information including:
- Trading status
- Precision settings
- Trading rules and filters
- Commission rates
- Supported order types

**Example Response**:
```json
{
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "status": "ENABLED",
      "baseAsset": "BTC",
      "baseAssetPrecision": 8,
      "quoteAsset": "USDT",
      "quotePrecision": 8,
      "quoteAssetPrecision": 8,
      "baseCommissionPrecision": 8,
      "quoteCommissionPrecision": 8,
      "orderTypes": ["LIMIT", "MARKET", "LIMIT_MAKER"],
      "quoteOrderQtyMarketAllowed": true,
      "isSpotTradingAllowed": true,
      "isMarginTradingAllowed": false,
      "quoteAmountPrecision": "8",
      "baseSizePrecision": "0.00001",
      "permissions": ["SPOT"]
    }
  ]
}
```

---

## Symbol Information Fields

### Basic Information

| Field | Description | Example |
|-------|-------------|---------|
| `symbol` | Trading pair identifier | "BTCUSDT" |
| `status` | Trading status | "ENABLED" |
| `baseAsset` | Base asset name | "BTC" |
| `quoteAsset` | Quote asset name | "USDT" |
| `permissions` | Allowed trading types | ["SPOT"] |

### Precision Settings

| Field | Description | Example |
|-------|-------------|---------|
| `baseAssetPrecision` | Base asset decimal places | 8 |
| `quoteAssetPrecision` | Quote asset decimal places | 8 |
| `quotePrecision` | Quote precision | 8 |
| `baseCommissionPrecision` | Commission precision for base | 8 |
| `quoteCommissionPrecision` | Commission precision for quote | 8 |
| `quoteAmountPrecision` | Quote amount precision string | "8" |
| `baseSizePrecision` | Base size precision string | "0.00001" |

### Trading Configuration

| Field | Description | Values |
|-------|-------------|--------|
| `orderTypes` | Supported order types | ["LIMIT", "MARKET", "LIMIT_MAKER"] |
| `quoteOrderQtyMarketAllowed` | Market orders by quote qty allowed | true/false |
| `isSpotTradingAllowed` | Spot trading enabled | true/false |
| `isMarginTradingAllowed` | Margin trading enabled | true/false |

### Limits and Fees

| Field | Description | Example |
|-------|-------------|---------|
| `maxQuoteAmount` | Maximum quote amount | "5000000" |
| `makerCommission` | Maker fee rate | "0.002" |
| `takerCommission` | Taker fee rate | "0.002" |

---

## Symbol Filters

Filters define trading rules and constraints for each symbol.

### PERCENT_PRICE_BY_SIDE

Defines price limits as percentage of current price:

```json
{
  "filterType": "PERCENT_PRICE_BY_SIDE",
  "bidMultiplierUp": "5",
  "bidMultiplierDown": "0.2",
  "askMultiplierUp": "5",
  "askMultiplierDown": "0.2"
}
```

- Buy orders: price between 20% and 500% of current
- Sell orders: price between 20% and 500% of current

### LOT_SIZE

Defines quantity constraints:

```json
{
  "filterType": "LOT_SIZE",
  "minQty": "0.00001",
  "maxQty": "9000000",
  "stepSize": "0.00001"
}
```

- `minQty`: Minimum order quantity
- `maxQty`: Maximum order quantity
- `stepSize`: Quantity increment step

### MIN_NOTIONAL

Defines minimum order value:

```json
{
  "filterType": "MIN_NOTIONAL",
  "minNotional": "5"
}
```

Order value (price × quantity) must be at least this amount.

---

## Symbol Validation Rules

### Case Sensitivity

- **All symbol names must be UPPERCASE**
- `BTCUSDT` ✓ Correct
- `btcusdt` ✗ Incorrect
- `BtcUsdt` ✗ Incorrect

### Format Consistency

**Spot API:**
```
Correct:   BTCUSDT
Incorrect: BTC_USDT
Incorrect: BTC-USDT
Incorrect: BTC/USDT
```

**Futures API:**
```
Correct:   BTC_USDT
Incorrect: BTCUSDT
Incorrect: BTC-USDT
Incorrect: BTC/USDT
```

### Symbol Existence

Always verify symbol exists and is tradable:
- Check `status` field is "ENABLED"
- Verify `isSpotTradingAllowed` for spot
- Check `permissions` array includes desired trading type

---

## Common Quote Assets

### Stablecoins
- `USDT`: Tether (most common)
- `USDC`: USD Coin
- `BUSD`: Binance USD

### Cryptocurrencies
- `BTC`: Bitcoin
- `ETH`: Ethereum
- `MX`: MX Token (MEXC native token)

### Fiat
- `USD`: US Dollar (limited availability)

---

## Converting Between Formats

### Spot to Futures Format

```rust
fn spot_to_futures(spot_symbol: &str) -> String {
    // Example: "BTCUSDT" -> "BTC_USDT"
    // This is a simplified example - actual implementation
    // should use exchange info to identify base/quote assets

    // For USDT pairs (most common)
    if spot_symbol.ends_with("USDT") {
        let base = &spot_symbol[..spot_symbol.len() - 4];
        return format!("{}_USDT", base);
    }

    // Handle other cases using exchangeInfo
    spot_symbol.to_string()
}
```

### Futures to Spot Format

```rust
fn futures_to_spot(futures_symbol: &str) -> String {
    // Example: "BTC_USDT" -> "BTCUSDT"
    futures_symbol.replace("_", "")
}
```

**Note**: Always validate the converted symbol exists using `/api/v3/exchangeInfo`.

---

## Parsing Symbol Information

### Extracting Base and Quote Assets

**From API Response:**
```rust
struct SymbolInfo {
    symbol: String,
    base_asset: String,
    quote_asset: String,
}

// Use exchangeInfo response
let info = SymbolInfo {
    symbol: "BTCUSDT".to_string(),
    base_asset: "BTC".to_string(),  // From baseAsset field
    quote_asset: "USDT".to_string(), // From quoteAsset field
};
```

**Manual Parsing (Not Recommended):**
```rust
// For spot symbols ending in common quotes
fn parse_spot_symbol(symbol: &str) -> Option<(String, String)> {
    let common_quotes = ["USDT", "USDC", "BUSD", "BTC", "ETH"];

    for quote in &common_quotes {
        if symbol.ends_with(quote) {
            let base = &symbol[..symbol.len() - quote.len()];
            return Some((base.to_string(), quote.to_string()));
        }
    }
    None
}
```

**Warning**: Manual parsing is unreliable. Always use `exchangeInfo` endpoint for accurate base/quote asset identification.

---

## Symbol Status

### Status Values

| Status | Description | Tradable |
|--------|-------------|----------|
| `ENABLED` | Normal trading | Yes |
| `DISABLED` | Trading suspended | No |
| `BREAK` | Circuit breaker active | No |

### Checking Trading Availability

Before placing orders, verify:

1. **Symbol exists**: Present in exchangeInfo
2. **Status is ENABLED**: `"status": "ENABLED"`
3. **Trading allowed**: `"isSpotTradingAllowed": true`
4. **Correct permissions**: `"permissions": ["SPOT"]`

```rust
fn is_tradable(symbol_info: &SymbolInfo) -> bool {
    symbol_info.status == "ENABLED"
        && symbol_info.is_spot_trading_allowed
        && symbol_info.permissions.contains(&"SPOT".to_string())
}
```

---

## Symbol Metadata Caching

### Cache Strategy

Recommended approach:
1. Fetch `exchangeInfo` on startup
2. Cache symbol information in memory
3. Refresh periodically (every 24 hours)
4. Update on reconnection or error

### Cache Structure

```rust
use std::collections::HashMap;

struct SymbolCache {
    symbols: HashMap<String, SymbolInfo>,
    last_update: u64,
}

impl SymbolCache {
    fn should_refresh(&self, max_age_seconds: u64) -> bool {
        let now = current_timestamp_seconds();
        now - self.last_update > max_age_seconds
    }

    fn get(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.get(symbol)
    }
}
```

---

## WebSocket Symbol Format

### Market Streams

WebSocket subscriptions use the same format as REST API:

**Spot Streams:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.deals.v3.api@BTCUSDT",
    "spot@public.kline.v3.api.pb@ETHUSDT@Min15"
  ]
}
```

**Futures Streams:**
```json
{
  "method": "sub.deal",
  "param": {
    "symbol": "BTC_USDT"
  }
}
```

### Important Notes

- All symbol names in WebSocket streams **must be UPPERCASE**
- Use correct format for market type (spot vs futures)
- Invalid symbols will be rejected

---

## Special Cases

### MX Token

MEXC's native token has special handling:

**Symbol**: `MXUSDT` (spot) or `MX_USDT` (futures)
**Features**:
- Can be used for fee deduction
- May have different commission rates
- Check MX deduction status via `/api/v3/mxDeduct/enable`

### Multi-Asset Symbols

Some symbols may have multiple quote assets:

- `BTCUSDT` - Bitcoin quoted in USDT
- `BTCUSDC` - Bitcoin quoted in USDC
- `BTCBUSD` - Bitcoin quoted in BUSD
- `ETHBTC` - Ethereum quoted in Bitcoin

Always specify the complete symbol with desired quote asset.

---

## Best Practices

### 1. Always Use exchangeInfo

```rust
// Good: Get base/quote from API
let response = client.get_exchange_info("BTCUSDT").await?;
let base = response.base_asset;
let quote = response.quote_asset;

// Bad: Manual parsing
let base = "BTC"; // Hardcoded - may be wrong
let quote = "USDT";
```

### 2. Validate Before Trading

```rust
fn validate_symbol(symbol: &str, info: &SymbolInfo) -> Result<(), Error> {
    if info.status != "ENABLED" {
        return Err(Error::SymbolDisabled);
    }
    if !info.is_spot_trading_allowed {
        return Err(Error::TradingNotAllowed);
    }
    Ok(())
}
```

### 3. Handle Format Differences

```rust
enum Market {
    Spot,
    Futures,
}

fn format_symbol(base: &str, quote: &str, market: Market) -> String {
    match market {
        Market::Spot => format!("{}{}", base, quote),
        Market::Futures => format!("{}_{}", base, quote),
    }
}
```

### 4. Cache Efficiently

- Don't fetch exchangeInfo on every request
- Refresh cache periodically
- Handle API errors gracefully
- Validate cached data age

---

## Examples

### Complete Symbol Validation

```rust
async fn validate_and_get_symbol_info(
    client: &MexcClient,
    symbol: &str,
) -> Result<SymbolInfo, Error> {
    // Fetch exchange info
    let exchange_info = client.get_exchange_info(None).await?;

    // Find symbol
    let symbol_info = exchange_info
        .symbols
        .iter()
        .find(|s| s.symbol == symbol)
        .ok_or(Error::SymbolNotFound)?;

    // Validate status
    if symbol_info.status != "ENABLED" {
        return Err(Error::SymbolDisabled);
    }

    // Validate trading allowed
    if !symbol_info.is_spot_trading_allowed {
        return Err(Error::TradingNotAllowed);
    }

    Ok(symbol_info.clone())
}
```

### Symbol Format Conversion

```rust
fn normalize_symbol(symbol: &str, target_market: Market) -> String {
    // Remove separator if present
    let clean = symbol.replace("_", "").replace("-", "").replace("/", "");

    // Convert to uppercase
    let upper = clean.to_uppercase();

    // Apply target format
    match target_market {
        Market::Spot => upper,
        Market::Futures => {
            // This requires knowing base/quote split
            // Use exchangeInfo in real implementation
            if upper.ends_with("USDT") {
                let base = &upper[..upper.len() - 4];
                format!("{}_USDT", base)
            } else {
                upper
            }
        }
    }
}
```

---

## Summary

1. **Spot Format**: No separator (e.g., `BTCUSDT`)
2. **Futures Format**: Underscore separator (e.g., `BTC_USDT`)
3. **Always Uppercase**: Symbol names must be uppercase
4. **Use exchangeInfo**: Don't parse symbols manually
5. **Validate Status**: Check symbol is enabled before trading
6. **Cache Data**: Store symbol info to reduce API calls
7. **Handle Errors**: Gracefully handle missing or disabled symbols
