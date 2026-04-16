# Dhan - Response Formats

## Standard Response Envelope

### Success Response
```json
{
  "status": "success",
  "data": { ... }
}
```

### Error Response
```json
{
  "errorType": "ValidationError",
  "errorCode": "OR4001",
  "errorMessage": "Invalid order parameters"
}
```

## Trading Endpoints

### POST /v2/orders (Place Order)

**Success Response**:
```json
{
  "orderId": "112211220001",
  "orderStatus": "PENDING",
  "remarks": "Order placed successfully"
}
```

**Fields**:
- `orderId` (string): Unique order ID assigned by Dhan
- `orderStatus` (string): PENDING, TRANSIT, REJECTED
- `remarks` (string): Additional information

### GET /v2/orders (Get Order Book)

**Response**:
```json
[
  {
    "orderId": "112211220001",
    "exchangeOrderId": "1000000012345",
    "dhanClientId": "1000000123",
    "orderStatus": "TRADED",
    "transactionType": "BUY",
    "exchangeSegment": "NSE_EQ",
    "productType": "INTRADAY",
    "orderType": "LIMIT",
    "validity": "DAY",
    "tradingSymbol": "RELIANCE",
    "securityId": "1333",
    "quantity": 10,
    "disclosedQuantity": 0,
    "price": 2500.00,
    "triggerPrice": 0.00,
    "afterMarketOrder": false,
    "boProfitValue": 0.00,
    "boStopLossValue": 0.00,
    "legName": "",
    "createTime": "2024-01-15 09:30:00",
    "updateTime": "2024-01-15 09:30:15",
    "exchangeTime": "2024-01-15 09:30:10",
    "drvExpiryDate": null,
    "drvOptionType": null,
    "drvStrikePrice": 0.00,
    "omsErrorCode": "",
    "omsErrorDescription": "",
    "filled": 10,
    "algoId": "",
    "remarks": "Order executed"
  }
]
```

**Field Descriptions**:
- `orderId`: Dhan's internal order ID
- `exchangeOrderId`: Exchange-assigned order ID
- `orderStatus`: PENDING, TRANSIT, REJECTED, CANCELLED, TRADED, EXPIRED
- `transactionType`: BUY, SELL
- `exchangeSegment`: NSE_EQ, NSE_FNO, BSE_EQ, MCX_COMM
- `productType`: CNC, INTRADAY, MARGIN, MTF, CO, BO
- `orderType`: MARKET, LIMIT, STOP_LOSS, STOP_LOSS_MARKET
- `validity`: DAY, IOC
- `filled`: Filled quantity
- `quantity`: Total quantity

### GET /v2/orders/{order-id} (Single Order)

**Response** (same structure as order book, single object):
```json
{
  "orderId": "112211220001",
  "exchangeOrderId": "1000000012345",
  "orderStatus": "TRADED",
  "transactionType": "BUY",
  "exchangeSegment": "NSE_EQ",
  "productType": "INTRADAY",
  "orderType": "LIMIT",
  "tradingSymbol": "RELIANCE",
  "securityId": "1333",
  "quantity": 10,
  "filled": 10,
  "price": 2500.00,
  "createTime": "2024-01-15 09:30:00",
  "updateTime": "2024-01-15 09:30:15"
}
```

### PUT /v2/orders/{order-id} (Modify Order)

**Success Response**:
```json
{
  "orderId": "112211220001",
  "orderStatus": "PENDING",
  "remarks": "Order modified successfully"
}
```

### DELETE /v2/orders/{order-id} (Cancel Order)

**Success Response**:
```json
{
  "orderId": "112211220001",
  "orderStatus": "CANCELLED",
  "remarks": "Order cancelled successfully"
}
```

### POST /v2/super/orders (Place Super Order)

**Success Response**:
```json
{
  "orderId": "112211220001",
  "orderStatus": "PENDING",
  "remarks": "Super order placed successfully"
}
```

### GET /v2/super/orders (Super Order Book)

**Response**:
```json
[
  {
    "orderId": "112211220001",
    "dhanClientId": "1000000123",
    "orderStatus": "TRADED",
    "transactionType": "BUY",
    "exchangeSegment": "NSE_EQ",
    "productType": "INTRADAY",
    "tradingSymbol": "RELIANCE",
    "securityId": "1333",
    "quantity": 10,
    "price": 2500.00,
    "targetPrice": 2550.00,
    "stopLossPrice": 2475.00,
    "trailingJump": 5.00,
    "entryLegStatus": "TRADED",
    "targetLegStatus": "PENDING",
    "stopLossLegStatus": "PENDING",
    "createTime": "2024-01-15 09:30:00"
  }
]
```

### GET /v2/trades/{order-id} (Trades by Order)

**Response**:
```json
[
  {
    "orderId": "112211220001",
    "exchangeOrderId": "1000000012345",
    "tradeId": "11221122000101",
    "exchangeTradeId": "1234567890123",
    "transactionType": "BUY",
    "exchangeSegment": "NSE_EQ",
    "productType": "INTRADAY",
    "tradingSymbol": "RELIANCE",
    "securityId": "1333",
    "tradedQuantity": 10,
    "tradedPrice": 2500.00,
    "tradeTime": "2024-01-15 09:30:10",
    "charges": 15.50
  }
]
```

### GET /v2/trades/{from-date}/{to-date}/{page} (Trade History)

**Response**:
```json
{
  "trades": [
    {
      "orderId": "112211220001",
      "tradeId": "11221122000101",
      "transactionType": "BUY",
      "tradingSymbol": "RELIANCE",
      "tradedQuantity": 10,
      "tradedPrice": 2500.00,
      "tradeTime": "2024-01-15 09:30:10"
    }
  ],
  "page": 1,
  "totalPages": 5,
  "totalRecords": 123
}
```

## Portfolio Endpoints

### GET /v2/holdings

**Response**:
```json
[
  {
    "dhanClientId": "1000000123",
    "tradingSymbol": "RELIANCE",
    "securityId": "1333",
    "exchangeSegment": "NSE_EQ",
    "isin": "INE002A01018",
    "totalQuantity": 100,
    "t1Quantity": 10,
    "deliveredQuantity": 90,
    "collateralQuantity": 50,
    "averagePrice": 2400.00,
    "currentPrice": 2500.00,
    "pnl": 10000.00,
    "pnlPercentage": 4.17
  }
]
```

**Field Descriptions**:
- `totalQuantity`: Total holdings
- `t1Quantity`: T+1 day quantity (not yet delivered)
- `deliveredQuantity`: Fully delivered quantity
- `collateralQuantity`: Quantity pledged as collateral
- `averagePrice`: Average buy price
- `currentPrice`: Current market price
- `pnl`: Profit/Loss in rupees
- `pnlPercentage`: P&L percentage

### GET /v2/positions

**Response**:
```json
[
  {
    "dhanClientId": "1000000123",
    "tradingSymbol": "NIFTY24FEB24000CE",
    "securityId": "52175",
    "positionType": "LONG",
    "exchangeSegment": "NSE_FNO",
    "productType": "INTRADAY",
    "buyAvg": 250.00,
    "buyQty": 50,
    "sellAvg": 0.00,
    "sellQty": 0,
    "netQty": 50,
    "realizedProfit": 0.00,
    "unrealizedProfit": 2500.00,
    "currentPrice": 300.00,
    "dayBuyAvg": 250.00,
    "dayBuyQty": 50,
    "daySellAvg": 0.00,
    "daySellQty": 0,
    "drvExpiryDate": "2024-02-29",
    "drvOptionType": "CALL",
    "drvStrikePrice": 24000.00
  }
]
```

**Field Descriptions**:
- `positionType`: LONG, SHORT
- `netQty`: Net position (buy - sell)
- `realizedProfit`: Closed position P&L
- `unrealizedProfit`: Open position P&L (marked to market)
- `dayBuyAvg`, `dayBuyQty`: Intraday buy average and quantity
- `daySellAvg`, `daySellQty`: Intraday sell average and quantity

### POST /v2/positions/convert

**Success Response**:
```json
{
  "status": "success",
  "remarks": "Position converted successfully"
}
```

## Funds Endpoints

### GET /v2/funds

**Response**:
```json
{
  "dhanClientId": "1000000123",
  "availableBalance": 500000.00,
  "sodLimit": 1000000.00,
  "collateralAmount": 200000.00,
  "utilizedAmount": 500000.00,
  "blockedPayoutAmount": 0.00,
  "withdrawableBalance": 500000.00
}
```

**Field Descriptions**:
- `availableBalance`: Available for trading
- `sodLimit`: Start-of-day limit (opening balance + collateral)
- `collateralAmount`: Value from pledged holdings
- `utilizedAmount`: Used margin
- `blockedPayoutAmount`: Blocked for payouts
- `withdrawableBalance`: Can be withdrawn

### GET /v2/ledger

**Response**:
```json
[
  {
    "date": "2024-01-15",
    "narration": "Trade Settlement - RELIANCE",
    "voucherType": "Trade",
    "debit": 0.00,
    "credit": 25000.00,
    "balance": 525000.00
  },
  {
    "date": "2024-01-14",
    "narration": "Trade Settlement - TCS",
    "voucherType": "Trade",
    "debit": 35000.00,
    "credit": 0.00,
    "balance": 500000.00
  }
]
```

## Market Data Endpoints

### POST /v2/marketfeed/ltp (Last Traded Price)

**Request**:
```json
{
  "NSE_EQ": ["1333", "11536"],
  "NSE_FNO": ["52175"]
}
```

**Response**:
```json
{
  "NSE_EQ": [
    {
      "securityId": "1333",
      "tradingSymbol": "RELIANCE",
      "LTP": 2500.00,
      "LTT": "15:30:00"
    },
    {
      "securityId": "11536",
      "tradingSymbol": "TCS",
      "LTP": 3600.00,
      "LTT": "15:30:05"
    }
  ],
  "NSE_FNO": [
    {
      "securityId": "52175",
      "tradingSymbol": "NIFTY24FEB24000CE",
      "LTP": 300.00,
      "LTT": "15:30:10"
    }
  ]
}
```

**Field Descriptions**:
- `LTP`: Last Traded Price
- `LTT`: Last Traded Time (HH:MM:SS)

### POST /v2/marketfeed/ohlc (OHLC + LTP)

**Response**:
```json
{
  "NSE_EQ": [
    {
      "securityId": "1333",
      "tradingSymbol": "RELIANCE",
      "open": 2480.00,
      "high": 2520.00,
      "low": 2475.00,
      "close": 2495.00,
      "LTP": 2500.00,
      "LTT": "15:30:00",
      "volume": 12345678
    }
  ]
}
```

**Field Descriptions**:
- `open`: Day's opening price
- `high`: Day's high
- `low`: Day's low
- `close`: Previous day's close
- `LTP`: Current price
- `volume`: Day's total volume

### POST /v2/marketfeed/quote (Full Quote + Depth)

**Response**:
```json
{
  "NSE_EQ": [
    {
      "securityId": "1333",
      "tradingSymbol": "RELIANCE",
      "open": 2480.00,
      "high": 2520.00,
      "low": 2475.00,
      "close": 2495.00,
      "LTP": 2500.00,
      "LTT": "15:30:00",
      "LTQ": 10,
      "volume": 12345678,
      "totalBuyQty": 500000,
      "totalSellQty": 480000,
      "averagePrice": 2497.50,
      "OI": 0,
      "prevOI": 0,
      "depth": {
        "buy": [
          {"price": 2499.95, "quantity": 100, "orders": 5},
          {"price": 2499.90, "quantity": 200, "orders": 8},
          {"price": 2499.85, "quantity": 150, "orders": 6},
          {"price": 2499.80, "quantity": 300, "orders": 12},
          {"price": 2499.75, "quantity": 250, "orders": 10}
        ],
        "sell": [
          {"price": 2500.00, "quantity": 120, "orders": 6},
          {"price": 2500.05, "quantity": 180, "orders": 7},
          {"price": 2500.10, "quantity": 160, "orders": 8},
          {"price": 2500.15, "quantity": 220, "orders": 9},
          {"price": 2500.20, "quantity": 200, "orders": 11}
        ]
      }
    }
  ]
}
```

**Field Descriptions**:
- `LTQ`: Last Traded Quantity
- `totalBuyQty`: Total buy quantity in orderbook
- `totalSellQty`: Total sell quantity in orderbook
- `averagePrice`: Weighted average price
- `OI`: Open Interest (for derivatives, 0 for equity)
- `prevOI`: Previous day's OI
- `depth.buy`: 5 best bid levels
- `depth.sell`: 5 best ask levels

### POST /v2/charts/historical (Daily Historical Data)

**Request**:
```json
{
  "securityId": "1333",
  "exchangeSegment": "NSE_EQ",
  "instrument": "EQUITY",
  "fromDate": "2023-01-01",
  "toDate": "2023-12-31"
}
```

**Response**:
```json
[
  {
    "timestamp": 1672531200000,
    "open": 2400.00,
    "high": 2425.00,
    "low": 2390.00,
    "close": 2410.00,
    "volume": 5678901
  },
  {
    "timestamp": 1672617600000,
    "open": 2415.00,
    "high": 2440.00,
    "low": 2405.00,
    "close": 2430.00,
    "volume": 6234567
  }
]
```

**Field Descriptions**:
- `timestamp`: Unix timestamp in milliseconds
- `open`, `high`, `low`, `close`: OHLC prices
- `volume`: Trading volume

### POST /v2/charts/intraday (Intraday Historical Data)

**Request**:
```json
{
  "securityId": "52175",
  "exchangeSegment": "NSE_FNO",
  "instrument": "OPTIDX",
  "interval": "5",
  "oi": true,
  "fromDate": "2024-01-01",
  "toDate": "2024-01-15"
}
```

**Response** (with OI for derivatives):
```json
[
  {
    "timestamp": 1704096000000,
    "open": 280.00,
    "high": 285.00,
    "low": 278.00,
    "close": 283.00,
    "volume": 15000,
    "open_interest": 1250000
  },
  {
    "timestamp": 1704096300000,
    "open": 283.00,
    "high": 287.00,
    "low": 282.00,
    "close": 286.00,
    "volume": 18000,
    "open_interest": 1255000
  }
]
```

**Field Descriptions**:
- `interval`: 1, 5, 15, 25, or 60 (minutes)
- `open_interest`: OI at end of interval (only if `oi: true` in request)

### POST /v2/optionchain (Option Chain)

**Request**:
```json
{
  "securityId": "13",
  "exchangeSegment": "NSE_FNO",
  "expiryDate": "2024-02-29"
}
```

**Response**:
```json
{
  "expiryDate": "2024-02-29",
  "underlyingSecurityId": "13",
  "underlyingSymbol": "NIFTY",
  "underlyingLTP": 21500.00,
  "optionData": [
    {
      "strikePrice": 21000.00,
      "call": {
        "securityId": "52100",
        "tradingSymbol": "NIFTY24FEB21000CE",
        "LTP": 550.00,
        "volume": 125000,
        "OI": 5000000,
        "iv": 15.5,
        "delta": 0.85,
        "gamma": 0.0012,
        "theta": -8.5,
        "vega": 12.3,
        "bidPrice": 549.50,
        "bidQty": 50,
        "askPrice": 550.50,
        "askQty": 75
      },
      "put": {
        "securityId": "52101",
        "tradingSymbol": "NIFTY24FEB21000PE",
        "LTP": 15.00,
        "volume": 80000,
        "OI": 3500000,
        "iv": 16.2,
        "delta": -0.15,
        "gamma": 0.0008,
        "theta": -2.1,
        "vega": 8.5,
        "bidPrice": 14.75,
        "bidQty": 100,
        "askPrice": 15.25,
        "askQty": 120
      }
    },
    {
      "strikePrice": 21500.00,
      "call": { ... },
      "put": { ... }
    }
  ]
}
```

**Field Descriptions**:
- `iv`: Implied Volatility (%)
- `delta`: Option delta
- `gamma`: Option gamma
- `theta`: Option theta (time decay)
- `vega`: Option vega (volatility sensitivity)
- `OI`: Open Interest (contracts)

### GET /v2/instrument/{exchangeSegment} (Instrument List)

**Response** (CSV format, not JSON):
```csv
SEM_EXM_EXCH_ID,SEM_SEGMENT,SEM_TRADING_SYMBOL,SEM_CUSTOM_SYMBOL,SM_SYMBOL_NAME,SEM_INSTRUMENT_NAME,SEM_EXPIRY_DATE,SM_KEY,SEM_STRIKE_PRICE,SEM_OPTION_TYPE,SEM_LOT_UNITS,SM_ISIN,SEM_EXCH_INSTRUMENT_TYPE,SEM_TICK_SIZE
NSE,EQ,RELIANCE-EQ,RELIANCE,RELIANCE INDUSTRIES LTD.,EQUITY,,1333,0,,1,INE002A01018,C,0.05
NSE,EQ,TCS-EQ,TCS,TATA CONSULTANCY SERVICES LTD.,EQUITY,,11536,0,,1,INE467B01029,C,0.05
NSE,FO,NIFTY24FEB21000CE,NIFTY24FEB21000CE,NIFTY,OPTIDX,29-FEB-2024,52100,21000,CALL,50,,D,0.05
```

**Headers**:
- `SM_KEY`: Security ID (use this in API calls)
- `SEM_TRADING_SYMBOL`: Trading symbol
- `SM_SYMBOL_NAME`: Company/Index name
- `SEM_EXPIRY_DATE`: Expiry (for derivatives)
- `SEM_STRIKE_PRICE`: Strike price (for options)
- `SEM_OPTION_TYPE`: CALL or PUT
- `SEM_LOT_UNITS`: Lot size
- `SM_ISIN`: ISIN code

## EDIS Endpoints

### POST /v2/edis/tpin

**Success Response**:
```json
{
  "status": "success",
  "remarks": "T-PIN sent to registered mobile number"
}
```

### POST /v2/edis/form

**Response** (escaped HTML):
```json
{
  "edisFormHtml": "<form action=\"https://edis.cdsl.com/...\" method=\"POST\">...</form>"
}
```

### POST /v2/edis/inquiry

**Response**:
```json
[
  {
    "isin": "INE002A01018",
    "quantity": 100,
    "status": "APPROVED",
    "requestDate": "2024-01-15",
    "approvalDate": "2024-01-15"
  }
]
```

## Error Responses

### Validation Error
```json
{
  "errorType": "ValidationError",
  "errorCode": "OR4001",
  "errorMessage": "Invalid security ID"
}
```

### Authentication Error
```json
{
  "errorType": "AuthenticationError",
  "errorCode": "AS4001",
  "errorMessage": "Invalid access token"
}
```

### Rate Limit Error
```json
{
  "errorType": "RateLimitError",
  "errorCode": "RL4001",
  "errorMessage": "Rate limit exceeded"
}
```

### Insufficient Funds
```json
{
  "errorType": "InsufficientFundsError",
  "errorCode": "FN4001",
  "errorMessage": "Insufficient funds to place order"
}
```

### Order Rejection
```json
{
  "errorType": "OrderRejection",
  "errorCode": "OR4002",
  "errorMessage": "Order rejected by exchange",
  "omsErrorCode": "17070",
  "omsErrorDescription": "Price out of circuit limits"
}
```

## Common Field Types

### Timestamps
- **Format**: Unix timestamp in milliseconds (e.g., 1672531200000)
- **OR**: String format "YYYY-MM-DD HH:MM:SS" (e.g., "2024-01-15 09:30:00")

### Dates
- **Format**: "YYYY-MM-DD" (e.g., "2024-01-15")

### Prices
- **Type**: Float (e.g., 2500.00)
- **Precision**: 2 decimal places typically

### Quantities
- **Type**: Integer for stocks (e.g., 10)
- **Type**: Integer for F&O (lot-based, e.g., 50 = 1 lot of Nifty options)

### Enums

**Order Status**:
- `PENDING` - Order placed, awaiting exchange
- `TRANSIT` - Order sent to exchange
- `REJECTED` - Order rejected
- `CANCELLED` - Order cancelled
- `TRADED` - Order executed fully
- `EXPIRED` - Order expired (end of day or validity)

**Transaction Type**:
- `BUY`
- `SELL`

**Product Type**:
- `CNC` - Cash and Carry (delivery)
- `INTRADAY` - Intraday/MIS
- `MARGIN` - Margin delivery
- `MTF` - Margin Trading Facility
- `CO` - Cover Order
- `BO` - Bracket Order

**Order Type**:
- `MARKET` - Market order
- `LIMIT` - Limit order
- `STOP_LOSS` - Stop loss limit order
- `STOP_LOSS_MARKET` - Stop loss market order

**Validity**:
- `DAY` - Valid for the day
- `IOC` - Immediate or Cancel

**Exchange Segment**:
- `NSE_EQ` - NSE Equity
- `NSE_FNO` - NSE F&O
- `BSE_EQ` - BSE Equity
- `MCX_COMM` - MCX Commodities

**Instrument Type**:
- `EQUITY` - Equity stock
- `FUTIDX` - Index future
- `FUTSTK` - Stock future
- `OPTIDX` - Index option
- `OPTSTK` - Stock option

## Notes

1. All REST API responses are JSON (except instrument list which is CSV)
2. WebSocket responses are BINARY (Little Endian), not JSON
3. Timestamps in WebSocket are Unix milliseconds
4. All prices in INR (Indian Rupees)
5. Option Greeks may be `null` if not calculated
6. Some fields may be `null` or empty string if not applicable
7. Error responses do NOT include HTTP 2xx status codes (they return 4xx or 5xx)
