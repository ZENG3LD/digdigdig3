# Gemini Exchange Symbol Format

Symbol formatting specification for implementing V5 connector endpoints.rs module.

---

## Symbol Format Overview

Gemini uses a **concatenated lowercase format** for trading pair symbols.

### General Pattern

```
{BASE_CURRENCY}{QUOTE_CURRENCY}
```

- **No separator**: No dash, slash, or underscore
- **All lowercase**: Even in responses, symbols are lowercase
- **Base currency first**: Cryptocurrency or asset being traded
- **Quote currency second**: Currency used for pricing

---

## Spot Trading Symbols

### Format

```
btcusd, ethusd, ethbtc, ltcusd, bchusd
```

### Common Examples

| Symbol | Base | Quote | Description |
|--------|------|-------|-------------|
| `btcusd` | BTC | USD | Bitcoin / US Dollar |
| `ethusd` | ETH | USD | Ethereum / US Dollar |
| `ethbtc` | ETH | BTC | Ethereum / Bitcoin |
| `ltcusd` | LTC | USD | Litecoin / US Dollar |
| `bchusd` | BCH | USD | Bitcoin Cash / US Dollar |
| `zecusd` | ZEC | USD | Zcash / US Dollar |
| `linkusd` | LINK | USD | Chainlink / US Dollar |
| `uniusd` | UNI | USD | Uniswap / US Dollar |
| `daiusd` | DAI | USD | Dai Stablecoin / US Dollar |
| `usdtusd` | USDT | USD | Tether / US Dollar |

### Fiat Quote Currencies

Gemini supports multiple fiat currencies:

| Quote Currency | Example Symbols |
|----------------|-----------------|
| **USD** | `btcusd`, `ethusd` |
| **EUR** | `btceur`, `etheur` |
| **GBP** | `btcgbp`, `ethgbp` |
| **SGD** | `btcsgd`, `ethsgd` |
| **AUD** | `btcaud`, `ethaud` |
| **CAD** | `btccad`, `ethcad` |
| **HKD** | `btchkd`, `ethhkd` |

### Crypto-to-Crypto Pairs

| Symbol | Base | Quote | Description |
|--------|------|-------|-------------|
| `ethbtc` | ETH | BTC | Ethereum / Bitcoin |
| `linketh` | LINK | ETH | Chainlink / Ethereum |
| `unibtc` | UNI | BTC | Uniswap / Bitcoin |

---

## Perpetual Futures Symbols

### Format

```
{BASE}gusdperp
```

- **Base currency**: Cryptocurrency
- **"gusd"**: Gemini USD (collateral)
- **"perp"**: Perpetual contract

### Examples

| Symbol | Description |
|--------|-------------|
| `btcgusdperp` | Bitcoin Perpetual (GUSD-margined) |
| `ethgusdperp` | Ethereum Perpetual (GUSD-margined) |
| `solusdperp` | Solana Perpetual |
| `linkgusdperp` | Chainlink Perpetual |
| `avaxgusdperp` | Avalanche Perpetual |

### Alternative Notation

Some perpetuals may use different suffixes:
- `{base}usdperp` - USD-margined perpetuals
- `{base}perp` - Generic perpetual notation

**Note**: Always verify current symbols via `/v1/symbols` endpoint.

---

## Symbol Casing Rules

### In Requests (User Input)

- **Accept both cases**: API accepts uppercase or lowercase
- **Normalize to lowercase**: Before sending to API

```rust
// Example normalization
fn normalize_symbol(symbol: &str) -> String {
    symbol.to_lowercase()
}

// Usage
let user_input = "BTCUSD";
let normalized = normalize_symbol(user_input); // "btcusd"
```

### In Responses (API Output)

- **Always lowercase** in most endpoints:
  ```json
  {
    "symbol": "btcusd"
  }
  ```

- **Uppercase in symbol details**:
  ```json
  {
    "symbol": "BTCUSD",
    "base_currency": "BTC",
    "quote_currency": "USD"
  }
  ```

### Display Convention

For user display, **uppercase** is conventional:
```
BTCUSD, ETHUSD, BTCGUSDPERP
```

But for API communication, use **lowercase**:
```
btcusd, ethusd, btcgusdperp
```

---

## Symbol Validation

### Getting Valid Symbols

**Endpoint**: `GET /v1/symbols`

```json
["btcusd", "ethusd", "ethbtc", "bchusd", "ltcusd", "zecusd", "btcgusdperp", ...]
```

### Symbol Details

**Endpoint**: `GET /v1/symbols/details/{symbol}`

```json
{
  "symbol": "BTCUSD",
  "base_currency": "BTC",
  "quote_currency": "USD",
  "tick_size": 1e-8,
  "quote_increment": 0.01,
  "min_order_size": "0.00001",
  "status": "open",
  "wrap_enabled": false
}
```

**Status Values**:
- `"open"`: Trading enabled
- `"closed"`: Trading disabled
- `"cancel_only"`: Can only cancel existing orders
- `"post_only"`: Only post orders (no market orders)
- `"limit_only"`: Only limit orders allowed

### Validation Logic

```rust
use std::collections::HashSet;

pub struct SymbolValidator {
    valid_symbols: HashSet<String>,
}

impl SymbolValidator {
    pub async fn new(client: &Client, base_url: &str) -> Result<Self, Error> {
        let url = format!("{}/v1/symbols", base_url);
        let symbols: Vec<String> = client.get(&url).send().await?.json().await?;

        Ok(Self {
            valid_symbols: symbols.into_iter().collect(),
        })
    }

    pub fn is_valid(&self, symbol: &str) -> bool {
        self.valid_symbols.contains(&symbol.to_lowercase())
    }

    pub fn is_perpetual(&self, symbol: &str) -> bool {
        symbol.to_lowercase().ends_with("perp")
    }

    pub fn is_spot(&self, symbol: &str) -> bool {
        !self.is_perpetual(symbol)
    }
}
```

---

## Symbol Parsing

### Extract Base and Quote

For **spot** symbols:

```rust
pub fn parse_spot_symbol(symbol: &str) -> Option<(String, String)> {
    let s = symbol.to_lowercase();

    // Common quote currencies (in order of precedence for matching)
    let quotes = ["usdt", "gusd", "usd", "eur", "gbp", "sgd", "aud", "cad", "hkd", "btc", "eth"];

    for quote in &quotes {
        if s.ends_with(quote) {
            let base = &s[..s.len() - quote.len()];
            if !base.is_empty() {
                return Some((base.to_string(), quote.to_string()));
            }
        }
    }

    None
}

// Usage
let (base, quote) = parse_spot_symbol("btcusd").unwrap();
assert_eq!(base, "btc");
assert_eq!(quote, "usd");

let (base, quote) = parse_spot_symbol("ethbtc").unwrap();
assert_eq!(base, "eth");
assert_eq!(quote, "btc");
```

For **perpetual** symbols:

```rust
pub fn parse_perp_symbol(symbol: &str) -> Option<(String, String)> {
    let s = symbol.to_lowercase();

    if s.ends_with("gusdperp") {
        let base = &s[..s.len() - 8]; // Remove "gusdperp"
        return Some((base.to_string(), "gusd".to_string()));
    }

    if s.ends_with("usdperp") {
        let base = &s[..s.len() - 7]; // Remove "usdperp"
        return Some((base.to_string(), "usd".to_string()));
    }

    if s.ends_with("perp") {
        let base = &s[..s.len() - 4]; // Remove "perp"
        return Some((base.to_string(), "".to_string()));
    }

    None
}

// Usage
let (base, quote) = parse_perp_symbol("btcgusdperp").unwrap();
assert_eq!(base, "btc");
assert_eq!(quote, "gusd");
```

---

## Symbol Construction

### Build Symbol from Components

```rust
pub fn build_spot_symbol(base: &str, quote: &str) -> String {
    format!("{}{}", base.to_lowercase(), quote.to_lowercase())
}

pub fn build_perp_symbol(base: &str, quote: &str) -> String {
    format!("{}{}perp", base.to_lowercase(), quote.to_lowercase())
}

// Usage
let symbol = build_spot_symbol("BTC", "USD");
assert_eq!(symbol, "btcusd");

let symbol = build_perp_symbol("ETH", "GUSD");
assert_eq!(symbol, "ethgusdperp");
```

---

## Symbol Formatting for API

### In URL Paths

Symbols appear in URL paths for many endpoints:

```
GET /v1/pubticker/{symbol}
GET /v1/book/{symbol}
GET /v2/candles/{symbol}/{time_frame}
```

**Format function**:

```rust
pub fn format_symbol_for_url(symbol: &str) -> String {
    symbol.to_lowercase()
}

// Usage
let url = format!("/v1/pubticker/{}", format_symbol_for_url("BTCUSD"));
// "/v1/pubticker/btcusd"
```

### In JSON Payloads

For private endpoints, symbols in payload are also lowercase:

```json
{
  "request": "/v1/order/new",
  "nonce": 1640000000000,
  "symbol": "btcusd",
  "amount": "0.5",
  "price": "50000.00",
  "side": "buy"
}
```

### In WebSocket Subscriptions

WebSocket subscriptions can use **uppercase** symbols:

```json
{
  "type": "subscribe",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["BTCUSD", "ETHUSD", "ETHBTC"]
    }
  ]
}
```

**Note**: WebSocket market data v2 accepts uppercase, but responses use uppercase in the `symbol` field.

---

## Instrument Types

### Identifying Instrument Type

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentType {
    Spot,
    Perpetual,
}

pub fn get_instrument_type(symbol: &str) -> InstrumentType {
    if symbol.to_lowercase().ends_with("perp") {
        InstrumentType::Perpetual
    } else {
        InstrumentType::Spot
    }
}

// Usage
assert_eq!(get_instrument_type("btcusd"), InstrumentType::Spot);
assert_eq!(get_instrument_type("btcgusdperp"), InstrumentType::Perpetual);
```

### From API Response

Symbol details response includes `product_type`:

```json
{
  "symbol": "BTCGUSDPERP",
  "product_type": "cfd"
}
```

Possible `product_type` values:
- Spot symbols: No `product_type` or empty
- Perpetuals: `"cfd"` (Contract for Difference)
- Futures: `"future"` (if supported in the future)

---

## Common Symbol Patterns

### USD-Denominated Crypto

Pattern: `{crypto}usd`

```
btcusd, ethusd, ltcusd, bchusd, linkusd, uniusd, aaveusd, maticusd
```

### GUSD-Margined Perpetuals

Pattern: `{crypto}gusdperp`

```
btcgusdperp, ethgusdperp, linkgusdperp, uniusdperp
```

### Stablecoin Pairs

Pattern: `{stablecoin}usd`

```
daiusd, usdtusd, usdcusd, gusd usd
```

### Cross-Crypto Pairs

Pattern: `{crypto1}{crypto2}`

```
ethbtc, linketh, unibtc, aaveeth
```

---

## Symbol Normalization for V5 Connector

### Recommended Approach

```rust
pub struct SymbolFormatter;

impl SymbolFormatter {
    /// Normalize symbol to Gemini format (lowercase)
    pub fn normalize(symbol: &str) -> String {
        symbol.to_lowercase()
    }

    /// Format symbol for URL path
    pub fn for_url(symbol: &str) -> String {
        Self::normalize(symbol)
    }

    /// Format symbol for JSON payload
    pub fn for_payload(symbol: &str) -> String {
        Self::normalize(symbol)
    }

    /// Format symbol for WebSocket subscription (uppercase)
    pub fn for_websocket(symbol: &str) -> String {
        symbol.to_uppercase()
    }

    /// Format symbol for display to user
    pub fn for_display(symbol: &str) -> String {
        symbol.to_uppercase()
    }

    /// Check if symbol is valid format
    pub fn is_valid_format(symbol: &str) -> bool {
        // Basic validation: alphanumeric only
        symbol.chars().all(|c| c.is_alphanumeric())
    }
}
```

---

## Special Cases

### Wrapped Tokens

Some symbols support "wrap" trading (converting between ETH and WETH, for example).

Symbols with `wrap_enabled: true` in symbol details support this.

### Prediction Markets

Gemini also has prediction market symbols (different API):
```
Pattern: {event}_{outcome}
Example: election2024_yes
```

**Note**: Prediction markets use a separate API and are not covered in standard trading endpoints.

---

## Symbol Lists by Category

### Major Pairs (High Liquidity)

```
btcusd, ethusd, ltcusd, bchusd, zecusd
```

### Stablecoins

```
daiusd, usdtusd, usdcusd, paxusd, busdusd
```

### DeFi Tokens

```
uniusd, aaveusd, linkusd, compusd, snxusd, makerdusd
```

### Layer 1 Blockchains

```
ethusd, solusd, adausd, dotusd, avaxusd
```

### Perpetual Futures

```
btcgusdperp, ethgusdperp, solgusdperp, linkgusdperp
```

---

## Testing Symbol Handling

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(SymbolFormatter::normalize("BTCUSD"), "btcusd");
        assert_eq!(SymbolFormatter::normalize("btcusd"), "btcusd");
        assert_eq!(SymbolFormatter::normalize("BtCuSd"), "btcusd");
    }

    #[test]
    fn test_parse_spot_symbol() {
        let (base, quote) = parse_spot_symbol("btcusd").unwrap();
        assert_eq!(base, "btc");
        assert_eq!(quote, "usd");

        let (base, quote) = parse_spot_symbol("ethbtc").unwrap();
        assert_eq!(base, "eth");
        assert_eq!(quote, "btc");

        let (base, quote) = parse_spot_symbol("usdtusd").unwrap();
        assert_eq!(base, "usdt");
        assert_eq!(quote, "usd");
    }

    #[test]
    fn test_parse_perp_symbol() {
        let (base, quote) = parse_perp_symbol("btcgusdperp").unwrap();
        assert_eq!(base, "btc");
        assert_eq!(quote, "gusd");

        let (base, quote) = parse_perp_symbol("ethusdperp").unwrap();
        assert_eq!(base, "eth");
        assert_eq!(quote, "usd");
    }

    #[test]
    fn test_instrument_type() {
        assert_eq!(get_instrument_type("btcusd"), InstrumentType::Spot);
        assert_eq!(get_instrument_type("btcgusdperp"), InstrumentType::Perpetual);
    }

    #[test]
    fn test_build_symbol() {
        assert_eq!(build_spot_symbol("BTC", "USD"), "btcusd");
        assert_eq!(build_perp_symbol("ETH", "GUSD"), "ethgusdperp");
    }
}
```

---

## Implementation in endpoints.rs

### Expected Structure

```rust
pub struct GeminiEndpoints;

impl GeminiEndpoints {
    /// Format symbol for use in endpoint URLs
    pub fn format_symbol(symbol: &str) -> String {
        symbol.to_lowercase()
    }

    /// Get ticker URL with formatted symbol
    pub fn ticker(symbol: &str) -> String {
        format!("/v1/pubticker/{}", Self::format_symbol(symbol))
    }

    /// Get orderbook URL with formatted symbol
    pub fn orderbook(symbol: &str) -> String {
        format!("/v1/book/{}", Self::format_symbol(symbol))
    }

    /// Get candles URL with formatted symbol
    pub fn candles(symbol: &str, timeframe: &str) -> String {
        format!("/v2/candles/{}/{}", Self::format_symbol(symbol), timeframe)
    }
}
```

---

## Summary

| Aspect | Value |
|--------|-------|
| **Format** | Lowercase, no separator |
| **Spot Pattern** | `{base}{quote}` (e.g., `btcusd`) |
| **Perpetual Pattern** | `{base}gusdperp` or `{base}usdperp` |
| **API Requests** | Always lowercase |
| **API Responses** | Usually lowercase, sometimes uppercase in details |
| **WebSocket** | Can use uppercase |
| **Display** | Uppercase conventional |
| **Validation** | Via `/v1/symbols` endpoint |
| **Supported Quotes** | USD, EUR, GBP, SGD, AUD, CAD, HKD, BTC, ETH, USDT, GUSD |

---

## References

- Symbol List: https://docs.gemini.com/rest/market-data (GET /v1/symbols)
- Symbol Details: https://docs.gemini.com/rest/market-data (GET /v1/symbols/details/{symbol})
- Trading Pairs: https://www.gemini.com/prices
