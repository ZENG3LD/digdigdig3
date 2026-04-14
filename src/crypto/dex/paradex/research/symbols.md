# Paradex Market Symbols

## Symbol Format

Paradex uses a **hyphen-separated** format for market symbols:

```
{BASE}-{QUOTE}-{TYPE}
```

### Components

| Component | Description | Examples |
|-----------|-------------|----------|
| **BASE** | Base asset/cryptocurrency | BTC, ETH, SOL, ARB |
| **QUOTE** | Quote currency | USD |
| **TYPE** | Instrument type | PERP, PERP_OPTION |

---

## Examples

### Perpetual Futures
- `BTC-USD-PERP`: Bitcoin perpetual futures
- `ETH-USD-PERP`: Ethereum perpetual futures
- `SOL-USD-PERP`: Solana perpetual futures
- `ARB-USD-PERP`: Arbitrum perpetual futures
- `AVAX-USD-PERP`: Avalanche perpetual futures

### Perpetual Options
- `BTC-USD-PERP_OPTION`: Bitcoin perpetual options
- `ETH-USD-PERP_OPTION`: Ethereum perpetual options

**Note**: Perpetual options include additional fields:
- `option_type`: "PUT" or "CALL"
- `strike_price`: Strike price for the option
- `expiry_at`: Expiration timestamp (may be null for perpetuals)

---

## Symbol Validation

### Valid Format Rules

1. **Three components** separated by hyphens
2. **BASE**: Uppercase letters (e.g., BTC, ETH)
3. **QUOTE**: Uppercase letters (typically USD)
4. **TYPE**: PERP or PERP_OPTION

### Regex Pattern

```regex
^[A-Z0-9]+\-[A-Z]+\-(PERP|PERP_OPTION)$
```

### Example Validation (Rust)

```rust
fn validate_symbol(symbol: &str) -> bool {
    let parts: Vec<&str> = symbol.split('-').collect();

    if parts.len() != 3 {
        return false;
    }

    let base = parts[0];
    let quote = parts[1];
    let instrument_type = parts[2];

    !base.is_empty()
        && !quote.is_empty()
        && (instrument_type == "PERP" || instrument_type == "PERP_OPTION")
        && base.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && quote.chars().all(|c| c.is_ascii_uppercase())
}
```

---

## Symbol Discovery

### GET /markets

Retrieve all available markets:

**Request**:
```
GET https://api.prod.paradex.trade/v1/markets
```

**Response**:
```json
{
  "results": [
    {
      "symbol": "BTC-USD-PERP",
      "base_currency": "BTC",
      "quote_currency": "USD",
      "settlement_currency": "USDC",
      "asset_kind": "PERP",
      "market_kind": "cross",
      ...
    },
    {
      "symbol": "ETH-USD-PERP",
      "base_currency": "ETH",
      "quote_currency": "USD",
      "settlement_currency": "USDC",
      "asset_kind": "PERP",
      "market_kind": "cross",
      ...
    }
  ]
}
```

### Filter by Symbol

**Request**:
```
GET https://api.prod.paradex.trade/v1/markets?market=BTC-USD-PERP
```

**Response**: Returns only BTC-USD-PERP market data

---

## Symbol Components

### Base Currency

The **base currency** is the asset being traded:

**Common Base Currencies**:
- **BTC**: Bitcoin
- **ETH**: Ethereum
- **SOL**: Solana
- **ARB**: Arbitrum
- **AVAX**: Avalanche
- **OP**: Optimism
- **MATIC**: Polygon
- **DOGE**: Dogecoin
- **XRP**: Ripple
- **ADA**: Cardano

**Note**: Available markets determined by Paradex. Check `/markets` endpoint for current list.

### Quote Currency

The **quote currency** is what the base is priced in:

**Primary Quote Currency**:
- **USD**: US Dollar (most common)

**Future Support**: Paradex may add other quote currencies (USDC, USDT, etc.)

### Settlement Currency

While quote is USD, **settlement** typically occurs in:

- **USDC**: USD Coin (on StarkNet)

**Example**:
- Symbol: `BTC-USD-PERP`
- Quote: USD (prices shown in USD)
- Settlement: USDC (profits/losses settled in USDC)

---

## Instrument Types

### PERP (Perpetual Futures)

**Characteristics**:
- No expiration date
- Funding rate mechanism (every 8 hours typically)
- Linear contracts (settled in USDC)
- Cross-margin or isolated-margin

**Fields**:
```json
{
  "asset_kind": "PERP",
  "funding_period_hours": 8,
  "funding_multiplier": "1.0",
  "max_funding_rate": "0.0005",
  "interest_rate": "0.0001"
}
```

### PERP_OPTION (Perpetual Options)

**Characteristics**:
- Perpetual option contracts
- PUT or CALL types
- Strike price defined
- Greeks calculated (delta, gamma, vega, etc.)

**Fields**:
```json
{
  "asset_kind": "PERP_OPTION",
  "option_type": "CALL",
  "strike_price": "70000.00",
  "iv_bands_width": "0.15"
}
```

---

## Market Kinds

### Cross-Margin Markets

**Symbol Suffix**: None (default)

**Example**: `BTC-USD-PERP`

**Characteristics**:
- Margin shared across all positions
- More capital efficient
- Higher leverage possible
- Risk of full account liquidation

### Isolated Margin Markets

**Symbol Suffix**: May have specific identifier (check docs)

**Characteristics**:
- Margin isolated per position
- Limited risk to position margin
- Lower leverage
- Cannot affect other positions

**Market Kind Values**:
- `"cross"`: Cross-margin
- `"isolated"`: Isolated margin
- `"isolated_margin"`: Alternative isolated identifier

---

## Symbol Usage in API

### Path Parameters

Use symbol exactly as returned by `/markets`:

```
GET /v1/orderbook/BTC-USD-PERP
GET /v1/bbo/ETH-USD-PERP/interactive
```

### Query Parameters

```
GET /v1/markets?market=BTC-USD-PERP
GET /v1/markets/summary?market=ALL
```

### Request Bodies

```json
{
  "market": "BTC-USD-PERP",
  "side": "BUY",
  "type": "LIMIT",
  "size": "0.5",
  "price": "65000.00",
  ...
}
```

### WebSocket Channels

```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "trades.BTC-USD-PERP"
  }
}
```

Or subscribe to all markets:

```json
{
  "params": {
    "channel": "trades.ALL"
  }
}
```

---

## Symbol Normalization

### Case Sensitivity

Paradex symbols are **case-sensitive** and always **uppercase**:

- ✅ `BTC-USD-PERP`
- ❌ `btc-usd-perp`
- ❌ `Btc-Usd-Perp`

### Normalization Function

```rust
fn normalize_symbol(input: &str) -> String {
    input.to_uppercase().trim().to_string()
}

// Usage
let user_input = "btc-usd-perp";
let normalized = normalize_symbol(user_input); // "BTC-USD-PERP"
```

### Symbol Parsing

```rust
struct Symbol {
    base: String,
    quote: String,
    instrument_type: InstrumentType,
}

enum InstrumentType {
    Perp,
    PerpOption,
}

impl Symbol {
    fn parse(symbol: &str) -> Result<Self, ParseError> {
        let parts: Vec<&str> = symbol.split('-').collect();

        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat);
        }

        let instrument_type = match parts[2] {
            "PERP" => InstrumentType::Perp,
            "PERP_OPTION" => InstrumentType::PerpOption,
            _ => return Err(ParseError::InvalidInstrumentType),
        };

        Ok(Symbol {
            base: parts[0].to_string(),
            quote: parts[1].to_string(),
            instrument_type,
        })
    }

    fn to_string(&self) -> String {
        let type_str = match self.instrument_type {
            InstrumentType::Perp => "PERP",
            InstrumentType::PerpOption => "PERP_OPTION",
        };

        format!("{}-{}-{}", self.base, self.quote, type_str)
    }
}
```

---

## Symbol Mapping

### From Other Exchanges

When integrating with multiple exchanges, map symbols:

| Paradex | Binance | Bybit | OKX |
|---------|---------|-------|-----|
| `BTC-USD-PERP` | `BTCUSDT` | `BTCUSDT` | `BTC-USDT-SWAP` |
| `ETH-USD-PERP` | `ETHUSDT` | `ETHUSDT` | `ETH-USDT-SWAP` |
| `SOL-USD-PERP` | `SOLUSDT` | `SOLUSDT` | `SOL-USDT-SWAP` |

### Conversion Function

```rust
fn to_paradex_symbol(exchange: &str, symbol: &str) -> Option<String> {
    match exchange {
        "binance" => {
            // BTCUSDT -> BTC-USD-PERP
            if symbol.ends_with("USDT") {
                let base = symbol.strip_suffix("USDT")?;
                Some(format!("{}-USD-PERP", base))
            } else {
                None
            }
        }
        "okx" => {
            // BTC-USDT-SWAP -> BTC-USD-PERP
            if symbol.ends_with("-USDT-SWAP") {
                let base = symbol.strip_suffix("-USDT-SWAP")?;
                Some(format!("{}-USD-PERP", base))
            } else {
                None
            }
        }
        _ => None,
    }
}
```

---

## Market Information

### Market Static Data

From `GET /markets` response:

```json
{
  "symbol": "BTC-USD-PERP",
  "base_currency": "BTC",
  "quote_currency": "USD",
  "settlement_currency": "USDC",
  "price_tick_size": "0.1",
  "order_size_increment": "0.001",
  "min_notional": "10",
  "max_order_size": "1000000",
  "position_limit": "10000000"
}
```

**Key Fields for Symbol Usage**:

| Field | Description | Example |
|-------|-------------|---------|
| `price_tick_size` | Minimum price increment | "0.1" (BTC can be $65432.1, not $65432.15) |
| `order_size_increment` | Minimum size increment | "0.001" (can order 0.001, 0.002, not 0.0015) |
| `min_notional` | Minimum order value | "10" (order must be ≥ $10) |

### Price/Size Rounding

```rust
use rust_decimal::Decimal;
use std::str::FromStr;

fn round_to_tick(price: Decimal, tick_size: &str) -> Decimal {
    let tick = Decimal::from_str(tick_size).unwrap();
    (price / tick).round() * tick
}

fn round_to_increment(size: Decimal, increment: &str) -> Decimal {
    let incr = Decimal::from_str(increment).unwrap();
    (size / incr).round() * incr
}

// Usage
let price = Decimal::from_str("65432.15").unwrap();
let rounded_price = round_to_tick(price, "0.1"); // 65432.1

let size = Decimal::from_str("0.1234").unwrap();
let rounded_size = round_to_increment(size, "0.001"); // 0.123
```

---

## Symbol Aliases

**Note**: Paradex does not appear to support symbol aliases. Always use the canonical format:

- ✅ `BTC-USD-PERP`
- ❌ `BTCUSD`
- ❌ `BTCPERP`
- ❌ `BTC/USD`

---

## Common Symbols (as of 2026)

### Tier 1 Assets
- `BTC-USD-PERP` - Bitcoin
- `ETH-USD-PERP` - Ethereum

### Major Altcoins
- `SOL-USD-PERP` - Solana
- `ARB-USD-PERP` - Arbitrum
- `OP-USD-PERP` - Optimism
- `AVAX-USD-PERP` - Avalanche
- `MATIC-USD-PERP` - Polygon

### Popular Tokens
- `DOGE-USD-PERP` - Dogecoin
- `XRP-USD-PERP` - Ripple
- `ADA-USD-PERP` - Cardano
- `DOT-USD-PERP` - Polkadot
- `LINK-USD-PERP` - Chainlink

**Note**: This list is not exhaustive. Use `GET /markets` for the current complete list.

---

## Symbol Caching

### Best Practices

1. **Cache market list** from `GET /markets` on startup
2. **Refresh periodically** (e.g., every hour) to detect new markets
3. **Validate symbols** against cache before making requests
4. **Handle new markets** gracefully (don't assume fixed list)

### Example Cache Implementation

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct MarketCache {
    markets: Arc<RwLock<HashMap<String, MarketInfo>>>,
}

impl MarketCache {
    async fn refresh(&self) -> Result<(), Error> {
        let markets_response = fetch_markets().await?;

        let mut cache = self.markets.write().await;
        cache.clear();

        for market in markets_response.results {
            cache.insert(market.symbol.clone(), market);
        }

        Ok(())
    }

    async fn get(&self, symbol: &str) -> Option<MarketInfo> {
        let cache = self.markets.read().await;
        cache.get(symbol).cloned()
    }

    async fn exists(&self, symbol: &str) -> bool {
        let cache = self.markets.read().await;
        cache.contains_key(symbol)
    }
}
```

---

## WebSocket Channel Symbols

### Market-Specific Channels

**Format**: `{channel}.{symbol}`

**Examples**:
- `trades.BTC-USD-PERP` - Trades for BTC perpetual
- `order_book.ETH-USD-PERP` - Order book for ETH perpetual
- `bbo.SOL-USD-PERP` - Best bid/offer for SOL perpetual

### All Markets

**Format**: `{channel}.ALL`

**Examples**:
- `trades.ALL` - All trades across all markets
- `fills.ALL` - All fills for account across all markets

### Subscription Examples

```json
// Single market
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "trades.BTC-USD-PERP"
  },
  "id": 1
}

// All markets
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "trades.ALL"
  },
  "id": 2
}
```

---

## Error Handling

### Invalid Symbol

**Request**:
```
GET /v1/orderbook/INVALID-SYMBOL
```

**Response (400)**:
```json
{
  "error": "INVALID_PARAMETER",
  "message": "Invalid market symbol",
  "details": {
    "parameter": "market",
    "value": "INVALID-SYMBOL"
  }
}
```

### Market Not Found

**Request**:
```
GET /v1/orderbook/UNKNOWN-USD-PERP
```

**Response (404)**:
```json
{
  "error": "NOT_FOUND",
  "message": "Market not found",
  "details": {
    "market": "UNKNOWN-USD-PERP"
  }
}
```

---

## Summary

1. **Format**: `{BASE}-{QUOTE}-{TYPE}`
2. **Example**: `BTC-USD-PERP`
3. **Case**: Always uppercase
4. **Separator**: Hyphen (`-`)
5. **Types**: `PERP`, `PERP_OPTION`
6. **Quote**: Typically `USD`
7. **Settlement**: Usually `USDC`
8. **Discovery**: `GET /markets`
9. **Validation**: Use regex or parsing function
10. **WebSocket**: `{channel}.{symbol}` or `{channel}.ALL`

---

## Additional Resources

- **Markets List**: https://docs.paradex.trade/api/prod/markets/get-markets
- **Market Summary**: https://docs.paradex.trade/api/prod/markets/get-markets-summary
- **Symbol Documentation**: https://docs.paradex.trade/trading/instruments-guide
- **Python SDK**: https://github.com/tradeparadex/paradex-py (see symbol handling)
