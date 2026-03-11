# Bitget Symbol Format and Conventions

## Symbol Naming Convention

### Spot Symbols

Bitget spot symbols combine the base and quote currencies with a suffix:

**Format:** `{BASE}{QUOTE}_SPBL`

**Examples:**
- Bitcoin/USDT: `BTCUSDT_SPBL`
- Ethereum/USDT: `ETHUSDT_SPBL`
- BNB/BTC: `BNBBTC_SPBL`

**Key Points:**
- No separator between base and quote (e.g., `BTCUSDT`, not `BTC-USDT` or `BTC/USDT`)
- Always uppercase
- Suffix `_SPBL` indicates spot trading
- SPBL = "Spot BuLL" (Bitget's spot trading designation)

### Futures Symbols

Futures symbols use different suffixes based on the product type:

#### USDT-Margined Perpetual Futures
**Format:** `{BASE}{QUOTE}_UMCBL`

**Examples:**
- `BTCUSDT_UMCBL`
- `ETHUSDT_UMCBL`
- `SOLUSDT_UMCBL`

**Suffix:** `_UMCBL` = "USDT Margined Coin-Based Leveraged"

#### Coin-Margined Perpetual Futures
**Format:** `{BASE}{QUOTE}_DMCBL`

**Examples:**
- `BTCUSD_DMCBL`
- `ETHUSD_DMCBL`

**Suffix:** `_DMCBL` = "Digital (Coin) Margined Coin-Based Leveraged"

#### USDC-Margined Perpetual Futures
**Format:** `{BASE}{QUOTE}_CMCBL`

**Examples:**
- `BTCUSDC_CMCBL`
- `ETHUSDC_CMCBL`

**Suffix:** `_CMCBL` = "USDC Margined Coin-Based Leveraged"

#### Simulated Trading (Testnet)
**Format:** `{BASE}{QUOTE}_SUMCBL`

**Examples:**
- `BTCUSDT_SUMCBL`
- `ETHUSDT_SUMCBL`

**Suffix:** `_SUMCBL` = "Simulated USDT Margined Coin-Based Leveraged"

### Delivery Futures (Quarterly/Monthly)

**Format:** `{BASE}{QUOTE}{MonthCode}{Year}`

**Month Codes:**
- H = March
- M = June
- U = September
- Z = December

**Examples:**
- Bitcoin Mar 2026: `BTCUSDTH26`
- Bitcoin Jun 2026: `BTCUSDTM26`
- Bitcoin Sep 2026: `BTCUSDTU26`
- Bitcoin Dec 2026: `BTCUSDTZ26`

## API Version Differences

### V1 API (Legacy)
Requires full suffix in symbol parameter:
```
symbol=BTCUSDT_SPBL
symbol=BTCUSDT_UMCBL
```

### V2 API (Current)
Removed business line suffixes, uses cleaner format:
```
symbol=BTCUSDT
```

The product type is specified separately:
```
GET /api/v2/spot/market/ticker?symbol=BTCUSDT
GET /api/v2/mix/market/ticker?symbol=BTCUSDT&productType=umcbl
```

**Note:** V1 endpoints are still widely used and supported. This documentation uses V1 format unless specified otherwise.

## Product Types

When using APIs (especially V2 or futures endpoints), specify product type:

| Product Type | Code | Description |
|-------------|------|-------------|
| USDT-Margined Perpetual | `umcbl` | USDT margin, perpetual contracts |
| Coin-Margined Perpetual | `dmcbl` | Coin margin, perpetual contracts |
| USDC-Margined Perpetual | `cmcbl` | USDC margin, perpetual contracts |
| Simulated Trading | `sumcbl` | Demo/testnet contracts |

**Usage in API:**
```json
{
  "symbol": "BTCUSDT_UMCBL",
  "productType": "umcbl"
}
```

## Symbol Information Endpoints

### Get All Spot Symbols
```
GET /api/spot/v1/public/products
```

**Response:**
```json
{
  "code": "00000",
  "data": [
    {
      "symbol": "BTCUSDT_SPBL",
      "baseCoin": "BTC",
      "quoteCoin": "USDT",
      "minTradeAmount": "0.0001",
      "maxTradeAmount": "10000000",
      "takerFeeRate": "0.002",
      "makerFeeRate": "0.002",
      "pricePrecision": "2",
      "quantityPrecision": "4",
      "status": "online"
    }
  ]
}
```

### Get Single Spot Symbol
```
GET /api/spot/v1/public/product?symbol=BTCUSDT_SPBL
```

### Get All Futures Symbols
```
GET /api/mix/v1/market/contracts?productType=umcbl
```

**Response:**
```json
{
  "code": "00000",
  "data": [
    {
      "symbol": "BTCUSDT_UMCBL",
      "baseCoin": "BTC",
      "quoteCoin": "USDT",
      "supportMarginCoins": ["USDT"],
      "minTradeNum": "0.001",
      "priceEndStep": "0.5",
      "volumePlace": "3",
      "sizeMultiplier": "0.001"
    }
  ]
}
```

## Symbol Fields

### Common Fields

| Field | Description | Example |
|-------|-------------|---------|
| `symbol` | Full trading symbol | `BTCUSDT_SPBL` |
| `baseCoin` | Base currency | `BTC` |
| `quoteCoin` | Quote currency | `USDT` |
| `status` | Trading status | `online`, `offline` |

### Spot-Specific Fields

| Field | Description | Example |
|-------|-------------|---------|
| `minTradeAmount` | Minimum order size (base) | `0.0001` |
| `maxTradeAmount` | Maximum order size (base) | `10000000` |
| `minTradeUSDT` | Minimum order value (USDT) | `5` |
| `pricePrecision` | Price decimal places | `2` |
| `quantityPrecision` | Quantity decimal places | `4` |
| `quotePrecision` | Quote precision | `8` |
| `buyLimitPriceRatio` | Max buy price deviation | `0.05` (5%) |
| `sellLimitPriceRatio` | Max sell price deviation | `0.05` (5%) |

### Futures-Specific Fields

| Field | Description | Example |
|-------|-------------|---------|
| `supportMarginCoins` | Supported margin currencies | `["USDT"]` |
| `minTradeNum` | Minimum contract size | `0.001` |
| `priceEndStep` | Price tick size | `0.5` |
| `volumePlace` | Volume precision | `3` |
| `sizeMultiplier` | Contract multiplier | `0.001` |
| `makerFeeRate` | Maker fee rate | `0.0002` |
| `takerFeeRate` | Taker fee rate | `0.0006` |

## Case Sensitivity

**IMPORTANT:** Symbols are case-sensitive and must be uppercase.

**Correct:**
```
BTCUSDT_SPBL
ETHUSDT_UMCBL
```

**Incorrect:**
```
btcusdt_spbl
BtcUsdt_SPBL
btcUSDT_spbl
```

## Special Symbol Differences

Some cryptocurrencies have different ticker symbols in spot vs futures markets:

| Spot Symbol | Futures Symbol | Cryptocurrency |
|------------|----------------|----------------|
| LUNA | LUNA2 | Terra |
| $ALT | ALT | AltLayer |
| MEMECOIN | MEME | Memecoin |

**Example:**
- Spot: `LUNA_SPBL`
- Futures: `LUNA2USDT_UMCBL`

Always verify the correct symbol for your target market.

## Symbol Validation

### Valid Symbol Characters
- Uppercase letters (A-Z)
- Numbers (0-9)
- Underscore (_)
- Dollar sign ($) - rare, specific coins

### Invalid Symbols
Symbols do NOT contain:
- Hyphens/dashes (-)
- Slashes (/)
- Spaces
- Lowercase letters

## WebSocket Symbol Format

WebSocket subscriptions use the same symbol format:

**Spot Subscription:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "instType": "SPOT",
      "channel": "ticker",
      "instId": "BTCUSDT"
    }
  ]
}
```

**Futures Subscription:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "instType": "USDT-FUTURES",
      "channel": "ticker",
      "instId": "BTCUSDT"
    }
  ]
}
```

**Note:** WebSocket uses `instId` for symbol and `instType` for market type.

### WebSocket Market Types (instType)

| instType | Market |
|----------|--------|
| `SPOT` | Spot trading |
| `USDT-FUTURES` | USDT-margined futures |
| `COIN-FUTURES` | Coin-margined futures |
| `USDC-FUTURES` | USDC-margined futures |

## Symbol Parsing

To parse a Bitget symbol:

```rust
fn parse_symbol(symbol: &str) -> Option<(String, String, String)> {
    // Split by underscore
    let parts: Vec<&str> = symbol.split('_').collect();
    if parts.len() != 2 {
        return None;
    }

    let pair = parts[0];
    let market_type = parts[1];

    // Common quote currencies (longest first for correct matching)
    let quote_currencies = ["USDT", "USDC", "BTC", "ETH", "USD"];

    for quote in &quote_currencies {
        if pair.ends_with(quote) {
            let base = &pair[..pair.len() - quote.len()];
            return Some((
                base.to_string(),
                quote.to_string(),
                market_type.to_string()
            ));
        }
    }

    None
}

// Example usage
let (base, quote, market) = parse_symbol("BTCUSDT_SPBL").unwrap();
// base = "BTC", quote = "USDT", market = "SPBL"
```

## Symbol Construction

To construct a Bitget symbol:

```rust
fn construct_spot_symbol(base: &str, quote: &str) -> String {
    format!("{}{}_{}", base.to_uppercase(), quote.to_uppercase(), "SPBL")
}

fn construct_futures_symbol(base: &str, quote: &str, product_type: &str) -> String {
    let suffix = match product_type {
        "umcbl" => "UMCBL",
        "dmcbl" => "DMCBL",
        "cmcbl" => "CMCBL",
        "sumcbl" => "SUMCBL",
        _ => "UMCBL", // default to USDT-margined
    };
    format!("{}{}_{}", base.to_uppercase(), quote.to_uppercase(), suffix)
}

// Examples
let spot = construct_spot_symbol("BTC", "USDT");
// "BTCUSDT_SPBL"

let futures = construct_futures_symbol("ETH", "USDT", "umcbl");
// "ETHUSDT_UMCBL"
```

## Common Quote Currencies

### Spot Market
- **USDT**: Tether (most common)
- **USDC**: USD Coin
- **BTC**: Bitcoin
- **ETH**: Ethereum

### Futures Market
- **USDT**: USDT-margined (most common)
- **USD**: Coin-margined
- **USDC**: USDC-margined

## Trading Pairs Discovery

To discover available trading pairs:

1. **Fetch all symbols:**
   ```
   GET /api/spot/v1/public/products (spot)
   GET /api/mix/v1/market/contracts?productType=umcbl (futures)
   ```

2. **Filter by base currency:**
   ```javascript
   const btcPairs = symbols.filter(s => s.baseCoin === "BTC");
   ```

3. **Filter by quote currency:**
   ```javascript
   const usdtPairs = symbols.filter(s => s.quoteCoin === "USDT");
   ```

4. **Filter by status:**
   ```javascript
   const onlinePairs = symbols.filter(s => s.status === "online");
   ```

## Symbol Caching

**Recommendation:** Cache symbol information locally to reduce API calls.

- Symbol configurations change infrequently
- Refresh cache periodically (e.g., every 24 hours)
- Update on receiving symbol-related errors

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct SymbolCache {
    symbols: HashMap<String, SymbolInfo>,
    last_update: Instant,
    ttl: Duration,
}

impl SymbolCache {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            last_update: Instant::now(),
            ttl: Duration::from_secs(86400), // 24 hours
        }
    }

    fn needs_refresh(&self) -> bool {
        self.last_update.elapsed() > self.ttl
    }

    async fn refresh(&mut self, api: &BitgetApi) -> Result<()> {
        let symbols = api.get_symbols().await?;
        self.symbols.clear();
        for symbol in symbols {
            self.symbols.insert(symbol.symbol.clone(), symbol);
        }
        self.last_update = Instant::now();
        Ok(())
    }

    fn get(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.get(symbol)
    }
}
```

## Error Messages

Common symbol-related errors:

| Error Code | Message | Cause |
|-----------|---------|-------|
| 40001 | Invalid symbol | Symbol doesn't exist or wrong format |
| 40002 | Symbol offline | Trading is suspended for this pair |
| 40807 | Symbol not found | Symbol doesn't exist in specified market |

## Best Practices

1. **Always validate symbols** before placing orders
2. **Use uppercase** for all symbol strings
3. **Cache symbol information** to reduce API calls
4. **Check `status` field** before trading (must be "online")
5. **Respect precision limits** (price, quantity)
6. **Handle special cases** (LUNA/LUNA2, $ALT/ALT, etc.)
7. **Use product type** parameter for futures endpoints
8. **Refresh symbol cache** periodically

## Example: Complete Symbol Handling

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct SymbolInfo {
    symbol: String,
    base_coin: String,
    quote_coin: String,
    min_trade_amount: String,
    max_trade_amount: String,
    price_precision: u8,
    quantity_precision: u8,
    status: String,
}

struct BitgetSymbolManager {
    spot_symbols: HashMap<String, SymbolInfo>,
    futures_symbols: HashMap<String, SymbolInfo>,
}

impl BitgetSymbolManager {
    fn new() -> Self {
        Self {
            spot_symbols: HashMap::new(),
            futures_symbols: HashMap::new(),
        }
    }

    async fn load_symbols(&mut self, api: &BitgetApi) -> Result<()> {
        // Load spot symbols
        let spot = api.get_spot_symbols().await?;
        for symbol in spot {
            self.spot_symbols.insert(symbol.symbol.clone(), symbol);
        }

        // Load futures symbols
        let futures = api.get_futures_symbols("umcbl").await?;
        for symbol in futures {
            self.futures_symbols.insert(symbol.symbol.clone(), symbol);
        }

        Ok(())
    }

    fn validate_spot_symbol(&self, symbol: &str) -> Result<&SymbolInfo> {
        self.spot_symbols
            .get(symbol)
            .filter(|s| s.status == "online")
            .ok_or_else(|| Error::InvalidSymbol(symbol.to_string()))
    }

    fn validate_futures_symbol(&self, symbol: &str) -> Result<&SymbolInfo> {
        self.futures_symbols
            .get(symbol)
            .filter(|s| s.status == "online")
            .ok_or_else(|| Error::InvalidSymbol(symbol.to_string()))
    }

    fn format_price(&self, symbol: &str, price: f64, is_spot: bool) -> Result<String> {
        let info = if is_spot {
            self.validate_spot_symbol(symbol)?
        } else {
            self.validate_futures_symbol(symbol)?
        };

        Ok(format!("{:.prec$}", price, prec = info.price_precision as usize))
    }

    fn format_quantity(&self, symbol: &str, qty: f64, is_spot: bool) -> Result<String> {
        let info = if is_spot {
            self.validate_spot_symbol(symbol)?
        } else {
            self.validate_futures_symbol(symbol)?
        };

        Ok(format!("{:.prec$}", qty, prec = info.quantity_precision as usize))
    }
}
```

## Sources

- [Bitget V2 API Update Guide](https://www.bitget.com/api-doc/common/release-note)
- [Bitget Symbol Info API](https://www.bitget.com/api-doc/spot/market/Get-Symbols)
- [Bitget Futures Config API](https://www.bitget.com/api-doc/contract/market/Get-All-Symbols-Contracts)
- [Bitget API FAQ](https://www.bitget.com/api-doc/common/faq)
