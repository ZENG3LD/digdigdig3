# Kraken API Response Formats

## General Response Structure

### Spot REST API

All Kraken Spot REST API responses follow a consistent JSON structure with two top-level keys:

```json
{
  "error": [],
  "result": {}
}
```

#### Success Response
```json
{
  "error": [],
  "result": {
    // Response data here
  }
}
```

#### Error Response
```json
{
  "error": ["EOrder:Insufficient funds"],
  "result": {}
}
```

#### Warning Response
```json
{
  "error": ["WGeneral:Temporary lockout"],
  "result": {
    // Partial data may still be returned
  }
}
```

**Key Points**:
- Successful requests or those with warnings contain both `error` and `result` keys
- Failed/rejected requests may contain only the `error` key
- HTTP status codes other than 200 indicate request didn't reach servers properly
- Check `error` array length to determine success (empty array = success)

---

### Futures REST API

Futures API uses a different response structure:

```json
{
  "result": "success",
  "serverTime": "2024-01-20T12:00:00.000Z",
  // Additional response data
}
```

**Key Points**:
- `result: "success"` indicates request was received and assessed successfully
- Does not guarantee order execution, only that request was valid
- Response includes `serverTime` timestamp

---

## HTTP Status Codes

### Spot API
- **200 OK**: Standard response (check `error` array for actual status)
- **Other codes**: Indicate infrastructure/network issues, not API logic errors

### Futures API
- **200**: Successful request
- **400**: Bad request (invalid parameters)
- **401**: Authentication failure
- **500**: Server error

---

## Error Message Format

### Structure
Error messages follow the pattern:

```
<severity><category>: <description>
```

### Severity Levels
- **E**: Error
- **W**: Warning

### Error Categories
- **General**: General errors
- **API**: API-specific errors
- **Query**: Query/request errors
- **Order**: Order-related errors
- **Trade**: Trade execution errors
- **Funding**: Funding/balance errors
- **Service**: Service availability errors

### Examples
```json
{
  "error": [
    "EGeneral:Invalid arguments",
    "EOrder:Insufficient funds",
    "EAPI:Invalid key",
    "EAPI:Invalid signature",
    "EAPI:Invalid nonce",
    "EAPI:Rate limit exceeded",
    "EService:Unavailable",
    "EService:Market in cancel_only mode"
  ]
}
```

---

## Common Error Messages

| Error Code | Description | Resolution |
|------------|-------------|------------|
| `EAPI:Invalid key` | API key not recognized | Verify API key is correct and active |
| `EAPI:Invalid signature` | Signature verification failed | Check signature generation algorithm |
| `EAPI:Invalid nonce` | Nonce validation failed | Ensure nonce is strictly increasing |
| `EAPI:Permission denied` | API key lacks permission | Grant required permission to API key |
| `EAPI:Rate limit exceeded` | Too many API calls | Reduce request frequency, check rate limits |
| `EGeneral:Invalid arguments` | Request parameters invalid | Validate parameter names and values |
| `EOrder:Insufficient funds` | Not enough balance | Check account balance |
| `EOrder:Invalid price` | Price outside acceptable range | Adjust order price |
| `EOrder:Unknown order` | Order ID not found | Verify transaction ID |
| `EService:Unavailable` | Service temporarily unavailable | Retry after delay |
| `EService:Market in cancel_only mode` | Trading halted, only cancels allowed | Wait for normal trading to resume |
| `EService:Throttled` | Concurrent request limit exceeded | Reduce parallel requests |

### Rate Limit Error Format
```json
{
  "error": ["EAPI:Rate limit exceeded"]
}
```

### Throttling Error
```json
{
  "error": ["EService: Throttled: 1234567890"]
}
```
(The number is a UNIX timestamp indicating when to retry)

---

## Response Formats by Endpoint

### Get Server Time
```json
{
  "error": [],
  "result": {
    "unixtime": 1234567890,
    "rfc1123": "Sat, 20 Jan 2024 12:00:00 +0000"
  }
}
```

---

### Get System Status
```json
{
  "error": [],
  "result": {
    "status": "online",
    "timestamp": "2024-01-20T12:00:00Z"
  }
}
```

**Status Values**:
- `online`: Fully operational
- `maintenance`: Scheduled maintenance
- `cancel_only`: Only order cancellations accepted
- `post_only`: Only post-only orders accepted

---

### Get Ticker Information
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": {
      "a": ["43210.10000", "1", "1.000"],
      "b": ["43210.00000", "2", "2.000"],
      "c": ["43210.10000", "0.00050000"],
      "v": ["1234.12345678", "2345.23456789"],
      "p": ["43150.00000", "43100.00000"],
      "t": [5000, 7500],
      "l": ["43000.00000", "42900.00000"],
      "h": ["43500.00000", "43600.00000"],
      "o": "43200.00000"
    }
  }
}
```

**Field Descriptions**:
- `a`: Ask [price, whole lot volume, lot volume]
- `b`: Bid [price, whole lot volume, lot volume]
- `c`: Last trade closed [price, lot volume]
- `v`: Volume [today, last 24 hours]
- `p`: Volume weighted average price [today, last 24 hours]
- `t`: Number of trades [today, last 24 hours]
- `l`: Low [today, last 24 hours]
- `h`: High [today, last 24 hours]
- `o`: Today's opening price

---

### Get Order Book
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": {
      "asks": [
        ["43210.50000", "5.123", 1705752000],
        ["43211.00000", "2.456", 1705752001]
      ],
      "bids": [
        ["43209.00000", "3.789", 1705752000],
        ["43208.50000", "1.234", 1705752001]
      ]
    }
  }
}
```

**Array Format**: `[price, volume, timestamp]`

---

### Get OHLC Data
```json
{
  "error": [],
  "result": {
    "XXBTZUSD": [
      [
        1705752000,
        "43100.0",
        "43250.0",
        "43090.0",
        "43200.0",
        "43150.0",
        "125.5",
        1500
      ]
    ],
    "last": 1705752000
  }
}
```

**Array Format**: `[time, open, high, low, close, vwap, volume, count]`

**Field Types**:
- `time`: integer (UNIX timestamp)
- `open`, `high`, `low`, `close`, `vwap`: string (decimal)
- `volume`: string (decimal)
- `count`: integer (number of trades)

---

### Get Balance
```json
{
  "error": [],
  "result": {
    "ZUSD": "10000.5000",
    "XXBT": "0.12345678",
    "XETH": "5.00000000",
    "USDT.F": "1000.00"
  }
}
```

**Balance Extensions**:
- `.B`: Yield-bearing product
- `.F`: Kraken Rewards (auto-earning)
- `.T`: Tokenized asset

**Note**: All balances are strings in decimal format

---

### Get Trade Balance
```json
{
  "error": [],
  "result": {
    "eb": "10000.0000",
    "tb": "9500.0000",
    "m": "50.0000",
    "n": "25.5000",
    "c": "200.0000",
    "v": "225.5000",
    "e": "9525.5000",
    "mf": "9475.5000",
    "ml": "19051.00"
  }
}
```

**Field Descriptions**:
- `eb`: Equivalent balance
- `tb`: Trade balance
- `m`: Margin amount of open positions
- `n`: Unrealized net profit/loss
- `c`: Cost basis of open positions
- `v`: Current floating valuation
- `e`: Equity = trade balance + unrealized P&L
- `mf`: Free margin = equity - initial margin
- `ml`: Margin level = (equity / initial margin) * 100

---

### Add Order (Success)
```json
{
  "error": [],
  "result": {
    "descr": {
      "order": "buy 0.1 XBTUSD @ market"
    },
    "txid": ["OUF4EM-FRGI2-MQMWZD"]
  }
}
```

**With Validation Only**:
```json
{
  "error": [],
  "result": {
    "descr": {
      "order": "buy 0.1 XBTUSD @ limit 43000.0"
    }
  }
}
```
(No `txid` when `validate=true`)

---

### Add Order (Error)
```json
{
  "error": ["EOrder:Insufficient funds"],
  "result": {}
}
```

---

### Cancel Order
```json
{
  "error": [],
  "result": {
    "count": 1
  }
}
```

**Field**: `count` indicates number of orders cancelled

---

### Query Orders Info
```json
{
  "error": [],
  "result": {
    "OUF4EM-FRGI2-MQMWZD": {
      "refid": null,
      "userref": 0,
      "status": "closed",
      "opentm": 1705752000.1234,
      "starttm": 0,
      "expiretm": 0,
      "descr": {
        "pair": "XBTUSD",
        "type": "buy",
        "ordertype": "market",
        "price": "0",
        "price2": "0",
        "leverage": "none",
        "order": "buy 0.10000000 XBTUSD @ market",
        "close": ""
      },
      "vol": "0.10000000",
      "vol_exec": "0.10000000",
      "cost": "4321.00",
      "fee": "11.23",
      "price": "43210.00",
      "stopprice": "0.00000",
      "limitprice": "0.00000",
      "misc": "",
      "oflags": "fciq",
      "trades": ["THVRQM-33VKH-UCI7BS"]
    }
  }
}
```

**Status Values**:
- `pending`: Order pending book entry
- `open`: Order open in book
- `closed`: Order closed (filled or cancelled)
- `canceled`: Order cancelled
- `expired`: Order expired

**Order Flags (`oflags`)**:
- `fcib`: Prefer fee in base currency
- `fciq`: Prefer fee in quote currency
- `nompp`: No market price protection
- `post`: Post-only order

---

### Get Open Orders
```json
{
  "error": [],
  "result": {
    "open": {
      "OUF4EM-FRGI2-MQMWZD": {
        // Same structure as Query Orders Info
      }
    }
  }
}
```

---

## Futures Response Formats

### Get Open Positions
```json
{
  "result": "success",
  "openPositions": [
    {
      "side": "long",
      "symbol": "PI_XBTUSD",
      "price": 43000.0,
      "fillTime": "2024-01-20T12:00:00.000Z",
      "size": 10000,
      "unrealizedFunding": 1.25,
      "pnl": 125.50
    }
  ],
  "serverTime": "2024-01-20T12:00:00.000Z"
}
```

---

### Get Accounts (Futures)
```json
{
  "result": "success",
  "accounts": [
    {
      "name": "USD Multi-Collateral",
      "type": "multiCollateralMarginAccount",
      "balances": {
        "USD": 10000.50,
        "BTC": 0.5
      },
      "marginRequirements": {
        "im": 500.0,
        "mm": 250.0,
        "lt": 100.0,
        "tt": 50.0
      },
      "triggerEstimates": {
        "im": 43500.0,
        "mm": 43000.0,
        "lt": 42500.0,
        "tt": 42000.0
      },
      "availableFunds": 9500.0,
      "pv": 10125.50
    }
  ],
  "serverTime": "2024-01-20T12:00:00.000Z"
}
```

**Margin Fields**:
- `im`: Initial margin
- `mm`: Maintenance margin
- `lt`: Liquidation threshold
- `tt`: Termination threshold
- `pv`: Portfolio value

---

### Historical Funding Rates
```json
{
  "rates": [
    {
      "timestamp": "2024-01-20T12:00:00.000Z",
      "fundingRate": 0.0001,
      "relativeFundingRate": 0.01
    }
  ],
  "serverTime": "2024-01-20T12:00:00.000Z"
}
```

---

### Send Order (Futures)
```json
{
  "result": "success",
  "sendStatus": {
    "order_id": "e35d61dd-8a30-4d5f-a574-b5593ef0c050",
    "status": "placed",
    "receivedTime": "2024-01-20T12:00:00.000Z",
    "orderEvents": [
      {
        "order": {
          "orderId": "e35d61dd-8a30-4d5f-a574-b5593ef0c050",
          "cliOrdId": "my-order-123",
          "type": "lmt",
          "symbol": "PI_XBTUSD",
          "side": "buy",
          "quantity": 1000,
          "filled": 0,
          "limitPrice": 43000.0,
          "timestamp": "2024-01-20T12:00:00.000Z"
        }
      }
    ]
  },
  "serverTime": "2024-01-20T12:00:00.000Z"
}
```

---

## WebSocket Response Formats

### Subscription Acknowledgment (v2)
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "snapshot": true,
    "symbol": "BTC/USD"
  },
  "success": true,
  "time_in": "2024-01-20T12:00:00.000000Z",
  "time_out": "2024-01-20T12:00:00.100000Z"
}
```

---

### Ticker Update (Spot v2)
```json
{
  "channel": "ticker",
  "type": "update",
  "data": [
    {
      "symbol": "BTC/USD",
      "bid": 43210.0,
      "bid_qty": 5.123,
      "ask": 43210.5,
      "ask_qty": 3.456,
      "last": 43210.1,
      "volume": 1234.567,
      "vwap": 43150.5,
      "low": 43000.0,
      "high": 43500.0,
      "change": 210.0,
      "change_pct": 0.49
    }
  ]
}
```

---

### Book Update (Spot v2)
```json
{
  "channel": "book",
  "type": "snapshot",
  "data": [
    {
      "symbol": "BTC/USD",
      "bids": [
        {"price": 43210.0, "qty": 5.123},
        {"price": 43209.5, "qty": 2.456}
      ],
      "asks": [
        {"price": 43210.5, "qty": 3.789},
        {"price": 43211.0, "qty": 1.234}
      ],
      "checksum": 123456789,
      "timestamp": "2024-01-20T12:00:00.000000Z"
    }
  ]
}
```

---

### Futures WebSocket Ticker
```json
{
  "feed": "ticker",
  "product_id": "PI_XBTUSD",
  "bid": 43210.0,
  "ask": 43210.5,
  "bid_size": 5000,
  "ask_size": 3000,
  "last": 43210.0,
  "volume": 1234567,
  "change": 210.0,
  "funding_rate": 0.0001,
  "time": 1705752000000,
  "seq": 12345
}
```

---

### Futures WebSocket Book Snapshot
```json
{
  "feed": "book_snapshot",
  "product_id": "PI_XBTUSD",
  "bids": [
    {"price": 43210.0, "qty": 5000},
    {"price": 43209.5, "qty": 3000}
  ],
  "asks": [
    {"price": 43210.5, "qty": 4000},
    {"price": 43211.0, "qty": 2000}
  ],
  "timestamp": 1705752000000,
  "seq": 12345
}
```

---

### Futures WebSocket Book Update
```json
{
  "feed": "book",
  "product_id": "PI_XBTUSD",
  "side": "buy",
  "price": 43210.0,
  "qty": 5500,
  "timestamp": 1705752001000,
  "seq": 12346
}
```

**Note**: `qty: 0` indicates price level removal

---

## Data Types Summary

| Field Type | Format | Example |
|------------|--------|---------|
| Price/Amount | String (decimal) | `"43210.50000"` |
| Volume | String (decimal) | `"125.12345678"` |
| Timestamp (REST) | Integer (UNIX seconds) or Float | `1705752000` or `1705752000.1234` |
| Timestamp (WebSocket) | Integer (milliseconds) | `1705752000000` |
| Timestamp (ISO) | String (RFC3339) | `"2024-01-20T12:00:00.000Z"` |
| Order ID | String (UUID or custom) | `"OUF4EM-FRGI2-MQMWZD"` |
| Boolean | Boolean | `true`, `false` |
| Count | Integer | `1500` |

---

## Summary

- **Spot REST**: Always check `error` array for success/failure
- **Futures REST**: Check `result: "success"` field
- **Decimals**: Prices and amounts are strings to preserve precision
- **Timestamps**: Multiple formats depending on API version
- **Errors**: Structured with severity and category prefixes
- **WebSocket**: Real-time updates with sequence numbers for ordering
