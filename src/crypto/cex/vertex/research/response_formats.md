# Vertex Protocol Response Formats

All Vertex Protocol API responses follow a consistent JSON structure with status indicators and data payloads.

## Response Wrapper

### Success Response

```json
{
  "status": "success",
  "data": { /* endpoint-specific data */ }
}
```

### Error Response

```json
{
  "status": "error",
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

## Common Error Codes

- `INVALID_SIGNATURE` - EIP-712 signature verification failed
- `INSUFFICIENT_BALANCE` - Not enough funds for operation
- `RATE_LIMIT_EXCEEDED` - Too many requests
- `INVALID_PRODUCT_ID` - Product does not exist
- `ORDER_NOT_FOUND` - Order digest not found
- `INVALID_SENDER` - Sender format invalid
- `EXPIRED_TRANSACTION` - Transaction expiration passed
- `DUPLICATE_NONCE` - Nonce already used

---

## Market Data Responses

### All Products

```json
{
  "status": "success",
  "data": {
    "spot_products": [
      {
        "product_id": 0,
        "oracle_price_x18": "1000000000000000000",
        "risk": {
          "long_weight_initial_x18": "1000000000000000000",
          "short_weight_initial_x18": "1000000000000000000",
          "long_weight_maintenance_x18": "950000000000000000",
          "short_weight_maintenance_x18": "1050000000000000000",
          "large_position_penalty_x18": "0"
        },
        "config": {
          "token": "0x...",
          "interest_inflection_util_x18": "800000000000000000",
          "interest_floor_x18": "10000000000000000",
          "interest_small_cap_x18": "40000000000000000",
          "interest_large_cap_x18": "1000000000000000000"
        },
        "state": {
          "cumulative_deposits_multiplier_x18": "1003419811982007193",
          "cumulative_borrows_multiplier_x18": "1005234234234234234",
          "total_deposits_normalized": "100000000000000000000000",
          "total_borrows_normalized": "50000000000000000000000"
        },
        "lp_state": {
          "supply": "1000000000000000000000",
          "quote": {
            "amount": "500000000000000000000",
            "last_cumulative_multiplier_x18": "1003419811982007193"
          },
          "base": {
            "amount": "500000000000000000000",
            "last_cumulative_multiplier_x18": "1003419811982007193"
          }
        },
        "book_info": {
          "size_increment": "1000000000000000",
          "price_increment_x18": "10000000000000000",
          "min_size": "10000000000000000",
          "collected_fees": "123456789000000000",
          "lp_spread_x18": "3000000000000000"
        }
      }
    ],
    "perp_products": [
      {
        "product_id": 2,
        "oracle_price_x18": "30000000000000000000000",
        "risk": {
          "long_weight_initial_x18": "900000000000000000",
          "short_weight_initial_x18": "1100000000000000000",
          "long_weight_maintenance_x18": "950000000000000000",
          "short_weight_maintenance_x18": "1050000000000000000",
          "large_position_penalty_x18": "0"
        },
        "config": {
          "token": "0x0000000000000000000000000000000000000000",
          "interest_inflection_util_x18": "0",
          "interest_floor_x18": "0",
          "interest_small_cap_x18": "0",
          "interest_large_cap_x18": "0"
        },
        "state": {
          "cumulative_funding_long_x18": "1000123456789012345",
          "cumulative_funding_short_x18": "999876543210987654",
          "available_settle": "1000000000000000000000",
          "open_interest": "50000000000000000000000"
        },
        "lp_state": {
          "supply": "5000000000000000000000",
          "quote": {
            "amount": "150000000000000000000000",
            "last_cumulative_funding_x18": "1000123456789012345"
          },
          "base": {
            "amount": "5000000000000000000",
            "last_cumulative_funding_x18": "1000123456789012345"
          }
        },
        "book_info": {
          "size_increment": "1000000000000000",
          "price_increment_x18": "1000000000000000000",
          "min_size": "10000000000000000",
          "collected_fees": "987654321000000000",
          "lp_spread_x18": "5000000000000000"
        }
      }
    ]
  }
}
```

**Key Fields**:
- `oracle_price_x18`: Current oracle price (18 decimal precision)
- `risk`: Risk parameters for margin calculations
- `config`: Product configuration
- `state`: Current product state (deposits, borrows, funding, OI)
- `lp_state`: Liquidity provider state
- `book_info`: Orderbook configuration

### Symbols

```json
{
  "status": "success",
  "symbols": {
    "0": "USDC",
    "1": "BTC",
    "2": "BTC-PERP",
    "3": "ETH",
    "4": "ETH-PERP",
    "5": "USDT",
    "6": "ARB-PERP",
    "7": "USDC.E"
  }
}
```

### Market Liquidity (Orderbook)

```json
{
  "status": "success",
  "data": {
    "product_id": 2,
    "timestamp": 1234567890,
    "bids": [
      ["29950000000000000000000", "5000000000000000000"],
      ["29900000000000000000000", "10000000000000000000"],
      ["29850000000000000000000", "15000000000000000000"]
    ],
    "asks": [
      ["30050000000000000000000", "3000000000000000000"],
      ["30100000000000000000000", "8000000000000000000"],
      ["30150000000000000000000", "12000000000000000000"]
    ]
  }
}
```

**Array Format**: `[price_x18, size]`

### Market Price (Ticker)

```json
{
  "status": "success",
  "data": {
    "product_id": 2,
    "bid_x18": "29950000000000000000000",
    "ask_x18": "30050000000000000000000",
    "last_updated": 1234567890
  }
}
```

### Candlesticks

```json
{
  "status": "success",
  "candlesticks": [
    {
      "product_id": 2,
      "granularity": 3600,
      "open_x18": "30000000000000000000000",
      "high_x18": "31000000000000000000000",
      "low_x18": "29500000000000000000000",
      "close_x18": "30500000000000000000000",
      "volume": "1500000000000000000000",
      "timestamp": 1234567890
    },
    {
      "product_id": 2,
      "granularity": 3600,
      "open_x18": "30500000000000000000000",
      "high_x18": "31200000000000000000000",
      "low_x18": "30200000000000000000000",
      "close_x18": "30800000000000000000000",
      "volume": "1800000000000000000000",
      "timestamp": 1234571490
    }
  ]
}
```

**Fields**:
- `granularity`: Candle period in seconds (60, 300, 900, 3600, 14400, 86400)
- All prices in X18 format
- `timestamp`: Unix timestamp (seconds)

### Contracts Info

```json
{
  "status": "success",
  "data": {
    "chain_id": 42161,
    "endpoint_addr": "0x...",
    "book_addrs": ["0x...", "0x..."],
    "engine_addr": "0x...",
    "sequencer_addr": "0x...",
    "clearinghouse_addr": "0x...",
    "spot_engine_addr": "0x...",
    "perp_engine_addr": "0x..."
  }
}
```

### Status

```json
{
  "status": "success",
  "data": {
    "status": "online",
    "server_time": 1234567890123
  }
}
```

---

## Trading Responses

### Place Order (Success)

```json
{
  "status": "success",
  "data": {
    "digest": "0x123abc456def789012345678901234567890abcdef1234567890abcdef123456"
  }
}
```

**digest**: Order identifier (keccak256 hash of order struct)

### Cancel Orders (Success)

```json
{
  "status": "success"
}
```

### Cancel Product Orders (Success)

```json
{
  "status": "success"
}
```

### Execute Error

```json
{
  "status": "error",
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Insufficient balance to place order. Required: 1000 USDC, Available: 500 USDC"
  }
}
```

---

## Account Responses

### Subaccount Info (Balances & Positions)

```json
{
  "status": "success",
  "data": {
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "exists": true,
    "healths": [
      {
        "assets": "75323297691833342306",
        "liabilities": "46329556869051092241",
        "health": "28993740822782250065"
      },
      {
        "assets": "75323297691833342306",
        "liabilities": "48562358925634512345",
        "health": "26760938766198829961"
      }
    ],
    "health_contributions": [
      ["75323297691833340000", "75323297691833340000", "75323297691833340000"],
      ["0", "0", "0"],
      ["-5000000000000000000", "-5000000000000000000", "-5000000000000000000"]
    ],
    "spot_count": 2,
    "perp_count": 1,
    "spot_balances": [
      {
        "product_id": 0,
        "balance": {
          "amount": "100000000000000000000",
          "last_cumulative_multiplier_x18": "1003419811982007193"
        },
        "lp_balance": {
          "amount": "0"
        }
      },
      {
        "product_id": 1,
        "balance": {
          "amount": "2000000000000000000",
          "last_cumulative_multiplier_x18": "1000000000000000000"
        },
        "lp_balance": {
          "amount": "0"
        }
      }
    ],
    "perp_balances": [
      {
        "product_id": 2,
        "balance": {
          "amount": "5000000000000000000",
          "v_quote_balance": "150000000000000000000",
          "last_cumulative_funding_x18": "1000123456789012345"
        },
        "lp_balance": {
          "amount": "0"
        }
      }
    ]
  }
}
```

**Health Array**:
- `healths[0]`: Initial health (used for opening positions)
- `healths[1]`: Maintenance health (used for liquidations)

**Health Fields**:
- `assets`: Total weighted assets
- `liabilities`: Total weighted liabilities
- `health`: assets - liabilities (positive = healthy, negative = underwater)

**Health Contributions**:
- Indexed by product_id
- 3 values per product: [initial_long, initial_short, maintenance]

**Spot Balance**:
- `amount`: Balance amount (positive = deposit, negative = borrow)
- `last_cumulative_multiplier_x18`: Interest checkpoint

**Perp Balance**:
- `amount`: Position size (positive = long, negative = short)
- `v_quote_balance`: Virtual quote balance (for funding calculations)
- `last_cumulative_funding_x18`: Funding payment checkpoint

### Fee Rates

```json
{
  "status": "success",
  "data": {
    "subaccount": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
    "taker_rate_x18": "500000000000000",
    "maker_rate_x18": "-100000000000000"
  }
}
```

**Rates in X18**:
- `taker_rate_x18`: Taker fee (0.0005 = 0.05%)
- `maker_rate_x18`: Maker rebate (negative = rebate, -0.0001 = -0.01%)

### Max Withdrawable

```json
{
  "status": "success",
  "data": {
    "max_withdrawable_x18": "50000000000000000000"
  }
}
```

---

## Position Responses

### Open Orders

```json
{
  "status": "success",
  "data": {
    "orders": [
      {
        "product_id": 2,
        "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
        "priceX18": "30000000000000000000000",
        "amount": "1000000000000000000",
        "expiration": 4611686018427387904,
        "nonce": 1234567890123,
        "unfilled_amount": "500000000000000000",
        "digest": "0x123abc456def789012345678901234567890abcdef1234567890abcdef123456",
        "placed_at": 1234567890
      },
      {
        "product_id": 4,
        "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
        "priceX18": "2000000000000000000000",
        "amount": "-5000000000000000000",
        "expiration": 1234567900,
        "nonce": 1234567890124,
        "unfilled_amount": "-5000000000000000000",
        "digest": "0x456def789012345678901234567890abcdef1234567890abcdef123456789abc",
        "placed_at": 1234567850
      }
    ]
  }
}
```

**Order Fields**:
- `amount`: Total order size (positive = buy, negative = sell)
- `unfilled_amount`: Remaining size to be filled
- `digest`: Order identifier
- `placed_at`: Order placement timestamp

### Single Order

```json
{
  "status": "success",
  "data": {
    "order": {
      "product_id": 2,
      "sender": "0x7a5ec2748e9065794491a8d29dcf3f9edb8d7c43000000000000000000000000",
      "priceX18": "30000000000000000000000",
      "amount": "1000000000000000000",
      "expiration": 4611686018427387904,
      "nonce": 1234567890123,
      "unfilled_amount": "500000000000000000",
      "digest": "0x123abc456def789012345678901234567890abcdef1234567890abcdef123456",
      "placed_at": 1234567890
    }
  }
}
```

### Max Order Size

```json
{
  "status": "success",
  "data": {
    "max_order_size_x18": "10000000000000000000"
  }
}
```

### Funding Rate

```json
{
  "status": "success",
  "data": {
    "product_id": 2,
    "funding_rate_x18": "50000000000000",
    "timestamp": 1234567890,
    "next_funding_time": 1234571490
  }
}
```

---

## WebSocket Responses

### Subscription Confirmation

```json
{
  "id": 10,
  "status": "subscribed",
  "stream": {
    "type": "trade",
    "product_id": 2
  }
}
```

### Trade Stream

```json
{
  "stream": "trade",
  "data": {
    "product_id": 2,
    "price_x18": "30500000000000000000000",
    "size": "1000000000000000000",
    "side": "buy",
    "timestamp": 1234567890,
    "digest": "0x..."
  }
}
```

### OrderUpdate Stream

```json
{
  "stream": "order_update",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec...",
    "order": {
      "digest": "0x123abc...",
      "priceX18": "30000000000000000000000",
      "amount": "1000000000000000000",
      "unfilled_amount": "200000000000000000",
      "status": "partially_filled"
    },
    "timestamp": 1234567890
  }
}
```

**Order Status Values**:
- `open`
- `partially_filled`
- `filled`
- `cancelled`
- `rejected`

### BestBidOffer Stream

```json
{
  "stream": "best_bid_offer",
  "data": {
    "product_id": 2,
    "bid_x18": "29950000000000000000000",
    "ask_x18": "30050000000000000000000",
    "bid_size": "5000000000000000000",
    "ask_size": "3000000000000000000",
    "timestamp": 1234567890
  }
}
```

### Fill Stream

```json
{
  "stream": "fill",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec...",
    "digest": "0x123abc...",
    "price_x18": "30500000000000000000000",
    "size": "800000000000000000",
    "side": "buy",
    "fee": "12200000000000000",
    "is_maker": false,
    "timestamp": 1234567890
  }
}
```

### PositionChange Stream

```json
{
  "stream": "position_change",
  "data": {
    "product_id": 2,
    "subaccount": "0x7a5ec...",
    "balance": {
      "amount": "5800000000000000000",
      "v_quote_balance": "174000000000000000000",
      "last_cumulative_funding_x18": "1000123456789012345"
    },
    "timestamp": 1234567890
  }
}
```

### BookDepth Stream

```json
{
  "stream": "book_depth",
  "data": {
    "product_id": 2,
    "bids": [
      ["29950000000000000000000", "5000000000000000000"],
      ["29900000000000000000000", "10000000000000000000"]
    ],
    "asks": [
      ["30050000000000000000000", "3000000000000000000"],
      ["30100000000000000000000", "8000000000000000000"]
    ],
    "timestamp": 1234567890
  }
}
```

---

## Data Format Notes

### X18 Precision

All numerical values use 18 decimal precision:
- Prices: `priceX18`
- Amounts: stored as int128 with 18 decimals
- Rates: stored as int128 with 18 decimals

**Conversion**:
```rust
// To human-readable
fn from_x18(value: &str) -> f64 {
    value.parse::<i128>().unwrap() as f64 / 1e18
}

// To X18
fn to_x18(value: f64) -> String {
    ((value * 1e18) as i128).to_string()
}
```

**Examples**:
- 1 USDC = `"1000000000000000000"`
- $30,000 BTC = `"30000000000000000000000"`
- 0.05% fee = `"500000000000000"`

### Timestamp Formats

- **Query responses**: Unix seconds (u64)
- **WebSocket auth**: Unix milliseconds (u64)
- **Candlesticks**: Unix seconds (u64)

### Order Amounts

- **Positive**: Buy order / Long position
- **Negative**: Sell order / Short position

**Example**: `"amount": "-1000000000000000000"` = Sell 1.0

### Nested Conversions

Some responses contain nested X18 values. Recursively convert all numeric strings.

---

## Error Response Examples

### Invalid Signature

```json
{
  "status": "error",
  "error": {
    "code": "INVALID_SIGNATURE",
    "message": "Signature verification failed. Expected signer: 0x7a5ec..., recovered: 0x123ab..."
  }
}
```

### Insufficient Balance

```json
{
  "status": "error",
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Insufficient balance to place order. Required: 30000.00 USDC, Available: 15000.00 USDC"
  }
}
```

### Rate Limit Exceeded

```json
{
  "status": "error",
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for place_order. Limit: 10 req/s, Current: 15 req/s. Retry after 1000ms"
  }
}
```

### Product Not Found

```json
{
  "status": "error",
  "error": {
    "code": "INVALID_PRODUCT_ID",
    "message": "Product ID 999 does not exist"
  }
}
```

### Order Not Found

```json
{
  "status": "error",
  "error": {
    "code": "ORDER_NOT_FOUND",
    "message": "Order with digest 0x123abc... not found"
  }
}
```
