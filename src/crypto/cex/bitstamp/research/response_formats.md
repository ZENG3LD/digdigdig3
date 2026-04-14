# Bitstamp API Response Formats

This document details the JSON response structures for all Bitstamp API endpoints.

---

## Response Structure

All Bitstamp API responses return JSON. Successful responses contain the data directly, while error responses follow a specific error format.

### Success Response

Direct JSON data without wrapper:
```json
{
  "field1": "value1",
  "field2": "value2"
}
```

or array:
```json
[
  { "field1": "value1" },
  { "field2": "value2" }
]
```

### Error Response

```json
{
  "status": "error",
  "reason": "Error description",
  "code": "API0007"
}
```

---

## Market Data Responses

### Ticker Response

**Endpoint**: `GET /api/v2/ticker/{pair}/`

```json
{
  "last": "2211.00",
  "high": "2811.00",
  "low": "2188.97",
  "vwap": "2189.80",
  "volume": "213.26801100",
  "bid": "2188.97",
  "ask": "2211.00",
  "timestamp": "1643640186",
  "open": "2211.00",
  "open_24": "2211.00",
  "percent_change_24": "13.57",
  "side": "0",
  "market_type": "spot",
  "mark_price": "2211.00",
  "index_price": "2210.50",
  "open_interest": "0",
  "open_interest_value": "0",
  "pair": "BTC/USD",
  "market": "btcusd"
}
```

**Field Types**:
- `last`: String (decimal number)
- `high`: String (decimal number)
- `low`: String (decimal number)
- `vwap`: String (volume weighted average price)
- `volume`: String (decimal number)
- `bid`: String (decimal number)
- `ask`: String (decimal number)
- `timestamp`: String (Unix timestamp in seconds)
- `open`: String (opening price)
- `open_24`: String (price 24h ago)
- `percent_change_24`: String (percentage change)
- `side`: String (numeric, market side indicator)
- `market_type`: String ("spot", "perpetual", etc.)
- `pair`: String (formatted pair "BTC/USD")
- `market`: String (market symbol "btcusd")

---

### Order Book Response

**Endpoint**: `GET /api/v2/order_book/{pair}/`

```json
{
  "timestamp": "1643643584",
  "microtimestamp": "1643643584684047",
  "bids": [
    ["9484.34", "1.00000000"],
    ["9483.00", "0.50000000"],
    ["9482.50", "2.00000000"]
  ],
  "asks": [
    ["9485.00", "1.00000000"],
    ["9486.50", "0.75000000"],
    ["9487.00", "1.50000000"]
  ]
}
```

**Field Types**:
- `timestamp`: String (Unix timestamp in seconds)
- `microtimestamp`: String (Unix timestamp in microseconds)
- `bids`: Array of [price, amount] arrays (both strings)
- `asks`: Array of [price, amount] arrays (both strings)

**Bid/Ask Entry**:
- `[0]`: Price (string, decimal)
- `[1]`: Amount (string, decimal)

---

### Transactions (Trades) Response

**Endpoint**: `GET /api/v2/transactions/{pair}/`

```json
[
  {
    "date": "1643643584",
    "tid": "21565524",
    "price": "212.80",
    "amount": "0.01513062",
    "type": "0"
  },
  {
    "date": "1643643590",
    "tid": "21565525",
    "price": "212.85",
    "amount": "0.02000000",
    "type": "1"
  }
]
```

**Field Types**:
- `date`: String (Unix timestamp in seconds)
- `tid`: String (trade ID)
- `price`: String (decimal number)
- `amount`: String (decimal number)
- `type`: String ("0" = buy, "1" = sell)

---

### OHLC (Candlestick) Response

**Endpoint**: `GET /api/v2/ohlc/{pair}/`

```json
{
  "data": {
    "ohlc": [
      {
        "timestamp": "1505558814",
        "open": "212.80",
        "high": "213.50",
        "low": "212.00",
        "close": "213.20",
        "volume": "1.23456789"
      },
      {
        "timestamp": "1505558874",
        "open": "213.20",
        "high": "214.00",
        "low": "213.00",
        "close": "213.80",
        "volume": "2.34567890"
      }
    ],
    "pair": "BTC/USD"
  }
}
```

**Field Types**:
- `data.ohlc`: Array of candle objects
- `timestamp`: String (Unix timestamp)
- `open`: String (decimal number)
- `high`: String (decimal number)
- `low`: String (decimal number)
- `close`: String (decimal number)
- `volume`: String (decimal number)
- `pair`: String (formatted pair)

---

### Trading Pairs Info Response

**Endpoint**: `GET /api/v2/markets/`

```json
[
  {
    "trading": "Enabled",
    "base_decimals": 8,
    "counter_decimals": 2,
    "instant_order_counter_decimals": 2,
    "minimum_order": "10.0 USD",
    "market_symbol": "btcusd",
    "base_currency": "BTC",
    "counter_currency": "USD",
    "url_symbol": "btcusd",
    "description": "Bitcoin / U.S. dollar"
  }
]
```

**Field Types**:
- `trading`: String ("Enabled" or "Disabled")
- `base_decimals`: Integer (decimal precision for base)
- `counter_decimals`: Integer (decimal precision for quote)
- `instant_order_counter_decimals`: Integer
- `minimum_order`: String (formatted minimum)
- `market_symbol`: String (lowercase pair)
- `base_currency`: String (base currency code)
- `counter_currency`: String (quote currency code)
- `url_symbol`: String (URL-safe symbol)
- `description`: String (human-readable description)

---

### Currencies Response

**Endpoint**: `GET /api/v2/currencies/`

```json
[
  {
    "code": "BTC",
    "name": "Bitcoin",
    "decimals": 8,
    "networks": ["bitcoin"],
    "minimum_withdrawal": "0.001",
    "minimum_deposit": "0.0001",
    "deposit_enabled": true,
    "withdrawal_enabled": true
  }
]
```

**Field Types**:
- `code`: String (currency code)
- `name`: String (full name)
- `decimals`: Integer (decimal precision)
- `networks`: Array of strings (supported blockchain networks)
- `minimum_withdrawal`: String (decimal number)
- `minimum_deposit`: String (decimal number)
- `deposit_enabled`: Boolean
- `withdrawal_enabled`: Boolean

---

## Trading Responses

### Order Creation Response

**Endpoints**:
- `POST /api/v2/buy/{pair}/`
- `POST /api/v2/sell/{pair}/`
- `POST /api/v2/buy/market/{pair}/`
- `POST /api/v2/sell/market/{pair}/`

```json
{
  "id": "2344851866",
  "datetime": "2018-11-05 16:16:39.532897",
  "type": "0",
  "price": "0.45701",
  "amount": "205.33880000"
}
```

**Field Types**:
- `id`: String (order ID)
- `datetime`: String (ISO-like datetime)
- `type`: String ("0" = buy, "1" = sell)
- `price`: String (decimal number)
- `amount`: String (decimal number)

---

### Order Status Response

**Endpoint**: `POST /api/v2/order_status/`

```json
{
  "status": "Open",
  "id": "2344851866",
  "amount_remaining": "1.00000000",
  "transactions": []
}
```

**For Filled/Partially Filled Orders**:
```json
{
  "status": "Finished",
  "id": "2344851866",
  "amount_remaining": "0.00000000",
  "transactions": [
    {
      "tid": "21565524",
      "usd": "212.80",
      "price": "212.80",
      "fee": "0.50",
      "btc": "0.01513062",
      "datetime": "2018-11-05 16:16:39"
    }
  ]
}
```

**Field Types**:
- `status`: String ("Open", "Finished", "Canceled")
- `id`: String (order ID)
- `amount_remaining`: String (decimal number)
- `transactions`: Array of transaction objects

**Transaction Object**:
- `tid`: String (trade ID)
- `usd` / currency code: String (amount in quote currency)
- `price`: String (execution price)
- `fee`: String (fee amount)
- `btc` / currency code: String (amount in base currency)
- `datetime`: String (execution time)

---

### Cancel Order Response

**Endpoint**: `POST /api/v2/cancel_order/`

```json
{
  "id": "2344851866",
  "amount": "1.00000000",
  "price": "1000.00",
  "type": "0",
  "status": "Canceled"
}
```

**Field Types**:
- `id`: String (order ID)
- `amount`: String (order amount)
- `price`: String (order price)
- `type`: String ("0" = buy, "1" = sell)
- `status`: String ("Canceled")

---

### Open Orders Response

**Endpoint**: `POST /api/v2/open_orders/all/`

```json
[
  {
    "id": "12345",
    "datetime": "2018-11-05 16:16:39",
    "type": "0",
    "price": "9484.34",
    "amount": "1.00000000",
    "currency_pair": "BTC/USD",
    "market": "btcusd"
  },
  {
    "id": "12346",
    "datetime": "2018-11-05 16:20:15",
    "type": "1",
    "price": "9490.00",
    "amount": "0.50000000",
    "currency_pair": "BTC/USD",
    "market": "btcusd"
  }
]
```

**Field Types**:
- `id`: String (order ID)
- `datetime`: String (ISO-like datetime)
- `type`: String ("0" = buy, "1" = sell)
- `price`: String (decimal number)
- `amount`: String (decimal number)
- `currency_pair`: String (formatted pair)
- `market`: String (market symbol)

---

## Account Responses

### Account Balances Response

**Endpoint**: `POST /api/v2/account_balances/`

```json
[
  {
    "currency": "usd",
    "total": "100.00",
    "available": "90.00",
    "reserved": "10.00"
  },
  {
    "currency": "btc",
    "total": "0.50000000",
    "available": "0.45000000",
    "reserved": "0.05000000"
  },
  {
    "currency": "eth",
    "total": "5.00000000",
    "available": "5.00000000",
    "reserved": "0.00000000"
  }
]
```

**Field Types**:
- `currency`: String (lowercase currency code)
- `total`: String (total balance)
- `available`: String (available for trading)
- `reserved`: String (reserved in orders)

**Formula**: `total = available + reserved`

---

### Balance Response (Legacy)

**Endpoint**: `POST /api/v2/balance/`

```json
{
  "usd_balance": "100.00",
  "usd_available": "90.00",
  "usd_reserved": "10.00",
  "btc_balance": "0.50000000",
  "btc_available": "0.45000000",
  "btc_reserved": "0.05000000",
  "eth_balance": "5.00000000",
  "eth_available": "5.00000000",
  "eth_reserved": "0.00000000",
  "fee": "0.25"
}
```

**Field Pattern**:
- `{currency}_balance`: String (total)
- `{currency}_available`: String (available)
- `{currency}_reserved`: String (reserved)
- `fee`: String (trading fee percentage)

---

### User Transactions Response

**Endpoint**: `POST /api/v2/user_transactions/`

```json
[
  {
    "datetime": "2018-11-05 16:16:39",
    "id": "123456789",
    "type": "2",
    "usd": "-100.00",
    "btc": "0.01000000",
    "btc_usd": "10000.00",
    "fee": "0.25",
    "order_id": "2344851866"
  },
  {
    "datetime": "2018-11-05 15:30:22",
    "id": "123456788",
    "type": "0",
    "usd": "1000.00",
    "btc": "0.00000000",
    "fee": "0.00",
    "order_id": "0"
  }
]
```

**Field Types**:
- `datetime`: String (ISO-like datetime)
- `id`: String (transaction ID)
- `type`: String (see transaction types below)
- Currency fields: String (decimal, negative for outgoing)
- `{base}_{quote}`: String (price, e.g., "btc_usd")
- `fee`: String (fee amount in quote currency)
- `order_id`: String (related order ID, "0" if none)

**Transaction Types**:
- `"0"`: Deposit
- `"1"`: Withdrawal
- `"2"`: Market trade
- `"14"`: Sub-account transfer

---

### Trading Fees Response

**Endpoint**: `POST /api/v2/fees/trading/`

```json
{
  "btcusd": {
    "maker": "0.25",
    "taker": "0.25"
  },
  "btceur": {
    "maker": "0.25",
    "taker": "0.25"
  }
}
```

**Field Types**:
- Market symbol as key
- `maker`: String (maker fee percentage)
- `taker`: String (taker fee percentage)

---

### Withdrawal Fees Response

**Endpoint**: `POST /api/v2/fees/withdrawal/`

```json
{
  "btc": "0.0005",
  "eth": "0.01",
  "usd": "25.00"
}
```

**Field Types**:
- Currency code as key
- Value: String (withdrawal fee amount)

---

### Crypto Deposit Address Response

**Endpoint**: `POST /api/v2/{coin}_address/`

**Bitcoin Example**:
```json
{
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
}
```

**Ripple/XRP Example**:
```json
{
  "address": "rDsbeomae4FXwgQTJp9Rs64Qg9vDiTCdBv",
  "destination_tag": "123456789"
}
```

**Field Types**:
- `address`: String (deposit address)
- `destination_tag`: String (for currencies that require it)

---

### Withdrawal Response

**Endpoint**: `POST /api/v2/{coin}_withdrawal/`

```json
{
  "id": "987654321",
  "status": "pending"
}
```

**Field Types**:
- `id`: String (withdrawal ID)
- `status`: String (withdrawal status)

---

### Withdrawal Requests Response

**Endpoint**: `POST /api/v2/withdrawal-requests/`

```json
[
  {
    "id": "987654321",
    "datetime": "2018-11-05 16:16:39",
    "type": "1",
    "amount": "0.01000000",
    "status": "2",
    "data": {
      "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
    }
  }
]
```

**Field Types**:
- `id`: String (withdrawal ID)
- `datetime`: String (request time)
- `type`: String (withdrawal type)
- `amount`: String (withdrawal amount)
- `status`: String (status code)
- `data`: Object (withdrawal details)

**Status Codes**:
- `"0"`: Open
- `"1"`: In process
- `"2"`: Finished
- `"3"`: Canceled
- `"4"`: Failed

---

## WebSocket Message Formats

### Subscription Request

```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_trades_btcusd"
  }
}
```

### Subscription Confirmation

```json
{
  "event": "bts:subscription_succeeded",
  "channel": "live_trades_btcusd",
  "data": {}
}
```

---

### Live Trade Message

**Channel**: `live_trades_{pair}`

```json
{
  "data": {
    "amount": 0.01513062,
    "buy_order_id": 297260696,
    "sell_order_id": 297260910,
    "amount_str": "0.01513062",
    "price_str": "212.80",
    "timestamp": "1505558814",
    "price": 212.8,
    "type": 1,
    "id": 21565524,
    "cost": 3.219795936
  },
  "channel": "live_trades_btcusd",
  "event": "trade"
}
```

**Field Types**:
- `amount`: Number
- `buy_order_id`: Number
- `sell_order_id`: Number
- `amount_str`: String
- `price_str`: String
- `timestamp`: String (Unix timestamp)
- `price`: Number
- `type`: Number (0 = buy, 1 = sell)
- `id`: Number (trade ID)
- `cost`: Number (amount * price)

---

### Order Book Snapshot

**Channel**: `order_book_{pair}`

```json
{
  "data": {
    "timestamp": "1643643584",
    "microtimestamp": "1643643584684047",
    "bids": [
      ["3284.06000000", "0.16927410"],
      ["3284.05000000", "1.00000000"]
    ],
    "asks": [
      ["3289.00000000", "3.16123001"],
      ["3291.99000000", "0.22000000"]
    ]
  },
  "channel": "order_book_btcusd",
  "event": "data"
}
```

**Field Types**:
- `timestamp`: String
- `microtimestamp`: String
- `bids`: Array of [price, amount] string arrays
- `asks`: Array of [price, amount] string arrays

---

### Differential Order Book Update

**Channel**: `diff_order_book_{pair}`

```json
{
  "data": {
    "timestamp": "1643643584",
    "microtimestamp": "1643643584684047",
    "bids": [
      ["3284.06000000", "0.16927410"]
    ],
    "asks": [
      ["3289.00000000", "0.00000000"]
    ]
  },
  "channel": "diff_order_book_btcusd",
  "event": "data"
}
```

**Note**: Amount "0.00000000" means the price level was removed.

---

## Data Type Notes

### Decimal Numbers

All decimal numbers (prices, amounts, balances) are returned as **strings** to preserve precision. Always parse as decimal/float in your code.

### Timestamps

- **Unix timestamp**: String or number, seconds since epoch
- **Microtimestamp**: String, microseconds since epoch
- **Datetime**: String, format: `"YYYY-MM-DD HH:MM:SS"` or `"YYYY-MM-DD HH:MM:SS.ffffff"`

### Order Types

- `"0"`: Buy
- `"1"`: Sell

### Currencies

Currency codes are lowercase in most responses (`"btc"`, `"usd"`, `"eth"`), but uppercase in some fields like `base_currency` and `counter_currency`.

### Empty Arrays

Endpoints that return arrays will return `[]` if no data is available (e.g., no open orders).

---

## Error Codes

Common error codes:

| Code | Reason |
|------|--------|
| `API0001` | Invalid API key |
| `API0002` | Invalid signature |
| `API0003` | Invalid nonce |
| `API0005` | Permission denied |
| `API0007` | Invalid signature |
| `API0011` | Timestamp too far from current time |
| `API0012` | Nonce must be unique |
| `API0013` | Missing parameter |
| `API0020` | Order not found |
| `400.002` | Rate limit exceeded |

---

## Summary

All Bitstamp responses use JSON with string-based decimal numbers for precision. The API is consistent in structure but varies between v1 legacy format (currency-specific fields) and v2 format (array of objects). Always use v2 endpoints for new implementations.
