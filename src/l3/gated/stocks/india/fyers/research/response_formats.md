# Fyers - Response Formats

**Note:** All examples below are based on official SDK documentation and community reports. Exact field names and structures are from the Fyers API V3.

---

## General Response Format

### Success Response
```json
{
  "s": "ok",
  "code": 200,
  "data": { ... }
}
```

### Error Response
```json
{
  "s": "error",
  "code": -100,
  "message": "Error description"
}
```

---

## Authentication Endpoints

### POST /api/v3/validate-authcode (Generate Access Token)

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJhcGkuZnllcnMuaW4iLCJpYXQiOjE2NDAwMDAwMDAsImV4cCI6MTY0MDAwMDAwMCwiYXVkIjoiIiwic3ViIjoiQUJDMTIzWFlaLTEwMCJ9.abcdefgh12345678"
}
```

**Fields:**
- `s` - Status ("ok" or "error")
- `code` - HTTP status code
- `access_token` - JWT access token (long string)

---

## User/Account Endpoints

### GET /api/v3/profile

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "data": {
    "fy_id": "ABC123XYZ",
    "name": "John Doe",
    "email_id": "john.doe@example.com",
    "mobile_number": "9876543210",
    "PAN": "ABCDE1234F",
    "access_token": "ABC123XYZ-100:eyJ0eXAi...",
    "display_name": "John Doe",
    "image": "",
    "pwd_to_modify": false
  }
}
```

**Fields:**
- `fy_id` - Fyers client ID
- `name` - Full name
- `email_id` - Email address
- `mobile_number` - Mobile number
- `PAN` - PAN card number
- `access_token` - Current access token
- `display_name` - Display name
- `pwd_to_modify` - Password modification required flag

---

### GET /api/v3/funds

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "fund_limit": [
    {
      "id": "1",
      "title": "Equity",
      "equityAmount": 50000.00,
      "commodityAmount": 0.00,
      "collateralAmount": 0.00,
      "total_balance": 50000.00,
      "withdrawable_balance": 48000.00,
      "used_margin": 2000.00,
      "available_margin": 48000.00
    },
    {
      "id": "2",
      "title": "Commodity",
      "equityAmount": 0.00,
      "commodityAmount": 10000.00,
      "collateralAmount": 0.00,
      "total_balance": 10000.00,
      "withdrawable_balance": 9500.00,
      "used_margin": 500.00,
      "available_margin": 9500.00
    }
  ]
}
```

**Fields (per segment):**
- `id` - Segment ID
- `title` - Segment name (Equity, Commodity)
- `equityAmount` - Equity balance
- `commodityAmount` - Commodity balance
- `collateralAmount` - Collateral amount
- `total_balance` - Total available balance
- `withdrawable_balance` - Amount available for withdrawal
- `used_margin` - Currently used margin
- `available_margin` - Available for trading

---

### GET /api/v3/holdings

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "holdings": [
    {
      "id": 1234567890,
      "fyToken": "101012345",
      "symbol": "NSE:SBIN-EQ",
      "holdingType": "CNC",
      "quantity": 100,
      "remainingQuantity": 100,
      "costPrice": 545.50,
      "marketVal": 55050.00,
      "ltp": 550.50,
      "pl": 500.00,
      "exchange": "NSE",
      "segment": "CM",
      "isin": "INE062A01020",
      "qty_t1": 0,
      "remainingPledgeQuantity": 0,
      "collateralQuantity": 0,
      "collateralValue": 0.00
    }
  ]
}
```

**Fields (per holding):**
- `id` - Holding ID
- `fyToken` - Fyers internal token
- `symbol` - Trading symbol
- `holdingType` - Type of holding (CNC, etc.)
- `quantity` - Total quantity
- `remainingQuantity` - Available quantity (not pledged)
- `costPrice` - Average purchase price
- `marketVal` - Current market value
- `ltp` - Last traded price
- `pl` - Profit/loss (absolute)
- `exchange` - Exchange (NSE, BSE)
- `segment` - Segment (CM, FO)
- `isin` - ISIN code
- `qty_t1` - T1 quantity (settlement pending)
- `remainingPledgeQuantity` - Remaining pledge quantity
- `collateralQuantity` - Pledged quantity
- `collateralValue` - Collateral value

---

## Transaction/Portfolio Endpoints

### GET /api/v3/positions

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "netPositions": [
    {
      "id": "987654321",
      "symbol": "NSE:NIFTY2411921500CE",
      "fyToken": "101012345",
      "side": 1,
      "segment": "FO",
      "netQty": 50,
      "buyQty": 50,
      "sellQty": 0,
      "buyAvg": 150.00,
      "sellAvg": 0.00,
      "netAvg": 150.00,
      "ltp": 155.50,
      "productType": "INTRADAY",
      "buyVal": 7500.00,
      "sellVal": 0.00,
      "pl": 275.00,
      "realized_profit": 0.00,
      "unrealized_profit": 275.00,
      "crossCurrency": "",
      "rbiRefRate": 0
    }
  ],
  "overall": {
    "count_total": 1,
    "count_open": 1,
    "pl_total": 275.00,
    "pl_realized": 0.00,
    "pl_unrealized": 275.00
  }
}
```

**Fields (per position):**
- `id` - Position ID
- `symbol` - Trading symbol
- `fyToken` - Fyers token
- `side` - 1 (long), -1 (short)
- `segment` - Market segment
- `netQty` - Net quantity (buy - sell)
- `buyQty` - Total buy quantity
- `sellQty` - Total sell quantity
- `buyAvg` - Average buy price
- `sellAvg` - Average sell price
- `netAvg` - Net average price
- `ltp` - Last traded price
- `productType` - INTRADAY, CNC, MARGIN
- `buyVal` - Total buy value
- `sellVal` - Total sell value
- `pl` - Total P&L
- `realized_profit` - Realized profit/loss
- `unrealized_profit` - Unrealized profit/loss
- `crossCurrency` - For currency derivatives
- `rbiRefRate` - RBI reference rate

**Overall Summary:**
- `count_total` - Total positions
- `count_open` - Open positions
- `pl_total` - Total P&L
- `pl_realized` - Total realized P&L
- `pl_unrealized` - Total unrealized P&L

---

### GET /api/v3/orderbook

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "orderBook": [
    {
      "id": "ORD123456789",
      "clientId": "ABC123XYZ",
      "orderNumber": "123456789",
      "exchangeOrderNo": "1000000012345",
      "symbol": "NSE:SBIN-EQ",
      "fyToken": "101012345",
      "side": 1,
      "segment": "CM",
      "instrument": "EQ",
      "productType": "INTRADAY",
      "type": 1,
      "orderDateTime": "2026-01-26 09:15:30",
      "orderValidity": "DAY",
      "orderStatus": 6,
      "filledQty": 0,
      "remainingQuantity": 100,
      "qty": 100,
      "limitPrice": 550.00,
      "stopPrice": 0.00,
      "tradedPrice": 0.00,
      "discloseQty": 0,
      "message": "Order placed successfully",
      "offlineOrder": false,
      "orderTag": "",
      "source": "API"
    }
  ]
}
```

**Fields (per order):**
- `id` - Order ID (internal)
- `clientId` - Client ID
- `orderNumber` - Order number
- `exchangeOrderNo` - Exchange order number
- `symbol` - Trading symbol
- `fyToken` - Fyers token
- `side` - 1 (BUY), -1 (SELL)
- `segment` - Market segment
- `instrument` - Instrument type
- `productType` - INTRADAY, CNC, MARGIN, CO, BO
- `type` - 1 (LIMIT), 2 (MARKET), 3 (STOP), 4 (STOPLIMIT)
- `orderDateTime` - Order placement time
- `orderValidity` - DAY, IOC
- `orderStatus` - 1 (cancelled), 2 (traded), 4 (transit), 5 (rejected), 6 (pending), 7 (expired)
- `filledQty` - Filled quantity
- `remainingQuantity` - Pending quantity
- `qty` - Total quantity
- `limitPrice` - Limit price
- `stopPrice` - Stop price
- `tradedPrice` - Average traded price
- `discloseQty` - Disclosed quantity
- `message` - Status message
- `offlineOrder` - AMO flag
- `orderTag` - Custom tag
- `source` - Order source (API, WEB, etc.)

---

### GET /api/v3/tradebook

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "tradeBook": [
    {
      "id": "TRD987654321",
      "orderNumber": "123456789",
      "exchangeOrderNo": "1000000012345",
      "symbol": "NSE:SBIN-EQ",
      "fyToken": "101012345",
      "tradeNumber": "1000000067890",
      "tradeSource": "API",
      "orderDateTime": "2026-01-26 09:15:30",
      "tradeDateTime": "2026-01-26 09:15:35",
      "quantity": 100,
      "tradePrice": 550.50,
      "tradeValue": 55050.00,
      "clientId": "ABC123XYZ",
      "productType": "INTRADAY",
      "exchange": "NSE",
      "segment": "CM",
      "instrument": "EQ",
      "side": 1,
      "type": 2
    }
  ]
}
```

**Fields (per trade):**
- `id` - Trade ID
- `orderNumber` - Parent order number
- `exchangeOrderNo` - Exchange order number
- `symbol` - Trading symbol
- `fyToken` - Fyers token
- `tradeNumber` - Exchange trade number
- `tradeSource` - Trade source
- `orderDateTime` - Order placement time
- `tradeDateTime` - Trade execution time
- `quantity` - Traded quantity
- `tradePrice` - Execution price
- `tradeValue` - Total trade value
- `clientId` - Client ID
- `productType` - Product type
- `exchange` - Exchange
- `segment` - Segment
- `instrument` - Instrument type
- `side` - 1 (BUY), -1 (SELL)
- `type` - Order type

---

## Order Management Endpoints

### POST /api/v3/orders (Place Order)

**Request:**
```json
{
  "symbol": "NSE:SBIN-EQ",
  "qty": 100,
  "type": 2,
  "side": 1,
  "productType": "INTRADAY",
  "limitPrice": 0,
  "stopPrice": 0,
  "validity": "DAY",
  "disclosedQty": 0,
  "offlineOrder": false
}
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "id": "ORD123456789"
}
```

**Fields:**
- `s` - Status
- `code` - Response code
- `id` - Order ID

---

### PUT /api/v3/orders (Modify Order)

**Request:**
```json
{
  "id": "ORD123456789",
  "type": 1,
  "limitPrice": 551.00,
  "qty": 100
}
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "id": "ORD123456789",
  "message": "Order modified successfully"
}
```

---

### DELETE /api/v3/orders (Cancel Order)

**Request:**
```json
{
  "id": "ORD123456789"
}
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "id": "ORD123456789",
  "message": "Order cancelled successfully"
}
```

---

## Market Data Endpoints

### GET /data/quotes

**Request:**
```
GET /data/quotes?symbols=NSE:SBIN-EQ,NSE:RELIANCE-EQ
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "d": [
    {
      "n": "NSE:SBIN-EQ",
      "v": {
        "symbol": "NSE:SBIN-EQ",
        "fyToken": "101012345",
        "description": "STATE BANK OF INDIA",
        "timestamp": 1640000000,
        "exchange": "NSE",
        "segment": "CM",
        "instrument_type": "EQ",
        "lp": 550.50,
        "open_price": 548.00,
        "high_price": 552.00,
        "low_price": 547.50,
        "close_price": 549.00,
        "prev_close_price": 549.00,
        "volume": 1234567,
        "short_name": "SBIN-EQ",
        "original_name": "SBIN-EQ",
        "tt": 1640000000,
        "ch": 1.50,
        "chp": 0.27,
        "bid": 550.45,
        "ask": 550.55,
        "spread": 0.10,
        "bid_size": 100,
        "ask_size": 150,
        "last_traded_qty": 50,
        "last_traded_time": 1640000000,
        "avg_trade_price": 550.25,
        "tot_buy_qty": 50000,
        "tot_sell_qty": 48000,
        "lower_circuit": 520.00,
        "upper_circuit": 580.00,
        "cmd": {
          "t": 0,
          "s": "ok"
        }
      }
    }
  ]
}
```

**Fields (per symbol):**
- `n` - Symbol name
- `v` - Values object:
  - `symbol` - Trading symbol
  - `fyToken` - Fyers token
  - `description` - Full company name
  - `timestamp` - Update timestamp (Unix)
  - `exchange` - Exchange
  - `segment` - Segment
  - `instrument_type` - Instrument type
  - `lp` - Last price (LTP)
  - `open_price`, `high_price`, `low_price`, `close_price` - OHLC
  - `prev_close_price` - Previous close
  - `volume` - Total volume
  - `ch` - Change (points)
  - `chp` - Change percentage
  - `bid`, `ask` - Best bid/ask
  - `spread` - Bid-ask spread
  - `bid_size`, `ask_size` - Bid/ask quantities
  - `last_traded_qty` - Last trade quantity
  - `last_traded_time` - Last trade time
  - `avg_trade_price` - VWAP
  - `tot_buy_qty`, `tot_sell_qty` - Total buy/sell quantities
  - `lower_circuit`, `upper_circuit` - Circuit limits

---

### GET /data/depth/

**Request:**
```
GET /data/depth/?symbol=NSE:SBIN-EQ&ohlcv_flag=1
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "d": {
    "NSE:SBIN-EQ": {
      "totalbuyqty": 45000,
      "totalsellqty": 43000,
      "bids": [
        {"price": 550.45, "volume": 100, "ord": 5},
        {"price": 550.40, "volume": 200, "ord": 8},
        {"price": 550.35, "volume": 150, "ord": 6},
        {"price": 550.30, "volume": 300, "ord": 12},
        {"price": 550.25, "volume": 250, "ord": 10}
      ],
      "ask": [
        {"price": 550.55, "volume": 150, "ord": 7},
        {"price": 550.60, "volume": 180, "ord": 9},
        {"price": 550.65, "volume": 220, "ord": 11},
        {"price": 550.70, "volume": 200, "ord": 8},
        {"price": 550.75, "volume": 300, "ord": 15}
      ],
      "o": 548.00,
      "h": 552.00,
      "l": 547.50,
      "c": 549.00,
      "v": 1234567,
      "lp": 550.50,
      "ch": 1.50,
      "chp": 0.27,
      "prev_close_price": 549.00,
      "timestamp": 1640000000
    }
  }
}
```

**Fields:**
- `totalbuyqty` - Total buy quantity
- `totalsellqty` - Total sell quantity
- `bids` - Array of bid levels (top 5)
  - `price` - Bid price
  - `volume` - Bid quantity
  - `ord` - Number of orders
- `ask` - Array of ask levels (top 5)
  - `price` - Ask price
  - `volume` - Ask quantity
  - `ord` - Number of orders
- OHLCV fields (if ohlcv_flag=1):
  - `o`, `h`, `l`, `c` - OHLC
  - `v` - Volume
  - `lp` - Last price
  - `ch`, `chp` - Change
  - `prev_close_price` - Previous close
  - `timestamp` - Update timestamp

---

### GET /data/history

**Request:**
```
GET /data/history?symbol=NSE:SBIN-EQ&resolution=5&date_format=0&range_from=1640000000&range_to=1640100000
```

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "candles": [
    [1640000000000, 548.00, 550.50, 547.80, 550.00, 12345],
    [1640000300000, 550.00, 551.00, 549.50, 550.50, 13456],
    [1640000600000, 550.50, 552.00, 550.00, 551.50, 14567]
  ]
}
```

**Candles Array Format:**
Each candle is an array: `[timestamp, open, high, low, close, volume]`

**Fields:**
- `[0]` - Timestamp (Unix milliseconds)
- `[1]` - Open
- `[2]` - High
- `[3]` - Low
- `[4]` - Close
- `[5]` - Volume

---

### GET /data/market-status

**Response:**
```json
{
  "s": "ok",
  "code": 200,
  "d": {
    "NSE": {
      "CM": {
        "status": "open",
        "open_time": "09:15:00",
        "close_time": "15:30:00",
        "current_time": "10:30:45"
      },
      "FO": {
        "status": "open",
        "open_time": "09:15:00",
        "close_time": "15:30:00",
        "current_time": "10:30:45"
      },
      "CD": {
        "status": "open",
        "open_time": "09:00:00",
        "close_time": "17:00:00",
        "current_time": "10:30:45"
      }
    },
    "BSE": {
      "CM": {
        "status": "open",
        "open_time": "09:15:00",
        "close_time": "15:30:00",
        "current_time": "10:30:45"
      }
    },
    "MCX": {
      "COMM": {
        "status": "open",
        "open_time": "09:00:00",
        "close_time": "23:30:00",
        "current_time": "10:30:45"
      }
    }
  }
}
```

**Fields (per exchange/segment):**
- `status` - "open" or "closed"
- `open_time` - Market open time
- `close_time` - Market close time
- `current_time` - Current server time

---

## Symbol Master

### GET /data/symbol-master

**Request:**
```
GET /data/symbol-master?exchange=NSE&segment=CM
```

**Response:** CSV file

**CSV Format:**
```csv
fytoken,symbol,exchange,segment,description,lot_size,tick_size,isin,series,expiry_date,strike_price,option_type
101012345,NSE:SBIN-EQ,NSE,CM,STATE BANK OF INDIA,1,0.05,INE062A01020,EQ,,,
101023456,NSE:RELIANCE-EQ,NSE,CM,RELIANCE INDUSTRIES LIMITED,1,0.05,INE002A01018,EQ,,,
101034567,NSE:NIFTY24JANFUT,NSE,FO,NIFTY 24 JAN FUT,50,0.05,,FUTIDX,2024-01-25,,
101045678,NSE:NIFTY2411921500CE,NSE,FO,NIFTY 24 JAN 21500 CE,50,0.05,,OPTIDX,2024-01-25,21500.00,CE
```

**Columns:**
- `fytoken` - Fyers internal token
- `symbol` - Trading symbol
- `exchange` - Exchange (NSE, BSE, MCX, NCDEX)
- `segment` - Segment (CM, FO, CD, COMM)
- `description` - Full name
- `lot_size` - Lot size (1 for cash, 50+ for derivatives)
- `tick_size` - Minimum price movement
- `isin` - ISIN code (equity only)
- `series` - Series (EQ, BE, FUTIDX, OPTIDX, etc.)
- `expiry_date` - Expiry date (derivatives only)
- `strike_price` - Strike price (options only)
- `option_type` - CE or PE (options only)

---

## Error Responses

### Rate Limit Error (429)

```json
{
  "s": "error",
  "code": 429,
  "message": "request limit reached"
}
```

---

### Authentication Error (401)

```json
{
  "s": "error",
  "code": -1600,
  "message": "Could not authenticate the user"
}
```

---

### Invalid Symbol Error

```json
{
  "s": "error",
  "code": -100,
  "message": "Invalid symbol format"
}
```

---

### WebSocket Subscription Error

```json
{
  "s": "error",
  "code": -351,
  "message": "You have provided symbols greater than 50"
}
```

---

## WebSocket Message Formats

### Data WebSocket - Symbol Update (Full Mode)

```json
{
  "type": "sf",
  "symbol": "NSE:SBIN-EQ",
  "timestamp": 1640000000000,
  "fytoken": "101012345",
  "ltp": 550.50,
  "open_price": 548.00,
  "high_price": 552.00,
  "low_price": 547.50,
  "close_price": 549.00,
  "prev_close_price": 549.00,
  "volume": 1234567,
  "chp": 1.50,
  "ch": 0.27,
  "bid_price": 550.45,
  "ask_price": 550.55,
  "bid_size": 100,
  "ask_size": 150,
  "last_traded_qty": 50,
  "last_traded_time": 1640000000,
  "avg_trade_price": 550.25,
  "tot_buy_qty": 50000,
  "tot_sell_qty": 48000
}
```

---

### Data WebSocket - Lite Mode (LTP Only)

```json
{
  "type": "ltp",
  "symbol": "NSE:SBIN-EQ",
  "ltp": 550.50,
  "timestamp": 1640000000000
}
```

---

### Data WebSocket - Depth Update

```json
{
  "type": "dp",
  "symbol": "NSE:SBIN-EQ",
  "timestamp": 1640000000000,
  "bids": [
    [550.45, 100, 5],
    [550.40, 200, 8],
    [550.35, 150, 6],
    [550.30, 300, 12],
    [550.25, 250, 10]
  ],
  "asks": [
    [550.55, 150, 7],
    [550.60, 180, 9],
    [550.65, 220, 11],
    [550.70, 200, 8],
    [550.75, 300, 15]
  ],
  "totalbuyqty": 45000,
  "totalsellqty": 43000
}
```

**Bids/Asks Format:** `[price, volume, order_count]`

---

### Order WebSocket - Order Update

```json
{
  "type": "order",
  "id": "ORD123456789",
  "symbol": "NSE:SBIN-EQ",
  "type": 1,
  "side": 1,
  "status": 6,
  "qty": 100,
  "filledQty": 0,
  "remainingQty": 100,
  "limitPrice": 550.00,
  "stopPrice": 0,
  "productType": "INTRADAY",
  "orderDateTime": "2026-01-26 09:15:30",
  "orderUpdateTime": "2026-01-26 09:15:30",
  "message": "Order placed successfully"
}
```

---

### Order WebSocket - Trade Update

```json
{
  "type": "trade",
  "tradeId": "TRD987654321",
  "orderId": "ORD123456789",
  "symbol": "NSE:SBIN-EQ",
  "side": 1,
  "qty": 100,
  "tradePrice": 550.50,
  "tradeTime": 1640000000000,
  "productType": "INTRADAY",
  "exchange": "NSE",
  "segment": "CM"
}
```

---

### Order WebSocket - Position Update

```json
{
  "type": "position",
  "symbol": "NSE:SBIN-EQ",
  "side": "LONG",
  "netQty": 100,
  "avgPrice": 550.50,
  "buyQty": 100,
  "sellQty": 0,
  "buyAvg": 550.50,
  "sellAvg": 0,
  "realizedProfit": 0,
  "unrealizedProfit": 50.00,
  "productType": "INTRADAY"
}
```

---

## Notes

1. **All timestamps in Unix milliseconds** (unless otherwise noted)
2. **Prices are float values** (not strings)
3. **Quantities are integers**
4. **Status code 200 = success**, other codes = error
5. **"s" field:** "ok" or "error"
6. **Order types:** 1=LIMIT, 2=MARKET, 3=STOP, 4=STOPLIMIT
7. **Side:** 1=BUY, -1=SELL
8. **Order status:** 1=cancelled, 2=traded, 4=transit, 5=rejected, 6=pending, 7=expired
9. **Symbol format:** EXCHANGE:SYMBOL-SERIES (e.g., NSE:SBIN-EQ)
10. **WebSocket messages are dictionaries** (Python) or objects (JavaScript)
