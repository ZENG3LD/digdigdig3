# Deribit Symbols and Instrument Naming

Complete specification of Deribit instrument naming conventions and symbol formats.

## Overview

Deribit is primarily a **derivatives exchange** specializing in:
- Bitcoin (BTC) and Ethereum (ETH) options and futures
- USDC-settled instruments (SOL, XRP, BNB)
- Perpetual contracts
- Combo instruments (multi-leg strategies)

## Instrument Name Format

Deribit uses structured instrument names with different formats based on instrument type.

### General Format Components

- **Base Currency**: `BTC`, `ETH`, `SOL`, `XRP`, `BNB`, `USDC`, `USDT`
- **Settlement Currency**: Typically same as base (BTC, ETH) or USDC for linear instruments
- **Date Format**: `DDMMMYY` (e.g., `27DEC24`, `31JAN25`)
- **Strike Price**: Integer (e.g., `50000`, `3000`)
- **Option Type**: `C` (Call) or `P` (Put)

---

## Perpetual Futures

### Format
```
{BASE_CURRENCY}-PERPETUAL
```

### Examples
- `BTC-PERPETUAL` - Bitcoin perpetual future (inverse, BTC-settled)
- `ETH-PERPETUAL` - Ethereum perpetual future (inverse, ETH-settled)

### Characteristics
- No expiration date
- Funding rate mechanism (8-hour intervals)
- Inverse contracts (settled in BTC/ETH)
- Quote currency: USD
- Settlement currency: BTC or ETH

---

## Linear Perpetuals

### Format
```
{BASE_CURRENCY}_USDC-PERPETUAL
```

### Examples
- `BTC_USDC-PERPETUAL` - Bitcoin linear perpetual (USDC-settled)
- `ETH_USDC-PERPETUAL` - Ethereum linear perpetual (USDC-settled)
- `SOL_USDC-PERPETUAL` - Solana linear perpetual (USDC-settled)
- `XRP_USDC-PERPETUAL` - Ripple linear perpetual (USDC-settled)
- `BNB_USDC-PERPETUAL` - Binance Coin linear perpetual (USDC-settled)

### Characteristics
- Settlement in USDC stablecoin
- Linear P&L (like spot margin trading)
- Funding rate mechanism
- Available for: SOL, XRP, BNB (and optionally BTC, ETH)

---

## Dated Futures

### Format
```
{BASE_CURRENCY}-{DDMMMYY}
```

### Examples
- `BTC-29MAR24` - Bitcoin futures expiring 29 March 2024
- `BTC-27DEC24` - Bitcoin futures expiring 27 December 2024
- `ETH-31JAN25` - Ethereum futures expiring 31 January 2025
- `ETH-28JUN24` - Ethereum futures expiring 28 June 2024

### Expiration Schedule
- **Weekly**: Every Friday
- **Monthly**: Last Friday of each month (or previous Friday if last Friday is a holiday)
- **Quarterly**: Last Friday of March, June, September, December

### Characteristics
- Cash-settled at expiry (delivery price = 30-min TWAP of index)
- Expiry time: 08:00 UTC
- Inverse contracts (quote in USD, settle in BTC/ETH)

---

## Options

### Format
```
{BASE_CURRENCY}-{DDMMMYY}-{STRIKE}-{C|P}
```

### Examples
- `BTC-27DEC24-50000-C` - BTC call option, Dec 27 2024, strike $50,000
- `BTC-27DEC24-50000-P` - BTC put option, Dec 27 2024, strike $50,000
- `ETH-29MAR24-3000-C` - ETH call option, Mar 29 2024, strike $3,000
- `ETH-29MAR24-2500-P` - ETH put option, Mar 29 2024, strike $2,500

### Components
- **Base**: `BTC` or `ETH`
- **Expiry Date**: `DDMMMYY` format
- **Strike Price**: Integer (in USD)
- **Type**: `C` (Call) or `P` (Put)

### Characteristics
- European-style (exercise only at expiry)
- Cash-settled at expiry
- Expiry time: 08:00 UTC
- Settled in BTC or ETH
- Greeks available (delta, gamma, theta, vega, rho)

### Strike Price Grid
- Strike prices follow exchange-defined intervals
- Denser strikes near at-the-money
- Wider strikes far from current price

---

## Linear Options (USDC-settled)

### Format
```
{BASE_CURRENCY}_USDC-{DDMMMYY}-{STRIKE}-{C|P}
```

### Examples
- `SOL_USDC-29MAR24-100-C` - SOL call option, USDC-settled
- `XRP_USDC-31JAN25-1.5-P` - XRP put option, USDC-settled
- `BNB_USDC-28JUN24-400-C` - BNB call option, USDC-settled

### Characteristics
- Settlement in USDC
- European-style
- Available for: SOL, XRP, BNB

---

## Combo Instruments

Combos allow trading multi-leg option or future strategies as a single instrument.

### Future Combos

**Format**:
```
{BASE_CURRENCY}-{COMBO_NAME}
```

**Examples**:
- `BTC-FS-29MAR24_27SEP24` - Future spread (buy Mar, sell Sep)

**Characteristics**:
- `kind`: `"future_combo"`
- Retrieved via `public/get_instruments` with `kind=future_combo`

### Option Combos

**Format**:
```
{BASE_CURRENCY}-{COMBO_NAME}
```

**Examples**:
- `BTC-CALENDAR-27DEC24-50000-C` - Calendar spread
- `BTC-STRADDLE-29MAR24-50000` - Straddle (buy call + put at same strike)
- `BTC-STRANGLE-29MAR24-48000-52000` - Strangle

**Characteristics**:
- `kind`: `"option_combo"`
- Retrieved via `public/get_instruments` with `kind=option_combo`
- Single order executes multiple legs
- Atomic execution (all-or-nothing)

---

## Instrument Kind Enumeration

Use the `kind` parameter in `public/get_instruments`:

| Kind | Description | Example |
|------|-------------|---------|
| `future` | Dated and perpetual futures | `BTC-PERPETUAL`, `BTC-29MAR24` |
| `option` | Vanilla options | `BTC-27DEC24-50000-C` |
| `spot` | Spot trading pairs | N/A (Deribit is derivatives-focused) |
| `future_combo` | Multi-leg future strategies | `BTC-FS-29MAR24_27SEP24` |
| `option_combo` | Multi-leg option strategies | `BTC-STRADDLE-29MAR24-50000` |

---

## Currency Support

### Base Currencies
- **BTC** (Bitcoin)
- **ETH** (Ethereum)
- **SOL** (Solana) - USDC-settled only
- **XRP** (Ripple) - USDC-settled only
- **BNB** (Binance Coin) - USDC-settled only
- **USDC** (USD Coin) - Stablecoin
- **USDT** (Tether) - Stablecoin
- **EURR** (Euro stablecoin)

### Settlement Currencies
- **BTC**: For BTC inverse instruments
- **ETH**: For ETH inverse instruments
- **USDC**: For linear instruments (SOL, XRP, BNB, BTC_USDC, ETH_USDC)

---

## Symbol Parsing

### Perpetual Futures
```rust
// Pattern: {BASE}-PERPETUAL
if instrument_name.ends_with("-PERPETUAL") {
    let base = instrument_name.strip_suffix("-PERPETUAL").unwrap();
    // base = "BTC", "ETH", etc.
}
```

### Linear Perpetuals
```rust
// Pattern: {BASE}_USDC-PERPETUAL
if instrument_name.ends_with("_USDC-PERPETUAL") {
    let base = instrument_name.strip_suffix("_USDC-PERPETUAL").unwrap();
    // base = "BTC", "ETH", "SOL", "XRP", "BNB"
}
```

### Dated Futures
```rust
// Pattern: {BASE}-{DDMMMYY}
// Example: BTC-29MAR24
let parts: Vec<&str> = instrument_name.split('-').collect();
if parts.len() == 2 && parts[1].len() == 7 {
    let base = parts[0]; // "BTC"
    let expiry_date = parts[1]; // "29MAR24"
    // Parse date: DD (2 chars), MMM (3 chars), YY (2 chars)
}
```

### Options
```rust
// Pattern: {BASE}-{DDMMMYY}-{STRIKE}-{C|P}
// Example: BTC-27DEC24-50000-C
let parts: Vec<&str> = instrument_name.split('-').collect();
if parts.len() == 4 && (parts[3] == "C" || parts[3] == "P") {
    let base = parts[0]; // "BTC"
    let expiry_date = parts[1]; // "27DEC24"
    let strike = parts[2].parse::<f64>().unwrap(); // 50000.0
    let option_type = parts[3]; // "C" or "P"
}
```

### Linear Options
```rust
// Pattern: {BASE}_USDC-{DDMMMYY}-{STRIKE}-{C|P}
// Example: SOL_USDC-29MAR24-100-C
if instrument_name.contains("_USDC-") {
    let parts: Vec<&str> = instrument_name.split('-').collect();
    let base_with_usdc = parts[0]; // "SOL_USDC"
    let base = base_with_usdc.strip_suffix("_USDC").unwrap(); // "SOL"
    let expiry_date = parts[1];
    let strike = parts[2].parse::<f64>().unwrap();
    let option_type = parts[3];
}
```

---

## Instrument State

Instruments have a `state` field indicating their lifecycle:

| State | Description |
|-------|-------------|
| `open` | Active trading |
| `closed` | Trading halted (temporary or permanent) |
| `pre_open` | Pre-opening auction phase |
| `settled` | Expired and settled |

---

## Instrument Metadata

Use `public/get_instruments` to retrieve full metadata:

```json
{
  "instrument_name": "BTC-PERPETUAL",
  "instrument_id": 139,
  "kind": "future",
  "settlement_period": "perpetual",
  "settlement_currency": "BTC",
  "base_currency": "BTC",
  "quote_currency": "USD",
  "contract_size": 10,
  "tick_size": 0.5,
  "min_trade_amount": 10,
  "is_active": true,
  "state": "open",
  "expiration_timestamp": 32503708800000
}
```

**Key Fields**:
- `instrument_name`: Full symbol name
- `instrument_id`: Unique numeric ID
- `kind`: Instrument type
- `settlement_currency`: Currency for P&L settlement
- `base_currency`: Underlying asset
- `quote_currency`: Price denomination
- `contract_size`: Multiplier (e.g., $10 for BTC-PERPETUAL)
- `tick_size`: Minimum price increment
- `min_trade_amount`: Minimum order size
- `is_active`: Trading enabled
- `expiration_timestamp`: Expiry time (milliseconds)

---

## Symbol Normalization

### Standardization Rules
1. **Case Sensitivity**: Instrument names are case-sensitive (use uppercase)
2. **Whitespace**: No whitespace allowed in instrument names
3. **Separators**: Use hyphens `-` or underscores `_` as shown in patterns
4. **Month Abbreviations**: Three-letter uppercase (JAN, FEB, MAR, APR, MAY, JUN, JUL, AUG, SEP, OCT, NOV, DEC)

### Validation
```rust
fn validate_instrument_name(name: &str) -> bool {
    // Must not be empty
    if name.is_empty() {
        return false;
    }

    // Must contain only alphanumeric, hyphens, underscores
    name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
```

---

## Filtering Instruments

### By Currency
```json
{
  "method": "public/get_instruments",
  "params": {
    "currency": "BTC"
  }
}
```
Returns all BTC instruments (futures, options, combos).

### By Kind
```json
{
  "method": "public/get_instruments",
  "params": {
    "currency": "BTC",
    "kind": "future"
  }
}
```
Returns only BTC futures (including perpetuals).

### Active vs Expired
```json
{
  "method": "public/get_instruments",
  "params": {
    "currency": "BTC",
    "expired": false
  }
}
```
`expired=false` (default): Active instruments only
`expired=true`: Recently expired instruments

---

## Mapping to Unified Symbols

For cross-exchange compatibility, you may want to map Deribit symbols to a unified format:

| Deribit | Unified Format | Notes |
|---------|----------------|-------|
| `BTC-PERPETUAL` | `BTCUSD_PERP` | Inverse perpetual |
| `BTC_USDC-PERPETUAL` | `BTCUSDC_PERP` | Linear perpetual |
| `BTC-29MAR24` | `BTCUSD_240329` | Dated future (YYMMDD) |
| `BTC-27DEC24-50000-C` | `BTCUSD_241227_50000_C` | Option |

**Parsing Logic**:
- Perpetuals: Detect `-PERPETUAL` suffix
- Linear: Detect `_USDC-` substring
- Futures: Detect date pattern (7 chars after first `-`)
- Options: Detect 4-part format ending in `C` or `P`

---

## Special Cases

### 1. MOVE Contracts
Deribit offers "MOVE" contracts (straddle products):
- Example: `BTC-MOVE-29MAR24`
- Settled based on absolute price movement

### 2. Block Trading
Block trades use the same instrument names but have special minimum sizes:
- `block_trade_min_trade_amount`: Minimum size for block trades
- Typically 25,000 USD or higher

### 3. Combo Notation
Combos may have custom naming:
- `BTC-FS-{DATE1}_{DATE2}`: Future spread
- `BTC-CALENDAR-{DATE}-{STRIKE}-{C|P}`: Calendar spread
- `BTC-STRADDLE-{DATE}-{STRIKE}`: Straddle

Retrieve all combos via:
```json
{
  "method": "public/get_instruments",
  "params": {
    "currency": "BTC",
    "kind": "option_combo"
  }
}
```

---

## Implementation Checklist

For V5 connector:

- [ ] Implement symbol parser for all formats (perpetuals, futures, options, linear)
- [ ] Handle both inverse (BTC/ETH-settled) and linear (USDC-settled) instruments
- [ ] Parse expiry dates from instrument names
- [ ] Extract strike prices and option types
- [ ] Validate instrument names before API calls
- [ ] Cache instrument metadata (avoid repeated `get_instruments` calls)
- [ ] Support filtering by currency, kind, active/expired
- [ ] Map Deribit symbols to unified format (if needed)
- [ ] Handle edge cases (MOVE contracts, combos)
- [ ] Implement `is_active` check before trading

---

## Examples for Testing

### Perpetuals
- `BTC-PERPETUAL`
- `ETH-PERPETUAL`
- `SOL_USDC-PERPETUAL`
- `XRP_USDC-PERPETUAL`

### Futures
- `BTC-29MAR24`
- `ETH-27DEC24`

### Options
- `BTC-27DEC24-50000-C`
- `BTC-27DEC24-50000-P`
- `ETH-29MAR24-3000-C`
- `SOL_USDC-29MAR24-100-C`

### Combos
- `BTC-FS-29MAR24_27SEP24`
- `BTC-STRADDLE-29MAR24-50000`

---

## References

- Deribit Instrument Specifications: https://www.deribit.com/
- Contract Introduction Policy: https://support.deribit.com/hc/en-us/articles/25944688876957-Contract-Introduction-Policy
- Linear Perpetuals: https://support.deribit.com/hc/en-us/articles/31424969384605-Linear-Perpetual
- Trading Combos: https://insights.deribit.com/education/trading-combos-on-deribit/
