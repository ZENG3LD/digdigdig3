# dYdX v4 Quantums and Subticks

## Overview

dYdX v4 uses an **integer-based** representation for prices and sizes at the protocol level to avoid floating-point precision issues. This system uses two key concepts:

1. **Quantums**: Smallest increment of position size
2. **Subticks**: Protocol-level price units

## Quantums (Position Size)

### Definition

**Quantum** = The smallest increment of position size, determined by the `atomicResolution` parameter.

**Formula**:
```
1 quantum = 10^(atomicResolution) base_asset
```

### Example: BTC-USD

**Parameters**:
- `atomicResolution`: -10

**Calculation**:
```
1 quantum = 10^(-10) BTC
1 quantum = 0.0000000001 BTC
```

**Converting Quantums to Size**:
```
human_readable_size = quantums × 10^(atomicResolution)
```

**Example**:
- 500,000,000 quantums × 10^(-10) = 0.05 BTC
- 1,000,000,000,000 quantums × 10^(-10) = 100 BTC

### Example: AVAX-USD

**Parameters**:
- `atomicResolution`: -7

**Calculation**:
```
1 quantum = 10^(-7) AVAX
1 quantum = 0.0000001 AVAX
```

**Example**:
- -500,000,000 quantums × 10^(-7) = -50 AVAX
- Note: Negative quantums = SHORT position

### Converting Human Size to Quantums

**Formula**:
```
quantums = size / 10^(atomicResolution)
quantums = size × 10^(-atomicResolution)
```

**Example (BTC-USD)**:
- Size: 1.5 BTC
- atomicResolution: -10
- quantums = 1.5 × 10^(10) = 15,000,000,000

**Rust Implementation**:
```rust
use rust_decimal::Decimal;

fn size_to_quantums(size: Decimal, atomic_resolution: i32) -> Result<u64, ExchangeError> {
    let multiplier = Decimal::from(10_i64.pow((-atomic_resolution) as u32));
    let quantums = size * multiplier;

    // Round to nearest integer
    let quantums_u64 = quantums.round().to_u64()
        .ok_or(ExchangeError::QuantumConversion)?;

    Ok(quantums_u64)
}

fn quantums_to_size(quantums: u64, atomic_resolution: i32) -> Decimal {
    let divisor = Decimal::from(10_i64.pow((-atomic_resolution) as u32));
    Decimal::from(quantums) / divisor
}

// Example usage
let size = Decimal::from_str("1.5")?; // 1.5 BTC
let atomic_resolution = -10;

let quantums = size_to_quantums(size, atomic_resolution)?;
// quantums = 15000000000

let back_to_size = quantums_to_size(quantums, atomic_resolution);
// back_to_size = 1.5
```

## Subticks (Price)

### Definition

**Subtick** = Protocol-level pricing unit, expressed as quote quantums divided by base quantums.

**Formula**:
```
1 subtick = 10^(quantum_conversion_exponent) USDC / 10^(atomicResolution) BASE
```

### Example: BTC-USD

**Parameters**:
- `atomicResolution`: -10 (for BTC)
- `quantum_conversion_exponent`: -9
- USDC `atomicResolution`: -6 (standard for USDC)

**Calculation**:
```
1 subtick = 10^(-9) / (10^(-10) × 10^(-6))
1 subtick = 10^(-9) / 10^(-16)
1 subtick = 10^7 × 10^(-16) = 10^(-9) USDC per 10^(-10) BTC
```

In practice, this is simplified:
```
1 subtick = 10^(-14) USDC / 10^(-10) BTC
```

**If BTC = 20,000 USD/BTC**:
- `subticksPerTick` might be 100,000
- 1 tick = 100,000 subticks = 1 USD increment

**If BTC = 200 USD/BTC** (hypothetical):
- `subticksPerTick` might be 1,000
- 1 tick = 1,000 subticks = 1 USD increment

### Converting Price to Subticks

**Formula** (from documentation):
```
price = abs((quote_amount + fees) / quantity) × 10^(-6 - atomicResolution)
```

Where:
- `quote_amount`: Total quote (USDC) spent/received
- `quantity`: Position size in quantums
- `-6`: USDC atomicResolution
- `atomicResolution`: Base asset atomicResolution

**Reverse (Price → Subticks)**:

Given human-readable price (e.g., 50,000 USD/BTC), convert to subticks:

```rust
fn price_to_subticks(
    price: Decimal,
    atomic_resolution: i32,
    quantum_conversion_exponent: i32,
    subticks_per_tick: u64,
) -> Result<u64, ExchangeError> {
    // Calculate tick value in human-readable terms
    let tick_size = Decimal::from(10_i64.pow(
        (quantum_conversion_exponent + atomic_resolution + 6) as u32
    ));

    // Calculate number of ticks
    let ticks = price / tick_size;

    // Convert to subticks
    let subticks = ticks * Decimal::from(subticks_per_tick);

    // Round to nearest integer
    subticks.round().to_u64()
        .ok_or(ExchangeError::SubtickConversion)
}
```

### Converting Subticks to Price

```rust
fn subticks_to_price(
    subticks: u64,
    atomic_resolution: i32,
    quantum_conversion_exponent: i32,
    subticks_per_tick: u64,
) -> Decimal {
    // Convert subticks to ticks
    let ticks = Decimal::from(subticks) / Decimal::from(subticks_per_tick);

    // Calculate tick value
    let tick_size = Decimal::from(10_i64.pow(
        (quantum_conversion_exponent + atomic_resolution + 6) as u32
    ));

    ticks * tick_size
}
```

## Practical Example: BTC-USD Order

### Market Parameters (from API)
```json
{
  "ticker": "BTC-USD",
  "clobPairId": "0",
  "atomicResolution": -10,
  "quantumConversionExponent": -9,
  "subticksPerTick": 100000,
  "stepBaseQuantums": 1000000,
  "stepSize": "0.0001",
  "tickSize": "1"
}
```

### Place Buy Order: 1.5 BTC @ 50,000 USD

**Step 1: Convert size to quantums**
```
size = 1.5 BTC
atomicResolution = -10

quantums = 1.5 × 10^(10) = 15,000,000,000
```

**Step 2: Convert price to subticks**

Using the helper from v4 client libraries (simplified):
```
price = 50,000 USD/BTC
tickSize = 1 USD (from market params)
subticksPerTick = 100,000

ticks = price / tickSize = 50,000 / 1 = 50,000
subticks = ticks × subticksPerTick = 50,000 × 100,000 = 5,000,000,000
```

**Step 3: Create order message**
```protobuf
MsgPlaceOrder {
    clobPairId: 0,
    side: ORDER_SIDE_BUY,
    quantums: 15000000000,
    subticks: 5000000000,
    // ... other fields
}
```

## stepBaseQuantums vs stepSize

### stepSize (Human-Readable)
- Minimum order size increment in human-readable terms
- Example: "0.0001" BTC

### stepBaseQuantums (Protocol)
- Minimum order size increment in quantums
- Example: 1,000,000 quantums

**Relationship**:
```
stepSize = stepBaseQuantums × 10^(atomicResolution)
```

**Example (BTC-USD)**:
```
stepBaseQuantums = 1,000,000
atomicResolution = -10

stepSize = 1,000,000 × 10^(-10) = 0.0001 BTC
```

**Validation**:
```rust
fn validate_order_size(quantums: u64, step_base_quantums: u64) -> Result<(), ExchangeError> {
    if quantums % step_base_quantums != 0 {
        return Err(ExchangeError::InvalidOrderSize);
    }
    Ok(())
}
```

## subticksPerTick

### Purpose
Allows protocol to adjust price granularity without changing `atomicResolution`.

### Example Scenario
**BTC at $20,000/BTC**:
- Want 1 tick = $100 increment
- Set `subticksPerTick` = 10,000

**BTC drops to $200/BTC** (hypothetical):
- Want 1 tick = $1 increment
- Adjust `subticksPerTick` = 100 via governance
- No need to change `atomicResolution`

### Governance Adjustment
`subticksPerTick` can be updated through governance proposals to maintain appropriate price granularity as market prices change.

## Complete Conversion Example

### Given Order Parameters
```
Order: BUY 0.05 BTC @ 50,000 USD
Market: BTC-USD
```

### Market Info (from API)
```rust
struct MarketInfo {
    atomic_resolution: i32,         // -10
    quantum_conversion_exponent: i32, // -9
    subticks_per_tick: u64,         // 100,000
    step_base_quantums: u64,        // 1,000,000
    tick_size: String,              // "1"
}
```

### Conversion Code
```rust
use rust_decimal::Decimal;
use std::str::FromStr;

fn create_order(
    size: &str,
    price: &str,
    market: &MarketInfo,
) -> Result<(u64, u64), ExchangeError> {
    // Parse inputs
    let size_decimal = Decimal::from_str(size)?;
    let price_decimal = Decimal::from_str(price)?;

    // Convert size to quantums
    let size_multiplier = Decimal::from(
        10_i64.pow((-market.atomic_resolution) as u32)
    );
    let quantums = (size_decimal * size_multiplier)
        .round()
        .to_u64()
        .ok_or(ExchangeError::QuantumConversion)?;

    // Validate against stepBaseQuantums
    if quantums % market.step_base_quantums != 0 {
        return Err(ExchangeError::InvalidOrderSize);
    }

    // Convert price to subticks
    let tick_size = Decimal::from_str(&market.tick_size)?;
    let ticks = (price_decimal / tick_size)
        .round()
        .to_u64()
        .ok_or(ExchangeError::PriceConversion)?;

    let subticks = ticks * market.subticks_per_tick;

    Ok((quantums, subticks))
}

// Usage
let market = MarketInfo {
    atomic_resolution: -10,
    quantum_conversion_exponent: -9,
    subticks_per_tick: 100_000,
    step_base_quantums: 1_000_000,
    tick_size: "1".to_string(),
};

let (quantums, subticks) = create_order("0.05", "50000", &market)?;

println!("Quantums: {}", quantums);  // 500,000,000
println!("Subticks: {}", subticks);  // 5,000,000,000
```

## Reading Orders from Indexer

When querying orders from the Indexer API, you receive human-readable values:

```json
{
  "id": "order-uuid-123",
  "size": "1.5",
  "price": "50000.0",
  "clobPairId": "0"
}
```

But when placing orders via gRPC, you must use quantums and subticks:

```protobuf
MsgPlaceOrder {
    quantums: 15000000000,
    subticks: 5000000000
}
```

## Block Data Interpretation

When reading trades from block data (not Indexer):

```json
{
  "quantums": -500000000,
  "subticks": 295900000000
}
```

**Interpreting Trade Size**:
```
quantums = -500,000,000
atomicResolution = -7 (for AVAX)

size = -500,000,000 × 10^(-7) = -50 AVAX
(negative = SELL)
```

**Interpreting Price**:
```
Use formula: abs((quote + fees) / quantity) × 10^(-6 - atomicResolution)

Result: 29.59 USD/AVAX
```

## Market Builder Helper (Official Clients)

Official dYdX clients provide a **Market Builder** to simplify these conversions:

```typescript
import { Market } from '@dydxprotocol/v4-client-js';

// Create market helper
const market = new Market(marketInfo);

// Convert size
const quantums = market.sizeToQuantums(1.5); // BTC

// Convert price
const subticks = market.priceToSubticks(50000); // USD

// Place order
await client.placeOrder(
  subaccount,
  market.clobPairId,
  side,
  quantums,
  subticks,
  // ...
);
```

**For Rust**: Implement similar helper struct.

## Common Errors

### Error: Invalid Order Size
**Cause**: Order size (quantums) not a multiple of `stepBaseQuantums`

**Fix**:
```rust
// Round to nearest valid size
let rounded_quantums = (quantums / step_base_quantums) * step_base_quantums;
```

### Error: Invalid Price
**Cause**: Price (subticks) not aligned with tick size

**Fix**:
```rust
// Round to nearest valid price
let ticks = subticks / subticks_per_tick;
let rounded_subticks = ticks * subticks_per_tick;
```

### Error: Overflow
**Cause**: Intermediate calculations exceed u64 limits

**Fix**: Use `Decimal` or `BigInt` types for calculations

## Summary

### Key Formulas

**Size Conversions**:
```
human_size = quantums × 10^(atomicResolution)
quantums = human_size × 10^(-atomicResolution)
```

**Price Conversions** (simplified):
```
human_price = (subticks / subticksPerTick) × tickSize
subticks = (human_price / tickSize) × subticksPerTick
```

**Validation**:
```
quantums % stepBaseQuantums == 0
subticks % subticksPerTick should align with tickSize
```

### Implementation Checklist

- [ ] Fetch market parameters (atomicResolution, quantumConversionExponent, etc.)
- [ ] Implement size → quantums conversion
- [ ] Implement price → subticks conversion
- [ ] Implement reverse conversions (for display)
- [ ] Validate against stepBaseQuantums and tickSize
- [ ] Handle rounding correctly
- [ ] Use Decimal types to avoid precision loss
- [ ] Test with various market parameters (BTC, ETH, small-cap assets)

### Rust Helper Struct

```rust
use rust_decimal::Decimal;

pub struct MarketConverter {
    atomic_resolution: i32,
    quantum_conversion_exponent: i32,
    subticks_per_tick: u64,
    step_base_quantums: u64,
    tick_size: Decimal,
}

impl MarketConverter {
    pub fn new(market_info: &MarketInfo) -> Self {
        Self {
            atomic_resolution: market_info.atomic_resolution,
            quantum_conversion_exponent: market_info.quantum_conversion_exponent,
            subticks_per_tick: market_info.subticks_per_tick,
            step_base_quantums: market_info.step_base_quantums,
            tick_size: Decimal::from_str(&market_info.tick_size).unwrap(),
        }
    }

    pub fn size_to_quantums(&self, size: Decimal) -> Result<u64, ExchangeError> {
        let multiplier = Decimal::from(10_i64.pow((-self.atomic_resolution) as u32));
        let quantums = (size * multiplier).round();

        let quantums_u64 = quantums.to_u64()
            .ok_or(ExchangeError::QuantumConversion)?;

        // Validate against stepBaseQuantums
        if quantums_u64 % self.step_base_quantums != 0 {
            // Round to nearest valid size
            let rounded = (quantums_u64 / self.step_base_quantums) * self.step_base_quantums;
            return Ok(rounded);
        }

        Ok(quantums_u64)
    }

    pub fn quantums_to_size(&self, quantums: u64) -> Decimal {
        let divisor = Decimal::from(10_i64.pow((-self.atomic_resolution) as u32));
        Decimal::from(quantums) / divisor
    }

    pub fn price_to_subticks(&self, price: Decimal) -> Result<u64, ExchangeError> {
        let ticks = (price / self.tick_size).round();
        let ticks_u64 = ticks.to_u64()
            .ok_or(ExchangeError::PriceConversion)?;

        Ok(ticks_u64 * self.subticks_per_tick)
    }

    pub fn subticks_to_price(&self, subticks: u64) -> Decimal {
        let ticks = Decimal::from(subticks) / Decimal::from(self.subticks_per_tick);
        ticks * self.tick_size
    }
}
```

## Resources

- **Official Docs**: https://docs.dydx.xyz/concepts/trading/quantums
- **TypeScript Client**: https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-js
- **Python Client**: https://github.com/dydxprotocol/v4-clients/tree/main/v4-client-py
- **Block Data Guide**: https://docs.dydx.exchange/guides/how_to_interpret_block_data_for_trades
