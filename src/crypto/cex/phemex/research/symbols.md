# Phemex Symbol Formats & Scaling

Complete symbol format and value scaling documentation for V5 connector implementation.

## Symbol Naming Conventions

### Spot Trading Symbols

Spot symbols are **prefixed with `s`**:

| Format | Example | Description |
|--------|---------|-------------|
| `s{BASE}{QUOTE}` | `sBTCUSDT` | Bitcoin/Tether spot pair |
| | `sETHUSDT` | Ethereum/Tether spot pair |
| | `sBTCUSDC` | Bitcoin/USDC spot pair |

**Pattern:** Lowercase `s` + uppercase base currency + uppercase quote currency (no separator)

### Contract/Perpetual Symbols

Contract symbols have **no prefix**:

| Format | Example | Description |
|--------|---------|-------------|
| `{BASE}{QUOTE}` | `BTCUSD` | BTC/USD perpetual futures |
| | `ETHUSD` | ETH/USD perpetual futures |
| | `SOLUSD` | SOL/USD perpetual futures |
| | `uBTCUSD` | USDT-margined BTC perpetual |

**Pattern:** Uppercase base + uppercase quote (no separator)

**Contract Types:**
- **Coin-margined:** No prefix (e.g., `BTCUSD` - settled in BTC)
- **USDT-margined:** Prefixed with `u` (e.g., `uBTCUSD` - settled in USDT)

### Index & Mark Price Symbols

| Type | Format | Example |
|------|--------|---------|
| Index Price | `.{BASE}` | `.BTC` |
| Mark Price | `.M{BASE}` | `.MBTC` |
| Funding Rate | `.{BASE}FR` | `.BTCFR` |
| 8h Funding Rate | `.{BASE}FR8H` | `.BTCFR8H` |

## Product Information Structure

Retrieved from `GET /public/products`:

```json
{
  "symbol": "BTCUSD",
  "displaySymbol": "BTC/USD",
  "type": "PerpetualV2",
  "status": "Listed",

  "quoteCurrency": "USD",
  "settleCurrency": "BTC",
  "contractUnderlyingAssets": "USD",

  "priceScale": 4,
  "ratioScale": 8,
  "valueScale": 8,
  "pricePrecision": 1,

  "contractSize": 1.0,
  "lotSize": 1,
  "tickSize": 0.5,

  "minPriceEp": 5000000,
  "maxPriceEp": 10000000000,
  "maxOrderQty": 1000000,
  "tipOrderQty": 1000000,

  "defaultLeverage": 0,
  "maxLeverage": 100,
  "initMarginEr": 1000000,
  "maintMarginEr": 500000,
  "defaultRiskLimitEv": 10000000000,

  "makerFeeRateEr": -25000,
  "takerFeeRateEr": 75000,

  "fundingInterval": 8
}
```

### Key Product Fields

| Field | Description | Example Value |
|-------|-------------|---------------|
| `symbol` | Trading symbol identifier | `"BTCUSD"` |
| `displaySymbol` | Human-readable format | `"BTC/USD"` |
| `type` | Product type | `"PerpetualV2"`, `"Spot"` |
| `status` | Trading status | `"Listed"`, `"Unlisted"` |
| `quoteCurrency` | Quote currency | `"USD"`, `"USDT"` |
| `settleCurrency` | Settlement currency | `"BTC"`, `"USDT"` |
| `contractSize` | Contract multiplier | `1.0` |
| `lotSize` | Minimum order increment | `1` |
| `tickSize` | Minimum price increment | `0.5` |
| `maxOrderQty` | Maximum order quantity | `1000000` |
| `maxLeverage` | Maximum leverage | `100` |

## Value Scaling System

Phemex uses integer representation with scaling factors for precision.

### Scale Types

| Suffix | Field Type | Scale Factor | Example |
|--------|------------|--------------|---------|
| `Ep` | **Price** | `priceScale` | `priceEp`, `stopPxEp`, `markPriceEp` |
| `Er` | **Ratio** | `ratioScale` | `leverageEr`, `feeRateEr`, `initMarginEr` |
| `Ev` | **Value** | `valueScale` | `balanceEv`, `valueEv`, `pnlEv`, `amountEv` |

### Conversion Formulas

**Scaled to Actual:**
```
actual_value = scaled_value / 10^scale_factor
```

**Actual to Scaled:**
```
scaled_value = actual_value * 10^scale_factor
```

### Price Scaling (Ep)

**Common `priceScale` values:** 4, 8

**Example 1: BTCUSD (priceScale = 4)**
```
priceEp = 87700000
actual_price = 87700000 / 10^4 = 87700000 / 10000 = 8770.0 USD
```

**Example 2: Spot sBTCUSDT (priceScale = 8)**
```
priceEp = 8770000000
actual_price = 8770000000 / 10^8 = 87.70 USDT
```

**Rust implementation:**
```rust
fn unscale_price(price_ep: i64, price_scale: u8) -> f64 {
    price_ep as f64 / 10_f64.powi(price_scale as i32)
}

fn scale_price(price: f64, price_scale: u8) -> i64 {
    (price * 10_f64.powi(price_scale as i32)).round() as i64
}
```

### Ratio Scaling (Er)

**Common `ratioScale` value:** 8

**Example: Leverage**
```
leverageEr = 2000000
actual_leverage = 2000000 / 10^8 = 0.02

Special case: Sign indicates margin mode
- Positive (2000000) = 20x isolated margin
- Zero/Negative (0, -1000000) = Cross margin
```

**Example: Fee Rates**
```
makerFeeRateEr = -25000
actual_maker_fee = -25000 / 10^8 = -0.00025 = -0.025%

takerFeeRateEr = 75000
actual_taker_fee = 75000 / 10^8 = 0.00075 = 0.075%
```

**Rust implementation:**
```rust
fn unscale_ratio(ratio_er: i64, ratio_scale: u8) -> f64 {
    ratio_er as f64 / 10_f64.powi(ratio_scale as i32)
}

fn scale_ratio(ratio: f64, ratio_scale: u8) -> i64 {
    (ratio * 10_f64.powi(ratio_scale as i32)).round() as i64
}
```

### Value Scaling (Ev)

**Common `valueScale` values:** 4, 8

**Example 1: BTC Balance (valueScale = 8)**
```
balanceEv = 100000000
actual_balance = 100000000 / 10^8 = 1.0 BTC
```

**Example 2: USD Value (valueScale = 4)**
```
valueEv = 87700000
actual_value = 87700000 / 10^4 = 8770.0 USD
```

**Example 3: USDT Amount (valueScale = 8)**
```
amountEv = 1000000000
actual_amount = 1000000000 / 10^8 = 10.0 USDT
```

**Rust implementation:**
```rust
fn unscale_value(value_ev: i64, value_scale: u8) -> f64 {
    value_ev as f64 / 10_f64.powi(value_scale as i32)
}

fn scale_value(value: f64, value_scale: u8) -> i64 {
    (value * 10_f64.powi(value_scale as i32)).round() as i64
}
```

## Currency Information

Retrieved from `GET /public/products` (currencies array):

```json
{
  "currency": "BTC",
  "valueScale": 8,
  "minValueEv": 1,
  "maxValueEv": 5000000000000000,
  "name": "Bitcoin"
}
```

### Common Currency Scales

| Currency | valueScale | Min Amount | Example |
|----------|------------|------------|---------|
| BTC | 8 | 0.00000001 BTC (1 satoshi) | `100000000` = 1.0 BTC |
| ETH | 8 | 0.00000001 ETH | `100000000` = 1.0 ETH |
| USDT | 8 | 0.00000001 USDT | `100000000` = 1.0 USDT |
| USDC | 8 | 0.00000001 USDC | `100000000` = 1.0 USDC |
| USD | 4 | 0.0001 USD | `10000` = 1.0 USD |

## Symbol Validation

### Spot Symbol Validation

```rust
fn is_valid_spot_symbol(symbol: &str) -> bool {
    symbol.starts_with('s') && symbol.len() > 1 && symbol[1..].chars().all(|c| c.is_uppercase())
}

fn normalize_spot_symbol(base: &str, quote: &str) -> String {
    format!("s{}{}", base.to_uppercase(), quote.to_uppercase())
}
```

**Examples:**
- `normalize_spot_symbol("btc", "usdt")` → `"sBTCUSDT"`
- `normalize_spot_symbol("ETH", "USDT")` → `"sETHUSDT"`

### Contract Symbol Validation

```rust
fn is_valid_contract_symbol(symbol: &str) -> bool {
    symbol.chars().all(|c| c.is_uppercase() || c == 'u')
}

fn normalize_contract_symbol(base: &str, quote: &str, usdt_margined: bool) -> String {
    if usdt_margined {
        format!("u{}{}", base.to_uppercase(), quote.to_uppercase())
    } else {
        format!("{}{}", base.to_uppercase(), quote.to_uppercase())
    }
}
```

**Examples:**
- `normalize_contract_symbol("btc", "usd", false)` → `"BTCUSD"`
- `normalize_contract_symbol("btc", "usd", true)` → `"uBTCUSD"`

## Product Type Filtering

### Get Spot Symbols Only

```rust
fn get_spot_symbols(products: &ProductsResponse) -> Vec<String> {
    products.data.products.iter()
        .filter(|p| p.symbol.starts_with('s'))
        .map(|p| p.symbol.clone())
        .collect()
}
```

### Get Perpetual Contract Symbols

```rust
fn get_perpetual_symbols(products: &ProductsResponse) -> Vec<String> {
    products.data.products.iter()
        .filter(|p| p.type_field == "PerpetualV2")
        .map(|p| p.symbol.clone())
        .collect()
}
```

### Get Active Symbols Only

```rust
fn get_active_symbols(products: &ProductsResponse) -> Vec<String> {
    products.data.products.iter()
        .filter(|p| p.status == "Listed")
        .map(|p| p.symbol.clone())
        .collect()
}
```

## Scale Factor Storage

For efficient scaling/unscaling, store scale factors per symbol:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub symbol: String,
    pub price_scale: u8,
    pub ratio_scale: u8,
    pub value_scale: u8,
    pub lot_size: u64,
    pub tick_size: f64,
    pub max_leverage: u32,
}

pub struct SymbolRegistry {
    symbols: HashMap<String, SymbolInfo>,
}

impl SymbolRegistry {
    pub fn from_products(products: &ProductsResponse) -> Self {
        let mut symbols = HashMap::new();

        for product in &products.data.products {
            symbols.insert(
                product.symbol.clone(),
                SymbolInfo {
                    symbol: product.symbol.clone(),
                    price_scale: product.price_scale,
                    ratio_scale: product.ratio_scale,
                    value_scale: product.value_scale,
                    lot_size: product.lot_size,
                    tick_size: product.tick_size,
                    max_leverage: product.max_leverage,
                },
            );
        }

        Self { symbols }
    }

    pub fn unscale_price(&self, symbol: &str, price_ep: i64) -> Option<f64> {
        self.symbols.get(symbol).map(|info| {
            price_ep as f64 / 10_f64.powi(info.price_scale as i32)
        })
    }

    pub fn scale_price(&self, symbol: &str, price: f64) -> Option<i64> {
        self.symbols.get(symbol).map(|info| {
            (price * 10_f64.powi(info.price_scale as i32)).round() as i64
        })
    }
}
```

## Order Quantity Types (Spot)

Spot orders use `qtyType` to specify quantity format:

| QtyType | Description | Required Field |
|---------|-------------|----------------|
| `ByBase` | Quantity in base currency | `baseQtyEv` |
| `ByQuote` | Quantity in quote currency | `quoteQtyEv` |

**Example for sBTCUSDT:**
```json
// Buy 0.1 BTC (ByBase)
{
  "symbol": "sBTCUSDT",
  "side": "Buy",
  "qtyType": "ByBase",
  "baseQtyEv": 10000000,  // 0.1 BTC (valueScale=8)
  "priceEp": 8770000000   // 87.70 USDT (priceScale=8)
}

// Buy 100 USDT worth (ByQuote)
{
  "symbol": "sBTCUSDT",
  "side": "Buy",
  "qtyType": "ByQuote",
  "quoteQtyEv": 10000000000,  // 100 USDT (valueScale=8)
  "priceEp": 8770000000        // 87.70 USDT (priceScale=8)
}
```

## Contract Size & Multipliers

### Perpetual Contracts

Contract value calculation:
```
contract_value = quantity * contract_size * price
```

**Example: BTCUSD**
- `contractSize = 1.0`
- `quantity = 1000` contracts
- `price = 87700 USD`
- Value = 1000 × 1.0 × 87700 = 87,700 USD worth of BTC

### Position Value in Settlement Currency

For coin-margined contracts (settled in BTC):
```
position_value_btc = quantity * contract_size / price
```

**Example:**
- 1000 contracts at 87,700 USD
- Value = 1000 × 1.0 / 87,700 = 0.0114 BTC

## Precision & Rounding

### Price Precision

Respect `tickSize` when placing orders:

```rust
fn round_price_to_tick(price: f64, tick_size: f64) -> f64 {
    (price / tick_size).round() * tick_size
}
```

**Example:**
- `tickSize = 0.5`
- Input: 8770.3 USD
- Output: 8770.5 USD

### Quantity Precision

Respect `lotSize` for order quantities:

```rust
fn round_quantity_to_lot(quantity: u64, lot_size: u64) -> u64 {
    (quantity / lot_size) * lot_size
}
```

**Example:**
- `lotSize = 1`
- Input: 1234 contracts
- Output: 1234 contracts (already aligned)

## Symbol Metadata Caching

Cache product information to avoid repeated API calls:

```rust
use std::time::{Duration, Instant};

pub struct SymbolCache {
    registry: SymbolRegistry,
    last_update: Instant,
    ttl: Duration,
}

impl SymbolCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            registry: SymbolRegistry { symbols: HashMap::new() },
            last_update: Instant::now() - Duration::from_secs(ttl_seconds + 1),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn needs_refresh(&self) -> bool {
        self.last_update.elapsed() > self.ttl
    }

    pub async fn refresh(&mut self, api_client: &ApiClient) -> Result<(), Error> {
        let products = api_client.get_products().await?;
        self.registry = SymbolRegistry::from_products(&products);
        self.last_update = Instant::now();
        Ok(())
    }
}
```

## Example Conversions Reference

| Value Type | Raw (Scaled) | Scale | Actual | Notes |
|------------|--------------|-------|--------|-------|
| BTC Price (Contract) | 87700000 | 4 | 8770.0 USD | BTCUSD priceEp |
| BTC Price (Spot) | 8770000000 | 8 | 87.70 USDT | sBTCUSDT priceEp |
| BTC Balance | 100000000 | 8 | 1.0 BTC | balanceEv |
| USDT Balance | 1000000000 | 8 | 10.0 USDT | balanceEv |
| Leverage (20x Isolated) | 2000000 | 8 | 0.02 | leverageEr (positive) |
| Leverage (Cross) | 0 | 8 | 0 | leverageEr (zero/negative) |
| Maker Fee (-0.025%) | -25000 | 8 | -0.00025 | makerFeeRateEr |
| Taker Fee (0.075%) | 75000 | 8 | 0.00075 | takerFeeRateEr |
| Position Value (USD) | 87700000 | 4 | 8770.0 USD | valueEv |
| Unrealized PnL (BTC) | 10000000 | 8 | 0.1 BTC | unrealisedPnlEv |

## Notes

1. **Always fetch product information** on startup to get scale factors
2. **Cache scale factors** per symbol for fast conversions
3. **Use integer arithmetic** when possible to avoid floating-point precision issues
4. **Validate tick size and lot size** before submitting orders
5. **Spot symbols ALWAYS start with lowercase `s`**
6. **Contract symbols NEVER have `s` prefix** (may have `u` for USDT-margined)
7. **Leverage sign matters:** positive = isolated, zero/negative = cross
