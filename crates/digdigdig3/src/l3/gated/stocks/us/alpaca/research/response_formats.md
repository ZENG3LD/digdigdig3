# Alpaca - Response Formats

All responses are in **JSON format** (or MessagePack for WebSocket if specified).

## REST API - Common Response Structure

All REST responses include rate limit headers:
```http
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 145
X-RateLimit-Reset: 1705599600
```

---

## Trading API Responses

### GET /v2/account

```json
{
  "id": "904837e3-3b76-47ec-b432-046db621571b",
  "account_number": "123456789",
  "status": "ACTIVE",
  "crypto_status": "ACTIVE",
  "currency": "USD",
  "buying_power": "262113.632",
  "regt_buying_power": "262113.632",
  "daytrading_buying_power": "0",
  "non_marginable_buying_power": "131056.816",
  "cash": "131056.816",
  "accrued_fees": "0",
  "pending_transfer_out": "0",
  "pending_transfer_in": "0",
  "portfolio_value": "131856.816",
  "pattern_day_trader": false,
  "trading_blocked": false,
  "transfers_blocked": false,
  "account_blocked": false,
  "created_at": "2024-01-01T00:00:00Z",
  "trade_suspended_by_user": false,
  "multiplier": "2",
  "shorting_enabled": true,
  "equity": "131856.816",
  "last_equity": "131000.00",
  "long_market_value": "800.00",
  "short_market_value": "0",
  "initial_margin": "400.00",
  "maintenance_margin": "240.00",
  "last_maintenance_margin": "230.00",
  "sma": "0",
  "daytrade_count": 0
}
```

**Key fields:**
- `buying_power`: Current available buying power
- `cash`: Cash balance
- `equity`: Total account value (cash + positions)
- `portfolio_value`: Total portfolio value
- `multiplier`: Margin multiplier (1=cash, 2=2x margin, 4=4x daytrading)
- `pattern_day_trader`: PDT flag (true if 4+ day trades in 5 days with <$25k)
- `trading_blocked`: Account restricted from trading

---

### POST /v2/orders (Create Order)

**Response (Order object):**
```json
{
  "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
  "client_order_id": "my_order_123",
  "created_at": "2024-01-18T15:30:00.123456Z",
  "updated_at": "2024-01-18T15:30:00.123456Z",
  "submitted_at": "2024-01-18T15:30:00.123456Z",
  "filled_at": null,
  "expired_at": null,
  "canceled_at": null,
  "failed_at": null,
  "replaced_at": null,
  "replaced_by": null,
  "replaces": null,
  "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
  "symbol": "AAPL",
  "asset_class": "us_equity",
  "notional": null,
  "qty": "100",
  "filled_qty": "0",
  "filled_avg_price": null,
  "order_class": "simple",
  "order_type": "market",
  "type": "market",
  "side": "buy",
  "time_in_force": "day",
  "limit_price": null,
  "stop_price": null,
  "status": "accepted",
  "extended_hours": false,
  "legs": null,
  "trail_percent": null,
  "trail_price": null,
  "hwm": null,
  "subtag": null,
  "source": "api"
}
```

**Status values:**
- `new`, `accepted`, `pending_new`, `partially_filled`, `filled`, `done_for_day`, `canceled`, `expired`, `replaced`, `pending_cancel`, `pending_replace`, `rejected`, `suspended`, `calculated`, `stopped`

---

### GET /v2/orders (List Orders)

```json
[
  {
    "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
    "client_order_id": "order_1",
    "created_at": "2024-01-18T15:30:00Z",
    "symbol": "AAPL",
    "qty": "100",
    "filled_qty": "100",
    "filled_avg_price": "150.25",
    "side": "buy",
    "type": "market",
    "status": "filled"
  },
  {
    "id": "74e21155-d465-4e67-9b5e-b5a5c2c4b5c6",
    "symbol": "TSLA",
    "qty": "50",
    "filled_qty": "0",
    "side": "sell",
    "type": "limit",
    "limit_price": "200.00",
    "status": "accepted"
  }
]
```

---

### GET /v2/positions (All Positions)

```json
[
  {
    "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
    "symbol": "AAPL",
    "exchange": "NASDAQ",
    "asset_class": "us_equity",
    "avg_entry_price": "148.50",
    "qty": "100",
    "qty_available": "100",
    "side": "long",
    "market_value": "15025.00",
    "cost_basis": "14850.00",
    "unrealized_pl": "175.00",
    "unrealized_plpc": "0.0118",
    "unrealized_intraday_pl": "50.00",
    "unrealized_intraday_plpc": "0.0034",
    "current_price": "150.25",
    "lastday_price": "149.00",
    "change_today": "0.0084"
  },
  {
    "symbol": "TSLA",
    "qty": "50",
    "avg_entry_price": "195.00",
    "market_value": "10000.00",
    "unrealized_pl": "250.00",
    "current_price": "200.00",
    "side": "long"
  }
]
```

**Key fields:**
- `avg_entry_price`: Average entry price (cost basis per share)
- `market_value`: Current position value (qty × current_price)
- `unrealized_pl`: Unrealized profit/loss in dollars
- `unrealized_plpc`: Unrealized P&L percentage
- `cost_basis`: Total cost (qty × avg_entry_price)

---

### GET /v2/assets (List Assets)

```json
[
  {
    "id": "904837e3-3b76-47ec-b432-046db621571b",
    "class": "us_equity",
    "exchange": "NASDAQ",
    "symbol": "AAPL",
    "name": "Apple Inc.",
    "status": "active",
    "tradable": true,
    "marginable": true,
    "maintenance_margin_requirement": 30,
    "shortable": true,
    "easy_to_borrow": true,
    "fractionable": true,
    "min_order_size": "1",
    "min_trade_increment": "0.000001",
    "price_increment": "0.01",
    "attributes": [],
    "options_enabled": true
  }
]
```

**Key fields:**
- `tradable`: Can place orders (true/false)
- `marginable`: Can trade on margin
- `shortable`: Can short sell
- `fractionable`: Supports fractional shares
- `options_enabled`: Has option contracts available

---

### GET /v2/calendar (Trading Calendar)

```json
[
  {
    "date": "2024-01-18",
    "open": "09:30",
    "close": "16:00",
    "session_open": "0400",
    "session_close": "2000"
  },
  {
    "date": "2024-12-24",
    "open": "09:30",
    "close": "13:00",
    "session_open": "0400",
    "session_close": "1300"
  }
]
```

**Times in Eastern Time (ET)**
- Early closure example: Christmas Eve closes at 13:00
- `session_open/close`: Extended hours (pre-market + after-hours)

---

### GET /v2/clock (Market Clock)

```json
{
  "timestamp": "2024-01-18T15:30:00.123456-05:00",
  "is_open": true,
  "next_open": "2024-01-19T09:30:00-05:00",
  "next_close": "2024-01-18T16:00:00-05:00"
}
```

---

## Market Data API Responses

### GET /v2/stocks/bars (Historical Bars)

```json
{
  "bars": {
    "AAPL": [
      {
        "t": "2024-01-18T14:30:00Z",
        "o": 150.00,
        "h": 150.50,
        "l": 149.80,
        "c": 150.25,
        "v": 125000,
        "n": 1500,
        "vw": 150.12
      },
      {
        "t": "2024-01-18T14:31:00Z",
        "o": 150.25,
        "h": 150.60,
        "l": 150.20,
        "c": 150.55,
        "v": 98000,
        "n": 1200,
        "vw": 150.40
      }
    ],
    "TSLA": [
      {
        "t": "2024-01-18T14:30:00Z",
        "o": 200.00,
        "h": 201.00,
        "l": 199.50,
        "c": 200.75,
        "v": 85000,
        "n": 900,
        "vw": 200.50
      }
    ]
  },
  "next_page_token": "QU1aTHx8MjAyNC0wMS0xOFQxNDozMjowMFo="
}
```

**Bar fields:**
- `t`: Timestamp (RFC-3339 format, UTC)
- `o`: Open price
- `h`: High price
- `l`: Low price
- `c`: Close price
- `v`: Volume (number of shares)
- `n`: Number of trades in bar
- `vw`: Volume-weighted average price

**Pagination:**
- `next_page_token`: Use in next request to get more data
- `null` when no more data available

---

### GET /v2/stocks/trades (Historical Trades)

```json
{
  "trades": {
    "AAPL": [
      {
        "t": "2024-01-18T15:30:45.123456789Z",
        "x": "V",
        "p": 150.25,
        "s": 100,
        "c": ["@", "I"],
        "i": 52983525029461,
        "z": "C"
      },
      {
        "t": "2024-01-18T15:30:45.234567890Z",
        "x": "D",
        "p": 150.26,
        "s": 50,
        "c": ["@"],
        "i": 52983525029462,
        "z": "C"
      }
    ]
  },
  "next_page_token": "QU1aTHx8..."
}
```

**Trade fields:**
- `t`: Timestamp (nanosecond precision)
- `x`: Exchange code (V=IEX, D=EDGX, Q=NASDAQ, etc.)
- `p`: Price
- `s`: Size (shares)
- `c`: Condition codes (array of strings, e.g., "@" = regular sale)
- `i`: Trade ID (unique)
- `z`: Tape (A/B/C for CTA tapes)

---

### GET /v2/stocks/quotes (Historical Quotes)

```json
{
  "quotes": {
    "AAPL": [
      {
        "t": "2024-01-18T15:30:45.123456789Z",
        "ax": "Q",
        "ap": 150.26,
        "as": 200,
        "bx": "U",
        "bp": 150.24,
        "bs": 100,
        "c": ["R"],
        "z": "C"
      }
    ]
  },
  "next_page_token": null
}
```

**Quote fields:**
- `t`: Timestamp
- `ax`: Ask exchange
- `ap`: Ask price
- `as`: Ask size
- `bx`: Bid exchange
- `bp`: Bid price
- `bs`: Bid size
- `c`: Condition codes
- `z`: Tape

---

### GET /v2/stocks/snapshots (Snapshots)

```json
{
  "AAPL": {
    "latestTrade": {
      "t": "2024-01-18T15:59:59.999Z",
      "x": "V",
      "p": 150.25,
      "s": 100,
      "c": ["@"],
      "i": 52983525029999,
      "z": "C"
    },
    "latestQuote": {
      "t": "2024-01-18T15:59:59.999Z",
      "ax": "Q",
      "ap": 150.27,
      "as": 200,
      "bx": "U",
      "bp": 150.25,
      "bs": 100,
      "c": ["R"],
      "z": "C"
    },
    "minuteBar": {
      "t": "2024-01-18T15:59:00Z",
      "o": 150.20,
      "h": 150.30,
      "l": 150.15,
      "c": 150.25,
      "v": 50000,
      "n": 600,
      "vw": 150.22
    },
    "dailyBar": {
      "t": "2024-01-18T05:00:00Z",
      "o": 148.50,
      "h": 151.00,
      "l": 147.80,
      "c": 150.25,
      "v": 50000000,
      "n": 250000,
      "vw": 149.85
    },
    "prevDailyBar": {
      "t": "2024-01-17T05:00:00Z",
      "o": 149.00,
      "h": 149.50,
      "l": 147.50,
      "c": 148.00,
      "v": 45000000,
      "n": 230000,
      "vw": 148.50
    }
  },
  "TSLA": {
    "latestTrade": {...},
    "latestQuote": {...},
    "minuteBar": {...},
    "dailyBar": {...},
    "prevDailyBar": {...}
  }
}
```

**Snapshot includes:**
- Latest trade execution
- Latest bid/ask quote
- Current minute bar
- Current daily bar (intraday)
- Previous day's bar

---

### GET /v1beta1/options/snapshots/{underlying} (Option Chain)

```json
{
  "snapshots": {
    "AAPL250117C00150000": {
      "latestTrade": {
        "t": "2024-01-18T15:30:00Z",
        "x": "C",
        "p": 5.25,
        "s": 10,
        "c": [""]
      },
      "latestQuote": {
        "t": "2024-01-18T15:30:00Z",
        "ax": "C",
        "ap": 5.30,
        "as": 25,
        "bx": "C",
        "bp": 5.20,
        "bs": 15
      },
      "impliedVolatility": 0.32,
      "greeks": {
        "delta": 0.65,
        "gamma": 0.05,
        "theta": -0.08,
        "vega": 0.12,
        "rho": 0.03
      }
    },
    "AAPL250117P00150000": {
      "latestTrade": {...},
      "latestQuote": {...},
      "impliedVolatility": 0.28,
      "greeks": {
        "delta": -0.35,
        "gamma": 0.05,
        "theta": -0.06,
        "vega": 0.10,
        "rho": -0.02
      }
    }
  }
}
```

**Option symbol format:** `{underlying}{YYMMDD}{C/P}{strike with 5 decimals}`
- Example: `AAPL250117C00150000` = AAPL Call expiring Jan 17, 2025, strike $150.00

**Greeks:**
- `delta`: Price change per $1 move in underlying
- `gamma`: Delta change per $1 move
- `theta`: Daily time decay
- `vega`: Price change per 1% IV change
- `rho`: Price change per 1% interest rate change

---

### GET /v1beta3/crypto/us/latest/orderbooks (Crypto Orderbook)

```json
{
  "orderbooks": {
    "BTC/USD": {
      "t": "2024-01-18T15:30:00Z",
      "b": [
        {"p": 45000.00, "s": 1.5},
        {"p": 44995.00, "s": 2.0},
        {"p": 44990.00, "s": 0.8}
      ],
      "a": [
        {"p": 45005.00, "s": 1.2},
        {"p": 45010.00, "s": 1.8},
        {"p": 45015.00, "s": 0.5}
      ]
    }
  }
}
```

**Orderbook fields:**
- `t`: Timestamp
- `b`: Bids array (price, size)
- `a`: Asks array (price, size)
- `p`: Price
- `s`: Size (crypto units)

---

### GET /v1beta1/news (News Articles)

```json
{
  "news": [
    {
      "id": 123456,
      "headline": "Apple Announces Record Earnings",
      "author": "Jane Doe",
      "created_at": "2024-01-18T15:00:00Z",
      "updated_at": "2024-01-18T15:00:00Z",
      "summary": "Apple Inc. reported record quarterly earnings...",
      "content": "<p>Full article content with HTML formatting...</p>",
      "url": "https://www.benzinga.com/news/...",
      "images": [
        {
          "size": "large",
          "url": "https://cdn.benzinga.com/images/..."
        },
        {
          "size": "small",
          "url": "https://cdn.benzinga.com/images/..."
        },
        {
          "size": "thumb",
          "url": "https://cdn.benzinga.com/images/..."
        }
      ],
      "symbols": ["AAPL"],
      "source": "benzinga"
    }
  ],
  "next_page_token": "bmV3c3w..."
}
```

---

### GET /v1beta1/corporate-actions/announcements (Corporate Actions)

```json
{
  "announcements": [
    {
      "id": "abc123",
      "corporate_action_id": "AAPL_DIV_20240315",
      "ca_type": "dividend",
      "ca_sub_type": "cash",
      "initiating_symbol": "AAPL",
      "initiating_original_cusip": "037833100",
      "target_symbol": "AAPL",
      "target_original_cusip": "037833100",
      "declaration_date": "2024-02-01",
      "ex_date": "2024-02-09",
      "record_date": "2024-02-12",
      "payable_date": "2024-02-16",
      "cash": "0.25",
      "old_rate": "1",
      "new_rate": "1"
    },
    {
      "ca_type": "stock_split",
      "ca_sub_type": "forward",
      "initiating_symbol": "TSLA",
      "ex_date": "2024-08-25",
      "old_rate": "1",
      "new_rate": "3",
      "cash": "0"
    }
  ],
  "next_page_token": null
}
```

**Corporate action types:**
- `dividend`: Cash or stock dividend
- `stock_split`: Forward or reverse split
- `merger`: Company merger
- `spinoff`: Spinoff event

---

## Error Responses

### Standard Error Format

```json
{
  "code": 40110000,
  "message": "request not authorized"
}
```

### Validation Error (422)

```json
{
  "code": 42210000,
  "message": "invalid request body",
  "fields": [
    {
      "field": "qty",
      "message": "qty must be a positive number"
    },
    {
      "field": "symbol",
      "message": "symbol is required"
    }
  ]
}
```

### Rate Limit Error (429)

```json
{
  "code": 42910000,
  "message": "rate limit exceeded"
}
```

**HTTP 429 headers:**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 30
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1705599630
```

### Insufficient Funds Error (403)

```json
{
  "code": 40310000,
  "message": "insufficient buying power",
  "buying_power": "1000.00",
  "cost": "15000.00"
}
```

---

## WebSocket Message Formats

See `websocket_full.md` for complete WebSocket message formats, including:
- Trades, Quotes, Bars (minute/daily)
- Trading status, LULD bands, corrections
- Crypto orderbooks
- Trading updates (order fills, cancellations)

All WebSocket messages arrive as **JSON arrays**: `[{...}, {...}]`
