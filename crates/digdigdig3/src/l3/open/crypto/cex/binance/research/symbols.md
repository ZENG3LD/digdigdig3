# Binance Symbol Formats

## Overview

Binance uses different symbol formats across its trading platforms. Understanding these formats is crucial for proper API integration.

---

## Spot Trading Symbols

### Format

Spot symbols combine base and quote assets **without separator**:

```
{BASE}{QUOTE}
```

### Examples

| Base Asset | Quote Asset | Symbol |
|------------|-------------|--------|
| BTC | USDT | BTCUSDT |
| ETH | USDT | ETHUSDT |
| BNB | BTC | BNBBTC |
| LTC | BTC | LTCBTC |
| ETH | BTC | ETHBTC |
| XRP | USDT | XRPUSDT |
| ADA | USDT | ADAUSDT |
| SOL | USDT | SOLUSDT |

### Rules

- **Case**: Always uppercase
- **Separator**: None (no dash, slash, or underscore)
- **Quote Asset**: Commonly USDT, BTC, ETH, BNB, BUSD
- **Length**: Variable (typically 6-10 characters)

### Common Quote Assets

- **USDT**: Tether USD (most common)
- **BUSD**: Binance USD (being phased out)
- **BTC**: Bitcoin
- **ETH**: Ethereum
- **BNB**: Binance Coin
- **USDC**: USD Coin
- **TUSD**: TrueUSD
- **DAI**: Dai Stablecoin

---

## Futures USDT-M Symbols

### Format

USDT-margined futures use the **same format as spot**:

```
{BASE}{QUOTE}
```

### Examples

| Symbol | Type | Description |
|--------|------|-------------|
| BTCUSDT | Perpetual | Bitcoin perpetual contract |
| ETHUSDT | Perpetual | Ethereum perpetual contract |
| BNBUSDT | Perpetual | BNB perpetual contract |
| ADAUSDT | Perpetual | Cardano perpetual contract |

### Perpetual vs Delivery

**Perpetual Contracts**:
- Symbol: `BTCUSDT`
- No expiry date
- Most common type
- Funding rate mechanism

**Quarterly Delivery** (less common):
- Symbol format: `{BASE}USDT_{YYMMDD}` (e.g., `BTCUSDT_231229`)
- Has expiry date
- Settles on expiry

### Rules

- **Quote Asset**: Always USDT for USDT-M futures
- **Case**: Always uppercase
- **Separator**: None for perpetuals
- **Delivery**: Uses underscore separator with date

---

## Futures COIN-M Symbols

### Format

COIN-margined (delivery) futures use **underscore separator**:

```
{BASE}USD_{YYMMDD}
```

For perpetuals:
```
{BASE}USD_PERP
```

### Examples

| Symbol | Type | Description |
|--------|------|-------------|
| BTCUSD_PERP | Perpetual | Bitcoin perpetual (COIN-M) |
| ETHUSD_PERP | Perpetual | Ethereum perpetual (COIN-M) |
| BTCUSD_231229 | Delivery | Bitcoin quarterly ending Dec 29, 2023 |
| ETHUSD_240329 | Delivery | Ethereum quarterly ending Mar 29, 2024 |

### Rules

- **Quote**: USD (not USDT)
- **Separator**: Underscore `_`
- **Perpetual**: `_PERP` suffix
- **Delivery**: `_YYMMDD` suffix (expiry date)
- **Margin**: Paid in base asset (e.g., BTC, ETH)

---

## Symbol Validation

### Valid Characters

- Uppercase letters: `A-Z`
- Numbers: `0-9`
- Underscore: `_` (COIN-M futures only)

### Invalid Characters

- Lowercase letters: `a-z`
- Hyphens: `-`
- Slashes: `/`
- Spaces
- Special characters

---

## Symbol Conversion

### External Format to Binance

Many trading platforms use different formats:

| External Format | Binance Format | Platform |
|-----------------|----------------|----------|
| BTC/USDT | BTCUSDT | Common |
| BTC-USDT | BTCUSDT | Some exchanges |
| btcusdt | BTCUSDT | Lowercase input |
| XBT/USD | BTCUSDT | BitMEX notation |

### Conversion Function (Rust)

```rust
/// Convert common symbol formats to Binance format
pub fn normalize_symbol(symbol: &str) -> String {
    symbol
        .to_uppercase()
        .replace("/", "")
        .replace("-", "")
        .replace("XBT", "BTC")  // BitMEX notation
        .replace(" ", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(normalize_symbol("btc/usdt"), "BTCUSDT");
        assert_eq!(normalize_symbol("BTC-USDT"), "BTCUSDT");
        assert_eq!(normalize_symbol("eth/btc"), "ETHBTC");
        assert_eq!(normalize_symbol("xbt/usd"), "BTCUSD");
    }
}
```

---

## Exchange Info Endpoint

To get all valid symbols, use the exchange information endpoint:

### Spot

**Endpoint**: `GET /api/v3/exchangeInfo`

**Response** (excerpt):
```json
{
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "status": "TRADING",
      "baseAsset": "BTC",
      "baseAssetPrecision": 8,
      "quoteAsset": "USDT",
      "quotePrecision": 8,
      "quoteAssetPrecision": 8,
      "orderTypes": ["LIMIT", "MARKET"],
      "icebergAllowed": true,
      "ocoAllowed": true,
      "quoteOrderQtyMarketAllowed": true,
      "allowTrailingStop": true,
      "isSpotTradingAllowed": true,
      "isMarginTradingAllowed": true
    }
  ]
}
```

### Futures USDT-M

**Endpoint**: `GET /fapi/v1/exchangeInfo`

**Response** (excerpt):
```json
{
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "pair": "BTCUSDT",
      "contractType": "PERPETUAL",
      "deliveryDate": 4133404800000,
      "onboardDate": 1569398400000,
      "status": "TRADING",
      "baseAsset": "BTC",
      "quoteAsset": "USDT",
      "marginAsset": "USDT",
      "pricePrecision": 2,
      "quantityPrecision": 3,
      "baseAssetPrecision": 8,
      "quotePrecision": 8,
      "underlyingType": "COIN",
      "orderTypes": ["LIMIT", "MARKET", "STOP", "STOP_MARKET"]
    }
  ]
}
```

### Futures COIN-M

**Endpoint**: `GET /dapi/v1/exchangeInfo`

---

## Symbol Status

Symbols can have different status values:

- **TRADING**: Normal trading
- **HALT**: Trading halted temporarily
- **BREAK**: Trading break (maintenance)
- **AUCTION_MATCH**: Auction matching
- **PRE_TRADING**: Pre-trading phase
- **POST_TRADING**: Post-trading phase
- **END_OF_DAY**: End of day
- **CLOSE**: Trading closed

**Note**: Only use symbols with status `TRADING` for active trading.

---

## Symbol Precision

Each symbol has different precision requirements:

### Price Precision

Number of decimal places allowed for price:

```json
{
  "symbol": "BTCUSDT",
  "pricePrecision": 2
}
```

Valid prices: `50000.00`, `50000.50`, `50000.99`
Invalid: `50000.001`, `50000.1234`

### Quantity Precision

Number of decimal places allowed for quantity:

```json
{
  "symbol": "BTCUSDT",
  "quantityPrecision": 5
}
```

Valid quantities: `0.00100`, `1.23456`, `10.00000`
Invalid: `0.000001`, `1.234567`

### Precision Formatting (Rust)

```rust
use rust_decimal::Decimal;

pub fn format_price(price: Decimal, precision: u32) -> String {
    format!("{:.prec$}", price, prec = precision as usize)
}

pub fn format_quantity(quantity: Decimal, precision: u32) -> String {
    format!("{:.prec$}", quantity, prec = precision as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_format_price() {
        let price = Decimal::from_str("50000.123456").unwrap();
        assert_eq!(format_price(price, 2), "50000.12");
    }

    #[test]
    fn test_format_quantity() {
        let qty = Decimal::from_str("1.123456").unwrap();
        assert_eq!(format_quantity(qty, 5), "1.12346");
    }
}
```

---

## Lot Size Filters

Symbols also have lot size restrictions:

```json
{
  "filterType": "LOT_SIZE",
  "minQty": "0.00100000",
  "maxQty": "9000.00000000",
  "stepSize": "0.00100000"
}
```

**Rules**:
- `quantity >= minQty`
- `quantity <= maxQty`
- `(quantity - minQty) % stepSize == 0`

---

## Notional Filters

Minimum notional value (price × quantity):

```json
{
  "filterType": "NOTIONAL",
  "minNotional": "10.00000000",
  "applyMinToMarket": true,
  "maxNotional": "9000000.00000000",
  "applyMaxToMarket": false,
  "avgPriceMins": 5
}
```

**Rules**:
- `price * quantity >= minNotional` (for LIMIT orders)
- `price * quantity <= maxNotional`

---

## Symbol Mapping

### Internal Symbol Storage

Store symbols in a consistent format internally:

```rust
use std::collections::HashMap;

pub struct SymbolInfo {
    pub exchange_symbol: String,    // "BTCUSDT"
    pub base_asset: String,          // "BTC"
    pub quote_asset: String,         // "USDT"
    pub price_precision: u32,
    pub quantity_precision: u32,
    pub min_qty: String,
    pub max_qty: String,
    pub step_size: String,
    pub min_notional: String,
}

pub struct SymbolMapper {
    symbols: HashMap<String, SymbolInfo>,
}

impl SymbolMapper {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, info: SymbolInfo) {
        self.symbols.insert(info.exchange_symbol.clone(), info);
    }

    pub fn get_symbol(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.get(symbol)
    }

    pub fn normalize_and_get(&self, symbol: &str) -> Option<&SymbolInfo> {
        let normalized = normalize_symbol(symbol);
        self.get_symbol(&normalized)
    }
}
```

---

## Best Practices

1. **Always Uppercase**: Convert input symbols to uppercase
2. **Validate**: Check symbol exists via `/exchangeInfo` before trading
3. **Cache Symbol Info**: Store symbol precision and filters locally
4. **Respect Precision**: Format prices and quantities correctly
5. **Check Status**: Only trade symbols with `TRADING` status
6. **Handle Delisting**: Monitor for symbol delistings and migrations

---

## Differences Summary

| Feature | Spot | Futures USDT-M | Futures COIN-M |
|---------|------|----------------|----------------|
| Format | BTCUSDT | BTCUSDT | BTCUSD_PERP |
| Separator | None | None | Underscore |
| Quote Asset | Various | USDT | USD |
| Margin | Spot | USDT | Coin (BTC/ETH) |
| Contract Type | N/A | Perpetual/Delivery | Perpetual/Delivery |
| Endpoint | /api/v3/* | /fapi/v1/* | /dapi/v1/* |

---

## Reference

For the latest symbol information, always check:
- Spot: `GET /api/v3/exchangeInfo`
- Futures USDT-M: `GET /fapi/v1/exchangeInfo`
- Futures COIN-M: `GET /dapi/v1/exchangeInfo`

Official documentation:
- [Binance Spot API Docs](https://developers.binance.com/docs/binance-spot-api-docs/rest-api)
- [Binance Futures API Docs](https://developers.binance.com/docs/derivatives/usds-margined-futures)
