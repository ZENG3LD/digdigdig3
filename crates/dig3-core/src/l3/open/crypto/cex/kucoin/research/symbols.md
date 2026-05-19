# KuCoin Symbol/Pair Formatting Research

**Research Date**: 2026-01-20
**Purpose**: Document exact KuCoin API symbol formatting, intervals, timestamps, and order parameters

---

## 1. SPOT Symbol Format

### Format
- **Pattern**: `BASE-QUOTE` with hyphen separator
- **Case**: UPPERCASE (all official documentation uses uppercase)
- **Examples**:
  - `BTC-USDT`
  - `ETH-BTC`
  - `KCS-BTC`
  - `XLM-USDT`

### Case Sensitivity
While not explicitly stated in documentation, **all official examples use UPPERCASE exclusively**. Best practice: always use uppercase symbols.

### API Response Format
```json
{
  "symbol": "BTC-USDT",
  "baseCurrency": "BTC",
  "quoteCurrency": "USDT"
}
```

---

## 2. FUTURES Symbol Format

### Perpetual Contracts

#### USDT-Margined Perpetual (Mini Contracts)
- **Pattern**: `{BASE}USDTM`
- **Bitcoin mapping**: `BTC` → `XBT` (ISO 4217 standard)
- **Examples**:
  - `XBTUSDTM` - BTC/USDT perpetual mini (0.001 BTC per contract)
  - `ETHUSDTM` - ETH/USDT perpetual mini (0.01 ETH per contract)
  - `SOLUSDTM` - SOL/USDT perpetual mini
  - `ADAUSDTM` - ADA/USDT perpetual mini

#### USD-Margined Perpetual (Inverse/Coin-Margined)
- **Pattern**: `{BASE}USDM`
- **Bitcoin mapping**: `BTC` → `XBT`
- **Examples**:
  - `XBTUSDM` - BTC/USD perpetual (1 USD of BTC per contract)
  - `ETHUSDM` - ETH/USD perpetual (1 USD of ETH per contract)

### Delivery Contracts (Quarterly)

- **Pattern**: `{BASE}USD-{DD}{MMM}{YY}`
- **Examples**:
  - `BTCUSD-27MAR26` - BTC quarterly contract expiring March 27, 2026
  - `BTCUSD-26DEC25` - BTC quarterly contract expiring December 26, 2025

**Notes**:
- Delivery date is the last Friday of the contract month
- Currently only BTC coin-margined quarterly contracts are offered
- New quarterly contract listed on Friday two weeks before current delivery

### Symbol Mapping Rules

| Crypto | Spot Symbol | Futures Symbol (USDT) | Futures Symbol (USD) |
|--------|-------------|----------------------|---------------------|
| Bitcoin | BTC-USDT | XBTUSDTM | XBTUSDM |
| Ethereum | ETH-USDT | ETHUSDTM | ETHUSDM |
| Solana | SOL-USDT | SOLUSDTM | (if available) |
| Cardano | ADA-USDT | ADAUSDTM | (if available) |

**Critical**: Only BTC uses the XBT mapping. ETH, SOL, ADA, and other cryptos use their standard tickers.

---

## 3. Kline Intervals

### SPOT Kline Intervals

**Parameter**: `type` (string)

**Supported values**:
- `1min`, `3min`, `5min`, `15min`, `30min`
- `1hour`, `2hour`, `4hour`, `6hour`, `8hour`, `12hour`
- `1day`, `1week`, `1month`

**API Endpoint**: `/api/v1/market/candles`

**Example Request**:
```
GET /api/v1/market/candles?symbol=BTC-USDT&type=1min&startAt=1566703297&endAt=1566789757
```

### FUTURES Kline Intervals

**Parameter**: `granularity` (integer representing minutes)

**Supported values**:
- `1` (1 minute)
- `5` (5 minutes)
- `15` (15 minutes)
- `30` (30 minutes)
- `60` (1 hour)
- `120` (2 hours)
- `240` (4 hours)
- `480` (8 hours)
- `720` (12 hours)
- `1440` (1 day)
- `10080` (1 week)

**API Endpoint**: `/api/v1/kline/query`

**Example Request**:
```
GET /api/v1/kline/query?symbol=XBTUSDTM&granularity=60&startAt=1566703297000&endAt=1566789757000
```

### WebSocket Kline Intervals

**Parameter**: `type` in topic subscription (string)

**Supported values** (same for both Spot and Futures WebSocket):
- `1min`, `3min`, `5min`, `15min`, `30min`
- `1hour`, `2hour`, `4hour`, `8hour`, `12hour`
- `1day`, `1week`, `1month`

### Differences Between Spot and Futures

| Feature | Spot | Futures |
|---------|------|---------|
| Parameter name | `type` | `granularity` |
| Value format | String (e.g., "1min") | Integer minutes (e.g., 1) |
| 3-minute interval | Yes (`3min`) | No |
| 6-hour interval | Yes (`6hour`) | No |
| Monthly interval | Yes (`1month`) | No |
| Week interval | Yes (`10080` = 1 week) | Yes (`10080` = 1 week) |

### Data Limits

- **Maximum records per request**: 1500 (Spot), 500 (Futures)
- If time range exceeds limit, only first 500/1500 records returned

---

## 4. Time Formats

### Timestamp Format

**All timestamps use MILLISECONDS (not seconds)**

### Request Parameters

- **Parameter names**: `startAt`, `endAt` (NOT `start`/`end` or `from`/`to`)
- **Format**: Unix timestamp in milliseconds
- **Type**: `int64` / `long`
- **Example**: `1566703297000` (not `1566703297`)

### Response Timestamps

- **Format**: Milliseconds since Unix Epoch
- **Example**: `1550653727731`

### Authentication Header

- **Header**: `KC-API-TIMESTAMP`
- **Format**: Milliseconds since Unix Epoch in UTC
- **Example**: `1547015186532`

### Kline Response Format

Kline data returns arrays:
```
[timestamp, open, close, high, low, volume, transaction_amount]
```

Where `timestamp` is in milliseconds.

---

## 5. Order Parameters

### Side (Transaction Direction)

**Parameter**: `side`

**Supported values**:
- `buy` - Buy order
- `sell` - Sell order

**Case**: Lowercase

### Type (Order Type)

**Parameter**: `type`

**Supported values**:
- `limit` - Limit order (price and size required)
- `market` - Market order (immediate execution)

**Case**: Lowercase

**Details**:
- **Limit**: Order filled at specified price or better. Remains in order book if not immediately filled.
- **Market**: Executed immediately at market price. No guarantee on execution price.

### Time in Force

**Parameter**: `timeInForce`

**Supported values**:
- `GTC` - Good Till Canceled (default if not specified)
- `IOC` - Immediate Or Cancel
- `GTT` - Good Till Time (less common)
- `FOK` - Fill or Kill (less common)

**Case**: UPPERCASE

**Details**:
- **GTC**: Order remains open until canceled
- **IOC**: Remaining size instantly canceled if not immediately matched
- **GTT**: Order valid until specified time
- **FOK**: Order must be completely filled immediately or canceled entirely

---

## 6. Comparison with Current Implementation

### Current `endpoints.rs` Implementation

```rust
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            format!("{}-{}", base, quote)
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // KuCoin Futures: BTC → XBT, формат XBTUSDM
            let base = if base == "BTC" { "XBT" } else { base };
            format!("{}{}M", base, quote)
        }
    }
}
```

### Issues Found

1. **Missing "T" in USDT futures**: Current implementation produces `XBTUSDSM` but should be `XBTUSDTM` for USDT-margined
2. **All futures get "M" suffix**: Should only apply to perpetuals, not delivery contracts
3. **No distinction**: Between USDT-margined (`USDTM`) and USD-margined (`USDM`)

### Corrected Implementation

```rust
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot: BASE-QUOTE with hyphen
            format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Futures perpetual: BASEUSDTM or BASEUSDM
            // BTC → XBT mapping for futures only
            let base = if base.to_uppercase() == "BTC" { "XBT" } else { &base.to_uppercase() };

            // Determine contract type by quote currency
            match quote.to_uppercase().as_str() {
                "USDT" => format!("{}USDTM", base),  // USDT-margined
                "USD" => format!("{}USDM", base),    // USD-margined (inverse)
                _ => format!("{}{}M", base, quote.to_uppercase()), // Generic fallback
            }
        }
    }
}
```

### Kline Interval Mapping

```rust
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "3m" => "3min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h" => "1hour",
        "2h" => "2hour",
        "4h" => "4hour",
        "6h" => "6hour",
        "8h" => "8hour",
        "12h" => "12hour",
        "1d" => "1day",
        "1w" => "1week",
        "1M" => "1month",
        _ => "1hour",
    }
}
```

**Status**: CORRECT for Spot API

**Issue**: Futures API uses numeric granularity (minutes as integer), not string intervals

### Needed: Separate Futures Interval Mapping

```rust
pub fn map_futures_granularity(interval: &str) -> u32 {
    match interval {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "2h" => 120,
        "4h" => 240,
        "8h" => 480,
        "12h" => 720,
        "1d" => 1440,
        "1w" => 10080,
        _ => 60, // default 1 hour
    }
}
```

---

## 7. Summary of Critical Findings

### Symbol Formatting

| Market | Format | BTC Example | ETH Example |
|--------|--------|-------------|-------------|
| Spot | `BASE-QUOTE` | BTC-USDT | ETH-USDT |
| Futures USDT | `BASEUSDTM` | XBTUSDTM | ETHUSDTM |
| Futures USD | `BASEUSDM` | XBTUSDM | ETHUSDM |
| Delivery | `BASEUSD-DDMMMYY` | BTCUSD-27MAR26 | N/A |

### Kline Intervals

- **Spot REST**: String format (`"1min"`, `"1hour"`)
- **Futures REST**: Integer minutes (`1`, `60`)
- **WebSocket (both)**: String format (`"1min"`, `"1hour"`)

### Timestamps

- **ALL timestamps**: Milliseconds (not seconds)
- **Parameter names**: `startAt`, `endAt`
- **Auth header**: `KC-API-TIMESTAMP` in milliseconds

### Order Parameters

- **side**: `"buy"` or `"sell"` (lowercase)
- **type**: `"limit"` or `"market"` (lowercase)
- **timeInForce**: `"GTC"`, `"IOC"`, `"GTT"`, `"FOK"` (uppercase)

### Special Mappings

- **BTC → XBT**: ONLY for futures contracts
- **ETH, SOL, ADA, etc.**: Use standard tickers (no mapping)

---

## Sources

- [KuCoin Get Symbol Detail - Spot](https://www.kucoin.com/docs/rest/spot-trading/market-data/get-symbol-detail)
- [KuCoin Get Symbols List - Futures](https://www.kucoin.com/docs/rest/futures-trading/market-data/get-symbols-list)
- [KuCoin Get Klines - Futures](https://www.kucoin.com/docs/rest/futures-trading/market-data/get-klines)
- [KuCoin Get Klines - Spot](https://www.kucoin.com/docs-new/rest/spot-trading/market-data/get-klines)
- [KuCoin Klines WebSocket](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/klines)
- [KuCoin Add Order](https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-order)
- [KuCoin Place Order - Futures](https://www.kucoin.com/docs/rest/futures-trading/orders/place-order)
- [KuCoin Timestamps](https://www.kucoin.com/docs/basic-info/connection-method/types/timestamps)
- [XBTUSDTM Contract Specifications](https://www.kucoin.com/futures/contract/detail/XBTUSDTM)
- [ETHUSDTM Contract Specifications](https://www.kucoin.com/futures/contract/detail/ETHUSDTM)
- [KuCoin Coin-Margined Contract Delivery Rules](https://www.kucoin.com/support/26692732384409)
- [KuCoin Contract Types](https://www.kucoin.com/support/4402362095897)
- [Python KuCoin API Documentation](https://python-kucoin.readthedocs.io/en/latest/kucoin.html)
- [KuCoin Futures Go SDK - kline.go](https://github.com/Kucoin/kucoin-futures-go-sdk/blob/main/kline.go)
- [KuCoin API Docs - GitHub](https://github.com/Kucoin/kucoin-api-docs/blob/master/source/localizable/index.html.md)
