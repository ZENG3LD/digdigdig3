# Lighter Exchange Market Symbols and Identifiers

## Overview

Lighter is a hybrid orderbook DEX supporting both perpetual futures and spot markets. The primary quote currency is **USDC** across most markets.

---

## Symbol Format

### Perpetual Futures

**Format**: `BASE` (single asset symbol)

**Examples**:
- `ETH` - Ethereum perpetual
- `BTC` - Bitcoin perpetual
- `SOL` - Solana perpetual

**Implied Quote**: All perpetual markets are quoted in USDC

**Full Representation**: When displaying to users, represent as `BASE/USDC` or `BASE-PERP`

---

### Spot Markets

**Format**: `BASE/QUOTE`

**Examples**:
- `ETH/USDC` - Ethereum spot
- `BTC/USDC` - Bitcoin spot

**Quote Currency**: Primarily USDC

---

## Market Identification

### Market ID

Each market has a unique **market_id** (integer).

**Perpetual Markets**: `market_id` ranges from 0 upwards
**Spot Markets**: `market_id` typically starts from 2048 (2^11)

**Special Values**:
- `255` - Used in API queries to mean "all markets"

**Examples**:
```json
{
  "symbol": "ETH",
  "market_id": 0,
  "market_type": "perp"
}
```

```json
{
  "symbol": "ETH/USDC",
  "market_id": 2048,
  "market_type": "spot"
}
```

---

### Market Type

**Field**: `market_type`

**Values**:
- `"perp"` - Perpetual futures
- `"spot"` - Spot trading

---

### Asset IDs

Each asset (base or quote) has a unique **asset_id** (integer).

**Common Asset IDs**:
- `1` - USDC (primary quote currency)
- Additional asset IDs assigned sequentially

**Example**:
```json
{
  "symbol": "ETH",
  "base_asset_id": 0,
  "quote_asset_id": 1,  // USDC
  "market_type": "perp"
}
```

---

## Market Discovery

### Get All Markets

**Endpoint**: `GET /api/v1/orderBooks`

**Parameters**:
- `market_id` (optional, default: 255 for all)

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "order_books": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 1,
      "status": "active"
    },
    {
      "symbol": "BTC",
      "market_id": 1,
      "market_type": "perp",
      "base_asset_id": 2,
      "quote_asset_id": 1,
      "status": "active"
    }
  ]
}
```

---

### Get Market Details

**Endpoint**: `GET /api/v1/orderBookDetails`

**Parameters**:
- `market_id` (optional, default: 255 for all)
- `filter` (optional): "all", "spot", "perp"

**Response**:
```json
{
  "code": 200,
  "message": "string",
  "order_book_details": [
    {
      "symbol": "ETH",
      "market_id": 0,
      "market_type": "perp",
      "base_asset_id": 0,
      "quote_asset_id": 1,
      "status": "active",
      "min_base_amount": "0.01",
      "min_quote_amount": "0.1",
      "supported_size_decimals": 4,
      "supported_price_decimals": 4
    }
  ],
  "spot_order_book_details": [
    {
      "symbol": "ETH/USDC",
      "market_id": 2048,
      "market_type": "spot",
      "base_asset_id": 1,
      "quote_asset_id": 3,
      "status": "active"
    }
  ]
}
```

**Note**: Perpetual and spot markets are returned in separate arrays.

---

## Market Coverage

### Total Markets

As of 2026, Lighter offers approximately **124 trading pairs**.

### Market Categories

1. **Crypto Perpetuals**:
   - Major cryptocurrencies: BTC, ETH, SOL, etc.
   - Altcoins and DeFi tokens
   - Primary focus of the platform

2. **Real-World Assets (RWA)**:
   - Gold via PAXG
   - Leverage: up to 8x
   - Quote: USDC

3. **Forex Perpetuals**:
   - Major currency pairs: EUR/USD, GBP/USD
   - Leverage: up to 25x
   - Quote: USDC

### Most Active Pairs

**Top Volume**:
- `BTC/USDC` - Most active trading pair
- `ETH/USDC` - Second most active

---

## Symbol Normalization

### Input Normalization

When users provide symbols, normalize to Lighter format:

**User Input** → **Lighter Format**
- `BTCUSDC` → `BTC` (for perp) or `BTC/USDC` (for spot)
- `ETH-USDC` → `ETH` (for perp) or `ETH/USDC` (for spot)
- `eth` → `ETH` (uppercase)
- `ETHPERP` → `ETH` (remove suffix)
- `ETH-PERP` → `ETH` (remove suffix)

**Algorithm**:
1. Convert to uppercase
2. Remove separators if present (-, _, etc.)
3. Check if ends with "USDC" and split
4. Check if ends with "PERP" and remove
5. For perpetuals, use base symbol only
6. For spot, use `BASE/QUOTE` format

---

### Output Formatting

When displaying symbols to users, consider context:

**For Perpetuals**:
- API format: `"ETH"`
- User display: `"ETH/USDC"` or `"ETH-PERP"` or `"ETHUSDC-PERP"`

**For Spot**:
- API format: `"ETH/USDC"`
- User display: `"ETH/USDC"` or `"ETHUSDC"`

---

## Market Status

**Field**: `status`

**Values**:
- `"active"` - Market is open for trading
- `"inactive"` - Market is suspended/closed

**Check Before Trading**:
Always verify market status before submitting orders to avoid rejections.

---

## Decimal Precision

Each market specifies decimal precision for sizes and prices.

### Fields

- `supported_size_decimals` - Decimal places for order size
- `supported_price_decimals` - Decimal places for price
- `supported_quote_decimals` - Decimal places for quote
- `size_decimals` - Actual decimals used for size
- `price_decimals` - Actual decimals used for price

**Example**:
```json
{
  "symbol": "ETH",
  "supported_size_decimals": 4,
  "supported_price_decimals": 4,
  "size_decimals": 4,
  "price_decimals": 4,
  "quote_multiplier": 10000
}
```

### Precision Conversion

**For Submissions (API expects integers)**:
- Size: `1.5` → `15000` (multiply by 10^size_decimals)
- Price: `3024.66` → `30246600` (multiply by 10^price_decimals)

**For Display (API returns strings)**:
- Size: `"15000"` → `1.5` (divide by 10^size_decimals)
- Price: `"30246600"` → `3024.66` (divide by 10^price_decimals)

**Helper Formula**:
```
multiplier = 10^decimals
integer_value = decimal_value * multiplier
decimal_value = integer_value / multiplier
```

---

## Order Size Limits

### Minimum Order Size

Each market defines minimum order sizes.

**Fields**:
- `min_base_amount` - Minimum order size in base asset
- `min_quote_amount` - Minimum order value in quote asset

**Example**:
```json
{
  "symbol": "ETH",
  "min_base_amount": "0.01",
  "min_quote_amount": "0.1"
}
```

**Validation**:
- Order size must be >= `min_base_amount`
- Order value (size * price) must be >= `min_quote_amount`

---

### Maximum Order Size

**Field**: `order_quote_limit`

**Description**: Maximum order value in quote currency

**Example**:
```json
{
  "symbol": "ETH",
  "order_quote_limit": "281474976.710655"
}
```

**Validation**:
- Order value (size * price) must be <= `order_quote_limit`

---

## Market Configuration

### Margin Requirements (Perpetuals Only)

**Fields**:
- `default_initial_margin_fraction` - Default margin requirement (basis points)
- `min_initial_margin_fraction` - Minimum margin requirement (basis points)
- `maintenance_margin_fraction` - Maintenance margin level (basis points)
- `closeout_margin_fraction` - Closeout threshold (basis points)

**Basis Points**: 100 basis points = 1%

**Example**:
```json
{
  "default_initial_margin_fraction": 100,    // 1% = 100x leverage
  "min_initial_margin_fraction": 100,        // 1% minimum
  "maintenance_margin_fraction": 50,         // 0.5% maintenance
  "closeout_margin_fraction": 100            // 1% closeout
}
```

**Leverage Calculation**:
```
leverage = 10000 / initial_margin_fraction
```

**Examples**:
- `100` basis points → 100x leverage
- `200` basis points → 50x leverage
- `500` basis points → 20x leverage
- `1000` basis points → 10x leverage
- `400` basis points → 25x leverage (forex)
- `1250` basis points → 8x leverage (RWA)

---

### Fee Structure

**Fields**:
- `taker_fee` - Taker fee as decimal string
- `maker_fee` - Maker fee as decimal string
- `liquidation_fee` - Liquidation fee as decimal string

**Account Types**:

**Standard Account** (Free):
- Maker: 0 bps (0%)
- Taker: 0 bps (0%)
- Fee-free trading

**Premium Account**:
- Maker: 0.2 bps (0.002%)
- Taker: 2 bps (0.02%)

**Example**:
```json
{
  "taker_fee": "0.0001",      // 0.01% for premium
  "maker_fee": "0.0000",      // 0% for premium
  "liquidation_fee": "0.01"   // 1%
}
```

---

## Market Statistics

Market details include 24-hour statistics.

**Fields**:
- `last_trade_price` - Most recent trade price
- `daily_trades_count` - Number of trades in last 24h
- `daily_base_token_volume` - 24h volume in base asset
- `daily_quote_token_volume` - 24h volume in quote asset
- `daily_price_low` - 24h low price
- `daily_price_high` - 24h high price
- `daily_price_change` - 24h price change
- `open_interest` - Current open interest (perpetuals only)

**Example**:
```json
{
  "symbol": "ETH",
  "last_trade_price": 3024.66,
  "daily_trades_count": 68,
  "daily_base_token_volume": 235.25,
  "daily_quote_token_volume": 93566.25,
  "daily_price_low": 3014.66,
  "daily_price_high": 3024.66,
  "daily_price_change": 3.66,
  "open_interest": 93.0
}
```

---

## Symbol Lookup Functions

### Example Implementation (Rust)

```rust
use std::collections::HashMap;

pub struct MarketInfo {
    pub symbol: String,
    pub market_id: u16,
    pub market_type: String,  // "perp" or "spot"
    pub base_asset_id: u16,
    pub quote_asset_id: u16,
    pub size_decimals: u8,
    pub price_decimals: u8,
    pub min_base_amount: String,
    pub min_quote_amount: String,
}

pub struct SymbolMapper {
    markets: HashMap<String, MarketInfo>,
    id_to_symbol: HashMap<u16, String>,
}

impl SymbolMapper {
    pub fn normalize_symbol(input: &str) -> String {
        let upper = input.to_uppercase();

        // Remove common separators
        let clean = upper.replace("-", "").replace("_", "");

        // Remove PERP suffix if present
        let clean = if clean.ends_with("PERP") {
            &clean[..clean.len()-4]
        } else {
            &clean
        };

        // Check if ends with USDC (spot market)
        if clean.ends_with("USDC") && clean.len() > 4 {
            let base = &clean[..clean.len()-4];
            format!("{}/USDC", base)
        } else {
            clean.to_string()
        }
    }

    pub fn get_market_id(&self, symbol: &str) -> Option<u16> {
        let normalized = Self::normalize_symbol(symbol);
        self.markets.get(&normalized).map(|m| m.market_id)
    }

    pub fn get_symbol(&self, market_id: u16) -> Option<&str> {
        self.id_to_symbol.get(&market_id).map(|s| s.as_str())
    }

    pub fn format_for_display(&self, symbol: &str, perp_format: &str) -> String {
        let market = self.markets.get(symbol);

        match market {
            Some(m) if m.market_type == "perp" => {
                match perp_format {
                    "suffix" => format!("{}-PERP", symbol),
                    "usdc" => format!("{}/USDC", symbol),
                    _ => symbol.to_string(),
                }
            },
            Some(m) if m.market_type == "spot" => symbol.to_string(),
            _ => symbol.to_string(),
        }
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(SymbolMapper::normalize_symbol("BTCUSDC"), "BTC/USDC");
        assert_eq!(SymbolMapper::normalize_symbol("ETH-PERP"), "ETH");
        assert_eq!(SymbolMapper::normalize_symbol("eth"), "ETH");
        assert_eq!(SymbolMapper::normalize_symbol("SOL-USDC"), "SOL/USDC");
    }
}
```

---

## API Query Examples

### Get Specific Market by ID

```http
GET /api/v1/orderBookDetails?market_id=0
```

**Response**: Single market (ETH perpetual with market_id=0)

---

### Get All Perpetual Markets

```http
GET /api/v1/orderBookDetails?filter=perp
```

**Response**: All perpetual markets

---

### Get All Spot Markets

```http
GET /api/v1/orderBookDetails?filter=spot
```

**Response**: All spot markets

---

### Get All Markets

```http
GET /api/v1/orderBookDetails?market_id=255&filter=all
```

**Response**: Both perpetual and spot markets

---

## Symbol Validation

Before submitting orders, validate:

1. **Market Exists**: Check symbol exists in market list
2. **Market Active**: Verify `status == "active"`
3. **Market Type**: Ensure correct market type (perp vs spot)
4. **Size Limits**: Check against `min_base_amount` and `order_quote_limit`
5. **Precision**: Round to `size_decimals` and `price_decimals`

**Example Validation**:
```rust
pub fn validate_order(
    market: &MarketInfo,
    size: f64,
    price: f64,
) -> Result<(), String> {
    // Check market status
    if market.status != "active" {
        return Err("Market is not active".to_string());
    }

    // Check minimum size
    let min_size: f64 = market.min_base_amount.parse().unwrap();
    if size < min_size {
        return Err(format!("Size {} below minimum {}", size, min_size));
    }

    // Check minimum quote value
    let min_quote: f64 = market.min_quote_amount.parse().unwrap();
    let quote_value = size * price;
    if quote_value < min_quote {
        return Err(format!("Order value {} below minimum {}", quote_value, min_quote));
    }

    // Check maximum quote value
    let max_quote: f64 = market.order_quote_limit.parse().unwrap();
    if quote_value > max_quote {
        return Err(format!("Order value {} exceeds maximum {}", quote_value, max_quote));
    }

    Ok(())
}
```

---

## Special Considerations

### Market Updates

Markets may be added, removed, or modified. Implement:
- Periodic refresh of market list (e.g., every hour)
- Cache market information
- Handle unknown market_id gracefully

### Market Outages

Markets can become inactive temporarily:
- Check `status` field before trading
- Handle "market inactive" errors gracefully
- Retry with backoff when market reopens

### Symbol Ambiguity

Avoid ambiguity between spot and perp:
- Store market_type with symbol
- Default to perpetual when ambiguous (most common)
- Let users specify market type explicitly

---

## Reference Data

### Example Market List (Perpetuals)

| Symbol | Market ID | Base Asset ID | Quote Asset ID | Decimals | Min Size | Leverage |
|--------|-----------|---------------|----------------|----------|----------|----------|
| BTC    | 1         | 2             | 1 (USDC)       | 4        | 0.001    | 100x     |
| ETH    | 0         | 0             | 1 (USDC)       | 4        | 0.01     | 100x     |
| SOL    | 2         | 3             | 1 (USDC)       | 4        | 0.1      | 100x     |
| PAXG   | N/A       | N/A           | 1 (USDC)       | 4        | 0.01     | 8x       |

### Example Market List (Spot)

| Symbol    | Market ID | Base Asset ID | Quote Asset ID | Decimals | Min Size |
|-----------|-----------|---------------|----------------|----------|----------|
| ETH/USDC  | 2048      | 1             | 3              | 4        | 0.01     |
| BTC/USDC  | 2049      | 2             | 3              | 4        | 0.001    |

**Note**: Actual values should be fetched from the API, as they may change.

---

## Implementation Checklist

- [ ] Fetch and cache market list on startup
- [ ] Implement symbol normalization function
- [ ] Implement market_id lookup by symbol
- [ ] Implement symbol lookup by market_id
- [ ] Validate market status before trading
- [ ] Validate order sizes against limits
- [ ] Handle decimal precision conversion
- [ ] Implement periodic market list refresh
- [ ] Handle unknown symbols gracefully
- [ ] Support both perpetual and spot markets
- [ ] Display symbols in user-friendly format
