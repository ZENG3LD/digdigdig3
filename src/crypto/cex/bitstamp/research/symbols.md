# Bitstamp Symbol Format and Trading Pairs

This document describes how Bitstamp formats trading pair symbols and currency codes.

---

## Symbol Format

Bitstamp uses **lowercase symbols without separators** for trading pairs.

### Format Pattern

```
{base}{quote}
```

**Examples**:
- Bitcoin/USD: `btcusd`
- Bitcoin/EUR: `btceur`
- Ethereum/USD: `ethusd`
- Ethereum/BTC: `ethbtc`
- Ripple/USD: `xrpusd`

### Characteristics

- **All lowercase**: No uppercase letters
- **No separators**: No hyphens, underscores, or slashes
- **Concatenated**: Base currency directly followed by quote currency
- **Consistent length**: Typically 6 characters (3 + 3)

---

## API Symbol Usage

### REST API Endpoints

Symbols are used in the URL path:

```
GET /api/v2/ticker/btcusd/
GET /api/v2/order_book/etheur/
POST /api/v2/buy/xrpusd/
```

### Response Format

API responses include both the URL symbol and formatted display symbol:

```json
{
  "market_symbol": "btcusd",
  "pair": "BTC/USD",
  "base_currency": "BTC",
  "counter_currency": "USD"
}
```

**Field Differences**:
- `market_symbol`: Lowercase, no separator (`btcusd`) - used in API URLs
- `pair`: Uppercase with slash (`BTC/USD`) - display format
- `base_currency`: Uppercase base currency code (`BTC`)
- `counter_currency`: Uppercase quote currency code (`USD`)

---

## WebSocket Channels

WebSocket channels use the same lowercase format:

```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_trades_btcusd"
  }
}
```

**Channel Format**: `{channel_type}_{pair}`

**Examples**:
- `live_trades_btcusd`
- `order_book_etheur`
- `diff_order_book_xrpusd`

---

## Currency Codes

### Base Currencies (Cryptocurrencies)

Common cryptocurrencies on Bitstamp:

| Code | Name | Example Pairs |
|------|------|---------------|
| `btc` | Bitcoin | `btcusd`, `btceur` |
| `eth` | Ethereum | `ethusd`, `etheur`, `ethbtc` |
| `xrp` | Ripple | `xrpusd`, `xrpeur`, `xrpbtc` |
| `ltc` | Litecoin | `ltcusd`, `ltceur`, `ltcbtc` |
| `bch` | Bitcoin Cash | `bchusd`, `bcheur`, `bchbtc` |
| `xlm` | Stellar | `xlmusd`, `xlmeur`, `xlmbtc` |
| `link` | Chainlink | `linkusd`, `linkeur`, `linkbtc` |
| `uni` | Uniswap | `uniusd`, `unieur` |
| `ada` | Cardano | `adausd`, `adaeur`, `adabtc` |
| `dot` | Polkadot | `dotusd`, `doteur` |
| `sol` | Solana | `solusd`, `soleur` |
| `avax` | Avalanche | `avaxusd`, `avaxeur` |
| `matic` | Polygon | `maticusd`, `maticeur` |

### Quote Currencies (Fiat & Stablecoins)

Common quote currencies:

| Code | Name | Type |
|------|------|------|
| `usd` | US Dollar | Fiat |
| `eur` | Euro | Fiat |
| `gbp` | British Pound | Fiat |
| `usdc` | USD Coin | Stablecoin |
| `usdt` | Tether | Stablecoin |
| `btc` | Bitcoin | Crypto (when used as quote) |

---

## Trading Pairs Information

### Getting Available Pairs

**Endpoint**: `GET /api/v2/markets/`

Returns all available trading pairs with details:

```json
[
  {
    "trading": "Enabled",
    "base_decimals": 8,
    "counter_decimals": 2,
    "instant_order_counter_decimals": 2,
    "minimum_order": "10.0 USD",
    "market_symbol": "btcusd",
    "base_currency": "BTC",
    "counter_currency": "USD",
    "url_symbol": "btcusd",
    "description": "Bitcoin / U.S. dollar"
  }
]
```

### Pair Details

- **trading**: `"Enabled"` or `"Disabled"`
- **base_decimals**: Decimal precision for base currency (e.g., 8 for BTC)
- **counter_decimals**: Decimal precision for quote currency (e.g., 2 for USD)
- **minimum_order**: Minimum order size (formatted string)
- **market_symbol**: Symbol used in API calls
- **url_symbol**: Symbol for URL paths (same as market_symbol)

---

## Symbol Parsing

### Breaking Down a Symbol

To parse a Bitstamp symbol:

1. **Know the currencies**: Bitstamp doesn't provide explicit delimiters
2. **Common pattern**: First 3-4 characters = base, remaining = quote
3. **Use markets endpoint**: Always fetch available pairs from `/api/v2/markets/`

**Example**:
- `btcusd` → base: `btc`, quote: `usd`
- `ethusd` → base: `eth`, quote: `usd`
- `linkeur` → base: `link`, quote: `eur` (4 chars + 3 chars)

### Edge Cases

Some symbols may have irregular lengths:
- `linkusd` (4 + 3 = 7 characters)
- `maticusd` (5 + 3 = 8 characters)

**Best Practice**: Always use the `/api/v2/markets/` endpoint to get the canonical list of pairs with their base and quote currencies explicitly defined.

---

## Converting Between Formats

### From Standard Format to Bitstamp Format

```
BTC/USD  →  btcusd
ETH/EUR  →  etheur
XRP/BTC  →  xrpbtc
```

**Algorithm**:
1. Split on `/`
2. Convert both parts to lowercase
3. Concatenate without separator

### From Bitstamp Format to Standard Format

**Without Markets Endpoint** (risky):
```rust
// Assumes 3-char base and quote (not always correct!)
let base = &symbol[0..3].to_uppercase();
let quote = &symbol[3..6].to_uppercase();
let standard = format!("{}/{}", base, quote);
```

**With Markets Endpoint** (recommended):
```rust
// Fetch from /api/v2/markets/ and build a map
let pair_info = markets.get("btcusd").unwrap();
let standard = format!("{}/{}", pair_info.base_currency, pair_info.counter_currency);
// Result: "BTC/USD"
```

---

## Symbol Validation

### Valid Symbol Characteristics

- All lowercase letters
- No special characters (no `-`, `_`, `/`, or spaces)
- Typically 6-8 characters long
- Comprised of known currency codes

### Validation Example

```rust
fn is_valid_bitstamp_symbol(symbol: &str) -> bool {
    symbol.chars().all(|c| c.is_ascii_lowercase())
        && symbol.len() >= 6
        && symbol.len() <= 10
}
```

**Better Validation**:
Fetch available pairs from `/api/v2/markets/` and validate against that list.

---

## Special Symbols

### Fiat Pairs

Some pairs are pure fiat (not crypto):
- `eurusd`: EUR/USD exchange rate

### Inverse Pairs

Bitstamp typically lists pairs with crypto as base and fiat as quote:
- Standard: `btcusd` (BTC is base)
- Rare: `usdbtc` (USD is base) - usually not available

### Stablecoin Pairs

- `btcusdc`: BTC/USDC
- `ethusdt`: ETH/USDT

---

## Getting All Trading Pairs

### Fetch Markets

```bash
curl https://www.bitstamp.net/api/v2/markets/
```

**Response**: Array of all available trading pairs with metadata.

### Parsing Markets Response

```json
[
  {
    "market_symbol": "btcusd",
    "base_currency": "BTC",
    "counter_currency": "USD"
  },
  {
    "market_symbol": "etheur",
    "base_currency": "ETH",
    "counter_currency": "EUR"
  }
]
```

Build a symbol map:
```rust
let mut symbol_map = HashMap::new();
for market in markets {
    symbol_map.insert(
        market.market_symbol.clone(),
        (market.base_currency, market.counter_currency)
    );
}
```

---

## Symbol Normalization

### For Connector Implementation

When implementing the V5 connector, normalize symbols:

1. **Input**: Accept both formats (`BTC/USD` or `btcusd`)
2. **Internal**: Convert to Bitstamp format (`btcusd`)
3. **Validation**: Check against markets list
4. **API Calls**: Use lowercase format
5. **Output**: Convert back to standard format if needed

### Example Normalization

```rust
fn normalize_symbol(input: &str) -> Result<String, Error> {
    // Remove slashes and convert to lowercase
    let normalized = input.replace("/", "").to_lowercase();

    // Validate against known pairs (fetch from /api/v2/markets/)
    if !is_valid_pair(&normalized) {
        return Err(Error::InvalidSymbol);
    }

    Ok(normalized)
}

// BTC/USD → btcusd
// btcusd → btcusd
// BTCUSD → btcusd
```

---

## Common Pairs

### Top Volume Pairs (as of 2026)

1. `btcusd` - Bitcoin/US Dollar
2. `btceur` - Bitcoin/Euro
3. `ethusd` - Ethereum/US Dollar
4. `etheur` - Ethereum/Euro
5. `xrpusd` - Ripple/US Dollar
6. `ltcusd` - Litecoin/US Dollar

### Crypto-to-Crypto Pairs

- `ethbtc` - Ethereum/Bitcoin
- `xrpbtc` - Ripple/Bitcoin
- `ltcbtc` - Litecoin/Bitcoin
- `bchbtc` - Bitcoin Cash/Bitcoin

---

## Symbol Format Summary

| Aspect | Format |
|--------|--------|
| **Case** | Lowercase |
| **Separator** | None |
| **Pattern** | `{base}{quote}` |
| **Example** | `btcusd` |
| **API Usage** | URL paths, channel names |
| **Display Format** | `BTC/USD` (with slash, uppercase) |
| **Length** | 6-10 characters typically |

---

## Implementation Notes

### For V5 Connector

1. **Store Symbol Map**: Fetch `/api/v2/markets/` on initialization
2. **Symbol Conversion**: Implement helper functions:
   - `to_bitstamp_format(symbol: &str) -> String`
   - `from_bitstamp_format(symbol: &str) -> String`
3. **Validation**: Always validate symbols against the markets list
4. **Caching**: Cache the markets list (refresh periodically)
5. **Error Handling**: Handle invalid/unsupported symbols gracefully

### Example Helper

```rust
pub struct BitstampSymbolConverter {
    markets: HashMap<String, (String, String)>,
}

impl BitstampSymbolConverter {
    pub fn to_bitstamp(&self, symbol: &str) -> Result<String, Error> {
        let normalized = symbol.replace("/", "").to_lowercase();
        if self.markets.contains_key(&normalized) {
            Ok(normalized)
        } else {
            Err(Error::UnsupportedSymbol)
        }
    }

    pub fn from_bitstamp(&self, symbol: &str) -> Result<String, Error> {
        if let Some((base, quote)) = self.markets.get(symbol) {
            Ok(format!("{}/{}", base, quote))
        } else {
            Err(Error::UnknownSymbol)
        }
    }
}
```

---

## Reference

- **Trading Pairs**: https://www.bitstamp.net/markets/
- **Markets API**: `GET /api/v2/markets/`
- **Currency Codes**: ISO 4217 (for fiat), ticker symbols (for crypto)
