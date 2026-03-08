# JQuants - Response Formats

All responses from JQuants API are in **JSON format**.

## Common Response Structure

Most endpoints return:
```json
{
  "data_key": [ /* array of objects */ ],
  "pagination_key": "optional_string_for_next_page"
}
```

## Authentication Endpoints

### POST /v1/token/auth_user (Get Refresh Token)

**Request:**
```json
{
  "mailaddress": "user@example.com",
  "password": "your_password"
}
```

**Response (200 OK):**
```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
}
```

**Response (400 Bad Request):**
```json
{
  "error": "'mailaddress' or 'password' is incorrect."
}
```

**Response (500 Internal Server Error):**
```json
{
  "error": "Unexpected error. Please try again later."
}
```

### POST /v1/token/auth_refresh (Get ID Token)

**Request:** (Query parameter)
```
POST /v1/token/auth_refresh?refreshtoken=YOUR_REFRESH_TOKEN
```

**Response (200 OK):**
```json
{
  "idToken": "eyJraWQiOiJhYmNkZWYxMjM0NTY3ODkwIiwiYWxnIjoiUlMyNTYifQ.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
}
```

**Response (400 Bad Request):**
```json
{
  "error": "'refreshtoken' is required."
}
```

## Stock Price Data

### GET /v1/prices/daily_quotes (Daily OHLC)

**Response (200 OK):**
```json
{
  "daily_quotes": [
    {
      "Date": "2024-01-15",
      "Code": "7203",
      "Open": 2500.0,
      "High": 2550.0,
      "Low": 2480.0,
      "Close": 2530.0,
      "Volume": 12345678,
      "TurnoverValue": 31234567890,
      "AdjustmentFactor": 1.0,
      "AdjustmentOpen": 2500.0,
      "AdjustmentHigh": 2550.0,
      "AdjustmentLow": 2480.0,
      "AdjustmentClose": 2530.0,
      "MorningOpen": 2495.0,
      "MorningHigh": 2520.0,
      "MorningLow": 2490.0,
      "MorningClose": 2510.0,
      "AfternoonOpen": 2512.0,
      "AfternoonHigh": 2550.0,
      "AfternoonLow": 2480.0,
      "AfternoonClose": 2530.0
    },
    {
      "Date": "2024-01-16",
      "Code": "7203",
      "Open": 2535.0,
      "High": 2600.0,
      "Low": 2530.0,
      "Close": 2580.0,
      "Volume": 15234567,
      "TurnoverValue": 39123456789,
      "AdjustmentFactor": 1.0,
      "AdjustmentOpen": 2535.0,
      "AdjustmentHigh": 2600.0,
      "AdjustmentLow": 2530.0,
      "AdjustmentClose": 2580.0
    }
  ],
  "pagination_key": "next_page_token_if_more_data"
}
```

**Field Details:**
- `Date` (String): Trading date in YYYY-MM-DD format
- `Code` (String): Stock code (4 or 5 digits)
- `Open`, `High`, `Low`, `Close` (Number): Raw prices in JPY
- `Volume` (Number): Number of shares traded
- `TurnoverValue` (Number): Total trading value in JPY
- `AdjustmentFactor` (Number): Cumulative adjustment for splits/dividends
- `AdjustmentOpen/High/Low/Close` (Number): Adjusted prices
- `Morning*` fields (Number): Morning session OHLC (Premium plan only)
- `Afternoon*` fields (Number): Afternoon session OHLC (Premium plan only)

**Note**: Morning/Afternoon fields only present for Premium plan users.

## Listed Issue Data

### GET /v1/listed/info (Listed Issue Master)

**Response (200 OK):**
```json
{
  "info": [
    {
      "Date": "2024-01-15",
      "Code": "7203",
      "CompanyName": "トヨタ自動車株式会社",
      "CompanyNameEnglish": "Toyota Motor Corporation",
      "Sector17Code": "10",
      "Sector17CodeName": "自動車",
      "Sector33Code": "3350",
      "Sector33CodeName": "自動車",
      "ScaleCategory": "TOPIX Core30",
      "MarketCode": "0111",
      "MarketCodeName": "プライム",
      "MarginCode": "1",
      "MarginCodeName": "制度信用銘柄"
    },
    {
      "Date": "2024-01-15",
      "Code": "6758",
      "CompanyName": "ソニーグループ株式会社",
      "CompanyNameEnglish": "Sony Group Corporation",
      "Sector17Code": "9",
      "Sector17CodeName": "電機・精密",
      "Sector33Code": "3650",
      "Sector33CodeName": "電気機器",
      "ScaleCategory": "TOPIX Core30",
      "MarketCode": "0111",
      "MarketCodeName": "プライム",
      "MarginCode": "1",
      "MarginCodeName": "制度信用銘柄"
    }
  ]
}
```

**Field Details:**
- `Date` (String): Information application date (YYYY-MM-DD)
- `Code` (String): Stock code
- `CompanyName` (String): Japanese company name
- `CompanyNameEnglish` (String): English company name
- `Sector17Code` (String): 17-sector classification code
- `Sector17CodeName` (String): 17-sector name (Japanese)
- `Sector33Code` (String): 33-sector classification code
- `Sector33CodeName` (String): 33-sector name (Japanese)
- `ScaleCategory` (String): TOPIX scale (e.g., "TOPIX Core30", "TOPIX Large70")
- `MarketCode` (String): Market segment code
- `MarketCodeName` (String): Market name (e.g., "プライム" = Prime)
- `MarginCode` (String): Margin trading classification (Standard/Premium only)
- `MarginCodeName` (String): Margin classification name (Standard/Premium only)

## Indices Data

### GET /v1/indices (TOPIX OHLC)

**Response (200 OK):**
```json
{
  "indices": [
    {
      "Date": "2024-01-15",
      "Code": "0000",
      "Open": 2650.50,
      "High": 2680.25,
      "Low": 2645.00,
      "Close": 2670.75
    },
    {
      "Date": "2024-01-15",
      "Code": "0028",
      "Open": 1250.30,
      "High": 1265.80,
      "Low": 1248.50,
      "Close": 1260.20
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `Date` (String): Trading date (YYYY-MM-DD)
- `Code` (String): Index code (e.g., "0000" = TOPIX)
- `Open`, `High`, `Low`, `Close` (Number): Index values

**Common Index Codes:**
- `0000`: TOPIX (Tokyo Stock Price Index)
- `0028`: Growth Market 250 Index (formerly Mothers Index)

## Derivatives Data

### GET /v1/derivatives/futures (Futures OHLC)

**Response (200 OK):**
```json
{
  "futures": [
    {
      "Code": "167060018",
      "ProdCat": "TOPIXF",
      "Date": "2024-01-15",
      "WholeDayOpen": 2650.0,
      "WholeDayHigh": 2680.0,
      "WholeDayLow": 2645.0,
      "WholeDayClose": 2670.0,
      "DaySessionOpen": 2650.0,
      "DaySessionHigh": 2675.0,
      "DaySessionLow": 2645.0,
      "DaySessionClose": 2665.0,
      "NightSessionOpen": "2666.0",
      "NightSessionHigh": "2680.0",
      "NightSessionLow": "2660.0",
      "NightSessionClose": "2670.0",
      "Volume": 123456,
      "OpenInterest": 987654,
      "SettlementPrice": 2668.5,
      "TheoreticalPrice": 2669.0
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `Code` (String): Futures contract code
- `ProdCat` (String): Product category (e.g., "TOPIXF", "NK225F")
- `Date` (String): Trading date (YYYY-MM-DD)
- `WholeDayOpen/High/Low/Close` (Number): Full day OHLC
- `DaySessionOpen/High/Low/Close` (Number): Daytime session OHLC
- `NightSessionOpen/High/Low/Close` (Number/String): Night session OHLC
- `Volume` (Number): Trading volume (contracts)
- `OpenInterest` (Number): Outstanding contracts
- `SettlementPrice` (Number): Official settlement price
- `TheoreticalPrice` (Number): Calculated theoretical value

### GET /v1/derivatives/options (Options OHLC)

**Response (200 OK):**
```json
{
  "options": [
    {
      "Code": "130060018",
      "Date": "2024-01-15",
      "WholeDayOpen": 125.5,
      "WholeDayHigh": 130.0,
      "WholeDayLow": 123.0,
      "WholeDayClose": 128.0,
      "DaySessionOpen": 125.5,
      "DaySessionHigh": 129.0,
      "DaySessionLow": 123.0,
      "DaySessionClose": 127.5,
      "NightSessionOpen": "127.8",
      "NightSessionHigh": "130.0",
      "NightSessionLow": "126.0",
      "NightSessionClose": "128.0",
      "Volume": 5432,
      "OpenInterest": 12345,
      "SettlementPrice": 127.8,
      "TheoreticalPrice": 128.2,
      "StrikePrice": 2650.0,
      "ImpliedVolatility": 18.5
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- Similar to futures, plus:
- `StrikePrice` (Number): Exercise price
- `ImpliedVolatility` (Number): Implied volatility (percentage)

## Financial Data

### GET /v1/fins/statements (Financial Statements)

**Response (200 OK):**
```json
{
  "statements": [
    {
      "DisclosedDate": "2024-01-15",
      "DisclosedTime": "15:00:00",
      "Code": "7203",
      "FiscalYear": "2024-03-31",
      "FiscalQuarter": "Q3",
      "TypeOfDocument": "1Q",
      "NetSales": 9000000000000,
      "OperatingProfit": 800000000000,
      "OrdinaryProfit": 850000000000,
      "Profit": 600000000000,
      "EarningsPerShare": 420.50,
      "DilutedEarningsPerShare": 419.80,
      "TotalAssets": 50000000000000,
      "Equity": 25000000000000,
      "EquityToAssetRatio": 50.0,
      "BookValuePerShare": 15000.00,
      "CashAndEquivalents": 5000000000000,
      "CashFlowsFromOperatingActivities": 1200000000000,
      "CashFlowsFromInvestingActivities": -800000000000,
      "CashFlowsFromFinancingActivities": -300000000000,
      "ResultDividendPerShare1stQuarter": 0,
      "ResultDividendPerShare2ndQuarter": 30.0,
      "ResultDividendPerShare3rdQuarter": 0,
      "ResultDividendPerShareFiscalYearEnd": 35.0,
      "ResultDividendPerShareAnnual": 65.0,
      "ForecastDividendPerShare1stQuarter": 0,
      "ForecastDividendPerShare2ndQuarter": 30.0,
      "ForecastDividendPerShare3rdQuarter": 0,
      "ForecastDividendPerShareFiscalYearEnd": 35.0,
      "ForecastDividendPerShareAnnual": 65.0
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `DisclosedDate` (String): Disclosure date (YYYY-MM-DD)
- `DisclosedTime` (String): Disclosure time (HH:MM:SS)
- `Code` (String): Stock code
- `FiscalYear` (String): Fiscal year end (YYYY-MM-DD)
- `FiscalQuarter` (String): Quarter (e.g., "Q3")
- `TypeOfDocument` (String): Document type code
- **Income Statement:**
  - `NetSales` (Number): Revenue in JPY
  - `OperatingProfit` (Number): Operating income in JPY
  - `OrdinaryProfit` (Number): Pre-tax profit in JPY
  - `Profit` (Number): Net income in JPY
  - `EarningsPerShare` (Number): Basic EPS
  - `DilutedEarningsPerShare` (Number): Diluted EPS
- **Balance Sheet:**
  - `TotalAssets` (Number): Total assets in JPY
  - `Equity` (Number): Shareholders' equity in JPY
  - `EquityToAssetRatio` (Number): Equity ratio (percentage)
  - `BookValuePerShare` (Number): Book value per share
  - `CashAndEquivalents` (Number): Cash position in JPY
- **Cash Flow:**
  - `CashFlowsFromOperatingActivities` (Number): Operating CF in JPY
  - `CashFlowsFromInvestingActivities` (Number): Investing CF in JPY
  - `CashFlowsFromFinancingActivities` (Number): Financing CF in JPY
- **Dividends:**
  - `ResultDividendPerShare*` (Number): Actual dividends per share
  - `ForecastDividendPerShare*` (Number): Forecast dividends per share

### GET /v1/fins/dividend (Cash Dividends)

**Response (200 OK):**
```json
{
  "dividend": [
    {
      "AnnouncementDate": "2024-01-15",
      "AnnouncementTime": "15:00:00",
      "Code": "7203",
      "ReferenceNumber": "12345",
      "CAReferenceNumber": "67890",
      "StatusCode": "1",
      "BoardMeetingDate": "2024-01-14",
      "InterimFinalCode": "2",
      "ForecastResultCode": "1",
      "CommemorativeSpecialCode": "0",
      "GrossDividendRate": 35.0,
      "DistributionAmount": 52500000000,
      "CommemorativeDividendRate": 0,
      "SpecialDividendRate": 0,
      "RetainedEarnings": 45000000000,
      "DeemedDividend": 0,
      "DeemedCapitalGains": 0,
      "RecordDate": "2024-03-31",
      "ExDate": "2024-03-29",
      "ActualRecordDate": "2024-03-31",
      "PayableDate": "2024-06-15",
      "InterimFinalTerm": "2024-03-31",
      "NetAssetDecreaseRatio": 0
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `AnnouncementDate`, `AnnouncementTime` (String): When announced
- `Code` (String): Stock code
- `StatusCode` (String): 1=new, 2=revised, 3=delete
- `InterimFinalCode` (String): 1=interim, 2=final
- `ForecastResultCode` (String): 1=result, 2=forecast
- `CommemorativeSpecialCode` (String): 0=normal, 1=commemorative, 2=special, 3=both
- `GrossDividendRate` (Number): Dividend per share in JPY
- `DistributionAmount` (Number): Total distribution in JPY
- `RecordDate`, `ExDate`, `PayableDate` (String): Key dates (YYYY-MM-DD)

### GET /v1/fins/announcement (Earnings Calendar)

**Response (200 OK):**
```json
{
  "announcement": [
    {
      "Date": "2024-01-16",
      "Code": "7203",
      "CompanyName": "トヨタ自動車株式会社",
      "FiscalYear": "2024-03-31",
      "SectorName": "自動車",
      "FiscalQuarter": "第3四半期",
      "Section": "プライム"
    },
    {
      "Date": "",
      "Code": "6758",
      "CompanyName": "ソニーグループ株式会社",
      "FiscalYear": "2024-03-31",
      "SectorName": "電気機器",
      "FiscalQuarter": "第3四半期",
      "Section": "プライム"
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `Date` (String): Announcement date (YYYY-MM-DD), empty if undecided
- `Code` (String): Stock code
- `CompanyName` (String): Company name in Japanese
- `FiscalYear` (String): Fiscal year end date (YYYY-MM-DD)
- `SectorName` (String): Sector in Japanese
- `FiscalQuarter` (String): Quarter in Japanese
- `Section` (String): Market segment in Japanese

## Market Data

### GET /v1/markets/short_selling (Short Selling Data)

**Response (200 OK):**
```json
{
  "short_selling": [
    {
      "Date": "2024-01-15",
      "Sector33Code": "0050",
      "SellingExcludingShortSellingTurnoverValue": 123456789012,
      "ShortSellingWithRestrictionsTurnoverValue": 12345678901,
      "ShortSellingWithoutRestrictionsTurnoverValue": 23456789012
    },
    {
      "Date": "2024-01-15",
      "Sector33Code": "3350",
      "SellingExcludingShortSellingTurnoverValue": 234567890123,
      "ShortSellingWithRestrictionsTurnoverValue": 23456789012,
      "ShortSellingWithoutRestrictionsTurnoverValue": 34567890123
    }
  ],
  "pagination_key": null
}
```

**Field Details:**
- `Date` (String): Trading date (YYYY-MM-DD)
- `Sector33Code` (String): 33-sector classification code
- `SellingExcludingShortSellingTurnoverValue` (Number): Long selling value in JPY
- `ShortSellingWithRestrictionsTurnoverValue` (Number): Restricted short sales in JPY
- `ShortSellingWithoutRestrictionsTurnoverValue` (Number): Unrestricted short sales in JPY

**Note**: Values in JPY (not rounded to millions like web display)

### GET /v1/markets/trading_calendar (Trading Calendar)

**Response (200 OK):**
```json
{
  "trading_calendar": [
    {
      "Date": "2024-01-01",
      "HolidayDivision": "休日"
    },
    {
      "Date": "2024-01-02",
      "HolidayDivision": "休日"
    },
    {
      "Date": "2024-01-03",
      "HolidayDivision": "休日"
    },
    {
      "Date": "2024-01-04",
      "HolidayDivision": "営業日"
    }
  ]
}
```

**Field Details:**
- `Date` (String): Calendar date (YYYY-MM-DD)
- `HolidayDivision` (String): Holiday classification (Japanese)
  - "営業日" = Business day
  - "休日" = Holiday

## Error Responses

### 400 Bad Request
```json
{
  "error": "Bad Request",
  "message": "Invalid parameter: 'code' or 'date' must be specified"
}
```

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token"
}
```

### 403 Forbidden
```json
{
  "error": "Forbidden",
  "message": "Insufficient plan tier for this endpoint"
}
```

### 413 Payload Too Large
```json
{
  "error": "Request Entity Too Large",
  "message": "Response data exceeds size limits. Use pagination or narrow date range."
}
```

### 429 Too Many Requests
```json
{
  "error": "Too Many Requests",
  "message": "Rate limit exceeded. Please wait before retrying."
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal Server Error",
  "message": "Unexpected error. Please try again later."
}
```

## Pagination

For large result sets, responses include `pagination_key`:

**First Request:**
```
GET /v1/prices/daily_quotes?date=2024-01-15
```

**First Response:**
```json
{
  "daily_quotes": [ /* 1000 records */ ],
  "pagination_key": "eyJuZXh0IjoiMTAwMCJ9"
}
```

**Next Request:**
```
GET /v1/prices/daily_quotes?date=2024-01-15&pagination_key=eyJuZXh0IjoiMTAwMCJ9
```

**Next Response:**
```json
{
  "daily_quotes": [ /* next 1000 records */ ],
  "pagination_key": "eyJuZXh0IjoiMjAwMCJ9"
}
```

**Final Response (no more data):**
```json
{
  "daily_quotes": [ /* remaining records */ ],
  "pagination_key": null
}
```

## Data Type Conventions

- **Strings**: ISO 8601 dates (YYYY-MM-DD), times (HH:MM:SS), codes, names
- **Numbers**: Floating-point for prices, integers for volumes/counts
- **null**: Missing or not applicable data
- **Empty string ""**: Undecided/unknown (e.g., announcement date TBD)
- **Japanese text**: UTF-8 encoded (company names, sector names, etc.)
- **Currency**: All monetary values in JPY (Japanese Yen)

## Response Size Limits

- Maximum response size: Not documented (enforced by 413 error)
- Pagination triggered: When result set exceeds internal threshold
- Recommended: Use date ranges to limit response size
- For bulk data: Use CSV downloads instead of API

## Common Patterns

### Empty Result Set
```json
{
  "daily_quotes": [],
  "pagination_key": null
}
```

### Single Record
```json
{
  "daily_quotes": [
    { /* single record */ }
  ],
  "pagination_key": null
}
```

### Multiple Pages
- `pagination_key` present: More data available
- `pagination_key` null: Last page

## V2 API Response Differences

V2 endpoints (e.g., `/v2/derivatives/bars/daily/futures`) may have slightly different response structures. Consult V2-specific documentation for exact formats.

As of January 2026, V2 documentation is being updated. Expect similar JSON structure but potentially different field names or nesting.
