# Crypto.com Exchange API v1 - Symbol Formats

## Overview

Crypto.com Exchange uses different naming conventions for different instrument types. Understanding these formats is critical for correct API requests.

---

## Instrument Types

### 1. Spot Trading Pairs

**Format:** `{BASE}_{QUOTE}`

**Examples:**
- `BTC_USDT` - Bitcoin vs Tether
- `ETH_USD` - Ethereum vs US Dollar
- `CRO_BTC` - Cronos vs Bitcoin
- `DOGE_USDC` - Dogecoin vs USD Coin

**Characteristics:**
- Underscore separator
- Base currency first
- Quote currency second
- Both uppercase

---

### 2. Perpetual Swaps

**Format:** `{BASE}{QUOTE}-PERP`

**Examples:**
- `BTCUSD-PERP` - Bitcoin perpetual swap (USD settled)
- `ETHUSD-PERP` - Ethereum perpetual swap
- `CROUPDT-PERP` - CRO perpetual swap (USDT settled)
- `SOLUSD-PERP` - Solana perpetual swap

**Characteristics:**
- No separator between base/quote
- Hyphen before `-PERP` suffix
- No underscore
- All uppercase

---

### 3. Futures Contracts

**Format:** `{BASE}{QUOTE}-{EXPIRY}`

**Examples:**
- `BTCUSD-210528m2` - Bitcoin futures expiring May 28, 2021
- `ETHUSD-220325m2` - Ethereum futures expiring March 25, 2022
- `CROUPDT-230630m2` - CRO futures expiring June 30, 2023

**Expiry Date Format:**
- `YYMMDD` - Year (2 digits), Month, Day
- Suffix `m2` indicates monthly contract

**Characteristics:**
- No separator between base/quote
- Hyphen before expiry code
- Expiry in `YYMMDD` format + contract type suffix

---

### 4. Index Instruments

**Format:** `{BASE}{QUOTE}-INDEX`

**Examples:**
- `BTCUSD-INDEX` - Bitcoin index price
- `ETHUSD-INDEX` - Ethereum index price

**Purpose:**
- Reference price for derivatives
- Used in mark price calculation
- Available via `public/get-valuations`

**Characteristics:**
- Similar to perpetual format
- `-INDEX` suffix instead of `-PERP`

---

## Symbol Formatting Functions

### Rust Implementation

```rust
pub enum InstrumentType {
    Spot,
    Perpetual,
    Futures,
    Index,
}

/// Convert unified symbol to Crypto.com format
pub fn format_symbol(base: &str, quote: &str, instrument_type: InstrumentType) -> String {
    let base_upper = base.to_uppercase();
    let quote_upper = quote.to_uppercase();

    match instrument_type {
        InstrumentType::Spot => {
            format!("{}_{}", base_upper, quote_upper)
        }
        InstrumentType::Perpetual => {
            format!("{}{}-PERP", base_upper, quote_upper)
        }
        InstrumentType::Futures => {
            // Note: Expiry date must be provided separately
            panic!("Futures require expiry date");
        }
        InstrumentType::Index => {
            format!("{}{}-INDEX", base_upper, quote_upper)
        }
    }
}

/// Parse Crypto.com symbol to components
pub fn parse_symbol(symbol: &str) -> Result<(String, String, InstrumentType), String> {
    if symbol.contains("_") {
        // Spot format: BTC_USDT
        let parts: Vec<&str> = symbol.split('_').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid spot symbol: {}", symbol));
        }
        Ok((parts[0].to_string(), parts[1].to_string(), InstrumentType::Spot))
    } else if symbol.ends_with("-PERP") {
        // Perpetual format: BTCUSD-PERP
        let base_quote = symbol.trim_end_matches("-PERP");
        // Need to determine split point (challenging without market data)
        // Common quote currencies: USD, USDT, USDC, BTC, ETH
        if let Some(base) = try_split_base_quote(base_quote) {
            let quote = &base_quote[base.len()..];
            Ok((base, quote.to_string(), InstrumentType::Perpetual))
        } else {
            Err(format!("Cannot parse perpetual symbol: {}", symbol))
        }
    } else if symbol.ends_with("-INDEX") {
        // Index format: BTCUSD-INDEX
        let base_quote = symbol.trim_end_matches("-INDEX");
        if let Some(base) = try_split_base_quote(base_quote) {
            let quote = &base_quote[base.len()..];
            Ok((base, quote.to_string(), InstrumentType::Index))
        } else {
            Err(format!("Cannot parse index symbol: {}", symbol))
        }
    } else if symbol.contains("-") && !symbol.ends_with("-PERP") && !symbol.ends_with("-INDEX") {
        // Futures format: BTCUSD-210528m2
        let parts: Vec<&str> = symbol.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid futures symbol: {}", symbol));
        }
        if let Some(base) = try_split_base_quote(parts[0]) {
            let quote = &parts[0][base.len()..];
            Ok((base, quote.to_string(), InstrumentType::Futures))
        } else {
            Err(format!("Cannot parse futures symbol: {}", symbol))
        }
    } else {
        Err(format!("Unknown symbol format: {}", symbol))
    }
}

/// Try to split base and quote from concatenated string
fn try_split_base_quote(s: &str) -> Option<String> {
    // Common quote currencies in order of likelihood
    let quote_currencies = ["USDT", "USD", "USDC", "BTC", "ETH", "CRO"];

    for quote in &quote_currencies {
        if s.ends_with(quote) {
            let base = &s[..s.len() - quote.len()];
            if !base.is_empty() {
                return Some(base.to_string());
            }
        }
    }
    None
}
```

---

## Common Trading Pairs

### Popular Spot Pairs
```
BTC_USDT
ETH_USDT
BTC_USD
ETH_USD
CRO_USDT
DOGE_USDT
SHIB_USDT
SOL_USDT
MATIC_USDT
AVAX_USDT
```

### Popular Perpetual Swaps
```
BTCUSD-PERP
ETHUSD-PERP
BTCUSDT-PERP
ETHUSDT-PERP
CROUPDT-PERP
SOLUSD-PERP
MATICUSD-PERP
AVAXUSD-PERP
DOGEUSDT-PERP
SHIBUSDT-PERP
```

---

## Retrieving Available Instruments

Use `public/get-instruments` to fetch all tradable instruments:

**Request:**
```json
{
  "id": 1,
  "method": "public/get-instruments",
  "nonce": 1587523073344
}
```

**Response:**
```json
{
  "code": 0,
  "result": {
    "data": [
      {
        "instrument_name": "BTCUSD-PERP",
        "quote_currency": "USD",
        "base_currency": "BTC",
        "instrument_type": "PERPETUAL_SWAP"
      },
      {
        "instrument_name": "BTC_USDT",
        "quote_currency": "USDT",
        "base_currency": "BTC",
        "instrument_type": "SPOT"
      }
    ]
  }
}
```

**Instrument Types:**
- `SPOT` - Spot trading
- `PERPETUAL_SWAP` - Perpetual futures
- `FUTURE` - Dated futures contracts
- `OPTION` - Options contracts (if supported)

---

## Symbol Validation

### Valid Characters
- Uppercase letters: A-Z
- Digits: 0-9
- Underscore: _ (spot only)
- Hyphen: - (derivatives only)

### Invalid Examples
```
btc_usdt        // Wrong: must be uppercase
BTC-USDT        // Wrong: spot uses underscore
BTC_USD-PERP    // Wrong: mixed separators
BTCUSD_PERP     // Wrong: perpetual uses hyphen
```

### Valid Examples
```
BTC_USDT        // Correct spot
BTCUSD-PERP     // Correct perpetual
BTCUSD-210528m2 // Correct futures
BTCUSD-INDEX    // Correct index
```

---

## Settlement Currencies

### Perpetual Swaps
- **USD-settled:** Quote currency is USD (e.g., `BTCUSD-PERP`)
- **USDT-settled:** Quote currency is USDT (e.g., `BTCUSDT-PERP`)
- **Coin-margined:** Quote currency is BTC/ETH (rare)

### Futures Contracts
Same settlement logic as perpetuals, determined by quote currency.

---

## Special Cases

### CRO Pairs
CRO is Crypto.com's native token:
- Spot: `CRO_USDT`, `CRO_BTC`, `CRO_USD`
- Perpetual: `CROUPDT-PERP`, `CROUPD-PERP`

### Stablecoin Pairs
```
USDT_USD    // Tether vs USD
USDC_USD    // USDC vs USD
USDT_USDC   // Stablecoin pair
```

### Exotic Pairs
Some pairs may have unusual quote currencies:
```
BTC_EUR     // Bitcoin vs Euro
ETH_GBP     // Ethereum vs British Pound
```

---

## Symbol Normalization

When working with multiple exchanges, normalize symbols:

```rust
pub struct NormalizedSymbol {
    pub base: String,
    pub quote: String,
    pub instrument_type: InstrumentType,
}

impl NormalizedSymbol {
    /// Convert to Crypto.com format
    pub fn to_crypto_com(&self) -> String {
        match self.instrument_type {
            InstrumentType::Spot => {
                format!("{}_{}", self.base.to_uppercase(), self.quote.to_uppercase())
            }
            InstrumentType::Perpetual => {
                format!("{}{}-PERP", self.base.to_uppercase(), self.quote.to_uppercase())
            }
            InstrumentType::Index => {
                format!("{}{}-INDEX", self.base.to_uppercase(), self.quote.to_uppercase())
            }
            _ => panic!("Unsupported instrument type"),
        }
    }

    /// Parse from Crypto.com format
    pub fn from_crypto_com(symbol: &str) -> Result<Self, String> {
        let (base, quote, instrument_type) = parse_symbol(symbol)?;
        Ok(NormalizedSymbol {
            base,
            quote,
            instrument_type,
        })
    }
}
```

---

## WebSocket Channel Naming

WebSocket channels use the same symbol format as REST endpoints:

**Market Data Channels:**
```
ticker.BTCUSD-PERP
book.BTCUSD-PERP.10
trade.BTCUSD-PERP
candlestick.1h.BTCUSD-PERP

ticker.BTC_USDT
book.BTC_USDT.50
trade.BTC_USDT
candlestick.5m.BTC_USDT
```

**User Channels:**
```
user.order.BTCUSD-PERP
user.trade.BTCUSD-PERP
user.positions
user.balance
```

---

## Best Practices

### 1. Cache Instrument List
```rust
pub struct InstrumentCache {
    instruments: HashMap<String, InstrumentInfo>,
    last_updated: SystemTime,
}

impl InstrumentCache {
    pub async fn refresh(&mut self, api_client: &ApiClient) -> Result<(), Error> {
        let instruments = api_client.get_instruments().await?;
        self.instruments = instruments
            .into_iter()
            .map(|i| (i.instrument_name.clone(), i))
            .collect();
        self.last_updated = SystemTime::now();
        Ok(())
    }

    pub fn is_valid_symbol(&self, symbol: &str) -> bool {
        self.instruments.contains_key(symbol)
    }
}
```

### 2. Validate Before Trading
```rust
pub fn validate_symbol(symbol: &str, cache: &InstrumentCache) -> Result<(), Error> {
    if !cache.is_valid_symbol(symbol) {
        return Err(Error::InvalidSymbol(symbol.to_string()));
    }
    Ok(())
}
```

### 3. Handle Quote Currency Ambiguity
When parsing concatenated symbols like `BTCUSD`, maintain a list of known quote currencies and match longest first:

```rust
// Prefer USDT over USD if both possible
let quote_currencies = ["USDT", "USDC", "USD", "BTC", "ETH"];
```

### 4. Use Instrument Info for Precision
```rust
pub struct InstrumentInfo {
    pub instrument_name: String,
    pub price_decimals: u8,
    pub quantity_decimals: u8,
}

pub fn format_price(price: f64, info: &InstrumentInfo) -> String {
    format!("{:.prec$}", price, prec = info.price_decimals as usize)
}
```

---

## Testing Symbols

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_symbol_format() {
        assert_eq!(
            format_symbol("btc", "usdt", InstrumentType::Spot),
            "BTC_USDT"
        );
    }

    #[test]
    fn test_perpetual_symbol_format() {
        assert_eq!(
            format_symbol("eth", "usd", InstrumentType::Perpetual),
            "ETHUSD-PERP"
        );
    }

    #[test]
    fn test_parse_spot_symbol() {
        let (base, quote, itype) = parse_symbol("BTC_USDT").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "USDT");
        assert!(matches!(itype, InstrumentType::Spot));
    }

    #[test]
    fn test_parse_perpetual_symbol() {
        let (base, quote, itype) = parse_symbol("BTCUSD-PERP").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "USD");
        assert!(matches!(itype, InstrumentType::Perpetual));
    }
}
```

---

## Migration Notes

### From V2 to V1 API
If migrating from Crypto.com V2 API:
- Spot format remains the same: `BTC_USDT`
- Perpetual format changed slightly (verify `-PERP` suffix)
- Always validate symbols with `public/get-instruments`

### Cross-Exchange Compatibility
When building multi-exchange systems:
- Store symbols in normalized format internally
- Convert to exchange-specific format at API boundary
- Maintain mapping tables for common pairs

---

## Quick Reference Table

| Type | Format | Example | Separator | Suffix |
|------|--------|---------|-----------|--------|
| Spot | `BASE_QUOTE` | `BTC_USDT` | `_` | None |
| Perpetual | `BASEQUOTE-PERP` | `BTCUSD-PERP` | None | `-PERP` |
| Futures | `BASEQUOTE-EXPIRY` | `BTCUSD-210528m2` | None | `-YYMMDD...` |
| Index | `BASEQUOTE-INDEX` | `BTCUSD-INDEX` | None | `-INDEX` |

---

## Common Mistakes

1. **Using hyphen for spot:** `BTC-USDT` is invalid (use `BTC_USDT`)
2. **Using underscore for perpetual:** `BTC_USD-PERP` is invalid (use `BTCUSD-PERP`)
3. **Lowercase symbols:** `btc_usdt` is invalid (use `BTC_USDT`)
4. **Wrong suffix:** `BTCUSD-SWAP` is invalid (use `BTCUSD-PERP`)
5. **Mixed separators:** Cannot mix `_` and `-` in base/quote portion
