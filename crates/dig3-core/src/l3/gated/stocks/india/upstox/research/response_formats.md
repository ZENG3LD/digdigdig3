# Upstox - Response Formats

**All examples are EXACT formats from official API documentation.**

---

## Market Data - LTP (Last Traded Price)

### GET /v2/market-quote/ltp

**Request:**
```
GET https://api.upstox.com/v2/market-quote/ltp?instrument_key=NSE_EQ|INE669E01016,BSE_EQ|INE002A01018
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "NSE_EQ|INE669E01016": {
      "instrument_token": "NSE_EQ|INE669E01016",
      "last_price": 2750.50
    },
    "BSE_EQ|INE002A01018": {
      "instrument_token": "BSE_EQ|INE002A01018",
      "last_price": 2748.75
    }
  }
}
```

---

## Market Data - Full Market Quote

### GET /v2/market-quote/quotes

**Request:**
```
GET https://api.upstox.com/v2/market-quote/quotes?instrument_key=NSE_EQ|INE669E01016
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "NSE_EQ|INE669E01016": {
      "ohlc": {
        "open": 2730.00,
        "high": 2755.00,
        "low": 2728.50,
        "close": 2725.00
      },
      "depth": {
        "buy": [
          {
            "quantity": 500,
            "price": 2750.25,
            "orders": 5
          },
          {
            "quantity": 1000,
            "price": 2750.00,
            "orders": 10
          },
          {
            "quantity": 750,
            "price": 2749.75,
            "orders": 8
          },
          {
            "quantity": 600,
            "price": 2749.50,
            "orders": 6
          },
          {
            "quantity": 450,
            "price": 2749.25,
            "orders": 4
          }
        ],
        "sell": [
          {
            "quantity": 300,
            "price": 2750.50,
            "orders": 3
          },
          {
            "quantity": 800,
            "price": 2750.75,
            "orders": 7
          },
          {
            "quantity": 1200,
            "price": 2751.00,
            "orders": 12
          },
          {
            "quantity": 650,
            "price": 2751.25,
            "orders": 6
          },
          {
            "quantity": 500,
            "price": 2751.50,
            "orders": 5
          }
        ]
      },
      "timestamp": "2024-01-26T10:15:30+05:30",
      "instrument_token": "NSE_EQ|INE669E01016",
      "symbol": "TCS",
      "last_price": 2750.50,
      "volume": 1234567,
      "average_price": 2742.35,
      "oi": 0,
      "net_change": 25.50,
      "total_buy_quantity": 123456,
      "total_sell_quantity": 234567,
      "lower_circuit_limit": 2588.75,
      "upper_circuit_limit": 2861.25,
      "last_trade_time": "1706252130000",
      "oi_day_high": 0,
      "oi_day_low": 0
    }
  }
}
```

**Field Descriptions:**
- `ohlc` - Open, High, Low, Close for the day
- `depth.buy` - Bid side market depth (5 levels)
- `depth.sell` - Ask side market depth (5 levels)
- `timestamp` - ISO 8601 format
- `last_price` - Latest traded price
- `volume` - Total traded volume
- `average_price` - Average traded price
- `oi` - Open Interest (0 for equities, positive for F&O)
- `net_change` - Change from previous close
- `total_buy_quantity` - Total buy quantity in orderbook
- `total_sell_quantity` - Total sell quantity in orderbook
- `lower_circuit_limit` - Lower price band
- `upper_circuit_limit` - Upper price band
- `last_trade_time` - Unix timestamp in milliseconds

---

## Historical Data - Candles V3

### GET /v3/historical-candle/{instrument_key}/{unit}/{interval}/{to_date}/{from_date}

**Request:**
```
GET https://api.upstox.com/v3/historical-candle/NSE_EQ|INE848E01016/days/1/2024-01-26/2024-01-20
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "candles": [
      [
        "2024-01-26T09:15:00+05:30",
        2750.00,
        2755.00,
        2728.50,
        2750.50,
        1234567,
        0
      ],
      [
        "2024-01-25T09:15:00+05:30",
        2725.00,
        2735.00,
        2720.00,
        2725.00,
        987654,
        0
      ],
      [
        "2024-01-24T09:15:00+05:30",
        2710.00,
        2728.00,
        2705.00,
        2720.00,
        1100000,
        0
      ]
    ]
  }
}
```

**Candle Array Format:**
- Index 0: Timestamp (ISO 8601 string)
- Index 1: Open price (float)
- Index 2: High price (float)
- Index 3: Low price (float)
- Index 4: Close price (float)
- Index 5: Volume (integer)
- Index 6: Open Interest (integer, 0 for equities)

---

## Intraday Candles V3

### GET /v3/historical-candle/intraday/{instrument_key}/{unit}/{interval}

**Request:**
```
GET https://api.upstox.com/v3/historical-candle/intraday/NSE_EQ|INE669E01016/minutes/5
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "candles": [
      [
        "2024-01-26T10:15:00+05:30",
        2750.00,
        2752.00,
        2748.00,
        2750.50,
        12345,
        0
      ],
      [
        "2024-01-26T10:10:00+05:30",
        2749.00,
        2751.00,
        2747.00,
        2750.00,
        11200,
        0
      ]
    ]
  }
}
```

---

## Option Chain

### GET /v2/option/chain

**Request:**
```
GET https://api.upstox.com/v2/option/chain?instrument_key=NSE_INDEX|Nifty 50&expiry_date=2024-01-25
```

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "strike_price": 18500,
      "expiry": "2024-01-25",
      "underlying_key": "NSE_INDEX|Nifty 50",
      "underlying_spot_price": 18456.75,
      "call_options": {
        "instrument_key": "NSE_FO|54321",
        "market_data": {
          "ltp": 125.50,
          "close_price": 123.00,
          "volume": 15000,
          "oi": 234567,
          "bid_price": 125.25,
          "bid_qty": 50,
          "ask_price": 125.75,
          "ask_qty": 75,
          "prev_oi": 230000
        },
        "option_greeks": {
          "vega": 12.34,
          "theta": -0.52,
          "gamma": 0.0045,
          "delta": 0.5234,
          "iv": 18.45
        }
      },
      "put_options": {
        "instrument_key": "NSE_FO|54322",
        "market_data": {
          "ltp": 43.25,
          "close_price": 45.00,
          "volume": 18000,
          "oi": 345678,
          "bid_price": 43.00,
          "bid_qty": 100,
          "ask_price": 43.50,
          "ask_qty": 125,
          "prev_oi": 340000
        },
        "option_greeks": {
          "vega": 12.30,
          "theta": -0.48,
          "gamma": 0.0044,
          "delta": -0.4766,
          "iv": 18.52
        }
      }
    },
    {
      "strike_price": 18550,
      "expiry": "2024-01-25",
      "underlying_key": "NSE_INDEX|Nifty 50",
      "underlying_spot_price": 18456.75,
      "call_options": {
        "instrument_key": "NSE_FO|54323",
        "market_data": {
          "ltp": 85.75,
          "close_price": 88.00,
          "volume": 12000,
          "oi": 156789,
          "bid_price": 85.50,
          "bid_qty": 60,
          "ask_price": 86.00,
          "ask_qty": 80,
          "prev_oi": 155000
        },
        "option_greeks": {
          "vega": 13.20,
          "theta": -0.58,
          "gamma": 0.0052,
          "delta": 0.3845,
          "iv": 19.10
        }
      },
      "put_options": {
        "instrument_key": "NSE_FO|54324",
        "market_data": {
          "ltp": 138.25,
          "close_price": 135.50,
          "volume": 14500,
          "oi": 267890,
          "bid_price": 138.00,
          "bid_qty": 90,
          "ask_price": 138.50,
          "ask_qty": 110,
          "prev_oi": 265000
        },
        "option_greeks": {
          "vega": 13.25,
          "theta": -0.55,
          "gamma": 0.0051,
          "delta": -0.6155,
          "iv": 19.15
        }
      }
    }
  ]
}
```

**Field Descriptions:**
- `strike_price` - Option strike price
- `expiry` - Expiry date (YYYY-MM-DD)
- `underlying_spot_price` - Current price of underlying
- `ltp` - Last traded price
- `oi` - Open interest
- `prev_oi` - Previous day's open interest
- `vega` - Sensitivity to volatility
- `theta` - Time decay
- `gamma` - Rate of change of delta
- `delta` - Rate of change vs underlying (call: positive, put: negative)
- `iv` - Implied volatility (%)

---

## Trading - Place Order

### POST /v3/order/place

**Request:**
```json
{
  "quantity": 10,
  "product": "D",
  "validity": "DAY",
  "price": 2750.00,
  "tag": "my-strategy-001",
  "instrument_token": "NSE_EQ|INE669E01016",
  "order_type": "LIMIT",
  "transaction_type": "BUY",
  "disclosed_quantity": 0,
  "trigger_price": 0,
  "is_amo": false,
  "slice": true
}
```

**Response (Success):**
```json
{
  "status": "success",
  "data": {
    "order_id": "240126000123456"
  }
}
```

**Response (Error):**
```json
{
  "status": "error",
  "errors": [
    {
      "errorCode": "UDAPI1052",
      "message": "Order quantity cannot be zero",
      "propertyPath": "quantity",
      "invalidValue": 0,
      "error_code": "UDAPI1052",
      "property_path": "quantity",
      "invalid_value": 0
    }
  ]
}
```

---

## Trading - Order Book

### GET /v2/order/details

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "order_id": "240126000123456",
      "trading_symbol": "TCS",
      "exchange": "NSE",
      "instrument_token": "NSE_EQ|INE669E01016",
      "product": "D",
      "order_type": "LIMIT",
      "transaction_type": "BUY",
      "quantity": 10,
      "disclosed_quantity": 0,
      "price": 2750.00,
      "trigger_price": 0,
      "validity": "DAY",
      "status": "complete",
      "status_message": "Order executed",
      "filled_quantity": 10,
      "pending_quantity": 0,
      "average_price": 2748.50,
      "order_timestamp": "2024-01-26T09:30:15+05:30",
      "exchange_timestamp": "26-Jan-2024 09:30:15",
      "exchange_order_id": "1100000012345678",
      "parent_order_id": "",
      "is_amo": false,
      "tag": "my-strategy-001"
    },
    {
      "order_id": "240126000123457",
      "trading_symbol": "RELIANCE",
      "exchange": "NSE",
      "instrument_token": "NSE_EQ|INE002A01018",
      "product": "I",
      "order_type": "MARKET",
      "transaction_type": "SELL",
      "quantity": 5,
      "disclosed_quantity": 0,
      "price": 0,
      "trigger_price": 0,
      "validity": "DAY",
      "status": "open",
      "status_message": "Order placed",
      "filled_quantity": 0,
      "pending_quantity": 5,
      "average_price": 0,
      "order_timestamp": "2024-01-26T10:15:22+05:30",
      "exchange_timestamp": "26-Jan-2024 10:15:22",
      "exchange_order_id": "1100000012345679",
      "parent_order_id": "",
      "is_amo": false,
      "tag": ""
    }
  ]
}
```

**Order Status Values:**
- `open pending` - Order placed, awaiting exchange
- `validation pending` - Validation in progress
- `open` - Open at exchange
- `complete` - Fully executed
- `rejected` - Rejected
- `cancelled` - Cancelled
- `trigger pending` - Stop-loss waiting for trigger

---

## Portfolio - Positions

### GET /v2/portfolio/short-term-positions

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "exchange": "NSE",
      "product": "I",
      "trading_symbol": "NIFTY24JAN18500CE",
      "instrument_token": "NSE_FO|54321",
      "quantity": 50,
      "last_price": 125.50,
      "pnl": 2500.00,
      "unrealised": 2500.00,
      "realised": 0.00,
      "day_buy_quantity": 50,
      "day_buy_value": 6000.00,
      "day_buy_price": 120.00,
      "day_sell_quantity": 0,
      "day_sell_value": 0.00,
      "day_sell_price": 0.00,
      "overnight_quantity": 0,
      "overnight_buy_amount": 0.00,
      "overnight_sell_amount": 0.00,
      "overnight_buy_quantity": 0,
      "overnight_sell_quantity": 0,
      "multiplier": 1.0,
      "average_price": 120.00
    }
  ]
}
```

**Field Descriptions:**
- `quantity` - Net position quantity (positive = long, negative = short)
- `pnl` - Total profit/loss
- `unrealised` - Unrealized P&L (open position)
- `realised` - Realized P&L (closed trades)
- `day_buy_quantity` - Quantity bought today
- `day_buy_value` - Total value of buys
- `day_buy_price` - Average buy price
- `day_sell_quantity` - Quantity sold today
- `overnight_quantity` - Carried forward from previous day
- `multiplier` - Contract multiplier (usually 1.0)
- `average_price` - Average entry price

---

## Portfolio - Holdings

### GET /v2/portfolio/long-term-holdings

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "trading_symbol": "RELIANCE",
      "exchange": "NSE",
      "instrument_token": "NSE_EQ|INE002A01018",
      "isin": "INE002A01018",
      "product": "D",
      "quantity": 100,
      "average_price": 2500.00,
      "last_price": 2750.00,
      "pnl": 25000.00,
      "collateral_quantity": 100,
      "collateral_type": "WC",
      "company_name": "Reliance Industries Limited",
      "close_price": 2725.00,
      "haircut": 0.10,
      "t1_quantity": 0,
      "buy_quantity": 100,
      "sell_quantity": 0
    }
  ]
}
```

**Field Descriptions:**
- `isin` - ISIN identifier
- `quantity` - Total holding quantity
- `average_price` - Average purchase price
- `pnl` - Profit/loss (quantity × (last_price - average_price))
- `collateral_quantity` - Quantity pledged as collateral
- `collateral_type` - WC (With Collateral) or WOC (Without Collateral)
- `haircut` - Collateral haircut percentage
- `t1_quantity` - T+1 settlement pending quantity

---

## Account - Funds & Margin

### GET /v2/user/get-funds-and-margin

**Response:**
```json
{
  "status": "success",
  "data": {
    "equity": {
      "enabled": true,
      "net": 500000.00,
      "available_margin": 450000.00,
      "used_margin": 50000.00,
      "category": "equity"
    },
    "commodity": {
      "enabled": true,
      "net": 100000.00,
      "available_margin": 95000.00,
      "used_margin": 5000.00,
      "category": "commodity"
    }
  }
}
```

**Note:** From July 19, 2025, equity and commodity funds are combined in the equity object.

---

## Account - Trade Charges

### GET /v2/trade/profit-loss/charges

**Request:**
```
GET https://api.upstox.com/v2/trade/profit-loss/charges?segment=EQ&financial_year=2324&from_date=01-01-2024&to_date=26-01-2024
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "total_charges": 1250.50,
    "brokerage": 500.00,
    "taxes": {
      "gst": 90.00,
      "stt": 250.00,
      "stamp_duty": 100.50
    },
    "other_charges": {
      "transaction_charge": 150.00,
      "clearing_charge": 80.00,
      "sebi_turnover_fee": 50.00,
      "ipft_charge": 20.00,
      "demat_transaction_charge": 10.00
    }
  }
}
```

**Field Descriptions:**
- `total_charges` - Sum of all charges
- `brokerage` - Broker commission
- `gst` - Goods and Services Tax on brokerage
- `stt` - Securities Transaction Tax
- `stamp_duty` - Government stamp duty
- `transaction_charge` - Exchange transaction fee
- `clearing_charge` - Clearing corporation fee
- `sebi_turnover_fee` - SEBI regulatory fee
- `ipft_charge` - IPFT fee (if applicable)
- `demat_transaction_charge` - Demat DP charge

---

## User Profile

### GET /v2/user/profile

**Response:**
```json
{
  "status": "success",
  "data": {
    "email": "user@example.com",
    "exchanges": ["NSE", "BSE", "MCX"],
    "products": ["I", "D", "CO", "MTF"],
    "broker": "UPSTOX",
    "user_id": "ABC123",
    "user_name": "John Doe",
    "user_type": "individual",
    "poa": false,
    "is_active": true
  }
}
```

---

## GTT Order Details

### GET /v2/gtt/order/{gtt_order_id}

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": "GTT123456",
    "instrument_key": "NSE_EQ|INE669E01016",
    "product": "D",
    "order_type": "LIMIT",
    "transaction_type": "BUY",
    "quantity": 5,
    "price": 0,
    "trigger_price": 2700.00,
    "status": "active",
    "created_at": "2024-01-26T08:00:00+05:30",
    "updated_at": "2024-01-26T08:00:00+05:30",
    "expires_at": "2024-02-26T15:30:00+05:30",
    "rules": [
      {
        "id": 1,
        "strategy": "ENTRY",
        "status": "pending",
        "trigger_price": 2700.00,
        "order_type": "LIMIT",
        "price": 2705.00,
        "quantity": 5
      },
      {
        "id": 2,
        "strategy": "TARGET",
        "status": "pending",
        "trigger_price": 2800.00,
        "order_type": "LIMIT",
        "price": 2795.00,
        "quantity": 5
      },
      {
        "id": 3,
        "strategy": "STOPLOSS",
        "status": "pending",
        "trigger_price": 2650.00,
        "order_type": "SL",
        "price": 2645.00,
        "quantity": 5
      }
    ]
  }
}
```

**GTT Strategies:**
- `ENTRY` - Entry trigger
- `TARGET` - Target/profit booking
- `STOPLOSS` - Stop loss

**GTT Status:**
- `active` - Waiting for trigger
- `triggered` - Triggered, order placed
- `cancelled` - Cancelled
- `expired` - Expired without trigger

---

## Instrument File (JSON Download)

### Complete.json.gz

**URL:** https://assets.upstox.com/market-quote/instruments/exchange/complete.json.gz

**Format:** Array of instrument objects

**Sample Record (Equity):**
```json
{
  "instrument_key": "NSE_EQ|INE669E01016",
  "exchange_token": "2885",
  "trading_symbol": "TCS",
  "name": "Tata Consultancy Services Limited",
  "last_price": 2750.50,
  "expiry": "",
  "strike": 0,
  "tick_size": 0.05,
  "lot_size": 1,
  "instrument_type": "EQ",
  "option_type": "",
  "exchange": "NSE",
  "segment": "NSE_EQ",
  "isin": "INE669E01016"
}
```

**Sample Record (Futures):**
```json
{
  "instrument_key": "NSE_FO|45678",
  "exchange_token": "45678",
  "trading_symbol": "NIFTY24JANFUT",
  "name": "NIFTY",
  "last_price": 18500.00,
  "expiry": "2024-01-25",
  "strike": 0,
  "tick_size": 0.05,
  "lot_size": 50,
  "instrument_type": "FUT",
  "option_type": "",
  "exchange": "NSE",
  "segment": "NSE_FO",
  "isin": ""
}
```

**Sample Record (Options):**
```json
{
  "instrument_key": "NSE_FO|54321",
  "exchange_token": "54321",
  "trading_symbol": "NIFTY24JAN18500CE",
  "name": "NIFTY",
  "last_price": 125.50,
  "expiry": "2024-01-25",
  "strike": 18500,
  "tick_size": 0.05,
  "lot_size": 50,
  "instrument_type": "CE",
  "option_type": "Call",
  "exchange": "NSE",
  "segment": "NSE_FO",
  "isin": ""
}
```

---

## Error Response Format

**Standard Error:**
```json
{
  "status": "error",
  "errors": [
    {
      "errorCode": "UDAPI1026",
      "message": "Instrument key required",
      "propertyPath": "instrument_key",
      "invalidValue": null,
      "error_code": "UDAPI1026",
      "property_path": "instrument_key",
      "invalid_value": null
    }
  ]
}
```

**Common Error Codes:**
- `UDAPI1026` - Instrument key required
- `UDAPI1004` - Valid order type required
- `UDAPI1052` - Order quantity cannot be zero
- `UDAPI100074` - API accessible 5:30 AM - 12:00 AM IST only
- `UDAPI100049` - Access restricted; use Uplink Business
- `UDAPI1087` - Invalid symbol or instrument_key format
- `UDAPI100042` - Max 500 instrument keys per request
- `UDAPI100011` - Invalid instrument key
- `UDAPI1088` - Improper date formatting

---

## Notes on Response Formats

1. **Status Field:** Always present, either "success" or "error"
2. **Data Field:** Contains actual response data on success
3. **Errors Array:** Contains error details on failure
4. **Timestamps:**
   - ISO 8601 format: "2024-01-26T10:15:30+05:30"
   - Unix milliseconds: "1706252130000"
   - Exchange format: "26-Jan-2024 10:15:30"
5. **Instrument Keys:** Format: "{SEGMENT}|{IDENTIFIER}"
   - Equity: "NSE_EQ|INE669E01016" (ISIN)
   - F&O: "NSE_FO|54321" (exchange token)
6. **Prices:** Float values with appropriate precision (usually 2 decimal places)
7. **Quantities:** Integer values
8. **Arrays:** Candle data, depth levels, order lists
9. **Nested Objects:** Market data, option Greeks, GTT rules

All response formats are subject to change. Always refer to official documentation for the latest specifications.
