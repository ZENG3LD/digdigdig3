# Vertex Protocol Symbols

Vertex Protocol uses numeric product IDs to identify trading pairs. The `/symbols` endpoint maps product IDs to human-readable symbols.

## Symbol Format

### Product Types

1. **Spot**: Base asset symbol (e.g., "BTC", "ETH", "USDC")
2. **Perpetuals**: Base-PERP format (e.g., "BTC-PERP", "ETH-PERP")

### Quote Currency

All markets quote in **USDC** (product_id 0).

## Symbols Endpoint

**URL**: `GET https://gateway.prod.vertexprotocol.com/v1/symbols`

**Response**:
```json
{
  "status": "success",
  "symbols": {
    "0": "USDC",
    "1": "BTC",
    "2": "BTC-PERP",
    "3": "ETH",
    "4": "ETH-PERP",
    "5": "USDT",
    "6": "ARB-PERP",
    "7": "USDC.E",
    "8": "SOL-PERP",
    "9": "ARB",
    "10": "MATIC-PERP",
    "11": "LINK",
    "12": "LINK-PERP"
  }
}
```

## Product ID Structure

### Even IDs (0, 2, 4, ...): Spot Markets
- `0`: USDC (quote currency)
- `2`: BTC-PERP (perpetual)
- `4`: ETH-PERP (perpetual)
- `6`: ARB-PERP (perpetual)

### Odd IDs (1, 3, 5, ...): Spot Assets
- `1`: BTC (spot)
- `3`: ETH (spot)
- `5`: USDT (spot)
- `7`: USDC.E (bridged USDC)
- `9`: ARB (spot)
- `11`: LINK (spot)

**Note**: Pattern is not strictly enforced; always query `/symbols` for accurate mapping.

## Symbol to Product ID Mapping

### Building the Map

```rust
use std::collections::HashMap;

async fn fetch_symbols(base_url: &str) -> Result<HashMap<String, u32>, Error> {
    let url = format!("{}/symbols", base_url);
    let response: serde_json::Value = reqwest::get(&url).await?.json().await?;

    let mut symbol_to_id = HashMap::new();

    if let Some(symbols) = response["symbols"].as_object() {
        for (id_str, symbol) in symbols {
            let product_id: u32 = id_str.parse()?;
            let symbol_str = symbol.as_str().unwrap().to_string();
            symbol_to_id.insert(symbol_str, product_id);
        }
    }

    Ok(symbol_to_id)
}
```

### Reverse Mapping

```rust
fn build_reverse_map(symbols: &HashMap<String, u32>) -> HashMap<u32, String> {
    symbols.iter().map(|(k, v)| (*v, k.clone())).collect()
}
```

## Trading Pair Normalization

### Vertex Format
- Perpetuals: `{BASE}-PERP` (e.g., "BTC-PERP")
- Spot: `{BASE}` (e.g., "BTC")

### Standard Format Conversion

Many exchanges use formats like `BTC/USDC` or `BTCUSDC`. Convert to Vertex format:

```rust
fn normalize_trading_pair(pair: &str) -> String {
    // BTC/USDC -> BTC
    // BTCUSDC -> BTC
    // BTC-USDC -> BTC
    let base = pair
        .replace("/USDC", "")
        .replace("USDC", "")
        .replace("-USDC", "");

    base.to_uppercase()
}

fn to_vertex_perp_symbol(base: &str) -> String {
    format!("{}-PERP", base.to_uppercase())
}
```

**Examples**:
- `BTC/USDC` → `BTC` (spot) or `BTC-PERP` (perp)
- `ETHUSDC` → `ETH` (spot) or `ETH-PERP` (perp)
- `ARB-USDC` → `ARB` (spot) or `ARB-PERP` (perp)

## Product Type Detection

```rust
fn is_perpetual(symbol: &str) -> bool {
    symbol.ends_with("-PERP")
}

fn is_spot(symbol: &str) -> bool {
    !symbol.ends_with("-PERP")
}

fn get_base_asset(symbol: &str) -> String {
    symbol.replace("-PERP", "")
}
```

## Symbol Filtering

### Get All Perpetuals

```rust
fn get_perpetual_symbols(symbols: &HashMap<String, u32>) -> Vec<(String, u32)> {
    symbols
        .iter()
        .filter(|(symbol, _)| is_perpetual(symbol))
        .map(|(symbol, id)| (symbol.clone(), *id))
        .collect()
}
```

### Get All Spot Assets

```rust
fn get_spot_symbols(symbols: &HashMap<String, u32>) -> Vec<(String, u32)> {
    symbols
        .iter()
        .filter(|(symbol, _)| is_spot(symbol))
        .map(|(symbol, id)| (symbol.clone(), *id))
        .collect()
}
```

## Product Information

Use `all_products` query to get detailed info for each product:

```rust
use serde_json::json;

async fn get_product_info(
    base_url: &str,
    product_id: u32,
) -> Result<ProductInfo, Error> {
    let url = format!("{}/query", base_url);
    let payload = json!({
        "type": "all_products"
    });

    let response = reqwest::Client::new()
        .post(&url)
        .json(&payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Parse spot_products or perp_products array
    // Find matching product_id
    // Return ProductInfo

    Ok(ProductInfo { /* ... */ })
}
```

## Example Symbol List (Mainnet)

### Spot Markets

| Product ID | Symbol | Name |
|------------|--------|------|
| 0 | USDC | USD Coin |
| 1 | BTC | Bitcoin |
| 3 | ETH | Ethereum |
| 5 | USDT | Tether USD |
| 7 | USDC.E | Bridged USDC |
| 9 | ARB | Arbitrum |
| 11 | LINK | Chainlink |

### Perpetual Markets

| Product ID | Symbol | Name |
|------------|--------|------|
| 2 | BTC-PERP | Bitcoin Perpetual |
| 4 | ETH-PERP | Ethereum Perpetual |
| 6 | ARB-PERP | Arbitrum Perpetual |
| 8 | SOL-PERP | Solana Perpetual |
| 10 | MATIC-PERP | Polygon Perpetual |
| 12 | LINK-PERP | Chainlink Perpetual |

**Note**: This list is illustrative. Always query the `/symbols` endpoint for current product IDs.

## Symbol Caching

Cache symbols to avoid repeated API calls:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

struct SymbolCache {
    symbols: Arc<RwLock<HashMap<String, u32>>>,
    last_update: Arc<RwLock<Instant>>,
    ttl: Duration,
}

impl SymbolCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    async fn get_symbols(&self, base_url: &str) -> Result<HashMap<String, u32>, Error> {
        let last_update = *self.last_update.read().await;

        if last_update.elapsed() > self.ttl {
            // Refresh cache
            let new_symbols = fetch_symbols(base_url).await?;
            *self.symbols.write().await = new_symbols.clone();
            *self.last_update.write().await = Instant::now();
            Ok(new_symbols)
        } else {
            // Use cached
            Ok(self.symbols.read().await.clone())
        }
    }

    async fn get_product_id(&self, base_url: &str, symbol: &str) -> Result<Option<u32>, Error> {
        let symbols = self.get_symbols(base_url).await?;
        Ok(symbols.get(symbol).copied())
    }
}
```

## Usage in Connector

```rust
// Initialize cache on startup
let symbol_cache = SymbolCache::new(3600); // 1 hour TTL

// Get product ID for symbol
let product_id = symbol_cache
    .get_product_id(&self.base_url, "BTC-PERP")
    .await?
    .ok_or(Error::InvalidSymbol)?;

// Place order using product_id
self.place_order(product_id, price, amount).await?;
```

## Symbol Validation

```rust
async fn validate_symbol(
    base_url: &str,
    symbol: &str,
) -> Result<bool, Error> {
    let symbols = fetch_symbols(base_url).await?;
    Ok(symbols.contains_key(symbol))
}
```

## Trading Pair Format

For consistency with other connectors, use this format:

### Internal Format
- `{BASE}-PERP` for perpetuals
- `{BASE}-USDC` for spot

### Conversion Functions

```rust
fn to_internal_format(symbol: &str, is_perp: bool) -> String {
    let base = get_base_asset(symbol);
    if is_perp {
        format!("{}-PERP", base)
    } else {
        format!("{}-USDC", base)
    }
}

fn to_vertex_format(internal_symbol: &str) -> String {
    if internal_symbol.ends_with("-PERP") {
        internal_symbol.to_string()
    } else {
        // BTC-USDC -> BTC
        internal_symbol.replace("-USDC", "")
    }
}
```

## Symbol Metadata

Track additional metadata for each symbol:

```rust
struct SymbolInfo {
    product_id: u32,
    symbol: String,
    base_asset: String,
    quote_asset: String, // Always USDC
    is_perpetual: bool,
    size_increment: String, // From book_info
    price_increment: String, // From book_info
    min_size: String, // From book_info
}

impl SymbolInfo {
    fn from_product(product_id: u32, symbol: &str, book_info: &BookInfo) -> Self {
        let is_perpetual = symbol.ends_with("-PERP");
        let base_asset = symbol.replace("-PERP", "");

        Self {
            product_id,
            symbol: symbol.to_string(),
            base_asset,
            quote_asset: "USDC".to_string(),
            is_perpetual,
            size_increment: book_info.size_increment.clone(),
            price_increment: book_info.price_increment_x18.clone(),
            min_size: book_info.min_size.clone(),
        }
    }
}
```

## Multi-Network Support

Symbols may differ across networks:

```rust
enum Network {
    ArbitrumOne,
    ArbitrumSepolia,
    Base,
    Blast,
}

async fn fetch_symbols_for_network(
    network: Network,
) -> Result<HashMap<String, u32>, Error> {
    let base_url = match network {
        Network::ArbitrumOne => "https://gateway.prod.vertexprotocol.com/v1",
        Network::ArbitrumSepolia => "https://gateway.sepolia-test.vertexprotocol.com/v1",
        _ => return Err(Error::UnsupportedNetwork),
    };

    fetch_symbols(base_url).await
}
```

## Error Handling

```rust
#[derive(Debug)]
enum SymbolError {
    SymbolNotFound(String),
    InvalidProductId(u32),
    ApiError(String),
}

impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SymbolNotFound(s) => write!(f, "Symbol not found: {}", s),
            Self::InvalidProductId(id) => write!(f, "Invalid product ID: {}", id),
            Self::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for SymbolError {}
```

## Best Practices

1. **Cache symbols** with reasonable TTL (1 hour)
2. **Validate symbols** before using in API calls
3. **Handle missing symbols** gracefully
4. **Refresh periodically** to catch new listings
5. **Log symbol changes** for debugging
6. **Use product IDs** in all API calls (not symbols)
7. **Normalize formats** for consistency

## Symbol Update Detection

```rust
async fn detect_new_symbols(
    base_url: &str,
    old_symbols: &HashMap<String, u32>,
) -> Result<Vec<String>, Error> {
    let new_symbols = fetch_symbols(base_url).await?;

    let new_listings: Vec<String> = new_symbols
        .keys()
        .filter(|symbol| !old_symbols.contains_key(*symbol))
        .cloned()
        .collect();

    if !new_listings.is_empty() {
        log::info!("New symbols detected: {:?}", new_listings);
    }

    Ok(new_listings)
}
```
