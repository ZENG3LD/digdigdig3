# Bybit V5 Symbol/Pair Formatting Research

**Research Date**: 2026-01-20
**Purpose**: Document exact Bybit V5 API symbol formatting, intervals, timestamps, and order parameters

---

## 1. SPOT Symbol Format

### Format
- **Pattern**: `BASEQUOTE` with no separator
- **Case**: UPPERCASE (all official documentation uses uppercase)
- **Examples**:
  - `BTCUSDT`
  - `ETHUSDT`
  - `SOLUSDT`
  - `ETHBTC`

### Case Sensitivity
While not explicitly stated in documentation, **all official examples use UPPERCASE exclusively**. Best practice: always use uppercase symbols.

### API Response Format
```json
{
  "symbol": "BTCUSDT",
  "baseCoin": "BTC",
  "quoteCoin": "USDT"
}
```

**Key Point**: No hyphen or separator - just `BTCUSDT`, not `BTC-USDT`

---

## 2. FUTURES Symbol Format

### Linear Perpetual (USDT-Margined)

**Pattern**: `BASEQUOTE` (same as spot)

**Examples**:
- `BTCUSDT` - BTC/USDT perpetual
- `ETHUSDT` - ETH/USDT perpetual
- `SOLUSDT` - SOL/USDT perpetual

**No Special Suffix**: Unlike KuCoin which uses `XBTUSDTM`, Bybit uses plain `BTCUSDT` for perpetuals

### Inverse Perpetual (Coin-Margined)

**Pattern**: `BASEUSD`

**Examples**:
- `BTCUSD` - BTC/USD inverse perpetual
- `ETHUSD` - ETH/USD inverse perpetual

### USDC Perpetual

**Pattern**: `BASEQUOTE-PERP`

**Examples**:
- `BTCUSDC-PERP` - BTC/USDC perpetual
- `ETHUSDC-PERP` - ETH/USDC perpetual

### USDC Futures (Delivery)

**Pattern**: `BASEUSD-DDMMMYY`

**Examples**:
- `BTCUSD-310125` - BTC futures expiring January 31, 2025
- `BTCUSD-280325` - BTC futures expiring March 28, 2025

### USDT Futures (Delivery)

**Pattern**: `BASEUSD-DDMMMYY`

**Examples**:
- `BTCUSDT-280325` - BTC/USDT futures expiring March 28, 2025

### Symbol Mapping Rules

| Crypto | Spot Symbol | Linear Perpetual | Inverse Perpetual | USDC Perpetual |
|--------|-------------|------------------|-------------------|----------------|
| Bitcoin | BTCUSDT | BTCUSDT | BTCUSD | BTCUSDC-PERP |
| Ethereum | ETHUSDT | ETHUSDT | ETHUSD | ETHUSDC-PERP |
| Solana | SOLUSDT | SOLUSDT | (not available) | SOLUSDC-PERP |

**Critical**: No BTC→XBT mapping like KuCoin. All symbols use standard tickers.

---

## 3. Category Parameter

Bybit V5 distinguishes product types using the `category` parameter:

| Category Value | Description | Example Symbols |
|---------------|-------------|-----------------|
| `spot` | Spot trading | `BTCUSDT`, `ETHUSDT` |
| `linear` | USDT/USDC Perpetual & Futures | `BTCUSDT`, `ETHUSDT` |
| `inverse` | Inverse Perpetual & Futures | `BTCUSD`, `ETHUSD` |
| `option` | Options | `BTC-30DEC22-50000-C` |

**Example**:
```
GET /v5/market/tickers?category=spot&symbol=BTCUSDT
GET /v5/market/tickers?category=linear&symbol=BTCUSDT
```

Same symbol `BTCUSDT` can be used for both spot and linear perpetual, differentiated by `category` parameter.

---

## 4. Kline Intervals

### Supported Interval Values

**Parameter**: `interval` (string or integer)

**Supported values**:
- Minutes: `1`, `3`, `5`, `15`, `30`, `60`, `120`, `240`, `360`, `720`
- Day: `D`
- Week: `W`
- Month: `M`

**API Endpoint**: `/v5/market/kline`

**Example Request**:
```
GET /v5/market/kline?category=spot&symbol=BTCUSDT&interval=60&limit=200
```

### Interval Mapping

| Standard | Bybit V5 | Description |
|----------|----------|-------------|
| 1m | `1` | 1 minute |
| 3m | `3` | 3 minutes |
| 5m | `5` | 5 minutes |
| 15m | `15` | 15 minutes |
| 30m | `30` | 30 minutes |
| 1h | `60` | 1 hour (60 minutes) |
| 2h | `120` | 2 hours |
| 4h | `240` | 4 hours |
| 6h | `360` | 6 hours |
| 12h | `720` | 12 hours |
| 1d | `D` | 1 day |
| 1w | `W` | 1 week |
| 1M | `M` | 1 month |

### WebSocket Kline Intervals

WebSocket uses the same interval format:
- Topic: `kline.{interval}.{symbol}`
- Example: `kline.60.BTCUSDT` for 1-hour BTC/USDT candles

### Data Limits

- **Maximum records per request**: 1000
- If time range exceeds limit, only 1000 records returned

### Differences from KuCoin

| Feature | Bybit V5 | KuCoin Spot | KuCoin Futures |
|---------|----------|-------------|----------------|
| Interval format | Integer minutes or letter | String (e.g., "1min") | Integer minutes |
| 1 hour | `60` | `"1hour"` | `60` |
| 1 day | `D` | `"1day"` | `1440` |
| 3 minutes | `3` | `"3min"` | Not available |
| 6 hours | `360` | `"6hour"` | Not available |

---

## 5. Time Formats

### Timestamp Format

**All timestamps use MILLISECONDS (not seconds)**

### Request Parameters

- **Parameter names**: `start`, `end` (NOT `startAt`, `endAt` like KuCoin)
- **Format**: Unix timestamp in milliseconds
- **Type**: `int64` / `long`
- **Example**: `1702617474601` (not `1702617474`)

### Response Timestamps

- **Format**: Milliseconds since Unix Epoch
- **Example**: `1702617474601`

### Authentication Header

- **Header**: `X-BAPI-TIMESTAMP`
- **Format**: Milliseconds since Unix Epoch in UTC
- **Example**: `1702617474601`

### Kline Response Format

Kline data returns arrays:
```
[startTime, open, high, low, close, volume, turnover]
```

Where `startTime` is in **milliseconds** (not seconds like KuCoin spot).

**Difference from KuCoin**:
- ✅ Bybit: All timestamps in milliseconds (consistent)
- ❌ KuCoin: Spot kline times in seconds, everything else in milliseconds

---

## 6. Order Parameters

### Side (Transaction Direction)

**Parameter**: `side`

**Supported values**:
- `Buy` - Buy order (capitalized!)
- `Sell` - Sell order (capitalized!)

**Case**: Capitalized (not lowercase like KuCoin)

**Example**:
```json
{
  "category": "spot",
  "symbol": "BTCUSDT",
  "side": "Buy",
  "orderType": "Limit",
  "qty": "0.01",
  "price": "40000"
}
```

### Type (Order Type)

**Parameter**: `orderType`

**Supported values**:
- `Limit` - Limit order (capitalized!)
- `Market` - Market order (capitalized!)

**Case**: Capitalized

**Details**:
- **Limit**: Order filled at specified price or better. Remains in order book if not immediately filled.
- **Market**: Executed immediately. Converts to IOC limit in futures to protect against slippage.

### Time in Force

**Parameter**: `timeInForce`

**Supported values**:
- `GTC` - Good Till Canceled (default if not specified)
- `IOC` - Immediate Or Cancel
- `FOK` - Fill or Kill
- `PostOnly` - Post-only (maker only)

**Case**: UPPERCASE

**Details**:
- **GTC**: Order remains open until canceled
- **IOC**: Remaining size instantly canceled if not immediately matched
- **FOK**: Order must be completely filled immediately or canceled entirely
- **PostOnly**: Order will only be maker, rejected if would take liquidity

### Position Index (Hedge Mode)

**Parameter**: `positionIdx`

**Supported values**:
- `0` - One-way mode
- `1` - Buy side of hedge mode
- `2` - Sell side of hedge mode

**Details**:
- One-way mode: Single position per symbol
- Hedge mode: Separate long and short positions

---

## 7. Comparison with Current Implementation

### Symbol Format Differences

| Exchange | Spot Format | Futures Format | Mapping Rules |
|----------|-------------|----------------|---------------|
| **Bybit** | `BTCUSDT` | `BTCUSDT` | No mapping |
| **KuCoin** | `BTC-USDT` | `XBTUSDTM` | BTC→XBT for futures |

### Needed Implementation

```rust
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot => {
            // Bybit Spot: BTCUSDT (no separator)
            format!("{}{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Bybit Linear Perpetual: BTCUSDT (same as spot)
            format!("{}{}", base.to_uppercase(), quote.to_uppercase())
        }
        _ => {
            format!("{}{}", base.to_uppercase(), quote.to_uppercase())
        }
    }
}
```

**Note**: Same format for spot and futures, differentiated by `category` parameter in API calls.

### Interval Mapping

```rust
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "3m" => "3",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "1h" => "60",
        "2h" => "120",
        "4h" => "240",
        "6h" => "360",
        "12h" => "720",
        "1d" => "D",
        "1w" => "W",
        "1M" => "M",
        _ => "60", // default 1 hour
    }
}
```

**Status**: Simple integer/letter format for all products

---

## 8. Summary of Critical Findings

### Symbol Formatting

| Market | Format | BTC Example | ETH Example |
|--------|--------|-------------|-------------|
| Spot | `BASEQUOTE` | BTCUSDT | ETHUSDT |
| Linear Perpetual | `BASEQUOTE` | BTCUSDT | ETHUSDT |
| Inverse Perpetual | `BASEUSD` | BTCUSD | ETHUSD |
| USDC Perpetual | `BASEQUOTE-PERP` | BTCUSDC-PERP | ETHUSDC-PERP |
| Delivery | `BASEUSD-DDMMMYY` | BTCUSD-280325 | ETHUSD-280325 |

### Kline Intervals

- **All products**: Integer minutes or letter (D/W/M)
- **1 hour**: `60` (not `"1hour"`)
- **1 day**: `D` (not `"1day"` or `1440`)

### Timestamps

- **ALL timestamps**: Milliseconds (consistent across all endpoints)
- **Parameter names**: `start`, `end` (not `startAt`, `endAt`)
- **Auth header**: `X-BAPI-TIMESTAMP` in milliseconds

### Order Parameters

- **side**: `"Buy"` or `"Sell"` (capitalized!)
- **orderType**: `"Limit"` or `"Market"` (capitalized!)
- **timeInForce**: `"GTC"`, `"IOC"`, `"FOK"`, `"PostOnly"` (uppercase)

### Special Characteristics

- **No BTC→XBT mapping**: Unlike KuCoin, uses standard BTC ticker
- **No separators**: `BTCUSDT`, not `BTC-USDT` or `BTC_USDT`
- **Unified format**: Same symbol for spot and perpetual, differentiated by `category`
- **Capitalized enums**: Side and orderType values are capitalized

---

## 9. WebSocket Symbol Format

### Public Channels

**Topic Format**: `{channel}.{depth}.{symbol}`

**Examples**:
- `orderbook.1.BTCUSDT` - Level 1 orderbook for BTC/USDT
- `orderbook.50.BTCUSDT` - 50-level orderbook for BTC/USDT
- `publicTrade.BTCUSDT` - Public trades for BTC/USDT
- `kline.60.BTCUSDT` - 1-hour klines for BTC/USDT

### Private Channels

**Topic Format**: `{channel}`

**Examples**:
- `order` - Order updates
- `execution` - Trade executions
- `wallet` - Wallet balance updates
- `position` - Position updates

Private channels apply to all symbols automatically based on account activity.

---

## Sources

Research compiled from official Bybit V5 API documentation:

- [Get Instruments Info | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/market/instrument)
- [Introduction | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/intro)
- [Get Tickers | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/market/tickers)
- [Get Kline | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/market/kline)
- [Place Order | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/order/create-order)
- [Ticker WebSocket | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker)
- [Bybit V5 Changelog](https://bybit-exchange.github.io/docs/changelog/v5)
- [Bybit NautilusTrader Integration](https://nautilustrader.io/docs/nightly/integrations/bybit/)
- [GitHub - philipperemy/bitpy](https://github.com/philipperemy/bitpy)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Key Finding:** Bybit uses unified symbol format across spot/futures, no separators, no BTC→XBT mapping
