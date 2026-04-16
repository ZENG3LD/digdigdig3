# HTX API Symbol Format

Complete symbol and trading pair documentation for HTX (formerly Huobi) exchange.

## Symbol Format

### Standard Format

HTX uses **lowercase, concatenated** symbol format:

```
{base_currency}{quote_currency}
```

**Examples:**
- Bitcoin/USDT: `btcusdt`
- Ethereum/BTC: `ethbtc`
- Litecoin/USDT: `ltcusdt`

**Rules:**
- All lowercase letters
- No separators (no dash, slash, underscore)
- Base currency first, quote currency second
- ASCII characters only

### Format Comparison

| Exchange | Format | Example |
|----------|--------|---------|
| HTX | lowercase, no separator | `btcusdt` |
| Binance | uppercase, no separator | `BTCUSDT` |
| Coinbase | uppercase with dash | `BTC-USDT` |
| Kraken | uppercase with slash | `BTC/USDT` |

## Symbol Information Endpoint

### Get Trading Symbols (V2)

```
GET /v2/settings/common/symbols
```

**Parameters:**
- `symbols` (optional): Comma-separated symbol list
- `ts` (optional): Response generation timestamp

**Response:**
```json
{
  "code": 200,
  "data": [
    {
      "symbol": "btcusdt",
      "state": "online",
      "bc": "btc",
      "qc": "usdt",
      "pp": 2,
      "ap": 6,
      "sp": "main",
      "vp": 8,
      "minov": "5",
      "maxov": "200000",
      "lominoa": "0.0001",
      "lomaxoa": "1000",
      "lomaxba": "10000000",
      "lomaxsa": "1000",
      "smminoa": "0.0001",
      "blmlt": "3",
      "slmlt": "3",
      "smmaxoa": "100",
      "bmmaxov": "1000000",
      "msormlt": "0.01",
      "mbormlt": "50",
      "maxov": "200000",
      "u": "btcusdt",
      "mfr": "0.001",
      "ct": "0.01",
      "tags": "etp,holdinglimit"
    }
  ]
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Trading symbol (btcusdt) |
| `state` | string | Trading state: online, offline, suspend, pre-online |
| `bc` | string | Base currency (btc) |
| `qc` | string | Quote currency (usdt) |
| `pp` | int | Price precision (decimal places) |
| `ap` | int | Amount precision (decimal places) |
| `sp` | string | Symbol partition (main, innovation, bifurcation) |
| `vp` | int | Value precision (decimal places) |
| `minov` | string | Minimum order value (in quote currency) |
| `maxov` | string | Maximum order value (in quote currency) |
| `lominoa` | string | Limit order minimum amount |
| `lomaxoa` | string | Limit order maximum amount |
| `lomaxba` | string | Limit order max buy amount |
| `lomaxsa` | string | Limit order max sell amount |
| `smminoa` | string | Sell-market minimum order amount |
| `blmlt` | string | Buy limit order max leverage |
| `slmlt` | string | Sell limit order max leverage |
| `smmaxoa` | string | Sell-market max order amount |
| `bmmaxov` | string | Buy-market max order value |
| `msormlt` | string | Min sell-market order rate limit |
| `mbormlt` | string | Min buy-market order rate limit |
| `mfr` | string | Maker fee rate |
| `ct` | string | Control type |
| `tags` | string | Symbol tags (comma-separated) |
| `u` | string | Underlying asset symbol |

### Trading States

| State | Description | Allowed Actions |
|-------|-------------|-----------------|
| `online` | Normal trading | All operations allowed |
| `offline` | Trading halted | No trading, view only |
| `suspend` | Temporarily suspended | Limited operations |
| `pre-online` | Pre-listing | Preparation phase |

### Symbol Partitions

| Partition | Description |
|-----------|-------------|
| `main` | Main market (major cryptocurrencies) |
| `innovation` | Innovation zone (new/experimental tokens) |
| `bifurcation` | Fork coins zone |

## Symbol Validation

### Valid Symbol Examples

```
btcusdt     ✓ Bitcoin/USDT
ethusdt     ✓ Ethereum/USDT
ethbtc      ✓ Ethereum/BTC
bnbusdt     ✓ BNB/USDT
adausdt     ✓ Cardano/USDT
dotusdt     ✓ Polkadot/USDT
linkusdt    ✓ Chainlink/USDT
ltcusdt     ✓ Litecoin/USDT
xrpusdt     ✓ Ripple/USDT
bchusdt     ✓ Bitcoin Cash/USDT
```

### Invalid Symbol Examples

```
BTCUSDT     ✗ Uppercase not allowed
BTC-USDT    ✗ Separator not allowed
BTC/USDT    ✗ Separator not allowed
btc_usdt    ✗ Separator not allowed
usdtbtc     ✗ Wrong order (quote/base reversed)
```

### Symbol Validation Rules

```rust
fn validate_symbol(symbol: &str) -> bool {
    // Must be lowercase
    if symbol != symbol.to_lowercase() {
        return false;
    }

    // Must be alphanumeric only
    if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    // Must be at least 6 characters (e.g., btcusdt)
    if symbol.len() < 6 {
        return false;
    }

    true
}
```

## Common Quote Currencies

HTX supports multiple quote currencies:

| Currency | Description | Example Pairs |
|----------|-------------|---------------|
| `usdt` | Tether USD | btcusdt, ethusdt |
| `btc` | Bitcoin | ethbtc, bnbbtc |
| `eth` | Ethereum | linketh, adaeth |
| `husd` | HUSD stablecoin | btchusd, ethhusd |
| `usdc` | USD Coin | btcusdc, ethusdc |
| `ht` | Huobi Token | btcht, ethht |

## Precision and Limits

### Price Precision

Symbol precision determines the number of decimal places:

```json
{
  "symbol": "btcusdt",
  "pp": 2  // Price precision: 2 decimals
}
```

**Valid prices:**
- `50000.00` ✓
- `50000.12` ✓
- `50000.123` ✗ (too many decimals)

### Amount Precision

```json
{
  "symbol": "btcusdt",
  "ap": 6  // Amount precision: 6 decimals
}
```

**Valid amounts:**
- `0.1` ✓
- `0.123456` ✓
- `0.1234567` ✗ (too many decimals)

### Order Limits

**Minimum Order Value:**
```json
{
  "symbol": "btcusdt",
  "minov": "5"  // Minimum $5 USDT
}
```

For `btcusdt` at $50,000:
- Min amount: 5 / 50000 = 0.0001 BTC

**Maximum Order Value:**
```json
{
  "symbol": "btcusdt",
  "maxov": "200000"  // Maximum $200,000 USDT
}
```

**Limit Order Min/Max Amounts:**
```json
{
  "lominoa": "0.0001",  // Min 0.0001 BTC
  "lomaxoa": "1000"     // Max 1000 BTC
}
```

## Symbol Formatting Functions

### Format Symbol for API

```rust
pub fn format_symbol(base: &str, quote: &str) -> String {
    format!("{}{}", base.to_lowercase(), quote.to_lowercase())
}

// Usage
let symbol = format_symbol("BTC", "USDT");
assert_eq!(symbol, "btcusdt");
```

### Parse Symbol Components

```rust
pub fn parse_symbol(symbol: &str, quote_currencies: &[&str]) -> Option<(String, String)> {
    for quote in quote_currencies {
        if symbol.ends_with(quote) {
            let base = &symbol[..symbol.len() - quote.len()];
            return Some((base.to_string(), quote.to_string()));
        }
    }
    None
}

// Usage
let quotes = vec!["usdt", "btc", "eth"];
let (base, quote) = parse_symbol("btcusdt", &quotes).unwrap();
assert_eq!(base, "btc");
assert_eq!(quote, "usdt");
```

### Normalize Symbol

```rust
pub fn normalize_symbol(input: &str) -> String {
    input
        .to_lowercase()
        .replace("-", "")
        .replace("/", "")
        .replace("_", "")
}

// Usage
assert_eq!(normalize_symbol("BTC-USDT"), "btcusdt");
assert_eq!(normalize_symbol("BTC/USDT"), "btcusdt");
assert_eq!(normalize_symbol("BTC_USDT"), "btcusdt");
```

## WebSocket Symbol Format

WebSocket topics use the same lowercase format:

```json
{
  "sub": "market.btcusdt.kline.1min"
}
```

**Topic format:**
```
market.{symbol}.{data_type}.{params}
```

**Examples:**
- Ticker: `market.btcusdt.detail`
- Depth: `market.btcusdt.depth.step0`
- Trades: `market.btcusdt.trade.detail`
- Klines: `market.btcusdt.kline.1day`

## Symbol Discovery

### List All Symbols

```rust
use reqwest;
use serde_json::Value;

async fn get_all_symbols() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = "https://api.huobi.pro/v2/settings/common/symbols";
    let response: Value = reqwest::get(url).await?.json().await?;

    let symbols = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["symbol"].as_str())
        .map(|s| s.to_string())
        .collect();

    Ok(symbols)
}
```

### Filter by Quote Currency

```rust
async fn get_symbols_by_quote(quote: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = "https://api.huobi.pro/v2/settings/common/symbols";
    let response: Value = reqwest::get(url).await?.json().await?;

    let symbols = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["qc"].as_str() == Some(quote))
        .filter_map(|s| s["symbol"].as_str())
        .map(|s| s.to_string())
        .collect();

    Ok(symbols)
}

// Usage
let usdt_pairs = get_symbols_by_quote("usdt").await?;
// Returns: ["btcusdt", "ethusdt", "bnbusdt", ...]
```

### Filter by State

```rust
async fn get_online_symbols() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = "https://api.huobi.pro/v2/settings/common/symbols";
    let response: Value = reqwest::get(url).await?.json().await?;

    let symbols = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["state"].as_str() == Some("online"))
        .filter_map(|s| s["symbol"].as_str())
        .map(|s| s.to_string())
        .collect();

    Ok(symbols)
}
```

## Symbol Cache

For performance, cache symbol information:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SymbolInfo {
    pub symbol: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub price_precision: u8,
    pub amount_precision: u8,
    pub min_order_value: String,
    pub max_order_value: String,
    pub state: String,
}

pub struct SymbolCache {
    symbols: Arc<RwLock<HashMap<String, SymbolInfo>>>,
}

impl SymbolCache {
    pub async fn refresh(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = "https://api.huobi.pro/v2/settings/common/symbols";
        let response: Value = reqwest::get(url).await?.json().await?;

        let mut cache = self.symbols.write().await;
        cache.clear();

        for item in response["data"].as_array().unwrap() {
            let info = SymbolInfo {
                symbol: item["symbol"].as_str().unwrap().to_string(),
                base_currency: item["bc"].as_str().unwrap().to_string(),
                quote_currency: item["qc"].as_str().unwrap().to_string(),
                price_precision: item["pp"].as_u64().unwrap() as u8,
                amount_precision: item["ap"].as_u64().unwrap() as u8,
                min_order_value: item["minov"].as_str().unwrap().to_string(),
                max_order_value: item["maxov"].as_str().unwrap().to_string(),
                state: item["state"].as_str().unwrap().to_string(),
            };
            cache.insert(info.symbol.clone(), info);
        }

        Ok(())
    }

    pub async fn get(&self, symbol: &str) -> Option<SymbolInfo> {
        let cache = self.symbols.read().await;
        cache.get(symbol).cloned()
    }
}
```

## Special Cases

### Stablecoins

Some stablecoin pairs have specific naming:

```
usdtusdc    ✓ USDT/USDC
daiusdt     ✓ DAI/USDT
busdusdt    ✓ BUSD/USDT
tusdusdt    ✓ TUSD/USDT
```

### Wrapped Tokens

```
wbtcusdt    ✓ Wrapped BTC/USDT
wethusdt    ✓ Wrapped ETH/USDT
```

### Leveraged Tokens

HTX ETP (Exchange Traded Products) have special suffixes:

```
btc3lusdt   ✓ BTC 3x Long/USDT
btc3susdt   ✓ BTC 3x Short/USDT
eth3lusdt   ✓ ETH 3x Long/USDT
```

## Best Practices

1. **Always lowercase:** Convert user input to lowercase before API calls
2. **Validate before use:** Check symbol exists and is online
3. **Cache symbol info:** Avoid repeated API calls for symbol metadata
4. **Handle precision:** Respect price and amount precision limits
5. **Check limits:** Validate order amounts against min/max values
6. **Monitor state:** Check trading state before placing orders
7. **Use V2 endpoint:** `/v2/settings/common/symbols` for complete info

## Migration from V1

HTX has two symbol endpoints:

**V1 (Legacy):**
```
GET /v1/settings/common/symbols
GET /v1/common/symbols
```

**V2 (Recommended):**
```
GET /v2/settings/common/symbols
```

V2 provides more fields and better structure. Use V2 for new implementations.

## Error Handling

### Invalid Symbol

```json
{
  "status": "error",
  "err-code": "invalid-parameter",
  "err-msg": "invalid symbol",
  "data": null
}
```

### Symbol Suspended

```json
{
  "status": "error",
  "err-code": "order-value-min-error",
  "err-msg": "trading suspended",
  "data": null
}
```

## Summary

- Format: lowercase, no separators (e.g., `btcusdt`)
- Get symbols: `GET /v2/settings/common/symbols`
- Parse: base + quote concatenated
- Validate: check state is "online"
- Respect: precision and order limits
- Cache: symbol information for performance
