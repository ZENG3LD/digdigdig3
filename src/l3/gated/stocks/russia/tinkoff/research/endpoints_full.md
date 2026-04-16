# Tinkoff Invest API - Complete Endpoint Reference

## Important Notes
- All endpoints are gRPC methods (Protocol Buffers v3)
- REST proxy available at: `https://invest-public-api.tbank.ru/rest/tinkoff.public.invest.api.contract.v1.{ServiceName}/{MethodName}`
- Authentication required for all methods via Bearer token
- Proto contracts: https://github.com/Tinkoff/investAPI/tree/main/src/docs/contracts

## Service: MarketDataService

Market data retrieval and real-time streaming.

### Request-Response Methods

| Method | Type | Description | Auth | Rate Limit | Notes |
|--------|------|-------------|------|------------|-------|
| GetCandles | Unary | Historical OHLC candles | Yes | Dynamic | 2500 candles max, intervals: 5s-1mo, depth up to 10 years |
| GetLastPrices | Unary | Last trade prices for instruments | Yes | Dynamic | Batch request, multiple FIGIs |
| GetOrderBook | Unary | Order book snapshot (L2) | Yes | Dynamic | Depth: 1, 10, 20, 30, 40, 50 levels |
| GetTradingStatus | Unary | Single instrument trading status | Yes | Dynamic | Limit/market order availability flags |
| GetTradingStatuses | Unary | Batch trading status | Yes | Dynamic | Multiple instruments |
| GetLastTrades | Unary | Anonymous trades (last hour) | Yes | Dynamic | Recent trade history |
| GetClosePrices | Unary | Session closing prices | Yes | Dynamic | End-of-day prices |

### Streaming Methods

| Method | Type | Description | Auth | Connection Limit | Notes |
|--------|------|-------------|------|------------------|-------|
| MarketDataStream | Bidirectional | Real-time market data | Yes | Dynamic | Candles, orderbook, trades, status, prices |
| MarketDataServerSideStream | Server-side | Server-initiated market data | Yes | Dynamic | Same subscriptions as bidirectional |

### GetCandles Parameters

**Request**: `GetCandlesRequest`
- `figi` (string, optional) - Financial Instrument Global Identifier
- `instrument_id` (string, optional) - Instrument UID (preferred for options)
- `from` (google.protobuf.Timestamp, required) - Start time (UTC)
- `to` (google.protobuf.Timestamp, required) - End time (UTC)
- `interval` (CandleInterval, required) - See intervals table below
- `limit` (int32, optional) - Max 2500 candles

**Candle Intervals & Historical Depth**:

| Interval | Code | Min Period | Max Period | Max Candles |
|----------|------|------------|------------|-------------|
| 5 seconds | CANDLE_INTERVAL_5_SEC | 5s | 200 min | 2500 |
| 10 seconds | CANDLE_INTERVAL_10_SEC | 10s | 200 min | 1250 |
| 30 seconds | CANDLE_INTERVAL_30_SEC | 30s | 20 hours | - |
| 1 minute | CANDLE_INTERVAL_1_MIN | 1m | 1 day | 2400 |
| 2 minutes | CANDLE_INTERVAL_2_MIN | 2m | 1 day | 1200 |
| 3 minutes | CANDLE_INTERVAL_3_MIN | 3m | 1 day | 750 |
| 5 minutes | CANDLE_INTERVAL_5_MIN | 5m | 1 week | 2400 |
| 10 minutes | CANDLE_INTERVAL_10_MIN | 10m | 1 week | - |
| 15 minutes | CANDLE_INTERVAL_15_MIN | 15m | 3 weeks | 2400 |
| 30 minutes | CANDLE_INTERVAL_30_MIN | 30m | 3 weeks | 1200 |
| 1 hour | CANDLE_INTERVAL_HOUR | 1h | 3 months | 2400 |
| 2 hours | CANDLE_INTERVAL_2_HOUR | 2h | 3 months | 2400 |
| 4 hours | CANDLE_INTERVAL_4_HOUR | 4h | 3 months | 700 |
| 1 day | CANDLE_INTERVAL_DAY | 1d | 6 years | 2400 |
| 1 week | CANDLE_INTERVAL_WEEK | 1w | 5 years | 300 |
| 1 month | CANDLE_INTERVAL_MONTH | 1mo | 10 years | 120 |

**Notes**:
- Historical data available from 1970-01-01
- All times in ISO UTC
- Max periods are approximate (may vary by instrument)

### GetOrderBook Parameters

**Request**: `GetOrderBookRequest`
- `figi` (string, optional) - Instrument FIGI
- `instrument_id` (string, optional) - Instrument UID
- `depth` (int32, required) - Order book depth: 1, 10, 20, 30, 40, or 50

## Service: InstrumentsService

Instrument information, trading schedules, and reference data.

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| TradingSchedules | Unary | Exchange trading hours/sessions | Yes | Returns schedule for date range |
| BondBy | Unary | Single bond by FIGI/ticker | Yes | Requires id_type + id or class_code |
| Bonds | Unary | List all bonds | Yes | Filter by instrument_status |
| GetBondCoupons | Unary | Bond coupon payment schedule | Yes | FIGI + date range |
| CurrencyBy | Unary | Single currency by identifier | Yes | |
| Currencies | Unary | List all currencies | Yes | |
| EtfBy | Unary | Single ETF by identifier | Yes | |
| Etfs | Unary | List all ETFs | Yes | |
| FutureBy | Unary | Single futures contract | Yes | |
| Futures | Unary | List all futures | Yes | |
| OptionBy | Unary | Single option by identifier | Yes | |
| OptionsBy | Unary | Options filtered by underlying | Yes | Requires basic_asset_uid or position_uid |
| ShareBy | Unary | Single stock by identifier | Yes | |
| Shares | Unary | List all stocks | Yes | ~1,900 shares on MOEX |
| GetAccruedInterests | Unary | Accrued coupon income | Yes | For bonds, FIGI + date range |
| GetFuturesMargin | Unary | Margin requirements for futures | Yes | Guarantee collateral amount |
| GetInstrumentBy | Unary | Generic instrument lookup | Yes | Returns basic info for any type |
| GetDividends | Unary | Dividend payment events | Yes | FIGI + date range |
| GetAssetBy | Unary | Single asset by UID | Yes | Asset-level data |
| GetAssets | Unary | List assets | Yes | Filter by instrument type |
| GetFavorites | Unary | User's favorite instruments | Yes | Personalized list |
| EditFavorites | Unary | Add/remove favorites | Yes | Action: add or delete |
| GetCountries | Unary | Country reference data | Yes | ISO codes + metadata |
| FindInstrument | Unary | Search instruments | Yes | Query string + type filter |
| GetBrands | Unary | List brands/companies | Yes | |
| GetBrandBy | Unary | Single brand by UID | Yes | Company information |

### Instrument Identifiers

**InstrumentIdType enum**:
- `INSTRUMENT_ID_TYPE_FIGI` - Financial Instrument Global Identifier (12-char code)
- `INSTRUMENT_ID_TYPE_TICKER` - Exchange ticker (requires class_code for uniqueness)
- `INSTRUMENT_ID_TYPE_UID` - Universal ID (PRIMARY, supports all types including options)

**Important**:
- FIGI not supported for options - use UID
- Ticker alone is NOT unique - must combine with class_code
- class_code = "Trading Mode" on Russian exchanges (MOEX, SPBE)

## Service: OrdersService

Order placement, management, and tracking.

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| PostOrder | Unary | Place market/limit order | Full-access | Requires account_id, direction, quantity, order_type, order_id |
| CancelOrder | Unary | Cancel active order | Full-access | account_id + order_id |
| GetOrderState | Unary | Get single order status | Yes | account_id + order_id |
| GetOrders | Unary | List active orders | Yes | account_id + optional filters |
| ReplaceOrder | Unary | Modify existing order | Full-access | Cancel + create new order atomically |

### Order Types

**OrderType enum**:
- `ORDER_TYPE_LIMIT` - Limit order (specific price)
- `ORDER_TYPE_MARKET` - Market order (immediate execution)
- `ORDER_TYPE_BESTPRICE` - Best price order (special type)

**OrderDirection enum**:
- `ORDER_DIRECTION_BUY` - Buy order
- `ORDER_DIRECTION_SELL` - Sell order

**OrderExecutionReportStatus enum** (order states):
- `EXECUTION_REPORT_STATUS_UNSPECIFIED` - Undefined
- `EXECUTION_REPORT_STATUS_FILL` - Fully filled
- `EXECUTION_REPORT_STATUS_REJECTED` - Rejected
- `EXECUTION_REPORT_STATUS_CANCELLED` - Cancelled
- `EXECUTION_REPORT_STATUS_NEW` - Accepted
- `EXECUTION_REPORT_STATUS_PARTIALLYFILL` - Partially filled

### PostOrder Parameters

**Request**: `PostOrderRequest`
- `figi` (string, optional) - Instrument FIGI
- `instrument_id` (string, optional) - Instrument UID
- `quantity` (int64, required) - Lots to buy/sell
- `price` (Quotation, optional) - Limit price (required for LIMIT orders)
- `direction` (OrderDirection, required) - BUY or SELL
- `account_id` (string, required) - Trading account ID
- `order_type` (OrderType, required) - LIMIT, MARKET, or BESTPRICE
- `order_id` (string, required) - Client-generated unique ID (idempotency)

## Service: StopOrdersService

Stop orders (conditional orders).

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| PostStopOrder | Unary | Place stop order | Full-access | Take-profit, stop-loss, stop-limit |
| GetStopOrders | Unary | List active stop orders | Yes | account_id filter |
| CancelStopOrder | Unary | Cancel stop order | Full-access | account_id + stop_order_id |

### Stop Order Types

**StopOrderType enum**:
- `STOP_ORDER_TYPE_TAKE_PROFIT` - Take profit (sell when price rises)
- `STOP_ORDER_TYPE_STOP_LOSS` - Stop loss (sell when price falls)
- `STOP_ORDER_TYPE_STOP_LIMIT` - Stop limit (limit order triggered at stop price)

**StopOrderExpirationType enum**:
- `STOP_ORDER_EXPIRATION_TYPE_GOOD_TILL_CANCEL` - Persistent until cancelled
- `STOP_ORDER_EXPIRATION_TYPE_GOOD_TILL_DATE` - Expires at specified UTC timestamp

**StopOrderDirection enum**:
- `STOP_ORDER_DIRECTION_BUY` - Buy stop
- `STOP_ORDER_DIRECTION_SELL` - Sell stop

### PostStopOrder Parameters

**Request**: `PostStopOrderRequest`
- `figi` (string, optional) - Instrument FIGI
- `instrument_id` (string, optional) - Instrument UID
- `quantity` (int64, required) - Lots
- `price` (Quotation, optional) - Limit price (for STOP_LIMIT)
- `stop_price` (Quotation, required) - Activation price
- `direction` (StopOrderDirection, required) - BUY or SELL
- `account_id` (string, required) - Trading account
- `expiration_type` (StopOrderExpirationType, required) - GTD or GTC
- `stop_order_type` (StopOrderType, required) - Type of stop order
- `expire_date` (google.protobuf.Timestamp, optional) - For GTD orders

## Service: OperationsService

Account operations, portfolio, positions, and reporting.

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| GetOperations | Unary | List account operations | Yes | Trades, commissions, dividends, etc. |
| GetPortfolio | Unary | Current portfolio holdings | Yes | Stocks, bonds, ETFs, currencies, futures, options |
| GetPositions | Unary | Current positions | Yes | Securities, futures, options + blocked amounts |
| GetWithdrawLimits | Unary | Available balance for withdrawal | Yes | Liquid funds, blocked amounts, margin |
| GetBrokerReport | Unary | Broker statement | Yes | Trade details, commissions, settlements |
| GetDividendsForeignIssuer | Unary | Foreign dividend report | Yes | Non-Russian dividends + tax withholding |
| GetOperationsByCursor | Unary | Operations with pagination | Yes | Cursor-based navigation, large datasets |

### GetOperations Parameters

**Request**: `OperationsRequest`
- `account_id` (string, required) - Trading account
- `from` (google.protobuf.Timestamp, required) - Start time
- `to` (google.protobuf.Timestamp, required) - End time
- `state` (OperationState, optional) - Filter by status
- `figi` (string, optional) - Filter by instrument

**OperationState enum**:
- `OPERATION_STATE_UNSPECIFIED` - All operations
- `OPERATION_STATE_EXECUTED` - Executed
- `OPERATION_STATE_CANCELED` - Cancelled

## Service: OperationsStreamService

Real-time streaming of portfolio and position changes.

| Method | Type | Description | Auth | Connection Limit |
|--------|------|-------------|------|------------------|
| PortfolioStream | Server-side | Portfolio updates stream | Yes | Dynamic |
| PositionsStream | Server-side | Positions updates stream | Yes | Dynamic |

## Service: OrdersStreamService

Real-time order execution tracking.

| Method | Type | Description | Auth | Connection Limit |
|--------|------|-------------|------|------------------|
| TradesStream | Server-side | Trade execution events | Yes | Dynamic |

## Service: UsersService

Account management and user information.

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| GetAccounts | Unary | List trading accounts | Yes | Returns all user accounts with types and statuses |
| GetMarginAttributes | Unary | Margin account attributes | Yes | Leverage, liquidity, margin requirements |
| GetUserTariff | Unary | User tariff/commission plan | Yes | Commission rates, service fees |
| GetInfo | Unary | User profile information | Yes | Qualification status, restrictions |

### Account Types

**AccountType enum**:
- `ACCOUNT_TYPE_UNSPECIFIED` - Undefined
- `ACCOUNT_TYPE_TINKOFF` - Tinkoff brokerage account
- `ACCOUNT_TYPE_TINKOFF_IIS` - Individual Investment Account (IIS)
- `ACCOUNT_TYPE_INVEST_BOX` - Invest Box account

**AccountStatus enum**:
- `ACCOUNT_STATUS_UNSPECIFIED` - Undefined
- `ACCOUNT_STATUS_NEW` - New account
- `ACCOUNT_STATUS_OPEN` - Active
- `ACCOUNT_STATUS_CLOSED` - Closed

## Service: SandboxService

Testing environment (does not affect real portfolio).

| Method | Type | Description | Auth | Notes |
|--------|------|-------------|------|-------|
| OpenSandboxAccount | Unary | Create sandbox account | Sandbox token | For testing strategies |
| GetSandboxAccounts | Unary | List sandbox accounts | Sandbox token | |
| CloseSandboxAccount | Unary | Delete sandbox account | Sandbox token | |
| PostSandboxOrder | Unary | Place order in sandbox | Sandbox token | Same as PostOrder |
| GetSandboxOrders | Unary | List sandbox orders | Sandbox token | |
| CancelSandboxOrder | Unary | Cancel sandbox order | Sandbox token | |
| GetSandboxOrderState | Unary | Get sandbox order status | Sandbox token | |
| GetSandboxPositions | Unary | Sandbox positions | Sandbox token | |
| GetSandboxOperations | Unary | Sandbox operations | Sandbox token | |
| GetSandboxPortfolio | Unary | Sandbox portfolio | Sandbox token | |
| SandboxPayIn | Unary | Add virtual funds | Sandbox token | For testing with capital |

**Important**: Sandbox token MUST be used exclusively with sandbox services. Using sandbox token with production services returns error.

## Rate Limiting

- **Dynamic limits** based on trading activity
- **Active traders**: Higher limits (more fees = more requests)
- **Low-volume traders**: Standard limits
- **Platform capacity**: 20,000 req/sec peak (shared)
- **Error code**: 80002 (request rate exceeded)
- **Response headers**: Not documented (gRPC metadata may include tracking-id)
- **Recommended handling**: Exponential backoff

## Common Parameters

### Timestamp Format
- **Type**: `google.protobuf.Timestamp`
- **Timezone**: UTC
- **Range**: 1970-01-01 to 2099-12-31

### Quotation Type
- **Fields**:
  - `units` (int64) - Integer part
  - `nano` (int32) - Fractional part (9 decimal places)
- **Example**: 150.25 = {units: 150, nano: 250000000}

### MoneyValue Type
- **Fields**:
  - `currency` (string) - ISO currency code (RUB, USD, EUR, etc.)
  - `units` (int64) - Integer part
  - `nano` (int32) - Fractional part
- **Example**: 1000.50 RUB = {currency: "RUB", units: 1000, nano: 500000000}

## Error Handling

All errors returned as gRPC status codes with detailed messages. See `authentication.md` and separate error documentation for complete list.

Common errors:
- `40003` - Invalid/expired token
- `30052` - Instrument forbidden for API trading
- `50002` - Instrument not found
- `80002` - Rate limit exceeded
- `90003` - Order value too high
