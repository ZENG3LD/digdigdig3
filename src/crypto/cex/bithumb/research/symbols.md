# Bithumb Symbol Formats and Market Structure

## Overview

Bithumb uses different symbol formats depending on the platform:
- **Bithumb Korea**: Underscore-separated with separate parameters
- **Bithumb Pro**: Hyphen-separated unified symbols

---

## Bithumb Korea Symbol Format

### Symbol Structure

**Format**: `{BASE}_{QUOTE}`

**Examples**:
- `BTC_KRW`
- `ETH_KRW`
- `XRP_KRW`
- `DOGE_KRW`

### Parameter-Based Representation

Bithumb Korea API uses **separate parameters** instead of unified symbols:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `order_currency` | Base currency (cryptocurrency) | `BTC` |
| `payment_currency` | Quote currency (fiat) | `KRW` |

**Usage in API Calls**:
```
GET /public/ticker/BTC_KRW
POST /trade/place?order_currency=BTC&payment_currency=KRW&units=0.5&price=50000000&type=bid
```

### Quote Currencies

**Primary**: `KRW` (Korean Won)

**Note**: Bithumb Korea primarily operates KRW markets. Some endpoints may support `BTC` markets, but KRW is the dominant quote currency.

### Base Currencies (Examples)

Major cryptocurrencies supported:
- `BTC` - Bitcoin
- `ETH` - Ethereum
- `XRP` - Ripple
- `BCH` - Bitcoin Cash
- `EOS` - EOS
- `LTC` - Litecoin
- `TRX` - Tron
- `ADA` - Cardano
- `DOGE` - Dogecoin
- `DOT` - Polkadot
- `LINK` - Chainlink
- `MATIC` - Polygon
- `SOL` - Solana
- And many more...

**Full List**: Use `/public/ticker/ALL_KRW` to get all available trading pairs

### Symbol Formatting Functions

```rust
// Split symbol into components
fn parse_bithumb_korea_symbol(symbol: &str) -> Result<(String, String), ExchangeError> {
    let parts: Vec<&str> = symbol.split('_').collect();
    if parts.len() != 2 {
        return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
    }
    Ok((parts[0].to_uppercase(), parts[1].to_uppercase()))
}

// Build symbol from components
fn build_bithumb_korea_symbol(base: &str, quote: &str) -> String {
    format!("{}_{}", base.to_uppercase(), quote.to_uppercase())
}

// Convert to API parameters
fn to_api_params(symbol: &str) -> Result<(String, String), ExchangeError> {
    let (base, quote) = parse_bithumb_korea_symbol(symbol)?;
    Ok((base, quote))  // order_currency, payment_currency
}
```

### Example API Calls

**Ticker - Single Symbol**:
```
GET /public/ticker/BTC_KRW
```

**Ticker - All Symbols**:
```
GET /public/ticker/ALL_KRW
```

**Order Book**:
```
GET /public/orderbook/BTC_KRW
```

**Recent Trades**:
```
GET /public/transaction_history/ETH_KRW
```

**Place Order**:
```
POST /trade/place
Body: {
  "order_currency": "BTC",
  "payment_currency": "KRW",
  "units": "0.5",
  "price": "50000000",
  "type": "bid"
}
```

---

## Bithumb Pro Symbol Format

### Symbol Structure

**Format**: `{BASE}-{QUOTE}`

**Examples**:
- `BTC-USDT`
- `ETH-USDT`
- `XRP-USDT`
- `BTC-BTC` (for some pairs)

**Separator**: Hyphen (`-`) instead of underscore

### Quote Currencies

**Primary**: `USDT` (Tether USD)

**Secondary**:
- `BTC` (Bitcoin)
- `ETH` (Ethereum)

### Base Currencies

Similar to Bithumb Korea, but with international focus:
- Major coins: `BTC`, `ETH`, `XRP`, `LTC`, `BCH`, `EOS`
- DeFi tokens: `UNI`, `AAVE`, `LINK`, `SUSHI`
- Layer 1: `SOL`, `AVAX`, `MATIC`, `DOT`, `ATOM`
- Meme coins: `DOGE`, `SHIB`
- And more...

### Symbol Formatting Functions

```rust
// Split symbol into components
fn parse_bithumb_pro_symbol(symbol: &str) -> Result<(String, String), ExchangeError> {
    let parts: Vec<&str> = symbol.split('-').collect();
    if parts.len() != 2 {
        return Err(ExchangeError::InvalidSymbol(symbol.to_string()));
    }
    Ok((parts[0].to_uppercase(), parts[1].to_uppercase()))
}

// Build symbol from components
fn build_bithumb_pro_symbol(base: &str, quote: &str) -> String {
    format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
}

// Convert between formats
fn korea_to_pro(korea_symbol: &str) -> Result<String, ExchangeError> {
    let (base, quote) = parse_bithumb_korea_symbol(korea_symbol)?;
    Ok(build_bithumb_pro_symbol(&base, &quote))
}

fn pro_to_korea(pro_symbol: &str) -> Result<String, ExchangeError> {
    let (base, quote) = parse_bithumb_pro_symbol(pro_symbol)?;
    Ok(build_bithumb_korea_symbol(&base, &quote))
}
```

### Example API Calls

**Ticker - Single Symbol**:
```
GET /spot/ticker?symbol=BTC-USDT
```

**Ticker - All Symbols**:
```
GET /spot/ticker?symbol=ALL
```

**Order Book**:
```
GET /spot/orderBook?symbol=BTC-USDT
```

**Recent Trades**:
```
GET /spot/trades?symbol=ETH-USDT
```

**Place Order**:
```
POST /spot/placeOrder
Body: {
  "apiKey": "...",
  "timestamp": 1712230310689,
  "signature": "...",
  "symbol": "BTC-USDT",
  "type": "limit",
  "side": "buy",
  "price": "50000",
  "quantity": "0.5"
}
```

---

## Symbol Normalization

### Unified Symbol Format for Connector

For V5 connector implementation, use a **unified internal format**:

**Recommended**: `{BASE}/{QUOTE}` (slash-separated)

**Examples**:
- `BTC/KRW`
- `BTC/USDT`
- `ETH/KRW`

### Conversion Layer

```rust
pub struct SymbolConverter;

impl SymbolConverter {
    // Unified to Bithumb Korea
    pub fn to_korea(unified: &str) -> Result<String, ExchangeError> {
        let (base, quote) = Self::parse_unified(unified)?;
        Ok(format!("{}_{}", base, quote))
    }

    // Unified to Bithumb Pro
    pub fn to_pro(unified: &str) -> Result<String, ExchangeError> {
        let (base, quote) = Self::parse_unified(unified)?;
        Ok(format!("{}-{}", base, quote))
    }

    // Bithumb Korea to Unified
    pub fn from_korea(korea: &str) -> Result<String, ExchangeError> {
        let parts: Vec<&str> = korea.split('_').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::InvalidSymbol(korea.to_string()));
        }
        Ok(format!("{}/{}", parts[0], parts[1]))
    }

    // Bithumb Pro to Unified
    pub fn from_pro(pro: &str) -> Result<String, ExchangeError> {
        let parts: Vec<&str> = pro.split('-').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::InvalidSymbol(pro.to_string()));
        }
        Ok(format!("{}/{}", parts[0], parts[1]))
    }

    // Parse unified format
    fn parse_unified(unified: &str) -> Result<(String, String), ExchangeError> {
        let parts: Vec<&str> = unified.split('/').collect();
        if parts.len() != 2 {
            return Err(ExchangeError::InvalidSymbol(unified.to_string()));
        }
        Ok((parts[0].to_uppercase(), parts[1].to_uppercase()))
    }
}
```

### Usage Example

```rust
// User provides unified format
let unified_symbol = "BTC/KRW";

// Convert for Bithumb Korea API
let korea_symbol = SymbolConverter::to_korea(unified_symbol)?; // "BTC_KRW"

// Convert for Bithumb Pro API
let pro_symbol = SymbolConverter::to_pro(unified_symbol)?; // "BTC-KRW"

// Extract components for Korea API parameters
let (order_currency, payment_currency) = parse_bithumb_korea_symbol(&korea_symbol)?;
// order_currency = "BTC", payment_currency = "KRW"
```

---

## Market Information

### Getting Available Symbols

**Bithumb Korea**:
```rust
// Request all tickers
let response = client.get("/public/ticker/ALL_KRW").send().await?;

// Parse response to extract available symbols
let data = response.json::<BithumbKoreaResponse<HashMap<String, TickerData>>>().await?;
let symbols: Vec<String> = data.data
    .keys()
    .filter(|k| *k != "date")  // Filter out metadata
    .map(|base| format!("{}_KRW", base))
    .collect();
```

**Bithumb Pro**:
```rust
// Request configuration
let response = client.get("/spot/config").send().await?;

// Or request all tickers
let response = client.get("/spot/ticker?symbol=ALL").send().await?;

// Parse to extract symbols
let data = response.json::<BithumbProResponse<Vec<TickerData>>>().await?;
let symbols: Vec<String> = data.data
    .iter()
    .map(|ticker| ticker.symbol.clone())
    .collect();
```

### Symbol Validation

```rust
pub fn validate_symbol(symbol: &str, platform: Platform) -> Result<(), ExchangeError> {
    match platform {
        Platform::Korea => {
            if !symbol.contains('_') {
                return Err(ExchangeError::InvalidSymbol(
                    format!("Korea symbol must use underscore: {}", symbol)
                ));
            }
            let parts: Vec<&str> = symbol.split('_').collect();
            if parts.len() != 2 {
                return Err(ExchangeError::InvalidSymbol(
                    format!("Invalid symbol format: {}", symbol)
                ));
            }
        }
        Platform::Pro => {
            if !symbol.contains('-') {
                return Err(ExchangeError::InvalidSymbol(
                    format!("Pro symbol must use hyphen: {}", symbol)
                ));
            }
            let parts: Vec<&str> = symbol.split('-').collect();
            if parts.len() != 2 {
                return Err(ExchangeError::InvalidSymbol(
                    format!("Invalid symbol format: {}", symbol)
                ));
            }
        }
    }
    Ok(())
}
```

---

## Currency Precision

### Bithumb Korea

**Price Precision** (KRW):
- BTC: No decimal (integer KRW)
- ETH: No decimal (integer KRW)
- Most coins: No decimal (integer KRW)

**Quantity Precision**:
- BTC: 8 decimals
- ETH: 8 decimals
- Varies by currency

**Example Order**:
```json
{
  "order_currency": "BTC",
  "payment_currency": "KRW",
  "units": "0.12345678",     // 8 decimals
  "price": "50000000",       // integer (50,000,000 KRW)
  "type": "bid"
}
```

### Bithumb Pro

**Price Precision** (USDT):
- BTC: 2 decimals (e.g., "50000.00")
- ETH: 2 decimals (e.g., "3000.00")
- Varies by pair

**Quantity Precision**:
- BTC: 8 decimals
- ETH: 8 decimals
- Varies by currency

**Example Order**:
```json
{
  "symbol": "BTC-USDT",
  "price": "50000.00",       // 2 decimals
  "quantity": "0.12345678",  // 8 decimals
  "side": "buy",
  "type": "limit"
}
```

### Precision Helper Functions

```rust
use rust_decimal::Decimal;

pub struct SymbolPrecision {
    pub price_decimals: u32,
    pub quantity_decimals: u32,
}

impl SymbolPrecision {
    pub fn for_korea_pair(base: &str) -> Self {
        Self {
            price_decimals: 0,  // KRW is always integer
            quantity_decimals: match base {
                "BTC" | "ETH" | "XRP" => 8,
                _ => 8,  // Default to 8
            }
        }
    }

    pub fn for_pro_pair(base: &str, quote: &str) -> Self {
        let price_decimals = match quote {
            "USDT" => 2,
            "BTC" => 8,
            _ => 2,
        };

        let quantity_decimals = match base {
            "BTC" | "ETH" => 8,
            _ => 8,
        };

        Self { price_decimals, quantity_decimals }
    }

    pub fn round_price(&self, price: Decimal) -> Decimal {
        price.round_dp(self.price_decimals)
    }

    pub fn round_quantity(&self, quantity: Decimal) -> Decimal {
        quantity.round_dp(self.quantity_decimals)
    }
}
```

---

## Special Symbols

### ALL Symbol

Both platforms support special `ALL` symbol for bulk queries:

**Bithumb Korea**:
- `ALL_KRW` - All KRW pairs
- Usage: `GET /public/ticker/ALL_KRW`

**Bithumb Pro**:
- `ALL` - All trading pairs
- Usage: `GET /spot/ticker?symbol=ALL`

### Handling ALL Symbol

```rust
pub fn is_all_symbol(symbol: &str) -> bool {
    symbol == "ALL" || symbol.starts_with("ALL_")
}

pub fn get_all_symbol(platform: Platform, quote: Option<&str>) -> String {
    match platform {
        Platform::Korea => {
            let q = quote.unwrap_or("KRW");
            format!("ALL_{}", q)
        }
        Platform::Pro => "ALL".to_string(),
    }
}
```

---

## Implementation Checklist

- [ ] Symbol parser for Korea format (`BTC_KRW`)
- [ ] Symbol parser for Pro format (`BTC-USDT`)
- [ ] Unified format converter (`BTC/KRW`)
- [ ] Symbol validation
- [ ] Parameter extractor (order_currency, payment_currency)
- [ ] Precision handler per trading pair
- [ ] ALL symbol support
- [ ] Symbol list fetcher from exchange
- [ ] Symbol normalization for WebSocket subscriptions
- [ ] Error handling for invalid symbols

---

## Testing Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_korea_symbol_parsing() {
        let (base, quote) = parse_bithumb_korea_symbol("BTC_KRW").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "KRW");
    }

    #[test]
    fn test_pro_symbol_parsing() {
        let (base, quote) = parse_bithumb_pro_symbol("BTC-USDT").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "USDT");
    }

    #[test]
    fn test_unified_conversion() {
        let unified = "BTC/KRW";
        let korea = SymbolConverter::to_korea(unified).unwrap();
        assert_eq!(korea, "BTC_KRW");

        let pro = SymbolConverter::to_pro(unified).unwrap();
        assert_eq!(pro, "BTC-KRW");
    }

    #[test]
    fn test_roundtrip() {
        let original = "BTC/USDT";
        let pro = SymbolConverter::to_pro(original).unwrap();
        let back = SymbolConverter::from_pro(&pro).unwrap();
        assert_eq!(original, back);
    }
}
```
