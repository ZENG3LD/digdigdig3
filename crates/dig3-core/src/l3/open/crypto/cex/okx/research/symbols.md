# OKX API v5 Symbols and Instrument Types

## Instrument Types (instType)

OKX API v5 supports multiple instrument types, specified using the `instType` parameter:

| instType | Description | Example |
|----------|-------------|---------|
| `SPOT` | Spot trading | `BTC-USDT` |
| `MARGIN` | Margin trading | `BTC-USDT` |
| `SWAP` | Perpetual swaps | `BTC-USDT-SWAP` |
| `FUTURES` | Futures contracts | `BTC-USD-240329` |
| `OPTION` | Options contracts | `BTC-USD-240329-50000-C` |
| `ANY` | All instrument types (for subscriptions) | - |

---

## Symbol Format by Instrument Type

### SPOT

**Format:** `BASE-QUOTE`

**Examples:**
- `BTC-USDT` - Bitcoin vs Tether (USDT)
- `ETH-USDT` - Ethereum vs Tether
- `BTC-USD` - Bitcoin vs USD
- `ETH-BTC` - Ethereum vs Bitcoin

**Characteristics:**
- Simple currency pair
- No expiration
- No leverage (unless using MARGIN mode)
- Settlement in quote currency

---

### MARGIN

**Format:** `BASE-QUOTE` (same as SPOT)

**Examples:**
- `BTC-USDT` - Bitcoin vs Tether (margin)
- `ETH-USDT` - Ethereum vs Tether (margin)

**Characteristics:**
- Same symbol format as SPOT
- Leverage is set **per currency**, not per instrument
- Distinguishable from SPOT by `tdMode` parameter (`cross` or `isolated`)
- Settlement in quote currency

**Note:** The same symbol `BTC-USDT` can be used for both SPOT and MARGIN trading. The mode is determined by the trade mode (`tdMode`) parameter when placing orders.

---

### SWAP (Perpetual Swaps)

**Format:** `BASE-QUOTE-SWAP`

**Examples:**
- `BTC-USDT-SWAP` - Bitcoin perpetual (USDT-margined)
- `ETH-USDT-SWAP` - Ethereum perpetual (USDT-margined)
- `BTC-USD-SWAP` - Bitcoin perpetual (coin-margined)
- `ETH-USD-SWAP` - Ethereum perpetual (coin-margined)

**Characteristics:**
- Perpetual contracts (no expiration)
- Include `-SWAP` suffix
- Support leverage
- Pay/receive funding fees every 8 hours
- Can be linear (USDT-margined) or inverse (coin-margined)

**Margining:**
- `BTC-USDT-SWAP`: Linear contract, margined in USDT
- `BTC-USD-SWAP`: Inverse contract, margined in BTC

---

### FUTURES

**Format:** `BASE-QUOTE-YYMMDD`

**Examples:**
- `BTC-USD-240329` - Bitcoin futures expiring March 29, 2024
- `BTC-USD-240628` - Bitcoin futures expiring June 28, 2024
- `ETH-USD-240329` - Ethereum futures expiring March 29, 2024
- `BTC-USDT-240329` - Bitcoin USDT-margined futures

**Characteristics:**
- Fixed expiration date
- Date format: `YYMMDD` (2-digit year, 2-digit month, 2-digit day)
- Quarterly contracts typically available (March, June, September, December)
- Settle on expiration date
- Can be linear (USDT-margined) or inverse (coin-margined)

**Common Expiration Dates:**
- Last Friday of contract month (quarterly: Mar, Jun, Sep, Dec)
- Settlement at 08:00:00 UTC on expiry date

---

### OPTION

**Format:** `BASE-QUOTE-YYMMDD-STRIKE-TYPE`

**Examples:**
- `BTC-USD-240329-50000-C` - Bitcoin call option, $50,000 strike, expires March 29, 2024
- `BTC-USD-240329-50000-P` - Bitcoin put option, $50,000 strike, expires March 29, 2024
- `ETH-USD-240628-3000-C` - Ethereum call option, $3,000 strike, expires June 28, 2024

**Components:**
- `BASE-QUOTE`: Underlying asset (e.g., `BTC-USD`)
- `YYMMDD`: Expiration date
- `STRIKE`: Strike price (e.g., `50000`)
- `TYPE`: `C` (call) or `P` (put)

**Characteristics:**
- European-style options (exercise at expiration only)
- Settled in the quote currency
- Multiple strikes available per expiration
- Weekly and monthly expirations

---

## Underlying and Instrument Family

### Underlying (uly)

The underlying represents the base asset pair without the derivative suffix.

**Examples:**
- `BTC-USD` - Underlying for `BTC-USD-SWAP`, `BTC-USD-240329`, etc.
- `ETH-USDT` - Underlying for `ETH-USDT-SWAP`, `ETH-USDT-240329`, etc.

**Usage:**
- Used to query all derivatives for a specific base pair
- Example: Get all BTC-USD futures and swaps by specifying `uly=BTC-USD`

### Instrument Family (instFamily)

Instrument family groups related instruments by underlying and contract type.

**Examples:**
- `BTC-USD` - All BTC-USD instruments (SWAP, FUTURES, OPTION)
- `BTC-USDT` - All BTC-USDT instruments

**Usage:**
- Similar to `uly` but may include additional grouping logic
- Used in subscriptions and bulk queries

---

## Instrument ID Components

### Parsing Symbol Components

```rust
// Example parsing logic
fn parse_instrument_id(inst_id: &str) -> (String, InstrumentType) {
    let parts: Vec<&str> = inst_id.split('-').collect();

    match parts.len() {
        2 => {
            // SPOT or MARGIN: BTC-USDT
            (inst_id.to_string(), InstrumentType::Spot)
        }
        3 => {
            if parts[2] == "SWAP" {
                // SWAP: BTC-USDT-SWAP
                (format!("{}-{}", parts[0], parts[1]), InstrumentType::Swap)
            } else {
                // FUTURES: BTC-USD-240329
                (format!("{}-{}", parts[0], parts[1]), InstrumentType::Futures)
            }
        }
        5 => {
            // OPTION: BTC-USD-240329-50000-C
            (format!("{}-{}", parts[0], parts[1]), InstrumentType::Option)
        }
        _ => panic!("Invalid instrument ID format")
    }
}
```

---

## Important Distinctions

### 1. SPOT vs MARGIN

**Same Symbol, Different Mode:**
- `BTC-USDT` can be traded as SPOT or MARGIN
- Differentiated by `tdMode` parameter:
  - `cash` = SPOT (no leverage)
  - `cross` = Cross margin
  - `isolated` = Isolated margin

**Example:**
```json
// SPOT order
{
  "instId": "BTC-USDT",
  "tdMode": "cash",
  "side": "buy"
}

// MARGIN order
{
  "instId": "BTC-USDT",
  "tdMode": "isolated",
  "side": "buy"
}
```

### 2. Linear vs Inverse Contracts

**Linear (USDT-margined):**
- Quote currency: USDT
- Margin: USDT
- P&L: USDT
- Examples: `BTC-USDT-SWAP`, `ETH-USDT-SWAP`

**Inverse (Coin-margined):**
- Quote currency: USD
- Margin: Base currency (BTC, ETH)
- P&L: Base currency
- Examples: `BTC-USD-SWAP`, `ETH-USD-SWAP`

### 3. Leverage Separation

**Important:** Leverage is **separate** for SWAP and FUTURES even with the same underlying.

- `BTC-USD-SWAP` leverage: 10x
- `BTC-USD-240329` leverage: 20x

These are **independent** settings. You must set leverage for each instrument type separately.

---

## Special Instrument Queries

### Get All Instruments of a Type

**Endpoint:** `GET /api/v5/public/instruments`

**Parameters:**
- `instType` (required): `SPOT`, `SWAP`, `FUTURES`, `OPTION`, `MARGIN`
- `uly` (optional): Filter by underlying (e.g., `BTC-USD`)
- `instFamily` (optional): Filter by instrument family
- `instId` (optional): Get specific instrument details

**Examples:**

**Get all SPOT instruments:**
```
GET /api/v5/public/instruments?instType=SPOT
```

**Get all BTC-USD derivatives:**
```
GET /api/v5/public/instruments?instType=SWAP&uly=BTC-USD
```

**Get specific instrument details:**
```
GET /api/v5/public/instruments?instType=SWAP&instId=BTC-USDT-SWAP
```

---

## Instrument Details Response

```json
{
  "code": "0",
  "msg": "",
  "data": [
    {
      "instType": "SWAP",
      "instId": "BTC-USDT-SWAP",
      "uly": "BTC-USDT",
      "instFamily": "BTC-USDT",
      "category": "1",
      "baseCcy": "BTC",
      "quoteCcy": "USDT",
      "settleCcy": "USDT",
      "ctVal": "0.01",
      "ctMult": "1",
      "ctValCcy": "BTC",
      "listTime": "1611916800000",
      "expTime": "",
      "lever": "125",
      "tickSz": "0.1",
      "lotSz": "1",
      "minSz": "1",
      "ctType": "linear",
      "alias": "this_week",
      "state": "live"
    }
  ]
}
```

**Key Fields:**

| Field | Description |
|-------|-------------|
| `instId` | Instrument ID |
| `instType` | Instrument type |
| `uly` | Underlying |
| `instFamily` | Instrument family |
| `baseCcy` | Base currency |
| `quoteCcy` | Quote currency |
| `settleCcy` | Settlement currency |
| `ctVal` | Contract value |
| `ctMult` | Contract multiplier |
| `tickSz` | Tick size (minimum price increment) |
| `lotSz` | Lot size (minimum order size increment) |
| `minSz` | Minimum order size |
| `ctType` | Contract type (`linear` or `inverse`) |
| `lever` | Maximum leverage |
| `expTime` | Expiration time (empty for perpetuals) |

---

## Symbol Formatting Examples

### Converting Symbols

**From Standard Format to OKX:**

| Standard | OKX SPOT | OKX SWAP | OKX FUTURES |
|----------|----------|----------|-------------|
| `BTCUSDT` | `BTC-USDT` | `BTC-USDT-SWAP` | `BTC-USDT-240329` |
| `ETHUSDT` | `ETH-USDT` | `ETH-USDT-SWAP` | `ETH-USDT-240329` |
| `BTCUSD` | `BTC-USD` | `BTC-USD-SWAP` | `BTC-USD-240329` |

**Rust Example:**
```rust
fn to_okx_spot_symbol(base: &str, quote: &str) -> String {
    format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
}

fn to_okx_swap_symbol(base: &str, quote: &str) -> String {
    format!("{}-{}-SWAP", base.to_uppercase(), quote.to_uppercase())
}

fn to_okx_futures_symbol(base: &str, quote: &str, expiry: &str) -> String {
    format!("{}-{}-{}", base.to_uppercase(), quote.to_uppercase(), expiry)
}
```

---

## WebSocket Channel Subscriptions

When subscribing to WebSocket channels, you specify both the channel and the instrument ID.

**Example:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "tickers",
      "instId": "BTC-USDT"
    },
    {
      "channel": "tickers",
      "instId": "BTC-USDT-SWAP"
    }
  ]
}
```

**Subscribe to All Positions:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "positions",
      "instType": "ANY"
    }
  ]
}
```

---

## Best Practices

1. **Always use hyphens:** OKX symbols use `-` as separator (not `/` or no separator)
2. **Case-sensitive:** Use uppercase for currencies (`BTC-USDT`, not `btc-usdt`)
3. **Validate instruments:** Call `/api/v5/public/instruments` to get valid symbols
4. **Check contract specs:** Get `tickSz`, `lotSz`, `minSz` before placing orders
5. **Handle expirations:** FUTURES and OPTION symbols change with expiration dates
6. **Separate leverage:** Remember that SWAP and FUTURES have independent leverage settings
7. **Use `instType` filter:** Reduces response size and improves performance

---

## Summary Table

| Type | Format | Example | Expiration | Leverage | Funding |
|------|--------|---------|------------|----------|---------|
| SPOT | `BASE-QUOTE` | `BTC-USDT` | No | No (unless MARGIN) | No |
| MARGIN | `BASE-QUOTE` | `BTC-USDT` | No | Yes (per currency) | No |
| SWAP | `BASE-QUOTE-SWAP` | `BTC-USDT-SWAP` | No | Yes (per instrument) | Yes (8h) |
| FUTURES | `BASE-QUOTE-YYMMDD` | `BTC-USD-240329` | Yes | Yes (per instrument) | No |
| OPTION | `BASE-QUOTE-YYMMDD-STRIKE-TYPE` | `BTC-USD-240329-50000-C` | Yes | No | No |

---

## Notes

- **Unified API:** All instrument types use the same endpoints (place order, get positions, etc.)
- **Context matters:** `BTC-USDT` can be SPOT or MARGIN depending on `tdMode`
- **Symbol validation:** Always validate symbols before trading to ensure they're active
- **Contract updates:** Futures and options symbols change regularly; query instruments API for current contracts
