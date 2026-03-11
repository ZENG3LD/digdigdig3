# Upbit Symbol/Pair Formatting Research

**Research Date**: 2026-01-20
**Purpose**: Document exact Upbit API symbol formatting, intervals, timestamps, and order parameters

---

## 1. Symbol Format

### Format Pattern

**Structure**: `{QUOTE}-{BASE}` with hyphen separator

**Key Characteristic**: **REVERSED** from most exchanges

- Most exchanges: `{BASE}-{QUOTE}` or `{BASE}{QUOTE}` (e.g., "BTCUSDT")
- **Upbit**: `{QUOTE}-{BASE}` (e.g., "KRW-BTC", "SGD-BTC")

### Examples

| Description | Upbit Format | Traditional Format | Explanation |
|-------------|--------------|-------------------|-------------|
| Bitcoin in Korean Won | **KRW-BTC** | BTCKRW | BTC priced in KRW |
| Ethereum in Singapore Dollar | **SGD-ETH** | ETHSGD | ETH priced in SGD |
| Bitcoin in US Dollar | **USDT-BTC** | BTCUSDT | BTC priced in USDT |
| Ethereum in Bitcoin | **BTC-ETH** | ETHBTC | ETH priced in BTC |
| Ripple in Thai Baht | **THB-XRP** | XRPTHB | XRP priced in THB |

### Case Sensitivity

**Always UPPERCASE**: All official examples and responses use uppercase letters exclusively.

**Examples**:
- ✓ Correct: `"KRW-BTC"`, `"SGD-ETH"`, `"USDT-BTC"`
- ✗ Incorrect: `"krw-btc"`, `"Sgd-Eth"`, `"usdt-btc"`

### Quote Currencies by Region

| Region | Primary Quote Currency | Alternative Quotes |
|--------|------------------------|-------------------|
| **Korea** | KRW (Korean Won) | BTC, USDT |
| **Singapore** | SGD (Singapore Dollar) | BTC, USDT |
| **Indonesia** | IDR (Indonesian Rupiah) | BTC, USDT |
| **Thailand** | THB (Thai Baht) | BTC, USDT |

### Symbol Components

Given symbol `"KRW-BTC"`:
- **Quote Currency**: `KRW` (what you pay)
- **Base Currency**: `BTC` (what you get)
- **Meaning**: Price of 1 BTC in KRW
- **Example**: If price = 85,000,000, then 1 BTC costs 85,000,000 KRW

---

## 2. API Response Format

### Market Field

All market data responses include `market` field with symbol:

```json
{
  "market": "KRW-BTC",
  "trade_price": 85000000.0,
  // ...
}
```

### Trading Pairs List

`GET /v1/trading-pairs` returns array of symbols:

```json
[
  "KRW-BTC",
  "KRW-ETH",
  "KRW-XRP",
  "BTC-ETH",
  "USDT-BTC"
]
```

---

## 3. Candle/Kline Intervals

Upbit uses **separate endpoints** for different timeframes rather than a single unified endpoint with interval parameter.

### 3.1 Minute Intervals

**Endpoint**: `GET /v1/candles/minutes/{unit}`

**Supported Units** (path parameter):
- `1` - 1 minute
- `3` - 3 minutes
- `5` - 5 minutes
- `10` - 10 minutes
- `15` - 15 minutes
- `30` - 30 minutes
- `60` - 1 hour (60 minutes)
- `240` - 4 hours (240 minutes)

**Example**:
```
GET /v1/candles/minutes/1?market=KRW-BTC&count=200
GET /v1/candles/minutes/60?market=SGD-ETH&count=100
```

### 3.2 Day Intervals

**Endpoint**: `GET /v1/candles/days`

**No unit parameter** - always daily candles

**Example**:
```
GET /v1/candles/days?market=KRW-BTC&count=30
```

### 3.3 Week Intervals

**Endpoint**: `GET /v1/candles/weeks`

**No unit parameter** - always weekly candles

**Example**:
```
GET /v1/candles/weeks?market=KRW-BTC&count=52
```

### 3.4 Month Intervals

**Endpoint**: `GET /v1/candles/months`

**No unit parameter** - always monthly candles

**Example**:
```
GET /v1/candles/months?market=KRW-BTC&count=12
```

### 3.5 Year Intervals

**Endpoint**: `GET /v1/candles/years`

**No unit parameter** - always yearly candles

**Example**:
```
GET /v1/candles/years?market=KRW-BTC&count=5
```

### 3.6 Second Intervals

**Endpoint**: `GET /v1/candles/seconds`

**No unit parameter** - always 1-second candles

**Example**:
```
GET /v1/candles/seconds?market=KRW-BTC&count=100
```

### Interval Mapping Table

| Standard | Upbit Endpoint | Path Param |
|----------|---------------|------------|
| 1m | `/v1/candles/minutes/1` | `1` |
| 3m | `/v1/candles/minutes/3` | `3` |
| 5m | `/v1/candles/minutes/5` | `5` |
| 10m | `/v1/candles/minutes/10` | `10` |
| 15m | `/v1/candles/minutes/15` | `15` |
| 30m | `/v1/candles/minutes/30` | `30` |
| 1h | `/v1/candles/minutes/60` | `60` |
| 4h | `/v1/candles/minutes/240` | `240` |
| 1d | `/v1/candles/days` | N/A |
| 1w | `/v1/candles/weeks` | N/A |
| 1M | `/v1/candles/months` | N/A |
| 1y | `/v1/candles/years` | N/A |

**Note**: Upbit does not support 2h, 6h, 8h, 12h intervals natively. Use 60 or 240 minute candles and aggregate if needed.

---

## 4. Time Formats

### 4.1 Timestamp Format

**All timestamps use MILLISECONDS** (not seconds)

**Type**: `int64` / `long`

**Example**: `1718788303000` (not `1718788303`)

### 4.2 Date/Time Strings

**Format**: ISO 8601

**UTC Example**: `"2024-06-19T08:31:43+00:00"`
**KST Example**: `"2024-06-19T17:31:43+09:00"`

### 4.3 Request Parameters

#### `to` Parameter (End Time)

**Format**: ISO 8601 string
**Example**: `"2024-06-19T08:31:43Z"`
**Usage**: Specify end time for candles or trades

```
GET /v1/candles/minutes/1?market=KRW-BTC&to=2024-06-19T08:31:43Z&count=100
```

#### `count` Parameter

**Type**: Integer
**Default**: Varies by endpoint (typically 1-200)
**Maximum**: 200 for candles, 500 for trades

### 4.4 Response Timestamps

#### Market Data (Ticker, Orderbook, Candles, Trades)

**Format**: Number (milliseconds)
**Field Name**: `timestamp`
**Example**: `1718788303000`

#### Account Data (Orders, Deposits, Withdrawals)

**Format**: ISO 8601 string
**Field Name**: `created_at`, `updated_at`, etc.
**Example**: `"2024-06-19T08:31:43+00:00"`

---

## 5. Order Parameters

### 5.1 Side (Transaction Direction)

**Parameter**: `side`

**Supported Values**:
- `"bid"` - Buy order
- `"ask"` - Sell order

**Case**: Lowercase

**Example**:
```json
{
  "market": "KRW-BTC",
  "side": "bid",
  "ord_type": "limit",
  "price": "85000000",
  "volume": "0.1"
}
```

### 5.2 Order Type

**Parameter**: `ord_type`

**Supported Values**:
- `"limit"` - Limit order (requires `price` and `volume`)
- `"price"` - Market buy order (requires `price` as total amount in quote currency)
- `"market"` - Market sell order (requires `volume` in base currency)

**Case**: Lowercase

**Details**:

| Type | Direction | Required Fields | Description |
|------|-----------|----------------|-------------|
| `limit` | Buy/Sell | `price`, `volume` | Order at specific price |
| `price` | Buy only | `price` | Market buy with total amount |
| `market` | Sell only | `volume` | Market sell with volume |

**Example (Limit Order)**:
```json
{
  "market": "KRW-BTC",
  "side": "bid",
  "ord_type": "limit",
  "price": "85000000",
  "volume": "0.1"
}
```

**Example (Market Buy)**:
```json
{
  "market": "KRW-BTC",
  "side": "bid",
  "ord_type": "price",
  "price": "8500000"
}
```

**Example (Market Sell)**:
```json
{
  "market": "KRW-BTC",
  "side": "ask",
  "ord_type": "market",
  "volume": "0.1"
}
```

### 5.3 Time in Force

**Parameter**: `time_in_force` (optional)

**Supported Values**:
- `"IOC"` - Immediate or Cancel (execute immediately, cancel remainder)
- `"FOK"` - Fill or Kill (execute fully immediately or cancel entirely)

**Case**: UPPERCASE

**Default**: If not specified, limit orders remain active until filled or canceled (GTC behavior)

**Example**:
```json
{
  "market": "KRW-BTC",
  "side": "bid",
  "ord_type": "limit",
  "price": "85000000",
  "volume": "0.1",
  "time_in_force": "IOC"
}
```

### 5.4 Identifier (Idempotency)

**Parameter**: `identifier` (optional)

**Type**: String (max 40 characters)

**Purpose**: Client-side order identifier for idempotency

**Usage**: If same `identifier` submitted multiple times, only first order placed

**Example**:
```json
{
  "market": "KRW-BTC",
  "side": "bid",
  "ord_type": "limit",
  "price": "85000000",
  "volume": "0.1",
  "identifier": "my-order-12345"
}
```

---

## 6. Symbol Mapping Rules

### 6.1 Reversed Format

**Upbit → Standard Conversion**:
```
Upbit: "KRW-BTC"
Standard: "BTCKRW"

Quote: KRW (first)
Base: BTC (second)
```

**Standard → Upbit Conversion**:
```
Standard: "BTCUSDT"
Upbit: "USDT-BTC"

Base: BTC → second position
Quote: USDT → first position
```

### 6.2 Implementation Helper

```python
def to_upbit_symbol(base: str, quote: str) -> str:
    """Convert base/quote to Upbit symbol format."""
    return f"{quote.upper()}-{base.upper()}"

def from_upbit_symbol(upbit_symbol: str) -> tuple[str, str]:
    """Parse Upbit symbol to base/quote."""
    parts = upbit_symbol.split('-')
    if len(parts) != 2:
        raise ValueError(f"Invalid Upbit symbol: {upbit_symbol}")
    quote, base = parts
    return base, quote

# Examples
assert to_upbit_symbol("BTC", "KRW") == "KRW-BTC"
assert to_upbit_symbol("ETH", "USDT") == "USDT-ETH"

base, quote = from_upbit_symbol("KRW-BTC")
assert base == "BTC"
assert quote == "KRW"
```

### 6.3 Rust Implementation Helper

```rust
pub fn format_symbol(base: &str, quote: &str) -> String {
    // Upbit uses QUOTE-BASE format (reversed)
    format!("{}-{}", quote.to_uppercase(), base.to_uppercase())
}

pub fn parse_symbol(symbol: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = symbol.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid Upbit symbol: {}", symbol));
    }
    // Upbit format: QUOTE-BASE
    let quote = parts[0].to_string();
    let base = parts[1].to_string();
    Ok((base, quote))
}

#[test]
fn test_format_symbol() {
    assert_eq!(format_symbol("BTC", "KRW"), "KRW-BTC");
    assert_eq!(format_symbol("ETH", "USDT"), "USDT-ETH");
}

#[test]
fn test_parse_symbol() {
    let (base, quote) = parse_symbol("KRW-BTC").unwrap();
    assert_eq!(base, "BTC");
    assert_eq!(quote, "KRW");
}
```

---

## 7. Summary of Critical Findings

### Symbol Formatting

| Aspect | Upbit Format | Notes |
|--------|-------------|-------|
| **Pattern** | `{QUOTE}-{BASE}` | **REVERSED from standard** |
| **Separator** | Hyphen (`-`) | Not underscore or no separator |
| **Case** | UPPERCASE | Always uppercase |
| **Example** | `KRW-BTC`, `SGD-ETH` | Quote currency first |

### Intervals

| Timeframe | Endpoint | Parameter |
|-----------|----------|-----------|
| 1m - 4h | `/v1/candles/minutes/{unit}` | Path: `1`, `3`, `5`, `10`, `15`, `30`, `60`, `240` |
| 1d | `/v1/candles/days` | None |
| 1w | `/v1/candles/weeks` | None |
| 1M | `/v1/candles/months` | None |
| 1y | `/v1/candles/years` | None |
| 1s | `/v1/candles/seconds` | None |

### Timestamps

| Context | Format | Example |
|---------|--------|---------|
| **Market Data** | Milliseconds (number) | `1718788303000` |
| **Account Data** | ISO 8601 (string) | `"2024-06-19T08:31:43+00:00"` |
| **Request Param** | ISO 8601 (string) | `"2024-06-19T08:31:43Z"` |

### Order Parameters

| Parameter | Values | Case | Required |
|-----------|--------|------|----------|
| `side` | `"bid"`, `"ask"` | Lowercase | Yes |
| `ord_type` | `"limit"`, `"price"`, `"market"` | Lowercase | Yes |
| `time_in_force` | `"IOC"`, `"FOK"` | Uppercase | Optional |

---

## 8. Key Implementation Notes

1. **Symbol Reversal**: Always reverse base/quote when converting to/from Upbit format
2. **Multiple Endpoints**: Use different endpoints for different timeframes (not interval param)
3. **Milliseconds**: All timestamps in milliseconds, not seconds
4. **Lowercase Orders**: Order `side` and `ord_type` use lowercase
5. **Uppercase TIF**: `time_in_force` uses uppercase
6. **Path Parameters**: Minute intervals passed as path parameter, not query param
7. **Hyphen Separator**: Symbol separator is hyphen (`-`), not underscore

---

## Sources

- [Upbit Open API - REST API Guide](https://global-docs.upbit.com/reference/rest-api-guide)
- [Upbit Open API - Minutes Candles](https://global-docs.upbit.com/v1.2.2/reference/minutes)
- [Upbit Open API - Market Ask Order Creation](https://global-docs.upbit.com/docs/market-ask-order-creation)
- [Tardis.dev - Upbit Historical Data](https://docs.tardis.dev/historical-data-details/upbit)
- [CCXT - Upbit Integration](https://github.com/ccxt/ccxt/blob/master/python/ccxt/upbit.py)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
