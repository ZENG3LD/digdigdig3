# Coinbase Advanced Trade API Symbol/Product Formatting Research

**Research Date**: 2026-01-20
**Purpose**: Document exact Coinbase API symbol formatting, intervals, timestamps, and order parameters

---

## 1. SPOT Symbol Format

### Format
- **Pattern**: `BASE-QUOTE` with hyphen separator
- **Case**: **Case-sensitive** - use UPPERCASE for consistency
- **Examples**:
  - `BTC-USD`
  - `ETH-USD`
  - `BTC-USDT`
  - `ETH-EUR`
  - `DOGE-USD`

### Case Sensitivity
- Coinbase API is **case-sensitive**
- Official documentation uses **UPPERCASE** exclusively
- **Best practice**: Always use uppercase symbols (e.g., `BTC-USD`, not `btc-usd`)

### API Response Format
```json
{
  "product_id": "BTC-USD",
  "base_currency_id": "BTC",
  "quote_currency_id": "USD",
  "base_display_symbol": "BTC",
  "quote_display_symbol": "USD",
  "base_name": "Bitcoin",
  "quote_name": "US Dollar"
}
```

---

## 2. FUTURES Symbol Format

**Important**: Coinbase Advanced Trade API **does not support traditional futures trading** like Binance or KuCoin.

Coinbase does offer:
- **Perpetual Futures (INTX)** - Available on Coinbase International Exchange, but uses different API
- **CFM (Coinbase Financial Markets)** - Derivatives for institutional clients

For retail Advanced Trade API:
- **Only SPOT trading** is available
- No perpetual contracts with `M` suffix like KuCoin (`XBTUSDTM`)
- No delivery contracts

---

## 3. Kline Intervals (Granularity)

### Supported Granularities

**Parameter**: `granularity` (enum string)

**Supported values**:
- `UNKNOWN_GRANULARITY` (default, not recommended)
- `ONE_MINUTE`
- `FIVE_MINUTE`
- `FIFTEEN_MINUTE`
- `THIRTY_MINUTE`
- `ONE_HOUR`
- `TWO_HOUR`
- `SIX_HOUR`
- `ONE_DAY`

**API Endpoint**: `GET /api/v3/brokerage/products/{product_id}/candles`

**Example Request**:
```
GET /api/v3/brokerage/products/BTC-USD/candles?start=1609459200&end=1609545600&granularity=ONE_HOUR
```

### Granularity Mapping

| Display | Enum Value | Seconds | Minutes |
|---------|------------|---------|---------|
| 1m | `ONE_MINUTE` | 60 | 1 |
| 5m | `FIVE_MINUTE` | 300 | 5 |
| 15m | `FIFTEEN_MINUTE` | 900 | 15 |
| 30m | `THIRTY_MINUTE` | 1800 | 30 |
| 1h | `ONE_HOUR` | 3600 | 60 |
| 2h | `TWO_HOUR` | 7200 | 120 |
| 6h | `SIX_HOUR` | 21600 | 360 |
| 1d | `ONE_DAY` | 86400 | 1440 |

### WebSocket Kline Intervals

**WebSocket candles channel** supports same intervals as REST API:
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "candles",
  "granularity": "ONE_MINUTE"
}
```

### Differences from KuCoin

| Feature | KuCoin Spot | KuCoin Futures | Coinbase |
|---------|-------------|----------------|----------|
| Parameter name | `type` | `granularity` | `granularity` |
| Value format | String (`"1min"`) | Integer (minutes) | Enum (`"ONE_MINUTE"`) |
| 3-minute interval | Yes (`"3min"`) | No | No |
| 6-hour interval | Yes (`"6hour"`) | No | Yes (`"SIX_HOUR"`) |
| Monthly interval | Yes (`"1month"`) | No | No |
| 2-hour interval | Yes (`"2hour"`) | Yes (`120`) | Yes (`"TWO_HOUR"`) |

### Data Limits

- **Maximum records per request**: 300 candles
- If time range exceeds 300 candles, request will be rejected
- Must adjust time range or increase granularity

---

## 4. Time Formats

### Timestamp Format

**Multiple formats used**:

1. **RFC3339** (most common):
   - Format: `"2023-10-26T10:05:30.123Z"`
   - Example: `"2023-01-15T14:30:00.000000Z"`
   - Used in: Order times, account updates, trade times

2. **Unix Seconds** (string):
   - Format: `"1698315930"`
   - Used in: Candle start times, query parameters

3. **Unix Milliseconds** (string):
   - Format: `"1698315930123"`
   - Used in: Server time response (`epochMillis`)

### Request Parameters

- **Parameter names**: `start`, `end` (for candles), `start_date`, `end_date` (for orders)
- **Format**:
  - Candles: Unix timestamp in **seconds** (integer or string)
  - Orders: RFC3339 string (e.g., `"2023-01-01T00:00:00Z"`)
- **Type**: String or integer
- **Example**:
  - `start=1609459200` (Jan 1, 2021 00:00:00 UTC)
  - `start_date=2023-01-01T00:00:00Z`

### Response Timestamps

- **Format**: Primarily RFC3339
- **Precision**: Microseconds (6 decimal places)
- **Example**: `"2023-10-26T10:05:30.123456Z"`

### Authentication Timestamp

- **JWT payload**: `nbf` and `exp` use Unix seconds (integer)
- **Example**: `"nbf": 1706986630, "exp": 1706986750`

### Server Time Endpoint

`GET /api/v3/brokerage/time` returns:
```json
{
  "iso": "2023-10-26T10:05:30.123Z",
  "epochSeconds": "1698315930",
  "epochMillis": "1698315930123"
}
```

---

## 5. Order Parameters

### Side (Transaction Direction)

**Parameter**: `side`

**Supported values**:
- `BUY` - Buy order
- `SELL` - Sell order

**Case**: **UPPERCASE** (different from KuCoin's lowercase)

### Order Types

**Parameter**: Specified in `order_configuration` object

**Supported order types**:

1. **Market IOC** (`market_market_ioc`):
   ```json
   {
     "market_market_ioc": {
       "quote_size": "1000.00"  // For BUY
       // OR
       "base_size": "0.01"       // For SELL
     }
   }
   ```

2. **Limit GTC** (`limit_limit_gtc`):
   ```json
   {
     "limit_limit_gtc": {
       "base_size": "0.01",
       "limit_price": "50000.00",
       "post_only": false
     }
   }
   ```

3. **Limit GTD** (`limit_limit_gtd`):
   ```json
   {
     "limit_limit_gtd": {
       "base_size": "0.01",
       "limit_price": "50000.00",
       "end_time": "2023-12-31T23:59:59Z",
       "post_only": false
     }
   }
   ```

4. **Limit FOK** (`limit_limit_fok`):
   ```json
   {
     "limit_limit_fok": {
       "base_size": "0.01",
       "limit_price": "50000.00"
     }
   }
   ```

5. **Stop Limit GTC** (`stop_limit_stop_limit_gtc`):
   ```json
   {
     "stop_limit_stop_limit_gtc": {
       "base_size": "0.01",
       "limit_price": "50000.00",
       "stop_price": "49000.00",
       "stop_direction": "STOP_DIRECTION_STOP_DOWN"
     }
   }
   ```

### Time in Force

**Implicit in order configuration key**:
- `GTC` - Good Till Canceled (in `limit_limit_gtc`)
- `GTD` - Good Till Date (in `limit_limit_gtd`)
- `IOC` - Immediate Or Cancel (in `market_market_ioc`)
- `FOK` - Fill or Kill (in `limit_limit_fok`)

**Case**: UPPERCASE

**Differences from KuCoin**:
- KuCoin uses separate `timeInForce` parameter
- Coinbase embeds time in force in order configuration type
- Coinbase uses double naming (e.g., `limit_limit_gtc`)

### Post-Only Flag

**Parameter**: `post_only` (boolean, within order configuration)
**Supported on**: Limit orders only
**Default**: `false`

---

## 6. Comparison with Current Implementation

### Symbol Formatting

**Spot Symbol Comparison**:

| Exchange | Format | Example | Case |
|----------|--------|---------|------|
| KuCoin Spot | `BASE-QUOTE` | `BTC-USDT` | UPPERCASE |
| Coinbase Spot | `BASE-QUOTE` | `BTC-USD` | UPPERCASE |

**No conversion needed** - both use hyphen separator.

**Futures Symbol Comparison**:

| Exchange | Format | Example |
|----------|--------|---------|
| KuCoin Futures | `BASEUSDTM` | `XBTUSDTM` |
| Coinbase | N/A | No futures support |

### Kline Interval Mapping

**From common format to Coinbase**:

```rust
pub fn map_coinbase_granularity(interval: &str) -> &'static str {
    match interval {
        "1m" => "ONE_MINUTE",
        "5m" => "FIVE_MINUTE",
        "15m" => "FIFTEEN_MINUTE",
        "30m" => "THIRTY_MINUTE",
        "1h" => "ONE_HOUR",
        "2h" => "TWO_HOUR",
        "6h" => "SIX_HOUR",
        "1d" => "ONE_DAY",
        _ => "ONE_HOUR",  // default
    }
}
```

**Reverse mapping** (Coinbase to seconds):

```rust
pub fn coinbase_granularity_to_seconds(granularity: &str) -> u64 {
    match granularity {
        "ONE_MINUTE" => 60,
        "FIVE_MINUTE" => 300,
        "FIFTEEN_MINUTE" => 900,
        "THIRTY_MINUTE" => 1800,
        "ONE_HOUR" => 3600,
        "TWO_HOUR" => 7200,
        "SIX_HOUR" => 21600,
        "ONE_DAY" => 86400,
        _ => 3600,
    }
}
```

---

## 7. Summary of Critical Findings

### Symbol Formatting

| Market | Format | Example |
|--------|--------|---------|
| Spot | `BASE-QUOTE` | `BTC-USD`, `ETH-USDT` |
| Futures | N/A | Not supported |

### Kline Intervals

- **Spot**: Enum strings (`ONE_MINUTE`, `FIVE_MINUTE`, etc.)
- **No 3-minute** interval (unlike KuCoin)
- **Has 2-hour** interval (like KuCoin)
- **No monthly** interval (unlike KuCoin)

### Timestamps

- **Requests**: Unix seconds (candles) or RFC3339 (orders)
- **Responses**: Primarily RFC3339 with microsecond precision
- **JWT**: Unix seconds for nbf/exp

### Order Parameters

- **side**: `"BUY"` or `"SELL"` (UPPERCASE)
- **order_configuration**: Complex nested object (not simple type string)
- **time_in_force**: Embedded in configuration key name

### Special Mappings

- **No BTC → XBT mapping** (Coinbase uses `BTC` directly)
- **Quote currencies**: USD, USDT, EUR, GBP, etc. (not just USDT)

---

## 8. Implementation Notes

### Format Symbol Function

```rust
pub fn format_symbol(base: &str, quote: &str) -> String {
    // Coinbase only supports spot trading
    // No futures conversion needed
    format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
}
```

**Simple implementation** - no account type distinction needed.

### Parse Symbol Function

```rust
pub fn parse_symbol(product_id: &str) -> (String, String) {
    let parts: Vec<&str> = product_id.split('-').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}
```

---

## Sources

- [Coinbase Advanced Trade API - Overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/overview)
- [Get Product Candles](https://docs.cdp.coinbase.com/api-reference/exchange-api/rest-api/products/get-product-candles)
- [Create Order](https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/orders/create-order)
- [Coinbase Advanced Python SDK](https://github.com/coinbase/coinbase-advanced-py)
- [Coinbase API Cheat Sheet](https://vezgo.com/blog/coinbase-api-cheat-sheet-for-developers/)
- [How to Use Coinbase API](https://apidog.com/blog/coinbase-api-5/)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Key Difference:** Coinbase uses enum strings for granularity (`ONE_MINUTE`) vs KuCoin's string intervals (`"1min"`) or integer minutes.
