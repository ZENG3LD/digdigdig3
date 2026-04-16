# BingX Symbol Format and Normalization

Complete documentation of BingX symbol formats and normalization requirements.

---

## Symbol Format Overview

BingX uses **hyphenated format** for trading pairs across most of its API:

**Standard Format:** `BASE-QUOTE`

**Examples:**
- `BTC-USDT`
- `ETH-USDT`
- `BNB-USDT`
- `SOL-USDT`

---

## Format by Market Type

### Spot Trading

**Format:** `BASE-USDT` (hyphen separator)

**Examples:**
```
BTC-USDT
ETH-USDT
BNB-USDT
XRP-USDT
ADA-USDT
DOT-USDT
DOGE-USDT
```

### Perpetual Swap (USDT-M)

**Format:** `BASE-USDT` (hyphen separator)

**Examples:**
```
BTC-USDT
ETH-USDT
BNB-USDT
SOL-USDT
MATIC-USDT
AVAX-USDT
```

### Coin-M Perpetual Futures

**Format:** Varies, uses different endpoint path (`/openApi/cswap/v1/`)

**Examples:**
```
BTCUSD (inverse perpetual)
ETHUSD (inverse perpetual)
```

---

## Symbol Normalization

### Standard to BingX Format

Convert standard formats to BingX format:

```
BTCUSDT   → BTC-USDT
ETHUSDT   → ETH-USDT
BTC/USDT  → BTC-USDT
BTC_USDT  → BTC-USDT
```

### BingX to Standard Format

Convert BingX format to other standards:

```
BTC-USDT  → BTCUSDT  (no separator)
BTC-USDT  → BTC/USDT (slash separator)
BTC-USDT  → BTC_USDT (underscore separator)
```

---

## Rust Implementation

### Symbol Normalization Functions

```rust
/// Convert any symbol format to BingX format (BASE-QUOTE)
pub fn normalize_symbol(symbol: &str) -> String {
    // Remove common separators
    let clean = symbol
        .replace("/", "")
        .replace("_", "")
        .replace("-", "");

    // Split into base and quote
    // Most common quote assets: USDT, USDC, BUSD, USD
    for quote in &["USDT", "USDC", "BUSD", "USD"] {
        if clean.ends_with(quote) {
            let base = &clean[..clean.len() - quote.len()];
            return format!("{}-{}", base, quote);
        }
    }

    // If no known quote asset, assume last 4 chars
    if clean.len() > 4 {
        let base = &clean[..clean.len() - 4];
        let quote = &clean[clean.len() - 4..];
        return format!("{}-{}", base, quote);
    }

    // Return as-is if can't determine
    clean
}

/// Convert BingX format to standard format (no separator)
pub fn to_standard_symbol(bingx_symbol: &str) -> String {
    bingx_symbol.replace("-", "")
}

/// Convert BingX format to slash format (BASE/QUOTE)
pub fn to_slash_symbol(bingx_symbol: &str) -> String {
    bingx_symbol.replace("-", "/")
}

/// Validate BingX symbol format
pub fn is_valid_bingx_symbol(symbol: &str) -> bool {
    symbol.contains("-") && symbol.split("-").count() == 2
}

/// Extract base and quote assets from BingX symbol
pub fn split_symbol(bingx_symbol: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = bingx_symbol.split("-").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}
```

### Usage Examples

```rust
fn main() {
    // Normalize to BingX format
    assert_eq!(normalize_symbol("BTCUSDT"), "BTC-USDT");
    assert_eq!(normalize_symbol("BTC/USDT"), "BTC-USDT");
    assert_eq!(normalize_symbol("BTC_USDT"), "BTC-USDT");
    assert_eq!(normalize_symbol("ETH-USDT"), "ETH-USDT");

    // Convert from BingX format
    assert_eq!(to_standard_symbol("BTC-USDT"), "BTCUSDT");
    assert_eq!(to_slash_symbol("BTC-USDT"), "BTC/USDT");

    // Validate format
    assert!(is_valid_bingx_symbol("BTC-USDT"));
    assert!(!is_valid_bingx_symbol("BTCUSDT"));

    // Split symbol
    if let Some((base, quote)) = split_symbol("BTC-USDT") {
        println!("Base: {}, Quote: {}", base, quote);
        // Output: Base: BTC, Quote: USDT
    }
}
```

---

## Symbol Information

### Get Available Symbols

#### Spot Symbols

**Endpoint:** `GET /openApi/spot/v1/common/symbols`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbols": [
      {
        "symbol": "BTC-USDT",
        "minQty": 0.0001,
        "maxQty": 100,
        "minNotional": 5,
        "maxNotional": 1000000,
        "status": "TRADING"
      }
    ]
  }
}
```

**Extract symbols:**
```rust
async fn get_spot_symbols(client: &reqwest::Client) -> Result<Vec<String>, Error> {
    let url = "https://open-api.bingx.com/openApi/spot/v1/common/symbols";
    let response: SymbolsResponse = client.get(url).send().await?.json().await?;

    Ok(response
        .data
        .symbols
        .into_iter()
        .filter(|s| s.status == "TRADING")
        .map(|s| s.symbol)
        .collect())
}
```

#### Swap Symbols

**Endpoint:** `GET /openApi/swap/v2/quote/contracts`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "symbol": "BTC-USDT",
      "currency": "USDT",
      "asset": "BTC",
      "size": 0.001,
      "quantityPrecision": 3,
      "pricePrecision": 1,
      "feeRate": 0.0005,
      "tradeMinQuantity": 0.001,
      "tradeMinUSDT": 5.0
    }
  ]
}
```

**Extract symbols:**
```rust
async fn get_swap_symbols(client: &reqwest::Client) -> Result<Vec<String>, Error> {
    let url = "https://open-api.bingx.com/openApi/swap/v2/quote/contracts";
    let response: ContractsResponse = client.get(url).send().await?.json().await?;

    Ok(response
        .data
        .into_iter()
        .map(|c| c.symbol)
        .collect())
}
```

---

## Symbol Precision

Different symbols have different precision requirements for price and quantity.

### Price Precision

**Definition:** Number of decimal places for price values.

**Example:**
```
BTC-USDT: pricePrecision = 1  → 43302.5 (1 decimal place)
ETH-USDT: pricePrecision = 2  → 2925.30 (2 decimal places)
DOGE-USDT: pricePrecision = 5 → 0.08234 (5 decimal places)
```

### Quantity Precision

**Definition:** Number of decimal places for quantity values.

**Example:**
```
BTC-USDT: quantityPrecision = 4 → 0.1234 (4 decimal places)
ETH-USDT: quantityPrecision = 3 → 1.250 (3 decimal places)
BNB-USDT: quantityPrecision = 2 → 10.50 (2 decimal places)
```

### Precision Handling in Rust

```rust
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Round price to symbol's precision
pub fn round_price(price: Decimal, precision: u32) -> Decimal {
    let scale = 10_u64.pow(precision);
    let scaled = (price * Decimal::from(scale))
        .round()
        .to_f64()
        .unwrap_or(0.0);
    Decimal::from_f64_retain(scaled).unwrap() / Decimal::from(scale)
}

/// Round quantity to symbol's precision
pub fn round_quantity(quantity: Decimal, precision: u32) -> Decimal {
    round_price(quantity, precision)
}

/// Format price with proper precision
pub fn format_price(price: Decimal, precision: u32) -> String {
    format!("{:.prec$}", price, prec = precision as usize)
}
```

**Usage:**
```rust
let btc_price = Decimal::from_str("43302.567").unwrap();
let rounded = round_price(btc_price, 1); // 43302.6
let formatted = format_price(rounded, 1); // "43302.6"
```

---

## Symbol Constraints

Each symbol has trading constraints that must be respected.

### Minimum/Maximum Quantity

**From symbol info:**
```json
{
  "symbol": "BTC-USDT",
  "minQty": 0.0001,
  "maxQty": 100
}
```

**Validation:**
```rust
pub fn validate_quantity(
    quantity: Decimal,
    min_qty: Decimal,
    max_qty: Decimal,
) -> Result<(), String> {
    if quantity < min_qty {
        return Err(format!(
            "Quantity {} is less than minimum {}",
            quantity, min_qty
        ));
    }
    if quantity > max_qty {
        return Err(format!(
            "Quantity {} exceeds maximum {}",
            quantity, max_qty
        ));
    }
    Ok(())
}
```

### Minimum/Maximum Notional

**Notional Value = Price × Quantity**

**From symbol info:**
```json
{
  "symbol": "BTC-USDT",
  "minNotional": 5,
  "maxNotional": 1000000
}
```

**Validation:**
```rust
pub fn validate_notional(
    price: Decimal,
    quantity: Decimal,
    min_notional: Decimal,
    max_notional: Decimal,
) -> Result<(), String> {
    let notional = price * quantity;

    if notional < min_notional {
        return Err(format!(
            "Notional value {} is less than minimum {}",
            notional, min_notional
        ));
    }
    if notional > max_notional {
        return Err(format!(
            "Notional value {} exceeds maximum {}",
            notional, max_notional
        ));
    }
    Ok(())
}
```

### Complete Validation

```rust
pub struct SymbolInfo {
    pub symbol: String,
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub min_notional: Decimal,
    pub max_notional: Decimal,
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub status: String,
}

impl SymbolInfo {
    pub fn validate_order(
        &self,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<(), String> {
        // Check symbol is trading
        if self.status != "TRADING" {
            return Err(format!("Symbol {} is not trading", self.symbol));
        }

        // Validate quantity
        validate_quantity(quantity, self.min_qty, self.max_qty)?;

        // Validate notional
        validate_notional(
            price,
            quantity,
            self.min_notional,
            self.max_notional,
        )?;

        Ok(())
    }

    pub fn round_order(
        &self,
        price: Decimal,
        quantity: Decimal,
    ) -> (Decimal, Decimal) {
        let rounded_price = round_price(price, self.price_precision);
        let rounded_qty = round_quantity(quantity, self.quantity_precision);
        (rounded_price, rounded_qty)
    }
}
```

---

## Symbol Status

Symbols can have different trading statuses:

| Status | Description | Can Trade |
|--------|-------------|-----------|
| `TRADING` | Normal trading | Yes |
| `HALT` | Trading halted | No |
| `BREAK` | Trading break | No |

**Filter for active symbols:**
```rust
async fn get_active_symbols(client: &reqwest::Client) -> Result<Vec<String>, Error> {
    let all_symbols = get_spot_symbols(client).await?;

    // Query each symbol's info
    let mut active = Vec::new();
    for symbol in all_symbols {
        let info = get_symbol_info(client, &symbol).await?;
        if info.status == "TRADING" {
            active.push(symbol);
        }
    }

    Ok(active)
}
```

---

## WebSocket Symbol Format

WebSocket streams use the same hyphenated format with `@` separator for stream type:

**Format:** `SYMBOL@STREAM_TYPE`

**Examples:**
```
BTC-USDT@depth        # Depth stream
BTC-USDT@trade        # Trade stream
BTC-USDT@kline_1min   # Kline stream
BTC-USDT@ticker       # Ticker stream
ETH-USDT@depth20      # Level 20 depth
```

**Subscribe message:**
```json
{
  "id": "unique-id",
  "reqType": "sub",
  "dataType": "BTC-USDT@depth"
}
```

---

## Symbol Cache

Cache symbol information to avoid repeated API calls:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct SymbolCache {
    symbols: Arc<RwLock<HashMap<String, SymbolInfo>>>,
    last_update: Arc<RwLock<Instant>>,
    ttl: Duration,
}

impl SymbolCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
            ttl,
        }
    }

    pub async fn get_symbol(&self, symbol: &str) -> Option<SymbolInfo> {
        let cache = self.symbols.read().await;
        cache.get(symbol).cloned()
    }

    pub async fn update(&self, symbols: Vec<SymbolInfo>) {
        let mut cache = self.symbols.write().await;
        let mut last_update = self.last_update.write().await;

        cache.clear();
        for symbol_info in symbols {
            cache.insert(symbol_info.symbol.clone(), symbol_info);
        }

        *last_update = Instant::now();
    }

    pub async fn is_stale(&self) -> bool {
        let last_update = self.last_update.read().await;
        last_update.elapsed() > self.ttl
    }

    pub async fn refresh_if_stale(&self, client: &reqwest::Client) -> Result<(), Error> {
        if self.is_stale().await {
            let symbols = fetch_all_symbol_info(client).await?;
            self.update(symbols).await;
        }
        Ok(())
    }
}

async fn fetch_all_symbol_info(client: &reqwest::Client) -> Result<Vec<SymbolInfo>, Error> {
    // Fetch from API and parse into SymbolInfo structs
    // Implementation depends on specific API response format
    unimplemented!()
}
```

**Usage:**
```rust
let cache = SymbolCache::new(Duration::from_secs(3600)); // 1 hour TTL

// Refresh if needed
cache.refresh_if_stale(&client).await?;

// Get symbol info
if let Some(info) = cache.get_symbol("BTC-USDT").await {
    let (price, qty) = info.round_order(
        Decimal::from_str("43302.567")?,
        Decimal::from_str("0.12345")?
    );
    println!("Rounded: {} @ {}", qty, price);
}
```

---

## Common Quote Assets

BingX supports various quote assets:

### Spot Market
- **USDT** - Most common, highest liquidity
- **USDC** - USD Coin stablecoin
- **BUSD** - Binance USD (may be deprecated)

### Perpetual Swap
- **USDT** - USDT-margined perpetuals (most common)
- **USD** - Coin-margined perpetuals (inverse contracts)

---

## Case Sensitivity

BingX symbols are **case-sensitive** in some contexts:

**Correct:**
```
BTC-USDT  ✓
ETH-USDT  ✓
```

**Incorrect:**
```
btc-usdt  ✗
Btc-Usdt  ✗
BTC-usdt  ✗
```

**Always use uppercase** for both base and quote assets.

---

## Special Characters

Symbol names use only:
- **Letters:** A-Z (uppercase)
- **Numbers:** 0-9
- **Separator:** `-` (hyphen)

**Valid:**
```
BTC-USDT   ✓
1INCH-USDT ✓
```

**Invalid:**
```
BTC_USDT   ✗
BTC/USDT   ✗
BTC USDT   ✗
```

---

## Migration Notes

### From Other Exchanges

When migrating from other exchanges:

**Binance:**
```
BTCUSDT → BTC-USDT
```

**Bybit:**
```
BTCUSDT → BTC-USDT
```

**OKX:**
```
BTC-USDT → BTC-USDT (already compatible)
```

**Kraken:**
```
XXBTZUSD → BTC-USDT (requires mapping)
XBT/USD  → BTC-USDT
```

### Universal Converter

```rust
pub fn convert_to_bingx(symbol: &str, from_exchange: &str) -> String {
    match from_exchange {
        "binance" | "bybit" => normalize_symbol(symbol),
        "okx" => symbol.to_string(), // Already compatible
        "kraken" => {
            // Kraken uses special prefixes
            let clean = symbol
                .replace("XXBT", "BTC")
                .replace("XBT", "BTC")
                .replace("ZUSD", "USD");
            normalize_symbol(&clean)
        }
        _ => normalize_symbol(symbol),
    }
}
```

---

## Best Practices

1. **Always normalize** symbols when accepting user input
2. **Cache symbol info** to reduce API calls
3. **Validate** before sending orders
4. **Round** price/quantity to proper precision
5. **Use uppercase** for all symbols
6. **Handle errors** when symbol not found
7. **Refresh cache** periodically (e.g., every hour)
8. **Check status** before trading

---

## Sources

- [BingX API Docs](https://bingx-api.github.io/docs/)
- [BingX Standard Contract API](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [CCXT BingX Implementation](https://docs.ccxt.com/exchanges/bingx)
- [BingX Symbol Format Issues](https://github.com/ccxt/ccxt/issues/24036)
