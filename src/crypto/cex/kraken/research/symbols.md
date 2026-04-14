# Kraken API Symbol Formats

Kraken uses different symbol naming conventions across its REST API, WebSocket APIs (v1 and v2), and between Spot and Futures markets. Understanding these formats is critical for proper API integration.

---

## Symbol Naming Conventions

### Spot REST API Symbols

Kraken Spot REST API uses an extended ISO 4217-A3 format with prefixes:

#### Prefix Convention
- **X prefix**: Cryptocurrencies
- **Z prefix**: Fiat currencies

#### Common Asset Codes

**Cryptocurrencies**:
- `XBT`: Bitcoin (BTC)
- `XETH`: Ethereum (ETH)
- `XLTC`: Litecoin (LTC)
- `XXRP`: Ripple (XRP)
- `XDOGE`: Dogecoin (DOGE)
- `XXMR`: Monero (XMR)

**Fiat Currencies**:
- `ZUSD`: US Dollar
- `ZEUR`: Euro
- `ZGBP`: British Pound
- `ZJPY`: Japanese Yen
- `ZCAD`: Canadian Dollar

#### Trading Pair Format

Trading pairs combine base and quote assets:

- **XXBTZUSD**: Bitcoin/US Dollar
- **XETHZUSD**: Ethereum/US Dollar
- **XXBTZEUR**: Bitcoin/Euro
- **XETHXXBT**: Ethereum/Bitcoin

**Important Quirk**:
- Request parameter: `pair=XBTUSD` (simplified)
- Response key: `"XXBTZUSD"` (full ISO format)

Example:
```bash
# Request
GET /0/public/Ticker?pair=XBTUSD

# Response
{
  "result": {
    "XXBTZUSD": { ... }  // Note the double XX
  }
}
```

---

### WebSocket v1 Symbols

WebSocket v1 uses ISO 4217-A3 format with **forward slash separator**:

- `XBT/USD`: Bitcoin/US Dollar
- `ETH/USD`: Ethereum/US Dollar
- `XBT/EUR`: Bitcoin/Euro
- `ETH/XBT`: Ethereum/Bitcoin

**Key Differences from REST**:
- Uses `/` separator
- Still uses `XBT` for Bitcoin (not `BTC`)
- Matches wsname field from AssetPairs endpoint

---

### WebSocket v2 Symbols

WebSocket v2 modernized the symbol format:

- `BTC/USD`: Bitcoin/US Dollar (uses BTC instead of XBT!)
- `ETH/USD`: Ethereum/US Dollar
- `MATIC/GBP`: Polygon/British Pound

**Major Change**:
- **BTC** is used instead of **XBT** for Bitcoin
- More readable, standard cryptocurrency symbols
- Still uses `/` separator

---

### Futures Symbols

Futures use different naming conventions:

#### Perpetual Futures
- `PI_XBTUSD`: Bitcoin perpetual (inverse)
- `PF_XBTUSD`: Bitcoin perpetual (linear)
- `PI_ETHUSD`: Ethereum perpetual

**Format**: `{Type}_{Base}{Quote}`
- `PI`: Perpetual Inverse
- `PF`: Perpetual Forward/Linear
- `FI`: Fixed maturity Inverse
- `FF`: Fixed maturity Forward

#### Fixed Maturity Futures
- `FI_XBTUSD_210625`: Bitcoin futures expiring June 25, 2021
- Format: `{Type}_{Base}{Quote}_{YYMMDD}`

---

## Symbol Translation

### REST API Symbol Handling

The REST API accepts **multiple formats** in requests but returns data with **full ISO format** keys.

**Request Formats Accepted**:
- Simplified: `XBTUSD`
- Full: `XXBTZUSD`
- WebSocket: `XBT/USD` (for some endpoints)

**Response Format**:
- Always full ISO: `XXBTZUSD`

### Getting Symbol Mappings

Use the **AssetPairs** endpoint to get all symbol variations:

**Endpoint**: `GET /0/public/AssetPairs`

**Response**:
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": {
      "altname": "XBTUSD",
      "wsname": "XBT/USD",
      "aclass_base": "currency",
      "base": "XXBT",
      "aclass_quote": "currency",
      "quote": "ZUSD",
      "pair_decimals": 1,
      "lot_decimals": 8,
      "lot_multiplier": 1,
      "leverage_buy": [2, 3, 4, 5],
      "leverage_sell": [2, 3, 4, 5],
      "fees": [[0, 0.26], [50000, 0.24]],
      "fees_maker": [[0, 0.16], [50000, 0.14]],
      "fee_volume_currency": "ZUSD",
      "margin_call": 80,
      "margin_stop": 40,
      "ordermin": "0.0001",
      "costmin": "0.5"
    }
  }
}
```

**Key Fields**:
- `altname`: Simplified name used in requests (`XBTUSD`)
- `wsname`: WebSocket v1 name (`XBT/USD`)
- `base`: Base asset with prefix (`XXBT`)
- `quote`: Quote asset with prefix (`ZUSD`)

---

## Symbol Format Comparison Table

| Market | REST Request | REST Response | WS v1 | WS v2 | Futures |
|--------|--------------|---------------|-------|-------|---------|
| BTC/USD | `XBTUSD` or `XXBTZUSD` | `XXBTZUSD` | `XBT/USD` | `BTC/USD` | `PI_XBTUSD` |
| ETH/USD | `ETHUSD` or `XETHZUSD` | `XETHZUSD` | `ETH/USD` | `ETH/USD` | `PI_ETHUSD` |
| BTC/EUR | `XBTEUR` or `XXBTZEUR` | `XXBTZEUR` | `XBT/EUR` | `BTC/EUR` | `PI_XBTEUR` |
| ETH/BTC | `ETHXBT` or `XETHXXBT` | `XETHXXBT` | `ETH/XBT` | `ETH/BTC` | - |

---

## Implementation Guidelines

### For REST API Integration

```rust
// Store mapping from simplified to full symbol
let symbol_map: HashMap<&str, &str> = [
    ("XBTUSD", "XXBTZUSD"),
    ("ETHUSD", "XETHZUSD"),
    ("XBTEUR", "XXBTZEUR"),
    // ...
].iter().cloned().collect();

// Request with simplified symbol
let request_symbol = "XBTUSD";
let response = get_ticker(request_symbol).await?;

// Parse response with full symbol
let response_symbol = "XXBTZUSD";
let ticker = response["result"][response_symbol].clone();
```

### For WebSocket v1 Integration

```rust
// Convert REST symbol to WebSocket format
fn rest_to_ws_v1(rest_symbol: &str) -> String {
    match rest_symbol {
        "XXBTZUSD" | "XBTUSD" => "XBT/USD",
        "XETHZUSD" | "ETHUSD" => "ETH/USD",
        "XXBTZEUR" | "XBTEUR" => "XBT/EUR",
        _ => rest_symbol, // Fallback
    }.to_string()
}

// Subscribe to ticker
let ws_symbol = rest_to_ws_v1("XBTUSD");
// ws_symbol = "XBT/USD"
```

### For WebSocket v2 Integration

```rust
// Convert REST symbol to WebSocket v2 format
fn rest_to_ws_v2(rest_symbol: &str) -> String {
    match rest_symbol {
        "XXBTZUSD" | "XBTUSD" => "BTC/USD",  // Note: BTC not XBT!
        "XETHZUSD" | "ETHUSD" => "ETH/USD",
        "XXBTZEUR" | "XBTEUR" => "BTC/EUR",
        _ => rest_symbol,
    }.to_string()
}

// Subscribe to ticker
let ws_symbol = rest_to_ws_v2("XBTUSD");
// ws_symbol = "BTC/USD"
```

### Dynamic Symbol Resolution

**Best Practice**: Query AssetPairs endpoint on initialization:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AssetPairInfo {
    altname: String,
    wsname: String,
    base: String,
    quote: String,
    pair_decimals: u8,
    lot_decimals: u8,
    ordermin: String,
    // ... other fields
}

#[derive(Debug, Deserialize)]
struct AssetPairsResponse {
    error: Vec<String>,
    result: HashMap<String, AssetPairInfo>,
}

async fn load_symbol_mappings() -> Result<HashMap<String, SymbolInfo>, Error> {
    let response: AssetPairsResponse = reqwest::get(
        "https://api.kraken.com/0/public/AssetPairs"
    )
    .await?
    .json()
    .await?;

    let mut mappings = HashMap::new();

    for (full_name, info) in response.result {
        mappings.insert(info.altname.clone(), SymbolInfo {
            full_name: full_name.clone(),
            alt_name: info.altname,
            ws_name: info.wsname,
            base: info.base,
            quote: info.quote,
        });
    }

    Ok(mappings)
}

struct SymbolInfo {
    full_name: String,    // XXBTZUSD
    alt_name: String,     // XBTUSD
    ws_name: String,      // XBT/USD
    base: String,         // XXBT
    quote: String,        // ZUSD
}
```

---

## Common Pitfalls

### 1. XBT vs BTC Confusion

**Problem**: Using `BTC` when API expects `XBT`

**Solution**:
- REST API and WebSocket v1: Use `XBT`
- WebSocket v2: Use `BTC`
- Futures: Use `XBT`

### 2. Request vs Response Symbol Mismatch

**Problem**: Requesting `XBTUSD` but looking for `XBTUSD` in response

```rust
// WRONG
let ticker = response["result"]["XBTUSD"]; // Will be null!

// CORRECT
let ticker = response["result"]["XXBTZUSD"];
```

### 3. Hardcoded Symbol Mappings

**Problem**: Hardcoded mappings break when Kraken adds new pairs

**Solution**: Dynamically load from AssetPairs endpoint

### 4. WebSocket Version Confusion

**Problem**: Using `XBT/USD` with WebSocket v2

**Solution**: Track which WebSocket version you're using:
- v1: `XBT/USD`
- v2: `BTC/USD`

---

## Special Cases

### Stablecoins

Many stablecoins don't use prefixes:

- `USDT`: Tether (no prefix)
- `USDC`: USD Coin (no prefix)
- `DAI`: Dai (no prefix)

Pairs:
- `USDTZUSD`: Tether/USD
- `USDCUSD`: USDC/USD (response: `USDCUSD`)

### Staking/Earning Assets

Assets with balance extensions:

- `ETH2`: Ethereum 2.0 staking
- `ETH2.S`: Ethereum 2.0 staked
- Asset codes with `.F`, `.B`, `.T` extensions (see response_formats.md)

---

## Futures Symbol Details

### Product ID Prefixes

| Prefix | Description | Collateral | Settlement |
|--------|-------------|------------|------------|
| `PI_` | Perpetual Inverse | Cryptocurrency | Non-linear |
| `PF_` | Perpetual Forward | Stablecoin | Linear |
| `FI_` | Fixed Inverse | Cryptocurrency | Non-linear |
| `FF_` | Fixed Forward | Stablecoin | Linear |

### Examples

```
PI_XBTUSD   - Bitcoin Perpetual Inverse (BTC collateral)
PF_XBTUSD   - Bitcoin Perpetual Linear (USD collateral)
PI_ETHUSD   - Ethereum Perpetual Inverse
FI_XBTUSD_250328 - BTC Inverse futures expiring March 28, 2025
```

---

## Testing Symbol Resolution

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_conversion() {
        // REST API behavior
        assert_eq!(normalize_rest_request("XBTUSD"), "XBTUSD");
        assert_eq!(normalize_rest_response("XBTUSD"), "XXBTZUSD");

        // WebSocket v1
        assert_eq!(rest_to_ws_v1("XBTUSD"), "XBT/USD");

        // WebSocket v2
        assert_eq!(rest_to_ws_v2("XBTUSD"), "BTC/USD");

        // Futures
        assert_eq!(rest_to_futures("XBTUSD", "perpetual"), "PI_XBTUSD");
    }

    #[test]
    fn test_symbol_roundtrip() {
        let rest_request = "XBTUSD";
        let rest_response = "XXBTZUSD";
        let ws_v1 = "XBT/USD";
        let ws_v2 = "BTC/USD";

        assert_eq!(ws_v1_to_rest(ws_v1), rest_response);
        assert_eq!(ws_v2_to_rest(ws_v2), rest_response);
    }
}
```

---

## Summary

| API Type | Symbol Format | Bitcoin Example | Separator |
|----------|---------------|----------------|-----------|
| Spot REST (request) | Simplified or Full | `XBTUSD` or `XXBTZUSD` | None |
| Spot REST (response) | Full ISO with prefixes | `XXBTZUSD` | None |
| WebSocket v1 | ISO with slash | `XBT/USD` | `/` |
| WebSocket v2 | Modern with slash | `BTC/USD` | `/` |
| Futures | Product type prefix | `PI_XBTUSD` | `_` |

**Key Takeaways**:
1. Always use AssetPairs endpoint for accurate symbol mappings
2. REST responses use full ISO format (XXBTZUSD)
3. WebSocket v2 uses BTC instead of XBT for Bitcoin
4. Futures have product-type prefixes (PI_, PF_, etc.)
5. Symbol format varies between request and response in Spot REST
