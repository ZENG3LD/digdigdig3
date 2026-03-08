# Angel One SmartAPI - Response Formats

This document contains EXACT JSON response examples from official documentation and SDK code.

## General Response Structure

All Angel One SmartAPI responses follow a standard envelope:

```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": { /* actual response data */ }
}
```

**Error Response**:
```json
{
  "status": false,
  "message": "Error description",
  "errorcode": "AB1004",
  "data": null
}
```

## Authentication & Session Management

### POST /rest/auth/angelbroking/user/v1/loginByPassword
**Generate Session (Login)**

Request:
```json
{
  "clientcode": "A12345",
  "password": "1234",
  "totp": "123456"
}
```

Response:
```json
{
  "status": true,
  "message": "User Logged In Successfully",
  "errorcode": "",
  "data": {
    "jwtToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6IkExMjM0NSIsImlhdCI6MTYxMjM0NTY3OCwiZXhwIjoxNjEyMzg4ODc4fQ.abcdef123456",
    "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6IkExMjM0NSIsInR5cGUiOiJyZWZyZXNoIiwiaWF0IjoxNjEyMzQ1Njc4fQ.ghijkl789012",
    "feedToken": "1234567890"
  }
}
```

### POST /rest/auth/angelbroking/jwt/v1/generateTokens
**Refresh Token**

Request:
```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

Response:
```json
{
  "status": true,
  "message": "Token Generated Successfully",
  "errorcode": "",
  "data": {
    "jwtToken": "new_jwt_token_here",
    "refreshToken": "new_refresh_token_here"
  }
}
```

### GET /rest/secure/angelbroking/user/v1/getProfile
**Get User Profile**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "clientcode": "A12345",
    "name": "JOHN DOE",
    "email": "john.doe@example.com",
    "mobileno": "9876543210",
    "exchanges": ["NSE", "BSE", "NFO", "MCX"],
    "products": ["CNC", "NRML", "MIS", "BO"],
    "lastlogintime": "26-Jan-2026 09:15:30",
    "broker": "ANGELONE"
  }
}
```

### POST /rest/secure/angelbroking/user/v1/logout
**Logout**

Request:
```json
{
  "clientId": "A12345"
}
```

Response:
```json
{
  "status": true,
  "message": "User Logged Out Successfully",
  "errorcode": "",
  "data": null
}
```

## Market Data - Real-time

### POST /rest/secure/angelbroking/market/v1/quote/
**Get Quote (LTP Mode)**

Request:
```json
{
  "mode": "LTP",
  "exchangeTokens": {
    "NSE": ["3045", "1594"]
  }
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "fetched": [
      {
        "exchange": "NSE",
        "tradingSymbol": "SBIN-EQ",
        "symbolToken": "3045",
        "ltp": 500.25,
        "close": 498.50
      },
      {
        "exchange": "NSE",
        "tradingSymbol": "INFY-EQ",
        "symbolToken": "1594",
        "ltp": 1450.75,
        "close": 1445.20
      }
    ]
  }
}
```

### POST /rest/secure/angelbroking/market/v1/quote/
**Get Quote (OHLC Mode)**

Request:
```json
{
  "mode": "OHLC",
  "exchangeTokens": {
    "NSE": ["3045"]
  }
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "fetched": [
      {
        "exchange": "NSE",
        "tradingSymbol": "SBIN-EQ",
        "symbolToken": "3045",
        "open": 495.00,
        "high": 502.50,
        "low": 493.75,
        "close": 498.50,
        "ltp": 500.25
      }
    ]
  }
}
```

### POST /rest/secure/angelbroking/market/v1/quote/
**Get Quote (FULL Mode)**

Request:
```json
{
  "mode": "FULL",
  "exchangeTokens": {
    "NSE": ["3045"]
  }
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "fetched": [
      {
        "exchange": "NSE",
        "tradingSymbol": "SBIN-EQ",
        "symbolToken": "3045",
        "open": 495.00,
        "high": 502.50,
        "low": 493.75,
        "close": 498.50,
        "ltp": 500.25,
        "bidprice": 500.20,
        "bidqty": 5000,
        "askprice": 500.30,
        "askqty": 3500,
        "volume": 12456789,
        "exchange_timestamp": "26-Jan-2026 15:30:00",
        "avgprice": 498.75,
        "opninterest": 0,
        "upperCircuitLimit": 548.35,
        "lowerCircuitLimit": 448.65,
        "52weekhigh": 550.00,
        "52weeklow": 420.00
      }
    ]
  }
}
```

## Market Data - Historical

### POST /rest/secure/angelbroking/historical/v1/getCandleData
**Get Historical Candles**

Request:
```json
{
  "exchange": "NSE",
  "symboltoken": "3045",
  "interval": "ONE_DAY",
  "fromdate": "2026-01-01 09:15",
  "todate": "2026-01-26 15:30"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    ["2026-01-02T09:15:00+05:30", 495.00, 502.50, 493.75, 500.25, 12456789],
    ["2026-01-03T09:15:00+05:30", 500.50, 508.00, 498.25, 505.75, 13567890],
    ["2026-01-06T09:15:00+05:30", 505.00, 510.50, 502.00, 508.25, 11234567]
  ]
}
```

**Data Format**: `[timestamp, open, high, low, close, volume]`

**Note**: Timestamps include timezone offset (+05:30 for IST).

### POST /rest/secure/angelbroking/historical/v1/getCandleData
**Intraday Candles (1 Minute)**

Request:
```json
{
  "exchange": "NSE",
  "symboltoken": "3045",
  "interval": "ONE_MINUTE",
  "fromdate": "2026-01-26 09:15",
  "todate": "2026-01-26 15:30"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    ["2026-01-26T09:15:00+05:30", 500.00, 500.50, 499.80, 500.25, 125678],
    ["2026-01-26T09:16:00+05:30", 500.25, 500.75, 500.10, 500.60, 98456],
    ["2026-01-26T09:17:00+05:30", 500.60, 501.00, 500.40, 500.85, 112345]
  ]
}
```

## Metadata & Search

### Instrument Master File
**GET https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json**

Response (array of instruments):
```json
[
  {
    "token": "3045",
    "symbol": "SBIN-EQ",
    "name": "STATE BANK OF INDIA",
    "expiry": "",
    "strike": "-1.000000",
    "lotsize": "1",
    "instrumenttype": "",
    "exch_seg": "NSE",
    "tick_size": "5.000000"
  },
  {
    "token": "26000",
    "symbol": "NIFTY26FEB24000CE",
    "name": "NIFTY",
    "expiry": "27FEB2026",
    "strike": "24000.000000",
    "lotsize": "25",
    "instrumenttype": "OPTIDX",
    "exch_seg": "NFO",
    "tick_size": "5.000000"
  },
  {
    "token": "234567",
    "symbol": "SBIN26FEBFUT",
    "name": "SBIN",
    "expiry": "27FEB2026",
    "strike": "-1.000000",
    "lotsize": "1500",
    "instrumenttype": "FUTIDX",
    "exch_seg": "NFO",
    "tick_size": "5.000000"
  }
]
```

### POST /rest/secure/angelbroking/order/v1/searchScrip
**Search Scrip**

Request:
```json
{
  "exchange": "NSE",
  "searchscrip": "SBIN"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    {
      "exchange": "NSE",
      "tradingSymbol": "SBIN-EQ",
      "symbolToken": "3045",
      "instrumentName": "STATE BANK OF INDIA",
      "instrumentType": "EQ"
    }
  ]
}
```

## Order Management

### POST /rest/secure/angelbroking/order/v1/placeOrder
**Place Order (Simple Response)**

Request:
```json
{
  "variety": "NORMAL",
  "tradingsymbol": "SBIN-EQ",
  "symboltoken": "3045",
  "transactiontype": "BUY",
  "exchange": "NSE",
  "ordertype": "LIMIT",
  "producttype": "DELIVERY",
  "duration": "DAY",
  "price": "500.00",
  "squareoff": "0",
  "stoploss": "0",
  "quantity": "10"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "orderid": "241126000012345"
  }
}
```

### POST /rest/secure/angelbroking/order/v1/placeOrderFullResponse
**Place Order (Full Response)**

Request: (same as placeOrder)

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "script": "SBIN-EQ",
    "orderid": "241126000012345",
    "uniqueorderid": "A12345-SBIN-EQ-241126000012345",
    "tradingsymbol": "SBIN-EQ",
    "transactiontype": "BUY",
    "exchange": "NSE",
    "producttype": "DELIVERY",
    "ordertype": "LIMIT",
    "price": 500.00,
    "quantity": 10,
    "status": "open",
    "ordertag": "",
    "text": "Order placed successfully",
    "symboltoken": "3045",
    "ordertimestamp": "26-Jan-2026 10:15:30"
  }
}
```

### POST /rest/secure/angelbroking/order/v1/modifyOrder
**Modify Order**

Request:
```json
{
  "variety": "NORMAL",
  "orderid": "241126000012345",
  "ordertype": "LIMIT",
  "producttype": "DELIVERY",
  "duration": "DAY",
  "price": "505.00",
  "quantity": "15",
  "tradingsymbol": "SBIN-EQ",
  "symboltoken": "3045",
  "exchange": "NSE"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "orderid": "241126000012345"
  }
}
```

### POST /rest/secure/angelbroking/order/v1/cancelOrder
**Cancel Order**

Request:
```json
{
  "variety": "NORMAL",
  "orderid": "241126000012345"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "orderid": "241126000012345"
  }
}
```

### GET /rest/secure/angelbroking/order/v1/getOrderBook
**Get Order Book**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    {
      "variety": "NORMAL",
      "ordertype": "LIMIT",
      "producttype": "DELIVERY",
      "duration": "DAY",
      "price": 500.00,
      "triggerprice": 0.00,
      "quantity": "10",
      "disclosedquantity": "0",
      "squareoff": 0.00,
      "stoploss": 0.00,
      "trailingstoploss": 0.00,
      "tradingsymbol": "SBIN-EQ",
      "transactiontype": "BUY",
      "exchange": "NSE",
      "symboltoken": "3045",
      "instrumenttype": "",
      "strikeprice": -1.00,
      "optiontype": "",
      "expirydate": "",
      "lotsize": "1",
      "cancelsize": "0",
      "averageprice": 500.25,
      "filledshares": "10",
      "unfilledshares": "0",
      "orderid": "241126000012345",
      "text": "Order executed",
      "status": "complete",
      "orderstatus": "complete",
      "updatetime": "26-Jan-2026 10:16:45",
      "exchtime": "26-Jan-2026 10:16:45",
      "exchorderupdatetime": "26-Jan-2026 10:16:45",
      "fillid": "",
      "filltime": "",
      "parentorderid": ""
    }
  ]
}
```

### GET /rest/secure/angelbroking/order/v1/details/{orderid}
**Get Individual Order Status**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "variety": "NORMAL",
    "ordertype": "LIMIT",
    "producttype": "DELIVERY",
    "price": 500.00,
    "quantity": "10",
    "tradingsymbol": "SBIN-EQ",
    "exchange": "NSE",
    "orderid": "241126000012345",
    "status": "complete",
    "filledshares": "10",
    "averageprice": 500.25,
    "orderhistory": [
      {
        "status": "open",
        "updatetime": "26-Jan-2026 10:15:30"
      },
      {
        "status": "complete",
        "updatetime": "26-Jan-2026 10:16:45"
      }
    ]
  }
}
```

### GET /rest/secure/angelbroking/order/v1/getTradeBook
**Get Trade Book**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    {
      "exchange": "NSE",
      "producttype": "DELIVERY",
      "tradingsymbol": "SBIN-EQ",
      "instrumenttype": "",
      "symbolgroup": "",
      "strikeprice": -1.00,
      "optiontype": "",
      "expirydate": "",
      "marketlot": "1",
      "precision": 2,
      "multiplier": -1,
      "tradevalue": 5002.50,
      "transactiontype": "BUY",
      "fillprice": 500.25,
      "fillsize": "10",
      "orderid": "241126000012345",
      "fillid": "10001234567",
      "filltime": "26-Jan-2026 10:16:45"
    }
  ]
}
```

## GTT Orders

### POST /rest/secure/angelbroking/gtt/v1/createRule
**Create GTT Rule (Single)**

Request:
```json
{
  "tradingsymbol": "SBIN-EQ",
  "symboltoken": "3045",
  "exchange": "NSE",
  "producttype": "DELIVERY",
  "transactiontype": "BUY",
  "price": 450.00,
  "qty": 10,
  "triggerprice": 450.00,
  "gtttype": "SINGLE"
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "id": 123456
  }
}
```

### POST /rest/secure/angelbroking/gtt/v1/createRule
**Create GTT Rule (OCO)**

Request:
```json
{
  "tradingsymbol": "SBIN-EQ",
  "symboltoken": "3045",
  "exchange": "NSE",
  "producttype": "DELIVERY",
  "transactiontype": "SELL",
  "price": 520.00,
  "qty": 10,
  "triggerprice": 520.00,
  "gtttype": "OCO",
  "stoploss": 480.00
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "id": 123457
  }
}
```

### POST /rest/secure/angelbroking/gtt/v1/ruleList
**List GTT Rules**

Request:
```json
{
  "status": ["NEW", "ACTIVE"],
  "page": 1,
  "count": 10
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    {
      "id": 123456,
      "tradingsymbol": "SBIN-EQ",
      "symboltoken": "3045",
      "exchange": "NSE",
      "producttype": "DELIVERY",
      "transactiontype": "BUY",
      "price": 450.00,
      "qty": 10,
      "triggerprice": 450.00,
      "gtttype": "SINGLE",
      "status": "NEW",
      "createddate": "26-Jan-2026 09:00:00",
      "updateddate": "26-Jan-2026 09:00:00",
      "expirydate": "26-Jan-2027 23:59:59"
    }
  ]
}
```

## Portfolio & Account

### GET /rest/secure/angelbroking/portfolio/v1/getHolding
**Get Holdings**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "holdings": [
      {
        "tradingsymbol": "SBIN-EQ",
        "exchange": "NSE",
        "isin": "INE062A01020",
        "t1quantity": 0,
        "realisedquantity": 100,
        "quantity": 100,
        "authorisedquantity": 0,
        "product": "CNC",
        "collateralquantity": 0,
        "collateraltype": "",
        "haircut": 0.00,
        "averageprice": 480.50,
        "ltp": 500.25,
        "symboltoken": "3045",
        "close": 498.50,
        "profitandloss": 1975.00,
        "pnlpercentage": 4.11
      },
      {
        "tradingsymbol": "INFY-EQ",
        "exchange": "NSE",
        "isin": "INE009A01021",
        "t1quantity": 0,
        "realisedquantity": 50,
        "quantity": 50,
        "authorisedquantity": 0,
        "product": "CNC",
        "collateralquantity": 0,
        "collateraltype": "",
        "haircut": 0.00,
        "averageprice": 1420.00,
        "ltp": 1450.75,
        "symboltoken": "1594",
        "close": 1445.20,
        "profitandloss": 1537.50,
        "pnlpercentage": 2.17
      }
    ],
    "totalholding": {
      "totalholdingvalue": 123456.78,
      "totalinvvalue": 118950.00,
      "totalprofitandloss": 4506.78,
      "totalpnlpercentage": 3.79
    }
  }
}
```

### GET /rest/secure/angelbroking/portfolio/v1/getPosition
**Get Positions**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": [
    {
      "exchange": "NSE",
      "symboltoken": "3045",
      "producttype": "INTRADAY",
      "tradingsymbol": "SBIN-EQ",
      "symbolname": "SBIN",
      "instrumenttype": "",
      "priceden": 1.00,
      "pricenum": 1.00,
      "genden": 1.00,
      "gennum": 1.00,
      "precision": 2,
      "multiplier": 1,
      "boardlotsize": 1,
      "buyqty": 100,
      "sellqty": 0,
      "buyavgprice": 500.00,
      "sellavgprice": 0.00,
      "netvalue": 50000.00,
      "netqty": 100,
      "totalbuyvalue": 50000.00,
      "totalsellvalue": 0.00,
      "cfbuyqty": 0,
      "cfsellqty": 0,
      "cfbuyavgprice": 0.00,
      "cfsellavgprice": 0.00,
      "totalbuyavgprice": 500.00,
      "totalsellavgprice": 0.00,
      "netprice": 500.00,
      "buyPrice": 500.00,
      "sellPrice": 0.00,
      "ltp": 502.50,
      "close": 498.50,
      "pnl": 250.00,
      "pnlpercentage": 0.50
    }
  ]
}
```

### GET /rest/secure/angelbroking/user/v1/getRMS
**Get RMS Limits**

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "net": 1234567.89,
    "availablecash": 500000.00,
    "availableintradaypayin": 50000.00,
    "availablelimitmargin": 0.00,
    "collateral": 0.00,
    "m2munrealized": 2500.00,
    "m2mrealized": 1500.00,
    "utiliseddebits": 735067.89,
    "utilisedspan": 150000.00,
    "utilisedoptionpremium": 0.00,
    "utilisedholdingsales": 0.00,
    "utilisedexposure": 585067.89,
    "utilisedturnover": 0.00,
    "utilisedpayout": 0.00
  }
}
```

### POST /rest/secure/angelbroking/margin/v1/batch
**Margin Calculator**

Request:
```json
{
  "positions": [
    {
      "exchange": "NSE",
      "tradingsymbol": "SBIN-EQ",
      "symboltoken": "3045",
      "transactiontype": "BUY",
      "quantity": 100,
      "price": 500.00,
      "producttype": "INTRADAY"
    },
    {
      "exchange": "NSE",
      "tradingsymbol": "INFY-EQ",
      "symboltoken": "1594",
      "transactiontype": "BUY",
      "quantity": 50,
      "price": 1450.00,
      "producttype": "INTRADAY"
    }
  ]
}
```

Response:
```json
{
  "status": true,
  "message": "SUCCESS",
  "errorcode": "",
  "data": {
    "required_margin": 35000.50,
    "available_margin": 500000.00,
    "margin_shortfall": 0.00,
    "positions": [
      {
        "tradingsymbol": "SBIN-EQ",
        "margin_required": 10000.00
      },
      {
        "tradingsymbol": "INFY-EQ",
        "margin_required": 25000.50
      }
    ]
  }
}
```

## WebSocket Messages

### Mode 1: LTP Update
```json
{
  "subscription_mode": 1,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 123456,
  "exchange_timestamp": 1706259600000,
  "last_traded_price": 50025
}
```

**Note**: Price in paise (50025 = ₹500.25)

### Mode 2: Quote Update
```json
{
  "subscription_mode": 2,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 123457,
  "exchange_timestamp": 1706259601000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 12456789,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50250,
  "low_price_of_the_day": 49375,
  "closed_price": 49850,
  "last_traded_timestamp": 1706259601,
  "open_interest": 0,
  "open_interest_change_percentage": 0.00
}
```

### Mode 3: Snap Quote Update
```json
{
  "subscription_mode": 3,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 123458,
  "exchange_timestamp": 1706259602000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 12456789,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50250,
  "low_price_of_the_day": 49375,
  "closed_price": 49850,
  "last_traded_timestamp": 1706259602,
  "open_interest": 0,
  "open_interest_change_percentage": 0.00,
  "upper_circuit_limit": 54835,
  "lower_circuit_limit": 44865,
  "52_week_high": 55000,
  "52_week_low": 42000,
  "best_5_buy_data": [
    {"flag": 0, "quantity": 1000, "price": 50020, "no_of_orders": 5},
    {"flag": 0, "quantity": 800, "price": 50015, "no_of_orders": 3},
    {"flag": 0, "quantity": 1200, "price": 50010, "no_of_orders": 7},
    {"flag": 0, "quantity": 600, "price": 50005, "no_of_orders": 2},
    {"flag": 0, "quantity": 900, "price": 50000, "no_of_orders": 4}
  ],
  "best_5_sell_data": [
    {"flag": 1, "quantity": 800, "price": 50030, "no_of_orders": 3},
    {"flag": 1, "quantity": 1100, "price": 50035, "no_of_orders": 6},
    {"flag": 1, "quantity": 700, "price": 50040, "no_of_orders": 2},
    {"flag": 1, "quantity": 1300, "price": 50045, "no_of_orders": 8},
    {"flag": 1, "quantity": 500, "price": 50050, "no_of_orders": 1}
  ]
}
```

### Mode 4: Depth 20 Update
```json
{
  "subscription_mode": 4,
  "exchange_type": 1,
  "token": "3045",
  "sequence_number": 123459,
  "exchange_timestamp": 1706259603000,
  "last_traded_price": 50025,
  "last_traded_quantity": 100,
  "average_traded_price": 50010,
  "volume_trade_for_the_day": 12456789,
  "total_buy_quantity": 500000,
  "total_sell_quantity": 480000,
  "open_price_of_the_day": 49950,
  "high_price_of_the_day": 50250,
  "low_price_of_the_day": 49375,
  "closed_price": 49850,
  "last_traded_timestamp": 1706259603,
  "open_interest": 0,
  "open_interest_change_percentage": 0.00,
  "upper_circuit_limit": 54835,
  "lower_circuit_limit": 44865,
  "52_week_high": 55000,
  "52_week_low": 42000,
  "best_20_buy_data": [
    {"flag": 0, "quantity": 1000, "price": 50020, "no_of_orders": 5},
    {"flag": 0, "quantity": 800, "price": 50015, "no_of_orders": 3},
    /* ... 18 more levels ... */
  ],
  "best_20_sell_data": [
    {"flag": 1, "quantity": 800, "price": 50030, "no_of_orders": 3},
    {"flag": 1, "quantity": 1100, "price": 50035, "no_of_orders": 6},
    /* ... 18 more levels ... */
  ]
}
```

## Error Responses

### Invalid API Key
```json
{
  "status": false,
  "message": "Invalid API Key",
  "errorcode": "AG8001",
  "data": null
}
```

### Invalid Credentials
```json
{
  "status": false,
  "message": "Invalid User Credentials",
  "errorcode": "AG8002",
  "data": null
}
```

### Token Exception (Expired JWT)
```json
{
  "status": false,
  "message": "Invalid Token or Token Expired",
  "errorcode": "TokenException",
  "data": null
}
```

### Rate Limit Exceeded
```json
{
  "status": false,
  "message": "Rate limit exceeded. Please try after some time.",
  "errorcode": "AB8003",
  "data": null
}
```

### General Error (AB1004)
```json
{
  "status": false,
  "message": "Something Went Wrong, Please Try After Sometime",
  "errorcode": "AB1004",
  "data": null
}
```

### Missing Parameter
```json
{
  "status": false,
  "message": "symboltoken is required",
  "errorcode": "AB1001",
  "data": null
}
```

### Invalid Date Format
```json
{
  "status": false,
  "message": "From date should be less than to date",
  "errorcode": "AB1004",
  "data": null
}
```

## Notes

1. **Prices in REST API**: Often in rupees (decimal format) for quote/historical data
2. **Prices in WebSocket**: Always in integer paise (divide by 100 for rupees)
3. **Timestamps**:
   - REST API: String format "DD-MMM-YYYY HH:MM:SS" (IST)
   - Historical data: ISO format with timezone "YYYY-MM-DDTHH:MM:SS+05:30"
   - WebSocket: Unix timestamp in milliseconds
4. **Exchange Type Codes** (WebSocket):
   - 1 = NSE
   - 2 = NFO
   - 3 = BSE
   - 4 = BFO
   - 5 = MCX
   - 7 = CDS
   - 13 = NCDEX
5. **Status Envelope**: All responses wrapped in `{status, message, errorcode, data}` structure
6. **Boolean Status**: `true` for success, `false` for errors
7. **Null Data**: Error responses have `data: null`
