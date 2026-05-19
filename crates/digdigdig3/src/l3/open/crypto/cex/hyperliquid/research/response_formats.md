# HyperLiquid Response Formats

All HyperLiquid API responses use JSON format with consistent structure patterns.

---

## General Response Structure

### Success Response (Info Endpoints)
```json
{
  // Direct data response, no wrapper
}
```

### Success Response (Exchange Endpoints)
```json
{
  "status": "ok",
  "response": {
    "type": "order" | "cancel" | "modify" | ...,
    "data": {
      // Action-specific response
    }
  }
}
```

### Error Response
```json
{
  "status": "error",
  "error": "Error message description"
}
```

---

## Market Data Responses

### Exchange Info / Meta

#### Perpetuals Meta
```json
{
  "universe": [
    {
      "name": "BTC",
      "szDecimals": 5,
      "maxLeverage": 50,
      "onlyIsolated": false
    },
    {
      "name": "ETH",
      "szDecimals": 4,
      "maxLeverage": 50,
      "onlyIsolated": false
    }
  ]
}
```

**Field Descriptions**:
- `name`: Coin symbol (e.g., "BTC", "ETH")
- `szDecimals`: Decimal places for size (5 = 0.00001 precision)
- `maxLeverage`: Maximum leverage allowed (e.g., 50x)
- `onlyIsolated`: If true, only isolated margin allowed

#### Spot Meta
```json
{
  "universe": [
    {
      "tokens": [150, 0],
      "name": "HYPE/USDC",
      "index": 107,
      "isCanonical": true
    }
  ],
  "tokens": [
    {
      "name": "USDC",
      "szDecimals": 8,
      "weiDecimals": 6,
      "index": 0,
      "tokenId": "0x6d1e7cde53ba9467b783cb7c530ce054",
      "isCanonical": true,
      "evmContract": null,
      "fullName": null
    },
    {
      "name": "HYPE",
      "szDecimals": 8,
      "weiDecimals": 18,
      "index": 150,
      "tokenId": "0x...",
      "isCanonical": true,
      "evmContract": "0x...",
      "fullName": "Hyperliquid"
    }
  ]
}
```

**Field Descriptions**:
- `tokens`: Array of token indices [base, quote]
- `index`: Spot pair index (use as `@{index}`)
- `szDecimals`: Size decimal precision
- `weiDecimals`: On-chain decimal precision
- `tokenId`: Unique token identifier
- `isCanonical`: Official/verified token
- `evmContract`: EVM contract address if applicable

---

### Meta and Asset Contexts

```json
{
  "universe": [...],  // Same as meta response
  "assetCtxs": [
    {
      "dayNtlVlm": "1234567890.5",
      "funding": "0.000012345",
      "openInterest": "987654.321",
      "prevDayPx": "50000.0",
      "markPx": "50123.45",
      "midPx": "50123.5",
      "impactPxs": ["50120.0", "50127.0"],
      "premium": "0.5",
      "oraclePx": "50122.95"
    }
  ]
}
```

**Field Descriptions**:
- `dayNtlVlm`: 24-hour notional volume in USD
- `funding`: Current funding rate (hourly)
- `openInterest`: Total open interest in base asset
- `prevDayPx`: Price 24 hours ago
- `markPx`: Mark price (for PnL calculation)
- `midPx`: Mid price (bid + ask) / 2
- `impactPxs`: [buy impact price, sell impact price]
- `premium`: Premium component of funding
- `oraclePx`: Oracle price reference

---

### All Mids (Ticker Prices)

```json
{
  "BTC": "50123.45",
  "ETH": "2500.67",
  "SOL": "100.234",
  "PURR/USDC": "0.000123"
}
```

**Format**: `{ "SYMBOL": "price_string" }`

---

### Order Book (L2)

```json
{
  "coin": "BTC",
  "time": 1704067200000,
  "levels": [
    [
      {"px": "50123.5", "sz": "1.234", "n": 3},
      {"px": "50123.0", "sz": "2.567", "n": 5},
      {"px": "50122.5", "sz": "0.891", "n": 2}
    ],
    [
      {"px": "50124.0", "sz": "0.567", "n": 1},
      {"px": "50124.5", "sz": "3.456", "n": 7},
      {"px": "50125.0", "sz": "1.234", "n": 4}
    ]
  ]
}
```

**Structure**:
- `levels[0]`: Bids (descending price)
- `levels[1]`: Asks (ascending price)
- `px`: Price level
- `sz`: Total size at level
- `n`: Number of orders at level

**Note**: Up to 20 levels per side

---

### Recent Trades

```json
[
  {
    "coin": "BTC",
    "side": "B",
    "px": "50123.45",
    "sz": "0.5",
    "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "time": 1704067200123,
    "tid": 123456789,
    "fee": "0.25"
  },
  {
    "coin": "BTC",
    "side": "A",
    "px": "50122.0",
    "sz": "1.2",
    "hash": "0x...",
    "time": 1704067199456,
    "tid": 123456788,
    "fee": "0.60"
  }
]
```

**Field Descriptions**:
- `side`: "B" = buy/long, "A" = sell/short
- `px`: Trade price
- `sz`: Trade size
- `hash`: Transaction hash
- `time`: Unix timestamp (milliseconds)
- `tid`: Trade ID
- `fee`: Fee amount (USD)

---

### Candles / Klines

```json
[
  {
    "t": 1704067200000,
    "T": 1704067259999,
    "s": "BTC",
    "i": "15m",
    "o": "50100.0",
    "c": "50200.0",
    "h": "50250.0",
    "l": "50050.0",
    "v": "123.456",
    "n": 1234
  }
]
```

**Field Descriptions**:
- `t`: Candle open time (ms)
- `T`: Candle close time (ms)
- `s`: Symbol
- `i`: Interval (e.g., "15m", "1h")
- `o`: Open price
- `c`: Close price
- `h`: High price
- `l`: Low price
- `v`: Volume (base asset)
- `n`: Number of trades

**Intervals**: "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M"

---

### Funding Rate History

```json
[
  {
    "coin": "BTC",
    "fundingRate": "0.00001234",
    "premium": "0.5",
    "time": 1704067200000
  }
]
```

**Field Descriptions**:
- `fundingRate`: Funding rate applied
- `premium`: Premium component
- `time`: Funding timestamp

**Note**: Funding occurs every hour

---

## Trading Responses

### Place Order - Success

```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "resting": {
            "oid": 123456789,
            "cloid": "0x1234567890abcdef1234567890abcdef"
          }
        }
      ]
    }
  }
}
```

**Resting Order**: Order placed on book

### Place Order - Filled

```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "filled": {
            "totalSz": "0.1",
            "avgPx": "50123.45",
            "oid": 123456790,
            "cloid": null
          }
        }
      ]
    }
  }
}
```

**Field Descriptions**:
- `totalSz`: Total size filled
- `avgPx`: Average fill price
- `oid`: Order ID
- `cloid`: Client order ID (if provided)

### Place Order - Error

```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "error": "Insufficient margin to place order."
        }
      ]
    }
  }
}
```

### Place Order - Batch Response

```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {"resting": {"oid": 123456789}},
        {"filled": {"totalSz": "0.5", "avgPx": "50100.0", "oid": 123456790}},
        {"error": "Price must be divisible by tick size."}
      ]
    }
  }
}
```

**Note**: Each status corresponds to order in request array (same index)

---

### Cancel Order - Success

```json
{
  "status": "ok",
  "response": {
    "type": "cancel",
    "data": {
      "statuses": [
        "success"
      ]
    }
  }
}
```

### Cancel Order - Error

```json
{
  "status": "ok",
  "response": {
    "type": "cancel",
    "data": {
      "statuses": [
        {
          "error": "Order was never placed, already canceled, or filled."
        }
      ]
    }
  }
}
```

---

### Modify Order - Success

```json
{
  "status": "ok",
  "response": {
    "type": "modify",
    "data": {
      "statuses": [
        {
          "resting": {
            "oid": 123456789
          }
        }
      ]
    }
  }
}
```

---

## Account Responses

### Clearinghouse State (Perpetuals)

```json
{
  "assetPositions": [
    {
      "position": {
        "coin": "BTC",
        "szi": "1.5",
        "leverage": {
          "type": "cross",
          "value": 5
        },
        "entryPx": "49500.0",
        "positionValue": "74250.0",
        "unrealizedPnl": "750.0",
        "returnOnEquity": "0.015",
        "liquidationPx": "40000.0",
        "marginUsed": "14850.0",
        "maxTradeSzs": ["10.0", "10.0"],
        "cumFunding": {
          "allTime": "12.34",
          "sinceChange": "5.67",
          "sinceOpen": "8.90"
        }
      },
      "type": "oneWay"
    }
  ],
  "crossMarginSummary": {
    "accountValue": "100000.0",
    "totalNtlPos": "74250.0",
    "totalRawUsd": "25750.0",
    "totalMarginUsed": "14850.0",
    "withdrawable": "10900.0"
  },
  "marginSummary": {
    "accountValue": "100000.0",
    "totalNtlPos": "74250.0",
    "totalRawUsd": "25750.0"
  },
  "time": 1704067200000
}
```

**Position Field Descriptions**:
- `szi`: Signed size (positive = long, negative = short)
- `leverage.type`: "cross" or "isolated"
- `leverage.value`: Leverage multiplier
- `entryPx`: Average entry price
- `positionValue`: Current position value (abs(szi) * markPx)
- `unrealizedPnl`: Unrealized profit/loss
- `returnOnEquity`: ROE percentage
- `liquidationPx`: Liquidation price
- `marginUsed`: Margin allocated to position
- `maxTradeSzs`: [max long size, max short size]
- `cumFunding`: Cumulative funding payments

**Account Summary Fields**:
- `accountValue`: Total account equity
- `totalNtlPos`: Total position notional value
- `totalRawUsd`: Free collateral (USDC)
- `totalMarginUsed`: Total margin used
- `withdrawable`: Amount available to withdraw

---

### Spot Clearinghouse State

```json
{
  "balances": [
    {
      "coin": "USDC",
      "hold": "1000.0",
      "total": "10000.0",
      "entryNtl": "10000.0",
      "token": 0
    },
    {
      "coin": "HYPE",
      "hold": "50.0",
      "total": "1000.0",
      "entryNtl": "5000.0",
      "token": 150
    }
  ]
}
```

**Field Descriptions**:
- `coin`: Token symbol
- `hold`: Amount locked in orders
- `total`: Total balance (available + hold)
- `entryNtl`: Entry notional value (USD)
- `token`: Token index

**Available Balance**: `total - hold`

---

### Portfolio

```json
{
  "marginSummary": {
    "accountValue": "100000.0",
    "totalNtlPos": "74250.0",
    "totalRawUsd": "25750.0"
  },
  "perpTimes": {
    "allTime": {
      "pnl": "12345.67",
      "vlm": "9876543.21"
    },
    "day": {
      "pnl": "123.45",
      "vlm": "54321.0"
    },
    "week": {
      "pnl": "567.89",
      "vlm": "234567.0"
    }
  },
  "spotTimes": {
    "allTime": {
      "pnl": "1234.56",
      "vlm": "123456.78"
    },
    "day": {
      "pnl": "12.34",
      "vlm": "5432.1"
    }
  }
}
```

**Time Periods**: `allTime`, `day`, `week` (if available)
**Fields per period**:
- `pnl`: Profit and loss
- `vlm`: Trading volume

---

### Open Orders

```json
[
  {
    "coin": "BTC",
    "limitPx": "50000.0",
    "oid": 123456789,
    "side": "B",
    "sz": "0.1",
    "timestamp": 1704067200000,
    "origSz": "0.15",
    "cloid": "0x1234567890abcdef1234567890abcdef"
  }
]
```

**Field Descriptions**:
- `limitPx`: Limit price
- `oid`: Order ID
- `side`: "B" = buy, "A" = sell
- `sz`: Current remaining size
- `timestamp`: Order placement time
- `origSz`: Original order size
- `cloid`: Client order ID (null if not provided)

---

### Order Status

```json
{
  "order": {
    "coin": "BTC",
    "side": "B",
    "limitPx": "50000.0",
    "sz": "0.1",
    "oid": 123456789,
    "timestamp": 1704067200000,
    "origSz": "0.1",
    "cloid": null,
    "orderType": {
      "limit": {
        "tif": "Gtc"
      }
    }
  },
  "status": "open",
  "statusTimestamp": 1704067200000
}
```

**Status Values**:
- `"open"`: Active on order book
- `"filled"`: Completely filled
- `"canceled"`: User canceled
- `"triggered"`: Trigger order activated
- `"rejected"`: Order rejected
- `"marginCanceled"`: Canceled due to insufficient margin
- `"partiallyFilled"`: Partially filled
- `"expired"`: Expired (if using expiresAfter)

---

### User Fills (Trade History)

```json
[
  {
    "coin": "BTC",
    "px": "50100.0",
    "sz": "0.1",
    "side": "B",
    "time": 1704067200123,
    "startPosition": "0.5",
    "dir": "Open Long",
    "closedPnl": "0.0",
    "hash": "0x...",
    "oid": 123456789,
    "crossed": true,
    "fee": "2.505",
    "feeToken": "USDC",
    "tid": 987654321,
    "builderFee": "0.0",
    "cloid": null
  }
]
```

**Field Descriptions**:
- `dir`: Trade direction (e.g., "Open Long", "Close Short", "Increase Short")
- `startPosition`: Position size before trade
- `closedPnl`: Realized PnL from this fill
- `crossed`: true if taker, false if maker
- `fee`: Fee paid
- `feeToken`: Token used for fee payment
- `builderFee`: Additional builder fee (if applicable)

---

### Historical Orders

```json
[
  {
    "order": {
      "coin": "BTC",
      "side": "B",
      "limitPx": "50000.0",
      "sz": "0.0",
      "oid": 123456789,
      "timestamp": 1704067200000,
      "origSz": "0.1",
      "cloid": null,
      "orderType": {
        "limit": {
          "tif": "Gtc"
        }
      }
    },
    "status": "filled",
    "statusTimestamp": 1704067201000
  }
]
```

**Note**: `sz: "0.0"` indicates fully filled order

---

### User Fees

```json
{
  "dailyUserVlm": [
    {
      "time": 1704067200000,
      "vlm": "123456.78"
    }
  ],
  "feeSchedule": {
    "tiers": [
      {
        "vlm": "0",
        "maker": "0.00020",
        "taker": "0.00035"
      },
      {
        "vlm": "1000000",
        "maker": "0.00015",
        "taker": "0.00030"
      }
    ]
  },
  "activeDiscounts": [],
  "staking": null
}
```

**Field Descriptions**:
- `dailyUserVlm`: Daily volume history
- `tiers`: Fee tiers based on volume
- `vlm`: Volume threshold (USD)
- `maker`: Maker fee rate
- `taker`: Taker fee rate
- `activeDiscounts`: Any active fee discounts
- `staking`: Staking-related fee benefits

---

### Rate Limit Status

```json
{
  "cumVlm": "12345678.90",
  "nRequestsUsed": 5432,
  "nRequestsCap": 12345678,
  "nRequestsSurplus": 10000
}
```

**Field Descriptions**:
- `cumVlm`: Cumulative volume traded (all-time)
- `nRequestsUsed`: Requests consumed
- `nRequestsCap`: Volume-based request allowance (1 per $1 traded)
- `nRequestsSurplus`: Initial 10K buffer remaining

**Remaining**: `nRequestsCap + nRequestsSurplus - nRequestsUsed`

---

### Subaccounts

```json
[
  "0xabcdef1234567890abcdef1234567890abcdef12",
  "0x1234567890abcdef1234567890abcdef12345678"
]
```

**Format**: Array of subaccount addresses

---

### User Role

```json
{
  "role": "User"
}
```

**Possible Values**:
- `"User"`: Regular user account
- `"Agent"`: API wallet / agent
- `"Vault"`: Vault account
- `"Subaccount"`: Subaccount
- `"Missing"`: Address not found

---

## Error Response Formats

### Order Errors

```json
{
  "status": "ok",
  "response": {
    "type": "order",
    "data": {
      "statuses": [
        {
          "error": "Insufficient margin to place order."
        }
      ]
    }
  }
}
```

**Common Order Errors**:
- `"Price must be divisible by tick size."`
- `"Order must have minimum value of $10."`
- `"Insufficient margin to place order."`
- `"Reduce only order would increase position."`
- `"Post only order would have immediately matched, bbo was {bbo}."`
- `"Order could not immediately match against any resting orders."` (IOC)
- `"Invalid TP/SL price."`
- `"No liquidity available for market order."`
- `"Order would increase open interest while capped."`

### Cancel Errors

```json
{
  "error": "Order was never placed, already canceled, or filled."
}
```

### Signature Errors

```json
{
  "status": "error",
  "error": "User or API Wallet 0x1234... does not exist"
}
```

**Common Causes**:
- Incorrect signature (recovers wrong address)
- Wrong private key used
- Address not funded
- Agent wallet not registered

---

## WebSocket Response Formats

### Subscription Acknowledgment

```json
{
  "channel": "subscriptionResponse",
  "data": {
    "method": "subscribe",
    "subscription": {
      "type": "trades",
      "coin": "BTC"
    }
  }
}
```

### Trade Updates

```json
{
  "channel": "trades",
  "data": [
    {
      "coin": "BTC",
      "side": "B",
      "px": "50123.45",
      "sz": "0.5",
      "hash": "0x...",
      "time": 1704067200123,
      "tid": 123456789
    }
  ]
}
```

### Order Book Updates

```json
{
  "channel": "l2Book",
  "data": {
    "coin": "BTC",
    "levels": [
      [
        {"px": "50123.5", "sz": "1.234", "n": 3}
      ],
      [
        {"px": "50124.0", "sz": "0.567", "n": 1}
      ]
    ],
    "time": 1704067200000
  }
}
```

### User Order Updates

```json
{
  "channel": "orderUpdates",
  "data": [
    {
      "order": {
        "coin": "BTC",
        "side": "B",
        "limitPx": "50000.0",
        "sz": "0.1",
        "oid": 123456789,
        "timestamp": 1704067200000,
        "origSz": "0.1"
      },
      "status": "open",
      "statusTimestamp": 1704067200000
    }
  ]
}
```

### User Fills Stream

```json
{
  "channel": "userFills",
  "data": {
    "isSnapshot": false,
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "fills": [
      {
        "coin": "BTC",
        "px": "50100.0",
        "sz": "0.1",
        "side": "B",
        "time": 1704067200123,
        "startPosition": "0.0",
        "dir": "Open Long",
        "closedPnl": "0.0",
        "hash": "0x...",
        "oid": 123456789,
        "crossed": true,
        "fee": "2.505",
        "tid": 987654321
      }
    ]
  }
}
```

**Note**: First message has `isSnapshot: true`, subsequent have `isSnapshot: false`

---

## Data Type Reference

### Numeric Values
All numeric values are **strings** in JSON to preserve precision:
```json
{
  "price": "50123.45",      // String, not number
  "size": "0.12345",
  "volume": "1234567.89"
}
```

### Timestamps
Unix timestamps in **milliseconds**:
```json
{
  "time": 1704067200000,
  "timestamp": 1704067200123
}
```

### Addresses
42-character hexadecimal (with 0x prefix):
```json
{
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```

### Hashes
Transaction hashes (66 characters with 0x prefix):
```json
{
  "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
}
```

### Client Order IDs
128-bit hex string (34 characters with 0x prefix):
```json
{
  "cloid": "0x1234567890abcdef1234567890abcdef"
}
```

---

## Summary

### Key Patterns
1. **Info endpoints**: Return data directly (no wrapper)
2. **Exchange endpoints**: Wrapped in `{status, response: {type, data}}`
3. **Batch responses**: Array of statuses matching request order
4. **All numbers as strings**: Preserves precision
5. **Timestamps in milliseconds**: Unix epoch
6. **WebSocket updates**: `{channel, data}` format

### Error Handling
- Check `status` field first
- For batched requests, check each status in array
- Signature errors return wrong address message
- Rate limit errors: HTTP 429 or specific error message

### Response Validation
```rust
// Example validation structure
match response.status.as_str() {
    "ok" => {
        // Parse response.data based on type
        match response.response.type_.as_str() {
            "order" => parse_order_response(response.data),
            "cancel" => parse_cancel_response(response.data),
            _ => Err(UnknownType),
        }
    },
    "error" => Err(ApiError::from(response.error)),
    _ => Err(UnknownStatus),
}
```
