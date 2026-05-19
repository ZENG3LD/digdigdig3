# Interactive Brokers Client Portal Web API - REST API Endpoints

## Base URLs

**Gateway (Local):** `https://localhost:5000/v1/api/`
**Production (OAuth):** `https://api.ibkr.com/v1/api/`

## Common Headers

```http
Host: localhost:5000
User-Agent: YourApp/1.0
Accept: application/json
Connection: keep-alive
Content-Type: application/json  # For POST/PUT/PATCH
Content-Length: 123             # For POST/PUT/PATCH
```

## Response Format

All responses are JSON format with standard HTTP status codes.

**Success Response (200):**
```json
{
  "data": { ... }
}
```

**Error Response (4xx/5xx):**
```json
{
  "error": "Error message description",
  "statusCode": 400
}
```

---

## Authentication & Session Management

### Check Authentication Status

```http
GET /iserver/auth/status
```

**Description:** Check if brokerage session is authenticated

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "authenticated": true,
  "competing": false,
  "connected": true,
  "message": "",
  "MAC": "AA:BB:CC:DD:EE:FF",
  "serverInfo": {
    "serverName": "JifN19089",
    "serverVersion": "Build 10.22.0b, Jan 4, 2024 4:53:08 PM"
  }
}
```

### Initialize Brokerage Session

```http
POST /iserver/auth/ssodh/init
```

**Description:** Initialize brokerage session after login

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "compete": false,
  "connected": true
}
```

### Validate SSO Session

```http
GET /sso/validate
```

**Description:** Validate SSO session (for institutional accounts)

**Rate Limit:** 1 request per minute

**Response:**
```json
{
  "USER_ID": 12345678,
  "USER_NAME": "user123",
  "RESULT": true,
  "AUTH_TIME": 1706282400000
}
```

### Keep Session Alive (Tickle)

```http
GET /tickle
```

**Description:** Prevent session timeout by sending periodic requests

**Rate Limit:** 1 request per second

**Response:**
```json
{
  "session": "123abc456def",
  "ssoExpires": 600000,
  "collission": false,
  "userId": 12345678,
  "iserver": {
    "tickle": true,
    "authStatus": {
      "authenticated": true,
      "competing": false,
      "connected": true
    }
  }
}
```

**Usage:** Call every 30-60 seconds to maintain session

### Logout

```http
POST /logout
```

**Description:** End current session

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "confirmed": true
}
```

---

## Portfolio & Account Information

### Get Portfolio Accounts

```http
GET /portfolio/accounts
```

**Description:** List all accessible accounts (must be called before other portfolio endpoints)

**Rate Limit:** 1 request per 5 seconds

**Response:**
```json
[
  {
    "id": "DU12345",
    "accountId": "DU12345",
    "accountVan": "DU12345",
    "accountTitle": "Test Account",
    "displayName": "DU12345",
    "accountAlias": null,
    "accountStatus": 1641455400000,
    "currency": "USD",
    "type": "DEMO",
    "tradingType": "STKCASH",
    "faclient": false,
    "clearingStatus": "O",
    "covestor": false,
    "parent": {},
    "desc": "DU12345"
  }
]
```

### Get Sub-Accounts

```http
GET /portfolio/subaccounts
```

**Description:** Retrieve sub-accounts (for FA accounts)

**Rate Limit:** 1 request per 5 seconds

**Response:**
```json
{
  "accounts": [
    "DU12345",
    "DU12346"
  ],
  "acctProps": {
    "DU12345": {
      "hasChildAccounts": false,
      "supportsCashQty": true,
      "supportsFractions": false
    }
  }
}
```

### Get Portfolio for Account

```http
GET /portfolio/{accountId}/positions/{page}
```

**Parameters:**
- `accountId` - Account identifier (required)
- `page` - Page number, starting at 0 (required)

**Query Parameters:**
- `model` - Portfolio model name (optional)
- `sort` - Sort column (optional)
- `direction` - Sort direction: "a" ascending, "d" descending (optional)
- `period` - Period for P&L calculation (optional)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "acctId": "DU12345",
    "conid": 265598,
    "contractDesc": "AAPL",
    "position": 100.0,
    "mktPrice": 185.50,
    "mktValue": 18550.00,
    "currency": "USD",
    "avgCost": 180.00,
    "avgPrice": 180.00,
    "realizedPnl": 0.0,
    "unrealizedPnl": 550.00,
    "exchs": null,
    "expiry": null,
    "putOrCall": null,
    "multiplier": 1,
    "strike": 0.0,
    "exerciseStyle": null,
    "conExchMap": [],
    "assetClass": "STK",
    "undConid": 0,
    "model": ""
  }
]
```

### Get Position Details

```http
GET /portfolio/{accountId}/position/{conid}
```

**Parameters:**
- `accountId` - Account identifier
- `conid` - Contract ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "acctId": "DU12345",
  "conid": 265598,
  "contractDesc": "AAPL",
  "position": 100.0,
  "avgCost": 180.00,
  "avgPrice": 180.00,
  "mktPrice": 185.50,
  "mktValue": 18550.00,
  "currency": "USD",
  "realizedPnl": 0.0,
  "unrealizedPnl": 550.00,
  "assetClass": "STK"
}
```

### Get Account Summary

```http
GET /portfolio/{accountId}/summary
```

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "accountready": {
    "amount": "true",
    "currency": null,
    "isNull": false,
    "timestamp": 1706282400000,
    "value": "true",
    "severity": 0
  },
  "netliquidation": {
    "amount": 125000.50,
    "currency": "USD",
    "isNull": false,
    "timestamp": 1706282400000,
    "value": "125000.50",
    "severity": 0
  },
  "totalcashvalue": {
    "amount": 100000.00,
    "currency": "USD",
    "isNull": false,
    "timestamp": 1706282400000,
    "value": "100000.00",
    "severity": 0
  },
  "equity": {
    "amount": 125000.50,
    "currency": "USD",
    "isNull": false,
    "timestamp": 1706282400000,
    "value": "125000.50",
    "severity": 0
  },
  "buyingpower": {
    "amount": 250001.00,
    "currency": "USD",
    "isNull": false,
    "timestamp": 1706282400000,
    "value": "250001.00",
    "severity": 0
  }
}
```

**Common Summary Fields:**
- `netliquidation` - Net liquidation value
- `equity` - Equity with loan value
- `totalcashvalue` - Total cash value
- `buyingpower` - Current buying power
- `grosspositionvalue` - Gross position value
- `realizedpnl` - Realized P&L
- `unrealizedpnl` - Unrealized P&L
- `availablefunds` - Available funds
- `excessliquidity` - Excess liquidity
- `cushion` - Cushion percentage
- `leverage` - Current leverage

### Get Account Ledger

```http
GET /portfolio/{accountId}/ledger
```

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "BASE": {
    "commoditymarketvalue": 0.0,
    "futuremarketvalue": 0.0,
    "settledcash": 100000.00,
    "exchangerate": 1,
    "sessionid": 1,
    "cashbalance": 100000.00,
    "corporatebondsmarketvalue": 0.0,
    "warrantsmarketvalue": 0.0,
    "netliquidationvalue": 125000.50,
    "interest": 0.0,
    "unrealizedpnl": 550.00,
    "stockmarketvalue": 25000.50,
    "moneyfunds": 0.0,
    "currency": "BASE",
    "realizedpnl": 0.0,
    "funds": 0.0,
    "acctcode": "DU12345",
    "issueroptionsmarketvalue": 0.0,
    "key": "LedgerList"
  },
  "USD": {
    "commoditymarketvalue": 0.0,
    "futuremarketvalue": 0.0,
    "settledcash": 100000.00,
    "exchangerate": 1,
    "sessionid": 1,
    "cashbalance": 100000.00,
    "corporatebondsmarketvalue": 0.0,
    "warrantsmarketvalue": 0.0,
    "netliquidationvalue": 125000.50,
    "interest": 0.0,
    "unrealizedpnl": 550.00,
    "stockmarketvalue": 25000.50,
    "moneyfunds": 0.0,
    "currency": "USD",
    "realizedpnl": 0.0,
    "funds": 0.0,
    "acctcode": "DU12345",
    "issueroptionsmarketvalue": 0.0,
    "key": "LedgerList"
  }
}
```

### Get Allocation Data

```http
GET /portfolio/{accountId}/allocation
```

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "assetClass": {
    "long": {
      "STK": 25000.50
    },
    "short": {}
  },
  "sector": {
    "long": {
      "Technology": 25000.50
    },
    "short": {}
  },
  "group": {
    "long": {
      "Computers": 25000.50
    },
    "short": {}
  }
}
```

### Get Partitioned P&L

```http
GET /iserver/account/pnl/partitioned
```

**Description:** Get profit/loss breakdown by account

**Rate Limit:** 1 request per 5 seconds

**Response:**
```json
{
  "DU12345": {
    "dpl": 550.00,
    "nl": 125000.50,
    "upl": 550.00,
    "el": 125000.50,
    "mv": 25000.50
  }
}
```

**Fields:**
- `dpl` - Daily P&L
- `nl` - Net liquidation
- `upl` - Unrealized P&L
- `el` - Equity with loan value
- `mv` - Market value

---

## Contract & Symbol Search

### Search Contracts by Symbol

```http
POST /iserver/secdef/search
```

**Request Body:**
```json
{
  "symbol": "AAPL",
  "name": false,
  "secType": "STK"
}
```

**Parameters:**
- `symbol` - Symbol to search (required)
- `name` - Search by name if true (optional, default: false)
- `secType` - Security type filter: STK, OPT, FUT, CASH, BOND, CFD, WAR, IND, FUND (optional)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "conid": 265598,
    "companyHeader": "Apple Inc - Common Stock",
    "companyName": "Apple Inc",
    "symbol": "AAPL",
    "description": "AAPL",
    "restricted": null,
    "fop": "",
    "opt": null,
    "war": null,
    "sections": [
      {
        "secType": "STK",
        "months": "",
        "exchange": "SMART"
      }
    ]
  }
]
```

### Get Contract Details

```http
GET /iserver/contract/{conid}/info
```

**Parameters:**
- `conid` - Contract ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "conid": 265598,
  "symbol": "AAPL",
  "secType": "STK",
  "exchange": "NASDAQ",
  "listingExchange": "NASDAQ",
  "right": "",
  "strike": "",
  "currency": "USD",
  "cusip": "037833100",
  "coupon": "",
  "desc1": "AAPL",
  "desc2": "COMMON STOCK",
  "maturityDate": "",
  "multiplier": "",
  "tradingClass": "NMS",
  "validExchanges": "SMART,AMEX,NYSE,CBOE,PHLX,ISE,CHX,ARCA,ISLAND,IDEAL,NASDAQ,DRCTEDGE,BEX,BATS,EDGEA,BYX,IEX,FOXRIVER,NYSENAT,PSX"
}
```

### Get Contract Info (Basic)

```http
GET /iserver/contract/{conid}/info-and-rules
```

**Parameters:**
- `conid` - Contract ID
- `isBuy` - true for buy, false for sell (query parameter, optional)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "cfi_code": "ESVUFR",
  "symbol": "AAPL",
  "cusip": null,
  "expiry_full": null,
  "con_id": 265598,
  "maturity_date": null,
  "industry": "Technology",
  "instrument_type": "STK",
  "trading_class": "AAPL",
  "valid_exchanges": "SMART,AMEX,NYSE...",
  "allow_sell_long": true,
  "is_zero_commission_security": false,
  "local_symbol": "AAPL",
  "classifier": "Common Stock",
  "currency": "USD",
  "text": "APPLE INC",
  "underlying_con_id": 265598,
  "r_t_h": true,
  "multiplier": "1",
  "strike": null,
  "right": null,
  "underlying_issuer": null,
  "contract_month": null,
  "company_name": "APPLE INC",
  "smart_available": true,
  "exchange": "NASDAQ",
  "rules": {
    "orderTypes": ["MKT", "LMT", "STP", "STP_LMT", "TRAIL"],
    "orderTypesOutside": ["LMT"],
    "defaultSize": 100,
    "cashSize": 0,
    "sizeIncrement": 1,
    "tifTypes": ["DAY", "GTC", "OPG", "IOC"],
    "limitPrice": 0,
    "stopprice": 0,
    "preview": true
  }
}
```

### Get Trading Rules for Contract

```http
GET /iserver/contract/{conid}/rules
```

**Parameters:**
- `conid` - Contract ID
- `isBuy` - true for buy, false for sell (query parameter)
- `modifyOrder` - true for modification, false for new order (query parameter, optional)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "orderTypes": ["MKT", "LMT", "STP", "STP_LMT", "TRAIL"],
  "orderTypesOutside": ["LMT"],
  "defaultSize": 100,
  "cashSize": 0,
  "cashCcy": "USD",
  "sizeIncrement": 1,
  "tifTypes": ["DAY", "GTC", "OPG", "IOC", "GTD"],
  "limitPrice": 185.50,
  "stopprice": 0,
  "orderOrigination": 0,
  "preview": true,
  "displaySize": false,
  "increment": 0.01,
  "incrementDigits": 2,
  "canTradeOutsideRth": true
}
```

### Get Algorithm Parameters

```http
GET /iserver/contract/{conid}/algos
```

**Parameters:**
- `conid` - Contract ID
- `algos` - Comma-separated list of algo names (optional)
- `addDescription` - Include algo descriptions (query parameter, optional, default: false)
- `addParams` - Include algo parameters (query parameter, optional, default: false)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "id": "Adaptive",
    "name": "Adaptive",
    "description": "Attempts to fill limit orders at best price",
    "params": [
      {
        "name": "priority",
        "type": "STRING",
        "description": "Urgency level",
        "values": ["Patient", "Normal", "Urgent"]
      }
    ]
  }
]
```

### Security Definition Info

```http
GET /iserver/secdef/info
```

**Query Parameters:**
- `conid` - Contract ID (required)
- `sectype` - Security type (required)
- `month` - Contract month (optional, for futures/options)
- `exchange` - Exchange (optional)
- `strike` - Strike price (optional, for options)
- `right` - Right: C for call, P for put (optional, for options)

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "conid": 265598,
  "symbol": "AAPL",
  "secType": "STK",
  "exchange": "NASDAQ",
  "currency": "USD",
  "companyName": "APPLE INC",
  "category": "Technology"
}
```

---

## Market Data

### Get Market Data Snapshot

```http
GET /iserver/marketdata/snapshot
```

**Query Parameters:**
- `conids` - Comma-separated contract IDs (required)
- `fields` - Comma-separated field IDs (required)

**Rate Limit:** 10 requests per second

**Example Request:**
```http
GET /iserver/marketdata/snapshot?conids=265598,8314&fields=31,84,86,88,85,7059
```

**Response:**
```json
[
  {
    "conid": 265598,
    "conidEx": "265598",
    "31": 185.50,      // Last price
    "84": 185.48,      // Bid price
    "86": 185.52,      // Ask price
    "88": 500,         // Bid size
    "85": 300,         // Ask size
    "7059": 100,       // Last size
    "55": "AAPL",      // Symbol
    "_updated": 1706282400000
  },
  {
    "conid": 8314,
    "conidEx": "8314",
    "31": 437.25,
    "84": 437.20,
    "86": 437.30,
    "88": 200,
    "85": 150,
    "7059": 50,
    "55": "SPY",
    "_updated": 1706282401000
  }
]
```

**Common Field IDs:**
- `31` - Last price
- `55` - Symbol
- `58` - Text (description)
- `70` - High (session)
- `71` - Low (session)
- `73` - Market value
- `74` - Avg volume
- `75` - Put/Call interest (options)
- `76` - Put/Call volume (options)
- `77` - Historical volatility (options)
- `78` - Implied volatility (options)
- `79` - Open interest (futures/options)
- `80` - Shortable shares
- `82` - Open price
- `83` - Close price
- `84` - Bid price
- `85` - Ask size
- `86` - Ask price
- `87` - Volume
- `88` - Bid size
- `6004` - Exchange name
- `6008` - Contract description
- `6070` - Security type
- `6072` - Months (futures/options)
- `6119` - Market data availability
- `6457` - Put or call (options)
- `6508` - Market cap
- `6509` - Company name
- `7051` - Last exchange
- `7057` - Ask exchange
- `7058` - Last size
- `7059` - Bid exchange
- `7068` - Conid + exchange
- `7084` - Shortable
- `7085` - Shortable shares (numeric)
- `7086` - Market data availability (detailed)
- `7087` - IPO date
- `7088` - Market scanner data
- `7094` - Dividend amount
- `7095` - Dividend yield
- `7096` - Ex-dividend date
- `7097` - Contract ID (ex)
- `7219` - Prior close
- `7220` - Bid size (aggregate)
- `7221` - Ask size (aggregate)
- `7282` - Bid exchange (aggregate)
- `7283` - Ask exchange (aggregate)
- `7284` - EMA (Exponential Moving Average)
- `7285` - Market value (detailed)
- `7286` - Market cap (detailed)
- `7287` - Delayed bid
- `7288` - Delayed ask
- `7289` - Delayed last
- `7290` - Delayed bid size
- `7291` - Delayed ask size
- `7292` - Delayed last size
- `7293` - Delayed high
- `7294` - Delayed low
- `7295` - Delayed volume
- `7296` - Delayed close

### Get Historical Market Data

```http
GET /iserver/marketdata/history
```

**Query Parameters:**
- `conid` - Contract ID (required)
- `period` - Time period: {value}{unit} where unit is s/d/w/m/y (required)
  - Examples: "1d", "5d", "1w", "1m", "3m", "6m", "1y"
  - Units: s (seconds), d (days), w (weeks), m (months), y (years)
- `bar` - Bar size: {value}{unit} (required)
  - Examples: "1min", "5min", "15min", "30min", "1h", "2h", "4h", "1d", "1w"
  - Supported: 1sec, 5sec, 10sec, 15sec, 30sec, 1min, 2min, 3min, 5min, 10min, 15min, 20min, 30min, 1h, 2h, 3h, 4h, 8h, 1d, 1w, 1m
- `outsideRth` - Include data outside regular trading hours (optional, default: false)
- `barType` - Bar type: Last, Bid, Ask, Midpoint (optional, default: Last)

**Rate Limit:** 5 concurrent requests maximum

**Example Request:**
```http
GET /iserver/marketdata/history?conid=265598&period=1d&bar=5min&outsideRth=false
```

**Response:**
```json
{
  "serverId": "12345",
  "symbol": "AAPL",
  "text": "APPLE INC",
  "priceFactor": 1,
  "startTime": "20240126-09:30:00",
  "high": "186.50",
  "low": "184.20",
  "timePeriod": "1d",
  "barLength": 300,
  "mdAvailability": "S",
  "mktDataDelay": 0,
  "outsideRth": false,
  "tradingDayDuration": 390,
  "volumeFactor": 1,
  "priceDisplayRule": 1,
  "priceDisplayValue": "2",
  "negativeCapable": false,
  "messageVersion": 2,
  "data": [
    {
      "t": 1706268600000,  // Unix timestamp
      "o": 185.00,          // Open
      "c": 185.25,          // Close
      "h": 185.50,          // High
      "l": 184.90,          // Low
      "v": 125000           // Volume
    },
    {
      "t": 1706268900000,
      "o": 185.25,
      "c": 185.10,
      "h": 185.40,
      "l": 185.00,
      "v": 98000
    }
  ],
  "points": 78,
  "travelTime": 12
}
```

**Data Fields:**
- `t` - Timestamp (Unix milliseconds)
- `o` - Open price
- `c` - Close price
- `h` - High price
- `l` - Low price
- `v` - Volume

**Historical Data Limitations:**
- Bars < 30 seconds: Only available for 6 months from current date
- Larger bars: Available based on data subscription

### Unsubscribe Market Data (Single Contract)

```http
DELETE /iserver/marketdata/{conid}/unsubscribe
```

**Parameters:**
- `conid` - Contract ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "confirmed": true
}
```

### Unsubscribe All Market Data

```http
DELETE /iserver/marketdata/unsubscribe
```

**Description:** Unsubscribe from all active market data subscriptions

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "confirmed": true
}
```

---

## Trading & Orders

### Place Order

```http
POST /iserver/account/{accountId}/orders
```

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** 1 request per 5 seconds

**Request Body (Market Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "secType": "265598:STK",
      "orderType": "MKT",
      "side": "BUY",
      "tif": "DAY",
      "quantity": 100
    }
  ]
}
```

**Request Body (Limit Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "secType": "265598:STK",
      "orderType": "LMT",
      "price": 185.00,
      "side": "BUY",
      "tif": "GTC",
      "quantity": 100,
      "outsideRth": false
    }
  ]
}
```

**Request Body (Stop Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "STP",
      "price": 180.00,
      "side": "SELL",
      "tif": "DAY",
      "quantity": 100
    }
  ]
}
```

**Request Body (Stop Limit Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "STP_LMT",
      "price": 179.50,
      "auxPrice": 180.00,
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100
    }
  ]
}
```

**Request Body (Trailing Stop Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "TRAIL",
      "price": 185.00,
      "auxPrice": 2.00,
      "trailingAmt": 2.00,
      "trailingType": "amt",
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100
    }
  ]
}
```

**Request Body (Trailing Stop with Percentage):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "TRAIL",
      "price": 185.00,
      "trailingAmt": 2,
      "trailingType": "%",
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100
    }
  ]
}
```

**Request Body (Bracket Order):**
```json
{
  "orders": [
    {
      "conid": 265598,
      "orderType": "MKT",
      "side": "BUY",
      "tif": "DAY",
      "quantity": 100
    },
    {
      "conid": 265598,
      "orderType": "LMT",
      "price": 190.00,
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100,
      "parentId": 1
    },
    {
      "conid": 265598,
      "orderType": "STP",
      "price": 180.00,
      "side": "SELL",
      "tif": "GTC",
      "quantity": 100,
      "parentId": 1
    }
  ]
}
```

**Common Order Parameters:**
- `conid` - Contract ID (required)
- `secType` - Security type with conid: "{conid}:STK" (optional but recommended)
- `orderType` - Order type: MKT, LMT, STP, STP_LMT, TRAIL, MOC, LOC, MIT, LIT, etc. (required)
- `side` - Order side: BUY or SELL (required)
- `quantity` - Order quantity (required)
- `tif` - Time in force: DAY, GTC, IOC, OPG, GTD (required)
- `price` - Limit price (for LMT, STP_LMT, LIT, LOC orders)
- `auxPrice` - Auxiliary price (stop trigger price for STP_LMT)
- `trailingAmt` - Trailing amount (for TRAIL orders)
- `trailingType` - Trailing type: "amt" or "%" (for TRAIL orders)
- `outsideRth` - Allow execution outside regular trading hours (optional, default: false)
- `cashQty` - Cash quantity for forex (optional)
- `fxQty` - FX quantity (optional)
- `isCcyConv` - Currency conversion (optional, for forex)
- `parentId` - Parent order ID (for bracket/child orders)
- `listingExchange` - Exchange routing (optional)
- `useAdaptive` - Use adaptive algo (optional, default: false)

**Response (Initial):**
```json
[
  {
    "id": "reply-123abc",
    "message": [
      "This order will be placed on the next trading day.",
      "Are you sure you want to submit this order?"
    ]
  }
]
```

**Confirmation Required:** If response includes `id` and `message`, order requires confirmation via `/iserver/reply/{replyId}` endpoint.

### Confirm Order

```http
POST /iserver/reply/{replyId}
```

**Parameters:**
- `replyId` - Reply ID from initial order response

**Request Body:**
```json
{
  "confirmed": true
}
```

**Response (Success):**
```json
[
  {
    "order_id": "987654321",
    "order_status": "Submitted",
    "encrypt_message": "1"
  }
]
```

### Get Live Orders

```http
GET /iserver/account/orders
```

**Query Parameters:**
- `filters` - Comma-separated filters (optional)
  - Values: Inactive, Pending, Submitted, Filled, Cancelled
- `force` - Force refresh (optional, default: false)

**Rate Limit:** 1 request per 5 seconds

**Response:**
```json
{
  "orders": [
    {
      "acct": "DU12345",
      "conid": 265598,
      "conidex": "265598",
      "orderId": 987654321,
      "cashCcy": "USD",
      "sizeAndFills": "100",
      "orderDesc": "Bought 100 @ 185.50",
      "description1": "AAPL",
      "ticker": "AAPL",
      "secType": "STK",
      "listingExchange": "NASDAQ",
      "remainingQuantity": 0.0,
      "filledQuantity": 100.0,
      "totalSize": 100.0,
      "companyName": "APPLE INC",
      "status": "Filled",
      "order_ref": "ClientRef123",
      "side": "BUY",
      "price": 185.50,
      "bgColor": "#FFFFFF",
      "fgColor": "#000000",
      "order_status": "Filled",
      "parentId": null,
      "timeInForce": "DAY",
      "lastExecutionTime": "240126 10:30:45",
      "orderType": "Limit",
      "order_ccp_status": "N",
      "avgPrice": 185.52,
      "supports_tax_opt": "1",
      "lastExecutionTime_r": 1706268645000,
      "text": "AAPL"
    }
  ],
  "snapshot": true
}
```

**Order Status Values:**
- `PendingSubmit` - Order pending submission
- `PendingCancel` - Cancellation pending
- `PreSubmitted` - Order pre-submitted
- `Submitted` - Order submitted to exchange
- `Filled` - Order completely filled
- `Cancelled` - Order cancelled
- `Inactive` - Order inactive
- `ApiCancelled` - Cancelled by API

### Get Trades (Executions)

```http
GET /iserver/account/trades
```

**Description:** Get execution details for filled orders

**Rate Limit:** 1 request per 5 seconds

**Response:**
```json
[
  {
    "execution_id": "0000e0d5.63d4e3e2.01.01",
    "symbol": "AAPL",
    "side": "B",
    "order_description": "Bought 100 Limit 185.50",
    "trade_time": "240126 10:30:45",
    "trade_time_r": 1706268645000,
    "size": 100.0,
    "price": "185.52",
    "order_ref": "ClientRef123",
    "submitter": "api_client",
    "exchange": "NASDAQ",
    "commission": "1.00",
    "net_amount": 18553.00,
    "account": "DU12345",
    "accountCode": "DU12345",
    "company_name": "APPLE INC",
    "contract_description_1": "AAPL",
    "sec_type": "STK",
    "conid": 265598,
    "conidEx": "265598",
    "position": "100",
    "clearing_id": "IB",
    "clearing_name": "IB"
  }
]
```

### Modify Order

```http
POST /iserver/account/{accountId}/order/{orderId}
```

**Parameters:**
- `accountId` - Account identifier
- `orderId` - Order ID to modify

**Rate Limit:** Global (10 req/s)

**Request Body:**
```json
{
  "conid": 265598,
  "orderType": "LMT",
  "price": 186.00,
  "quantity": 150,
  "tif": "GTC"
}
```

**Response:**
```json
[
  {
    "order_id": "987654321",
    "order_status": "Modified",
    "encrypt_message": "1"
  }
]
```

### Cancel Order

```http
DELETE /iserver/account/{accountId}/order/{orderId}
```

**Parameters:**
- `accountId` - Account identifier
- `orderId` - Order ID to cancel

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "msg": "Request was submitted",
  "conid": 265598,
  "order_id": 987654321
}
```

### What-If Order (Preview)

```http
POST /iserver/account/{accountId}/whatiforder
```

**Description:** Preview order impact without actual execution

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** Global (10 req/s)

**Request Body:**
```json
{
  "conid": 265598,
  "orderType": "MKT",
  "side": "BUY",
  "quantity": 100,
  "tif": "DAY"
}
```

**Response:**
```json
{
  "amount": {
    "amount": "-18550.00",
    "commission": "1.00",
    "total": "-18551.00"
  },
  "equity": {
    "current": "125000.50",
    "change": "-18551.00",
    "after": "106449.50"
  },
  "initial": {
    "current": "31250.13",
    "change": "4637.75",
    "after": "35887.88"
  },
  "maintenance": {
    "current": "25000.10",
    "change": "3710.20",
    "after": "28710.30"
  },
  "position": {
    "current": 0,
    "change": 100,
    "after": 100
  },
  "warn": ""
}
```

---

## Market Scanner

### Get Scanner Parameters

```http
GET /iserver/scanner/params
```

**Description:** Retrieve available scanner filter options

**Rate Limit:** 1 request per 15 minutes

**Response:**
```json
{
  "scan_type_list": [
    "TOP_PERC_GAIN",
    "TOP_PERC_LOSE",
    "MOST_ACTIVE",
    "HOT_BY_VOLUME",
    "TOP_OPEN_PERC_GAIN",
    "TOP_OPEN_PERC_LOSE",
    "HIGH_SYNTH_BID_REV_NAT_YIELD",
    "LOW_SYNTH_ASK_REV_NAT_YIELD"
  ],
  "instrument_list": [
    "STK",
    "STK.US",
    "STK.EU",
    "BOND",
    "FUT",
    "OPT"
  ],
  "location_tree": [
    {
      "type": "STK",
      "locations": [
        {
          "type": "STK.US",
          "locations": ["STK.US.MAJOR", "STK.US.MINOR"]
        }
      ]
    }
  ],
  "filter_list": [
    "topOptionImpVolPct",
    "priceAbove",
    "priceBelow",
    "marketCapAboveBelow",
    "afterHoursChangePerc"
  ]
}
```

### Run Market Scanner

```http
POST /iserver/scanner/run
```

**Description:** Execute market scanner with specified criteria

**Rate Limit:** 1 request per second

**Request Body:**
```json
{
  "instrument": "STK",
  "type": "TOP_PERC_GAIN",
  "filter": [
    {
      "code": "priceAbove",
      "value": 1
    },
    {
      "code": "priceBelow",
      "value": 100
    }
  ],
  "location": "STK.US.MAJOR",
  "size": 50
}
```

**Parameters:**
- `instrument` - Instrument type (from scanner params)
- `type` - Scan type (from scanner params)
- `filter` - Array of filter objects (optional)
- `location` - Market location (from scanner params)
- `size` - Number of results (max 250)

**Response:**
```json
{
  "total": 50,
  "size": 50,
  "offset": 0,
  "scanTime": "20240126-15:30:00",
  "contracts": {
    "contracts": [
      {
        "conid": 265598,
        "server_id": "m1",
        "symbol": "AAPL",
        "companyName": "APPLE INC",
        "chg": 5.25,
        "chgpct": 2.91,
        "price": 185.50,
        "volume": 55234000,
        "avg_volume": 45000000,
        "market_cap": 2850000000000
      }
    ]
  }
}
```

---

## Alerts & Notifications

### Create Alert

```http
POST /iserver/account/{accountId}/alert
```

**Parameters:**
- `accountId` - Account identifier

**Request Body (Price Alert):**
```json
{
  "alertName": "AAPL Price Alert",
  "alertMessage": "AAPL reached $200",
  "orderId": 0,
  "alertRepeatable": 0,
  "email": "user@example.com",
  "sendMessage": 1,
  "tif": "GTC",
  "expire": "",
  "outsideRth": 1,
  "conditions": [
    {
      "type": "Price",
      "conidex": "265598",
      "operator": ">=",
      "value": 200.00,
      "timeZone": "US/Eastern",
      "triggerMethod": "Default"
    }
  ]
}
```

**Alert Types:**
- `Price` - Price-based alert
- `Time` - Time-based alert
- `Margin` - Margin cushion alert
- `Trade` - Trade execution alert
- `Volume` - Volume-based alert
- `MTA` - Mobile Trading Assistant alert

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "order_id": 123456789,
  "success": true,
  "text": "Alert created successfully"
}
```

### Get Alerts

```http
GET /iserver/account/{accountId}/alerts
```

**Parameters:**
- `accountId` - Account identifier

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "order_id": 123456789,
    "account": "DU12345",
    "alert_name": "AAPL Price Alert",
    "alert_active": 1,
    "order_time": "20240126-10:30:00",
    "alert_triggered": false,
    "alert_repeatable": 0
  }
]
```

### Delete Alert

```http
DELETE /iserver/account/alert/{orderId}
```

**Parameters:**
- `orderId` - Alert order ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "order_id": 123456789,
  "success": true,
  "text": "Alert deleted successfully"
}
```

### Get Unread Notifications Count

```http
GET /fyi/unreadnumber
```

**Description:** Get count of unread FYI notifications

**Rate Limit:** 1 request per second

**Response:**
```json
{
  "BN": 3,
  "total": 3
}
```

### Get Notifications

```http
GET /fyi/notifications
```

**Query Parameters:**
- `max` - Maximum number of notifications (default: 10)
- `exclude` - Comma-separated notification IDs to exclude
- `include` - Comma-separated notification types to include

**Rate Limit:** 1 request per second

**Response:**
```json
{
  "notifications": [
    {
      "R": 1706282400000,
      "D": "Your order for AAPL has been filled",
      "MD": "Order #987654321 - Bought 100 AAPL @ 185.52",
      "ID": "notif123",
      "FC": "Trade"
    }
  ]
}
```

**Fields:**
- `R` - Timestamp
- `D` - Description
- `MD` - Message detail
- `ID` - Notification ID
- `FC` - Notification category

### Mark Notification as Read

```http
PUT /fyi/notification/{notificationId}
```

**Parameters:**
- `notificationId` - Notification ID

**Rate Limit:** 1 request per second

**Response:**
```json
{
  "success": true
}
```

---

## Watchlists

### Create Watchlist

```http
POST /iserver/watchlists
```

**Request Body:**
```json
{
  "name": "My Watchlist",
  "instruments": [
    {"conid": 265598},
    {"conid": 8314},
    {"conid": 756733}
  ]
}
```

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "id": "wl_12345",
  "name": "My Watchlist",
  "readOnly": false
}
```

### Get All Watchlists

```http
GET /iserver/watchlists
```

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "id": "wl_12345",
    "name": "My Watchlist",
    "readOnly": false
  }
]
```

### Get Watchlist Details

```http
GET /iserver/watchlists/{watchlistId}
```

**Parameters:**
- `watchlistId` - Watchlist ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "id": "wl_12345",
  "name": "My Watchlist",
  "instruments": [
    {
      "conid": 265598,
      "symbol": "AAPL",
      "name": "APPLE INC",
      "price": 185.50,
      "chg": 2.30,
      "chgpct": 1.26
    },
    {
      "conid": 8314,
      "symbol": "SPY",
      "name": "SPDR S&P 500 ETF",
      "price": 437.25,
      "chg": -1.50,
      "chgpct": -0.34
    }
  ]
}
```

### Delete Watchlist

```http
DELETE /iserver/watchlists/{watchlistId}
```

**Parameters:**
- `watchlistId` - Watchlist ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "success": true
}
```

---

## Portfolio Analytics

### Get Performance Metrics

```http
POST /pa/performance
```

**Request Body:**
```json
{
  "acctIds": ["DU12345"],
  "period": "1M",
  "benchmark": "SPY"
}
```

**Rate Limit:** 1 request per 15 minutes

**Response:**
```json
{
  "id": "perf_123",
  "data": {
    "returns": {
      "total": 5.25,
      "mtd": 2.10,
      "ytd": 8.50
    },
    "benchmark": {
      "returns": 3.50
    },
    "alpha": 1.75,
    "beta": 1.05,
    "sharpe": 1.85
  }
}
```

### Get Performance Summary

```http
POST /pa/summary
```

**Request Body:**
```json
{
  "acctIds": ["DU12345"],
  "period": "1Y"
}
```

**Rate Limit:** 1 request per 15 minutes

**Response:**
```json
{
  "returns": {
    "total": 15.30,
    "ytd": 15.30,
    "mtd": 2.10
  },
  "nav": {
    "start": 108695.65,
    "end": 125000.50
  }
}
```

### Get Transaction History

```http
POST /pa/transactions
```

**Request Body:**
```json
{
  "acctIds": ["DU12345"],
  "conids": [265598],
  "currency": "USD",
  "days": 30
}
```

**Rate Limit:** 1 request per 15 minutes

**Response:**
```json
{
  "transactions": [
    {
      "date": "20240126",
      "type": "Trade",
      "description": "BOT 100 AAPL @ 185.52",
      "conid": 265598,
      "symbol": "AAPL",
      "quantity": 100,
      "price": 185.52,
      "amount": -18553.00,
      "proceeds": 0,
      "comm": 1.00,
      "net": -18553.00
    }
  ]
}
```

---

## Flex Web Service

### Generate Flex Report

```http
POST /pa/flex/generate
```

**Request Body:**
```json
{
  "queryId": "123456",
  "token": "your_flex_token"
}
```

**Parameters:**
- `queryId` - Flex query ID (from Account Management)
- `token` - Flex Web Service token

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "referenceCode": "789012345",
  "url": "https://gdcdyn.interactivebrokers.com/Universal/servlet/FlexStatementService.GetStatement"
}
```

### Check Flex Report Status

```http
GET /pa/flex/status/{requestId}
```

**Parameters:**
- `requestId` - Reference code from generate request

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "status": "Success",
  "reportUrl": "https://...",
  "expirationTime": "2024-01-27T00:00:00Z"
}
```

---

## Financial Advisor (FA) Endpoints

### Get Allocation Groups

```http
GET /iserver/account/allocation/groups
```

**Description:** List FA allocation groups

**Rate Limit:** Global (10 req/s)

**Response:**
```json
[
  {
    "id": "group1",
    "name": "Conservative Clients",
    "accounts": ["U123456", "U123457"],
    "defaultMethod": "AvailableEquity"
  }
]
```

### Create Allocation Group

```http
POST /iserver/account/allocation/groups
```

**Request Body:**
```json
{
  "name": "Aggressive Clients",
  "accounts": [
    {"acct": "U123458", "amount": 50},
    {"acct": "U123459", "amount": 50}
  ],
  "defaultMethod": "NetLiq"
}
```

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "id": "group2",
  "success": true
}
```

### Modify Allocation Group

```http
PUT /iserver/account/allocation/groups/{groupId}
```

**Parameters:**
- `groupId` - Allocation group ID

**Request Body:**
```json
{
  "name": "Updated Group Name",
  "accounts": [
    {"acct": "U123458", "amount": 60},
    {"acct": "U123459", "amount": 40}
  ]
}
```

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "success": true
}
```

### Delete Allocation Group

```http
DELETE /iserver/account/allocation/groups/{groupId}
```

**Parameters:**
- `groupId` - Allocation group ID

**Rate Limit:** Global (10 req/s)

**Response:**
```json
{
  "success": true
}
```

---

## Error Codes & Status Codes

### HTTP Status Codes

- **200 OK** - Request successful
- **400 Bad Request** - Invalid request parameters
- **401 Unauthorized** - Authentication required or failed
- **403 Forbidden** - Access denied
- **404 Not Found** - Endpoint or resource not found
- **429 Too Many Requests** - Rate limit exceeded
- **500 Internal Server Error** - Server-side error
- **502 Bad Gateway** - Gateway error
- **503 Service Unavailable** - Service temporarily unavailable

### Common Error Responses

**Authentication Error:**
```json
{
  "error": "Not authenticated",
  "statusCode": 401
}
```

**Rate Limit Error:**
```json
{
  "error": "Request was throttled. Expected available in 15 seconds.",
  "statusCode": 429
}
```

**Validation Error:**
```json
{
  "error": "Invalid conid parameter",
  "statusCode": 400
}
```

**Order Rejection:**
```json
{
  "error": "Order rejected: Insufficient funds",
  "statusCode": 400,
  "orderId": 0
}
```

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**Documentation:** https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/
