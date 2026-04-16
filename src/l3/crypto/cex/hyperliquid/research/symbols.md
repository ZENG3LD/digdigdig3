# HyperLiquid Symbols and Asset IDs

HyperLiquid uses different naming and ID conventions for perpetuals and spot markets.

---

## Symbol Format Overview

| Market Type | Symbol Format | Asset ID Format | Example |
|-------------|---------------|-----------------|---------|
| Perpetuals (Main DEX) | Coin name | Integer index | "BTC" → 0 |
| Perpetuals (Builder DEX) | `dex:COIN` | `100000 + dex_idx*10000 + coin_idx` | "xyz:ABC" → 110000 |
| Spot (PURR/USDC) | `PURR/USDC` or `@0` | `10000` | "@0" → 10000 |
| Spot (Other pairs) | `@{index}` | `10000 + index` | "@107" → 10107 |

---

## Perpetuals

### Main DEX Perpetuals

#### Symbol Format
Direct coin name:
- `"BTC"`
- `"ETH"`
- `"SOL"`
- `"AVAX"`
- etc.

#### Asset ID (Integer)
Index from the `meta` info response:
- BTC = `0` (mainnet)
- ETH = `1` (mainnet)
- SOL = `2` (mainnet)
- etc.

**Important**: Asset indices may differ between mainnet and testnet!

#### Example Mapping
```json
{
  "type": "meta",
  "dex": ""
}
```

Response:
```json
{
  "universe": [
    {"name": "BTC", "szDecimals": 5, ...},  // index 0
    {"name": "ETH", "szDecimals": 4, ...},  // index 1
    {"name": "SOL", "szDecimals": 3, ...}   // index 2
  ]
}
```

**Usage**:
- **Info endpoints**: Use coin name (`"BTC"`)
- **Exchange endpoints**: Use integer asset ID (`0`)

---

### Builder-Deployed Perpetuals (HIP-3)

#### Symbol Format
Format: `"{dex}:{coin}"`

Examples:
- `"xyz:ABC"`
- `"test:BTC100"`
- `"builder:TOKEN"`

#### Asset ID Calculation
```
asset_id = 100000 + (perp_dex_index * 10000) + index_in_meta
```

**Example**:
- DEX: `"test"` with `perp_dex_index = 1`
- Coin: `"ABC"` at `index_in_meta = 0`
- Asset ID: `100000 + (1 * 10000) + 0 = 110000`

#### Identifying Builder DEXs
Query available DEXs:
```json
{
  "type": "perpDexs"
}
```

Response:
```json
[
  {
    "name": "test",
    "perpDexIndex": 1,
    "deployer": "0x...",
    "oracleUpdater": "0x..."
  }
]
```

---

## Spot Markets

### Special Case: PURR/USDC

The first spot pair (index 0) can be referenced two ways:
- `"PURR/USDC"` (name format)
- `"@0"` (index format)

**Asset ID**: `10000` (always for index 0)

---

### Other Spot Pairs

#### Symbol Format
Format: `"@{index}"`

Examples:
- `"@1"` (second spot pair)
- `"@107"` (108th spot pair, e.g., HYPE/USDC on mainnet)
- `"@50"` (51st spot pair)

#### Asset ID Calculation
```
spot_asset_id = 10000 + spot_index
```

**Examples**:
- `@1` → Asset ID `10001`
- `@107` → Asset ID `10107`
- `@0` → Asset ID `10000`

---

### Spot Pair Details

#### Retrieving Spot Metadata
```json
{
  "type": "spotMeta"
}
```

Response:
```json
{
  "universe": [
    {
      "tokens": [150, 0],
      "name": "HYPE/USDC",
      "index": 107,
      "isCanonical": true
    },
    {
      "tokens": [1, 0],
      "name": "PURR/USDC",
      "index": 0,
      "isCanonical": true
    }
  ],
  "tokens": [
    {
      "name": "USDC",
      "szDecimals": 8,
      "weiDecimals": 6,
      "index": 0,
      "tokenId": "0x6d1e7cde53ba9467b783cb7c530ce054",
      "isCanonical": true
    },
    {
      "name": "HYPE",
      "szDecimals": 8,
      "weiDecimals": 18,
      "index": 150,
      "tokenId": "0x...",
      "isCanonical": true
    }
  ]
}
```

#### Understanding Token vs Spot Index

**Token Index**: Position in `tokens` array
**Spot Index**: Position in `universe` array

**Example (HYPE on Mainnet)**:
- Token index: `150` (HYPE token)
- Quote token: `0` (USDC)
- Spot pair index: `107` (HYPE/USDC pair)
- Spot pair tokens: `[150, 0]` (base: HYPE, quote: USDC)

**Critical**: Spot ID ≠ Token ID!

---

### Finding Spot Index

#### By Pair Name
Search `universe` for matching `name`:
```rust
let spot_meta = get_spot_meta().await?;
let pair = spot_meta.universe.iter()
    .find(|p| p.name == "HYPE/USDC")
    .ok_or(PairNotFound)?;
let spot_index = pair.index;  // 107
let asset_id = 10000 + spot_index;  // 10107
```

#### By Token Indices
Search `universe` for matching `tokens`:
```rust
let base_token = 150;  // HYPE
let quote_token = 0;   // USDC

let pair = spot_meta.universe.iter()
    .find(|p| p.tokens == vec![base_token, quote_token])
    .ok_or(PairNotFound)?;
let spot_index = pair.index;
```

---

## Symbol Normalization

### Input Symbol → API Format

#### Perpetuals
```rust
fn normalize_perp_symbol(symbol: &str) -> String {
    // User input: "BTC-PERP", "BTC/USDT:PERP", "BTCPERP"
    // API expects: "BTC"

    symbol
        .replace("-PERP", "")
        .replace("/USDT:PERP", "")
        .replace("PERP", "")
        .replace("/", "")
        .to_uppercase()
}
```

#### Spot
```rust
fn normalize_spot_symbol(symbol: &str, spot_meta: &SpotMeta) -> Result<String> {
    // User input: "HYPE/USDC", "HYPEUSDC", "HYPE-USDC"
    // API expects: "@107" or "HYPE/USDC"

    // Try exact match first
    if let Some(pair) = spot_meta.universe.iter().find(|p| p.name == symbol) {
        return Ok(format!("@{}", pair.index));
    }

    // Parse base/quote
    let (base, quote) = parse_pair(symbol)?;

    // Find tokens
    let base_token = spot_meta.tokens.iter()
        .find(|t| t.name == base)
        .ok_or(TokenNotFound)?;
    let quote_token = spot_meta.tokens.iter()
        .find(|t| t.name == quote)
        .ok_or(TokenNotFound)?;

    // Find pair
    let pair = spot_meta.universe.iter()
        .find(|p| p.tokens == vec![base_token.index, quote_token.index])
        .ok_or(PairNotFound)?;

    Ok(format!("@{}", pair.index))
}
```

---

## Symbol to Asset ID Conversion

### Perpetuals
```rust
fn perp_symbol_to_asset_id(symbol: &str, meta: &Meta) -> Result<u32> {
    // For main DEX perpetuals
    if !symbol.contains(':') {
        return meta.universe.iter()
            .position(|coin| coin.name == symbol)
            .map(|idx| idx as u32)
            .ok_or(SymbolNotFound);
    }

    // For builder DEX perpetuals
    let parts: Vec<&str> = symbol.split(':').collect();
    let dex_name = parts[0];
    let coin_name = parts[1];

    let dex = get_perp_dex(dex_name).await?;
    let coin_idx = dex.meta.universe.iter()
        .position(|c| c.name == coin_name)
        .ok_or(SymbolNotFound)?;

    Ok(100000 + (dex.perp_dex_index * 10000) + coin_idx as u32)
}
```

### Spot
```rust
fn spot_symbol_to_asset_id(symbol: &str, spot_meta: &SpotMeta) -> Result<u32> {
    // Direct @index format
    if symbol.starts_with('@') {
        let index: u32 = symbol[1..].parse()?;
        return Ok(10000 + index);
    }

    // Name lookup
    let pair = spot_meta.universe.iter()
        .find(|p| p.name == symbol)
        .ok_or(PairNotFound)?;

    Ok(10000 + pair.index as u32)
}
```

---

## Common Symbols (Mainnet)

### Perpetuals (Main DEX)

| Symbol | Asset ID | Description |
|--------|----------|-------------|
| BTC | 0 | Bitcoin |
| ETH | 1 | Ethereum |
| SOL | 2 | Solana |
| ARB | 3 | Arbitrum |
| AVAX | 4 | Avalanche |
| ... | ... | (Query meta for full list) |

**Note**: Indices change over time as new assets are added!

---

### Spot Pairs (Mainnet Examples)

| Symbol | Spot Index | Asset ID | Description |
|--------|------------|----------|-------------|
| PURR/USDC | 0 | 10000 | First pair |
| @107 | 107 | 10107 | HYPE/USDC |
| ... | ... | ... | (Query spotMeta for full list) |

---

## Symbol Validation

### Checking Symbol Exists

#### Perpetuals
```rust
async fn validate_perp_symbol(symbol: &str) -> Result<bool> {
    let meta = get_meta().await?;

    if symbol.contains(':') {
        // Builder DEX - need to validate DEX exists
        let dex_name = symbol.split(':').next().unwrap();
        let dexs = get_perp_dexs().await?;
        return Ok(dexs.iter().any(|d| d.name == dex_name));
    }

    // Main DEX
    Ok(meta.universe.iter().any(|c| c.name == symbol))
}
```

#### Spot
```rust
async fn validate_spot_symbol(symbol: &str) -> Result<bool> {
    let spot_meta = get_spot_meta().await?;

    if symbol.starts_with('@') {
        let index: u32 = symbol[1..].parse()?;
        return Ok(spot_meta.universe.iter().any(|p| p.index == index));
    }

    Ok(spot_meta.universe.iter().any(|p| p.name == symbol))
}
```

---

## Symbol Caching Strategy

### Why Cache?
- Symbol metadata rarely changes
- Avoid repeated API calls
- Improve performance

### Cache Invalidation
Update cache when:
- New assets deployed (monitor blockchain events)
- Manual refresh (every 1-6 hours recommended)
- Cache miss (symbol not found)

### Implementation Example
```rust
use std::sync::RwLock;
use std::time::{Duration, Instant};

struct SymbolCache {
    perp_meta: RwLock<Option<(Meta, Instant)>>,
    spot_meta: RwLock<Option<(SpotMeta, Instant)>>,
    ttl: Duration,
}

impl SymbolCache {
    fn new(ttl: Duration) -> Self {
        Self {
            perp_meta: RwLock::new(None),
            spot_meta: RwLock::new(None),
            ttl,
        }
    }

    async fn get_perp_meta(&self) -> Result<Meta> {
        {
            let cache = self.perp_meta.read().unwrap();
            if let Some((meta, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.ttl {
                    return Ok(meta.clone());
                }
            }
        }

        // Fetch fresh data
        let meta = fetch_meta().await?;
        let mut cache = self.perp_meta.write().unwrap();
        *cache = Some((meta.clone(), Instant::now()));
        Ok(meta)
    }

    async fn get_spot_meta(&self) -> Result<SpotMeta> {
        // Similar implementation
    }
}
```

---

## Precision and Tick Sizes

### Size Decimals
From metadata:
```json
{
  "name": "BTC",
  "szDecimals": 5
}
```

**Meaning**: 5 decimal places → minimum size increment = 0.00001 BTC

### Calculating Tick Size
```rust
fn get_size_precision(sz_decimals: u8) -> f64 {
    10_f64.powi(-(sz_decimals as i32))
}

// Example: BTC with szDecimals = 5
// Precision = 10^-5 = 0.00001
```

### Price Tick Size
Not directly in metadata - inferred from market or use standard:
```rust
fn get_price_precision(symbol: &str) -> f64 {
    match symbol {
        "BTC" | "ETH" => 0.5,      // $0.50 increments
        "SOL" | "AVAX" => 0.01,    // $0.01 increments
        _ => 0.001,                 // $0.001 default
    }
}
```

**Note**: Query recent trades to determine actual tick size used.

---

## Special Cases

### USDC Representation
- **Perpetuals margin**: USDC balance implied, not explicitly shown
- **Spot**: USDC is token index `0` with 6 wei decimals

### Cross-Chain Assets
Some assets may have EVM contracts:
```json
{
  "name": "HYPE",
  "evmContract": "0x...",
  "fullName": "Hyperliquid"
}
```

### Canonical vs Non-Canonical
```json
{
  "isCanonical": true
}
```

**Canonical**: Official/verified token
**Non-canonical**: User-deployed, use caution

---

## Error Cases

### Symbol Not Found
```rust
// Perpetuals
if meta.universe.iter().position(|c| c.name == symbol).is_none() {
    return Err(ExchangeError::SymbolNotFound(symbol.to_string()));
}

// Spot
if spot_meta.universe.iter().find(|p| p.name == symbol).is_none() {
    return Err(ExchangeError::PairNotFound(symbol.to_string()));
}
```

### Invalid Spot Index
```rust
if symbol.starts_with('@') {
    let index: u32 = symbol[1..].parse()
        .map_err(|_| ExchangeError::InvalidSymbolFormat)?;

    if index >= spot_meta.universe.len() as u32 {
        return Err(ExchangeError::InvalidSpotIndex(index));
    }
}
```

---

## Symbol Mapping Reference

### Info Endpoints (Use Symbol Names)
```json
{
  "type": "l2Book",
  "coin": "BTC"           // ✓ Use coin name
}

{
  "type": "candleSnapshot",
  "req": {
    "coin": "HYPE/USDC"   // ✓ Use pair name or @107
  }
}
```

### Exchange Endpoints (Use Asset IDs)
```json
{
  "action": {
    "type": "order",
    "orders": [{
      "a": 0,             // ✓ Use asset ID (BTC = 0)
      "b": true,
      "p": "50000.0",
      "s": "0.1"
    }]
  }
}

{
  "action": {
    "type": "order",
    "orders": [{
      "a": 10107,         // ✓ Spot asset ID (HYPE/USDC = @107)
      "b": true,
      "p": "2.5",
      "s": "100.0"
    }]
  }
}
```

---

## Implementation Checklist

- [ ] Fetch and cache perpetuals metadata (`meta`)
- [ ] Fetch and cache spot metadata (`spotMeta`)
- [ ] Implement symbol normalization (user input → API format)
- [ ] Implement symbol to asset ID conversion
- [ ] Implement asset ID to symbol conversion (for responses)
- [ ] Handle builder DEX perpetuals separately
- [ ] Validate symbols before API calls
- [ ] Cache metadata with TTL (1-6 hours recommended)
- [ ] Handle metadata refresh on cache miss
- [ ] Extract tick sizes and size decimals
- [ ] Support both `@index` and `NAME/QUOTE` for spot
- [ ] Handle PURR/USDC special case
- [ ] Validate spot indices are within range
- [ ] Log warnings for non-canonical tokens

---

## Summary

### Key Differences

| Aspect | Perpetuals | Spot |
|--------|-----------|------|
| **Symbol Format** | Coin name | `@{index}` or `PAIR/QUOTE` |
| **Asset ID** | Index (0, 1, 2...) | `10000 + index` |
| **Builder DEX** | `dex:COIN` format | N/A |
| **Builder Asset ID** | `100000 + dex*10000 + idx` | N/A |
| **Metadata Endpoint** | `meta` | `spotMeta` |

### Common Pitfalls

1. **Using coin name in exchange endpoint**: Use asset ID instead
2. **Confusing token index with spot index**: They are different!
3. **Hardcoding asset IDs**: Query metadata, indices change
4. **Not handling builder DEX format**: Check for `:` in symbol
5. **Forgetting `10000` offset for spot**: Spot IDs start at 10000

### Best Practices

1. **Always query metadata on startup**: Don't hardcode mappings
2. **Cache metadata appropriately**: Reduce API calls, improve performance
3. **Validate symbols before use**: Prevent invalid requests
4. **Support flexible input formats**: Normalize user input
5. **Handle both spot formats**: `@index` and `NAME/QUOTE`
