# Gate.io Symbol Format

**Research Date**: 2026-01-21

---

## Symbol Format

### Spot Trading

**Format**: `BASE_QUOTE` with **underscore** separator

**Examples**:
- `BTC_USDT`
- `ETH_USDT`
- `ETH_BTC`
- `SOL_USDT`
- `DOGE_USDT`

**Pattern**: `{BASE_CURRENCY}_{QUOTE_CURRENCY}`

### Futures Trading

**Format**: `BASE_QUOTE` with **underscore** separator (same as spot!)

**Examples**:
- `BTC_USDT` (USDT-margined perpetual)
- `ETH_USDT`
- `BTC_USD` (inverse perpetual, settled in BTC)
- `ETH_USD`

**Pattern**: `{BASE_CURRENCY}_{QUOTE_CURRENCY}`

**Settlement Currency**:
- USDT contracts: Use `{settle}=usdt` in API path
- BTC contracts: Use `{settle}=btc` in API path

**Example URLs**:
- `/futures/usdt/tickers?contract=BTC_USDT`
- `/futures/btc/tickers?contract=BTC_USD`

---

## Comparison with Other Exchanges

| Exchange | Spot Format | Futures Format | Notes |
|----------|-------------|----------------|-------|
| **Gate.io** | `BTC_USDT` | `BTC_USDT` | Underscore, same for both |
| Binance | `BTCUSDT` | `BTCUSDT` | No separator |
| KuCoin | `BTC-USDT` | `XBTUSDM` | Hyphen for spot, suffix for futures |
| OKX | `BTC-USDT` | `BTC-USDT-SWAP` | Hyphen, suffix for perpetuals |
| Bybit | `BTCUSDT` | `BTCUSDT` | No separator |

**Gate.io is consistent**: Spot and futures use **same format** with underscore.

---

## Symbol Conversion Functions

### From Standard Format

**Standard format**: `"BTC/USDT"` (with slash)

**Convert to Gate.io**:
```rust
fn to_gateio_symbol(standard: &str) -> String {
    standard.replace("/", "_")
}

// Examples:
// "BTC/USDT" -> "BTC_USDT"
// "ETH/BTC" -> "ETH_BTC"
```

### To Standard Format

**Gate.io format**: `"BTC_USDT"` (with underscore)

**Convert to standard**:
```rust
fn from_gateio_symbol(gateio: &str) -> String {
    gateio.replace("_", "/")
}

// Examples:
// "BTC_USDT" -> "BTC/USDT"
// "ETH_BTC" -> "ETH/BTC"
```

### Case Sensitivity

**Gate.io symbols are case-insensitive** in the API, but conventionally **uppercase**.

**Recommended**: Always use uppercase (`BTC_USDT`, not `btc_usdt`)

---

## Symbol Information Endpoint

### Get All Spot Symbols

**Endpoint**: `GET /spot/currency_pairs`

**Response**:
```json
[
  {
    "id": "BTC_USDT",
    "base": "BTC",
    "quote": "USDT",
    "fee": "0.2",
    "min_base_amount": "0.0001",
    "min_quote_amount": "1.0",
    "amount_precision": 4,
    "precision": 2,
    "trade_status": "tradable",
    "sell_start": 0,
    "buy_start": 0
  }
]
```

**Key fields**:
- `id`: Symbol identifier (e.g., "BTC_USDT")
- `base`: Base currency (e.g., "BTC")
- `quote`: Quote currency (e.g., "USDT")
- `trade_status`: "tradable" or "untradable"
- `min_base_amount`: Minimum order size (base currency)
- `min_quote_amount`: Minimum order size (quote currency)
- `amount_precision`: Decimal places for amount
- `precision`: Decimal places for price

### Get All Futures Contracts

**Endpoint**: `GET /futures/{settle}/contracts`

**Response**:
```json
[
  {
    "name": "BTC_USDT",
    "type": "direct",
    "quanto_multiplier": "0.0001",
    "leverage_min": "1",
    "leverage_max": "100",
    "maintenance_rate": "0.005",
    "mark_type": "index",
    "order_size_min": 1,
    "order_size_max": 1000000,
    ...
  }
]
```

**Key fields**:
- `name`: Contract symbol (e.g., "BTC_USDT")
- `type`: "direct" (linear/USDT) or "inverse" (BTC)
- `leverage_min`, `leverage_max`: Leverage range
- `order_size_min`, `order_size_max`: Order size limits

---

## Parsing Symbol Components

### Extract Base and Quote

```rust
fn parse_gateio_symbol(symbol: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = symbol.split('_').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

// Examples:
// "BTC_USDT" -> Some(("BTC", "USDT"))
// "ETH_BTC" -> Some(("ETH", "BTC"))
// "INVALID" -> None
```

### Build Symbol from Components

```rust
fn build_gateio_symbol(base: &str, quote: &str) -> String {
    format!("{}_{}", base.to_uppercase(), quote.to_uppercase())
}

// Examples:
// ("BTC", "USDT") -> "BTC_USDT"
// ("eth", "btc") -> "ETH_BTC"
```

---

## Symbol Validation

### Valid Symbol Pattern

```rust
use regex::Regex;

fn is_valid_gateio_symbol(symbol: &str) -> bool {
    let re = Regex::new(r"^[A-Z0-9]+_[A-Z0-9]+$").unwrap();
    re.is_match(symbol)
}

// Valid:
// "BTC_USDT" -> true
// "ETH_BTC" -> true
// "SHIB_USDT" -> true

// Invalid:
// "BTCUSDT" -> false (no underscore)
// "BTC-USDT" -> false (hyphen instead of underscore)
// "btc_usdt" -> false (lowercase)
// "BTC_" -> false (missing quote)
```

---

## Common Symbol Mappings

### USDT Pairs (Spot & Futures)

| Standard | Gate.io Spot | Gate.io Futures USDT |
|----------|--------------|----------------------|
| BTC/USDT | `BTC_USDT` | `BTC_USDT` |
| ETH/USDT | `ETH_USDT` | `ETH_USDT` |
| SOL/USDT | `SOL_USDT` | `SOL_USDT` |
| DOGE/USDT | `DOGE_USDT` | `DOGE_USDT` |
| XRP/USDT | `XRP_USDT` | `XRP_USDT` |

### BTC Pairs (Spot)

| Standard | Gate.io Spot |
|----------|--------------|
| ETH/BTC | `ETH_BTC` |
| XRP/BTC | `XRP_BTC` |
| LTC/BTC | `LTC_BTC` |
| DOGE/BTC | `DOGE_BTC` |

### Inverse Futures (BTC-settled)

| Standard | Gate.io Futures BTC |
|----------|---------------------|
| BTC/USD | `BTC_USD` |
| ETH/USD | `ETH_USD` |

**API Path**: Use `/futures/btc/...` for these contracts.

---

## Special Cases

### Stablecoins

**USDT pairs**: Most common quote currency
```
BTC_USDT, ETH_USDT, SOL_USDT
```

**USDC pairs**: Alternative stablecoin
```
BTC_USDC, ETH_USDC
```

**USD pairs**: For inverse futures
```
BTC_USD, ETH_USD
```

### Tokens with Numbers

Some tokens include numbers in their name:
```
1INCH_USDT
SAND_USDT
MANA_USDT
```

Format remains: `{TOKEN}_{QUOTE}`

### Long Token Names

```
AVAX_USDT
MATIC_USDT
```

No abbreviation, use full token symbol.

---

## Implementation Recommendations

### 1. Symbol Normalization Function

```rust
pub fn normalize_symbol(symbol: &str) -> String {
    // Remove common separators, convert to Gate.io format
    let normalized = symbol
        .replace("/", "_")
        .replace("-", "_")
        .to_uppercase();

    // Validate format
    if !normalized.contains('_') {
        // Try to infer separator position
        // e.g., "BTCUSDT" -> "BTC_USDT"
        // This is exchange-specific logic
    }

    normalized
}
```

### 2. Symbol Registry

Maintain a mapping of standard symbols to Gate.io symbols:

```rust
use std::collections::HashMap;

pub struct SymbolRegistry {
    standard_to_gateio: HashMap<String, String>,
    gateio_to_standard: HashMap<String, String>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            standard_to_gateio: HashMap::new(),
            gateio_to_standard: HashMap::new(),
        };

        // Register common pairs
        registry.register("BTC/USDT", "BTC_USDT");
        registry.register("ETH/USDT", "ETH_USDT");
        // ... more pairs

        registry
    }

    fn register(&mut self, standard: &str, gateio: &str) {
        self.standard_to_gateio.insert(standard.to_string(), gateio.to_string());
        self.gateio_to_standard.insert(gateio.to_string(), standard.to_string());
    }

    pub fn to_gateio(&self, standard: &str) -> Option<&String> {
        self.standard_to_gateio.get(standard)
    }

    pub fn to_standard(&self, gateio: &str) -> Option<&String> {
        self.gateio_to_standard.get(gateio)
    }
}
```

### 3. Dynamic Symbol Loading

Fetch available symbols from API on startup:

```rust
pub async fn load_spot_symbols() -> Result<Vec<String>> {
    let response = fetch("/spot/currency_pairs").await?;
    let pairs: Vec<CurrencyPair> = parse_response(response)?;

    Ok(pairs.into_iter()
        .filter(|p| p.trade_status == "tradable")
        .map(|p| p.id)
        .collect())
}

pub async fn load_futures_symbols(settle: &str) -> Result<Vec<String>> {
    let response = fetch(&format!("/futures/{}/contracts", settle)).await?;
    let contracts: Vec<Contract> = parse_response(response)?;

    Ok(contracts.into_iter()
        .filter(|c| c.in_delisting == false)
        .map(|c| c.name)
        .collect())
}
```

---

## Testing Symbol Conversion

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_conversion() {
        assert_eq!(to_gateio_symbol("BTC/USDT"), "BTC_USDT");
        assert_eq!(to_gateio_symbol("ETH/BTC"), "ETH_BTC");

        assert_eq!(from_gateio_symbol("BTC_USDT"), "BTC/USDT");
        assert_eq!(from_gateio_symbol("ETH_BTC"), "ETH/BTC");
    }

    #[test]
    fn test_symbol_validation() {
        assert!(is_valid_gateio_symbol("BTC_USDT"));
        assert!(is_valid_gateio_symbol("ETH_BTC"));
        assert!(!is_valid_gateio_symbol("BTCUSDT"));
        assert!(!is_valid_gateio_symbol("BTC-USDT"));
        assert!(!is_valid_gateio_symbol("btc_usdt"));
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            parse_gateio_symbol("BTC_USDT"),
            Some(("BTC".to_string(), "USDT".to_string()))
        );
        assert_eq!(
            parse_gateio_symbol("ETH_BTC"),
            Some(("ETH".to_string(), "BTC".to_string()))
        );
        assert_eq!(parse_gateio_symbol("INVALID"), None);
    }
}
```

---

## Summary

### Key Points

1. **Format**: Gate.io uses `BASE_QUOTE` with **underscore** separator
2. **Consistency**: Same format for both spot and futures
3. **Case**: Use **uppercase** (API accepts lowercase but uppercase is standard)
4. **Separator**: **Underscore `_`**, not hyphen `-` or nothing
5. **Conversion**: Simple replace `/` with `_` for most cases

### Symbol Format Reference

| Component | Example | Notes |
|-----------|---------|-------|
| **Base** | BTC, ETH, SOL | Token being traded |
| **Quote** | USDT, BTC, USD | Currency used to buy/sell |
| **Separator** | `_` | Underscore (required) |
| **Case** | Uppercase | BTC_USDT, not btc_usdt |
| **Full Symbol** | `BTC_USDT` | Complete trading pair |

### Implementation Checklist

- [ ] Symbol conversion: standard format <-> Gate.io format
- [ ] Symbol validation (regex pattern matching)
- [ ] Symbol parsing (extract base and quote)
- [ ] Symbol registry (maintain mappings)
- [ ] Dynamic symbol loading from API
- [ ] Handle both spot and futures symbols
- [ ] Case normalization (always uppercase)
- [ ] Unit tests for conversions

---

## Sources

- [Gate.io Spot API Documentation](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Futures API Documentation](https://www.gate.com/docs/futures/api/index.html)
- [Gate.io API v4 Reference](http://www.gate.com/docs/apiv4/en/index.html)

---

**Research completed**: 2026-01-21
**Implementation note**: Gate.io symbol format is straightforward and consistent across spot and futures markets.
