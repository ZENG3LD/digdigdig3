# JQuants - Complete Endpoint Reference

## Category: Authentication

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| POST | /v1/token/auth_user | Get refresh token | Yes | No | N/A | Email + password |
| POST | /v1/token/auth_refresh | Get ID token | Yes | Refresh token | N/A | Returns 24h token |

## Category: Stock Price Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/prices/daily_quotes | Daily OHLC prices | Yes (12w delay) | Yes | 5-500/min | All listed stocks |
| GET | /v1/prices/morning_quotes | Morning session prices | No (Premium) | Yes | 5-500/min | Premium only |
| GET | /v2/prices/bars/minute | Minute bars | Add-on | Yes | 60/min | Add-on plan required |
| GET | /v2/prices/ticks | Tick data | Add-on | Yes | 60/min | Add-on plan required |

## Category: Listed Issues / Symbols

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/listed/info | Listed issue master | Yes (12w delay) | Yes | 5-500/min | Symbols, sectors, markets |

## Category: Indices

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/indices | TOPIX OHLC prices | Paid (Standard+) | Yes | 5-500/min | Multiple indices |
| GET | /v1/indices/topix | TOPIX specific data | Paid (Standard+) | Yes | 5-500/min | TOPIX only |

## Category: Derivatives (Futures & Options)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/derivatives/futures | Futures OHLC | Paid (Premium) | Yes | 5-500/min | TOPIX, Nikkei 225 futures |
| GET | /v2/derivatives/bars/daily/futures | Futures daily bars | Paid (Premium) | Yes | 5-500/min | V2 endpoint |
| GET | /v1/derivatives/options | Options prices | Paid (Premium) | Yes | 5-500/min | Index options |
| GET | /v2/derivatives/bars/daily/options | Options daily bars | Paid (Premium) | Yes | 5-500/min | V2 endpoint |

## Category: Financial Data (Fundamentals)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/fins/statements | Financial statements | Yes (12w delay) | Yes | 5-500/min | BS, PL, CF |
| GET | /v1/fins/dividend | Cash dividends | Paid (Premium) | Yes | 5-500/min | Dividend announcements |
| GET | /v1/fins/announcement | Earnings calendar | Yes | Yes | 5-500/min | Next day announcements |

## Category: Market Trading Data

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/markets/trading_by_type | Trading by investor type | Paid (Standard+) | Yes | 5-500/min | Weekly updates |
| GET | /v1/markets/short_selling | Short sale value & ratio | Paid (Standard+) | Yes | 5-500/min | By sector |
| GET | /v1/markets/breakdown | Detail breakdown trading | Paid (Premium) | Yes | 5-500/min | Premium only |
| GET | /v1/markets/margin | Margin trading outstanding | Paid (Standard+) | Yes | 5-500/min | Weekly updates |
| GET | /v1/markets/trading_calendar | Trading calendar | Yes (12w delay) | Yes | 5-500/min | Holidays, business days |

## Category: Options-Specific (Index Options)

| Method | Endpoint | Description | Free? | Auth? | Rate Limit | Notes |
|--------|----------|-------------|-------|-------|------------|-------|
| GET | /v1/option/index_option | Index option prices | Paid (Standard+) | Yes | 5-500/min | TOPIX, Nikkei options |

## Parameters Reference

### GET /v1/prices/daily_quotes
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| code | string | Either code or date | - | Stock code (4 or 5 digits, e.g., "27800" or "2780") |
| date | string | Either code or date | - | Specific date (YYYYMMDD or YYYY-MM-DD) |
| from | string | No | - | Period start date (YYYYMMDD or YYYY-MM-DD) |
| to | string | No | - | Period end date (YYYYMMDD or YYYY-MM-DD) |
| pagination_key | string | No | - | For retrieving subsequent pages |

**Response Fields:**
- Date: Trading date (YYYY-MM-DD)
- Code: Stock code
- Open, High, Low, Close: Prices (Number)
- Volume: Trading volume (Number)
- TurnoverValue: Trading value (Number)
- AdjustmentFactor: Split/dividend adjustment (Number)
- AdjustmentOpen, AdjustmentHigh, AdjustmentLow, AdjustmentClose: Adjusted prices
- MorningOpen, MorningHigh, MorningLow, MorningClose: Morning session (Premium only)
- AfternoonOpen, AfternoonHigh, AfternoonLow, AfternoonClose: Afternoon session (Premium only)

### GET /v1/listed/info
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| code | string | No | - | Stock code (4 or 5 digits) |
| date | string | No | - | Target date (YYYYMMDD or YYYY-MM-DD) |

**Response Fields:**
- Date: Information application date
- Code: Security code
- CompanyName: Japanese company name
- CompanyNameEnglish: English company name
- Sector17Code, Sector17CodeName: 17-sector classification
- Sector33Code, Sector33CodeName: 33-sector classification
- ScaleCategory: TOPIX size (Core30, Large70, Mid400, Small, etc.)
- MarketCode, MarketCodeName: Market segment (Prime, Standard, Growth)
- MarginCode, MarginCodeName: Margin trading classification (Standard/Premium only)

### GET /v1/indices
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| code | string | Either code or date | - | Index code (e.g., "0000" for TOPIX) |
| date | string | Either code or date | - | Target date (YYYYMMDD or YYYY-MM-DD) |
| from | string | No | - | Period start date |
| to | string | No | - | Period end date |
| pagination_key | string | No | - | For pagination |

**Response Fields:**
- Date: Trading date
- Code: Index code
- Open, High, Low, Close: Index values (Number)

### GET /v1/derivatives/futures
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| date | string | Yes | - | Trading date (YYYYMMDD or YYYY-MM-DD) |
| category | string | No | - | Derivative product category |
| code | string | No | - | Security options code (required if category specified) |
| contract_flag | string | No | - | Central contract month filter |
| pagination_key | string | No | - | For pagination |

**Response Fields:**
- Code: Issue code
- Date: Trading date
- WholeDayOpen, WholeDayHigh, WholeDayLow, WholeDayClose: Full day OHLC
- DaySessionOpen, DaySessionHigh, DaySessionLow, DaySessionClose: Day session OHLC
- NightSessionOpen, NightSessionHigh, NightSessionLow, NightSessionClose: Night session OHLC
- SettlementPrice: Official settlement price
- TheoreticalPrice: Theoretical value
- StrikePrice: Exercise price (for options)
- ImpliedVolatility: IV (for options)
- Volume: Trading volume
- OpenInterest: Outstanding contracts

### GET /v1/fins/statements
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| code | string | Either code or date | - | Stock code |
| date | string | Either code or date | - | Disclosure date (YYYYMMDD or YYYY-MM-DD) |
| pagination_key | string | No | - | For pagination |

**Response Fields (Japanese GAAP):**
- DisclosedDate, DisclosedTime: Disclosure timestamp
- Code: Stock code
- FiscalYear, FiscalQuarter: Financial period
- NetSales, OperatingProfit, OrdinaryProfit, Profit: Income statement
- EarningsPerShare, DilutedEarningsPerShare: EPS data
- TotalAssets, Equity, EquityToAssetRatio: Balance sheet
- BookValuePerShare: Book value
- CashFlowsFromOperatingActivities, CashFlowsFromInvestingActivities, CashFlowsFromFinancingActivities: Cash flows
- CashAndEquivalents: Cash position
- ResultDividendPerShare1stQuarter through ResultDividendPerShareAnnual: Dividend results
- ForecastDividendPerShare1stQuarter through ForecastDividendPerShareAnnual: Dividend forecasts

### GET /v1/fins/dividend
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| code | string | Either code or date | - | Stock code |
| date | string | Either code or date | - | Announcement date (YYYYMMDD or YYYY-MM-DD) |
| from | string | No | - | Period start |
| to | string | No | - | Period end |
| pagination_key | string | No | - | For pagination |

**Response Fields:**
- AnnouncementDate, AnnouncementTime: Announcement timestamp
- Code: Stock code
- StatusCode: 1=new, 2=revised, 3=delete
- InterimFinalCode: 1=interim, 2=final
- ForecastResultCode: 1=result, 2=forecast
- CommemorativeSpecialCode: 0=normal, 1=commemorative, 2=special, 3=both
- GrossDividendRate: Dividend per share
- DistributionAmount: Total distribution
- CommemorativeDividendRate, SpecialDividendRate: Special dividends
- RecordDate, ExDate, PayableDate: Key dates

### GET /v1/fins/announcement
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| pagination_key | string | No | - | For pagination |

**Response Fields:**
- Date: Announcement date (empty if undecided)
- Code: Stock code
- CompanyName: Company name (Japanese)
- FiscalYear: Fiscal year end
- SectorName: Industry sector (Japanese)
- FiscalQuarter: Quarter (Japanese)
- Section: Market segment (Japanese)

### GET /v1/markets/short_selling
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| date | string | At least one required | - | Single date (YYYYMMDD or YYYY-MM-DD) |
| sector33code | string | At least one required | - | 33-sector code ("0050" or "50") |
| from | string | No | - | Period start |
| to | string | No | - | Period end |
| pagination_key | string | No | - | For pagination |

**Response Fields:**
- Date: Trading date
- Sector33Code: Industry sector
- SellingExcludingShortSellingTurnoverValue: Long selling value (yen)
- ShortSellingWithRestrictionsTurnoverValue: Restricted short sales (yen)
- ShortSellingWithoutRestrictionsTurnoverValue: Unrestricted short sales (yen)

### GET /v1/markets/trading_calendar
**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| holidaydivision | string | Either this or from/to | - | Holiday category |
| from | string | Either this or holidaydivision | - | Start date (YYYYMMDD or YYYY-MM-DD) |
| to | string | Either this or holidaydivision | - | End date (YYYYMMDD or YYYY-MM-DD) |

**Response Fields:**
- Date: Calendar date
- HolidayDivision: Holiday classification

## Authentication Headers

All authenticated endpoints require:
```
Authorization: Bearer {idToken}
```

Where `idToken` is obtained from the `/v1/token/auth_refresh` endpoint using a refresh token.

## Pagination

Endpoints returning large datasets include `pagination_key` in the response. To retrieve subsequent pages, include this key in the next request.

## Error Responses

All endpoints may return:
- **200**: Success
- **400**: Bad Request (invalid parameters)
- **401**: Unauthorized (invalid/expired token)
- **403**: Forbidden (insufficient permissions/plan tier)
- **413**: Payload Too Large (response exceeds size limits)
- **429**: Too Many Requests (rate limit exceeded)
- **500**: Internal Server Error

## Rate Limit Scope

- Per API key
- Rate limits vary by subscription tier (5/min Free to 500/min Premium)
- Add-on APIs (minute/tick) have separate 60/min limit regardless of base plan
- Burst handling: Exceeding limits triggers HTTP 429; persistent violations result in ~5min complete block

## V1 vs V2 Endpoints

- V1 endpoints: `/v1/*` (legacy, being deprecated)
- V2 endpoints: `/v2/*` (current, recommended)
- Authentication changed: V2 uses API key-based auth (simpler than V1 token flow)
- Migrate to V2 as V1 will be discontinued

## CSV Bulk Downloads

As of January 2026, CSV bulk download available for historical data:
- Access method: TBD (likely through user portal)
- Available for: Historical stock prices and other datasets
- Complements API for bulk historical retrieval
