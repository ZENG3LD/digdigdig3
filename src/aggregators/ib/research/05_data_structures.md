# Interactive Brokers Client Portal Web API - Data Structures

## Common Data Types

### Contract Identifier (conid)

**Type:** Integer
**Description:** Unique identifier for financial instruments in IB system
**Example:** `265598` (Apple Inc.)
**Characteristics:**
- Static (never changes for a given instrument)
- Required for all market data and trading operations
- Obtained via contract search endpoints

### Security Types (secType)

**Type:** String
**Values:**
- `STK` - Stock/Equity
- `OPT` - Option
- `FUT` - Future
- `CASH` - Forex
- `BOND` - Bond
- `CFD` - Contract for Difference
- `WAR` - Warrant
- `IND` - Index
- `FUND` - Mutual Fund
- `CMDTY` - Commodity

### Currency Codes

**Type:** String (ISO 4217)
**Common Values:**
- `USD` - US Dollar
- `EUR` - Euro
- `GBP` - British Pound
- `JPY` - Japanese Yen
- `CHF` - Swiss Franc
- `CAD` - Canadian Dollar
- `AUD` - Australian Dollar
- `HKD` - Hong Kong Dollar

### Timestamps

**Type:** Long integer (Unix milliseconds)
**Example:** `1706282450123`
**Conversion:**
```python
from datetime import datetime

# Unix ms to datetime
timestamp_ms = 1706282450123
dt = datetime.fromtimestamp(timestamp_ms / 1000.0)

# Datetime to Unix ms
timestamp_ms = int(dt.timestamp() * 1000)
```

---

## Account Structures

### Account Object

```json
{
  "id": "DU12345",
  "accountId": "DU12345",
  "accountVan": "DU12345",
  "accountTitle": "Individual Trading Account",
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
```

**Fields:**
- `id` - Account identifier
- `accountId` - Account ID (same as id)
- `accountVan` - Account VAN (Virtual Account Number)
- `accountTitle` - Human-readable account title
- `displayName` - Display name for UI
- `accountAlias` - Custom alias (if set)
- `accountStatus` - Account status timestamp
- `currency` - Base currency
- `type` - Account type: DEMO, LIVE
- `tradingType` - Trading type: STKCASH, MARGIN, etc.
- `faclient` - Financial Advisor account flag
- `clearingStatus` - Clearing status: O (Open), C (Closed)
- `covestor` - Covestor account flag
- `parent` - Parent account (for sub-accounts)
- `desc` - Account description

### Account Summary Object

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
  }
}
```

**Summary Field Structure:**
- `amount` - Numeric value
- `currency` - Currency code (or null for non-currency values)
- `isNull` - Whether value is null
- `timestamp` - Last update timestamp (Unix ms)
- `value` - String representation of amount
- `severity` - Severity level: 0 (normal), 1 (warning), 2 (error)

**Common Summary Keys:**
- `accountready` - Account ready status
- `netliquidation` - Net liquidation value
- `totalcashvalue` - Total cash value
- `equity` - Equity with loan value
- `buyingpower` - Current buying power
- `grosspositionvalue` - Gross position value
- `realizedpnl` - Realized P&L
- `unrealizedpnl` - Unrealized P&L
- `availablefunds` - Available funds
- `excessliquidity` - Excess liquidity
- `cushion` - Cushion (%)
- `leverage` - Leverage ratio
- `initmarginreq` - Initial margin requirement
- `maintmarginreq` - Maintenance margin requirement
- `daytradesremaining` - Day trades remaining (PDT accounts)

### Ledger Object

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
  }
}
```

**Ledger Fields:**
- `commoditymarketvalue` - Commodity market value
- `futuremarketvalue` - Futures market value
- `settledcash` - Settled cash
- `exchangerate` - Exchange rate to base currency
- `sessionid` - Session identifier
- `cashbalance` - Cash balance
- `corporatebondsmarketvalue` - Corporate bonds market value
- `warrantsmarketvalue` - Warrants market value
- `netliquidationvalue` - Net liquidation value
- `interest` - Interest accrued
- `unrealizedpnl` - Unrealized P&L
- `stockmarketvalue` - Stock market value
- `moneyfunds` - Money market funds value
- `currency` - Currency (BASE for base currency aggregation)
- `realizedpnl` - Realized P&L
- `funds` - Funds
- `acctcode` - Account code
- `issueroptionsmarketvalue` - Issuer options market value

---

## Position Structures

### Position Object

```json
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
```

**Fields:**
- `acctId` - Account identifier
- `conid` - Contract ID
- `contractDesc` - Contract description/symbol
- `position` - Position size (positive for long, negative for short)
- `mktPrice` - Current market price
- `mktValue` - Market value of position (position * mktPrice * multiplier)
- `currency` - Position currency
- `avgCost` - Average cost basis
- `avgPrice` - Average entry price
- `realizedPnl` - Realized profit/loss
- `unrealizedPnl` - Unrealized profit/loss
- `exchs` - Exchanges (if applicable)
- `expiry` - Expiration date (for derivatives, format: YYYYMMDD)
- `putOrCall` - Put or Call (for options: "P" or "C")
- `multiplier` - Contract multiplier
- `strike` - Strike price (for options)
- `exerciseStyle` - Exercise style (for options: "A" American, "E" European)
- `conExchMap` - Contract exchange map
- `assetClass` - Asset class: STK, OPT, FUT, CASH, BOND, etc.
- `undConid` - Underlying contract ID (for derivatives)
- `model` - Portfolio model name (if using models)

### Allocation Data

```json
{
  "assetClass": {
    "long": {
      "STK": 25000.50,
      "OPT": 2500.00
    },
    "short": {
      "OPT": 1000.00
    }
  },
  "sector": {
    "long": {
      "Technology": 15000.50,
      "Healthcare": 10000.00
    },
    "short": {}
  },
  "group": {
    "long": {
      "Computers": 15000.50,
      "Pharmaceuticals": 10000.00
    },
    "short": {}
  }
}
```

**Structure:**
- `assetClass` - Allocation by asset class
  - `long` - Long positions by asset class
  - `short` - Short positions by asset class
- `sector` - Allocation by sector
- `group` - Allocation by industry group

---

## Contract Structures

### Contract Search Result

```json
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
```

**Fields:**
- `conid` - Contract ID
- `companyHeader` - Full company description
- `companyName` - Company name
- `symbol` - Trading symbol
- `description` - Short description
- `restricted` - Restricted trading indicator
- `fop` - Futures on options indicator
- `opt` - Options available indicator
- `war` - Warrants available indicator
- `sections` - Available contract sections
  - `secType` - Security type
  - `months` - Available expiration months (for derivatives)
  - `exchange` - Primary exchange

### Contract Details

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

**Fields:**
- `conid` - Contract ID
- `symbol` - Symbol
- `secType` - Security type
- `exchange` - Primary exchange
- `listingExchange` - Listing exchange
- `right` - Option right (C/P for options, empty for stocks)
- `strike` - Strike price (for options, empty for stocks)
- `currency` - Trading currency
- `cusip` - CUSIP identifier
- `coupon` - Coupon rate (for bonds, empty for stocks)
- `desc1` - Primary description
- `desc2` - Secondary description
- `maturityDate` - Maturity date (for bonds/derivatives, format: YYYYMMDD)
- `multiplier` - Contract multiplier
- `tradingClass` - Trading class
- `validExchanges` - Comma-separated list of valid exchanges

### Contract Info with Rules

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

**Trading Rules:**
- `orderTypes` - Allowed order types during regular trading hours
- `orderTypesOutside` - Allowed order types outside RTH
- `defaultSize` - Default order size
- `cashSize` - Cash quantity (for forex)
- `sizeIncrement` - Minimum size increment
- `tifTypes` - Allowed time-in-force types
- `limitPrice` - Suggested limit price
- `stopprice` - Suggested stop price
- `preview` - Preview available for this contract

---

## Order Structures

### Order Request

```json
{
  "orders": [
    {
      "conid": 265598,
      "secType": "265598:STK",
      "orderType": "LMT",
      "side": "BUY",
      "tif": "GTC",
      "quantity": 100,
      "price": 185.00,
      "outsideRth": false,
      "useAdaptive": false
    }
  ]
}
```

**Common Fields:**
- `conid` - Contract ID (required)
- `secType` - Security type with conid: "{conid}:STK" (optional)
- `orderType` - Order type (required)
  - `MKT` - Market
  - `LMT` - Limit
  - `STP` - Stop
  - `STP_LMT` - Stop Limit
  - `TRAIL` - Trailing Stop
  - `MOC` - Market on Close
  - `LOC` - Limit on Close
  - `MIT` - Market if Touched
  - `LIT` - Limit if Touched
- `side` - Order side (required)
  - `BUY` - Buy
  - `SELL` - Sell
- `tif` - Time in force (required)
  - `DAY` - Day order
  - `GTC` - Good til cancelled
  - `IOC` - Immediate or cancel
  - `OPG` - At the open
  - `GTD` - Good til date
- `quantity` - Order quantity (required)
- `price` - Limit price (for LMT, STP_LMT, LIT, LOC)
- `auxPrice` - Stop price (for STP_LMT, trigger price)
- `trailingAmt` - Trailing amount (for TRAIL orders)
- `trailingType` - Trailing type: "amt" or "%" (for TRAIL orders)
- `outsideRth` - Allow execution outside regular trading hours
- `cashQty` - Cash quantity (for forex)
- `fxQty` - FX quantity (for forex)
- `isCcyConv` - Currency conversion flag
- `parentId` - Parent order ID (for bracket/child orders)
- `listingExchange` - Exchange routing
- `useAdaptive` - Use adaptive algo

**Advanced Fields:**
- `referrer` - Order reference
- `isSingleGroup` - Single group flag
- `isLotAllocation` - Lot allocation flag (for FA)
- `allocation` - FA allocation method
- `strategy` - Strategy type
- `strategyParameters` - Strategy parameters

### Order Response

```json
{
  "order_id": "987654321",
  "order_status": "Submitted",
  "encrypt_message": "1",
  "local_order_id": "local_123",
  "order_ref": "ClientRef123"
}
```

**Fields:**
- `order_id` - IB order ID (string)
- `order_status` - Initial order status
- `encrypt_message` - Encryption flag
- `local_order_id` - Local order ID (client-assigned)
- `order_ref` - Order reference

### Order Confirmation Request

```json
{
  "id": "reply-123abc",
  "message": [
    "This order will be placed on the next trading day.",
    "Are you sure you want to submit this order?"
  ]
}
```

**Reply:**
```json
{
  "confirmed": true
}
```

### Live Order Object

```json
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
- `PendingVerify` - Pending verification
- `Rejected` - Order rejected

---

## Trade/Execution Structures

### Execution Object

```json
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
```

**Fields:**
- `execution_id` - Unique execution identifier
- `symbol` - Trading symbol
- `side` - Side: "B" (Buy) or "S" (Sell)
- `order_description` - Human-readable order description
- `trade_time` - Execution time (formatted string)
- `trade_time_r` - Execution timestamp (Unix ms)
- `size` - Executed quantity
- `price` - Execution price (string)
- `order_ref` - Order reference
- `submitter` - Order submitter
- `exchange` - Execution exchange
- `commission` - Commission charged (string)
- `net_amount` - Net amount (price * size +/- commission)
- `account` - Account identifier
- `company_name` - Company name
- `sec_type` - Security type
- `conid` - Contract ID
- `position` - Position after execution (string)
- `clearing_id` - Clearing broker ID
- `clearing_name` - Clearing broker name

---

## Market Data Structures

### Market Data Snapshot

```json
{
  "conid": 265598,
  "conidEx": "265598",
  "31": 185.50,
  "84": 185.48,
  "86": 185.52,
  "88": 500,
  "85": 300,
  "87": 55234000,
  "7059": 100,
  "55": "AAPL",
  "70": 186.50,
  "71": 184.20,
  "82": 185.00,
  "83": 185.25,
  "_updated": 1706282450123,
  "server_id": "m1"
}
```

**Field ID Mapping:**
- `31` - Last price
- `55` - Symbol
- `70` - High (session)
- `71` - Low (session)
- `82` - Open price
- `83` - Close price (previous)
- `84` - Bid price
- `85` - Ask size
- `86` - Ask price
- `87` - Volume
- `88` - Bid size
- `7059` - Last size
- `7051` - Last exchange
- `7057` - Ask exchange
- `7058` - Bid exchange
- `7219` - Prior close
- `_updated` - Update timestamp (Unix ms)
- `server_id` - Server identifier

### Historical Data Bar

```json
{
  "t": 1706268600000,
  "o": 185.00,
  "c": 185.25,
  "h": 185.50,
  "l": 184.90,
  "v": 125000
}
```

**Fields:**
- `t` - Timestamp (Unix ms)
- `o` - Open price
- `c` - Close price
- `h` - High price
- `l` - Low price
- `v` - Volume

### Historical Data Response

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
  "data": [],
  "points": 78,
  "travelTime": 12
}
```

**Fields:**
- `serverId` - Server identifier
- `symbol` - Symbol
- `text` - Company name
- `priceFactor` - Price multiplication factor
- `startTime` - Start time (formatted)
- `high` - Session high
- `low` - Session low
- `timePeriod` - Requested time period
- `barLength` - Bar length in seconds
- `mdAvailability` - Market data availability: S (Subscribed), D (Delayed)
- `mktDataDelay` - Market data delay in minutes
- `outsideRth` - Outside regular trading hours included
- `tradingDayDuration` - Trading day duration in minutes
- `volumeFactor` - Volume multiplication factor
- `priceDisplayRule` - Price display rule
- `priceDisplayValue` - Decimal places for price display
- `negativeCapable` - Can price be negative
- `messageVersion` - Message format version
- `data` - Array of bar objects
- `points` - Number of data points
- `travelTime` - Response time in milliseconds

---

## Scanner Structures

### Scanner Request

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
    },
    {
      "code": "marketCapAboveBelow",
      "value": 1000000000
    }
  ],
  "location": "STK.US.MAJOR",
  "size": 50
}
```

**Fields:**
- `instrument` - Instrument type
- `type` - Scan type
- `filter` - Array of filter objects
  - `code` - Filter code
  - `value` - Filter value
- `location` - Market location
- `size` - Result limit (max 250)

### Scanner Result

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

**Contract Fields:**
- `conid` - Contract ID
- `server_id` - Server identifier
- `symbol` - Symbol
- `companyName` - Company name
- `chg` - Price change
- `chgpct` - Percent change
- `price` - Current price
- `volume` - Current volume
- `avg_volume` - Average volume
- `market_cap` - Market capitalization

---

## Alert Structures

### Alert Request

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

**Condition Types:**
- `Price` - Price-based condition
- `Time` - Time-based condition
- `Margin` - Margin cushion condition
- `Trade` - Trade execution condition
- `Volume` - Volume-based condition

**Operators:**
- `>=` - Greater than or equal
- `<=` - Less than or equal
- `>` - Greater than
- `<` - Less than
- `=` - Equal to

### Alert Object

```json
{
  "order_id": 123456789,
  "account": "DU12345",
  "alert_name": "AAPL Price Alert",
  "alert_active": 1,
  "order_time": "20240126-10:30:00",
  "alert_triggered": false,
  "alert_repeatable": 0
}
```

---

## Notification Structures

### Notification Object

```json
{
  "R": 1706282400000,
  "D": "Your order for AAPL has been filled",
  "MD": "Order #987654321 - Bought 100 AAPL @ 185.52",
  "ID": "notif123",
  "FC": "Trade"
}
```

**Fields:**
- `R` - Timestamp (Unix ms)
- `D` - Description (short)
- `MD` - Message detail (full message)
- `ID` - Notification ID
- `FC` - Notification category

**Notification Categories:**
- `Trade` - Trade execution
- `Order` - Order status
- `Account` - Account-related
- `Margin` - Margin notifications
- `System` - System messages

---

## P&L Structures

### P&L Object

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
- `nl` - Net liquidation value
- `upl` - Unrealized P&L
- `el` - Equity with loan value
- `mv` - Market value of positions

---

## Error Structures

### Error Response

```json
{
  "error": "Invalid conid parameter",
  "statusCode": 400
}
```

### Order Error Response

```json
{
  "error": "Order rejected: Insufficient funds",
  "statusCode": 400,
  "orderId": 0,
  "details": {
    "reason": "INSUFFICIENT_FUNDS",
    "requiredMargin": 18551.00,
    "availableMargin": 10000.00
  }
}
```

---

## Validation Rules

### Numeric Precision

**Prices:**
- Stocks: Typically 2 decimal places (0.01 increment)
- Forex: 4-5 decimal places
- Futures: Varies by contract

**Quantities:**
- Stocks: Whole shares or fractional (if supported)
- Options: Whole contracts
- Forex: Varies by pair

### Size Limits

**Order Quantities:**
- Minimum: 1 (or fraction if supported)
- Maximum: Varies by instrument and account
- Increment: Specified in contract rules (`sizeIncrement`)

**Dollar Values:**
- Forex: Minimum notional varies by pair
- No maximum (subject to margin requirements)

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**Data Format:** JSON
