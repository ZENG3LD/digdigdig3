# Bithumb API Response Formats

## Overview

Bithumb has different response formats depending on the platform:
- **Bithumb Korea**: Simple status-based format
- **Bithumb Pro**: Extended format with success flags and codes

---

## Bithumb Korea Response Format

### Standard Success Response

```json
{
  "status": "0000",
  "data": {...}
}
```

### Status Codes

| Status Code | Description | Type |
|-------------|-------------|------|
| `0000` | Success | Success |
| `5100` | Bad Request | Error |
| `5200` | Not Member | Error |
| `5300` | Invalid Apikey | Error |
| `5302` | Method Not Allowed | Error |
| `5400` | Database Fail | Error |
| `5500` | Invalid Parameter | Error |
| `5600` | CUSTOM NOTICE (usually maintenance) | Error |

### Data Field Types

The `data` field structure varies by endpoint:

**Single Object**:
```json
{
  "status": "0000",
  "data": {
    "field1": "value1",
    "field2": "value2"
  }
}
```

**Array**:
```json
{
  "status": "0000",
  "data": [
    {...},
    {...}
  ]
}
```

**Nested by Currency** (common for ticker/orderbook):
```json
{
  "status": "0000",
  "data": {
    "BTC": {
      "opening_price": "50000000",
      "closing_price": "51000000",
      ...
    },
    "ETH": {
      "opening_price": "3000000",
      "closing_price": "3100000",
      ...
    }
  }
}
```

---

## Bithumb Korea Endpoint-Specific Formats

### Ticker Response

**Endpoint**: `GET /public/ticker/{symbol}`

**Single Symbol** (`BTC_KRW`):
```json
{
  "status": "0000",
  "data": {
    "opening_price": "50000000",
    "closing_price": "51000000",
    "min_price": "49500000",
    "max_price": "52000000",
    "units_traded": "123.45678900",
    "acc_trade_value": "6234567890.12345",
    "prev_closing_price": "50500000",
    "units_traded_24H": "234.56789000",
    "acc_trade_value_24H": "11876543210.98765",
    "fluctate_24H": "500000",
    "fluctate_rate_24H": "0.99"
  }
}
```

**All Symbols** (`ALL_KRW`):
```json
{
  "status": "0000",
  "data": {
    "BTC": {
      "opening_price": "50000000",
      "closing_price": "51000000",
      ...
    },
    "ETH": {...},
    "XRP": {...},
    "date": "1712230310689"
  }
}
```

**Field Types**:
- All price fields: String (formatted as integer for KRW)
- Volume fields: String (with decimal precision)
- Timestamp: String (milliseconds)

### Order Book Response

**Endpoint**: `GET /public/orderbook/{symbol}`

```json
{
  "status": "0000",
  "data": {
    "timestamp": "1712230310689",
    "order_currency": "BTC",
    "payment_currency": "KRW",
    "bids": [
      {
        "quantity": "0.12345678",
        "price": "50000000"
      },
      {
        "quantity": "0.23456789",
        "price": "49990000"
      }
    ],
    "asks": [
      {
        "quantity": "0.34567890",
        "price": "50010000"
      },
      {
        "quantity": "0.45678901",
        "price": "50020000"
      }
    ]
  }
}
```

**Array Format**:
- `bids`: Sorted highest to lowest price
- `asks`: Sorted lowest to highest price
- Maximum 30 levels (default 30)

### Recent Trades Response

**Endpoint**: `GET /public/transaction_history/{symbol}`

```json
{
  "status": "0000",
  "data": [
    {
      "transaction_date": "2024-04-04 12:34:56",
      "type": "bid",
      "units_traded": "0.12345678",
      "price": "50000000",
      "total": "6172839000"
    },
    {
      "transaction_date": "2024-04-04 12:34:55",
      "type": "ask",
      "units_traded": "0.23456789",
      "price": "49990000",
      "total": "11728394555"
    }
  ]
}
```

**Field Details**:
- `transaction_date`: Format `YYYY-MM-DD HH:mm:ss`
- `type`: `"bid"` (buy) or `"ask"` (sell)
- `units_traded`: String with decimal
- `price`: String (KRW price)
- `total`: String (price * quantity in KRW)

### Candlestick Response

**Endpoint**: `GET /public/candlestick/{base}_{quote}/{interval}`

```json
{
  "status": "0000",
  "data": [
    [
      1712230310000,    // timestamp (ms)
      "50000000",       // open
      "51000000",       // close
      "52000000",       // high
      "49500000",       // low
      "123.45678900"    // volume
    ],
    [
      1712230370000,
      "51000000",
      "50500000",
      "51500000",
      "50000000",
      "234.56789000"
    ]
  ]
}
```

**Array Format**: `[timestamp, open, close, high, low, volume]`

### Balance Response

**Endpoint**: `POST /info/balance` with `currency=ALL`

```json
{
  "status": "0000",
  "data": {
    "total_btc": "1.23456789",
    "total_krw": "10000000",
    "in_use_btc": "0.12345678",
    "in_use_krw": "1000000",
    "available_btc": "1.11111111",
    "available_krw": "9000000",
    "xcoin_last_btc": "50000000",
    "total_eth": "10.23456789",
    "in_use_eth": "1.12345678",
    "available_eth": "9.11111111",
    "xcoin_last_eth": "3000000"
  }
}
```

**Field Pattern**: `{type}_{currency}`
- `total_`: Total balance
- `in_use_`: Locked in orders
- `available_`: Available for trading
- `xcoin_last_`: Average purchase price

### Order Details Response

**Endpoint**: `POST /info/order_detail`

```json
{
  "status": "0000",
  "data": {
    "order_id": "1234567890",
    "order_currency": "BTC",
    "payment_currency": "KRW",
    "order_date": "1712230310689",
    "type": "bid",
    "status": "placed",
    "units": "1.00000000",
    "units_remaining": "0.50000000",
    "price": "50000000",
    "fee": "0.00025000",
    "contract": [
      {
        "transaction_date": "1712230320000",
        "price": "50000000",
        "units": "0.25000000",
        "fee_currency": "KRW",
        "fee": "31250"
      },
      {
        "transaction_date": "1712230330000",
        "price": "50010000",
        "units": "0.25000000",
        "fee_currency": "KRW",
        "fee": "31256"
      }
    ]
  }
}
```

**Order Status Values**:
- `"placed"`: Order is open
- `"completed"`: Order is fully filled
- `"cancelled"`: Order was cancelled

### Open Orders Response

**Endpoint**: `POST /info/orders`

```json
{
  "status": "0000",
  "data": [
    {
      "order_id": "1234567890",
      "order_currency": "BTC",
      "payment_currency": "KRW",
      "order_date": "1712230310689",
      "type": "bid",
      "units": "1.00000000",
      "units_remaining": "1.00000000",
      "price": "50000000"
    },
    {
      "order_id": "1234567891",
      "order_currency": "ETH",
      "payment_currency": "KRW",
      "order_date": "1712230320689",
      "type": "ask",
      "units": "10.00000000",
      "units_remaining": "10.00000000",
      "price": "3000000"
    }
  ]
}
```

### Create/Cancel Order Response

**Endpoint**: `POST /trade/place` or `POST /trade/cancel`

```json
{
  "status": "0000",
  "order_id": "1234567890",
  "data": null
}
```

**Market Order Response**:
```json
{
  "status": "0000",
  "order_id": "1234567890",
  "data": [
    {
      "cont_id": "9876543210",
      "units": "0.25000000",
      "price": "50000000",
      "total": "12500000",
      "fee": "3125"
    }
  ]
}
```

---

## Bithumb Pro Response Format

### Standard Success Response

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {...},
  "params": []
}
```

### Response Code System

**Success Codes**: `"0"` or codes below `10000`

**Error Codes**:
| Code | Description |
|------|-------------|
| `10001` | System error |
| `10002` | Invalid parameter |
| `10003` | Illegal request |
| `10004` | Verification failed |
| `10005` | Invalid apiKey |
| `10006` | Invalid sign |
| `10007` | Illegal IP |
| `10008` | Invalid timestamp |
| `20001` | Insufficient balance |
| `20002` | Order not found |
| `20003` | Order cancelled |

### Fields Explanation

| Field | Type | Description |
|-------|------|-------------|
| `code` | String | Status code ("0" = success, others = error) |
| `msg` | String | Message (usually "success" or error description) |
| `success` | Boolean | Quick success check |
| `data` | Object/Array | Response payload |
| `params` | Array | Additional parameters (usually empty) |

---

## Bithumb Pro Endpoint-Specific Formats

### Ticker Response

**Endpoint**: `GET /spot/ticker?symbol=BTC-USDT`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "c": "51000.00",      // current/close price
    "h": "52000.00",      // 24h high
    "l": "49500.00",      // 24h low
    "p": "2.00",          // 24h change percent
    "v": "12345.678",     // 24h volume
    "s": "BTC-USDT",      // symbol
    "ver": "123456789"    // version
  },
  "params": []
}
```

**All Symbols** (`symbol=ALL`):
```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": [
    {
      "c": "51000.00",
      "h": "52000.00",
      "l": "49500.00",
      "p": "2.00",
      "v": "12345.678",
      "s": "BTC-USDT"
    },
    {
      "c": "3100.00",
      "h": "3200.00",
      "l": "2950.00",
      "p": "3.33",
      "v": "123456.789",
      "s": "ETH-USDT"
    }
  ],
  "params": []
}
```

### Order Book Response

**Endpoint**: `GET /spot/orderBook?symbol=BTC-USDT`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "b": [                    // bids
      ["50000.00", "0.123"],
      ["49990.00", "0.234"]
    ],
    "s": [                    // asks
      ["50010.00", "0.345"],
      ["50020.00", "0.456"]
    ],
    "ver": "123456789"        // version number
  },
  "params": []
}
```

**Array Format**: `[price, quantity]`
- `b`: Bids (buy orders)
- `s`: Asks (sell orders)

### Recent Trades Response

**Endpoint**: `GET /spot/trades?symbol=BTC-USDT`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": [
    {
      "p": "50000.00",      // price
      "s": "buy",           // side (buy/sell)
      "v": "0.123",         // volume
      "t": 1712230310689    // timestamp (ms)
    },
    {
      "p": "49990.00",
      "s": "sell",
      "v": "0.234",
      "t": 1712230309689
    }
  ],
  "params": []
}
```

**Last 100 trades** are returned

### Candlestick Response

**Endpoint**: `GET /spot/kline?symbol=BTC-USDT&type=m1&start=1712230000&end=1712233600`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": [
    [
      1712230310000,      // timestamp (ms)
      "50000.00",         // open
      "52000.00",         // high
      "49500.00",         // low
      "51000.00",         // close
      "123.456"           // volume
    ],
    [
      1712230370000,
      "51000.00",
      "51500.00",
      "50000.00",
      "50500.00",
      "234.567"
    ]
  ],
  "params": []
}
```

**Array Format**: `[timestamp, open, high, low, close, volume]`

### Account Balance Response

**Endpoint**: `POST /spot/account`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": [
    {
      "coinType": "BTC",
      "count": "1.23456789",
      "frozen": "0.12345678",
      "available": "1.11111111",
      "btcValue": "1.23456789",
      "type": "coin"
    },
    {
      "coinType": "USDT",
      "count": "100000.00",
      "frozen": "10000.00",
      "available": "90000.00",
      "btcValue": "1.96078431",
      "type": "coin"
    }
  ],
  "params": []
}
```

**Field Details**:
- `count`: Total balance
- `frozen`: Locked in orders
- `available`: Available for trading
- `btcValue`: Approximate value in BTC

### Create Order Response

**Endpoint**: `POST /spot/placeOrder`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "orderId": "1234567890123456789"
  },
  "params": []
}
```

### Cancel Order Response

**Endpoint**: `POST /spot/cancelOrder`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "orderId": "1234567890123456789"
  },
  "params": []
}
```

### Order Detail Response

**Endpoint**: `POST /spot/orderDetail`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "orderId": "1234567890123456789",
    "symbol": "BTC-USDT",
    "type": "limit",
    "side": "buy",
    "price": "50000.00",
    "quantity": "1.00000000",
    "dealQuantity": "0.50000000",
    "dealPrice": "50000.00",
    "status": "trading",
    "fee": "0.00025000",
    "createTime": 1712230310689,
    "updateTime": 1712230320689
  },
  "params": []
}
```

**Order Status Values**:
- `"trading"`: Partially filled
- `"traded"`: Fully filled
- `"cancelled"`: Cancelled
- `"pending"`: Not filled yet

### Open Orders Response

**Endpoint**: `POST /spot/openOrders`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "num": 2,
    "list": [
      {
        "orderId": "1234567890123456789",
        "symbol": "BTC-USDT",
        "type": "limit",
        "side": "buy",
        "price": "50000.00",
        "quantity": "1.00000000",
        "dealQuantity": "0.00000000",
        "status": "pending",
        "createTime": 1712230310689
      },
      {
        "orderId": "1234567890123456790",
        "symbol": "ETH-USDT",
        "type": "limit",
        "side": "sell",
        "price": "3000.00",
        "quantity": "10.00000000",
        "dealQuantity": "0.00000000",
        "status": "pending",
        "createTime": 1712230320689
      }
    ]
  },
  "params": []
}
```

**Paginated Format**:
- `num`: Total count
- `list`: Array of orders

### Deposit/Withdrawal History Response

**Endpoint**: `POST /wallet/depositHistory` or `POST /wallet/withdrawHistory`

```json
{
  "code": "0",
  "msg": "success",
  "success": true,
  "data": {
    "num": 2,
    "list": [
      {
        "id": "9876543210",
        "coin": "BTC",
        "quantity": "0.12345678",
        "status": "7",
        "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        "txId": "abc123...def456",
        "createTime": 1712230310689,
        "updateTime": 1712230320689
      }
    ]
  },
  "params": []
}
```

**Status Codes** (withdrawal):
- `0/1/2/3`: Pending
- `7`: Success
- `8`: Failed

---

## Error Response Examples

### Bithumb Korea Error

```json
{
  "status": "5300",
  "message": "Invalid API Key"
}
```

### Bithumb Pro Error

```json
{
  "code": "10005",
  "msg": "Invalid apiKey",
  "success": false,
  "data": null,
  "params": []
}
```

---

## Parsing Guidelines for Rust

### Bithumb Korea

```rust
#[derive(Debug, Deserialize)]
struct BithumbKoreaResponse<T> {
    status: String,
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
}

impl<T> BithumbKoreaResponse<T> {
    fn is_success(&self) -> bool {
        self.status == "0000"
    }
}
```

### Bithumb Pro

```rust
#[derive(Debug, Deserialize)]
struct BithumbProResponse<T> {
    code: String,
    msg: String,
    success: bool,
    data: Option<T>,
    params: Vec<serde_json::Value>,
}

impl<T> BithumbProResponse<T> {
    fn is_success(&self) -> bool {
        self.code == "0" || self.code.parse::<i32>().unwrap_or(10000) < 10000
    }
}
```

---

## Field Type Conversions

### Common Conversions Needed

| JSON Type | Bithumb Format | Rust Type |
|-----------|----------------|-----------|
| Price | String | `Decimal` or `f64` |
| Quantity | String | `Decimal` or `f64` |
| Timestamp | String or i64 | `i64` (ms) |
| Order ID | String | `String` or `u64` |
| Status | String | `enum OrderStatus` |
| Side | String | `enum Side` |

### Example Struct

```rust
use serde::{Deserialize, Deserializer};
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
struct Ticker {
    #[serde(deserialize_with = "deserialize_string_to_decimal")]
    opening_price: Decimal,

    #[serde(deserialize_with = "deserialize_string_to_decimal")]
    closing_price: Decimal,

    #[serde(deserialize_with = "deserialize_string_to_decimal")]
    units_traded: Decimal,
}

fn deserialize_string_to_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<Decimal>().map_err(serde::de::Error::custom)
}
```
