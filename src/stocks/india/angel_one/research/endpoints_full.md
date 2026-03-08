# Angel One SmartAPI - Complete Endpoint Reference

## Category: Session Management & Authentication

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/auth/angelbroking/user/v1/loginByPassword | Generate session with credentials | Yes | No | - | Requires clientcode, password, TOTP |
| POST | /rest/auth/angelbroking/jwt/v1/generateTokens | Refresh access token | Yes | Yes | - | Uses refresh token |
| GET | /rest/secure/angelbroking/user/v1/getProfile | Get user profile | Yes | Yes | - | Returns exchange info, account details |
| POST | /rest/secure/angelbroking/user/v1/logout | Terminate session | Yes | Yes | - | Logout user session |

## Category: Market Data - Real-time

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/market/v1/quote/ | Get LTP (Last Traded Price) | Yes | Yes | - | Mode: LTP, OHLC, or FULL |
| GET | /rest/secure/angelbroking/market/v1/quote/{mode}/{exchange}/{token} | Get market quote | Yes | Yes | - | Supports multiple modes |

### Quote Modes
- **LTP**: Last traded price only
- **OHLC**: Open, High, Low, Close with volume
- **FULL**: Complete market depth and ticker information

## Category: Market Data - Historical

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/historical/v1/getCandleData | Historical OHLC candles | Yes | Yes | - | Max 8000 candles per request |

### Historical Data Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| exchange | string | Yes | - | NSE, BSE, NFO, MCX, BFO, CDS |
| symboltoken | string | Yes | - | Instrument token |
| interval | string | Yes | - | ONE_MINUTE, THREE_MINUTE, FIVE_MINUTE, TEN_MINUTE, FIFTEEN_MINUTE, THIRTY_MINUTE, ONE_HOUR, ONE_DAY |
| fromdate | string | Yes | - | Format: "YYYY-MM-DD HH:MM" |
| todate | string | Yes | - | Format: "YYYY-MM-DD HH:MM" |

### Historical Data Limits by Interval
- **ONE_MINUTE**: 30 days (max 8000 records)
- **THREE_MINUTE**: 60 days
- **FIVE_MINUTE**: 100 days
- **TEN_MINUTE**: 100 days
- **FIFTEEN_MINUTE**: 200 days
- **THIRTY_MINUTE**: 200 days
- **ONE_HOUR**: 400 days
- **ONE_DAY**: 2000 days

**Note**: Historical data for expired F&O contracts is NOT available.

## Category: Metadata & Reference Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json | Download instrument master | Yes | No | - | Public JSON file with all symbols |
| POST | /rest/secure/angelbroking/order/v1/searchScrip | Search scrip/symbol | Yes | Yes | - | Search by exchange and symbol name |

### SearchScrip Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| exchange | string | Yes | - | NSE, BSE, NFO, MCX, BFO, CDS, NCDEX |
| searchscrip | string | Yes | - | Symbol name or partial name |

## Category: Order Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/order/v1/placeOrder | Place new order | Yes | Yes | 20/sec | Returns order ID only |
| POST | /rest/secure/angelbroking/order/v1/placeOrderFullResponse | Place order with full response | Yes | Yes | 20/sec | Returns complete order details |
| POST | /rest/secure/angelbroking/order/v1/modifyOrder | Modify existing order | Yes | Yes | 20/sec | Change price, quantity, etc. |
| POST | /rest/secure/angelbroking/order/v1/cancelOrder | Cancel order | Yes | Yes | 20/sec | Cancel pending order |
| GET | /rest/secure/angelbroking/order/v1/getOrderBook | Get all orders | Yes | Yes | - | Today's order book |
| GET | /rest/secure/angelbroking/order/v1/details/{orderid} | Get individual order status | Yes | Yes | 10/sec | Single order details |
| GET | /rest/secure/angelbroking/order/v1/getTradeBook | Get trade book | Yes | Yes | - | Executed trades for the day |

### Place Order Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| variety | string | Yes | - | NORMAL, STOPLOSS, AMO, ROBO |
| tradingsymbol | string | Yes | - | Trading symbol (e.g., "SBIN-EQ") |
| symboltoken | string | Yes | - | Instrument token |
| transactiontype | string | Yes | - | BUY, SELL |
| exchange | string | Yes | - | NSE, BSE, NFO, MCX, BFO, CDS |
| ordertype | string | Yes | - | MARKET, LIMIT, STOPLOSS_LIMIT, STOPLOSS_MARKET |
| producttype | string | Yes | - | DELIVERY, CARRYFORWARD, MARGIN, INTRADAY, BO |
| duration | string | Yes | - | DAY, IOC (Immediate or Cancel) |
| price | string | Yes | "0" | Limit price (0 for market orders) |
| squareoff | string | No | "0" | Target profit for bracket orders |
| stoploss | string | No | "0" | Stop loss for bracket orders |
| quantity | string | Yes | - | Order quantity |

### Order Variety Types
- **NORMAL**: Regular orders
- **STOPLOSS**: Stop loss orders
- **AMO**: After Market Orders (placed post-market, executed next day)
- **ROBO**: Bracket Orders

### Order Types
- **MARKET**: Market order (immediate execution at best price)
- **LIMIT**: Limit order (execute at specified price or better)
- **STOPLOSS_LIMIT**: Stop loss limit order
- **STOPLOSS_MARKET**: Stop loss market order

### Product Types
- **DELIVERY**: Cash & Carry for equity (CNC)
- **CARRYFORWARD**: Normal for futures and options (NRML)
- **MARGIN**: Margin Delivery
- **INTRADAY**: Margin Intraday Squareoff (MIS)
- **BO**: Bracket Order (for ROBO variety only)

## Category: GTT (Good Till Triggered) Orders

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/gtt/v1/createRule | Create GTT rule | Yes | Yes | 20/sec | Includes OCO support |
| POST | /rest/secure/angelbroking/gtt/v1/modifyRule | Modify GTT rule | Yes | Yes | 20/sec | Update existing GTT |
| POST | /rest/secure/angelbroking/gtt/v1/cancelRule | Cancel GTT rule | Yes | Yes | 20/sec | Cancel GTT order |
| POST | /rest/secure/angelbroking/gtt/v1/ruleDetails | Get GTT rule details | Yes | Yes | - | Single GTT details |
| POST | /rest/secure/angelbroking/gtt/v1/ruleList | List GTT rules | Yes | Yes | - | Paginated list of GTT rules |

### GTT Create Rule Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| tradingsymbol | string | Yes | - | Trading symbol |
| symboltoken | string | Yes | - | Instrument token |
| exchange | string | Yes | - | NSE, BSE (GTT supported on NSE & BSE only) |
| producttype | string | Yes | - | DELIVERY, MARGIN only |
| transactiontype | string | Yes | - | BUY, SELL |
| price | number | Yes | - | Trigger price |
| qty | number | Yes | - | Quantity |
| triggerprice | number | Yes | - | Trigger price for execution |
| gtttype | string | Yes | - | SINGLE, OCO |

### GTT Types
- **SINGLE**: Single trigger GTT
- **OCO**: One Cancels Other (two trigger prices, one cancels the other when hit)

**GTT Validity**: GTT orders are valid for 1 year from creation.

**GTT Segments**: Available only on NSE & BSE in DELIVERY and MARGIN segments.

## Category: Portfolio Management

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /rest/secure/angelbroking/portfolio/v1/getHolding | Get holdings | Yes | Yes | - | Long-term positions |
| GET | /rest/secure/angelbroking/portfolio/v1/getPosition | Get positions | Yes | Yes | - | Intraday and open positions |
| GET | /rest/secure/angelbroking/portfolio/v1/convertPosition | Convert position | Yes | Yes | - | Convert between product types |

### Holdings Response
- Includes individual holdings with P&L
- **totalholding** section with aggregate data:
  - `totalholdingvalue`: Total value of all holdings
  - Total quantity, average price, current price, P&L

## Category: Account & Funds

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /rest/secure/angelbroking/user/v1/getRMS | Get RMS (Risk Management) limits | Yes | Yes | - | Margin, funds, limits |

## Category: Margin Calculator

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/margin/v1/batch | Calculate margin for basket | Yes | Yes | 10/sec | Launched June 2025 |

### Margin Calculator Parameters
- Accepts a basket of positions (multiple orders)
- Calculates total margin requirement before execution
- Supports all product types and order types

## Category: WebSocket Control

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /rest/secure/angelbroking/user/v1/getfeedToken | Get feed token for WebSocket | Yes | Yes | - | Required for WebSocket auth |

**Note**: Feed token is separate from JWT auth token and specifically used for WebSocket V2 authentication.

## Error Codes

### Common Error Codes
| Code | Description | Resolution |
|------|-------------|------------|
| AB1004 | Something went wrong, try later | Retry after some time; check date formats |
| AG8001 | Invalid API Key | Verify API key is correct |
| TokenException | JWT token expired/invalid | Refresh token using generateTokens endpoint |

### HTTP Status Codes
- **200**: Success
- **400**: Bad request (missing/invalid parameters)
- **401**: Unauthorized (invalid credentials)
- **403**: Forbidden (token exception, authentication failure)
- **429**: Rate limit exceeded
- **500**: Internal server error

### Error Response Format
```json
{
  "status": false,
  "message": "Error description",
  "errorcode": "AB1004",
  "data": null
}
```

## Rate Limits Summary

| Category | Limit | Notes |
|----------|-------|-------|
| Order APIs (place/modify/cancel) | 20/sec | All order and GTT endpoints |
| Individual Order Status | 10/sec | Single order details endpoint |
| Margin Calculator | 10/sec | Margin calculation endpoint |
| WebSocket Subscriptions | 1000 tokens | Max symbols per connection |
| Other Endpoints | Not publicly specified | Reasonable usage expected |

**Additional Limits**: Rate limits also apply on per-minute and per-hour basis (specific values not publicly documented).

## Exchange Segments Supported

| Exchange | Full Name | Segment Type | Supported |
|----------|-----------|--------------|-----------|
| NSE | National Stock Exchange | Equity | Yes |
| BSE | Bombay Stock Exchange | Equity | Yes |
| NFO | NSE Futures & Options | Derivatives | Yes |
| BFO | BSE Futures & Options | Derivatives | Yes |
| MCX | Multi Commodity Exchange | Commodities | Yes |
| CDS | Currency Derivatives Segment | Currency | Yes |
| NCDEX | National Commodity & Derivatives Exchange | Commodities | Yes |

## Notes

1. **Authentication Required**: All endpoints except the instrument master file download require authentication.
2. **TOTP Required**: Login requires Time-based One-Time Password (TOTP) for 2FA.
3. **Session Validity**: Sessions are valid until midnight (market close).
4. **No Expired Contracts**: Historical data for expired F&O contracts is not available.
5. **Free Historical Data**: All historical data is FREE for all segments as of 2024-2026.
6. **8000 Candle Limit**: Maximum 8000 candles per historical data request.
7. **GTT Limitations**: GTT orders only on NSE & BSE in DELIVERY and MARGIN segments.
