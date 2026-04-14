# Tinkoff Invest API - Response Formats

## Important Notes

- All responses use Protocol Buffers (proto3) format
- JSON representation follows protobuf-to-JSON mapping
- Quotation type: `{units: int64, nano: int32}` for prices
- MoneyValue type: `{currency: string, units: int64, nano: int32}` for amounts
- Timestamps: ISO 8601 format in JSON, google.protobuf.Timestamp in proto
- All times in UTC

## Core Data Types

### Quotation
```json
{
  "units": 150,
  "nano": 250000000
}
```
Represents: 150.25 (9 decimal places precision)

### MoneyValue
```json
{
  "currency": "RUB",
  "units": 1000,
  "nano": 500000000
}
```
Represents: 1000.50 RUB

### Timestamp (google.protobuf.Timestamp)
```json
"2026-01-26T10:30:45.123456Z"
```
ISO 8601 format in UTC

## Market Data Service

### GetCandles

**Endpoint**: `MarketDataService/GetCandles`

**Response**: `GetCandlesResponse`
```json
{
  "candles": [
    {
      "open": {
        "units": 150,
        "nano": 0
      },
      "high": {
        "units": 150,
        "nano": 500000000
      },
      "low": {
        "units": 149,
        "nano": 800000000
      },
      "close": {
        "units": 150,
        "nano": 250000000
      },
      "volume": 1234567,
      "time": "2026-01-26T10:30:00Z",
      "isComplete": true,
      "figi": "BBG004730N88",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    },
    {
      "open": {
        "units": 150,
        "nano": 250000000
      },
      "high": {
        "units": 150,
        "nano": 750000000
      },
      "low": {
        "units": 150,
        "nano": 100000000
      },
      "close": {
        "units": 150,
        "nano": 600000000
      },
      "volume": 987654,
      "time": "2026-01-26T10:31:00Z",
      "isComplete": true,
      "figi": "BBG004730N88",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    }
  ]
}
```

**Fields**:
- `candles[]` - Array of historical candles
  - `open` (Quotation) - Opening price
  - `high` (Quotation) - Highest price
  - `low` (Quotation) - Lowest price
  - `close` (Quotation) - Closing price
  - `volume` (int64) - Volume in lots
  - `time` (Timestamp) - Candle start time
  - `isComplete` (bool) - Whether candle is closed (true) or still forming (false)
  - `figi` (string) - Instrument FIGI (optional)
  - `instrumentUid` (string) - Instrument UID (optional)

### GetLastPrices

**Endpoint**: `MarketDataService/GetLastPrices`

**Response**: `GetLastPricesResponse`
```json
{
  "lastPrices": [
    {
      "figi": "BBG004730N88",
      "price": {
        "units": 150,
        "nano": 250000000
      },
      "time": "2026-01-26T10:30:45.123Z",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    },
    {
      "figi": "BBG0047315Y7",
      "price": {
        "units": 75,
        "nano": 500000000
      },
      "time": "2026-01-26T10:30:47.456Z",
      "instrumentUid": "e6123145-9665-43e0-8413-cd61b8aa9b13"
    }
  ]
}
```

**Fields**:
- `lastPrices[]` - Array of last prices
  - `figi` (string) - Instrument FIGI
  - `price` (Quotation) - Last trade price
  - `time` (Timestamp) - Last trade time
  - `instrumentUid` (string) - Instrument UID

### GetOrderBook

**Endpoint**: `MarketDataService/GetOrderBook`

**Response**: `GetOrderBookResponse`
```json
{
  "figi": "BBG004730N88",
  "depth": 10,
  "bids": [
    {
      "price": {
        "units": 150,
        "nano": 240000000
      },
      "quantity": 1000
    },
    {
      "price": {
        "units": 150,
        "nano": 230000000
      },
      "quantity": 2500
    },
    {
      "price": {
        "units": 150,
        "nano": 220000000
      },
      "quantity": 1500
    }
  ],
  "asks": [
    {
      "price": {
        "units": 150,
        "nano": 250000000
      },
      "quantity": 800
    },
    {
      "price": {
        "units": 150,
        "nano": 260000000
      },
      "quantity": 1200
    },
    {
      "price": {
        "units": 150,
        "nano": 270000000
      },
      "quantity": 2000
    }
  ],
  "lastPrice": {
    "units": 150,
    "nano": 245000000
  },
  "closePrice": {
    "units": 149,
    "nano": 900000000
  },
  "limitUp": {
    "units": 165,
    "nano": 0
  },
  "limitDown": {
    "units": 135,
    "nano": 0
  },
  "lastPriceTs": "2026-01-26T10:30:45Z",
  "closePriceTs": "2026-01-25T18:50:00Z",
  "orderbookTs": "2026-01-26T10:30:46.123Z",
  "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
}
```

**Fields**:
- `figi` (string) - Instrument FIGI
- `depth` (int32) - Depth requested
- `bids[]` - Buy orders (best prices first)
  - `price` (Quotation) - Bid price
  - `quantity` (int64) - Quantity in lots
- `asks[]` - Sell orders (best prices first)
  - `price` (Quotation) - Ask price
  - `quantity` (int64) - Quantity in lots
- `lastPrice` (Quotation) - Last trade price
- `closePrice` (Quotation) - Previous day close
- `limitUp` (Quotation) - Upper price limit
- `limitDown` (Quotation) - Lower price limit
- `lastPriceTs` (Timestamp) - Last trade time
- `closePriceTs` (Timestamp) - Close price time
- `orderbookTs` (Timestamp) - Order book snapshot time
- `instrumentUid` (string) - Instrument UID

### GetTradingStatus

**Endpoint**: `MarketDataService/GetTradingStatus`

**Response**: `GetTradingStatusResponse`
```json
{
  "figi": "BBG004730N88",
  "tradingStatus": "SECURITY_TRADING_STATUS_NORMAL_TRADING",
  "limitOrderAvailableFlag": true,
  "marketOrderAvailableFlag": true,
  "apiTradeAvailableFlag": true,
  "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
}
```

**SecurityTradingStatus enum values**:
- `SECURITY_TRADING_STATUS_UNSPECIFIED` (0)
- `SECURITY_TRADING_STATUS_NOT_AVAILABLE_FOR_TRADING` (1)
- `SECURITY_TRADING_STATUS_OPENING_PERIOD` (2)
- `SECURITY_TRADING_STATUS_CLOSING_PERIOD` (3)
- `SECURITY_TRADING_STATUS_BREAK_IN_TRADING` (4)
- `SECURITY_TRADING_STATUS_NORMAL_TRADING` (5)
- `SECURITY_TRADING_STATUS_CLOSING_AUCTION` (6)
- `SECURITY_TRADING_STATUS_DARK_POOL_AUCTION` (7)
- `SECURITY_TRADING_STATUS_DISCRETE_AUCTION` (8)
- Additional status codes up to 17

### GetLastTrades

**Endpoint**: `MarketDataService/GetLastTrades`

**Response**: `GetLastTradesResponse`
```json
{
  "trades": [
    {
      "figi": "BBG004730N88",
      "direction": "TRADE_DIRECTION_BUY",
      "price": {
        "units": 150,
        "nano": 250000000
      },
      "quantity": 100,
      "time": "2026-01-26T10:30:45.123Z",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    },
    {
      "figi": "BBG004730N88",
      "direction": "TRADE_DIRECTION_SELL",
      "price": {
        "units": 150,
        "nano": 240000000
      },
      "quantity": 50,
      "time": "2026-01-26T10:30:47.456Z",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    }
  ]
}
```

**TradeDirection enum**:
- `TRADE_DIRECTION_UNSPECIFIED` (0)
- `TRADE_DIRECTION_BUY` (1)
- `TRADE_DIRECTION_SELL` (2)

## Instruments Service

### Shares (GetShares)

**Endpoint**: `InstrumentsService/Shares`

**Response**: `SharesResponse`
```json
{
  "instruments": [
    {
      "figi": "BBG004730N88",
      "ticker": "SBER",
      "classCode": "TQBR",
      "isin": "RU0009029540",
      "lot": 10,
      "currency": "RUB",
      "shortEnabledFlag": true,
      "name": "Сбербанк",
      "exchange": "MOEX",
      "countryOfRisk": "RU",
      "countryOfRiskName": "Российская Федерация",
      "sector": "financial",
      "issueSize": 21586948000,
      "nominal": {
        "currency": "RUB",
        "units": 3,
        "nano": 0
      },
      "tradingStatus": "SECURITY_TRADING_STATUS_NORMAL_TRADING",
      "otcFlag": false,
      "buyAvailableFlag": true,
      "sellAvailableFlag": true,
      "divYieldFlag": false,
      "shareType": "SHARE_TYPE_COMMON",
      "minPriceIncrement": {
        "units": 0,
        "nano": 10000000
      },
      "apiTradeAvailableFlag": true,
      "uid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297",
      "realExchange": "REAL_EXCHANGE_MOEX",
      "positionUid": "a1e6df49-5ff7-4f7a-b3a6-89f56f19cfb4",
      "forIisFlag": true,
      "forQualInvestorFlag": false,
      "weekendFlag": false,
      "blockedTcaFlag": false,
      "instrumentType": "share",
      "first1minCandleDate": "2015-01-01T00:00:00Z",
      "first1dayCandleDate": "1998-07-27T00:00:00Z"
    }
  ]
}
```

### Bonds (GetBonds)

**Response**: `BondsResponse`
```json
{
  "instruments": [
    {
      "figi": "BBG00M0C8SY3",
      "ticker": "SU26238RMFS3",
      "classCode": "TQOB",
      "isin": "RU000A104H74",
      "lot": 1,
      "currency": "RUB",
      "name": "ОФЗ 26238",
      "nominal": {
        "currency": "RUB",
        "units": 1000,
        "nano": 0
      },
      "stateRegDate": "2021-09-15T00:00:00Z",
      "placementDate": "2021-09-22T00:00:00Z",
      "maturityDate": "2034-07-19T00:00:00Z",
      "couponQuantityPerYear": 2,
      "aciValue": {
        "currency": "RUB",
        "units": 15,
        "nano": 450000000
      },
      "tradingStatus": "SECURITY_TRADING_STATUS_NORMAL_TRADING",
      "floatingCouponFlag": false,
      "perpetualFlag": false,
      "amortizationFlag": false,
      "minPriceIncrement": {
        "units": 0,
        "nano": 10000000
      },
      "apiTradeAvailableFlag": true,
      "uid": "d5e25d30-51d0-40a9-a8e0-5e3d4c3e44e8"
    }
  ]
}
```

**Additional bond fields**:
- `stateRegDate` - State registration date
- `placementDate` - Initial placement date
- `maturityDate` - Bond maturity date
- `couponQuantityPerYear` (int32) - Coupon payments per year
- `aciValue` (MoneyValue) - Accrued coupon income (НКД)
- `floatingCouponFlag` (bool) - Floating rate coupon
- `perpetualFlag` (bool) - Perpetual bond (no maturity)
- `amortizationFlag` (bool) - Amortization schedule

## Operations Service

### GetPortfolio

**Endpoint**: `OperationsService/GetPortfolio`

**Response**: `PortfolioResponse`
```json
{
  "totalAmountShares": {
    "currency": "RUB",
    "units": 500000,
    "nano": 0
  },
  "totalAmountBonds": {
    "currency": "RUB",
    "units": 100000,
    "nano": 0
  },
  "totalAmountEtf": {
    "currency": "RUB",
    "units": 50000,
    "nano": 0
  },
  "totalAmountCurrencies": {
    "currency": "RUB",
    "units": 25000,
    "nano": 0
  },
  "totalAmountFutures": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "expectedYield": {
    "units": 15000,
    "nano": 500000000
  },
  "positions": [
    {
      "figi": "BBG004730N88",
      "instrumentType": "share",
      "quantity": {
        "units": 100,
        "nano": 0
      },
      "averagePositionPrice": {
        "currency": "RUB",
        "units": 145,
        "nano": 0
      },
      "expectedYield": {
        "units": 525,
        "nano": 0
      },
      "currentNkd": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "averagePositionPricePt": {
        "units": 0,
        "nano": 0
      },
      "currentPrice": {
        "currency": "RUB",
        "units": 150,
        "nano": 250000000
      },
      "averagePositionPriceFifo": {
        "currency": "RUB",
        "units": 145,
        "nano": 0
      },
      "quantityLots": {
        "units": 10,
        "nano": 0
      },
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297",
      "varMargin": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "expectedYieldFifo": {
        "units": 525,
        "nano": 0
      }
    }
  ]
}
```

**Fields**:
- `totalAmountShares` (MoneyValue) - Total value of stocks
- `totalAmountBonds` (MoneyValue) - Total value of bonds
- `totalAmountEtf` (MoneyValue) - Total value of ETFs
- `totalAmountCurrencies` (MoneyValue) - Total value of currencies
- `totalAmountFutures` (MoneyValue) - Total futures value
- `expectedYield` (Quotation) - Total expected P&L
- `positions[]` - Array of positions
  - `quantity` (Quotation) - Position size (units, not lots)
  - `quantityLots` (Quotation) - Position size in lots
  - `averagePositionPrice` (MoneyValue) - Average entry price
  - `averagePositionPriceFifo` (MoneyValue) - FIFO average price
  - `currentPrice` (MoneyValue) - Current market price
  - `currentNkd` (MoneyValue) - Current accrued interest (bonds)
  - `expectedYield` (Quotation) - Position P&L
  - `varMargin` (MoneyValue) - Variation margin (futures)

### GetPositions

**Endpoint**: `OperationsService/GetPositions`

**Response**: `PositionsResponse`
```json
{
  "money": [
    {
      "currency": "RUB",
      "balance": 25000.50,
      "blocked": 5000.00
    },
    {
      "currency": "USD",
      "balance": 1500.00,
      "blocked": 0
    }
  ],
  "blocked": [
    {
      "figi": "BBG004730N88",
      "blocked": 50,
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    }
  ],
  "securities": [
    {
      "figi": "BBG004730N88",
      "blocked": 50,
      "balance": 100,
      "positionUid": "a1e6df49-5ff7-4f7a-b3a6-89f56f19cfb4",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297",
      "exchangeBlocked": false,
      "instrumentType": "share"
    }
  ],
  "futures": [],
  "options": []
}
```

**Fields**:
- `money[]` - Cash balances
  - `currency` (string) - Currency code
  - `balance` (double) - Available balance
  - `blocked` (double) - Blocked in orders
- `securities[]` - Security positions
  - `figi` (string)
  - `balance` (int64) - Total balance (lots)
  - `blocked` (int64) - Blocked in orders (lots)
  - `positionUid` (string)
  - `instrumentUid` (string)
  - `exchangeBlocked` (bool) - Exchange-level block
  - `instrumentType` (string)

### GetOperations

**Endpoint**: `OperationsService/GetOperations`

**Response**: `OperationsResponse`
```json
{
  "operations": [
    {
      "id": "1234567890",
      "parentOperationId": "",
      "currency": "RUB",
      "payment": {
        "currency": "RUB",
        "units": -15025,
        "nano": 0
      },
      "price": {
        "units": 150,
        "nano": 250000000
      },
      "state": "OPERATION_STATE_EXECUTED",
      "quantity": 100,
      "quantityRest": 0,
      "figi": "BBG004730N88",
      "instrumentType": "share",
      "date": "2026-01-26T10:30:45Z",
      "type": "Покупка ЦБ",
      "operationType": "OPERATION_TYPE_BUY",
      "trades": [
        {
          "tradeId": "11223344",
          "dateTime": "2026-01-26T10:30:45.123Z",
          "quantity": 100,
          "price": {
            "units": 150,
            "nano": 250000000
          }
        }
      ],
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    },
    {
      "id": "1234567891",
      "parentOperationId": "1234567890",
      "currency": "RUB",
      "payment": {
        "currency": "RUB",
        "units": -5,
        "nano": 0
      },
      "state": "OPERATION_STATE_EXECUTED",
      "figi": "BBG004730N88",
      "date": "2026-01-26T10:30:45Z",
      "type": "Комиссия брокера",
      "operationType": "OPERATION_TYPE_BROKER_FEE"
    }
  ]
}
```

**OperationState enum**:
- `OPERATION_STATE_UNSPECIFIED` (0)
- `OPERATION_STATE_EXECUTED` (1) - Executed
- `OPERATION_STATE_CANCELED` (2) - Cancelled

**OperationType enum** (partial list):
- `OPERATION_TYPE_BUY` - Buy order
- `OPERATION_TYPE_SELL` - Sell order
- `OPERATION_TYPE_BROKER_FEE` - Broker commission
- `OPERATION_TYPE_DIVIDEND` - Dividend payment
- `OPERATION_TYPE_COUPON` - Coupon payment
- `OPERATION_TYPE_INPUT` - Money deposit
- `OPERATION_TYPE_OUTPUT` - Money withdrawal
- And many more...

## Orders Service

### PostOrder

**Endpoint**: `OrdersService/PostOrder`

**Response**: `PostOrderResponse`
```json
{
  "orderId": "12345678",
  "executionReportStatus": "EXECUTION_REPORT_STATUS_NEW",
  "lotsRequested": 10,
  "lotsExecuted": 0,
  "initialOrderPrice": {
    "currency": "RUB",
    "units": 150,
    "nano": 250000000
  },
  "executedOrderPrice": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "totalOrderAmount": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "initialCommission": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "executedCommission": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "aciValue": {
    "currency": "RUB",
    "units": 0,
    "nano": 0
  },
  "figi": "BBG004730N88",
  "direction": "ORDER_DIRECTION_BUY",
  "initialSecurityPrice": {
    "currency": "RUB",
    "units": 150,
    "nano": 250000000
  },
  "orderType": "ORDER_TYPE_LIMIT",
  "message": "",
  "initialOrderPricePt": {
    "units": 0,
    "nano": 0
  },
  "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
}
```

### GetOrders

**Endpoint**: `OrdersService/GetOrders`

**Response**: `GetOrdersResponse`
```json
{
  "orders": [
    {
      "orderId": "12345678",
      "executionReportStatus": "EXECUTION_REPORT_STATUS_NEW",
      "lotsRequested": 10,
      "lotsExecuted": 0,
      "initialOrderPrice": {
        "currency": "RUB",
        "units": 150,
        "nano": 250000000
      },
      "executedOrderPrice": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "totalOrderAmount": {
        "currency": "RUB",
        "units": 15025,
        "nano": 0
      },
      "averagePositionPrice": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "initialCommission": {
        "currency": "RUB",
        "units": 5,
        "nano": 0
      },
      "executedCommission": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "figi": "BBG004730N88",
      "direction": "ORDER_DIRECTION_BUY",
      "initialSecurityPrice": {
        "currency": "RUB",
        "units": 150,
        "nano": 250000000
      },
      "stages": [],
      "serviceCommission": {
        "currency": "RUB",
        "units": 0,
        "nano": 0
      },
      "currency": "RUB",
      "orderType": "ORDER_TYPE_LIMIT",
      "orderDate": "2026-01-26T10:30:00Z",
      "instrumentUid": "6afa6f80-03a7-4d83-9cf0-c19d7d366297"
    }
  ]
}
```

## Users Service

### GetAccounts

**Endpoint**: `UsersService/GetAccounts`

**Response**: `GetAccountsResponse`
```json
{
  "accounts": [
    {
      "id": "2000123456",
      "type": "ACCOUNT_TYPE_TINKOFF",
      "name": "Основной счет",
      "status": "ACCOUNT_STATUS_OPEN",
      "openedDate": "2020-05-15T00:00:00Z",
      "closedDate": "0001-01-01T00:00:00Z",
      "accessLevel": "ACCOUNT_ACCESS_LEVEL_FULL_ACCESS"
    },
    {
      "id": "2000123457",
      "type": "ACCOUNT_TYPE_TINKOFF_IIS",
      "name": "ИИС",
      "status": "ACCOUNT_STATUS_OPEN",
      "openedDate": "2021-03-10T00:00:00Z",
      "closedDate": "0001-01-01T00:00:00Z",
      "accessLevel": "ACCOUNT_ACCESS_LEVEL_FULL_ACCESS"
    }
  ]
}
```

**AccountType enum**:
- `ACCOUNT_TYPE_TINKOFF` (1) - Standard brokerage
- `ACCOUNT_TYPE_TINKOFF_IIS` (2) - Individual Investment Account
- `ACCOUNT_TYPE_INVEST_BOX` (3) - Invest Box

**AccountStatus enum**:
- `ACCOUNT_STATUS_OPEN` (1) - Active
- `ACCOUNT_STATUS_CLOSED` (2) - Closed

## Error Responses

### gRPC Error Format

```json
{
  "code": 30052,
  "message": "Instrument forbidden for trading by API",
  "details": [
    {
      "@type": "type.googleapis.com/tinkoff.public.invest.api.contract.v1.ErrorDetail",
      "code": "30052",
      "message": "Instrument forbidden for trading by API"
    }
  ]
}
```

### Common Error Codes (see authentication.md for full list)

- `40003` - Invalid/expired token
- `30052` - Instrument forbidden for API trading
- `50002` - Instrument not found
- `80002` - Rate limit exceeded
- `90003` - Order value too high

## Proto Field Naming

### JSON Format Options

**Standard (camelCase)**:
```json
{
  "orderId": "123",
  "executionReportStatus": "NEW"
}
```

**Proto-compatible (snake_case)**:
```json
{
  "order_id": "123",
  "execution_report_status": "NEW"
}
```

Both formats are accepted in requests and may appear in responses depending on client configuration.

## Summary

- All responses use Protocol Buffers schema
- Prices: Quotation type with `units` and `nano` fields
- Money: MoneyValue with `currency`, `units`, `nano`
- Timestamps: ISO 8601 UTC strings in JSON
- Enums: String names in JSON, integer values in proto
- Arrays: Standard JSON arrays
- Nested objects: Standard JSON objects
- Null values: Omitted fields in proto3 (not present in JSON if default value)

For complete proto definitions, see: https://github.com/Tinkoff/investAPI/tree/main/src/docs/contracts
